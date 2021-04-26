use crate::event::{Event, KeyEvent, MouseEvent};
use libc::{c_char, c_int, c_void};
use num::ToPrimitive;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{io::Result, mem::MaybeUninit};

static mut SIGWINCH_PIPE: [c_int; 2] = [0, 0];

fn sigwinch_handler(_sig: u32) {
	let dummy = 0u8;
	unsafe { libc::write(SIGWINCH_PIPE[1], &dummy as *const u8 as *const c_void, 1) };
}

pub struct Terminal {
	fd_set: Option<libc::fd_set>,
	saved_termios: Option<libc::termios>,
	tty_fd: c_int,
}

impl Terminal {
	pub fn new() -> Result<Self> {
		let tty_fd = unsafe { libc::open("/dev/tty\0".as_ptr() as *const c_char, libc::O_RDWR) };
		if tty_fd == -1 {
			return Err(std::io::Error::last_os_error());
		}
		Ok(Self {
			fd_set: None,
			saved_termios: None,
			tty_fd,
		})
	}

	pub fn size(&self) -> Result<(u16, u16)> {
		let mut sz = std::mem::MaybeUninit::uninit();
		let err = unsafe { libc::ioctl(self.tty_fd, libc::TIOCGWINSZ, sz.as_mut_ptr()) };
		if err != 0 {
			return Err(std::io::Error::last_os_error());
		}
		let sz: libc::winsize = unsafe { sz.assume_init() };
		Ok((sz.ws_row, sz.ws_col))
	}

	pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		let n = unsafe { libc::write(self.tty_fd, buffer.as_ptr() as *const c_void, buffer.len()) };
		if n < 0 {
			return Err(std::io::Error::last_os_error());
		}
		Ok(n.to_usize().unwrap())
	}

	pub fn enable_raw_mode(&mut self) -> Result<()> {
		let tty_fd = self.tty_fd;
		// Save the current termios.
		let mut saved_termios = MaybeUninit::uninit();
		let err = unsafe { libc::tcgetattr(tty_fd, saved_termios.as_mut_ptr()) };
		if err != 0 {
			return Err(std::io::Error::last_os_error());
		}
		let saved_termios: libc::termios = unsafe { saved_termios.assume_init() };
		// Create a file descriptor set that contains the tty and sigwinch file descriptors.
		let mut fd_set = MaybeUninit::uninit();
		unsafe { libc::FD_ZERO(fd_set.as_mut_ptr()) };
		let mut fd_set = unsafe { fd_set.assume_init() };
		// Create a pipe for the window size change handler to write to.
		let err = unsafe { libc::pipe(SIGWINCH_PIPE.as_mut_ptr()) };
		if err != 0 {
			return Err(std::io::Error::last_os_error());
		}
		if tty_fd > unsafe { SIGWINCH_PIPE[0] } {
			unsafe { libc::FD_SET(SIGWINCH_PIPE[0], &mut fd_set) };
			unsafe { libc::FD_SET(tty_fd, &mut fd_set) };
		} else {
			unsafe { libc::FD_SET(tty_fd, &mut fd_set) };
			unsafe { libc::FD_SET(SIGWINCH_PIPE[0], &mut fd_set) };
		}
		// Register the window change handler.
		let res =
			unsafe { libc::signal(libc::SIGWINCH, sigwinch_handler as *const c_void as usize) };
		if res == libc::SIG_ERR {
			return Err(std::io::Error::last_os_error());
		}
		// Create the new termios with raw mode enabled.
		let mut new_termios = MaybeUninit::uninit();
		let err = unsafe { libc::tcgetattr(tty_fd, new_termios.as_mut_ptr()) };
		if err != 0 {
			return Err(std::io::Error::last_os_error());
		}
		let mut new_termios: libc::termios = unsafe { new_termios.assume_init() };
		unsafe { libc::cfmakeraw(&mut new_termios) };
		// Set the new termios.
		let err = unsafe { libc::tcsetattr(tty_fd, libc::TCSAFLUSH, &new_termios) };
		if err != 0 {
			return Err(std::io::Error::last_os_error());
		}
		self.saved_termios = Some(saved_termios);
		self.fd_set = Some(fd_set);
		Ok(())
	}

	pub fn listen(&mut self) -> Result<Event> {
		let fd_set = self.fd_set.as_mut().unwrap();
		// Wait for data to be ready on one of the file descriptors.
		unsafe {
			libc::select(
				i32::max(self.tty_fd, SIGWINCH_PIPE[0]),
				fd_set,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
			)
		};
		// Check which file descriptor has data ready and return the appropriate event.
		if unsafe { libc::FD_ISSET(SIGWINCH_PIPE[0], fd_set) } {
			let size = self.size()?;
			Ok(Event::Resize(size.0, size.1))
		} else if unsafe { libc::FD_ISSET(self.tty_fd, fd_set) } {
			let mut bytes = [0u8; 32];
			let n_bytes_read =
				unsafe { libc::read(self.tty_fd, bytes.as_mut_ptr() as *mut c_void, 32) };
			if n_bytes_read < 0 {
				Err(std::io::Error::last_os_error())
			} else {
				Ok(parse_event(&bytes[..n_bytes_read as usize]))
			}
		} else {
			Ok(Event::Unknown)
		}
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		// Restore the previously saved termios if any.
		if let Some(saved_termios) = self.saved_termios {
			let err = unsafe { libc::tcsetattr(self.tty_fd, libc::TCSAFLUSH, &saved_termios) };
			assert!(err == 0);
		}
		// Close the tty file.
		let err = unsafe { libc::close(self.tty_fd) };
		assert!(err == 0);
	}
}

