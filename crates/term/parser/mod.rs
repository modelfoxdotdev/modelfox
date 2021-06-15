//! This module implements a parser for [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
//! The parser is based on [Paul Williams' ANSI-compatible video terminal parser](https://vt100.net/emu/dec_ansi_parser) modified to support UTF-8 input.
//! The implementation uses a static state transition table to minimize branches.
//!
//! ## Usage
//!
//! To read escape codes, create an [`ActionIter`] from any [`std::io::Read`] and consume the [`Action`]s the iterator returns.
//! This crate comes with a "logger" example that will print a description of the actions produced by stdin.
//! To give it a try, run the following:
//!
//! ```bash
//! echo -n "\x1b[30mhello\x1b[0m" | cargo run --example logger
//! ```
//!
//! Below is the source for the logger example, which demonstrates how to read escape codes:
//!
//! ```rust
//! let stdin = std::io::stdin();
//! let stdin = stdin.lock();
//! let stdin = std::io::BufReader::new(stdin);
//! let action_iter = term_parser::ActionIter::new(stdin);
//! for action in action_iter {
//!   println!("{:?}", action);
//! }
//! ```
//!
//! [`ActionIter`]: struct.ActionIter.html
//! [`Action`]: enum.Action.html
//! [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
//!

use error::ParseError;
use std::io::Read;

mod error;
mod table;

pub use error::Error;

const MAX_INTERMEDIATES: usize = 2;
const MAX_PARAMS: usize = 16;

/// An action, as described in [Paul Williams' ANSI-compatible video terminal parser](https://vt100.net/emu/dec_ansi_parser).
/// To prevent heap allocation, intermediate and param bytes use arrays instead of Vecs.
/// Be sure to only read `intermediates_count` bytes from `intermediates` and `params_count` bytes from `params`.
#[derive(Debug, PartialEq)]
pub enum Action {
	Csi {
		intermediates: [u8; MAX_INTERMEDIATES],
		intermediates_count: usize,
		params: [usize; MAX_PARAMS],
		params_count: usize,
		byte: u8,
	},
	Esc {
		intermediates: [u8; MAX_INTERMEDIATES],
		intermediates_count: usize,
		params: [usize; MAX_PARAMS],
		params_count: usize,
	},
	Execute(u8),
	Hook {
		intermediates: [u8; MAX_INTERMEDIATES],
		intermediates_count: usize,
		params: [usize; MAX_PARAMS],
		params_count: usize,
	},
	OscEnd,
	OscPut(u8),
	OscStart,
	Print(char),
	Put(u8),
	Unhook(u8),
}

/// An [`Iterator`] that returns [`Action`]s read from a [`std::io::Read`]er.
///
/// [`Iterator`]: https://doc.rust-lang.org/std/iter/index.html
/// [`Action`]: enum.Action.html
/// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
///
pub struct ActionIter<R>
where
	R: Read,
{
	bytes: std::io::Bytes<R>,
	intermediates: [u8; MAX_INTERMEDIATES],
	intermediates_count: usize,
	params: [usize; MAX_PARAMS],
	params_count: usize,
	state: table::State,
	table_actions_queue: [(table::Action, u8); 3],
	table_actions_count: usize,
}

impl<R> ActionIter<R>
where
	R: Read,
{
	/// Create a new ActionIter from a [`Read`]er.
	///
	/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
	///
	pub fn new(reader: R) -> Self {
		ActionIter {
			bytes: reader.bytes(),
			intermediates: [0u8; MAX_INTERMEDIATES],
			intermediates_count: 0,
			params: [0usize; MAX_PARAMS],
			params_count: 0,
			state: table::State::Ground,
			table_actions_queue: [(table::Action::None, 0u8); 3],
			table_actions_count: 0,
		}
	}
}

impl<R: Read> Iterator for ActionIter<R> {
	type Item = Result<Action, Error>;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// if the table action queue is empty,
			// read the next byte and add any
			// new table actions to the queue
			if self.table_actions_count == 0 {
				let byte = match self.bytes.next() {
					None => return None,
					Some(result) => match result {
						Err(err) => return Some(Err(err.into())),
						Ok(byte) => byte,
					},
				};
				let old_state = self.state;
				let (table_action, new_state) = table::query(old_state, byte);
				self.state = new_state;
				if new_state == table::State::Error {
					return Some(Err(Error::Parse(ParseError {
						state: old_state,
						byte,
					})));
				} else if new_state != old_state {
					let mut table_actions_count = 0;
					let exit_action = table::EXIT_ACTIONS[old_state as usize];
					if exit_action != table::Action::None {
						self.table_actions_queue[table_actions_count] = (exit_action, byte);
						table_actions_count += 1;
					}
					if table_action != table::Action::None {
						self.table_actions_queue[table_actions_count] = (table_action, byte);
						table_actions_count += 1;
					}
					let enter_action = table::ENTRY_ACTIONS[new_state as usize];
					if enter_action != table::Action::None {
						self.table_actions_queue[table_actions_count] = (enter_action, byte);
						table_actions_count += 1;
					}
					self.table_actions_count = table_actions_count;
				} else {
					self.table_actions_queue[0] = (table_action, byte);
					self.table_actions_count = 1;
				}
			}

			// pop a table action off the queue
			let (table_action, byte) = self.table_actions_queue[0];
			self.table_actions_count -= 1;
			self.table_actions_queue[0] = self.table_actions_queue[1];
			self.table_actions_queue[1] = self.table_actions_queue[2];

