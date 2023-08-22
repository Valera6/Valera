use chrono::{DateTime, NaiveDateTime, Utc};
use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Timestamp {
	pub ns: i64,
	pub s: i64,
	pub ms: i64,
	pub us: i64,
	pub datetime: DateTime<Utc>,
	pub isoformat: String,
}

impl Timestamp {
	pub fn new<T: Into<Timestamp>>(timestamp: T) -> Self {
		timestamp.into()
	}

	fn from_ns(ns: i64) -> Result<Self, &'static str> {
		let s = ns / 1_000_000_000;
		let ms = ns / 1_000_000;
		let us = ns / 1_000;
		let naive_datetime = NaiveDateTime::from_timestamp_opt(ns / 1_000_000_000, 0).ok_or("Invalid timestamp")?;
		let datetime = DateTime::<Utc>::from_utc(naive_datetime, Utc);
		let isoformat = datetime.to_rfc3339();
		Ok(Timestamp { ns, s, ms, us, datetime, isoformat })
	}
}

impl TryFrom<i64> for Timestamp {
	type Error = &'static str;

	fn try_from(timestamp: i64) -> Result<Self, Self::Error> {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp * 1_000_000_000,
			13 => timestamp * 1_000_000,
			16 => timestamp * 1_000,
			19 => timestamp,
			_ => return Err("Provided timestamp type isn't supported."),
		};
		Timestamp::from_ns(ns)
	}
}

impl TryFrom<i32> for Timestamp {
	type Error = &'static str;

	fn try_from(timestamp: i32) -> Result<Self, Self::Error> {
		let len = timestamp.to_string().len();
		let ns = match len {
			10 => timestamp as i64 * 1_000_000_000,
			_ => return Err("Provided timestamp type isn't supported."),
		};
		Timestamp::from_ns(ns)
	}
}

impl TryFrom<&str> for Timestamp {
	type Error = &'static str;

	fn try_from(timestamp: &str) -> Result<Self, Self::Error> {
		let dt = timestamp.parse::<DateTime<Utc>>().map_err(|_| "Invalid ISO format")?;
		let ns = dt.timestamp_nanos();
		Timestamp::from_ns(ns)
	}
}

impl TryFrom<DateTime<Utc>> for Timestamp {
	type Error = &'static str;

	fn try_from(timestamp: DateTime<Utc>) -> Result<Self, Self::Error> {
		let ns = timestamp.timestamp_nanos();
		Timestamp::from_ns(ns)
	}
}

impl TryFrom<SystemTime> for Timestamp {
	type Error = &'static str;

	fn try_from(timestamp: SystemTime) -> Result<Self, Self::Error> {
		let duration_since_epoch = timestamp.duration_since(UNIX_EPOCH).unwrap();
		let ns = duration_since_epoch.as_nanos() as i64;
		Timestamp::from_ns(ns)
	}
}
