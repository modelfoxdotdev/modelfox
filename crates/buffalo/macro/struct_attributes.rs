use quote::quote;

pub const DEFAULT_STRUCT_INDEX_TYPE: StructIndexType = StructIndexType::U16;

pub struct StructAttributes {
	pub size: StructSize,
	pub index_type: Option<StructIndexType>,
}

pub enum StructSize {
	Dynamic,
	Static,
}

pub enum StructIndexType {
	U8,
	U16,
}

impl quote::ToTokens for StructIndexType {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let code = match self {
			StructIndexType::U8 => quote! { u8 },
			StructIndexType::U16 => quote! { u16 },
		};
		code.to_tokens(tokens);
	}
}

pub fn struct_attributes(
	attrs: &[syn::Attribute],
	span: proc_macro2::Span,
) -> syn::Result<StructAttributes> {
	let attr = attrs
		.iter()
		.find(|attr| attr.path.is_ident("buffalo"))
		.ok_or_else(|| syn::Error::new(span, "buffalo attribute is required"))?;
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"attribute arguments must be a list",
			))
		}
	};
	let mut size = None;
	let mut index_type = None;
	for item in list.nested.iter() {
		match item {
			syn::NestedMeta::Meta(syn::Meta::NameValue(item)) if item.path.is_ident("size") => {
				size = match &item.lit {
					syn::Lit::Str(value) if value.value() == "dynamic" => Some(StructSize::Dynamic),
					syn::Lit::Str(value) if value.value() == "static" => Some(StructSize::Static),
					_ => {
						return Err(syn::Error::new_spanned(
							&item,
							"value for attribute \"size\" must be \"dynamic\" or \"static\"",
						))
					}
				};
			}
			syn::NestedMeta::Meta(syn::Meta::NameValue(item))
				if item.path.is_ident("index_size") =>
			{
				let value = if let syn::Lit::Int(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"index_size\" must be an integer",
					)
				})?;
				let value = value.base10_parse().map_err(|_| {
					syn::Error::new_spanned(
						&item,
						"value for attribute \"index_size\" must be an integer",
					)
				})?;
				let value = match value {
					1 => StructIndexType::U8,
					2 => StructIndexType::U16,
					_ => {
						return Err(syn::Error::new_spanned(
							&item,
							"value for attribute \"index_size\" must be 1 or 2",
						))
					}
				};
				index_type = Some(value);
			}
			_ => return Err(syn::Error::new_spanned(&item, "unknown attribute")),
		}
	}
	let size = size
		.ok_or_else(|| syn::Error::new_spanned(&list.nested, "\"size\" attribute is required"))?;
	Ok(StructAttributes { size, index_type })
}
