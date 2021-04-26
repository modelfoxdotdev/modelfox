use bitflags::bitflags;

#[derive(Debug, PartialEq, Clone)]
pub struct Style {
	pub background_color: Color,
	pub foreground_color: Color,
	pub style_mask: StyleMask,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
	Default,
	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Magenta,
	Cyan,
	White,
	BrightBlack,
	BrightRed,
	BrightGreen,
	BrightYellow,
	BrightBlue,
	BrightMagenta,
	BrightCyan,
	BrightWhite,
	Rgb(u8, u8, u8),
}

bitflags! {
	pub struct StyleMask: u8 {
		const NORMAL        = 0b0000_0000;
		const BOLD          = 0b0000_0001;
		const FAINT         = 0b0000_0010;
		const ITALIC        = 0b0000_0100;
		const UNDERLINE     = 0b0000_1000;
		const STRIKETHROUGH = 0b0001_0000;
	}
}

impl Default for Style {
	fn default() -> Self {
		Self {
			background_color: Color::Default,
			foreground_color: Color::Default,
			style_mask: StyleMask::NORMAL,
		}
	}
}
