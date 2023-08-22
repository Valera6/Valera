use valera_requests::request;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_main() {
		main();
	}
}

fn main() {
	let url = "https://api.binance.com/api/v3/klines";
	let r = request(url, None);

	match r {
		Ok(response) => println!("{:#?}", response.len()),
		Err(error) => println!("Error: {:?}", error),
	}
}
