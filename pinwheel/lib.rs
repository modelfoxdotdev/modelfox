pub mod app;
pub mod attribute_value;
pub mod classes;
pub mod component;
pub mod dehydrate;
pub mod elements;
pub mod hydrate;
pub mod style_value;
pub mod text_value;
pub mod zip;

#[cfg(target_arch = "wasm32")]
mod client;
#[cfg(target_arch = "wasm32")]
pub use crate::client::*;

#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::server::*;

pub mod prelude {
	pub use crate::{
		app::App,
		classes, clone,
		component::Component,
		dehydrate::Dehydrate,
		elements::{html, html::*, svg},
		fragment, html,
		hydrate::hydrate,
		text,
		zip::{ZipSignal, ZipSignalTrait},
		Element, Fragment, Namespace, Node, SignalNode, SignalVecNode, Text,
	};
	pub use crate::{BoxSignal, BoxSignalTrait, State, StateVec};
	pub use futures_signals::{
		signal::{Signal, SignalExt},
		signal_vec::{SignalVec, SignalVecExt},
	};
	pub use pinwheel_macro::ComponentBuilder;
}

#[macro_export]
macro_rules! clone {
	($($name:ident),*$(,)?) => {
		$(let $name = $name.clone();)*
	}
}

pub fn html<T: component::Component>(component: T) -> String {
	format!("<!doctype html>{}", component.into_node())
}

use futures_signals::signal::{Broadcaster, Signal};

pub type State<T> = futures_signals::signal::Mutable<T>;
pub type StateVec<T> = futures_signals::signal_vec::MutableVec<T>;

pub struct BoxSignal<T> {
	#[allow(dead_code)]
	state: Option<State<T>>,
	signal: Broadcaster<Box<dyn 'static + Unpin + Signal<Item = T>>>,
}

impl<T> std::ops::Deref for BoxSignal<T> {
	type Target = Broadcaster<Box<dyn 'static + Unpin + Signal<Item = T>>>;
	fn deref(&self) -> &Self::Target {
		&self.signal
	}
}

pub trait BoxSignalTrait<T> {
	fn boxed(self) -> BoxSignal<T>;
}

impl<T, S> BoxSignalTrait<T> for S
where
	S: 'static + Unpin + Signal<Item = T>,
{
	fn boxed(self) -> BoxSignal<T> {
		BoxSignal {
			state: None,
			signal: Broadcaster::new(Box::new(self)),
		}
	}
}

impl<T> From<T> for BoxSignal<T>
where
	T: 'static + Clone,
{
	fn from(value: T) -> Self {
		let state = State::new(value);
		let signal: Broadcaster<Box<dyn 'static + Unpin + Signal<Item = T>>> =
			Broadcaster::new(Box::new(state.signal_cloned()));
		BoxSignal {
			state: Some(state),
			signal,
		}
	}
}
