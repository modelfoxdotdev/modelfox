use super::Handle;
use crate::string_value::IntoStringValue;
use futures::{future::abortable, prelude::*};
use futures_signals::signal::{Signal, SignalExt};
use std::future::ready;
use wasm_bindgen_futures::spawn_local;
use web_sys as dom;

pub fn text(text: impl IntoStringValue) -> Text {
	Text::new(text)
}

pub struct Text {
	pub(crate) text: dom::Text,
	#[allow(dead_code)]
	pub(crate) handle: Option<Handle>,
}

impl Text {
	pub fn new(value: impl IntoStringValue) -> Text {
		let document = dom::window().unwrap().document().unwrap();
		let text = document.create_text_node(value.into_string_value().0.as_ref());
		Text { text, handle: None }
	}

	pub fn new_signal<T, S>(value: S) -> Text
	where
		T: IntoStringValue,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let document = dom::window().unwrap().document().unwrap();
		let text = document.create_text_node("");
		let text_node = text.clone();
		let (future, handle) = abortable(value.for_each(move |value| {
			text_node.set_data(value.into_string_value().0.as_ref());
			ready(())
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		Text {
			text,
			handle: Some(handle),
		}
	}
}
