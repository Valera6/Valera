use crate::types::*;
use polars::prelude::{ParquetReader, SerReader};

pub fn build_payloads(name: &str) -> Vec<(Box<dyn Symbol>, Timestamp, Timestamp, String)> {	
	//TODO!: make into gRPC call

	let filename = [name, ".parquet"].concat();
	let _file = std::fs::File::open(filename.as_str()).unwrap();
	let df = ParquetReader::new(_file).finish().unwrap();

	let mut payloads: Vec<(Box<dyn Symbol>, Timestamp, Timestamp, String)> = Vec::new();
	let n_rows = df.height();
	for i in 0..n_rows {
		let row = df.get_row(i).unwrap();
		let timestamp: i64 = row.0.get(2).unwrap().try_extract::<i64>().unwrap();
		let coin = row.0.get(6).unwrap().get_str().unwrap();
		let id = row.0.get(8).unwrap().get_str().unwrap();

		let symbol = UsdtSymbol::from(coin);
		let start_time = Timestamp::from(timestamp).subtract(30);
		let end_time = Timestamp::from(timestamp).add(150);
		let id = id.to_owned();

		payloads.push((Box::new(symbol), start_time, end_time, id));
	}
	payloads
}
