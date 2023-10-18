use crate::types::*;
use chrono::{NaiveDateTime, Utc};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TradesPayload {
	pub symbol: String,
	pub start_time: Timestamp,
	pub end_time: Timestamp,
	pub id: String,
}
impl TradesPayload {
	pub fn build(symbol: String, start_time: Timestamp, end_time: Timestamp, id: String) -> TradesPayload {
		assert!(start_time.ms < end_time.ms, "Panic during building the TradesPayload: start_time must be less than end_time");
		TradesPayload { symbol, start_time, end_time, id }
	}
}

#[derive(Debug, Default)]
pub struct RateLimit {
	minute: Mutex<String>,
	used: AtomicI32,
}

impl RateLimit {
	pub fn new() -> Self {
		let minute = Mutex::new(Self::now_minute());
		let used = AtomicI32::new(0);
		RateLimit { minute, used }
	}
	pub fn now_minute() -> String {
		Utc::now().format("%Y-%m-%d %H:%M").to_string()
	}
	pub async fn update(&self, used_update: i32) {
		let current_minute = Self::now_minute();
		let mut minute_guard = self.minute.lock().unwrap();

		if *minute_guard == current_minute {
			self.used.store(used_update, Ordering::Relaxed);
		} else {
			*minute_guard = current_minute;
			self.used.store(used_update, Ordering::Relaxed);
		}
		eprintln!("#From RateLimit# Used: {}", self.used.load(Ordering::Relaxed));
	}
	pub fn sleep_if_needed(&self) {
		let minute = self.minute.lock().unwrap();
		let used = self.used.load(Ordering::Relaxed);
		let THRESHOLD = 5500;
		if used > THRESHOLD {
			let stored_minute = NaiveDateTime::parse_from_str(&minute, "%Y-%m-%d %H:%M").expect("failed to parse the provided minute string of RateLimit");
			let next_minute = stored_minute + chrono::Duration::minutes(1);
			let current_time = Utc::now().naive_utc();
			let duration = next_minute.signed_duration_since(current_time);
			let sleep_ms = duration.num_milliseconds();
			if sleep_ms > 0 {
				eprintln!("Hit the threashold, sleeping for {}ms", &sleep_ms);
				std::thread::sleep(std::time::Duration::from_millis(sleep_ms as u64));
			}
		}
	}
}
