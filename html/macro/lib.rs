use quote::{format_ident, quote, quote_spanned};
use syn::{ext::IdentExt, spanned::Spanned};

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	html_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn html_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let node: Node = syn::parse2(input)?;
	let code = quote! { #node };
	Ok(code)
}

enum Node {
	String(syn::LitStr),
	Block(syn::Block),
	Fragment(Fragment),
	Element(Element),
	Component(Component),
}

struct Fragment {
	pub children: Vec<Node>,
}

struct Element {
	pub name: syn::Ident,
	pub attributes: Vec<Attribute>,
	pub children: Vec<Node>,
	pub self_closing: bool,
}

enum Attribute {
	Shorthand(AttributeKey),
	Longhand(AttributeKey, AttributeValue),
}

type AttributeKey = syn::punctuated::Punctuated<syn::Ident, syn::Token![-]>;

enum AttributeValue {
	String(syn::LitStr),
	Block(syn::Block),
}

struct Component {
	pub path: syn::Path,
	pub props: Props,
	pub children: Option<Vec<Node>>,
	pub closing_path: Option<syn::Path>,
}

enum Props {
	All(syn::Block),
	Inline(Vec<Prop>),
}

enum Prop {
	Shorthand(PropKey, bool),
	Longhand(PropKey, bool, PropValue),
}

type PropKey = syn::Ident;

enum PropValue {
	String(syn::LitStr),
	Block(syn::Block),
}

impl syn::parse::Parse for Node {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		if let Ok(string) = input.parse::<syn::LitStr>() {
			return Ok(Self::String(string));
		}
		if let Ok(block) = input.parse::<syn::Block>() {
			return Ok(Self::Block(block));
		}
		if input.peek(syn::Token![<]) && input.peek2(syn::Token![>]) {
			return Ok(Self::Fragment(input.parse::<Fragment>()?));
		}
		if input.peek(syn::Token![<]) {
			let fork = input.fork();
			fork.parse::<syn::Token![<]>()?;
			let path = fork.parse::<syn::Path>()?;
			let is_element = path
				.get_ident()
				.map(|ident| {
					ident
						.to_string()
						.chars()
						.next()
						.unwrap()
						.is_ascii_lowercase()
				})
				.unwrap_or(false);
			if is_element {
				return Ok(Self::Element(input.parse::<Element>()?));
			} else {
				return Ok(Self::Component(input.parse::<Component>()?));
			}
		}
		Err(syn::Error::new(input.span(), "failed to parse node"))
	}
}

impl syn::parse::Parse for Fragment {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		input.parse::<syn::Token![<]>()?;
		input.parse::<syn::Token![>]>()?;
		let mut children = Vec::new();
		while !(input.peek(syn::Token![<]) && input.peek2(syn::Token![/])) {
			let child = input.parse::<Node>()?;
			children.push(child);
		}
		input.parse::<syn::Token![<]>()?;
		input.parse::<syn::Token![/]>()?;
		input.parse::<syn::Token![>]>()?;
		Ok(Self { children })
	}
}

impl syn::parse::Parse for Element {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		input.parse::<syn::Token![<]>()?;
		let name = input.parse::<syn::Ident>()?;
		let mut attributes = Vec::new();
		while !(input.peek(syn::Token![>]) || input.peek(syn::Token![/])) {
			let key = AttributeKey::parse_separated_nonempty_with(input, syn::Ident::parse_any)?;
			if !input.peek(syn::Token![=]) {
				attributes.push(Attribute::Shorthand(key));
			} else {
				input.parse::<syn::Token![=]>()?;
				let value = input
					.parse::<syn::LitStr>()
					.map(AttributeValue::String)
					.or_else(|_| input.parse::<syn::Block>().map(AttributeValue::Block))?;
				attributes.push(Attribute::Longhand(key, value));
			}
		}
		let mut children = Vec::new();
		let self_closing = input.peek(syn::Token![/]);
		if self_closing {
			input.parse::<syn::Token![/]>()?;
			input.parse::<syn::Token![>]>()?;
		} else {
			input.parse::<syn::Token![>]>()?;
			while !(input.peek(syn::Token![<]) && input.peek2(syn::Token![/])) {
				let child = input.parse::<Node>()?;
				children.push(child);
			}
			input.parse::<syn::Token![<]>()?;
			input.parse::<syn::Token![/]>()?;
			let close_name = input.parse::<syn::Ident>()?;
			if close_name != name {
				panic!();
			}
			input.parse::<syn::Token![>]>()?;
		}
		Ok(Self {
			name,
			attributes,
			children,
			self_closing,
		})
	}
}

