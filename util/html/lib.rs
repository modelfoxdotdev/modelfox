use derive_more::From;
use std::{borrow::Cow, fmt::Write};

pub use html_macro::{component, html, Props};

#[cfg(test)]
mod test;

#[derive(From)]
pub enum Node {
	RawText(RawTextNode),
	EscapedText(EscapedTextNode),
	Fragment(FragmentNode),
	Element(ElementNode),
	Component(ComponentNode),
	Option(Option<Box<Node>>),
	Vec(Vec<Node>),
}

pub struct RawTextNode(pub Cow<'static, str>);

pub struct EscapedTextNode(pub Cow<'static, str>);

pub struct FragmentNode {
	pub children: Vec<Node>,
}

pub struct ElementNode {
	pub name: &'static str,
	pub attributes: Vec<(AttributeKey, AttributeValue)>,
	pub children: Vec<Node>,
	pub self_closing: bool,
}

pub type AttributeKey = &'static str;

#[derive(From)]
pub enum AttributeValue {
	Bool(Option<bool>),
	String(Option<Cow<'static, str>>),
}

pub enum ComponentNode {
	Unrendered(Option<Box<dyn FnOnce() -> Node>>),
	Rendered(Box<Node>),
}

pub trait Props {
	type RequiredProps;
	type OptionalProps: Default;
	fn combine(required: Self::RequiredProps, optional: Self::OptionalProps) -> Self;
}

pub trait Component {
	type Props;
	fn render(props: Self::Props, children: Vec<Node>) -> Node;
}

impl Node {
	pub fn render_to_string(mut self) -> String {
		self.render();
		self.to_string()
	}

	fn render(&mut self) {
		match self {
			Node::Fragment(node) => {
				for child in node.children.iter_mut() {
					child.render();
				}
			}
			Node::Element(node) => {
				for child in node.children.iter_mut() {
					child.render();
				}
			}
			Node::Component(node) => {
				if let ComponentNode::Unrendered(component) = node {
					let component = component.take().unwrap();
					let mut rendered = component();
					rendered.render();
					*node = ComponentNode::Rendered(Box::new(rendered));
				}
			}
			Node::Vec(node) => {
				for child in node.iter_mut() {
					child.render();
				}
			}
			Node::Option(Some(node)) => {
				node.render();
			}
			_ => {}
		};
	}
}

impl std::fmt::Display for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Node::RawText(node) => {
				write!(f, "{}", node)?;
			}
			Node::EscapedText(node) => {
				write!(f, "{}", node)?;
			}
			Node::Fragment(node) => {
				for child in &node.children {
					write!(f, "{}", child)?;
				}
			}
			Node::Element(node) => {
				write!(f, "{}", node)?;
			}
			Node::Component(node) => {
				write!(f, "{}", node)?;
			}
			Node::Option(node) => {
				if let Some(node) = node {
					write!(f, "{}", node)?;
				}
			}
			Node::Vec(node) => {
				for node in node {
					write!(f, "{}", node)?;
				}
			}
		};
		Ok(())
	}
}

impl std::fmt::Display for FragmentNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for child in self.children.iter() {
			write!(f, "{}", child)?;
		}
		Ok(())
	}
}

impl std::fmt::Display for ElementNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}", self.name)?;
		for (key, value) in self.attributes.iter() {
			match value {
				AttributeValue::Bool(value) => {
					if let Some(true) = value {
						write!(f, r#" {}="""#, key)?;
					}
				}
				AttributeValue::String(value) => {
					if let Some(value) = value {
						write!(f, r#" {}="{}""#, key, EscapedText(&value))?;
					}
				}
			}
		}
		if self.self_closing {
			write!(f, " /")?;
		}
		write!(f, ">")?;
		if !self.self_closing {
			for child in self.children.iter() {
				write!(f, "{}", child)?;
			}
			write!(f, "</{}>", self.name)?;
		}
		Ok(())
	}
}

impl std::fmt::Display for ComponentNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let rendered = match self {
			ComponentNode::Rendered(r) => r,
			_ => panic!("attempted to display component that has not yet been rendered"),
		};
		write!(f, "{}", rendered)?;
		Ok(())
	}
}

impl std::fmt::Display for RawTextNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::fmt::Display for EscapedTextNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", EscapedText(&self.0))
	}
}

