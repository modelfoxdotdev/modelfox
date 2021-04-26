use heck::*;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, convert::TryInto};
use syn::spanned::Spanned;

type PointerType = u64;

const DEFAULT_ENUM_DISCRIMINANT_TYPE: EnumDiscriminantType = EnumDiscriminantType::U8;
const DEFAULT_STRUCT_INDEX_TYPE: StructIndexType = StructIndexType::U16;

#[proc_macro_derive(Read, attributes(tangram_serialize))]
pub fn read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	read_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

#[proc_macro_derive(Write, attributes(tangram_serialize))]
pub fn write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	write_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn read_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
	let data = &input.data;
	match data {
		syn::Data::Struct(data) => struct_read(&input, data),
		syn::Data::Enum(data) => enum_read(&input, data),
		_ => Err(syn::Error::new_spanned(
			input,
			"this macro can only be used on a struct or enum",
		)),
	}
}

fn write_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
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

fn struct_read(
	input: &syn::DeriveInput,
	data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
	let attributes = struct_attributes(&input.attrs, input.span())?;
	match attributes.size {
		StructSize::Dynamic => dynamic_struct_read(input, &attributes, data),
		StructSize::Static => static_struct_read(input, &attributes, data),
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

fn enum_read(
	input: &syn::DeriveInput,
	data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
	let attributes = enum_attributes(&input)?;
	match attributes.size {
		EnumSize::Dynamic => dynamic_enum_read(input, &attributes, data),
		EnumSize::Static => static_enum_read(input, &attributes, data),
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

fn dynamic_struct_read(
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
	let reader_ident = format_ident!("{}Reader", ident);
	let reader_decl = quote! {
		#[derive(Clone, Copy, Debug)]
		#visibility struct #reader_ident<'a>(tangram_serialize::DynamicStructReader<'a, #index_type>);
	};
	let read_impl = quote! {
		impl<'a> tangram_serialize::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: tangram_serialize::Position<Self>) -> Self::Output {
				Self(tangram_serialize::DynamicStructReader::read(bytes, position.cast()))
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> tangram_serialize::ReadType<'a> for #ident {
			type ReadType = tangram_serialize::Pointer<#reader_ident<'a>>;
		}
	};
	let accessors = data
		.fields
		.iter()
		.map(|field| {
			let field_ident = field.ident.as_ref().unwrap();
			let field_ty = &field.ty;
			let field_attributes = field_attributes(&field)?;
			let field_id = match index_type {
			    StructIndexType::U8 => FieldIdValue::U8(field_attributes.id.try_into().unwrap()),
			    StructIndexType::U16 => FieldIdValue::U16(field_attributes.id.try_into().unwrap()),
			};
			if field_attributes.required {
				let code = quote! {
					#visibility fn #field_ident(self) -> <<#field_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output {
						let field_id = tangram_serialize::DynamicStructIndexFieldId(#field_id);
						self.0.get_field_value::<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType>(field_id).unwrap()
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					#visibility fn #field_ident(self) -> Option<<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output> {
						let field_id = tangram_serialize::DynamicStructIndexFieldId(#field_id);
						self.0.get_field_value::<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType>(field_id)
					}
				};
				Ok(code)
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let accessors_impl = quote! {
		impl<'a> #reader_ident<'a> {
			#(#accessors)*
		}
	};
	let code = quote! {
		#reader_decl
		#read_impl
		#read_type_impl
		#accessors_impl
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
			quote!(#visibility #field_ident: <#field_ty as tangram_serialize::WriteType>::WriteType)
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
				offset += <<#field_ty as tangram_serialize::WriteType>::WriteType as tangram_serialize::StaticSize>::STATIC_SIZE as #index_ty;
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
		impl tangram_serialize::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut tangram_serialize::Writer) -> tangram_serialize::Position<Self::Output> {
				let mut offset = <tangram_serialize::PointerType as tangram_serialize::StaticSize>::STATIC_SIZE as #index_ty;
				let mut index = tangram_serialize::DynamicStructIndexWriterI::<#index_ty>::new(#index_field_count as #index_ty);
				#(#index_statements)*
				let index = tangram_serialize::DynamicStructIndexWriter::#index_variant(index);
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
		impl tangram_serialize::WriteType for #ident {
			type WriteType = tangram_serialize::Position<#writer_ident>;
		}
	};
	let code = quote! {
		#writer_decl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}

fn static_struct_read(
	input: &syn::DeriveInput,
	_attributes: &StructAttributes,
	data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
	let visibility = &input.vis;
	let ident = &input.ident;
	let reader_ident = format_ident!("{}Reader", ident);
	let reader_decl = quote! {
		#[derive(Clone, Copy, Debug)]
		#visibility struct #reader_ident<'a>(tangram_serialize::StaticStructReader<'a>);
	};
	let size_exprs = data
		.fields
		.iter()
		.map(|field| {
			let field_ty = &field.ty;
			quote! { <<#field_ty as tangram_serialize::ReadType>::ReadType as tangram_serialize::StaticSize>::STATIC_SIZE }
		})
		.collect::<Vec<_>>();
	let static_size_impl = quote! {
		impl<'a> tangram_serialize::StaticSize for #reader_ident<'a> {
			const STATIC_SIZE: tangram_serialize::PointerType = #(#size_exprs)+*;
		}
	};
	let read_impl = quote! {
		impl<'a> tangram_serialize::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: tangram_serialize::Position<Self>) -> Self::Output {
				Self(tangram_serialize::StaticStructReader::read(bytes, position.cast()))
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> tangram_serialize::ReadType<'a> for #ident {
			type ReadType = #reader_ident<'a>;
		}
	};
	let accessors = data
		.fields
		.iter()
		.enumerate()
		.map(|(field_index, field)| {
			let field_ident = field.ident.as_ref().unwrap();
			let field_ty = &field.ty;
			let field_attributes = field_attributes(&field)?;
			let previous_size_exprs = size_exprs.iter().take(field_index);
			if field_attributes.required {
				let code = quote! {
					#visibility fn #field_ident(self) -> <<#field_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output {
						let mut field_offset = 0;
						#(field_offset += #previous_size_exprs;)*;
						self.0.get_field_value::<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType>(field_offset).unwrap()
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					#visibility fn #field_ident(self) -> Option<<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output> {
						let mut field_offset = 0;
						#(field_offset += #previous_size_exprs;)*;
						self.0.get_field_value::<<#field_ty as tangram_serialize::ReadType<'a>>::ReadType>(field_offset)
					}
				};
				Ok(code)
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let accessors_impl = quote! {
		impl<'a> #reader_ident<'a> {
			#(#accessors)*
		}
	};
	let code = quote! {
		#reader_decl
		#static_size_impl
		#read_impl
		#read_type_impl
		#accessors_impl
	};
	Ok(code)
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
			quote!(#visibility #field_ident: <#field_ty as tangram_serialize::WriteType>::WriteType)
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
			quote! { <<#field_ty as tangram_serialize::ReadType>::ReadType as tangram_serialize::StaticSize>::STATIC_SIZE }
		})
		.collect::<Vec<_>>();
	let static_size_impl = quote! {
		impl tangram_serialize::StaticSize for #writer_ident {
			const STATIC_SIZE: tangram_serialize::PointerType = #(#size_exprs)+*;
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
		impl tangram_serialize::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut tangram_serialize::Writer) -> tangram_serialize::Position<Self::Output> {
				let position = writer.position();
				#(#writer_statements)*
				position
			}
		}
	};
	let write_type_impl = quote! {
		impl tangram_serialize::WriteType for #ident {
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

fn dynamic_enum_read(
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
	let reader_ident = format_ident!("{}Reader", ident);
	let reader_variants = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_ty = variant_ty(&variant)?;
			let code = quote! {
				#variant_ident(tangram_serialize::VariantReader<'a, <#variant_ty as tangram_serialize::ReadType<'a>>::ReadType>),
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let reader_decl = quote! {
		#[derive(Clone, Copy, Debug)]
		#visibility enum #reader_ident<'a> {
			#(#reader_variants)*
		}
	};
	let read_match_arms = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_attributes = variant_attributes(&variant)?;
			let variant_id = discriminant_type.value(variant_attributes.id);
			let code = quote! {
				#variant_id => {
					Self::#variant_ident(tangram_serialize::VariantReader::new(
						bytes,
						position.offset(#discriminant_size),
					))
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let read_impl = quote! {
		impl<'a> tangram_serialize::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: tangram_serialize::Position<Self>) -> Self::Output {
				let variant_id = #discriminant_type::read(bytes, position.cast());
				match variant_id {
					#(#read_match_arms)*
					_ => panic!("unknown variant"),
				}
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> tangram_serialize::ReadType<'a> for #ident {
			type ReadType = tangram_serialize::Pointer<#reader_ident<'a>>;
		}
	};
	let accessors = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let accessor_fn_name =
				format_ident!("as_{}", variant_ident.to_string().to_snake_case());
			let variant_ty = variant_ty(&variant)?;
			let code = quote! {
				#visibility fn #accessor_fn_name(self) -> Option<<<#variant_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output> {
					match self {
						Self::#variant_ident(s) => Some(s.read()),
						_ => None,
					}
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let accessors_impl = quote! {
		impl<'a> #reader_ident<'a> {
			#(#accessors)*
		}
	};
	let code = quote! {
		#reader_decl
		#read_impl
		#read_type_impl
		#accessors_impl
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
				Ok(quote!(#variant_ident(<#variant_ty as tangram_serialize::WriteType>::WriteType)))
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
		impl tangram_serialize::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut tangram_serialize::Writer) -> tangram_serialize::Position<Self::Output> {
				match self {
					#(#write_match_arms)*
				}
			}
		}
	};
	let write_type_impl = quote! {
		impl tangram_serialize::WriteType for #ident {
			type WriteType = tangram_serialize::Position<#writer_ident>;
		}
	};
	let code = quote! {
		#writer_decl
		#write_impl
		#write_type_impl
	};
	Ok(code)
}

fn static_enum_read(
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
	let reader_ident = format_ident!("{}Reader", ident);
	let reader_variants = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_ty = variant_ty(&variant)?;
			let code = quote! {
				#variant_ident(tangram_serialize::VariantReader<'a, <#variant_ty as tangram_serialize::ReadType<'a>>::ReadType>),
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let reader_decl = quote! {
		#[derive(Clone, Copy, Debug)]
		#visibility enum #reader_ident<'a> {
			#(#reader_variants)*
		}
	};
	let static_size = discriminant_size + value_size;
	let static_size_impl = quote! {
		impl<'a> tangram_serialize::StaticSize for #reader_ident<'a> {
			const STATIC_SIZE: tangram_serialize::PointerType = #static_size;
		}
	};
	let read_match_arms = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let variant_attributes = variant_attributes(&variant)?;
			let variant_id = discriminant_type.value(variant_attributes.id);
			let code = quote! {
				#variant_id => {
					Self::#variant_ident(tangram_serialize::VariantReader::new(
						bytes,
						position.offset(#discriminant_size),
					))
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let read_impl = quote! {
		impl<'a> tangram_serialize::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: tangram_serialize::Position<Self>) -> Self::Output {
				let variant_id = #discriminant_type::read(bytes, position.cast());
				match variant_id {
					#(#read_match_arms)*
					_ => panic!("unknown variant"),
				}
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> tangram_serialize::ReadType<'a> for #ident {
			type ReadType = #reader_ident<'a>;
		}
	};
	let accessors = data
		.variants
		.iter()
		.map(|variant| {
			let variant_ident = &variant.ident;
			let accessor_fn_name =
				format_ident!("as_{}", variant_ident.to_string().to_snake_case());
			let variant_ty = variant_ty(&variant)?;
			let code = quote! {
				#visibility fn #accessor_fn_name(self) -> Option<<<#variant_ty as tangram_serialize::ReadType<'a>>::ReadType as tangram_serialize::Read<'a>>::Output> {
					match self {
						Self::#variant_ident(s) => Some(s.read()),
						_ => None,
					}
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let accessors_impl = quote! {
		impl<'a> #reader_ident<'a> {
			#(#accessors)*
		}
	};
	let code = quote! {
		#reader_decl
		#static_size_impl
		#read_impl
		#read_type_impl
		#accessors_impl
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
				Ok(quote!(#variant_ident(<#variant_ty as tangram_serialize::WriteType>::WriteType)))
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
		impl tangram_serialize::StaticSize for #writer_ident {
			const STATIC_SIZE: tangram_serialize::PointerType = #static_size;
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
						const variant_size: tangram_serialize::PointerType = <<#variant_ty as tangram_serialize::WriteType>::WriteType as tangram_serialize::StaticSize>::STATIC_SIZE;
						const extra_bytes_count: tangram_serialize::PointerType = #value_size - variant_size;
						writer.write_raw::<()>(&vec![0u8; extra_bytes_count as usize]);
						position
					}
				};
				Ok(code)
			}
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let write_impl = quote! {
		impl tangram_serialize::Write for #writer_ident {
			type Output = Self;
			fn write(&self, writer: &mut tangram_serialize::Writer) -> tangram_serialize::Position<Self::Output> {
				match self {
					#(#write_match_arms)*
				}
			}
		}
	};
	let write_type_impl = quote! {
		impl tangram_serialize::WriteType for #ident {
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

struct StructAttributes {
	size: StructSize,
	index_type: Option<StructIndexType>,
}

enum StructSize {
	Dynamic,
	Static,
}

enum StructIndexType {
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

fn struct_attributes(
	attrs: &[syn::Attribute],
	span: proc_macro2::Span,
) -> syn::Result<StructAttributes> {
	let attr = attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram_serialize"))
		.ok_or_else(|| syn::Error::new(span, "tangram_serialize attribute is required"))?;
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

struct EnumAttributes {
	size: EnumSize,
	discriminant_type: Option<EnumDiscriminantType>,
	value_size: Option<PointerType>,
}

enum EnumSize {
	Dynamic,
	Static,
}

enum EnumDiscriminantType {
	U8,
	U16,
}

impl EnumDiscriminantType {
	fn value(&self, value: u64) -> EnumDiscriminantValue {
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

enum EnumDiscriminantValue {
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

fn enum_attributes(input: &syn::DeriveInput) -> syn::Result<EnumAttributes> {
	let attr = input
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram_serialize"))
		.ok_or_else(|| syn::Error::new(input.span(), "tangram_serialize attribute is required"))?;
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"tangram_serialize attribute must contain a list",
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

struct FieldAttributes {
	id: u64,
	required: bool,
}

enum FieldIdValue {
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

fn field_attributes(field: &syn::Field) -> syn::Result<FieldAttributes> {
	let attr = field
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram_serialize"))
		.ok_or_else(|| syn::Error::new(field.span(), "tangram_serialize attribute is required"))?;
	let meta = attr.parse_meta()?;
	let list = match meta {
		syn::Meta::List(list) => list,
		_ => {
			return Err(syn::Error::new_spanned(
				attr,
				"tangram_serialize attribute must contain a list",
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

struct VariantAttributes {
	id: u64,
}

fn variant_attributes(variant: &syn::Variant) -> syn::Result<VariantAttributes> {
	let attr = variant
		.attrs
		.iter()
		.find(|attr| attr.path.is_ident("tangram_serialize"))
		.ok_or_else(|| {
			syn::Error::new(variant.span(), "tangram_serialize attribute is required")
		})?;
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

fn variant_ty(variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream> {
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
