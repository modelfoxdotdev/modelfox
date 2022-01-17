use super::cookies::parse_cookies;

pub fn get_timezone(request: &http::Request<hyper::Body>) -> time::UtcOffset {
	request
		.headers()
		.get(http::header::COOKIE)
		.and_then(|cookie_header_value| cookie_header_value.to_str().ok())
		.and_then(|cookie_header_value| parse_cookies(cookie_header_value).ok())
		.and_then(|cookies| cookies.get("tangram_timezone").cloned())
		.and_then(|timezone_str| timezone_str.parse().ok())
		.unwrap_or(time::UtcOffset::UTC)
}
