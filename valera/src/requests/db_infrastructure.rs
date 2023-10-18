use crate::requests::TradesPayload;
use crate::types::{Timestamp, UsdtSymbol};
use polars::prelude::{ParquetReader, SerReader};

pub fn build_payloads(name: &str) -> Vec<TradesPayload> {
	// I'm going to try to make this be a gRPC call.
	// Hence, an always online server repulling the trades from discord every 15m, storing, then sharing on request.
	//-- for now just loading the file, until my api is up

	let filename = [name, ".parquet"].concat();
	let _file = std::fs::File::open(filename.as_str()).unwrap();
	let df = ParquetReader::new(_file).finish().unwrap();
	dbg!(&df);

	let mut payloads: Vec<TradesPayload> = Vec::new();
	let n_rows = df.height();
	for i in 0..n_rows {
		let row = df.get_row(i).unwrap();
		let timestamp: i64 = row.0.get(2).unwrap().try_extract::<i64>().unwrap();
		let coin = row.0.get(6).unwrap().get_str().unwrap();
		let id = row.0.get(8).unwrap().get_str().unwrap();

		let symbol: String = UsdtSymbol::from(coin).0;
		let start_time = Timestamp::from(timestamp).subtract(30);
		let end_time = Timestamp::from(timestamp).add(150);
		let id = id.to_owned();

		let payload = TradesPayload::build(symbol, start_time, end_time, id);
		payloads.push(payload);
	}
	payloads
}
