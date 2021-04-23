use crate::event::{Event, MouseEvent};
use num::ToPrimitive;
use std::{io::Result, mem::MaybeUninit};
use winapi::{
	shared::minwindef::DWORD,
	um::consoleapi::{GetConsoleMode, ReadConsoleInputA, SetConsoleMode, WriteConsoleA},
	um::fileapi::{CreateFileA, OPEN_EXISTING},
	um::handleapi::INVALID_HANDLE_VALUE,
	um::wincon::{
		GetConsoleScreenBufferInfo, ENABLE_EXTENDED_FLAGS, ENABLE_MOUSE_INPUT,
		ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT,
		ENABLE_WRAP_AT_EOL_OUTPUT, INPUT_RECORD, KEY_EVENT, LEFT_CTRL_PRESSED, MOUSE_EVENT,
		MOUSE_MOVED, MOUSE_WHEELED, RIGHT_CTRL_PRESSED, WINDOW_BUFFER_SIZE_EVENT,
	},
	um::winnt::{
		CHAR, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE, VOID,
	},
};

pub struct Terminal {
	input_handle: HANDLE,
	output_handle: HANDLE,
	saved_console_mode: Option<DWORD>,
}

unsafe impl Send for Terminal {}

impl Terminal {
	pub fn new() -> Result<Self> {
		let input_handle = unsafe {
			CreateFileA(
				"CONIN$\0".as_ptr() as *const CHAR,
				GENERIC_READ | GENERIC_WRITE,
				FILE_SHARE_READ | FILE_SHARE_WRITE,
				std::ptr::null_mut(),
				OPEN_EXISTING,
				0,
				std::ptr::null_mut(),
			)
		};
		if input_handle == INVALID_HANDLE_VALUE {
			return Err(std::io::Error::last_os_error());
		}
		let output_handle = unsafe {
			CreateFileA(
				"CONOUT$\0".as_ptr() as *const CHAR,
				GENERIC_READ | GENERIC_WRITE,
				FILE_SHARE_READ | FILE_SHARE_WRITE,
				std::ptr::null_mut(),
				OPEN_EXISTING,
				0,
				std::ptr::null_mut(),
			)
		};
		if output_handle == INVALID_HANDLE_VALUE {
			return Err(std::io::Error::last_os_error());
		}
		let output_mode = ENABLE_PROCESSED_OUTPUT
			| ENABLE_WRAP_AT_EOL_OUTPUT
			| ENABLE_VIRTUAL_TERMINAL_PROCESSING;
		let err = unsafe { SetConsoleMode(output_handle, output_mode) };
		if err == 0 {
			return Err(std::io::Error::last_os_error());
		}
		Ok(Self {
			input_handle,
			output_handle,
			saved_console_mode: None,
		})
	}

	pub fn enable_raw_mode(&mut self) -> Result<()> {
		let mut saved_console_mode = MaybeUninit::uninit();
		let err = unsafe { GetConsoleMode(self.input_handle, saved_console_mode.as_mut_ptr()) };
		if err == 0 {
			return Err(std::io::Error::last_os_error());
		}
		let saved_console_mode = unsafe { saved_console_mode.assume_init() };
		let console_mode = ENABLE_MOUSE_INPUT | ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS;
		let err = unsafe { SetConsoleMode(self.input_handle, console_mode) };
		if err == 0 {
			return Err(std::io::Error::last_os_error());
		}
		self.saved_console_mode = Some(saved_console_mode);
		Ok(())
	}

	pub fn size(&self) -> Result<(u16, u16)> {
		unsafe {
			let mut screen_buffer_info = MaybeUninit::uninit();
			let err =
				GetConsoleScreenBufferInfo(self.output_handle, screen_buffer_info.as_mut_ptr());
			if err == 0 {
				return Err(std::io::Error::last_os_error());
			}
			let screen_buffer_info = screen_buffer_info.assume_init();
			let rows = screen_buffer_info.srWindow.Bottom - screen_buffer_info.srWindow.Top;
			let rows = rows.to_u16().unwrap();
			let cols = screen_buffer_info.srWindow.Right - screen_buffer_info.srWindow.Left;
			let cols = cols.to_u16().unwrap();
			Ok((rows, cols))
		}
	}

	pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		let mut n: DWORD = 0;
		let err = unsafe {
			WriteConsoleA(
				self.output_handle,
				buffer.as_ptr() as *const VOID,
				buffer.len() as DWORD,
				&mut n,
				std::ptr::null_mut(),
			)
		};
		if err == 0 {
			return Err(std::io::Error::last_os_error());
		}
		Ok(n.to_usize().unwrap())
	}

	pub fn listen(&mut self) -> Result<Event> {
		let mut event = MaybeUninit::uninit();
		let mut n_events_read = MaybeUninit::uninit();
		unsafe {
			ReadConsoleInputA(
				self.input_handle,
				event.as_mut_ptr(),
				1,
				n_events_read.as_mut_ptr(),
			)
		};
		let event: INPUT_RECORD = unsafe { event.assume_init() };
		let n_events_read: DWORD = unsafe { n_events_read.assume_init() };
		if n_events_read != 1 {
			return Err(std::io::Error::last_os_error());
		}
		match event.EventType {
			KEY_EVENT => {
				let key_event = unsafe { event.Event.KeyEvent() };
				let c = unsafe { *key_event.uChar.AsciiChar() } as u8 as char;
				if key_event.dwControlKeyState & LEFT_CTRL_PRESSED != 0
					|| key_event.dwControlKeyState & RIGHT_CTRL_PRESSED != 0
				{
					Ok(Event::Ctrl(c))
				} else {
					Ok(Event::Char(c))
				}
			}
			MOUSE_EVENT => {
				let mouse_event = unsafe { event.Event.MouseEvent() };
				let position = mouse_event.dwMousePosition;
				let (row, col) = (position.Y as u16, position.X as u16);
				match mouse_event.dwEventFlags {
					0 => {
						if mouse_event.dwButtonState > 0 {
							Ok(Event::Mouse(MouseEvent::Down(row, col)))
						} else {
							Ok(Event::Mouse(MouseEvent::Up(row, col)))
						}
					}
					MOUSE_MOVED => {
						if mouse_event.dwButtonState > 0 {
							Ok(Event::Mouse(MouseEvent::Drag(row, col)))
						} else {
							Ok(Event::Unknown)
						}
					}
					MOUSE_WHEELED => {
						if mouse_event.dwButtonState > 0 {
							Ok(Event::Mouse(MouseEvent::ScrollDown(row, col)))
						} else {
							Ok(Event::Mouse(MouseEvent::ScrollUp(row, col)))
						}
					}
					_ => Ok(Event::Unknown),
				}
			}
			WINDOW_BUFFER_SIZE_EVENT => {
				let window_event = unsafe { event.Event.WindowBufferSizeEvent() };
				let size = window_event.dwSize;
				let (rows, cols) = (size.Y as u16, size.X as u16);
				Ok(Event::Resize(rows, cols))
			}
			_ => Ok(Event::Unknown),
		}
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		if let Some(saved_console_mode) = self.saved_console_mode {
			let err = unsafe { SetConsoleMode(self.input_handle, saved_console_mode) };
			assert!(err != 0);
		}
	}
}
