use crate::types::*;
use anyhow::{anyhow, Result};
use polars::prelude::*;

//todo add `impl TryFrom for serde_jsonValue
// propagete it up with anyhow::Result. No reason not to propagate errors at this level, so I will.

/// Note: we're assuming the first column of the DataFrame is always its index. Other parts of code rely on this assumption.
#[derive(Debug, Default)]
pub struct Klines {
	pub df: DataFrame,
	pub market: Providers,
	pub tf: Timeframe,
	pub normalized: bool,
	pub full: bool,
}

//todo add .to_full() // which includes normalized volume too
impl Klines {
	pub fn normalize(&mut self, zero_index: Option<usize>) {
		if !&self.normalized {
			let mut df = self.df.clone();

			let find_index = |target: usize| -> usize {
				let ch_array = df.select_at_idx(0).unwrap().i64().unwrap();
				// do the binary search. NB: assuming the 0 column is time index and thus sorted
				let mut a = 0 as usize;
				let mut b = ch_array.len() - 1 as usize;
				while a <= b {
					let mean = (a + b) / 2;
					let value_at_index = ch_array.get(mean).unwrap() as usize;
					if value_at_index < target {
						a = mean + 1;
					} else if value_at_index > target {
						b = mean - 1;
					} else {
						return mean;
					}
				}
				unreachable!();
			};

			let zero_index = match zero_index {
				Some(index) => find_index(index),
				None => 0 as usize,
			};
			let columns = ["open", "high", "low", "close", "oi", "lsr"];
			for name in columns {
				if let Ok(series) = df.column(name) {
					let zero_value: f64 = series.get(zero_index.try_into().unwrap()).unwrap().try_extract::<f64>().unwrap();
					let normalize_series = |s: &Series| -> Series { s.f64().unwrap().apply(|x| (x / zero_value).ln()).into_series() };
					df.apply(name, normalize_series).unwrap();
				}
			}
			self.df = df;
			self.normalized = true;
		}
	}
}

impl TryFrom<DataFrame> for Klines {
	type Error = anyhow::Error;
	fn try_from(mut df: DataFrame) -> Result<Self> {
		// checking the index is i64 and making all others be f64
		let index_dtype = df.select_at_idx(0).unwrap().dtype();
		if !matches!(index_dtype, DataType::Int64) {
			// maybe later will make the check try to convert to Timestamp instead
			return Err(anyhow!("the first column of the dataframe should be a milliseconds index in i64\nHave: {}", index_dtype));
		}
		unsafe {
			for series in df.get_columns_mut().iter_mut().skip(1) {
				if !matches!(series.dtype(), DataType::Float64) {
					eprintln!("Found non-index column that is not f64. Trying to cast.");
					*series = series.cast(&DataType::Float64).unwrap();
				}
			}
		}
		// now check that what we have is sufficient to be klines
		let required = ["open", "high", "low", "close"];
		let names: std::collections::HashSet<_> = df.get_column_names().clone().into_iter().collect();
		let mut missing: Vec<&str> = Vec::new();
		for &n in &required {
			if !names.contains(n) {
				missing.push(n);
			}
		}
		if !missing.is_empty() {
			return Err(anyhow::anyhow!(
				"The provided dataframe does not have all of the required fields\nRequired: {:?}\nMissing: {:?}",
				required,
				missing
			));
		}

		// probably not the most officient way of doing this

		let second_close_ms: i64 = df.select_at_idx(0).unwrap().get(1).unwrap().try_extract().unwrap();
		let third_close_ms: i64 = df.select_at_idx(0).unwrap().get(2).unwrap().try_extract().unwrap();
		let step_ms: i64 = third_close_ms - second_close_ms;
		let guess_tf = Timeframe::try_from(step_ms);

		let mut k = Klines::default();
		k.df = df;
		match guess_tf {
			Ok(tf) => k.tf = tf,
			Err(_) => eprintln!("WARNING: Couldn't infer the tf of the provided DataFrame. Not sure if that can break anything, so no error."),
		}
		Ok(k) // the fieds `normalize` and `full` are thus still false, and Market needs to be set manually in orrder to proceed with calling to_full()
	}
}
