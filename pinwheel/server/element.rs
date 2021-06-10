use super::{text::EscapedText, Node};
use crate::{
	attribute_value::{AttributeValue, IntoAttributeValue},
	option_string_value::{IntoOptionStringValue, OptionStringValue},
	string_value::IntoStringValue,
};
use futures::{executor::block_on, stream::StreamExt};
use futures_signals::{
	signal::{Signal, SignalExt},
	signal_vec::{SignalVec, SignalVecExt},
};
use std::{borrow::Cow, future::Future};
use wasm_bindgen::prelude::*;
use web_sys as dom;

pub struct Element {
	tag: &'static str,
	kind: Option<HtmlElementKind>,
	attributes: Vec<Attribute>,
	classes: Vec<Class>,
	styles: Vec<Style>,
	children: Vec<Node>,
	inner_html: Option<Cow<'static, str>>,
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

pub struct Attribute {
	name: Cow<'static, str>,
	value: AttributeValue,
}

pub struct Class {
	value: OptionStringValue,
}

pub struct Style {
	name: Cow<'static, str>,
	value: OptionStringValue,
}

impl Element {
	pub fn new(tag: &'static str, _namespace: Namespace, kind: Option<HtmlElementKind>) -> Element {
		Element {
			tag,
			kind,
			attributes: Vec::new(),
			classes: Vec::new(),
			styles: Vec::new(),
			children: Vec::new(),
			inner_html: None,
		}
	}

	pub fn future<F>(self, _f: impl FnOnce(&Element) -> F) -> Element
	where
		F: 'static + Future<Output = ()>,
	{
		self
	}

	pub fn attribute<T>(mut self, name: impl Into<Cow<'static, str>>, value: T) -> Element
	where
		T: IntoAttributeValue,
	{
		self.attributes.push(Attribute {
			name: name.into(),
			value: value.into_attribute_value(),
		});
		self
	}

	pub fn attribute_signal<T, S>(mut self, name: impl Into<Cow<'static, str>>, value: S) -> Element
	where
		T: IntoAttributeValue,
		S: 'static + Unpin + Signal<Item = T>,
	{
		self.attributes.push(Attribute {
			name: name.into(),
			value: block_on(value.first().to_future()).into_attribute_value(),
		});
		self
	}

	pub fn class<T>(mut self, value: T) -> Element
	where
		T: IntoOptionStringValue,
	{
		self.classes.push(Class {
			value: value.into_option_string_value(),
		});
		self
	}

	pub fn class_signal<S, T>(mut self, value: S) -> Element
	where
		S: 'static + Unpin + Signal<Item = T>,
		T: IntoOptionStringValue,
	{
		self.classes.push(Class {
			value: block_on(value.first().to_future()).into_option_string_value(),
		});
		self
	}

	pub fn style<T>(mut self, name: impl Into<Cow<'static, str>>, value: T) -> Element
	where
		T: IntoOptionStringValue,
	{
		self.styles.push(Style {
			name: name.into(),
			value: value.into_option_string_value(),
		});
		self
	}

	pub fn style_signal<S, T>(mut self, name: impl Into<Cow<'static, str>>, value: S) -> Element
	where
		S: 'static + Unpin + Signal<Item = T>,
		T: IntoOptionStringValue,
	{
		self.styles.push(Style {
			name: name.into(),
			value: block_on(value.first().to_future()).into_option_string_value(),
		});
		self
	}

	pub fn event(
		self,
		_name: impl Into<Cow<'static, str>>,
		_closure: impl 'static + FnMut(JsValue),
	) -> Element {
		self
	}

	pub fn child<T>(mut self, child: T) -> Element
	where
		T: Into<Node>,
	{
		let child = child.into();
		self.children.push(child);
		self
	}

	pub fn children<T, I>(mut self, children: I) -> Element
	where
		T: Into<Node>,
		I: IntoIterator<Item = T>,
	{
		for child in children.into_iter() {
			let child = child.into();
			self.children.push(child);
		}
		self
	}

	pub fn child_signal<T, S>(mut self, signal: S) -> Element
	where
		T: Into<Node>,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let child = block_on(signal.first().to_future()).into();
		self.children.push(child);
		self
	}

	pub fn child_signal_vec<T, S>(mut self, signal_vec: S) -> Element
	where
		T: Into<Node>,
		S: 'static + Unpin + SignalVec<Item = T>,
	{
		let mut children = Vec::new();
		block_on(
			signal_vec
				.map(|child| child.into())
				.to_stream()
				.map(|diff| {
					diff.apply_to_vec(&mut children);
				})
				.next(),
		);
		self.children.append(&mut children);
		self
	}

	pub fn inner_html(mut self, value: impl IntoStringValue) -> Element {
		let html = value.into_string_value();
		self.inner_html = Some(html.0);
		self.children.clear();
		self
	}

	pub fn dom_element(&self) -> dom::Element {
		unimplemented!()
	}
}

impl std::fmt::Display for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}", self.tag)?;
		for attribute in self.attributes.iter() {
			match &attribute.value {
				AttributeValue::Bool(value) => {
					if value.unwrap_or(false) {
						write!(f, " {}", attribute.name)?;
					}
				}
				AttributeValue::String(value) => {
					if let Some(value) = value.as_ref() {
						write!(f, " {}=\"{}\"", attribute.name, EscapedText(value.as_ref()))?;
					}
				}
			}
		}
		if !self.classes.is_empty() {
			write!(f, " class=\"")?;
			let mut first = true;
			for class in self.classes.iter() {
				if let Some(value) = class.value.0.as_ref() {
					if first {
						first = false;
					} else {
						write!(f, " ")?;
					}
					write!(f, "{}", value)?;
				}
			}
			write!(f, "\"")?;
		}
		if !self.styles.is_empty() {
			write!(f, " style=\"")?;
			let mut first = true;
			for style in self.styles.iter() {
				if let Some(value) = style.value.0.as_ref() {
					if first {
						first = false;
					} else {
						write!(f, " ")?;
					}
					write!(f, "{}: {};", style.name, value)?;
				}
			}
			write!(f, "\"")?;
		}
		if self.kind == Some(HtmlElementKind::Void) {
			write!(f, " />")?;
			return Ok(());
		}
		write!(f, ">")?;
		if let Some(inner_html) = &self.inner_html {
			write!(f, "{}", inner_html)?;
		} else {
			for child in self.children.iter() {
				write!(f, "{}", child)?;
			}
		}
		if self.kind != Some(HtmlElementKind::Void) {
			write!(f, "</{}>", self.tag)?;
		}
		Ok(())
	}
}
