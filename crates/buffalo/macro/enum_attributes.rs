use quote::quote;
use std::convert::TryInto;
use syn::spanned::Spanned;

pub type PointerType = u64;
pub const DEFAULT_ENUM_DISCRIMINANT_TYPE: EnumDiscriminantType = EnumDiscriminantType::U8;

pub struct EnumAttributes {
	pub size: EnumSize,
	pub discriminant_type: Option<EnumDiscriminantType>,
	pub value_size: Option<PointerType>,
}

pub enum EnumSize {
	Dynamic,
	Static,
}

pub enum EnumDiscriminantType {
	U8,
	U16,
}

impl EnumDiscriminantType {
	pub fn value(&self, value: u64) -> EnumDiscriminantValue {
		match self {
			EnumDiscriminantType::U8 => EnumDiscriminantValue::U8(value.try_into().unwrap()),
			EnumDiscriminantType::U16 => EnumDiscriminantValue::U16(value.try_into().unwrap()),
		}
	}
}

impl quote::ToTokens for EnumDiscriminantType {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let code = match self {
			EnumDiscriminantType::U8 => quote! { u8 },
			EnumDiscriminantType::U16 => quote! { u16 },
		};
		code.to_tokens(tokens);
	}
}

pub enum EnumDiscriminantValue {
	U8(u8),
	U16(u16),
}

impl quote::ToTokens for EnumDiscriminantValue {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let code = match self {
			EnumDiscriminantValue::U8(value) => quote! { #value },
			EnumDiscriminantValue::U16(value) => quote! { #value },
		};
		code.to_tokens(tokens);
	}
}

pub fn enum_attributes(input: &syn::DeriveInput) -> syn::Result<EnumAttributes> {
	let attr = input
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("buffalo"))
		.ok_or_else(|| syn::Error::new(input.span(), "buffalo attribute is required"))?;
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"buffalo attribute must contain a list",
			))
		}
	};
	let mut size = None;
	let mut discriminant_type = None;
	let mut value_size = None;
	for item in list.nested.iter() {
		match item {
			syn::NestedMeta::Meta(syn::Meta::NameValue(item)) if item.path.is_ident("size") => {
				size = match &item.lit {
					syn::Lit::Str(value) if value.value() == "dynamic" => Some(EnumSize::Dynamic),
					syn::Lit::Str(value) if value.value() == "static" => Some(EnumSize::Static),
					_ => {
						return Err(syn::Error::new_spanned(
							&item,
							"value for attribute \"size\" must be \"dynamic\" or \"static\"",
						))
					}
				};
			}
			syn::NestedMeta::Meta(syn::Meta::NameValue(item))
				if item.path.is_ident("discriminant_size") =>
			{
				let value = if let syn::Lit::Int(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"discriminant_size\" must be an integer",
					)
				})?;
				let value = value.base10_parse().map_err(|_| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"discriminant_size\" must be an integer",
					)
				})?;
				let value = match value {
					1 => EnumDiscriminantType::U8,
					2 => EnumDiscriminantType::U16,
					_ => {
						return Err(syn::Error::new_spanned(
							&item,
							"value for attribute \"discriminant_size\" must be 1 or 2",
						))
					}
				};
				discriminant_type = Some(value);
			}
			syn::NestedMeta::Meta(syn::Meta::NameValue(item))
				if item.path.is_ident("value_size") =>
			{
				let value = if let syn::Lit::Int(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"value_size\" must be an integer",
					)
				})?;
				let value = value.base10_parse().map_err(|_| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"value_size\" must be an integer",
					)
				})?;
				value_size = Some(value);
			}
			_ => return Err(syn::Error::new_spanned(&item, "unknown attribute")),
		}
	}
	let size = size
		.ok_or_else(|| syn::Error::new_spanned(&list.nested, "\"size\" attribute is required"))?;
	Ok(EnumAttributes {
		size,
		discriminant_type,
		value_size,
	})
}
