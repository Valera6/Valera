#![allow(unused_imports, dead_code)]
use polars::prelude::*;
use std::collections::HashMap;
use valera::display::lib::plotly_closes;
use valera::exchanges::*;
use valera::prelude::*;
use valera::requests::{self, *};
use valera::types::*;

use crate::requests::db_infrastructure::LogEntry;

#[tokio::main]
async fn main() {
	// 1) load and dump the pandas datatframes with the trades

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
	async fn test_plotly_klines() {
		let closes_df = requests::api::get_closes_df().await;
		plotly_closes(closes_df);
	}
	fn unit_build_payloads() {
		let _payloads: Vec<LogEntry> = requests::db_infrastructure::build_payloads("main-trades-log");
		dbg!(&_payloads);
	}
	async fn integration_collect_trades() {
		let payloads: Vec<LogEntry> = requests::db_infrastructure::build_payloads("main-trades-log");

		let provider = Templates::BinancePerp.build();

		provider.collect_and_dump_trades(payloads[0]).await; //might overwrite the previous query
	}
}
