use heck::*;
use quote::{format_ident, quote};

pub fn element(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let element: Element = syn::parse2(input)?;
	let code = quote! { #element };
	Ok(code)
}

struct Element {
	attrs: Vec<syn::Attribute>,
	namespace: Namespace,
	tag: String,
	kind: Option<syn::Expr>,
	element: Option<syn::Path>,
	attributes: Option<Attributes>,
	events: Option<Events>,
}

enum Namespace {
	Html,
	Svg,
	MathMl,
}

struct Attributes(Vec<Attribute>);

struct Attribute {
	attrs: Vec<syn::Attribute>,
	name: String,
	#[allow(dead_code)]
	is_void: bool,
}

struct Events(Vec<Event>);

struct Event {
	attrs: Vec<syn::Attribute>,
	name: String,
	ty: syn::Type,
}

impl syn::parse::Parse for Element {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attrs = input.call(syn::Attribute::parse_outer)?;
		let mut namespace = None;
		let mut tag = None;
		let mut kind = None;
		let mut element = None;
		let mut attributes = None;
		let mut events = None;
		while !input.is_empty() {
			let ident = input.parse::<syn::Ident>()?;
			input.parse::<syn::Token![=]>()?;
			let ident_string = ident.to_string();
			match ident_string.as_str() {
				"namespace" => {
					namespace = Some(match input.parse::<syn::LitStr>()?.value().as_str() {
						"html" => Namespace::Html,
						"svg" => Namespace::Svg,
						"mathml" => Namespace::MathMl,
						_ => return Err(input.error("unexpected namespace")),
					});
				}
				"tag" => {
					tag = Some(input.parse::<syn::LitStr>()?.value());
				}
				"kind" => {
					kind = Some(input.parse()?);
				}
				"element" => {
					element = Some(input.parse()?);
				}
				"attributes" => {
					let content;
					syn::braced!(content in input);
					attributes = Some(content.parse()?);
				}
				"events" => {
					let content;
					syn::braced!(content in input);
					events = Some(content.parse()?)
				}
				_ => return Err(input.error("unexpected key")),
			}
			input.parse::<syn::Token![,]>().ok();
		}
		Ok(Element {
			attrs,
			namespace: namespace.unwrap(),
			tag: tag.unwrap(),
			kind,
			element,
			attributes,
			events,
		})
	}
}

impl syn::parse::Parse for Attributes {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attributes = input.parse_terminated::<_, syn::Token![,]>(Attribute::parse)?;
		let attributes = attributes.into_iter().collect();
		Ok(Attributes(attributes))
	}
}

impl syn::parse::Parse for Attribute {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attrs = input.call(syn::Attribute::parse_outer)?;
		let name = input.parse::<syn::Ident>()?.to_string();
		let is_void = input.parse::<syn::Token![?]>().ok().is_some();
		Ok(Attribute {
			attrs,
			name,
			is_void,
		})
	}
}

impl syn::parse::Parse for Events {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let events = input.parse_terminated::<_, syn::Token![,]>(Event::parse)?;
		let events = events.into_iter().collect();
		Ok(Events(events))
	}
}

impl syn::parse::Parse for Event {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attrs = input.call(syn::Attribute::parse_outer)?;
		let name = input.parse::<syn::Ident>()?.to_string();
		input.parse::<syn::Token![:]>()?;
		let ty = input.parse()?;
		Ok(Event { attrs, name, ty })
	}
}