impl syn::parse::Parse for Component {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		input.parse::<syn::Token![<]>()?;
		let path = input.parse::<syn::Path>()?;
		let props = if let Ok(block) = input.parse::<syn::Block>() {
			Props::All(block)
		} else {
			let mut props = Vec::new();
			while !(input.peek(syn::Token![>]) || input.peek(syn::Token![/])) {
				let key = input.parse::<syn::Ident>()?;
				let optional = input.parse::<syn::Token![?]>().is_ok();
				if input.peek(syn::Token![=]) {
					input.parse::<syn::Token![=]>()?;
					let value = input
						.parse::<syn::LitStr>()
						.map(PropValue::String)
						.or_else(|_| input.parse::<syn::Block>().map(PropValue::Block))?;
					props.push(Prop::Longhand(key, optional, value));
				} else {
					props.push(Prop::Shorthand(key, optional));
				}
			}
			Props::Inline(props)
		};
		let mut children = Vec::new();
		let (children, closing_path) = if input.peek(syn::Token![/]) {
			input.parse::<syn::Token![/]>()?;
			input.parse::<syn::Token![>]>()?;
			(None, None)
		} else {
			input.parse::<syn::Token![>]>()?;
			while !(input.peek(syn::Token![<]) && input.peek2(syn::Token![/])) {
				let child = input.parse::<Node>()?;
				children.push(child);
			}
			input.parse::<syn::Token![<]>()?;
			input.parse::<syn::Token![/]>()?;
			let closing_path = input.parse::<syn::Path>()?;
			if closing_path != path {
				return Err(syn::Error::new(
					closing_path.span(),
					format!(
						"closing path {:?} does not match opening path {:?}",
						&closing_path, &path
					),
				));
			}
			input.parse::<syn::Token![>]>()?;
			(Some(children), Some(closing_path))
		};
		Ok(Self {
			path,
			props,
			children,
			closing_path,
		})
	}
}

impl quote::ToTokens for Node {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::String(string) => string.to_tokens(tokens),
			Self::Block(block) => block.to_tokens(tokens),
			Self::Fragment(fragment) => fragment.to_tokens(tokens),
			Self::Element(element) => element.to_tokens(tokens),
			Self::Component(component) => component.to_tokens(tokens),
		}
	}
}

