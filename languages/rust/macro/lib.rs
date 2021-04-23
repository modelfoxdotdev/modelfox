use quote::quote;
use syn::spanned::Spanned;

#[proc_macro]
pub fn predict_input(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	predict_input_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

struct PredictInput {
	entries: Vec<PredictInputEntry>,
}

impl syn::parse::Parse for PredictInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let entries =
			<syn::punctuated::Punctuated<PredictInputEntry, syn::Token![,]>>::parse_terminated(
				input,
			)?;
		let entries = entries.into_iter().collect();
		Ok(PredictInput { entries })
	}
}

struct PredictInputEntry {
	column_name: syn::LitStr,
	value: syn::Expr,
}

impl syn::parse::Parse for PredictInputEntry {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let column_name = input.parse()?;
		input.parse::<syn::Token![:]>()?;
		let value = input.parse()?;
		Ok(PredictInputEntry { column_name, value })
	}
}

fn predict_input_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: PredictInput = syn::parse2(input)?;
	let column_names = input
		.entries
		.iter()
		.map(|entry| &entry.column_name)
		.collect::<Vec<_>>();
	let values = input
		.entries
		.iter()
		.map(|entry| &entry.value)
		.collect::<Vec<_>>();
	let code = quote! {{
		let mut map = std::collections::BTreeMap::new();
		#(
			map.insert(#column_names.to_owned(), #values.into());
		)*
		tangram::PredictInput(map)
	}};
	Ok(code)
}

#[proc_macro_derive(PredictInput, attributes(tangram))]
pub fn predict_input_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	predict_input_derive_macro_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn predict_input_derive_macro_impl(
	input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
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
	let insert_statements = data
		.fields
		.iter()
		.map(|field| {
			let field_ident = field
				.ident
				.as_ref()
				.ok_or_else(|| syn::Error::new(field.span(), "field must have ident"))?;
			let column_name =
				predict_input_field_rename(field)?.unwrap_or_else(|| field_ident.to_string());
			let code = quote! {
				map.insert(#column_name.to_owned(), value.#field_ident.into());
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let code = quote! {
		impl From<#ident> for tangram::PredictInput {
			fn from(value: #ident) -> tangram::PredictInput {
				let mut map = std::collections::BTreeMap::new();
				#(#insert_statements)*
				tangram::PredictInput(map)
			}
		}
	};
	Ok(code)
}

fn predict_input_field_rename(field: &syn::Field) -> syn::Result<Option<String>> {
	let attr = field
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram"));
	let attr = if let Some(attr) = attr {
		attr
	} else {
		return Ok(None);
	};
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"tangram attribute must contain a list",
			))
		}
	};
	let mut rename = None;
	for item in list.nested.iter() {
		match item {
			syn::NestedMeta::Meta(syn::Meta::NameValue(item)) if item.path.is_ident("rename") => {
				let value = if let syn::Lit::Str(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(&item, "value for attribute \"value\" must be a string")
				})?;
				rename = Some(value);
			}
			_ => {}
		}
	}
	let rename = rename.ok_or_else(|| {
		syn::Error::new_spanned(&list.nested, "an attribute with key \"value\" is required")
	})?;
	let rename = rename.value();
	Ok(Some(rename))
}

#[proc_macro_derive(PredictInputValue, attributes(tangram))]
pub fn predict_input_value_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	predict_input_value_derive_macro_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn predict_input_value_derive_macro_impl(
	input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
	let ident = &input.ident;
	let data = match &input.data {
		syn::Data::Enum(data) => data,
		_ => {
			return Err(syn::Error::new(
				input.span(),
				"this macro can only be used on an enum",
			))
		}
	};
	let match_arms = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_value = predict_input_value_variant_value(variant)?
				.unwrap_or_else(|| variant_ident.to_string());
			let code = quote! { #ident::#variant_ident => #variant_value };
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let code = quote! {
		impl From<#ident> for tangram::PredictInputValue {
			fn from(value: #ident) -> tangram::PredictInputValue {
				let value = match value {
					#(#match_arms,)*
				};
				tangram::PredictInputValue::String(value.to_owned())
			}
		}
	};
	Ok(code)
}

fn predict_input_value_variant_value(variant: &syn::Variant) -> syn::Result<Option<String>> {
	let attr = variant
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram"));
	let attr = if let Some(attr) = attr {
		attr
	} else {
		return Ok(None);
	};
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"tangram attribute must contain a list",
			))
		}
	};
	let mut input_value = None;
	for item in list.nested.iter() {
		match item {
			syn::NestedMeta::Meta(syn::Meta::NameValue(item)) if item.path.is_ident("value") => {
				let value = if let syn::Lit::Str(value) = &item.lit {
					Some(value)
				} else {
					None
				};
				let value = value.ok_or_else(|| {
					syn::Error::new_spanned(&item, "value for attribute \"value\" must be a string")
				})?;
				input_value = Some(value);
			}
			_ => {}
		}
	}
	let input_value = input_value.ok_or_else(|| {
		syn::Error::new_spanned(&list.nested, "an attribute with key \"value\" is required")
	})?;
	let input_value = input_value.value();
	Ok(Some(input_value))
}
