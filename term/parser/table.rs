use std::mem::transmute;

pub fn query(from_state: State, byte: u8) -> (Action, State) {
	let table_value = TABLE[from_state as usize][byte as usize];
	let to_action: Action = unsafe { transmute((table_value & 0xF0) >> 4) };
	let to_state: State = unsafe { transmute(table_value & 0x0F) };
	(to_action, to_state)
}

const TABLE: [[u8; 256]; 16] = build_state_table();

const fn build_state_table() -> [[u8; 256]; 16] {
	let mut table = [[0u8; 256]; 16];
	let mut state_index = 0;
	while state_index < STATES.len() {
		let from_state = STATES[state_index];
		let mut i: usize = 0;
		while i <= 0xFF {
			let (to_action, to_state) = table_fn(from_state, i as u8);
			let value = ((to_action as u8) << 4) | (to_state as u8);
			table[from_state as usize][i] = value;
			i += 1;
		}
		state_index += 1;
	}
	table
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum State {
	CsiEntry = 0,
	CsiIgnore = 1,
	CsiIntermediate = 2,
	CsiParam = 3,
	DcsEntry = 4,
	DcsIgnore = 5,
	DcsIntermediate = 6,
	DcsParam = 7,
	DcsPassthrough = 8,
	Error = 9,
	Escape = 10,
	EscapeIntermediate = 11,
	Ground = 12,
	OscString = 13,
	SosPmApcString = 14,
}

const STATES: &[State] = &[
	State::CsiEntry,
	State::CsiIgnore,
	State::CsiIntermediate,
	State::CsiParam,
	State::DcsEntry,
	State::DcsIgnore,
	State::DcsIntermediate,
	State::DcsParam,
	State::DcsPassthrough,
	State::Error,
	State::Escape,
	State::EscapeIntermediate,
	State::Ground,
	State::OscString,
	State::SosPmApcString,
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Action {
	None = 0,
	Clear = 1,
	Collect = 2,
	CsiDispatch = 3,
	EscDispatch = 4,
	Execute = 5,
	Hook = 6,
	Ignore = 7,
	OscEnd = 8,
	OscPut = 9,
	OscStart = 10,
	Param = 11,
	Print = 12,
	Put = 13,
	Unhook = 14,
}

pub static ENTRY_ACTIONS: &[Action] = &[
	Action::Clear,    // State::CsiEntry
	Action::None,     // State::CsiIgnore
	Action::None,     // State::CsiIntermediate
	Action::None,     // State::CsiParam
	Action::Clear,    // State::DcsEntry
	Action::None,     // State::DcsIgnore
	Action::None,     // State::DcsIntermediate
	Action::None,     // State::DcsParam
	Action::Hook,     // State::DcsPassthrough
	Action::None,     // State::Error
	Action::Clear,    // State::Escape
	Action::None,     // State::EscapeIntermediate
	Action::None,     // State::Ground
	Action::OscStart, // State::OscString
	Action::None,     // State::SosPmApcString
];

pub static EXIT_ACTIONS: &[Action] = &[
	Action::None,   // State::CsiEntry
	Action::None,   // State::CsiIgnore
	Action::None,   // State::CsiIntermediate
	Action::None,   // State::CsiParam
	Action::None,   // State::DcsEntry
	Action::None,   // State::DcsIgnore
	Action::None,   // State::DcsIntermediate
	Action::None,   // State::DcsParam
	Action::Unhook, // State::DcsPassthrough
	Action::None,   // State::Error
	Action::None,   // State::Escape
	Action::None,   // State::EscapeIntermediate
	Action::None,   // State::Ground
	Action::OscEnd, // State::OscString
	Action::None,   // State::SosPmApcString
];

#[rustfmt::skip]
pub static UTF8_CHAR_WIDTH: [u8; 256] = [
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x0F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x2F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x4F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x6F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x8F
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xAF
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
	0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xCF
	2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
	3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
	4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

// https://vt100.net/emu/dec_ansi_parser
const fn table_fn(state: State, byte: u8) -> (Action, State) {
	match (state, byte) {
		(_, 0x18) => (Action::Execute, State::Ground),
		(_, 0x1A) => (Action::Execute, State::Ground),
		(_, 0x80..=0x8F) => (Action::Execute, State::Ground),
		(_, 0x91..=0x97) => (Action::Execute, State::Ground),
		(_, 0x99) => (Action::Execute, State::Ground),
		(_, 0x9A) => (Action::Execute, State::Ground),

		(_, 0x9C) => (Action::None, State::Ground),
		(_, 0x1B) => (Action::None, State::Escape),

		(_, 0x98) => (Action::None, State::SosPmApcString),
		(_, 0x9E) => (Action::None, State::SosPmApcString),
		(_, 0x9F) => (Action::None, State::SosPmApcString),

		(_, 0x90) => (Action::None, State::DcsEntry),
		(_, 0x9D) => (Action::None, State::OscString),
		(_, 0x9B) => (Action::None, State::CsiEntry),

		(State::CsiEntry, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::CsiEntry),
			0x19 => (Action::Execute, State::CsiEntry),
			0x1C..=0x1F => (Action::Execute, State::CsiEntry),
			0x7F => (Action::Ignore, State::CsiEntry),

			0x30..=0x39 => (Action::Param, State::CsiParam),
			0x3C..=0x3F => (Action::Collect, State::CsiParam),

			0x3A => (Action::None, State::CsiIgnore),

			0x20..=0x2F => (Action::Collect, State::CsiIntermediate),

			0x40..=0x7E => (Action::CsiDispatch, State::Ground),

			_ => (Action::None, State::Error),
		},
		(State::CsiIgnore, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::CsiIgnore),
			0x19 => (Action::Execute, State::CsiIgnore),
			0x1C..=0x1F => (Action::Execute, State::CsiIgnore),
			0x20..=0x3F => (Action::Ignore, State::CsiIgnore),
			0x7F => (Action::Ignore, State::CsiIgnore),

			0x40..=0x7E => (Action::None, State::Ground),

			_ => (Action::None, State::Error),
		},
		(State::CsiIntermediate, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::CsiIntermediate),
			0x19 => (Action::Execute, State::CsiIntermediate),
			0x1C..=0x1F => (Action::Execute, State::CsiIntermediate),
			0x20..=0x2F => (Action::Collect, State::CsiIntermediate),
			0x7F => (Action::Ignore, State::CsiIntermediate),

			0x40..=0x7E => (Action::CsiDispatch, State::Ground),

			_ => (Action::None, State::Error),
		},
		(State::CsiParam, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::CsiParam),
			0x19 => (Action::Execute, State::CsiParam),
			0x1C..=0x1F => (Action::Execute, State::CsiParam),
			0x30..=0x39 => (Action::Param, State::CsiParam),
			0x3B => (Action::Param, State::CsiParam),
			0x7F => (Action::Ignore, State::CsiParam),

			0x40..=0x7E => (Action::CsiDispatch, State::Ground),

			0x20..=0x2F => (Action::Collect, State::CsiIntermediate),

			0x3A => (Action::None, State::CsiIgnore),
			0x3C..=0x3F => (Action::None, State::CsiIgnore),

			_ => (Action::None, State::Error),
		},
		(State::DcsEntry, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::DcsEntry),
			0x19 => (Action::Ignore, State::DcsEntry),
			0x1C..=0x1F => (Action::Ignore, State::DcsEntry),
			0x7F => (Action::Ignore, State::DcsEntry),

			0x20..=0x2F => (Action::Collect, State::DcsIntermediate),

			0x3A => (Action::None, State::DcsIgnore),

			0x30..=0x39 => (Action::Param, State::DcsParam),
			0x3C..=0x3F => (Action::Collect, State::DcsParam),

			0x40..=0x7E => (Action::None, State::DcsPassthrough),

			_ => (Action::None, State::Error),
		},
		(State::DcsIgnore, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::DcsIgnore),
			0x19 => (Action::Ignore, State::DcsIgnore),
			0x1C..=0x1F => (Action::Ignore, State::DcsIgnore),
			0x20..=0x7F => (Action::Ignore, State::DcsIgnore),

			_ => (Action::None, State::Error),
		},
		(State::DcsIntermediate, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::DcsIntermediate),
			0x19 => (Action::Ignore, State::DcsIntermediate),
			0x1C..=0x1F => (Action::Ignore, State::DcsIntermediate),
			0x20..=0x2F => (Action::Collect, State::DcsIntermediate),
			0x7F => (Action::Ignore, State::DcsIntermediate),

			0x40..=0x7E => (Action::None, State::DcsPassthrough),

			0x30..=0x3F => (Action::None, State::DcsIgnore),

			_ => (Action::None, State::Error),
		},
		(State::DcsParam, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::DcsParam),
			0x19 => (Action::Ignore, State::DcsParam),
			0x1C..=0x1F => (Action::Ignore, State::DcsParam),
			0x30..=0x39 => (Action::Param, State::DcsParam),
			0x3B => (Action::Param, State::DcsParam),
			0x7F => (Action::Ignore, State::DcsParam),

			0x3A => (Action::None, State::DcsIgnore),
			0x3C..=0x3F => (Action::None, State::DcsIgnore),

			0x20..=0x2F => (Action::Collect, State::DcsIntermediate),

			0x40..=0x7E => (Action::None, State::DcsPassthrough),

			_ => (Action::None, State::Error),
		},
		(State::DcsPassthrough, byte) => match byte {
			0x00..=0x17 => (Action::Put, State::DcsPassthrough),
			0x19 => (Action::Put, State::DcsPassthrough),
			0x1C..=0x1F => (Action::Put, State::DcsPassthrough),
			0x20..=0x7E => (Action::Put, State::DcsPassthrough),
			0x7F => (Action::Ignore, State::DcsPassthrough),

			_ => (Action::None, State::Error),
		},
		(State::Error, _) => (Action::None, State::Error),
		(State::Escape, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::Escape),
			0x19 => (Action::Execute, State::Escape),
			0x1C..=0x1F => (Action::Execute, State::Escape),
			0x7F => (Action::Ignore, State::Escape),

			0x20..=0x2F => (Action::Collect, State::EscapeIntermediate),

			0x30..=0x4F => (Action::EscDispatch, State::Ground),
			0x51..=0x57 => (Action::EscDispatch, State::Ground),
			0x59 => (Action::EscDispatch, State::Ground),
			0x5A => (Action::EscDispatch, State::Ground),
			0x5C => (Action::EscDispatch, State::Ground),
			0x60..=0x7E => (Action::EscDispatch, State::Ground),

			0x5B => (Action::None, State::CsiEntry),

			0x5D => (Action::None, State::OscString),

			0x50 => (Action::None, State::DcsEntry),

			0x58 => (Action::None, State::SosPmApcString),
			0x5E => (Action::None, State::SosPmApcString),
			0x5F => (Action::None, State::SosPmApcString),

			_ => (Action::None, State::Error),
		},
		(State::EscapeIntermediate, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::EscapeIntermediate),
			0x19 => (Action::Execute, State::EscapeIntermediate),
			0x1C..=0x1F => (Action::Execute, State::EscapeIntermediate),
			0x20..=0x2F => (Action::Collect, State::EscapeIntermediate),
			0x7F => (Action::Ignore, State::EscapeIntermediate),

			0x30..=0x7E => (Action::EscDispatch, State::Ground),

			_ => (Action::None, State::Error),
		},
		(State::Ground, byte) => match byte {
			0x00..=0x17 => (Action::Execute, State::Ground),
			0x19 => (Action::Execute, State::Ground),
			0x1C..=0x1F => (Action::Execute, State::Ground),

			// UTF-8 1 byte code point
			0x20..=0x7F => (Action::Print, State::Ground),
			// UTF-8 2 byte code point
			0xC0..=0xDF => (Action::Print, State::Ground),
			// UTF-8 3 byte code point
			0xE0..=0xEF => (Action::Print, State::Ground),
			// UTF-8 4 byte code point
			0xF0..=0xF7 => (Action::Print, State::Ground),

			_ => (Action::None, State::Error),
		},
		(State::OscString, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::OscString),
			0x19 => (Action::Ignore, State::OscString),
			0x1C..=0x1F => (Action::Ignore, State::OscString),
			0x20..=0x7F => (Action::OscPut, State::OscString),

			_ => (Action::None, State::Error),
		},
		(State::SosPmApcString, byte) => match byte {
			0x00..=0x17 => (Action::Ignore, State::SosPmApcString),
			0x19 => (Action::Ignore, State::SosPmApcString),
			0x1C..=0x1F => (Action::Ignore, State::SosPmApcString),
			0x20..=0x7F => (Action::Ignore, State::SosPmApcString),

			_ => (Action::None, State::Error),
		},
	}
}