impl quote::ToTokens for Fragment {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let children = self.children.iter();
		let children = quote! { vec![#(#children.into()),*] };
		let code = quote! {
			html::Node::Fragment(html::FragmentNode {
				children: #children,
			})
		};
		code.to_tokens(tokens);
	}
}

impl quote::ToTokens for Element {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let name = self.name.to_string();
		let attributes = self.attributes.iter().map(|attribute| {
			let (key, value) = match attribute {
				Attribute::Shorthand(key) => (key, quote! { #key.into() }),
				Attribute::Longhand(key, value) => match value {
					AttributeValue::String(string) => (key, quote! { #string.into() }),
					AttributeValue::Block(block) => (key, quote! {  #block.into() }),
				},
			};
			let key = key
				.iter()
				.map(|key| key.to_string())
				.collect::<Vec<_>>()
				.join("-");
			quote! { (#key, #value) }
		});
		let attributes = quote! { vec![#(#attributes),*] };
		let children = self.children.iter();
		let children = quote! { vec![#(#children.into()),*] };
		let self_closing = self.self_closing;
		let code = quote! {
			html::Node::Element(html::ElementNode {
				name: #name,
				attributes: #attributes.into_iter().collect(),
				children: #children,
				self_closing: #self_closing,
			})
		};
		code.to_tokens(tokens);
	}
}

impl quote::ToTokens for Component {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let path = &self.path;
		let closing_path = &self.closing_path.as_ref().unwrap_or(&self.path);
		let props = match &self.props {
			Props::All(block) => {
				quote! {
					let _props = #block;
				}
			}
			Props::Inline(props) => {
				let mut required_fields = Vec::new();
				let mut optional_fields = Vec::new();
				for prop in props.iter() {
					let (field, optional) = match prop {
						Prop::Shorthand(key, optional) => {
							let span = key.span();
							let field = quote_spanned! {span=>
								#key: #key
							};
							(field, *optional)
						}
						Prop::Longhand(key, optional, value) => {
							let field = match value {
								PropValue::String(string) => {
									let span = key.span();
									quote_spanned! {span=>
										#key: html::ConvertFromStr::convert_from_str(#string)
									}
								}
								PropValue::Block(block) => {
									let span = key.span();
									quote_spanned! {span=>
										#key: #block
									}
								}
							};
							(field, *optional)
						}
					};
					if !optional {
						required_fields.push(field);
					} else {
						optional_fields.push(field);
					}
				}
				quote! {
					type _Props = <#path as html::Component>::Props;
					type _RequiredProps = <_Props as html::Props>::RequiredProps;
					type _OptionalProps = <_Props as html::Props>::OptionalProps;
					#[allow(clippy::useless_conversion)]
					let _required = _RequiredProps {
						#(#required_fields,)*
					};
					#[allow(clippy::useless_conversion, clippy::needless_update)]
					let _optional = _OptionalProps {
						#(#optional_fields,)*
						..Default::default()
					};
					let _props = <_Props as html::Props>::combine(_required, _optional);
				}
			}
		};
		let children = self
			.children
			.as_ref()
			.map(|children| {
				quote! { vec![#(#children.into()),*] }
			})
			.unwrap_or_else(|| quote! { vec![] });
		let span = self.path.span();
		let code = quote_spanned! {span=>
			html::Node::Component(html::ComponentNode::Unrendered({
				#props
				let _children = #children;
				Some(Box::new(move || {
					<#closing_path as html::Component>::render(_props, _children)
				}))
			}))
		};
		code.to_tokens(tokens);
	}
}

#[proc_macro_derive(Props, attributes(optional))]
pub fn props(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	props_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn props_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
	let vis = &input.vis;
	let ident = &input.ident;
	let data = match &input.data {
		syn::Data::Struct(data) => data,
		_ => {
			return Err(syn::Error::new_spanned(
				input,
				"this macro can only be used on a struct",
			))
		}
	};
	let required_props_ident = format_ident!("{}Required", ident);
	let optional_props_ident = format_ident!("{}Optional", ident);
	let mut required_props_fields = Vec::new();
	let mut optional_props_fields = Vec::new();
	for field in data.fields.iter() {
		let optional = field
			.attrs
			.iter()
			.any(|attr| attr.path.is_ident("optional"));
		let field_ident = &field.ident;
		let field_ty = &field.ty;
		if !optional {
			required_props_fields.push(quote! {
				pub #field_ident: #field_ty
			});
		} else {
			optional_props_fields.push(quote! {
				pub #field_ident: #field_ty
			});
		}
	}
	let mut required_props_combine = Vec::new();
	let mut optional_props_combine = Vec::new();
	for field in data.fields.iter() {
		let optional = field
			.attrs
			.iter()
			.any(|attr| attr.path.is_ident("optional"));
		let field_ident = &field.ident;
		if !optional {
			required_props_combine.push(quote! {
				#field_ident: required.#field_ident
			});
		} else {
			optional_props_combine.push(quote! {
				#field_ident: optional.#field_ident
			});
		}
	}
	let code = quote! {
		#vis struct #required_props_ident {
			#(#required_props_fields,)*
		}
		#[derive(Default)]
		#vis struct #optional_props_ident {
			#(#optional_props_fields,)*
		}
		impl html::Props for #ident {
			type RequiredProps = #required_props_ident;
			type OptionalProps = #optional_props_ident;
			fn combine(required: Self::RequiredProps, optional: Self::OptionalProps) -> Self {
				Self {
					#(#required_props_combine,)*
					#(#optional_props_combine,)*
				}
			}
		}
	};
	Ok(code)
}

#[proc_macro_attribute]
pub fn component(
	_attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	component_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn component_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let f: syn::ItemFn = syn::parse2(input)?;
	let visibility = &f.vis;
	let struct_ident = &f.sig.ident;
	let inputs = f.sig.inputs.iter().collect::<Vec<_>>();
	let (props_type, empty_props_struct) = match inputs.as_slice() {
		[syn::FnArg::Typed(typed)] => {
			let ty = &typed.ty;
			(quote! { #ty }, None)
		}
		[] => {
			let props_struct_ident = format_ident!("{}Props", struct_ident);
			(
				quote! { #props_struct_ident },
				Some(quote! {
					#[derive(html::Props)]
					#visibility struct #props_struct_ident {}
				}),
			)
		}
		_ => {
			return Err(syn::Error::new(
				f.span(),
				"component must have one props argument",
			))
		}
	};
	let block = &f.block;
	let code = quote! {
		#visibility struct #struct_ident;
		#empty_props_struct
		impl html::Component for #struct_ident {
			type Props = #props_type;
			fn render(props: Self::Props, children: Vec<html::Node>) -> html::Node {
				#block
			}
		}
	};
	Ok(code)
}
