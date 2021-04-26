#[derive(Debug, PartialEq)]
pub enum Event {
	Char(char),
	Ctrl(char),
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize(u16, u16),
	Unknown,
}

impl std::fmt::Display for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Event::Char(c) => write!(f, "Char: {}", c),
			Event::Ctrl(c) => write!(f, "Ctrl: {}", c),
			Event::Key(KeyEvent::ArrowDown) => write!(f, "Arrow Down"),
			Event::Key(KeyEvent::ArrowLeft) => write!(f, "Arrow Left"),
			Event::Key(KeyEvent::ArrowRight) => write!(f, "Arrow Right"),
			Event::Key(KeyEvent::ArrowUp) => write!(f, "Arrow Up"),
			Event::Key(KeyEvent::Backspace) => write!(f, "Backspace"),
			Event::Key(KeyEvent::Delete) => write!(f, "Delete"),
			Event::Key(KeyEvent::End) => write!(f, "End"),
			Event::Key(KeyEvent::Enter) => write!(f, "Enter"),
			Event::Key(KeyEvent::Escape) => write!(f, "Escape"),
			Event::Key(KeyEvent::F1) => write!(f, "F1"),
			Event::Key(KeyEvent::F10) => write!(f, "F10"),
			Event::Key(KeyEvent::F11) => write!(f, "F11"),
			Event::Key(KeyEvent::F12) => write!(f, "F12"),
			Event::Key(KeyEvent::F2) => write!(f, "F2"),
			Event::Key(KeyEvent::F3) => write!(f, "F3"),
			Event::Key(KeyEvent::F4) => write!(f, "F4"),
			Event::Key(KeyEvent::F5) => write!(f, "F5"),
			Event::Key(KeyEvent::F6) => write!(f, "F6"),
			Event::Key(KeyEvent::F7) => write!(f, "F7"),
			Event::Key(KeyEvent::F8) => write!(f, "F8"),
			Event::Key(KeyEvent::F9) => write!(f, "F9"),
			Event::Key(KeyEvent::Home) => write!(f, "Home"),
			Event::Key(KeyEvent::Insert) => write!(f, "Insert"),
			Event::Key(KeyEvent::PageDown) => write!(f, "PageDown"),
			Event::Key(KeyEvent::PageUp) => write!(f, "PageUp"),
			Event::Key(KeyEvent::Tab) => write!(f, "Tab"),
			Event::Mouse(MouseEvent::Down(r, c)) => write!(f, "Mouse Down: {}, {}", r, c),
			Event::Mouse(MouseEvent::Drag(r, c)) => write!(f, "Mouse Drag: {}, {}", r, c),
			Event::Mouse(MouseEvent::ScrollDown(r, c)) => write!(f, "Scroll Down: {}, {}", r, c),
			Event::Mouse(MouseEvent::ScrollUp(r, c)) => write!(f, "Scroll Up: {}, {}", r, c),
			Event::Mouse(MouseEvent::Up(r, c)) => write!(f, "Mouse Up: {}, {}", r, c),
			Event::Resize(r, c) => write!(f, "Resize: {} x {}", r, c),
			Event::Unknown => write!(f, "Unknown"),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum KeyEvent {
	ArrowDown,
	ArrowLeft,
	ArrowRight,
	ArrowUp,
	Backspace,
	Delete,
	End,
	Enter,
	Escape,
	F1,
	F10,
	F11,
	F12,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	Home,
	Insert,
	PageDown,
	PageUp,
	Tab,
}

#[derive(Debug, PartialEq)]
pub enum MouseEvent {
	Down(u16, u16),
	Drag(u16, u16),
	ScrollDown(u16, u16),
	ScrollUp(u16, u16),
	Up(u16, u16),
}