static MOUSE_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r"\x1b\[(<?)([0-9]+);([0-9]+);([0-9]+)(M|m)").unwrap());

fn parse_event(bytes: &[u8]) -> Event {
	let s = std::str::from_utf8(bytes).unwrap();
	// mouse event
	if MOUSE_REGEX.is_match(s) {
		let captures = MOUSE_REGEX.captures(s).unwrap();
		let col = captures[3].parse::<u16>().unwrap() - 1;
		let row = captures[4].parse::<u16>().unwrap() - 1;
		return match (&captures[1], &captures[2], &captures[5]) {
			("<", "0", "m") => Event::Mouse(MouseEvent::Up(row, col)),
			("<", "0", "M") => Event::Mouse(MouseEvent::Down(row, col)),
			("<", "32", "M") => Event::Mouse(MouseEvent::Drag(row, col)),
			("<", "64", "M") => Event::Mouse(MouseEvent::ScrollUp(row, col)),
			("<", "65", "M") => Event::Mouse(MouseEvent::ScrollDown(row, col)),
			_ => Event::Unknown,
		};
	}
	// single byte key
	if bytes.len() == 1 {
		match bytes[0] {
			b'\n' | b'\r' => return Event::Key(KeyEvent::Enter),
			b'\x1b' => return Event::Key(KeyEvent::Escape),
			b'\t' => return Event::Key(KeyEvent::Tab),
			b'\x7F' => return Event::Key(KeyEvent::Backspace),
			byte @ b'\x01'..=b'\x1A' => return Event::Ctrl((byte as u8 - 0x1 + b'a') as char),
			byte @ b'\x1C'..=b'\x1F' => return Event::Ctrl((byte as u8 - 0x1C + b'4') as char),
			_ => {}
		};
	}
	// escaped key
	if bytes[0] == b'\x1b' {
		match s {
			"\x1bOP" => return Event::Key(KeyEvent::F1),
			"\x1bOQ" => return Event::Key(KeyEvent::F2),
			"\x1bOR" => return Event::Key(KeyEvent::F3),
			"\x1bOS" => return Event::Key(KeyEvent::F4),
			"\x1b[15~" => return Event::Key(KeyEvent::F5),
			"\x1b[17~" => return Event::Key(KeyEvent::F6),
			"\x1b[18~" => return Event::Key(KeyEvent::F7),
			"\x1b[19~" => return Event::Key(KeyEvent::F8),
			"\x1b[20~" => return Event::Key(KeyEvent::F9),
			"\x1b[21~" => return Event::Key(KeyEvent::F10),
			"\x1b[23~" => return Event::Key(KeyEvent::F11),
			"\x1b[24~" => return Event::Key(KeyEvent::F12),
			"\x1b[A" => return Event::Key(KeyEvent::ArrowUp),
			"\x1b[B" => return Event::Key(KeyEvent::ArrowDown),
			"\x1b[C" => return Event::Key(KeyEvent::ArrowRight),
			"\x1b[D" => return Event::Key(KeyEvent::ArrowLeft),
			_ => {}
		};
	}
	// character
	if let Some(c) = s.chars().next() {
		return Event::Char(c);
	}
	Event::Unknown
}
