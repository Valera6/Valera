use crate::types::*;
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

async fn load_trades_over_interval(exchange: Arc<Binance>, payload: TradesPayload, market: Providers, mut base_path: PathBuf) -> Result<()> {
	let symbol = payload.symbol;
	let start_time = payload.start_time;
	let end_time = payload.end_time;
	let id = payload.id;

	let base_url = market.get_base_url();
	//todo make this also be determined by Market:
	let api_key = "AZ9qXn8S1RCJWur3bfKSsbWWKB9lNOESywiFoKF8WAh2xyHFAT6euDYuuKJr2CXg";

	let find_fromId = async {
		let url = format!("{}/aggTrades?symbol={}&startTime={}&limit=1", &base_url, &symbol, &start_time.ms);
		let json = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();
		json[0]["f"].as_i64().unwrap().to_string()
	}
	.await;

	let url = [&base_url, "/historicalTrades"].concat();

	let mut params: HashMap<&str, &str> = HashMap::new();
	params.insert("symbol", symbol.as_str());
	params.insert("limit", "1000");
	params.insert("fromId", &find_fromId);

	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("X-MBX-APIKEY", api_key.parse().unwrap());
	headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

	let client = reqwest::Client::new();
	let mut last_reached_ms = *start_time.get_ms();
	let mut buffer_df = DataFrame::default();
	while last_reached_ms < end_time.ms {
		exchange.rate_limits.get("normal").unwrap().sleep_if_needed();

		let r = client.get(url.as_str()).query(&params).headers(headers.clone()).send().await.unwrap();
		// todo match statement, printing out r if it doesn't have headers. In the perfect world check the code, and never print out the same error code twice.
		let header_used = r.headers().get("x-mbx-used-weight-1m").unwrap();

		if let Ok(used_str) = header_used.to_str() {
			if let Ok(used) = used_str.parse::<i32>() {
				exchange.rate_limits.get("normal").unwrap().update(used).await;
			}
		}

		let r_json = r.json::<serde_json::Value>().await.unwrap();

		let array_of_values = r_json.as_array().unwrap().to_vec();
		let mut new_fromId = String::new();
		for v in array_of_values.iter() {
			let t = v.get("time").unwrap().as_i64().unwrap();
			if t <= end_time.ms {
				let row: DataFrame = market.trades_entry_into_row(&v);
				buffer_df.vstack_mut(&row)?;
			}
			new_fromId = (v.get("id").unwrap().as_i64().unwrap() + 1).to_string(); // because the thing is inclusive, I checked.
			last_reached_ms = v.get("time").unwrap().as_i64().unwrap();
		}
		params.insert("fromId", Box::leak(new_fromId.clone().into_boxed_str()));
	}

	//todo figure out how to gradually unload with sink_parquet
	base_path.push(id + ".parquet");
	buffer_df.lazy().sink_parquet(base_path, ParquetWriteOptions::default()).unwrap();
	Ok(())
}

/// Function that schedules the `get_trades()` in batches of 30. Later will be upgraded to be unlimited, taking advantage of streaming directly into the according files on every yield, but that I haven't figured out yet.
pub async fn collect_trades(mut payloads: Vec<TradesPayload>, market: Providers) {
	//todo assert Timeframe is a valide Binance timeframe ( currently is true by the virtue of current implementation, but that will change )

	let exchange = Arc::new(Binance::new().await);
	// the following will be done for each proxy thread of the carousel:
	exchange.rate_limits.insert("normal".to_owned(), RateLimit::new());

	use std::fs;
	let mut dump_path = PathBuf::from("ongoing_collection");
	// note that this does not overwrite already existing directories by default
	fs::create_dir_all(&dump_path).unwrap();
	dump_path.push(market.name());
	if dump_path.exists() {
		fs::remove_dir_all(&dump_path).unwrap();
	}
	fs::create_dir_all(&dump_path).unwrap();

	let mut bar = valera_utils::ProgressBar::new(payloads.len());
	let mut i = 1_usize;
	while !payloads.is_empty() {
		let mut handles = Vec::new();
		while handles.len() < 30 {
			let to_pass_payload = payloads.pop().unwrap();
			let to_pass_market = market.clone();
			let to_pass_path = dump_path.clone();
			let to_pass_exchange = exchange.clone(); // clones only the Arc reference, all still point to one thing
			let handle = tokio::task::spawn(async move {
				load_trades_over_interval(to_pass_exchange, to_pass_payload, to_pass_market, to_pass_path).await.unwrap();
			});
			handles.push(handle);
			i += 1;
		}
		for handle in handles {
			let _res = handle.await;
		}
		bar.progress(i);
	}
	eprintln!("  DONE  ");
	valera_utils::tg();
}

//? should it be here or at say transformers.rs
pub async fn get_closes_df() -> DataFrame {
	let mut k = get_24hr(Providers::BinancePerp).await;

	let mut closes_init: Vec<Series> = Vec::new();
	// probably will add shared index later, for now without it.
	for (_key, value) in k.iter_mut() {
		value.normalize(None);
		let mut closes_series = value.df.column("open").unwrap().clone();
		closes_series.rename(_key);
		closes_init.push(closes_series);
	}
	DataFrame::new(closes_init).unwrap()
}

pub async fn get_24hr(market: Providers) -> HashMap<String, Klines> {
	let b = Binance::new().await;
	//todo make be based on market
	let url = "https://fapi.binance.com/fapi/v1/klines";
	let symbols = b.get_perp();
	// let symbols = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT", "BNBUSDT", "SOLUSDT", "XRPUSDT"]; //dbg
	let mut params = std::collections::HashMap::new();
	params.insert("interval", "5m");
	let binding = Timestamp::now().subtract(24 * 60 * 60).ms.to_string();
	params.insert("startTime", &binding);
	let binding = Timestamp::now().ms.to_string(); //dbg
	params.insert("endTime", &binding);

	let map_of_vecs = b.requests(url, symbols, params).await.unwrap();

	let mut klines_map = HashMap::new();
	for (key, value) in map_of_vecs {
		klines_map.insert(key, market.convert_into_klines(value));
	}
	klines_map
}
