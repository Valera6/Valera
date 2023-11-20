#![allow(unused_imports, dead_code)]
use polars::prelude::*;
use std::collections::HashMap;
use valera::display::lib::plotly_closes;
use valera::exchanges::*;
use valera::prelude::*;
use valera::requests;
use valera::types::*;

#[tokio::main]
async fn main() {
	let payloads = requests::db_infrastructure::build_payloads("main-trades-log");
	//dbg
	let symbol = payloads[0].0;
	let start_time = payloads[0].0;
	let end_time = payloads[0].0;
	let id = payloads[0].0;

	// if we want to persist Id for a specific query, have to do it here.
	//let query_id: String = match id {
	//	Some(provided) => provided,
	//	None => rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect(),
	//};

	requests::api::collect_trades(symbol, Some(start_time), Some(end_time), Some(id)).await;

	// 2) pull norm volumes against weighted last 4-1m.

	// 3) adapt plotly-closes to be able to just plot it(just make a new function, duplicate everything, generalize later)

	// 4) adapt plotly-closes to also take in a separate metric to sort the legends based on

	// 5) calculate the simplest DUMB and start passing it for this metric.

	// 6) make pretty where needed, make it print basic info on requesting id, write unit tests where needed

	// Great Success.
}

#[cfg(test)]
mod types {
	use super::*;
	// fn test_get_klines() {
	// 	let b = Binance::new();
	// 	let k = b.get_klines("perp".into(), &UsdtSymbol::from("btc"), Timestamp::now().subtract(3 * 5 * 60), Timestamp::now(), "5m".into());
	// 	assert_eq!(k.tf.as_str(), "5m");
	// }
	async fn test_plotly_klines() {
		let closes_df = requests::api::get_closes_df().await;
		plotly_closes(closes_df);
	}
	fn test_build_payloads() {
		let _payloads = requests::db_infrastructure::build_payloads("main-trades-log");
	}
	// async fn test_collect_trades() {
	// 	let payloads = requests::db_infrastructure::build_payloads("main-trades-log");
	// 	collect_trades(payloads, Market::BinancePerp).await; //might overwrite existing things
	// }
}
