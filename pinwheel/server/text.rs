use crate::text_value::{IntoTextValue, TextValue};
use futures::executor::block_on;
use futures_signals::signal::{Signal, SignalExt};
use std::fmt::Write;

pub fn text(text: impl IntoTextValue) -> Text {
	Text::new(text)
}

pub struct Text {
	value: TextValue,
}

impl Text {
	pub fn new(value: impl IntoTextValue) -> Text {
		Text {
			value: value.into_text_value(),
		}
	}

	pub fn new_signal<T, S>(value: S) -> Text
	where
		T: IntoTextValue,
		S: 'static + Unpin + Signal<Item = T>,
	{
		Text {
			value: block_on(value.first().to_future()).into_text_value(),
		}
	}
}

impl std::fmt::Display for Text {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", EscapedText(&self.value.0))
	}
}

pub(crate) struct EscapedText<'a>(pub(crate) &'a str);

impl<'a> std::fmt::Display for EscapedText<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for c in self.0.chars() {
			match c {
				'>' => write!(f, "&gt;")?,
				'<' => write!(f, "&lt;")?,
				'"' => write!(f, "&quot;")?,
				'&' => write!(f, "&amp;")?,
				'\'' => write!(f, "&apos;")?,
				c => f.write_char(c)?,
			}
		}
		Ok(())
	}
}
