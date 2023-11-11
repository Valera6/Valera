use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use tokio::task::spawn;

async fn naked_request(url: &str, payload: Vec<HashMap<&str, &str>>, rate_limit: RateLimit) -> Result<HashMap<String, Vec<serde_json::Value>>> {
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
async fn r(url: &str) -> serde_json::Value {
	reqwest::get(url)
		.await
		.expect("Failed to fetch from endpoint")
		.json::<serde_json::Value>()
		.await
		.expect("Failed to deserialize response")
}
