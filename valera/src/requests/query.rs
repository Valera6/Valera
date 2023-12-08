use anyhow::{Context, Result};
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use polars::prelude::{df, NamedFrom, DataFrame};

use crate::requests::client::*;
use crate::requests::providers::*;
use crate::types::*;

/// Schedulers put the split parts on the 2D grid, to later reconstruct.
/// `horizontal` used for coins, `vertical` used for time intervals.
pub struct QueryGridPos {
	pub x: u32,
	pub y: u32,
}

pub struct Query {
	sender: Sender,
	url: String,
	logic: Box<dyn Fn()>,
	grid_pos: QueryGridPos,
	start_time: Option<Timestamp>,
	end_time: Option<Timestamp>,
	/// includes symbol too
	other_params: HashMap<String, String>,
	/// time per one server interaction is pretty much constant for the provider. So no matter the query, the request weight itself can be used to determine cost of running the query per unit of time.
	weight: u32,
}

impl Query {
	pub fn build(
		sender: Sender,
		url: String,
		logic: Box<dyn Fn()>,
		grid_pos: QueryGridPos,
		start_time: Option<Timestamp>,
		end_time: Option<Timestamp>,
		other_params: HashMap<String, String>,
		weight: u32,
	) -> Self {
		Query {
			sender,
			url,
			logic,
			grid_pos,
			start_time,
			end_time,
			other_params,
			weight,
		}
	}
	/// A wrapper function around `logic` field of the Query, that 1) allows access to all its fields, which would normally be difficult to get the compiler to like, 2) convenient way to implement repetetivve things, like checking whether `start_time` and `end_time` are `Some` to determine whether the request is singular, or we should loop.
	pub async fn execute(&self, client: &Client) {
		pub fn trades_entry_into_row(entry: &serde_json::Value) -> DataFrame {
			df!(
				"time_ms" => vec![entry.get("time").unwrap().as_i64().unwrap()],
				"price" => vec![entry.get("price").unwrap().as_str().unwrap().parse::<f64>().unwrap()],
				"qty" => vec![entry.get("quoteQty").unwrap().as_str().unwrap().parse::<f64>().unwrap()],
				"isBuyerMaker" => vec![entry.get("isBuyerMaker").unwrap().as_bool().unwrap()],
			)
				.unwrap()
		}

		//TODO!: 1) put the logic into a closure, use `move` keywoard if needed.
		//TODO!: 2) move out to be held by the `logic` field

		//NB: this function is very specifc to Binance. That's why I will have it and some other things be passed as closure
		// also, this allows us to assume start_time and end_time are definitely provided.
		let find_fromid = async {
			let base_url = self.url.clone().rsplitn(2, '/').nth(1).unwrap_or_default();
			let url = format!(
				"{}/aggTrades?symbol={}&startTime={}&limit=1",
				base_url,
				self.other_params.get("symbol").unwrap(),
				self.start_time.clone().unwrap().ms
			);
			let json = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();
			json[0]["f"].as_i64().unwrap().to_string()
		}
		.await;

		let url = self.url; // later could be optimised with some kind of reference or a static lifetime. For now just String.

		let mut params = self.other_params;
		params.insert("fromId".to_owned(), find_fromid);

		let mut last_reached_ms = self.start_time.unwrap().ms;
		let mut buffer_df = DataFrame::default();
		while last_reached_ms < self.end_time.unwrap().ms {
			// In the perfect world check the code, and never print out the same error code twice.
			let r = match client.request(url, &params).await {
				Ok(response) => response,
				Err(e) => eprintln!("Request errored: {}", e),
			};

			let r_json = r.json::<serde_json::Value>().await.unwrap();

			let array_of_values = r_json.as_array().unwrap().to_vec();
			let mut new_fromid = String::new();
			for v in array_of_values.iter() {
				let t = v.get("time").unwrap().as_i64().unwrap();
				if t <= end_time.ms {
					let row: DataFrame = trades_entry_into_row(&v);
					buffer_df.vstack_mut(&row).unwrap();
				}
				new_from_id = (v.get("id").unwrap().as_i64().unwrap() + 1).to_string(); // because the thing is inclusive, I checked.
				last_reached_ms = v.get("time").unwrap().as_i64().unwrap();
			}
			params.insert("fromId".to_owned(), Box::leak(new_from_id.clone().into_boxed_str()));
		}
		self.parent.append_result(buffer_df);
	}
}
