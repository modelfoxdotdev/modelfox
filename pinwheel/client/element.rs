use super::{node::Node, signal::SignalNode, signal_vec::SignalVecNode, Handle};
use crate::{
	attribute_value::{AttributeValue, IntoAttributeValue},
	option_string_value::{IntoOptionStringValue, OptionStringValue},
	string_value::IntoStringValue,
};
use futures::{future::abortable, prelude::*};
use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::SignalVec;
use std::{borrow::Cow, future::ready, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys as dom;

pub struct Element {
	pub(crate) element: dom::Element,
	pub(crate) handles: Vec<Handle>,
	pub(crate) events: Vec<Event>,
	pub(crate) children: Vec<Node>,
}

pub enum Namespace {
	Html,
	Svg,
	MathMl,
}

#[derive(PartialEq)]
pub enum HtmlElementKind {
	Void,
	Template,
	RawText,
	EscapableRawText,
	Foreign,
	Normal,
}

pub struct Event {
	element: dom::Element,
	name: Cow<'static, str>,
	closure: Rc<Closure<dyn 'static + FnMut(JsValue)>>,
}

impl Element {
	pub fn new(tag: &'static str, namespace: Namespace, _kind: Option<HtmlElementKind>) -> Element {
		let document = dom::window().unwrap().document().unwrap();
		let namespace = match namespace {
			Namespace::Html => None,
			Namespace::Svg => Some("http://www.w3.org/2000/svg"),
			Namespace::MathMl => Some("http://www.w3.org/1998/Math/MathML"),
		};
		let element = match namespace {
			None => document.create_element(tag).unwrap(),
			Some(namespace) => document.create_element_ns(Some(namespace), tag).unwrap(),
		};
		Element {
			element,
			handles: Vec::new(),
			events: Vec::new(),
			children: Vec::new(),
		}
	}

	pub fn future<F>(mut self, f: impl FnOnce(&Element) -> F) -> Element
	where
		F: 'static + Future<Output = ()>,
	{
		let (future, handle) = abortable(f(&self));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		self.handles.push(handle);
		self
	}

	pub fn attribute<T>(self, name: impl Into<Cow<'static, str>>, value: T) -> Element
	where
		T: IntoAttributeValue,
	{
		let name = name.into();
		Self::set_attribute(&self.element, &name, value);
		self
	}

	pub fn attribute_signal<T, S>(mut self, name: impl Into<Cow<'static, str>>, value: S) -> Element
	where
		T: IntoAttributeValue,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let name = name.into();
		let element = self.element.clone();
		let (future, handle) = abortable(value.for_each(move |value| {
			Self::set_attribute(&element, &name, value);
			ready(())
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		self.handles.push(handle);
		self
	}

	fn set_attribute(element: &dom::Element, name: &str, value: impl IntoAttributeValue) {
		match value.into_attribute_value() {
			AttributeValue::Bool(value) => {
				if value.unwrap_or(false) {
					element.set_attribute(&name, "").unwrap();
				} else {
					element.remove_attribute(&name).unwrap();
				}
			}
			AttributeValue::String(value) => {
				if let Some(value) = value {
					element.set_attribute(&name, &value).unwrap();
				} else {
					element.remove_attribute(&name).unwrap();
				}
			}
		};
	}

	pub fn class<T>(self, value: T) -> Element
	where
		T: IntoOptionStringValue,
	{
		let value = value.into_option_string_value();
		Self::set_class(&self.element, &value, None);
		self
	}

	pub fn class_signal<S, T>(mut self, value: S) -> Element
	where
		S: 'static + Unpin + Signal<Item = T>,
		T: IntoOptionStringValue,
	{
		let element = self.element.clone();
		let mut previous_value = None;
		let (future, handle) = abortable(value.for_each(move |value| {
			let value = value.into_option_string_value();
			Self::set_class(&element, &value, previous_value.as_ref());
			previous_value = Some(value);
			ready(())
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		self.handles.push(handle);
		self
	}

	fn set_class(
		element: &dom::Element,
		value: &OptionStringValue,
		previous_value: Option<&OptionStringValue>,
	) {
		if let Some(previous_value) = previous_value.and_then(|value| value.0.as_ref()) {
			for class in previous_value.split(' ') {
				element.class_list().remove_1(&class).unwrap();
			}
		}
		if let Some(value) = value.0.as_ref() {
			for class in value.split(' ') {
				element.class_list().add_1(&class).unwrap();
			}
		}
	}

	pub fn style<T>(self, name: impl Into<Cow<'static, str>>, value: T) -> Element
	where
		T: IntoOptionStringValue,
	{
		let name = name.into();
		Self::set_style(&self.element, &name, value);
		self
	}

	pub fn style_signal<S, T>(mut self, name: impl Into<Cow<'static, str>>, value: S) -> Element
	where
		S: 'static + Unpin + Signal<Item = T>,
		T: IntoOptionStringValue,
	{
		let name = name.into();
		let element = self.element.clone();
		let (future, handle) = abortable(value.for_each(move |value| {
			Self::set_style(&element, &name, value);
			ready(())
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		self.handles.push(handle);
		self
	}

	fn set_style(element: &dom::Element, name: &str, value: impl IntoOptionStringValue) {
		let value = value.into_option_string_value();
		if let Some(element) = element.dyn_ref::<dom::HtmlElement>() {
			if let Some(value) = value.0 {
				element.style().set_property(&name, &value).unwrap();
			} else {
				element.style().remove_property(&name).unwrap();
			}
		}
	}

	pub fn event(
		mut self,
		name: impl Into<Cow<'static, str>>,
		closure: impl 'static + FnMut(JsValue),
	) -> Element {
		let name = name.into();
		let closure = Rc::new(Closure::<dyn FnMut(JsValue)>::wrap(Box::new(closure)));
		self.element
			.add_event_listener_with_callback(
				name.as_ref(),
				closure.as_ref().as_ref().unchecked_ref(),
			)
			.unwrap();
		self.events.push(Event {
			element: self.element.clone(),
			name,
			closure,
		});
		self
	}

	pub fn child<T>(mut self, child: T) -> Element
	where
		T: Into<Node>,
	{
		let mut child = child.into();
		child.insert_before(self.element.unchecked_ref(), None);
		self.children.push(child);
		self
	}

	pub fn children<T, I>(mut self, children: I) -> Element
	where
		T: Into<Node>,
		I: IntoIterator<Item = T>,
	{
		for child in children.into_iter() {
			let mut child = child.into();
			child.insert_before(self.element.unchecked_ref(), None);
			self.children.push(child);
		}
		self
	}

	pub fn child_signal<T, S>(mut self, signal: S) -> Element
	where
		T: Into<Node>,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let mut child = Node::Signal(SignalNode::new(signal));
		child.insert_before(self.element.unchecked_ref(), None);
		self.children.push(child);
		self
	}

	pub fn child_signal_vec<T, S>(mut self, signal_vec: S) -> Element
	where
		T: Into<Node>,
		S: 'static + Unpin + SignalVec<Item = T>,
	{
		let mut child = Node::SignalVec(SignalVecNode::new(signal_vec));
		child.insert_before(self.element.unchecked_ref(), None);
		self.children.push(child);
		self
	}

	pub fn inner_html(mut self, value: impl IntoStringValue) -> Element {
		let html = value.into_string_value();
		self.element.set_inner_html(html.0.as_ref());
		self.children.clear();
		self
	}

	pub fn dom_element(&self) -> dom::Element {
		self.element.clone()
	}
}

impl Drop for Event {
	fn drop(&mut self) {
		self.element
			.remove_event_listener_with_callback(
				self.name.as_ref(),
				self.closure.as_ref().as_ref().unchecked_ref(),
			)
			.unwrap();
	}
}
