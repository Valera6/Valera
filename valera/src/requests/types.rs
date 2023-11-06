use crate::types::*;
use chrono::{NaiveDateTime, Utc};
use reqwest::Response;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

//#[derive(Debug, Clone)]
//pub struct Client {
//
//}

//TODO: probably put into enum `Params`
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

pub struct RateLimit {
	minute: Mutex<String>,
	used: AtomicI32,
	threshold: i32,
	calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>,
}

impl RateLimit {
	pub fn build(threshold: i32, calc_used: Box<dyn Fn(i32, &reqwest::Response) -> i32>) -> Self {
		let minute = Mutex::new(Self::now_minute());
		let used = AtomicI32::new(0);
		RateLimit { minute, used, threshold, calc_used }
	}
	pub fn now_minute() -> String {
		Utc::now().format("%Y-%m-%d %H:%M").to_string()
	}
	pub async fn update(&self, r: &Response) {
		//TODO!!!!!!: change to direct acquiring of the value's lock.
		let current_used: i32 = self.used.load(Ordering::Relaxed);

		let new_used: i32 = (self.calc_used)(current_used, r);

		let current_minute = Self::now_minute();
		let mut minute_guard = self.minute.lock().unwrap();

		if *minute_guard == current_minute {
			self.used.store(new_used, Ordering::Relaxed);
		} else {
			*minute_guard = current_minute;
			self.used.store(new_used, Ordering::Relaxed);
		}
		eprintln!("#From RateLimit# Used: {}", current_used);
	}
	pub fn sleep_if_needed(&self) {
		let minute = self.minute.lock().unwrap();
		let used = self.used.load(Ordering::Relaxed);
		if used > (self.threshold as f32 * 0.9) as i32 {
			let stored_minute = NaiveDateTime::parse_from_str(&minute, "%Y-%m-%d %H:%M").expect("failed to parse the provided minute string of RateLimit");
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
