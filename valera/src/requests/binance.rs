use crate::requests::*;
use crate::types::*;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use tokio::task::spawn;

#[derive(Default)]
pub struct Binance {
	perp: Vec<UsdtSymbol>,
	spot: Vec<UsdtSymbol>,
	symbols: Vec<UsdtSymbol>,
	// the hash map will be needed in the future when I'm doing proxy carousels.
	pub rate_limits: DashMap<String, RateLimit>,
}

impl Binance {
	pub fn get_perp(&self) -> Vec<&str> {
		let usdt_symbols: &Vec<UsdtSymbol> = &self.perp;
		let strings: Vec<&str> = usdt_symbols.iter().map(|us| us.as_str()).collect();
		strings
	}
	pub fn get_spot(&self) -> Vec<&str> {
		let usdt_symbols: &Vec<UsdtSymbol> = &self.spot;
		let strings: Vec<&str> = usdt_symbols.iter().map(|us| us.as_str()).collect();
		strings
	}
	pub fn get_symbols(&self) -> Vec<&str> {
		let usdt_symbols: &Vec<UsdtSymbol> = &self.symbols;
		let strings: Vec<&str> = usdt_symbols.iter().map(|us| us.as_str()).collect();
		strings
	}
	pub async fn new() -> Self {
		let perp_future = Binance::r("https://fapi.binance.com/fapi/v1/ticker/24hr");
		let spot_future = Binance::r("https://api.binance.com/api/v3/ticker/24hr");
		let (perp_data, spot_data) = tokio::join!(perp_future, spot_future);

		fn extract_symbols(data: serde_json::Value) -> Vec<UsdtSymbol> {
			let to_drop = &["USDC", "BTCDOM", "BTCST"];
			let symbols = data
				.as_array()
				.unwrap()
				.iter()
				.filter_map(|ticker| {
					let symbol = ticker["symbol"].as_str()?;
					if symbol.ends_with("USDT") && !to_drop.contains(&&symbol[..symbol.len() - 4]) {
						Some(UsdtSymbol::from(symbol))
					} else {
						None
					}
				})
				.collect();
			symbols
		}
		let perp = extract_symbols(perp_data);
		let spot = extract_symbols(spot_data);
		let symbols = [&perp[..], &spot[..]].concat();
		let rate_limits = DashMap::new();

		Binance { perp, spot, symbols, rate_limits }
	}
}

/// Request has .requests() for queries with multiple coins and its individual-case .request(), which is just a shorthand for calling the former, but for one symbol only.
/// On my side I'm trying to split the provided params into the smallest number of separate queries possible, and then running them concurrently in a 429-aware manner.
pub trait Request {
	async fn requests(&self, url: &str, symbols: Vec<&str>, params: HashMap<&str, &str>) -> Result<HashMap<String, Vec<serde_json::Value>>>;
	async fn request(&self, url: &str, params: HashMap<&str, &str>) -> Result<Vec<serde_json::Value>>;
	async fn r(url: &str) -> serde_json::Value;
}

//cool in reqwest-middleware we have enum Retryable for classifying status codes returned by reqwest
// Might just copy their code instead of importing the lib. Should be pretty darn simple, considering the codes are mostly constant.
impl Request for Binance {
	async fn requests(&self, url: &str, symbols: Vec<&str>, params: HashMap<&str, &str>) -> Result<HashMap<String, Vec<serde_json::Value>>> {
		// the name of the symbol parameter is going to be infered from the Market, because why wouldn't it be
		// so no need to pass it as a separate argument, or leave one key in the params have "" for value.
		//todo a full function for processing params here, based on the knowledge that it's for Binance.

		async fn perform_requests(client: reqwest::Client, url: String, symbols: Vec<String>, params: HashMap<String, String>) -> Result<HashMap<String, Vec<serde_json::Value>>> {
			let mut handles = Vec::new();

			for s in symbols {
				let client = client.clone();
				let u = url.clone();
				let mut p = params.clone();
				p.insert("symbol".to_owned(), s.clone());
				let s = s.clone();

				let handle = spawn(async move {
					let json = client.get(u).query(&p).send().await?.json::<serde_json::Value>().await?;
					if let serde_json::Value::Object(map) = &json {
						if map.contains_key("code") {
							return Err(anyhow!("{}$Unsuccessful:\n{:#?}", &s, map));
						}
					}
					let array_res = json.as_array().unwrap().to_vec();
					Ok((s, array_res))
				});
				handles.push(handle);
			}

			let mut results = HashMap::new();
			let mut distinct_errors = HashSet::new();
			let mut errored_on = HashSet::new();
			for handle in handles {
				let res: Result<(String, Vec<serde_json::Value>)> = handle.await?;
				match res {
					Ok((symbol, data)) => {
						results.insert(symbol, data);
					}
					Err(error) => {
						let err_string = error.to_string();
						let split: Vec<&str> = err_string.split('$').collect();
						distinct_errors.insert(split[1].to_owned());
						errored_on.insert(split[0].to_owned());
					}
				}
			}
			if !errored_on.is_empty() {
				eprintln!("The requests for the following coins produced errors: {:?}", errored_on);
			}
			if errored_on.len() as f64 / results.len() as f64 > 0.15 {
				return Err(anyhow!("Distinct errors during requesting of data:\n{:#?}", distinct_errors));
			}
			Ok(results)
		}

		let client = reqwest::Client::new();
		let u = url.to_owned();
		let s: Vec<String> = symbols.iter().map(|&s| s.to_owned()).collect();
		let mut p: HashMap<String, String> = HashMap::new();
		for (k, v) in &params {
			p.insert((*k).to_owned(), (*v).to_owned());
		}

		let future = perform_requests(client, u, s, p);
		future.await
	}
	async fn request(&self, url: &str, params: HashMap<&str, &str>) -> Result<Vec<serde_json::Value>> {
		// This will be an individual case for the requests. Client-facing only, so don't need it now.
		// Not gonna do it now to prevent introducing inconsistencies until all abstractions are fixed.
		// No need for speed in these things, so we just wrap the provided values in a layer of generalization, pass to requests, then unwrap and pass out.
		todo!();
	}
	async fn r(url: &str) -> serde_json::Value {
		reqwest::get(url)
			.await
			.expect("Failed to fetch from spot endpoint")
			.json::<serde_json::Value>()
			.await
			.expect("Failed to deserialize response")
	}
}

#[derive(Deserialize, Debug)]
struct KlinesResponse(Vec<HashMap<String, String>>);

use std::fmt;
impl fmt::Debug for Binance {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.symbols)
	}
}