			// execute the table action and produce the parser action, if any
			let action = match table_action {
				table::Action::None => None,
				table::Action::Clear => {
					self.params_count = 0;
					self.intermediates_count = 0;
					None
				}
				table::Action::Collect => {
					if self.intermediates_count < MAX_INTERMEDIATES {
						self.intermediates[self.intermediates_count] = byte;
						self.intermediates_count += 1;
					}
					None
				}
				table::Action::CsiDispatch => {
					let intermediates = self.intermediates;
					let intermediates_count = self.intermediates_count;
					let params = self.params;
					let params_count = self.params_count;
					self.intermediates = [0; MAX_INTERMEDIATES];
					self.intermediates_count = 0;
					self.params = [0; MAX_PARAMS];
					self.params_count = 0;
					Some(Action::Csi {
						intermediates,
						intermediates_count,
						params,
						params_count,
						byte,
					})
				}
				table::Action::EscDispatch => {
					let intermediates = self.intermediates;
					let intermediates_count = self.intermediates_count;
					let params = self.params;
					let params_count = self.params_count;
					self.intermediates = [0; MAX_INTERMEDIATES];
					self.intermediates_count = 0;
					self.params = [0; MAX_PARAMS];
					self.params_count = 0;
					Some(Action::Esc {
						intermediates,
						intermediates_count,
						params,
						params_count,
					})
				}
				table::Action::Execute => Some(Action::Execute(byte)),
				table::Action::Hook => {
					let intermediates = self.intermediates;
					let intermediates_count = self.intermediates_count;
					let params = self.params;
					let params_count = self.params_count;
					self.intermediates = [0; MAX_INTERMEDIATES];
					self.intermediates_count = 0;
					self.params = [0; MAX_PARAMS];
					self.params_count = 0;
					Some(Action::Hook {
						intermediates,
						intermediates_count,
						params,
						params_count,
					})
				}
				table::Action::Ignore => None,
				table::Action::OscEnd => Some(Action::OscEnd),
				table::Action::OscPut => Some(Action::OscPut(byte)),
				table::Action::OscStart => Some(Action::OscStart),
				table::Action::Param => {
					if byte == b';' {
						self.params[self.params_count] = 0;
						self.params_count += 1;
					} else {
						if self.params_count == 0 {
							self.params[self.params_count] = 0;
							self.params_count = 1;
						}
						let param_index = self.params_count - 1;
						self.params[param_index] =
							self.params[param_index] * 10 + ((byte - b'0') as usize);
					}
					None
				}
				table::Action::Print => {
					let n_bytes = table::UTF8_CHAR_WIDTH[byte as usize] as usize;
					let mut bytes = [0u8; 4];
					bytes[0] = byte;
					for byte in &mut bytes[1..n_bytes] {
						*byte = match self.bytes.next() {
							None => return None,
							Some(result) => match result {
								Err(err) => return Some(Err(err.into())),
								Ok(byte) => byte,
							},
						};
					}
					let c = std::str::from_utf8(&bytes[0..n_bytes])
						.ok()?
						.chars()
						.next()?;
					Some(Action::Print(c))
				}
				table::Action::Put => Some(Action::Put(byte)),
				table::Action::Unhook => Some(Action::Unhook(byte)),
			};

			// return the parser action if any, otherwise loop again
			if let Some(result) = action {
				return Some(Ok(result));
			} else {
				continue;
			}
		}
	}
}

#[test]
fn test_text() {
	let mut action_iter = ActionIter::new("x😀yß".as_bytes());
	let actions = vec![
		Action::Print('x'),
		Action::Print('😀'),
		Action::Print('y'),
		Action::Print('ß'),
	];
	for action in actions.into_iter() {
		assert_eq!(action_iter.next().unwrap().unwrap(), action);
	}
	assert!(action_iter.next().is_none());
}

#[test]
fn test_csi() {
	let bytes = "\x1b[m\x1b[30mx\x1b[12;14H😀".as_bytes();
	let mut action_iter = ActionIter::new(bytes);
	let actions = vec![
		Action::Csi {
			intermediates: [0, 0],
			intermediates_count: 0,
			params: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			params_count: 0,
			byte: b'm',
		},
		Action::Csi {
			intermediates: [0, 0],
			intermediates_count: 0,
			params: [30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			params_count: 1,
			byte: b'm',
		},
		Action::Print('x'),
		Action::Csi {
			intermediates: [0, 0],
			intermediates_count: 0,
			params: [12, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			params_count: 2,
			byte: b'H',
		},
		Action::Print('😀'),
	];
	for action in actions.into_iter() {
		assert_eq!(action_iter.next().unwrap().unwrap(), action);
	}
	assert!(action_iter.next().is_none());
}

#[test]
fn test_osc() {
	let bytes: &[u8] = b"\x1b]hi\x9c";
	let mut action_iter = ActionIter::new(bytes);
	let actions = vec![
		Action::OscStart,
		Action::OscPut(b'h'),
		Action::OscPut(b'i'),
		Action::OscEnd,
	];
	for action in actions.into_iter() {
		assert_eq!(action_iter.next().unwrap().unwrap(), action);
	}
	assert!(action_iter.next().is_none());
}

#[test]
fn test_multiple_table_actions_per_byte() {
	let mut action_iter = ActionIter::new("\x1b\x50\x3f\x40\x1b\x5b\x39\x40🐶".as_bytes());
	let actions = vec![
		Action::Hook {
			intermediates: [0x3f, 0],
			intermediates_count: 1,
			params: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			params_count: 0,
		},
		Action::Unhook(0x1b),
		Action::Csi {
			intermediates: [0, 0],
			intermediates_count: 0,
			params: [9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			params_count: 1,
			byte: 0x40,
		},
		Action::Print('🐶'),
	];
	for action in actions.into_iter() {
		assert_eq!(action_iter.next().unwrap().unwrap(), action);
	}
	assert!(action_iter.next().is_none());
}
