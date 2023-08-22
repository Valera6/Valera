use backtrace::Backtrace;
use reqwest::blocking::Client;

pub fn get_caller_name() -> String {
	let bt = Backtrace::new();
	let get_name = |index: usize| -> String { bt.frames().get(index).and_then(|frame| frame.symbols().get(0)).and_then(|symbol| symbol.filename()).and_then(|filename| filename.file_stem()).map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "unknown".to_string()) };
	format!("{}", get_name(1))
}

pub fn tg_msg(text: Option<&str>) {
	let message = match text {
		Some(t) => t.to_string(),
		None => format!("{} has finished", get_caller_name()),
	};
	let params = [("chat_id", "-1001800341082"), ("text", &message)];
	let _ = Client::new().post("https://api.telegram.org/bot6225430873:AAEYlbJ2bY-WsLADxlWY1NS-z4r75sf9X5I/sendMessage").form(&params).send();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_tg_msg() {
//         tg_msg(None);
//     }
// }
