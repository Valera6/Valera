use std::convert::TryInto;
use std::{collections::HashMap, time::SystemTime};
use tokio;
use valera::types::Timestamp::Timestamp;

pub fn request(url: &str, params: Option<HashMap<&str, &str>>) -> Result<String, reqwest::Error> {
	let r = tokio::runtime::Runtime::new().unwrap().block_on(get_request(url, None));
	r
}

async fn get_request(url: &str, params: Option<HashMap<&str, &str>>) -> Result<String, reqwest::Error> {
	let now_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;
	let t: Timestamp = now_ms.try_into().unwrap();

	let start_time = (t.ms - 100 * 5 * 60 * 1000).to_string();
	let end_time = t.ms.to_string();

	let mut params = HashMap::new();
	params.insert("symbol", "BTCUSDT");
	params.insert("interval", "5m");
	params.insert("startTime", &start_time);
	params.insert("endTime", &end_time);

	let client = reqwest::Client::new();
	let r = client.get(url).query(&params).send().await?.text().await?;
	Ok(r)
}
