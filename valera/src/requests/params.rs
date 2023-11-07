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
pub enum Params {
	TradesParams,
}
