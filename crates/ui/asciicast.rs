use num::ToPrimitive;
use pinwheel::prelude::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

pub struct Asciicast {
	pub id: String,
	pub height: String,
	pub options: AsciicastOptions,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AsciicastOptions {
	pub events: Vec<AsciicastEvent>,
	pub repeat: bool,
	pub repeat_delay: f32,
}

impl Default for AsciicastOptions {
	fn default() -> AsciicastOptions {
		AsciicastOptions {
			events: vec![],
			repeat: true,
			repeat_delay: 1000.0,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AsciicastEvent(pub f32, pub String);

impl Component for Asciicast {
	fn into_node(self) -> Node {
		let options = serde_json::to_string(&self.options).unwrap();
		div()
			.class("asciicast")
			.style(style::HEIGHT, self.height)
			.attribute("id", self.id)
			.attribute("data-options", options)
			.into_node()
	}
}

pub fn boot_asciicast(id: &str) {
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	let container = document
		.get_element_by_id(id)
		.unwrap()
		.dyn_into::<dom::HtmlElement>()
		.unwrap();
	let options = container.dataset().get("options").unwrap();
	let options = serde_json::from_str(&options).unwrap();
	let player = AsciicastPlayer::new(options, container);
	player.borrow_mut().start();
	std::mem::forget(player);
}

struct AsciicastPlayer {
	options: AsciicastOptions,
	state: Option<AsciicastPlayerState>,
	render_callback: Option<Closure<dyn Fn()>>,
	start_callback: Option<Closure<dyn Fn()>>,
	element: dom::HtmlElement,
}

struct AsciicastPlayerState {
	frame_index: usize,
	start_time: f64,
}

impl AsciicastPlayer {
	fn new(options: AsciicastOptions, element: dom::HtmlElement) -> Rc<RefCell<AsciicastPlayer>> {
		let player = Rc::new(RefCell::new(Self {
			options,
			state: None,
			render_callback: None,
			start_callback: None,
			element,
		}));
		let player_ref = Rc::downgrade(&player);
		let start_callback = Closure::<dyn Fn()>::wrap(Box::new(move || {
			let player = player_ref.upgrade().unwrap();
			player.borrow_mut().start();
		}));
		player.borrow_mut().start_callback = Some(start_callback);
		let player_ref = Rc::downgrade(&player);
		let render_callback = Closure::<dyn Fn()>::wrap(Box::new(move || {
			let player = player_ref.upgrade().unwrap();
			player.borrow_mut().render();
		}));
		player.borrow_mut().render_callback = Some(render_callback);
		player
	}

	fn start(&mut self) {
		let window = dom::window().unwrap();
		let performance = window.performance().unwrap();
		self.element.set_inner_html("");
		self.state = Some(AsciicastPlayerState {
			start_time: performance.now(),
			frame_index: 0,
		});
		self.render();
	}

	fn render(&mut self) {
		let window = dom::window().unwrap();
		let performance = window.performance().unwrap();
		let start_time = self.state.as_ref().unwrap().start_time;
		let current_time = performance.now() - start_time;
		let state = self.state.as_mut().unwrap();
		// Apply changes from all unapplied events up to the current time.
		let mut html = self.element.inner_html();
		loop {
			if state.frame_index >= self.options.events.len() {
				break;
			}
			let event = &self.options.events[state.frame_index];
			let event_time = event.0 * 1000.0;
			if current_time < event_time.to_f64().unwrap() {
				break;
			}
			html.push_str(&event.1);
			state.frame_index += 1;
		}
		self.element.set_inner_html(&html);
		if state.frame_index < self.options.events.len() {
			let next_event = &self.options.events[state.frame_index];
			let next_event_time = next_event.0 * 1000.0;
			let current_time = performance.now() - start_time;
			let timeout = (next_event_time.to_f64().unwrap() - current_time)
				.max(0.0)
				.to_i32()
				.unwrap();
			window
				.set_timeout_with_callback_and_timeout_and_arguments_0(
					self.render_callback
						.as_ref()
						.unwrap()
						.as_ref()
						.unchecked_ref(),
					timeout,
				)
				.unwrap();
		} else if self.options.repeat {
			window
				.set_timeout_with_callback_and_timeout_and_arguments_0(
					self.start_callback
						.as_ref()
						.unwrap()
						.as_ref()
						.unchecked_ref(),
					self.options.repeat_delay.to_i32().unwrap(),
				)
				.unwrap();
		}
	}
}

impl Drop for AsciicastPlayer {
	fn drop(&mut self) {
		self.element.set_inner_html("");
	}
}
