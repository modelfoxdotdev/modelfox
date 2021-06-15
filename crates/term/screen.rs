use crate::{
	event::Event,
	style::{Style, StyleMask},
	terminal::{Clear, Terminal},
};
use ndarray::prelude::*;
use num::ToPrimitive;
use std::io::{Result, Write};
use tangram_zip::zip;

pub struct Screen {
	terminal: Terminal,
	buffer: Array2<Cell>,
	cursor_hidden: bool,
}

#[derive(Debug, Clone)]
struct Cell {
	character: char,
	style: Style,
	dirty: bool,
}

impl Default for Cell {
	fn default() -> Self {
		Self {
			character: ' ',
			style: Style::default(),
			dirty: false,
		}
	}
}

impl Screen {
	/** Attempt to open a `Screen` if it is supported in the environment the application is running in. This will return `Err(_)` if the terminal is not supported. */
	pub fn open() -> Result<Self> {
		let mut terminal = Terminal::new()?;
		let (rows, cols) = terminal.size()?;
		let (rows, cols) = (rows.into(), cols.into());
		let buffer = Array::from_elem((rows, cols), Cell::default());
		terminal.enable_raw_mode()?;
		terminal.enable_mouse_events()?;
		terminal.show_alternate_screen()?;
		terminal.flush()?;
		Ok(Self {
			terminal,
			buffer,
			cursor_hidden: false,
		})
	}

	pub fn close(self) {}

	pub fn size(&self) -> (usize, usize) {
		(self.buffer.nrows(), self.buffer.ncols())
	}

	pub fn listen(&mut self) -> Result<Event> {
		self.terminal.listen()
	}

	pub fn hide_cursor(&mut self) -> Result<()> {
		self.terminal.hide_cursor()?;
		self.terminal.flush()?;
		self.cursor_hidden = true;
		Ok(())
	}

	pub fn put(&mut self, row: u16, col: u16, style: Style, character: char) {
		if let Some(cell) = self.buffer.get_mut((row.into(), col.into())) {
			if cell.style != style || cell.character != character {
				*cell = Cell {
					character,
					style,
					dirty: true,
				};
			}
		}
	}

	pub fn clear(&mut self) -> Result<()> {
		self.terminal.clear(Clear::EntireScreen)?;
		self.buffer.fill(Cell::default());
		Ok(())
	}

	pub fn clear_row(&mut self, row: u16) {
		for i in 0..self.buffer.ncols() {
			self.put(row, i.to_u16().unwrap(), Style::default(), ' ');
		}
	}

	pub fn put_str(&mut self, row: u16, col: u16, style: Style, string: &str) {
		for (i, character) in string.chars().enumerate() {
			self.put(row, col + i.to_u16().unwrap(), style.clone(), character);
		}
	}

	pub fn flush(&mut self) -> Result<()> {
		self.resize_if_necessary()?;
		self.terminal.save_cursor_position()?;
		if !self.cursor_hidden {
			self.terminal.hide_cursor()?;
		}
		let mut previous_cell_pos = None;
		for ((row, col), cell) in self.buffer.indexed_iter_mut() {
			if !cell.dirty {
				continue;
			}
			match previous_cell_pos {
				None => {
					self.terminal
						.set_cursor_position(row.to_u16().unwrap(), col.to_u16().unwrap())?;
				}
				Some((_, previous_col)) if previous_col + 1 != col => {
					self.terminal
						.set_cursor_position(row.to_u16().unwrap(), col.to_u16().unwrap())?;
				}
				_ => {}
			};
			self.terminal.reset_style()?;
			self.terminal
				.set_background_color(cell.style.background_color)?;
			self.terminal
				.set_foreground_color(cell.style.foreground_color)?;
			if cell.style.style_mask.contains(StyleMask::BOLD) {
				self.terminal.set_bold()?;
			}
			if cell.style.style_mask.contains(StyleMask::FAINT) {
				self.terminal.set_faint()?;
			}
			if cell.style.style_mask.contains(StyleMask::ITALIC) {
				self.terminal.set_italic()?;
			}
			if cell.style.style_mask.contains(StyleMask::UNDERLINE) {
				self.terminal.set_underline()?;
			}
			if cell.style.style_mask.contains(StyleMask::STRIKETHROUGH) {
				self.terminal.set_strikethrough()?;
			}
			write!(self.terminal, "{}", cell.character)?;
			cell.dirty = false;
			previous_cell_pos = Some((row, col));
		}
		self.terminal.restore_cursor_position()?;
		if !self.cursor_hidden {
			self.terminal.show_cursor()?;
		}
		self.terminal.flush()?;
		Ok(())
	}

	fn resize_if_necessary(&mut self) -> Result<()> {
		let (new_rows, new_cols) = self.terminal.size()?;
		let (new_rows, new_cols) = (new_rows.into(), new_cols.into());
		if self.buffer.nrows() != new_rows || self.buffer.ncols() != new_cols {
			// clear the terminal to minimize the amount of time something corrupted is shown
			self.terminal.clear(Clear::EntireScreen)?;
			self.terminal.flush()?;
			let old_rows = self.buffer.nrows();
			let old_cols = self.buffer.ncols();
			let mut new_buffer = Array::from_elem((new_rows, new_cols), Cell::default());
			let common_rows = new_rows.min(old_rows);
			let common_cols = new_cols.min(old_cols);
			for (new, old) in zip!(
				new_buffer.slice_mut(s![0..common_rows, 0..common_cols]),
				self.buffer.slice(s![0..common_rows, 0..common_cols]),
			) {
				*new = Cell {
					style: old.style.clone(),
					character: old.character,
					dirty: true,
				}
			}
			self.buffer = new_buffer;
		}
		Ok(())
	}
}

impl Drop for Screen {
	fn drop(&mut self) {
		self.terminal.hide_alternate_screen().unwrap();
		self.terminal.disable_mouse_events().unwrap();
		self.terminal.flush().unwrap();
	}
}
