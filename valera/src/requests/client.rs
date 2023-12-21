use anyhow::{self, Result};
use std::sync::mpsc::Sender;
use chrono::{NaiveDateTime, Utc};
use reqwest::Response;
use serde::Serialize;
#[cfg(feature = "json")]
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use crate::requests::providers::*;
use crate::requests::query::*;
use crate::types::*;

#[derive(Debug)]
pub struct ClientInit {
	pub api_key: Option<String>,
	pub proxy: Option<String>,
}
pub struct Client {
	rate_limit: Mutex<RateLimit>,
	api_key: Option<String>,
	proxy: Option<String>,
	queries: Mutex<Vec<Query>>,
}
impl Client {
	pub fn build(client_specific: ClientInit, rate_limit: i32, calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>) -> Self {
		let api_key = client_specific.api_key;
		let proxy = client_specific.proxy;
		let rate_limit = RateLimit::build(rate_limit, calc_used);
		return Client { api_key, rate_limit, proxy, queries: Vec::<Query>::new().into() };
	}
	pub async fn request<T: Serialize + ?Sized>(&self, url: String, params: &T) -> Result<reqwest::Response> {
		let mut headers = reqwest::header::HeaderMap::new();
		//TODO!!!!: move into the provider, so it supplies an immidiately appendable to headers bit
		if let Some(key) = &self.api_key {
			headers.insert("X-MBX-APIKEY", key.parse().unwrap());
		}
		headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());

		// Wrapping getting Mutex locks in scopes is to evade `Send` trait requirement check of any async call
		{
			let rate_limit = self.rate_limit.lock().unwrap();
			rate_limit.sleep_if_needed();
		}

		//TODO!!!: handle errors
		let r = match &self.proxy {
			None => reqwest::Client::new().get(url.as_str()).query(params).headers(headers).send().await.unwrap(),
			Some(proxy) => {
				todo!()
			}
		};

		{
			let mut rate_limit = self.rate_limit.lock().unwrap();
			rate_limit.update(&r);
		}

		Ok(r)
	}	
	pub fn try_start_more(&self) {
		todo!();
	}
}

struct RateLimit {
	minute: String,
	used: i32,
	safe_threshold: i32,
	calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>,
}

impl RateLimit {
	pub fn build(threshold: i32, calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>) -> Mutex<Self> {
		let minute = Self::now_minute();
		let used = 0;
		let safe_threshold = (threshold as f32 * 0.9) as i32;
		Mutex::from(RateLimit {
			minute,
			used,
			safe_threshold,
			calc_used,
		})
	}
	fn now_minute() -> String {
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
		if self.used > self.safe_threshold {
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

//=============================================================================
impl fmt::Debug for RateLimit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("RateLimit")
			.field("minute", &self.minute)
			.field("used", &self.used)
			.field("threshold", &self.safe_threshold)
			.finish()
	}
}
