//! //TODO!: write documentation.
//! A thing that does a thing
//!
//! Errors:
//! NB: all client-facing creation methods are presented as infallible, as per non-continuous nature of intended application of the lib. While on the inside guidelines for error-propagation are strictly followed, most .from() or .new() calls will fail in absence of internet connection or if provided with out-of-bounds arguments.

pub mod backtest;
pub mod data_science;
pub mod display;
pub mod requests;
pub mod types;
