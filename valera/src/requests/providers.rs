use anyhow::Result;
use polars::prelude::*;
use polars::prelude::{df, DataFrame, NamedFrom};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::mspc::Seder;
use std::path::PathBuf;

use crate::requests::client::*;
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
				vec![ClientSpecific {
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
	pub fn build<F>(clients: Vec<ClientSpecific>, rate_limit: i32, base_url: Option<&str>, calc_used: Box<F>, name: String) -> Self
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
	pub fn submit<T>(&self, query: SubQuery<T>) {
		//TODO!!: do checking of the busyness of clients and its implementation on client's side
		self.clients[0].assign(query)
	}
	/// One of the API endpoints. And as such some things that are optional for the `SubQuery`, are required here.
	pub async fn collect_and_dump_trades(&self, end_url: String, symbols: Symbols, start_time: Timestamp, end_time: Timestamp) {
		// // init params for the request
		let symbols = symbols.as_strings();
		let grid_pos = QueryGridPos { x: 0, y: 0 };
		let url = self.url(end_url);

		let symbol = symbols[0].clone(); //dbg
		let mut other_params = HashMap::new();
		other_params.insert("symbol".to_owned(), symbol); //dbg
		other_params.insert("limit".to_owned(), 1000.to_string()); //dbg

		let logic = || println!("Implement this later. For now simplest thing to get the infrastructure working");

		let query = SubQuery::<DataFrame>::build(url, self, logic, grid_pos, Some(start_time), Some(end_time), other_params, 20);
		//

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
		let df: DataFrame = self.submit(query);
		dump_path.push(entry_id + ".parquet");
		df.lazy().sink_parquet(dump_path, ParquetWriteOptions::default()).unwrap();
	}
}

//=============================================================================
// everything what's left in here is actually query-specific, and needs to be transfered to be provided on their creation
impl Providers {
	pub fn trades_entry_into_row(&self, entry: &serde_json::Value) -> DataFrame {
		df!(
			"time_ms" => vec![entry.get("time").unwrap().as_i64().unwrap()],
			"price" => vec![entry.get("price").unwrap().as_str().unwrap().parse::<f64>().unwrap()],
			"qty" => vec![entry.get("quoteQty").unwrap().as_str().unwrap().parse::<f64>().unwrap()],
			"isBuyerMaker" => vec![entry.get("isBuyerMaker").unwrap().as_bool().unwrap()],
		)
		.unwrap()
	}
	pub fn convert_into_klines(&self, array_of_values: Vec<serde_json::Value>) -> Klines {
		match self {
			Providers::BinancePerp | Providers::BinanceSpot => {
				// these are the values that every array returned by /klines endpoint carries:
				//let columns = ["open_ms", "open", "high", "low", "close", "volume", "close_ms", "quote_asset_volume", "trades", "taker_buy_base", "taker_buy_quote", "ignore"];
				// let indeces = [6, 1, 2, 3, 4, 7, 8, 10]; // these are the ones we care about

				let mut close_ms: Vec<i64> = Vec::new();
				let mut open: Vec<f64> = Vec::new();
				let mut high: Vec<f64> = Vec::new();
				let mut low: Vec<f64> = Vec::new();
				let mut close: Vec<f64> = Vec::new();
				let mut volume: Vec<f64> = Vec::new();
				let mut trades: Vec<f64> = Vec::new();
				let mut taker_volume: Vec<f64> = Vec::new();
				for v in array_of_values.iter() {
					close_ms.push(v[6].as_i64().unwrap());
					open.push(v[1].as_str().unwrap().parse().unwrap());
					high.push(v[2].as_str().unwrap().parse().unwrap());
					low.push(v[3].as_str().unwrap().parse().unwrap());
					close.push(v[4].as_str().unwrap().parse().unwrap());
					volume.push(v[7].as_str().unwrap().parse().unwrap());
					trades.push(v[8].as_f64().unwrap());
					taker_volume.push(v[10].as_str().unwrap().parse().unwrap());
				}

				let df = df!(
					"close_ms" => close_ms,
					"open" => open,
					"high" => high,
					"low" => low,
					"close" => close,
					"volume" => volume,
					"trades" => trades,
					"taker_volume" => taker_volume,
				)
				.unwrap();
				let k: Klines = df.try_into().unwrap();
				k
			}
			_ => panic!("Conversion to klines for this Market is not supported yet"),
		}
	}
}
