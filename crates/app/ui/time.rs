use crate::date_window::{DateWindow, DateWindowInterval};

pub fn format_date_window(
	date: time::OffsetDateTime,
	date_window: &DateWindow,
	offset: time::UtcOffset,
) -> String {
	let date = date.to_offset(offset);
	match date_window {
		DateWindow::Today => format_day(date),
		DateWindow::ThisMonth => format_month(date),
		DateWindow::ThisYear => format_year(date),
	}
}

pub fn format_date_window_interval(
	date: time::OffsetDateTime,
	date_window_interval: &DateWindowInterval,
	offset: time::UtcOffset,
) -> String {
	let date = date.to_offset(offset);
	match date_window_interval {
		DateWindowInterval::Hourly => format_hour(date),
		DateWindowInterval::Daily => format_day_of_month(date),
		DateWindowInterval::Monthly => format_month(date),
	}
}

pub fn interval_chart_title(date_window_interval: &DateWindowInterval, title: String) -> String {
	match date_window_interval {
		DateWindowInterval::Hourly => format!("Hourly {}", title),
		DateWindowInterval::Daily => format!("Daily {}", title),
		DateWindowInterval::Monthly => format!("Monthly {}", title),
	}
}

pub fn overall_chart_title(date_window: &DateWindow, title: String) -> String {
	match date_window {
		DateWindow::Today => format!("Today's {}", title),
		DateWindow::ThisMonth => format!("This Month's {}", title),
		DateWindow::ThisYear => format!("This Year's {}", title),
	}
}

pub fn format_hour(date: time::OffsetDateTime) -> String {
	let format = time::format_description::parse("%-l%P").unwrap();
	date.format(&format).unwrap()
}

pub fn format_day(date: time::OffsetDateTime) -> String {
	let format = time::format_description::parse("%a %b %d %Y").unwrap();
	date.format(&format).unwrap()
}

pub fn format_day_of_month(date: time::OffsetDateTime) -> String {
	let format = time::format_description::parse("%b %d").unwrap();
	date.format(&format).unwrap()
}

pub fn format_month(date: time::OffsetDateTime) -> String {
	let format = time::format_description::parse("%b %Y").unwrap();
	date.format(&format).unwrap()
}

pub fn format_year(date: time::OffsetDateTime) -> String {
	let format = time::format_description::parse("%Y").unwrap();
	date.format(&format).unwrap()
}
