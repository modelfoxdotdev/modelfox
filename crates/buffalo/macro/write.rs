use crate::{
	enum_attributes::{
		enum_attributes, EnumAttributes, EnumDiscriminantType, EnumSize, PointerType,
		DEFAULT_ENUM_DISCRIMINANT_TYPE,
	},
	field_attributes::field_attributes,
	struct_attributes::{
		struct_attributes, StructAttributes, StructIndexType, StructSize, DEFAULT_STRUCT_INDEX_TYPE,
	},
	variant_attributes::{variant_attributes, variant_ty},
};
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::spanned::Spanned;

pub fn write(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
	let data = &input.data;
	match data {
		syn::Data::Struct(data) => struct_write(&input, data),
		syn::Data::Enum(data) => enum_write(&input, data),
		_ => Err(syn::Error::new_spanned(
			input,
			"this macro can only be used on a struct or enum",
		)),
	}
}

fn struct_write(
	input: &syn::DeriveInput,
	data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
	let attributes = struct_attributes(&input.attrs, input.span())?;
	match attributes.size {
		StructSize::Dynamic => dynamic_struct_write(input, &attributes, data),
		StructSize::Static => static_struct_write(input, &attributes, data),
	}
}

fn enum_write(
	input: &syn::DeriveInput,
	data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
	let attributes = enum_attributes(&input)?;
	match attributes.size {
		EnumSize::Dynamic => dynamic_enum_write(input, &attributes, data),
		EnumSize::Static => static_enum_write(input, &attributes, data),
	}
}

