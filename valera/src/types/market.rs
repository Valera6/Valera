use crate::types::Klines;
use polars::prelude::{df, DataFrame, NamedFrom};

/// Market handles all the nuances for interacting with different exchanges.
#[derive(Debug, Default, Clone)]
pub enum Market {
	#[default]
	None,
	BinancePerp,
	BinanceSpot,
}

impl Market {
	pub fn name(&self) -> &'static str {
		match self {
			Market::BinancePerp => "binance-perp",
			Market::BinanceSpot => "binance-spot",
			Market::None => panic!("The Market is None"),
		}
	}
	pub fn get_base_url(&self) -> &'static str {
		match self {
			Market::BinancePerp => "https://fapi.binance.com/fapi/v1",
			Market::BinanceSpot => "https://api.binance.com/api/v3",
			Market::None => panic!("The Market is None"),
		}
	}
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
			Market::BinancePerp | Market::BinanceSpot => {
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

impl From<&str> for Market {
	fn from(s: &str) -> Self {
		match s {
			"binance-perp" => Market::BinancePerp,
			"binance-spot" => Market::BinanceSpot,
			_ => panic!("Can't convert provided string to Market.\nHave: {s}\nWant: exchange-market; e.g., binance-perp"),
		}
	}
}
impl From<String> for Market {
	fn from(s: String) -> Self {
		Self::from(s.as_str())
	}
}

#[cfg(test)]
mod types_market {
	use super::*;

	#[test]
	fn test_get_url() {
		assert_eq!("https://fapi.binance.com/fapi/v1", Market::BinancePerp.get_base_url());
	}
}
