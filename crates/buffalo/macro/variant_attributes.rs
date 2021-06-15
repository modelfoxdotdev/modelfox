use quote::quote;
use syn::spanned::Spanned;

pub struct VariantAttributes {
	pub id: u64,
}

pub fn variant_attributes(variant: &syn::Variant) -> syn::Result<VariantAttributes> {
	let attr = variant
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("buffalo"))
		.ok_or_else(|| syn::Error::new(variant.span(), "buffalo attribute is required"))?;
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
	let mut id = None;
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
			_ => {}
		}
	}
	let id = id.ok_or_else(|| {
		syn::Error::new_spanned(&list.nested, "an attribute with key \"id\" is required")
	})?;
	Ok(VariantAttributes { id })
}

pub fn variant_ty(variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream> {
	match &variant.fields {
		syn::Fields::Named(_) => Err(syn::Error::new_spanned(
			variant,
			"variants with named fields are not supported",
		)),
		syn::Fields::Unnamed(fields) => {
			let variant_ty = &fields.unnamed.first().unwrap().ty;
			Ok(quote! { #variant_ty })
		}
		syn::Fields::Unit => Ok(quote! { () }),
	}
}
