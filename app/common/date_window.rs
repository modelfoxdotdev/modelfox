use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum DateWindow {
	Today,
	ThisMonth,
	ThisYear,
}

impl Display for DateWindow {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		match *self {
			DateWindow::Today => f.write_str("today"),
			DateWindow::ThisMonth => f.write_str("this_month"),
			DateWindow::ThisYear => f.write_str("this_year"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum DateWindowInterval {
	Hourly,
	Daily,
	Monthly,
}

pub fn get_date_window_and_interval(
	search_params: &Option<BTreeMap<String, String>>,
) -> Option<(DateWindow, DateWindowInterval)> {
	let date_window = search_params
		.as_ref()
		.and_then(|query| query.get("date_window"));
	let date_window = date_window.map_or("this_month", |dw| dw.as_str());
	let date_window = match date_window {
		"today" => DateWindow::Today,
		"this_month" => DateWindow::ThisMonth,
		"this_year" => DateWindow::ThisYear,
		_ => return None,
	};
	let date_window_interval = match date_window {
		DateWindow::Today => DateWindowInterval::Hourly,
		DateWindow::ThisMonth => DateWindowInterval::Daily,
		DateWindow::ThisYear => DateWindowInterval::Monthly,
	};
	Some((date_window, date_window_interval))
}
