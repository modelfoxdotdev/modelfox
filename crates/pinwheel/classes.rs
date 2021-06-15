use std::borrow::Cow;

pub struct Class<'a>(pub Option<Cow<'a, str>>);

impl<'a> From<Option<Cow<'a, str>>> for Class<'a> {
	fn from(option: Option<Cow<'a, str>>) -> Self {
		Class(option)
	}
}

impl<'a> From<Option<String>> for Class<'a> {
	fn from(option: Option<String>) -> Self {
		Class(option.map(Cow::Owned))
	}
}

impl<'a> From<Option<&'a str>> for Class<'a> {
	fn from(option: Option<&'a str>) -> Self {
		Class(option.map(|string| Cow::Borrowed(string)))
	}
}

impl<'a> From<Cow<'a, str>> for Class<'a> {
	fn from(string: Cow<'a, str>) -> Self {
		Class(Some(string))
	}
}

impl<'a> From<String> for Class<'a> {
	fn from(string: String) -> Self {
		Class(Some(Cow::Owned(string)))
	}
}

impl<'a> From<&'a String> for Class<'a> {
	fn from(string: &'a String) -> Self {
		Class(Some(Cow::Borrowed(string)))
	}
}

impl<'a> From<&'a str> for Class<'a> {
	fn from(string: &'a str) -> Self {
		Class(Some(Cow::Borrowed(string)))
	}
}

#[macro_export]
macro_rules! classes {
	($($class:expr),*$(,)?) => {
		{
			let mut first = true;
			let mut classes = String::new();
			$(
				let class = pinwheel::classes::Class::from($class);
				if let Some(class) = class.0 {
					if first {
						first = false;
					} else {
						classes.push(' ');
					}
					classes.push_str(class.as_ref());
				}
			)*
			classes
		}
	};
}
