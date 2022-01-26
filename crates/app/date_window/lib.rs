
use std::fmt::Display;

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum DateWindow {
	#[serde(rename = "today")]
	Today,
	#[serde(rename = "this_month")]
	ThisMonth,
	#[serde(rename = "this_year")]
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
	date_window: &Option<DateWindow>,
) -> Option<(DateWindow, DateWindowInterval)> {
	let date_window = date_window.unwrap_or(DateWindow::ThisMonth);
	let date_window_interval = match date_window {
		DateWindow::Today => DateWindowInterval::Hourly,
		DateWindow::ThisMonth => DateWindowInterval::Daily,
		DateWindow::ThisYear => DateWindowInterval::Monthly,
	};
	Some((date_window, date_window_interval))
}
