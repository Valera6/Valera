#![allow(dead_code, clippy::useless_format)]

use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_caller_name() -> String {
	let bt = backtrace::Backtrace::new();
	let get_name = |index: usize| -> String {
		bt.frames()
			.get(index)
			.and_then(|frame| frame.symbols().get(0))
			.and_then(|symbol| symbol.filename())
			.and_then(|filename| filename.file_stem())
			.map(|name| name.to_string_lossy().into_owned())
			.unwrap_or_else(|| "unknown".to_string())
	};
	format!("{}", get_name(1))
}

#[derive(Clone, Debug)]
pub struct ProgressBar {
	bar_width: f64,
	timestamp_ms: u128,
	total: f64,
}
impl ProgressBar {
	pub fn new(total: usize) -> Self {
		let bar_width: f64 = 133.0;
		let timestamp_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
		let total = total as f64;
		ProgressBar { bar_width, timestamp_ms, total }
	}
	pub fn progress(&mut self, i: usize) {
		const CLEAR: &str = "\x1B[2J\x1B[1;1H";
		let scalar: f64 = self.bar_width / self.total;
		let display_i = (i as f64 * scalar) as usize;
		let display_total = (self.total * scalar) as usize;

		println!("{}", CLEAR);
		println!("[{}{}]", "*".repeat(display_i), " ".repeat(display_total - display_i));

		let since_timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - self.timestamp_ms;
		let progress_left_scalar = (self.total - i as f64) / i as f64;
		let left_s = (since_timestamp_ms as f64 * progress_left_scalar / 1000.0) as usize;
		println!("Time left: ≈ {}s", left_s);
	}
}

// Json
//Todo: move this under feature-flag json, so I don't import all this crap for every project. Do same for all others.
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
pub fn jdump<T: Serialize>(filepath: String, object: T) {
	let parent_dir = std::path::Path::new(&filepath).parent().unwrap();
	let _ = std::fs::create_dir_all(parent_dir);
	let mut file = File::create(&filepath).unwrap();
	let serialized = serde_json::to_string(&object).unwrap();
	file.write_all(serialized.as_bytes()).unwrap();
}
// to use do `let my_struct: MyStruct = jload(path);`. So definition of the struct should just be accessible somewhere in the code.
pub fn jload<T: DeserializeOwned>(filepath: String) -> T {
	let mut file = File::open(&filepath).unwrap();
	let mut contents = String::new();
	file.read_to_string(&mut contents).unwrap();
	return serde_json::from_str(&contents).unwrap();
}
pub fn jadd<T>(filepath: String, object: T)
where
	T: for<'de> Deserialize<'de> + Serialize,
{
	let parent_dir = std::path::Path::new(&filepath).parent().unwrap();
	let _ = std::fs::create_dir_all(parent_dir);

	let mut contents = String::new();
	if let Ok(mut file) = File::open(&filepath) {
		file.read_to_string(&mut contents).unwrap();
	}

	let mut objects: VecDeque<T> = serde_json::from_str(&contents).unwrap_or_else(|_| VecDeque::new());

	objects.push_back(object);

	let mut file = File::create(&filepath).unwrap();
	file.write_all(serde_json::to_string(&objects).unwrap().as_bytes()).unwrap();
}
//
