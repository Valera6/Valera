//! A thing that does a thing
//!
//! Errors:
//! NB: all client-facing creation methods are presented as infallible, as per non-continuous nature of intended application of the lib. While on the inside guidelines for error-propagation are strictly followed, most .from() or .new() calls will fail in absence of internet connection or if provided with out-of-bounds arguments.

#![cfg_attr(nightly_error_messages, allow(internal_features), feature(rustc_attrs))]
#![warn(
	clippy::all,
	clippy::todo,
	clippy::empty_enum,
	clippy::enum_glob_use,
	clippy::mem_forget,
	clippy::unused_self,
	clippy::filter_map_next,
	clippy::needless_continue,
	clippy::needless_borrow,
	clippy::match_wildcard_for_single_variants,
	clippy::if_let_mutex,
	clippy::mismatched_target_os,
	clippy::await_holding_lock,
	clippy::match_on_vec_items,
	clippy::imprecise_flops,
	clippy::suboptimal_flops,
	clippy::lossy_float_literal,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::fn_params_excessive_bools,
	clippy::exit,
	clippy::inefficient_to_string,
	clippy::linkedlist,
	clippy::macro_use_imports,
	clippy::option_option,
	clippy::verbose_file_reads,
	clippy::unnested_or_patterns,
	clippy::str_to_string,
	rust_2018_idioms,
	future_incompatible,
	nonstandard_style,
	missing_debug_implementations
)] //, missing_docs)]
#![deny(unreachable_pub)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
// #![allow(dead_code, unused_imports, unused_variables)] //dbg
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]
#![feature(async_fn_in_trait)]

pub mod prelude;

pub mod backtest;
pub mod data_science;
pub mod display;
pub mod exchanges;
pub mod requests;
pub mod types;
