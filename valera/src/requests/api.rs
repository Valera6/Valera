use rand::{distributions::Alphanumeric, Rng};
use crate::types::*;
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::requests::Provider;

#[derive(Default)]
pub struct Args{
	start_time: Option<Timestamp>,
	end_time: Option<Timestamp>,
	params: Option<HashMap<String, String>>,
	id: Option<String>,
}
impl Args {
	pub fn new() -> Self {
		Args::default()
	}
	pub fn start_time(mut self, start_time: Timestamp) -> Self {
		self.start_time = Some(start_time);
		self
	}
	pub fn end_time(mut self, end_time: Timestamp) -> Self {
		self.end_time = Some(end_time);
		self
	}
	pub fn params(mut self, params: HashMap<String, String>) -> Self {
		self.params = Some(params);
		self
	}
	pub fn id(mut self, id: String) -> Self {
		self.id = Some(id);
		self
	}
	pub async fn collect_and_dump_trades(&self, provider: &Provider, end_url: String, symbols: Symbols) {
		collect_and_dump_trades(provider, end_url, symbols, self.start_time.clone(), self.end_time.clone(), self.params.clone(), self.id.clone()).await;
	}
}

async fn load_trades_over_interval(provider_ref: &Provider, params: HashMap<String, String>, mut base_path: PathBuf) -> Result<()> {
	let symbol = params.symbol;
	let start_time = params.start_time;
	let end_time = params.end_time;
	let id = params.id;

	//let base_url = market.get_base_url();
	//let api_key = Some(std::env::var("BINANCE_MAIN_KEY").unwrap());
	//let client = Client::build(provider_ref, api_key);

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

	let mut last_reached_ms = *start_time.get_ms();
	let mut buffer_df = DataFrame::default();
	while last_reached_ms < end_time.ms {
		// In the perfect world check the code, and never print out the same error code twice.
		let r = match client::request(url, params)?.await() {
			Error(e) => eprintln!("Request errored: {}", e),
			Result(response) => response,
		};

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

	//TODO: figure out how to gradually unload with sink_parquet
	base_path.push(id + ".parquet");
	buffer_df.lazy().sink_parquet(base_path, ParquetWriteOptions::default()).unwrap();
	Ok(())
}

fn generate_random_id() -> String {
    let random_part: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    format!("GENERATED_{}", random_part)
}

///_args_: `end_url` will be appended to the `base_url` of the provider, if any.
//TODO!!!!!!!: append optional args with functions
pub async fn collect_and_dump_trades(provider: &Provider, end_url: String, symbols: Symbols, start_time: Option<Timestamp>, end_time: Option<Timestamp>, params: Option<HashMap<String, String>>, id: Option<String>) {
	let query_id: String = match id {
		Some(provided) => provided,
		None => generate_random_id(),
	};
	let symbols = symbols.as_strings();

	let mut dump_path = PathBuf::from("ongoing_collection");
	std::fs::create_dir_all(&dump_path).unwrap();
	dump_path.push(provider.name());
	if dump_path.exists() {
		std::fs::remove_dir_all(&dump_path).unwrap();
	}
	std::fs::create_dir_all(&dump_path).unwrap();

	//let mut bar = valera_utils::ProgressBar::new(payloads.len());
	let mut i = 1_usize;
	while !payloads.is_empty() {
		let mut handles = Vec::new();
		while handles.len() < 30 {
			let to_pass_payload = payloads.pop().unwrap();
			let to_pass_market = market.clone();
			let to_pass_path = dump_path.clone();
			let to_pass_exchange = exchange.clone();
			let handle = tokio::task::spawn(async move {
				load_trades_over_interval(to_pass_exchange, to_pass_payload, to_pass_market, to_pass_path).await.unwrap();
			});
			handles.push(handle);
			i += 1;
		}
		for handle in handles {
			let _res = handle.await;
		}
		//bar.progress(i);
	}
	eprintln!("  DONE  ");
	valera_utils::tg();
}

pub async fn get_closes_df() -> DataFrame {
	let mut k = get_24hr(Providers::BinancePerp).await;

	let mut closes_init: Vec<Series> = Vec::new();
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
