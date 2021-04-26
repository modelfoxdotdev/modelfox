use crate::{event::Event, style::Color};
use std::{
	borrow::Cow,
	io::{Result, Write},
};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
type TerminalInner = self::unix::Terminal;
#[cfg(windows)]
type TerminalInner = self::windows::Terminal;

#[derive(Clone, Copy)]
pub enum Clear {
	FromCursorToEndOfLine,
	FromCursorToBeginningOfLine,
	EntireLine,
	FromCursorToEndOfScreen,
	FromCursorToBeginningOfScreen,
	EntireScreen,
}

const BUFFER_SIZE: usize = 8096;

pub struct Terminal {
	buffer: Vec<u8>,
	inner: TerminalInner,
	cursor_hidden: bool,
}

impl Terminal {
	/// create a new terminal
	pub fn new() -> Result<Self> {
		let buffer = Vec::with_capacity(BUFFER_SIZE);
		let inner = TerminalInner::new()?;
		Ok(Self {
			cursor_hidden: false,
			buffer,
			inner,
		})
	}

	/// get the current size of the terminal as (rows, cols)
	pub fn size(&self) -> Result<(u16, u16)> {
		self.inner.size()
	}

	/// enable raw mode, required to call `.listen()`
	pub fn enable_raw_mode(&mut self) -> Result<()> {
		self.inner.enable_raw_mode()
	}

	pub fn listen(&mut self) -> Result<Event> {
		self.inner.listen()
	}

	pub fn show_alternate_screen(&mut self) -> Result<()> {
		write!(self, "\x1b[?1049h")
	}

	pub fn hide_alternate_screen(&mut self) -> Result<()> {
		write!(self, "\x1b[?1049l")
	}

	pub fn clear(&mut self, clear_type: Clear) -> Result<()> {
		match clear_type {
			Clear::FromCursorToEndOfLine => write!(self, "\x1b[0K"),
			Clear::FromCursorToBeginningOfLine => write!(self, "\x1b[1K"),
			Clear::EntireLine => write!(self, "\x1b[2K"),
			Clear::FromCursorToEndOfScreen => write!(self, "\x1b[0J"),
			Clear::FromCursorToBeginningOfScreen => write!(self, "\x1b[1J"),
			Clear::EntireScreen => write!(self, "\x1b[2J"),
		}
	}

	pub fn show_cursor(&mut self) -> Result<()> {
		self.cursor_hidden = false;
		write!(self, "\x1b[?25h")
	}

	pub fn hide_cursor(&mut self) -> Result<()> {
		self.cursor_hidden = true;
		write!(self, "\x1b[?25l")
	}

	pub fn cursor_down(&mut self, rows: usize) -> Result<()> {
		write!(self, "\x1b[{}E", rows)
	}

	pub fn cursor_up(&mut self, rows: usize) -> Result<()> {
		write!(self, "\x1b[{}F", rows)
	}

	pub fn set_cursor_position(&mut self, row: u16, col: u16) -> Result<()> {
		write!(self, "\x1b[{};{}H", row + 1, col + 1)
	}

	pub fn enable_mouse_events(&mut self) -> Result<()> {
		write!(self, "\x1b[?1000h\x1b[?1006h\x1b[?1002h")
	}

	pub fn disable_mouse_events(&mut self) -> Result<()> {
		write!(self, "\x1b[?1002l\x1b[?1006l\x1b[?1000l")
	}

	pub fn save_cursor_position(&mut self) -> Result<()> {
		write!(self, "\x1b[s")
	}

	pub fn restore_cursor_position(&mut self) -> Result<()> {
		write!(self, "\x1b[u")
	}

	pub fn reset_style(&mut self) -> Result<()> {
		write!(self, "\x1b[0m")
	}

	pub fn set_background_color(&mut self, color: Color) -> Result<()> {
		let code: Cow<str> = match color {
			Color::Default => "49".into(),
			Color::Black => "40".into(),
			Color::Red => "41".into(),
			Color::Green => "42".into(),
			Color::Yellow => "43".into(),
			Color::Blue => "44".into(),
			Color::Magenta => "45".into(),
			Color::Cyan => "46".into(),
			Color::White => "47".into(),
			Color::BrightBlack => "100".into(),
			Color::BrightRed => "101".into(),
			Color::BrightGreen => "102".into(),
			Color::BrightYellow => "103".into(),
			Color::BrightBlue => "104".into(),
			Color::BrightMagenta => "105".into(),
			Color::BrightCyan => "106".into(),
			Color::BrightWhite => "107".into(),
			Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b).into(),
		};
		write!(self, "\x1b[{}m", code)
	}

	pub fn set_foreground_color(&mut self, color: Color) -> Result<()> {
		let code: Cow<str> = match color {
			Color::Default => "39".into(),
			Color::Black => "30".into(),
			Color::Red => "31".into(),
			Color::Green => "32".into(),
			Color::Yellow => "33".into(),
			Color::Blue => "34".into(),
			Color::Magenta => "35".into(),
			Color::Cyan => "36".into(),
			Color::White => "37".into(),
			Color::BrightBlack => "90".into(),
			Color::BrightRed => "91".into(),
			Color::BrightGreen => "92".into(),
			Color::BrightYellow => "93".into(),
			Color::BrightBlue => "94".into(),
			Color::BrightMagenta => "95".into(),
			Color::BrightCyan => "96".into(),
			Color::BrightWhite => "97".into(),
			Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b).into(),
		};
		write!(self, "\x1b[{}m", code)
	}

	pub fn set_bold(&mut self) -> Result<()> {
		write!(self, "\x1b[1m")
	}

	pub fn set_faint(&mut self) -> Result<()> {
		write!(self, "\x1b[2m")
	}

	pub fn set_italic(&mut self) -> Result<()> {
		write!(self, "\x1b[3m")
	}

	pub fn set_underline(&mut self) -> Result<()> {
		write!(self, "\x1b[4m")
	}

	pub fn set_strikethrough(&mut self) -> Result<()> {
		write!(self, "\x1b[9m")
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		if self.cursor_hidden {
			self.show_cursor().unwrap();
			self.flush().unwrap();
		}
	}
}

impl std::io::Write for Terminal {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.buffer.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		while !self.buffer.is_empty() {
			let n = self.inner.write(self.buffer.as_slice())?;
			self.buffer = self.buffer.split_off(n);
		}
		Ok(())
	}
}
