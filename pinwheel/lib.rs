pub mod app;
pub mod attribute_value;
pub mod classes;
pub mod component;
pub mod dehydrate;
pub mod elements;
pub mod hydrate;
pub mod option_string_value;
pub mod string_value;
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
	pub use crate::BoxSignal;
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
	pub use futures_signals::{
		signal::{Mutable, Signal, SignalExt},
		signal_vec::{MutableVec, SignalVec, SignalVecExt},
	};
	pub use pinwheel_macro::ComponentBuilder;
}

pub use futures_signals::signal;
pub use futures_signals::signal_vec;

#[macro_export]
macro_rules! clone {
	($($name:ident),*$(,)?) => {
		$(let $name = $name.clone();)*
	}
}

pub fn html<T: component::Component>(component: T) -> String {
	format!("<!doctype html>{}", component.into_node())
}

pub type BoxSignal<T> = Box<dyn 'static + Unpin + futures_signals::signal::Signal<Item = T>>;
