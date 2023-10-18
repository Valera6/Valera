use plotly::common::Line;
use plotly::{Plot, Scatter};
use polars::prelude::*;
use std::mem::transmute;

pub fn plotly_closes(normalized_closes_df: DataFrame) {
	let performance = normalized_closes_df.tail(Some(1));
	let mut tuples: Vec<(String, f64)> = performance
		.get_column_names()
		.iter()
		.map(|&s| (s.to_owned(), performance.column(s).unwrap().get(0).unwrap().try_extract().unwrap()))
		.collect();
	tuples.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
	let log_size = (tuples.len() as f64).ln().round() as usize;
	let top: Vec<String> = tuples.iter().rev().take(log_size).map(|x| x.0.clone()).collect();
	let bottom: Vec<String> = tuples.iter().take(log_size).map(|x| x.0.clone()).collect();

	let mut plot = Plot::new();

	let mut add_trace = |name: &str, width: f64, color: Option<&str>, legend: Option<String>| {
		let color_static: Option<&'static str> = color.map(|c| unsafe { transmute::<&str, &'static str>(c) });
		let polars_vec: Vec<Option<f64>> = normalized_closes_df.column(name).unwrap().f64().unwrap().to_vec();
		let y_values: Vec<f64> = polars_vec.iter().filter_map(|&x| x).collect();
		let x_values: Vec<usize> = (0..y_values.len()).collect();

		let mut line = Line::new().width(width);
		if let Some(c) = color_static {
			line = line.color(c);
		}

		let mut trace = Scatter::new(x_values, y_values).mode(plotly::common::Mode::Lines).line(line);
		if let Some(l) = legend {
			trace = trace.name(&l);
		} else {
			trace = trace.show_legend(false);
		}
		plot.add_trace(trace);
	};

	let mut contains_btcusdt = false;
	for col_name in normalized_closes_df.get_column_names() {
		if col_name == "BTCUSDT" {
			contains_btcusdt = true;
			continue;
		}
		if top.contains(&col_name.to_string()) || bottom.contains(&col_name.to_string()) {
			continue;
		}
		add_trace(col_name, 1.0, Some("grey"), None);
	}
	for col_name in top.iter() {
		let p: f64 = performance.column(col_name).unwrap().get(0).unwrap().try_extract().unwrap();
		let mut symbol = col_name[0..col_name.len() - 4].to_string();
		symbol = symbol.replace("1000", "");
		let sign = if p >= 0.0 { '+' } else { '-' };
		let change = format!("{:.2}", 100.0 * p.abs());
		let legend = format!("{:<5}{}{:>5}%", symbol, sign, change);
		add_trace(col_name, 2.0, None, Some(legend));
	}
	if contains_btcusdt {
		let p: f64 = performance.column("BTCUSDT").unwrap().get(0).unwrap().try_extract().unwrap();
		add_trace("BTCUSDT", 3.5, Some("gold"), Some(format!("~BTC~ {:>5}", format!("{:.2}", 100.0 * p))));
	}
	for col_name in bottom.iter().rev() {
		let p: f64 = performance.column(col_name).unwrap().get(0).unwrap().try_extract().unwrap();
		let mut symbol = col_name[0..col_name.len() - 4].to_string();
		symbol = symbol.replace("1000", "");
		let sign = if p >= 0.0 { '+' } else { '-' };
		let change = format!("{:.2}", 100.0 * p.abs());
		let legend = format!("{:<5}{}{:>5}%", symbol, sign, change);
		add_trace(col_name, 2.0, None, Some(legend));
	}

	plot.show();
}

//     fig = go.Figure()

//     def add_trace(*args):
//         y, name, line, legend = args
//         fig.add_trace(
//                 go.Scatter(
//                     x=normalizedClosesDf.index,
//                     y=y,
//                     mode='lines',
//                     name=name,
//                     line=line,
//                     showlegend=legend
//                 )
//             )
//     def add_performers(column):
//         symbol = column[:-4]
//         symbol.replace('1000', '', 1)
//         sign = f"{performance[column]:+}"[0]
//         change = f"{round(100*performance[column], 2):.2f}"
//         change = change[1:] if change[0]=='-' else change
//         name = f"{symbol:<5}{sign}{change:>5}%"
//         add_trace(normalizedClosesDf[column], name, dict(width=2), True)
//     def add_empty(name):
//         add_trace([1]*len(normalizedClosesDf.index), name, dict(width=0), True)

//     # <plotting>
//     for column in normalizedClosesDf.columns:
//         if column not in top_performers and column not in bottom_performers and column != 'BTCUSDT':
//             add_trace(normalizedClosesDf[column], '', dict(width=1, color='grey'), False)
//     for column in top_performers:
//         add_performers(column)
//     add_trace(normalizedClosesDf['BTCUSDT'], f"~BTC~ {round(100*performance['BTCUSDT'], 2):>5}", dict(width=5, color='gold'), True)
//     for column in bottom_performers[::-1]:
//         add_performers(column)
//     add_empty('')
//     add_empty(f"V:  {variance:.5f}")
//     add_empty(f"K:  {round(kurtosis, 1)}")
//     add_empty(f"C:  {round(mean_correlation, 2)}")
//     add_empty(f"AV: {av_move*100:.5f}%")
//     # </plotting>

//     fig.update_layout(template='plotly_dark', autosize=True, margin=dict(l=0, r=0, b=0, t=0), font={"family":"Courier New, monospace"})
//     fig.update_xaxes(range=[normalizedClosesDf.index.min(), normalizedClosesDf.index.max()])
//     fig.update_yaxes(range=[normalizedClosesDf.min().min(), normalizedClosesDf.max().max()])

//     return fig
