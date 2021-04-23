use std::collections::BTreeMap;

pub struct ParseCookiesError;

pub fn parse_cookies(cookies_str: &str) -> Result<BTreeMap<&str, &str>, ParseCookiesError> {
	cookies_str
		.split("; ")
		.map(|cookie| {
			let mut components = cookie.split('=');
			let key = match components.next() {
				Some(key) => key,
				None => return Err(ParseCookiesError),
			};
			let value = match components.next() {
				Some(value) => value,
				None => return Err(ParseCookiesError),
			};
			Ok((key, value))
		})
		.collect()
}
