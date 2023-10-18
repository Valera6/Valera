/// For now that's just binance timeframes. Will need refactoring to be inclusive of all possible sets, having is_binance_tf as an additional struct, with try_from implemented for conversion into it from Timeframe
use anyhow::Result;
#[derive(Debug, Default)]
pub struct Timeframe(String);

impl Timeframe {
	pub fn inner(&self) -> &str {
		&self.0
	}
	pub fn as_str(&self) -> &str {
		self.inner()
	}

	pub fn get_seconds(&self) -> i64 {
		let tf = &self.0;
		let num = tf[0..tf.len() - 1].parse::<i64>().expect("Invalid format");
		let interval = tf.chars().last().unwrap();
		let multiplier = match interval {
			'm' => 60,
			'h' => 60 * 60,
			'd' => 24 * 60 * 60,
			'w' => 7 * 24 * 60 * 60,
			'M' => 30 * 24 * 60 * 60,
			_ => panic!("Invalid interval"),
		};
		num * multiplier
	}
}
/// Should I put TryFrom instead?
impl From<&str> for Timeframe {
	fn from(tf: &str) -> Self {
		let valid_values = vec!["1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w", "1M"];
		if !valid_values.contains(&tf) {
			panic!("Invalid tf format. Received: {}", tf);
		}
		Timeframe(tf.to_owned())
	}
}
/// # Bugs
/// High potential for them. We're doing us a little guessing.
/// This thing assumes we were passed seconds. Because the value is i32.
/// Both guesses for integers are Tryrom because are intended only for internal use.
impl TryFrom<u32> for Timeframe {
	type Error = anyhow::Error;
	fn try_from(mut seconds: u32) -> Result<Self> {
		let mut at_least_tf = None;
		if seconds % 60 == 0 {
			seconds /= 60;
			at_least_tf = Some('m');
		}
		if seconds % 60 == 0 {
			seconds /= 60;
			at_least_tf = Some('h');
		}
		if seconds % 24 == 0 {
			seconds /= 24;
			at_least_tf = Some('d');
		}
		if seconds % 7 == 0 {
			if seconds % 30 == 0 {
				seconds /= 30;
				at_least_tf = Some('M');
			} else {
				seconds /= 7;
				at_least_tf = Some('w');
			}
		}
		if at_least_tf == None {
			return Err(anyhow::anyhow!("Tf was too small, we don't support anything below 60s yet. You can complain loudly to me to get this fixed or submit PR.\nYou requested (seconds): {}", seconds));
		}
		Ok(Timeframe(format!("{}{}", seconds, at_least_tf.unwrap())))
	}
}
/// # Bugs
/// This thing assumes we were passed milliseconds, because it's i64.
impl TryFrom<u64> for Timeframe {
	type Error = anyhow::Error;
	fn try_from(milliseconds: u64) -> Result<Self> {
		let seconds = (milliseconds / 1000) as u32;
		Timeframe::try_from(seconds)
	}
}
impl TryFrom<i32> for Timeframe {
	type Error = anyhow::Error;
	fn try_from(seconds: i32) -> Result<Self> {
		Timeframe::try_from(seconds as u32)
	}
}
impl TryFrom<i64> for Timeframe {
	type Error = anyhow::Error;
	fn try_from(milliseconds: i64) -> Result<Self> {
		let seconds = (milliseconds / 1000) as i32;
		Timeframe::try_from(seconds as u32)
	}
}

// good practices
#[cfg(test)]
mod types_timeframe {
	use super::*;

	#[test]
	fn test_into_from_string() {
		let tf: Timeframe = "5m".try_into().unwrap();
		assert_eq!("5m", tf.inner());
	}
	#[test]
	#[should_panic]
	fn test_into_out_of_bounds() {
		let _tf: Timeframe = "4m".try_into().unwrap();
	}
	#[test]
	fn test_into_from_seconds() {
		let tf: Timeframe = 60.try_into().unwrap();
		assert_eq!("1m", tf.inner())
	}
	#[test]
	#[should_panic]
	fn test_from_negative() {
		let _tf = Timeframe::try_from(-300).unwrap();
	}
}
