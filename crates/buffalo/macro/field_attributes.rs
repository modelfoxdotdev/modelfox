use quote::quote;
use syn::spanned::Spanned;

pub struct FieldAttributes {
	pub id: u64,
	pub required: bool,
}

pub enum FieldIdValue {
	U8(u8),
	U16(u16),
}

impl quote::ToTokens for FieldIdValue {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let code = match self {
			FieldIdValue::U8(value) => quote! { #value },
			FieldIdValue::U16(value) => quote! { #value },
		};
		code.to_tokens(tokens);
	}
}

pub fn field_attributes(field: &syn::Field) -> syn::Result<FieldAttributes> {
	let attr = field
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("buffalo"))
		.ok_or_else(|| syn::Error::new(field.span(), "buffalo attribute is required"))?;
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
	let mut id = None;
	let mut required = None;
	for item in list.nested.iter() {
		match item {
			syn::NestedMeta::Meta(syn::Meta::NameValue(item)) if item.path.is_ident("id") => {
				let value = if let syn::Lit::Int(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(&item, "value for attribute \"id\" must be an integer")
				})?;
				let value = value.base10_parse().map_err(|_| {
					syn::Error::new_spanned(&item, "value for attribute \"id\" must be an integer")
				})?;
				id = Some(value);
			}
			syn::NestedMeta::Meta(syn::Meta::Path(item)) if item.is_ident("required") => {
				required = Some(true);
			}
			_ => {}
		}
	}
	let id = id.ok_or_else(|| {
		syn::Error::new_spanned(&list.nested, "an attribute with key \"id\" is required")
	})?;
	Ok(FieldAttributes {
		id,
		required: required.unwrap_or(false),
	})
}
