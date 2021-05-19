use num::{Float, ToPrimitive};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum NumberFormatter {
	Float(FloatFormatter),
	Percent(PercentFormatter),
}

impl Default for NumberFormatter {
	fn default() -> Self {
		NumberFormatter::Float(Default::default())
	}
}

impl NumberFormatter {
	pub fn float_default() -> NumberFormatter {
		NumberFormatter::Float(FloatFormatter::default())
	}

	pub fn float(digits: u8) -> NumberFormatter {
		NumberFormatter::Float(FloatFormatter::new(digits))
	}

	pub fn percent_default() -> NumberFormatter {
		NumberFormatter::Percent(PercentFormatter::default())
	}

	pub fn percent(decimal_places: usize) -> NumberFormatter {
		NumberFormatter::Percent(PercentFormatter::new(decimal_places))
	}

	pub fn format<F>(&self, value: F) -> String
	where
		F: Float,
	{
		match self {
			NumberFormatter::Float(formatter) => formatter.format(value),
			NumberFormatter::Percent(formatter) => formatter.format(value),
		}
	}

	pub fn format_option<F>(&self, value: Option<F>) -> String
	where
		F: Float,
	{
		match value {
			Some(value) => self.format(value),
			None => "N/A".to_owned(),
		}
	}
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FloatFormatter {
	digits: u8,
}

impl Default for FloatFormatter {
	fn default() -> Self {
		FloatFormatter { digits: 6 }
	}
}

impl FloatFormatter {
	pub fn new(digits: u8) -> FloatFormatter {
		FloatFormatter { digits }
	}

	pub fn format<T: Float>(&self, value: T) -> String {
		let value = value.to_f64().unwrap();
		if value.is_nan() {
			return "NaN".to_owned();
		}
		if value.is_infinite() {
			if value.is_sign_positive() {
				return "inf".to_owned();
			} else {
				return "-inf".to_owned();
			}
		}
		if value == 0.0 || value == -0.0 {
			return "0".to_owned();
		}
		let digits = self.digits;
		let e = value.abs().log10().floor().to_i32().unwrap();
		let n = value / 10.0f64.powi(e);
		let (n, e) = if e.abs() as u8 >= digits {
			// Format the value with scientific notation.
			let digits = digits as usize - digits_before_decimal(n);
			let n = format!("{:.*}", digits, n);
			(n, Some(e))
		} else {
			// Format the value without scientific notation.
			let digits = digits as usize - digits_before_decimal(value);
			let n = format!("{:.*}", digits, value);
			(n, None)
		};
		let mut string = n;
		// If the string contains a decimal point, strip trailing zeros and a trailing decimal point if it is left behind.
		if string.contains('.') {
			string.truncate(string.trim_end_matches('0').len());
			string.truncate(string.trim_end_matches('.').len());
		}
		if let Some(e) = e {
			string.push('e');
			string.push_str(&format!("{}", e));
		}
		string
	}

	pub fn format_option<F>(&self, value: Option<F>) -> String
	where
		F: Float,
	{
		match value {
			Some(value) => self.format(value),
			None => "N/A".to_owned(),
		}
	}
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PercentFormatter {
	decimal_places: usize,
}

impl Default for PercentFormatter {
	fn default() -> Self {
		PercentFormatter { decimal_places: 2 }
	}
}

impl PercentFormatter {
	pub fn new(decimal_places: usize) -> PercentFormatter {
		PercentFormatter { decimal_places }
	}

	pub fn format<T: Float>(&self, value: T) -> String {
		if (value - T::one()).abs() <= T::epsilon() {
			"100%".to_owned()
		} else {
			format!(
				"{:.1$}%",
				value.to_f64().unwrap() * 100.0,
				self.decimal_places
			)
		}
	}

	pub fn format_option<F>(&self, value: Option<F>) -> String
	where
		F: Float,
	{
		match value {
			Some(value) => self.format(value),
			None => "N/A".to_owned(),
		}
	}
}

pub fn format_float<T: Float>(value: T) -> String {
	FloatFormatter::default().format(value)
}

pub fn format_float_with_digits<T: Float>(value: T, digits: u8) -> String {
	FloatFormatter::new(digits).format(value)
}

pub fn format_option_float<T: Float>(value: Option<T>) -> String {
	FloatFormatter::default().format_option(value)
}

pub fn format_percent<T: Float>(value: T) -> String {
	PercentFormatter::default().format(value)
}

pub fn format_option_percent<T: Float>(value: Option<T>) -> String {
	PercentFormatter::default().format_option(value)
}

#[test]
fn test_format_float() {
	fn test(x: f64, p: u8, s: &str) {
		assert_eq!(format_float_with_digits(x, p), s);
	}
	test(12345000.067, 3, "1.23e7");
	test(-12345000.067, 3, "-1.23e7");
	test(12345000.0, 3, "1.23e7");
	test(-12345000.0, 3, "-1.23e7");
	test(1234500.0, 3, "1.23e6");
	test(-1234500.0, 3, "-1.23e6");
	test(123450.0, 3, "1.23e5");
	test(-123450.0, 3, "-1.23e5");
	test(12345.0, 3, "1.23e4");
	test(-12345.0, 3, "-1.23e4");
	test(1234.5, 3, "1.23e3");
	test(-1234.5, 3, "-1.23e3");
	test(123.45, 3, "123");
	test(-123.45, 3, "-123");
	test(12.345, 3, "12.3");
	test(-12.345, 3, "-12.3");
	test(1.2345, 3, "1.23");
	test(-1.2345, 3, "-1.23");
	test(1.00, 3, "1");
	test(-1.00, 3, "-1");
	test(0.0, 3, "0");
	test(0.12345, 3, "0.123");
	test(-0.12345, 3, "-0.123");
	test(0.012345, 3, "0.012");
	test(-0.012345, 3, "-0.012");
	test(0.0012345, 3, "1.23e-3");
	test(-0.0012345, 3, "-1.23e-3");
	test(0.00012345, 3, "1.23e-4");
	test(-0.00012345, 3, "-1.23e-4");
}

#[test]
fn test_format_percent() {
	assert_eq!(format_percent(0.0), "0.00%");
	assert_eq!(format_percent(0.42), "42.00%");
	assert_eq!(format_percent(0.424249), "42.42%");
	assert_eq!(format_percent(0.424250), "42.43%");
	assert_eq!(format_percent(1.00), "100%");
}

fn digits_before_decimal(value: f64) -> usize {
	let value = value.trunc().abs();
	if value != 0.0 {
		value.log10().floor().to_usize().unwrap() + 1
	} else {
		0
	}
}

#[test]
fn test_digits_before_decimal() {
	assert_eq!(digits_before_decimal(12345.0), 5);
	assert_eq!(digits_before_decimal(1234.5), 4);
	assert_eq!(digits_before_decimal(123.45), 3);
	assert_eq!(digits_before_decimal(12.345), 2);
	assert_eq!(digits_before_decimal(1.2345), 1);
	assert_eq!(digits_before_decimal(0.12345), 0);
}
