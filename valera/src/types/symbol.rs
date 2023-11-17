use serde::{Deserialize, Serialize};
use std::fmt;

/// Used by most requests::api endpoints
///_NB_: in the `.as_strings()` we don't check for validity of provided strings
pub enum Symbols{
	CoinAsString(String),
	CoinsAsStrings(Vec<String>),
	CoinAsSymbol(Box<dyn Symbol>),
	CoinsAsSymbols(Vec<Box<dyn Symbol>>),
}
impl Symbols {
	pub fn as_strings(&self) -> Vec<String> {
		match self {
			Self::CoinAsString(s) => vec![s.clone()],
			Self::CoinsAsStrings(v) => v.clone(),
			Self::CoinAsSymbol(s)=> vec![s.inner()],
			Self::CoinsAsSymbols(v) => v.iter().map(|s| s.inner()).collect(),
		}
	}
}

/// So functions can say they just want any symbol. Could use enum, but community suggested following logic: `likely to add more structs ? use trait : likely to implement more functionality on each ? use enum`
pub trait Symbol {
	fn inner(&self) -> String {
		self.inner()
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UsdtSymbol(pub String);
impl Symbol for UsdtSymbol {
	fn inner(&self) -> String {
		self.0.clone()
	}
}
impl fmt::Debug for UsdtSymbol {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.inner())
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CoinmSymbol(pub String);
impl Symbol for CoinmSymbol {
	fn inner(&self) -> String {
		self.0.clone()
	}
}
impl fmt::Debug for CoinmSymbol {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.inner())
	}
}

impl std::convert::From<&str> for UsdtSymbol {
	fn from(value: &str) -> Self {
		if !["BUSD", "ETH", "BTC", "BNB", "USDC", "TUSD"].into_iter().all(|quote| !(value.ends_with(quote) && value != quote)) {
			panic!("UsdtSymbol must be quoted against USDT.\nHave: {}\nWant: BTCUSDT", value);
		}

		let mut s: String = value.to_owned().to_uppercase();
		if !s.ends_with("USDT") {
			s += "USDT";
		}
		Self(s.to_owned())
	}
}

impl std::convert::From<&str> for CoinmSymbol {
	fn from(value: &str) -> Self {
		if !["USDT"].into_iter().all(|quote| !(value.ends_with(quote) && value != quote)) {
			panic!("CoinmSymbol cannot be quoted against USDT.\nHave: {}\nWant: COINBTC", value);
		}

		let s: String = value.to_owned().to_uppercase();
		Self(s.to_owned())
	}
}

#[cfg(test)]
mod types_symbol {
	use super::*;

	#[test]
	fn test_into_from_just_coinname() {
		let s: UsdtSymbol = "btc".into();
		assert_eq!(s.inner(), "BTCUSDT");
	}
	#[test]
	fn test_into_standard() {
		let _: UsdtSymbol = "ETHUSDT".into();
	}
	#[test]
	#[should_panic]
	fn test_into_not_quoted_against_usdt() {
		let _: UsdtSymbol = "SOLBNB".into();
	}
	#[test]
	fn test_coinm() {
		let _: CoinmSymbol = "USDTBTC".into();
	}
	#[test]
	#[should_panic]
	fn test_coinm_quoted_against_usdt() {
		let _: CoinmSymbol = "BTCUSDT".into();
	}
}