impl quote::ToTokens for Element {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let doc = &self.attrs;
		let tag = &self.tag;
		let namespace = match self.namespace {
			Namespace::Html => quote! { crate::Namespace::Html },
			Namespace::Svg => quote! { crate::Namespace::Svg },
			Namespace::MathMl => quote! { crate::Namespace::MathMl },
		};
		let kind = match self.namespace {
			Namespace::Html => self
				.kind
				.as_ref()
				.map(|kind| quote! { Some(#kind) })
				.unwrap_or_else(|| quote! { Some(crate::HtmlElementKind::Normal) }),
			_ => quote! { None },
		};
		let fn_ident = format_ident!("{}", tag);
		let element_ident = format_ident!("{}Element", tag.to_camel_case());
		let dom_element_ty = self
			.element
			.as_ref()
			.map(|element| quote! { #element })
			.unwrap_or_else(|| quote! { web_sys::Element });
		let basic_impl = quote! {
			impl #element_ident {
				pub fn new() -> Self {
					#element_ident(crate::Element::new(#tag, #namespace, #kind))
				}
				pub fn future<F>(mut self, f: impl FnOnce(&crate::Element) -> F ) -> Self where F: 'static + std::future::Future<Output = ()> {
					self.0 = self.0.future(f);
					self
				}
				pub fn attribute<T>(mut self, name: impl Into<std::borrow::Cow<'static, str>>, value: T) -> Self
				where
					T: crate::attribute_value::IntoAttributeValue,
				{
					self.0 = self.0.attribute(name, value);
					self
				}
				pub fn attribute_signal<T, S>(mut self, name: impl Into<std::borrow::Cow<'static, str>>, value: S) -> Self
				where
					T: crate::attribute_value::IntoAttributeValue,
					S: 'static + Unpin + futures_signals::signal::Signal<Item = T>,
				{
					self.0 = self.0.attribute_signal(name, value);
					self
				}
				pub fn class<T>(mut self, value: T) -> Self
				where
					T: crate::option_string_value::IntoOptionStringValue,
				{
					self.0 = self.0.class(value);
					self
				}
				pub fn class_signal<S, T>(mut self, value: S) -> Self
				where
					S: 'static + Unpin + futures_signals::signal::Signal<Item = T>,
					T: crate::option_string_value::IntoOptionStringValue,
				{
					self.0 = self.0.class_signal(value);
					self
				}
				pub fn style<T>(mut self, name: impl Into<std::borrow::Cow<'static, str>>, value: T) -> Self
				where
					T: crate::option_string_value::IntoOptionStringValue,
				{
					self.0 = self.0.style(name, value);
					self
				}
				pub fn style_signal<S, T>(mut self, name: impl Into<std::borrow::Cow<'static, str>>, value: S) -> Self
				where
					S: 'static + Unpin + futures_signals::signal::Signal<Item = T>,
					T: crate::option_string_value::IntoOptionStringValue,
				{
					self.0 = self.0.style_signal(name, value);
					self
				}
				pub fn event(
					mut self,
					name: impl Into<std::borrow::Cow<'static, str>>,
					closure: impl 'static + FnMut(wasm_bindgen::JsValue),
				) -> Self {
					self.0 = self.0.event(name, closure);
					self
				}
				pub fn child<T>(mut self, child: T) -> Self
				where
					T: Into<crate::Node>,
				{
					self.0 = self.0.child(child);
					self
				}
				pub fn children<T, I>(mut self, children: I) -> Self
				where
					T: Into<crate::Node>,
					I: IntoIterator<Item = T>,
				{
					for child in children.into_iter() {
						self.0 = self.0.child(child);
					}
					self
				}
				pub fn child_signal<T, S>(mut self, signal: S) -> Self
				where
					T: Into<crate::Node>,
					S: 'static + Unpin + futures_signals::signal::Signal<Item = T>,
				{
					self.0 = self.0.child_signal(signal);
					self
				}
				pub fn child_signal_vec<T, S>(mut self, signal_vec: S) -> Self
				where
					T: Into<crate::Node>,
					S: 'static + Unpin + futures_signals::signal_vec::SignalVec<Item = T>,
				{
					self.0 = self.0.child_signal_vec(signal_vec);
					self
				}
				pub fn inner_html(mut self, value: impl crate::string_value::IntoStringValue) -> Self {
					self.0 = self.0.inner_html(value);
					self
				}
				#[cfg(target_arch = "wasm32")]
				pub fn dom_element(&self) -> #dom_element_ty {
					wasm_bindgen::JsCast::unchecked_into(self.0.dom_element())
				}
			}
		};
		let global_attributes = match self.namespace {
			Namespace::Html => html_element_attributes(),
			_ => Attributes(Vec::new()),
		};
		let empty_attributes = Attributes(Vec::new());
		let attributes = self.attributes.as_ref().unwrap_or(&empty_attributes);
		let attribute_fns =
			global_attributes
				.0
				.iter()
				.chain(attributes.0.iter())
				.map(|attribute| {
					let attribute_attrs = &attribute.attrs;
					let attribute_name = attribute.name.to_string();
					let attribute_ident = format_ident!("{}", attribute_name);
					let attribute_signal_ident = format_ident!("{}_signal", attribute_name);
					quote! {
						#(#attribute_attrs)*
						pub fn #attribute_ident<T>(mut self, value: T) -> Self
						where
							T: crate::attribute_value::IntoAttributeValue,
						{
							self.0 = self.0.attribute(#attribute_name, value);
							self
						}
						pub fn #attribute_signal_ident<T, S>(mut self, value: S) -> Self
						where
							T: crate::attribute_value::IntoAttributeValue,
							S: 'static + Unpin + futures_signals::signal::Signal<Item = T>,
						{
							self.0 = self.0.attribute_signal(#attribute_name, value);
							self
						}
					}
				});
		let attributes_impl = quote! {
			impl #element_ident {
				#(#attribute_fns)*
			}
		};
		let global_events = match self.namespace {
			Namespace::Html => html_element_events(),
			_ => Events(Vec::new()),
		};
		let empty_events = Events(Vec::new());
		let events = self.events.as_ref().unwrap_or(&empty_events);
		let event_fns = global_events.0.iter().chain(events.0.iter()).map(|event| {
			let event_attrs = &event.attrs;
			let event_name = event.name.to_string();
			let event_fn_ident = format_ident!("on{}", event_name);
			let event_ty = &event.ty;
			quote! {
				#(#event_attrs)*
				pub fn #event_fn_ident(
					mut self,
					mut closure: impl 'static + FnMut(web_sys::#event_ty),
				) -> Self {
					self.0 = self.0.event(
						#event_name,
						move |event| closure(wasm_bindgen::JsCast::unchecked_into(event)),
					);
					self
				}
			}
		});
		let events_impl = quote! {
			impl #element_ident {
				#(#event_fns)*
			}
		};
		let from_impls = quote! {
			impl crate::component::Component for #element_ident {
				fn into_node(self) -> crate::Node {
					crate::Node::Element(self.0)
				}
			}
		};
		let code = quote! {
			#(#doc)*
			pub fn #fn_ident() -> #element_ident {
				#element_ident::new()
			}
			#(#doc)*
			pub struct #element_ident(crate::Element);
			#basic_impl
			#attributes_impl
			#events_impl
			#from_impls
		};
		code.to_tokens(tokens);
	}
}

fn html_element_attributes() -> Attributes {
	syn::parse2(quote!(
		/// Provides a hint for generating a keyboard shortcut for the current element. This attribute consists of a space-separated list of characters. The browser should use the first one that exists on the computer keyboard layout.
		accesskey,
		/**
		Controls whether and how text input is automatically capitalized as it is entered/edited by the user. It can have the following values:
			* off or none, no autocapitalization is applied (all letters default to lowercase)
			* on or sentences, the first letter of each sentence defaults to a capital letter; all other letters default to lowercase
			* words, the first letter of each word defaults to a capital letter; all other letters default to lowercase
			* characters, all letters should default to uppercase
		*/
		autocapitalize,
		/// A space-separated list of the classes of the element. Classes allows CSS and JavaScript to select and access specific elements via the class selectors or functions like the method `Document.getElementsByClassName()`.
		// class,
		/**
		An enumerated attribute indicating if the element should be editable by the user. If so, the browser modifies its widget to allow editing. The attribute must take one of the following values:
		true or the empty string, which indicates that the element must be editable;
		false, which indicates that the element must not be editable.
		*/
		contenteditable,
		/// The id of a `<menu>` to use as the contextual menu for this element.
		contextmenu,
		/**
		An enumerated attribute indicating the directionality of the element's text. It can have the following values:
			* ltr, which means left to right and is to be used for languages that are written from the left to the right (like English);
			* rtl, which means right to left and is to be used for languages that are written from the right to the left (like Arabic);
			* auto, which lets the user agent decide. It uses a basic algorithm as it parses the characters inside the element until it finds a character with a strong directionality, then it applies that directionality to the whole element.
		*/
		dir,
		/**
		An enumerated attribute indicating whether the element can be dragged, using the Drag and Drop API. It can have the following values:
			* true, which indicates that the element may be dragged
			* false, which indicates that the element may not be dragged.
		*/
		draggable,
		/// Hints what action label (or icon) to present for the enter key on virtual keyboards.
		enterkeyhint,
		/// Used to transitively export shadow parts from a nested shadow tree into a containing light tree.
		exportparts,
		/// A Boolean attribute indicates that the element is not yet, or is no longer, relevant. For example, it can be used to hide elements of the page that can't be used until the login process has been completed. The browser won't render such elements. This attribute must not be used to hide content that could legitimately be shown.
		hidden?,
		/// Defines a unique identifier (ID) which must be unique in the whole document. Its purpose is to identify the element when linking (using a fragment identifier), scripting, or styling (with CSS).
		id,
		/// Provides a hint to browsers as to the type of virtual keyboard configuration to use when editing this element or its contents. Used primarily on `<input>` elements, but is usable on any element while in contenteditable mode.
		inputmode,
		/// Allows you to specify that a standard HTML element should behave like a registered custom built-in element (see Using custom elements for more details).
		is,
		/// The unique, global identifier of an item.
		itemid,
		/// Used to add properties to an item. Every HTML element may have an itemprop attribute specified, where an itemprop consists of a name and value pair.
		itemprop,
		/// Properties that are not descendants of an element with the itemscope attribute can be associated with the item using an itemref. It provides a list of element ids (not itemids) with additional properties elsewhere in the document.
		itemref,
		/// itemscope (usually) works along with itemtype to specify that the HTML contained in a block is about a particular item. itemscope creates the Item and defines the scope of the itemtype associated with it. itemtype is a valid URL of a vocabulary (such as schema.org) that describes the item and its properties context.
		itemscope?,
		/// Specifies the URL of the vocabulary that will be used to define itemprops (item properties) in the data structure. itemscope is used to set the scope of where in the data structure the vocabulary set by itemtype will be active.
		itemtype,
		/// Helps define the language of an element: the language that non-editable elements are in, or the language that editable elements should be written in by the user. The attribute contains one \u201clanguage tag\u201d (made of hyphen-separated \u201clanguage subtags\u201d) in the format defined in Tags for Identifying Languages (BCP47). xml:lang has priority over it.
		lang,
		/// A cryptographic nonce ("number used once") which can be used by Content Security Policy to determine whether or not a given fetch will be allowed to proceed.
		nonce,
		/// A space-separated list of the part names of the element. Part names allows CSS to select and style specific elements in a shadow tree via the ::part pseudo-element.
		part,
		/// Assigns a slot in a shadow DOM shadow tree to an element: An element with a slot attribute is assigned to the slot created by the `<slot>` element whose name attribute's value matches that slot attribute's value.
		slot,
		/**
		An enumerated attribute defines whether the element may be checked for spelling errors. It may have the following values:
			* true, which indicates that the element should be, if possible, checked for spelling errors;
			* false, which indicates that the element should not be checked for spelling errors.
		*/
		spellcheck,
		// /// Contains CSS styling declarations to be applied to the element. Note that it is recommended for styles to be defined in a separate file or files. This attribute and the `<style>` element have mainly the purpose of allowing for quick styling, for example for testing purposes.
		// style,
		/**
		An integer attribute indicating if the element can take input focus (is focusable), if it should participate to sequential keyboard navigation, and if so, at what position. It can take several values:
			* a negative value means that the element should be focusable, but should not be reachable via sequential keyboard navigation;
			* 0 means that the element should be focusable and reachable via sequential keyboard navigation, but its relative order is defined by the platform convention;
			* a positive value means that the element should be focusable and reachable via sequential keyboard navigation; the order in which the elements are focused is the increasing value of the tabindex. If several elements share the same tabindex, their relative order follows their relative positions in the document.
		*/
		tabindex,
		/// Contains a text representing advisory information related to the element it belongs to. Such information can typically, but not necessarily, be presented to the user as a tooltip.
		title,
		/**
		An enumerated attribute that is used to specify whether an element's attribute values and the values of its Text node children are to be translated when the page is localized, or whether to leave them unchanged. It can have the following values:
			* empty string and yes, which indicates that the element will be translated.
			* no, which indicates that the element will not be translated.
		*/
		translate,
	))
	.unwrap()
}

fn html_element_events() -> Events {
	syn::parse2(quote!(
		/// Fires on a <dialog> when the user instructs the browser that they wish to dismiss the current open dialog. For example, the browser might fire this event when the user presses the Esc key or clicks a "Close dialog" button which is part of the browser's UI.
		cancel: Event,
		/// Fired when a resource failed to load, or can't be used. For example, if a script has an execution error or an image can't be found or is invalid.
		error: Event,
		/// Fired when the document view or an element has been scrolled.
		scroll: Event,
		/// Fired when some text has been selected.
		select: Event,
		/// Fired when a contextmenu event was fired on/bubbled to an element that has a contextmenu attribute.
		show: Event,
		/// Fired when the user rotates a wheel button on a pointing device (typically a mouse).
		wheel: Event,
		/// Fired when the user initiates a copy action through the browser's user interface.
		copy: Event,
		/// Fired when the user initiates a cut action through the browser's user interface.
		cut: Event,
		/// Fired when the user initiates a paste action through the browser's user interface.
		paste: Event,
		/// Fired when a text composition system such as an input method editor completes or cancels the current composition session.
		compositionend: CompositionEvent,
		/// Fired when a text composition system such as an input method editor starts a new composition session.
		compositionstart: CompositionEvent,
		/// Fired when a new character is received in the context of a text composition session controlled by a text composition system such as an input method editor.
		compositionupdate: CompositionEvent,
		/// Fired when an element has lost focus.
		blur: FocusEvent,
		/// Fired when an element has gained focus.
		focus: FocusEvent,
		/// Fired when an element is about to gain focus.
		focusin: FocusEvent,
		/// Fired when an element is about to lose focus.
		focusout: FocusEvent,
		/// Sent to an Element when it transitions into or out of full-screen mode.
		fullscreenchange: Event,
		/// Sent to an Element if an error occurs while attempting to switch it into or out of full-screen mode.
		fullscreenerror: Event,
		/// Fired when a key is pressed.
		keydown: KeyboardEvent,
		/// Fired when a key that produces a character value is pressed down.
		keypress: KeyboardEvent,
		/// Fired when a key is released.
		keyup: KeyboardEvent,
		/// Fired when a non-primary pointing device button (e.g., any mouse button other than the left button) has been pressed and released on an element.
		auxclick: MouseEvent,
		/// Fired when a pointing device button (e.g., a mouse's primary button) is pressed and released on a single element.
		click: MouseEvent,
		/// Fired when the user attempts to open a context menu.
		contextmenu: MouseEvent,
		/// Fired when a pointing device button (e.g., a mouse's primary button) is clicked twice on a single element.
		dblclick: MouseEvent,
		/// Occurs when an element is activated, for instance, through a mouse click or a keypress.
		DOMActivate: MouseEvent,
		/// Fired when a pointing device button is pressed on an element.
		mousedown: MouseEvent,
		/// Fired when a pointing device (usually a mouse) is moved over the element that has the listener attached.
		mouseenter: MouseEvent,
		/// Fired when the pointer of a pointing device (usually a mouse) is moved out of an element that has the listener attached to it.
		mouseleave: MouseEvent,
		/// Fired when a pointing device (usually a mouse) is moved while over an element.
		mousemove: MouseEvent,
		/// Fired when a pointing device (usually a mouse) is moved off the element to which the listener is attached or off one of its children.
		mouseout: MouseEvent,
		/// Fired when a pointing device is moved onto the element to which the listener is attached or onto one of its children.
		mouseover: MouseEvent,
		/// Fired when a pointing device button is released on an element.
		mouseup: MouseEvent,
		/// Fired each time the amount of pressure changes on the trackpadtouchscreen.
		webkitmouseforcechanged: MouseEvent,
		/// Fired after the mousedown event as soon as sufficient pressure has been applied to qualify as a "force click".
		webkitmouseforcedown: MouseEvent,
		/// Fired before the mousedown event.
		webkitmouseforcewillbegin: MouseEvent,
		/// Fired after the webkitmouseforcedown event as soon as the pressure has been reduced sufficiently to end the "force click".
		webkitmouseforceup: MouseEvent,
		/// Fired when one or more touch points have been disrupted in an implementation-specific manner (for example, too many touch points are created).
		touchcancel: TouchEvent,
		/// Fired when one or more touch points are removed from the touch surface.
		touchend: TouchEvent,
		/// Fired when one or more touch points are moved along the touch surface.
		touchmove: TouchEvent,
		/// Fired when one or more touch points are placed on the touch surface.
		touchstart: TouchEvent,
	))
	.unwrap()
}
