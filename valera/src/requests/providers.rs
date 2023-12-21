use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::requests::client::*;
use crate::requests::db_infrastructure::LogEntry;
use crate::requests::query::*;
use crate::types::*;

pub enum Templates {
	//TODO!: add the rest of the binance providers.
	BinancePerp,
	BinanceSpot,
	BinanceMargin,
	BinanceData,
	BinanceWebsocket, //?
	SomethingElseSayCoinmarketcap,
}
impl Templates {
	pub fn build(&self) -> Provider {
		match self {
			Self::BinancePerp => Provider::build(
				vec![ClientInit {
					api_key: Some(std::env::var("BINANCE_MAIN_KEY").unwrap()),
					proxy: None,
				}],
				2400, // and 1200 for orders
				Some("https://fapi.binance.com/fapi/v1"),
				Box::new(|current_used: i32, r: &reqwest::Response| -> i32 {
					let header_value = r.headers().get("x-mbx-used-weight-1m").unwrap();
					match header_value.to_str() {
						Ok(used_str) => used_str.parse::<i32>().unwrap_or(current_used),
						Err(_) => {
							eprintln!("Error: failed to extract new used from reqwest::Response");
							current_used
						}
					}
				}),
				"BinancePerp".to_owned(), // There has to be a way to automate it! GPT suggests `use strum::ToString; and then .to_string() on the member of the enum`, but it doesn't work
			),
			//Self::BinanceSpot => Provider::build(), // but want to have `rate_limit`=6000 and different `base_url`
			_ => panic!("Not implemented yet"),
		}
	}
}

pub struct Provider {
	clients: Vec<Client>,
	base_url: String,
	name: String,
}
impl Provider {
	pub fn default() -> Self {
		todo!()
	}
	pub fn build<F>(clients: Vec<ClientInit>, rate_limit: i32, base_url: Option<&str>, calc_used: Box<F>, name: String) -> Self
	where
		F: Fn(i32, &reqwest::Response) -> i32 + Clone,
	{
		let base_url = match base_url {
			Some(s) => s.to_owned(),
			None => "".to_owned(),
		};
		let clients: Vec<Client> = clients.iter().map(|&client_specific| Client::build(client_specific, rate_limit, calc_used.clone())).collect();
		Provider { clients, base_url, name }
	}
	pub fn name(&self) -> String {
		self.name.clone()
	}
	/// Simply concatenates base_url and end_url.
	/// If during the creation of `Provider` `base_url` wasn't provided, just provide full url as end_url here.
	pub fn url(&self, end_url: String) -> String {
		format!("{}{}", self.base_url.clone(), end_url)
	}
	pub fn submit(&self, queries: Vec<Query>) {
		//TODO!!: do checking of the busyness of clients (inlined!)
		//dbg
		{
			self.clients[0].queries.lock().unwrap().append(queries);
		}
		self.clients[0].try_start_more();
	}
	/// One of the API endpoints. And as such some things that are optional for the `SubQuery`, are required here.
	pub async fn collect_and_dump_trades(&self, log_entry: LogEntry) {
		// // init params for the request
		//TODO!!!!!!!!: make the Box<symbol> into a string
		let symbol: String = log_entry.symbol.inner();
		let grid_pos = QueryGridPos { x: 0, y: 0 };
		let url = self.url("/historicalTrades".to_owned());

		let (tx, mut rx) = mpsc::unbounded_channel::<Box<dyn std::any::Any + Send + Sync>>();

		let mut other_params = HashMap::new();
		other_params.insert("symbol".to_owned(), symbol); //dbg
		other_params.insert("limit".to_owned(), 1000.to_string()); //dbg

		let logic = || println!("Implement this later. For now simplest thing to get the infrastructure working");
		//

		let queries = vec![Query::build(
			tx,
			url,
			Box::new(logic),
			grid_pos,
			Some(log_entry.start_time),
			Some(log_entry.end_time),
			other_params,
			20,
		)];

		self.submit(queries);

		//dbg
		while let Some(i) = rx.recv().await {
			dbg!("here we go: {}", i);
		}

		// // Create dir to be dumping into. For now it doesn't have to be before the submition of the queries, but later this might be important for dynamic unloading with polar's `parquet_sync`.
		let mut dump_path = PathBuf::from("/tmp/ongoing_collection");
		std::fs::create_dir_all(&dump_path).unwrap();
		dump_path.push(self.name());
		if dump_path.exists() {
			std::fs::remove_dir_all(&dump_path).unwrap();
		}
		std::fs::create_dir_all(&dump_path).unwrap();
		//

		// later will be `Vec<DataFrame>`
		//BUG: not how this works.
		//dump_path.push(entry_id + ".parquet");
		//df.lazy().sink_parquet(dump_path, ParquetWriteOptions::default()).unwrap();
	}
}

//=============================================================================
// everything what's left in here is actually query-specific, and needs to be transfered to be provided on their creation
//fn trades_into_klines(array_of_values: Vec<serde_json::Value>) -> Klines {
//	// these are the values that every array returned by /klines endpoint carries:
//	//let columns = ["open_ms", "open", "high", "low", "close", "volume", "close_ms", "quote_asset_volume", "trades", "taker_buy_base", "taker_buy_quote", "ignore"];
//	// let indeces = [6, 1, 2, 3, 4, 7, 8, 10]; // these are the ones we care about
//
//	let mut close_ms: Vec<i64> = Vec::new();
//	let mut open: Vec<f64> = Vec::new();
//	let mut high: Vec<f64> = Vec::new();
//	let mut low: Vec<f64> = Vec::new();
//	let mut close: Vec<f64> = Vec::new();
//	let mut volume: Vec<f64> = Vec::new();
//	let mut trades: Vec<f64> = Vec::new();
//	let mut taker_volume: Vec<f64> = Vec::new();
//	for v in array_of_values.iter() {
//		close_ms.push(v[6].as_i64().unwrap());
//		open.push(v[1].as_str().unwrap().parse().unwrap());
//		high.push(v[2].as_str().unwrap().parse().unwrap());
//		low.push(v[3].as_str().unwrap().parse().unwrap());
//		close.push(v[4].as_str().unwrap().parse().unwrap());
//		volume.push(v[7].as_str().unwrap().parse().unwrap());
//		trades.push(v[8].as_f64().unwrap());
//		taker_volume.push(v[10].as_str().unwrap().parse().unwrap());
//	}
//
//	let df = df!(
//	"close_ms" => close_ms,
//	"open" => open,
//	"high" => high,
//	"low" => low,
//	"close" => close,
//	"volume" => volume,
//	"trades" => trades,
//	"taker_volume" => taker_volume,
//	)
//		.unwrap();
//	let k: Klines = df.try_into().unwrap();
//	k
//}
