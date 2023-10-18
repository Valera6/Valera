use chrono::{DateTime, NaiveDateTime, Utc};
use std::convert::From;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Timestamp {
	pub ns: i64,
	pub s: i64,
	pub ms: i64,
	pub us: i64,
	pub dt: DateTime<Utc>,
	pub iso: String,
}

/// Core
///
impl Timestamp {
	pub fn get_ms(&self) -> &i64 {
		&self.ms
	}
	fn from_ns(ns: i64) -> Self {
		let s = ns / 1_000_000_000;
		let ms = ns / 1_000_000;
		let us = ns / 1_000;
		let naive_datetime = NaiveDateTime::from_timestamp_micros(us).unwrap();
		let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
		let iso = dt.to_rfc3339();
		Timestamp { ns, s, ms, us, dt, iso }
	}
}
impl From<i64> for Timestamp {
	fn from(timestamp: i64) -> Self {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp * 1_000_000_000,
			13 => timestamp * 1_000_000,
			16 => timestamp * 1_000,
			19 => timestamp,
			_ => panic!("Provided timestamp type isn't supported: {}", timestamp),
		};
		Timestamp::from_ns(ns)
	}
}
impl From<i32> for Timestamp {
	fn from(timestamp: i32) -> Self {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp as i64 * 1_000_000_000,
			_ => panic!("Provided timestamp type isn't supported: {}", timestamp),
		};
		Timestamp::from_ns(ns)
	}
}
impl From<u64> for Timestamp {
	fn from(timestamp: u64) -> Self {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp * 1_000_000_000,
			13 => timestamp * 1_000_000,
			16 => timestamp * 1_000,
			19 => timestamp,
			_ => panic!("Provided timestamp type isn't supported: {}", timestamp),
		};
		Timestamp::from_ns(ns as i64)
	}
}
impl From<u32> for Timestamp {
	fn from(timestamp: u32) -> Self {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp as i64 * 1_000_000_000,
			_ => panic!("Provided timestamp type isn't supported: {}", timestamp),
		};
		Timestamp::from_ns(ns)
	}
}
impl From<&str> for Timestamp {
	fn from(timestamp: &str) -> Self {
		let dt = timestamp.parse::<DateTime<Utc>>().expect(&format!("Invalid ISO format: {}", timestamp));
		let ns = dt.timestamp_nanos();
		Timestamp::from_ns(ns)
	}
}
impl From<DateTime<Utc>> for Timestamp {
	fn from(timestamp: DateTime<Utc>) -> Self {
		let ns = timestamp.timestamp_nanos();
		Timestamp::from_ns(ns)
	}
}
impl From<SystemTime> for Timestamp {
	fn from(timestamp: SystemTime) -> Self {
		let duration_since_epoch = timestamp.duration_since(UNIX_EPOCH).unwrap();
		let ns = duration_since_epoch.as_nanos() as i64;
		Timestamp::from_ns(ns)
	}
}

/// additional direct functions
///
impl Timestamp {
	/// Creates timestamp object using current time
	pub fn now() -> Self {
		let now_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;
		Self::from_ns(now_ms * 1_000_000)
	}
	pub fn new() -> Self {
		Self::now()
	}

	/// mutably subtracts defined number of seconds from all fields of the object
	pub fn subtract<T: Into<i64>>(&self, s: T) -> Self {
		let ns_to_subtract = s.into() * 1_000_000_000;
		Self::from_ns(self.ns - ns_to_subtract)
	}
	pub fn add<T: Into<i64>>(&self, s: T) -> Self {
		let ns_to_add = s.into() * 1_000_000_000;
		Self::from_ns(self.ns + ns_to_add)
	}
}

/// good practices:
///
use std::fmt;
impl fmt::Debug for Timestamp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.iso)
	}
}

impl Default for Timestamp {
	fn default() -> Self {
		Timestamp::now()
	}
}

#[cfg(test)]
mod types_timestamp {
	use super::*;

	#[test]
	fn test_new_and_substract() {
		let t = Timestamp::now();
		use std::time::SystemTime;
		let now_s = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
		assert_eq!(t.s, now_s);
		assert_eq!(t.subtract(100 * 5 * 60).s, now_s - 100 * 5 * 60);
	}
}