struct EscapedText<'a>(&'a str);

impl<'a> std::fmt::Display for EscapedText<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for c in self.0.chars() {
			match c {
				'>' => write!(f, "&gt;")?,
				'<' => write!(f, "&lt;")?,
				'"' => write!(f, "&quot;")?,
				'&' => write!(f, "&amp;")?,
				'\'' => write!(f, "&apos;")?,
				c => f.write_char(c)?,
			};
		}
		Ok(())
	}
}

impl From<Option<String>> for AttributeValue {
	fn from(value: Option<String>) -> AttributeValue {
		AttributeValue::String(value.map(|value| value.into()))
	}
}

impl From<String> for AttributeValue {
	fn from(value: String) -> AttributeValue {
		AttributeValue::String(Some(value.into()))
	}
}

impl From<&'static str> for AttributeValue {
	fn from(value: &'static str) -> AttributeValue {
		AttributeValue::String(Some(value.into()))
	}
}

impl From<bool> for AttributeValue {
	fn from(value: bool) -> AttributeValue {
		AttributeValue::Bool(Some(value))
	}
}

impl From<String> for Node {
	fn from(value: String) -> Node {
		Node::EscapedText(EscapedTextNode(value.into()))
	}
}

impl From<&'static str> for Node {
	fn from(value: &'static str) -> Node {
		Node::EscapedText(EscapedTextNode(value.into()))
	}
}

pub trait ConvertFromStr {
	fn convert_from_str(s: &'static str) -> Self;
}

impl ConvertFromStr for String {
	fn convert_from_str(s: &'static str) -> Self {
		s.to_owned()
	}
}

impl ConvertFromStr for Option<String> {
	fn convert_from_str(s: &'static str) -> Self {
		Some(s.to_owned())
	}
}

impl<T> From<Option<T>> for Node
where
	T: Into<Node>,
{
	fn from(value: Option<T>) -> Node {
		Node::Option(value.map(|value| Box::new(value.into())))
	}
}

pub trait AsOptionStr<'a> {
	fn as_option_str(&'a self) -> Option<&'a str>;
}

impl<'a> AsOptionStr<'a> for String {
	fn as_option_str(&'a self) -> Option<&'a str> {
		Some(self.as_ref())
	}
}

impl<'a> AsOptionStr<'a> for &'a str {
	fn as_option_str(&'a self) -> Option<&'a str> {
		Some(self)
	}
}

impl<'a> AsOptionStr<'a> for Cow<'a, str> {
	fn as_option_str(&'a self) -> Option<&'a str> {
		Some(self)
	}
}

impl<'a> AsOptionStr<'a> for Option<String> {
	fn as_option_str(&'a self) -> Option<&'a str> {
		self.as_deref()
	}
}

impl<'a> AsOptionStr<'a> for Option<&'a str> {
	fn as_option_str(&'a self) -> Option<&'a str> {
		*self
	}
}

impl<'a> AsOptionStr<'a> for Option<Cow<'a, str>> {
	fn as_option_str(&'a self) -> Option<&'a str> {
		self.as_deref()
	}
}

#[macro_export]
macro_rules! style {
	($($key:expr => $value:expr),*$(,)?) => {
		{
			let mut first = true;
			let mut style = String::new();
			$(
				let value = &$value;
				let value = html::AsOptionStr::as_option_str(value);
				if let Some(value) = value {
					if first {
						first = false;
					} else {
						style.push(' ');
					}
					style.push_str($key);
					style.push_str(": ");
					style.push_str(value);
					style.push(';');
				}
			)*
			style
		}
	};
}

#[macro_export]
macro_rules! classes {
	($($class:expr),*$(,)?) => {
		{
			let mut first = true;
			let mut classes = String::new();
			$(
				let class = &$class;
				let class = html::AsOptionStr::as_option_str(class);
				if let Some(class) = class {
					if first {
						first = false;
					} else {
						classes.push(' ');
					}
					classes.push_str(class);
				}
			)*
			classes
		}
	};
}

#[macro_export]
macro_rules! raw {
	($t:expr) => {
		html::RawTextNode($t.into())
	};
}

#[macro_export]
macro_rules! text {
	($t:expr) => {
		html::EscapedTextNode($t.into())
	};
}
