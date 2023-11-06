use crate::types::*;
use anyhow::{self, Result};
use chrono::{NaiveDateTime, Utc};
use reqwest::Response;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Client {
	api_key: String,
	rate_limit: Mutex<RateLimit>,
}
// sleep_if_needed();
impl Client {
	pub async fn request(&self, url: String, params: &HashMap<&str, &str>) -> Result<reqwest::Response> {
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert("X-MBX-APIKEY", self.api_key.parse().unwrap()); // not sure why not just `.as_str()`
		headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

		{
			let mut rate_limit = self.rate_limit.lock().unwrap();
			rate_limit.sleep_if_needed();
		}

		let r = reqwest::Client::new().get(url.as_str()).query(&params).headers(headers).send().await.unwrap();

		{
			let mut rate_limit = self.rate_limit.lock().unwrap();
			rate_limit.update(&r);
		}

		Ok(r)
	}
}

pub struct RateLimit {
	minute: String,
	used: i32,
	threshold: i32,
	calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>,
}

impl RateLimit {
	pub fn build(threshold: i32, calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>) -> Self {
		let minute = Self::now_minute();
		let used = 0;
		RateLimit { minute, used, threshold, calc_used }
	}
	pub fn now_minute() -> String {
		Utc::now().format("%Y-%m-%d %H:%M").to_string()
	}
	pub async fn update(&mut self, r: &Response) {
		let new_used: i32 = (self.calc_used)(self.used, r);

		let current_minute = Self::now_minute();
		if self.minute == current_minute {
			self.used = new_used;
		} else {
			self.minute = current_minute;
			self.used = new_used;
		}
		eprintln!("#From RateLimit# Used: {}", self.used);
	}
	pub fn sleep_if_needed(&self) {
		if self.used > (self.threshold as f32 * 0.9) as i32 {
			let stored_minute = NaiveDateTime::parse_from_str(&self.minute, "%Y-%m-%d %H:%M").expect("failed to parse the provided minute string of RateLimit");
			let next_minute = stored_minute + chrono::Duration::minutes(1);
			let current_time = Utc::now().naive_utc();
			let duration = next_minute.signed_duration_since(current_time);
			let sleep_ms = duration.num_milliseconds();
			if sleep_ms > 0 {
				eprintln!("Hit 90% of the threshold, sleeping for {}ms", &sleep_ms);
				std::thread::sleep(std::time::Duration::from_millis(sleep_ms as u64));
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct TradesParams {
	pub symbol: String,
	pub start_time: Timestamp,
	pub end_time: Timestamp,
	pub id: String,
}
impl TradesParams {
	pub fn build(symbol: String, start_time: Timestamp, end_time: Timestamp, id: String) -> TradesParams {
		assert!(start_time.ms < end_time.ms, "Panic during building the TradesParams: start_time must be less than end_time");
		TradesParams { symbol, start_time, end_time, id }
	}
}
pub enum Params {
	TradesParams,
}

//=============================================================================
impl fmt::Debug for RateLimit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("RateLimit")
			.field("minute", &self.minute.lock().unwrap())
			.field("used", &self.used.load(std::sync::atomic::Ordering::SeqCst))
			.field("threshold", &self.threshold)
			.finish()
	}
}