fn static_struct_write(
	input: &syn::DeriveInput,
	_attributes: &StructAttributes,
	data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
	let visibility = &input.vis;
	let ident = &input.ident;
	let writer_ident = format_ident!("{}Writer", ident);
	let writer_fields = data
		.fields
		.iter()
		.map(|field| {
			let field_ident = &field.ident;
			let field_ty = &field.ty;
			quote!(#visibility #field_ident: <#field_ty as buffalo::WriteType>::WriteType)
		})
		.collect::<Vec<_>>();
	let writer_decl = quote! {
		#visibility struct #writer_ident {
			#(#writer_fields,)*
		}
	};
	let size_exprs = data
		.fields
		.iter()
		.map(|field| {
			let field_ty = &field.ty;
			quote! { <<#field_ty as buffalo::ReadType>::ReadType as buffalo::StaticSize>::STATIC_SIZE }
		})
		.collect::<Vec<_>>();
	let static_size_impl = quote! {
		impl buffalo::StaticSize for #writer_ident {
			const STATIC_SIZE: buffalo::PointerType = #(#size_exprs)+*;
		}
	};
	let fields = data
		.fields
		.iter()
		.map(|field| {
			let field_attributes = field_attributes(&field)?;
			let field_id = field_attributes.id;
			Ok((field_id, field))
		})
		.collect::<syn::Result<BTreeMap<_, _>>>()?;
	let writer_statements = fields.values().map(|field| {
		let field_ident = &field.ident;
		quote! {
			writer.write(&self.#field_ident);
		}
	});
	let write_impl = quote! {
		impl buffalo::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut buffalo::Writer) -> buffalo::Position<Self::Output> {
				let position = writer.position();
				#(#writer_statements)*
				position
			}
		}
	};
	let write_type_impl = quote! {
		impl buffalo::WriteType for #ident {
			type WriteType = #writer_ident;
		}
	};
	let code = quote! {
		#writer_decl
		#static_size_impl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}

fn dynamic_enum_write(
	input: &syn::DeriveInput,
	attributes: &EnumAttributes,
	data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
	let visibility = &input.vis;
	let ident = &input.ident;
	let discriminant_type = attributes
		.discriminant_type
		.as_ref()
		.unwrap_or(&DEFAULT_ENUM_DISCRIMINANT_TYPE);
	let writer_ident = format_ident!("{}Writer", ident);
	let writer_variants = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			if matches!(variant.fields, syn::Fields::Unit) {
				Ok(quote!(#variant_ident))
			} else {
				let variant_ty = variant_ty(&variant)?;
				Ok(quote!(#variant_ident(<#variant_ty as buffalo::WriteType>::WriteType)))
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let writer_decl = quote! {
		#visibility enum #writer_ident {
			#(#writer_variants,)*
		}
	};
	let write_match_arms = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_attributes = variant_attributes(&variant)?;
			let variant_id = discriminant_type.value(variant_attributes.id);
			if matches!(variant.fields, syn::Fields::Unit) {
				let code = quote! {
					Self::#variant_ident => {
						let position = writer.position();
						writer.write(&#variant_id);
						position
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					Self::#variant_ident(s) => {
						let position = writer.position();
						writer.write(&#variant_id);
						writer.write(s);
						position
					}
				};
				Ok(code)
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let write_impl = quote! {
		impl buffalo::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut buffalo::Writer) -> buffalo::Position<Self::Output> {
				match self {
					#(#write_match_arms)*
				}
			}
		}
	};
	let write_type_impl = quote! {
		impl buffalo::WriteType for #ident {
			type WriteType = buffalo::Position<#writer_ident>;
		}
	};
	let code = quote! {
		#writer_decl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}

fn dynamic_struct_write(
	input: &syn::DeriveInput,
	attributes: &StructAttributes,
	data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
	let visibility = &input.vis;
	let ident = &input.ident;
	let index_type = attributes
		.index_type
		.as_ref()
		.unwrap_or(&DEFAULT_STRUCT_INDEX_TYPE);
	let index_ty = match index_type {
		StructIndexType::U8 => quote!(u8),
		StructIndexType::U16 => quote!(u16),
	};
	let index_variant = match index_type {
		StructIndexType::U8 => quote!(U8),
		StructIndexType::U16 => quote!(U16),
	};
	let writer_ident = format_ident!("{}Writer", ident);
	let writer_fields = data
		.fields
		.iter()
		.map(|field| {
			let field_ident = &field.ident;
			let field_ty = &field.ty;
			quote!(#visibility #field_ident: <#field_ty as buffalo::WriteType>::WriteType)
		})
		.collect::<Vec<_>>();
	let writer_decl = quote! {
		#visibility struct #writer_ident {
			#(#writer_fields,)*
		}
	};
	let fields = data
		.fields
		.iter()
		.map(|field| {
			let field_attributes = field_attributes(&field)?;
			let field_id = field_attributes.id;
			Ok((field_id, field))
		})
		.collect::<syn::Result<BTreeMap<_, _>>>()?;
	let index_field_count = *fields.keys().max().unwrap() + 1;
	let index_statements = (0..index_field_count).map(|field_id| {
		if let Some(field) = fields.get(&field_id) {
			let field_ty = &field.ty;
			Some(quote! {
				index.set_field_offset(#field_id as #index_ty, offset);
				offset += <<#field_ty as buffalo::WriteType>::WriteType as buffalo::StaticSize>::STATIC_SIZE as #index_ty;
			})
		} else {
			None
		}
	});
	let writer_statements = fields.values().map(|field| {
		let field_ident = &field.ident;
		quote! {
			writer.write(&self.#field_ident);
		}
	});
	let write_impl = quote! {
		impl buffalo::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut buffalo::Writer) -> buffalo::Position<Self::Output> {
				let mut offset = <buffalo::PointerType as buffalo::StaticSize>::STATIC_SIZE as #index_ty;
				let mut index = buffalo::DynamicStructIndexWriterI::<#index_ty>::new(#index_field_count as #index_ty);
				#(#index_statements)*
				let index = buffalo::DynamicStructIndexWriter::#index_variant(index);
				let index_position = if let Some(index_position) = writer.get_index(&index) {
					*index_position
				} else {
					let index_position = writer.write(&index);
					writer.add_index(index, index_position);
					index_position
				};
				let position = writer.position();
				writer.write(&index_position);
				#(#writer_statements)*
				position
			}
		}
	};
	let write_type_impl = quote! {
		impl buffalo::WriteType for #ident {
			type WriteType = buffalo::Position<#writer_ident>;
		}
	};
	let code = quote! {
		#writer_decl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}

fn static_enum_write(
	input: &syn::DeriveInput,
	attributes: &EnumAttributes,
	data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
	let visibility = &input.vis;
	let ident = &input.ident;
	let discriminant_type = attributes
		.discriminant_type
		.as_ref()
		.unwrap_or(&DEFAULT_ENUM_DISCRIMINANT_TYPE);
	let discriminant_size: PointerType = match discriminant_type {
		EnumDiscriminantType::U8 => 1,
		EnumDiscriminantType::U16 => 2,
	};
	let value_size = &attributes.value_size.ok_or_else(|| {
		syn::Error::new(
			input.span(),
			"attribute \"value_size\" is required when \"size\" = \"static\"",
		)
	})?;
	let writer_ident = format_ident!("{}Writer", ident);
	let writer_variants = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			if matches!(variant.fields, syn::Fields::Unit) {
				Ok(quote!(#variant_ident))
			} else {
				let variant_ty = variant_ty(&variant)?;
				Ok(quote!(#variant_ident(<#variant_ty as buffalo::WriteType>::WriteType)))
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let writer_decl = quote! {
		#visibility enum #writer_ident {
			#(#writer_variants,)*
		}
	};
	let static_size = discriminant_size + value_size;
	let static_size_impl = quote! {
		impl buffalo::StaticSize for #writer_ident {
			const STATIC_SIZE: buffalo::PointerType = #static_size;
		}
	};
	let write_match_arms = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_attributes = variant_attributes(&variant)?;
			let variant_ty = variant_ty(variant)?;
			let variant_id = discriminant_type.value(variant_attributes.id);
			if matches!(variant.fields, syn::Fields::Unit) {
				let code = quote! {
					Self::#variant_ident => {
						let position = writer.position();
						writer.write(&#variant_id);
						writer.write_raw::<()>(&vec![0u8; #value_size as usize]);
						position
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					Self::#variant_ident(value) => {
						let position = writer.position();
						writer.write(&#variant_id);
						writer.write(value);
						const variant_size: buffalo::PointerType = <<#variant_ty as buffalo::WriteType>::WriteType as buffalo::StaticSize>::STATIC_SIZE;
						const extra_bytes_count: buffalo::PointerType = #value_size - variant_size;
						writer.write_raw::<()>(&vec![0u8; extra_bytes_count as usize]);
						position
					}
				};
				Ok(code)
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let write_impl = quote! {
		impl buffalo::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut buffalo::Writer) -> buffalo::Position<Self::Output> {
				match self {
					#(#write_match_arms)*
				}
			}
		}
	};
	let write_type_impl = quote! {
		impl buffalo::WriteType for #ident {
			type WriteType = #writer_ident;
		}
	};
	let code = quote! {
		#writer_decl
		#static_size_impl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}
