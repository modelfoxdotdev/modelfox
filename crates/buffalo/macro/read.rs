use crate::{
	enum_attributes::{
		enum_attributes, EnumAttributes, EnumDiscriminantType, EnumSize, PointerType,
		DEFAULT_ENUM_DISCRIMINANT_TYPE,
	},
	field_attributes::{field_attributes, FieldIdValue},
	struct_attributes::{
		struct_attributes, StructAttributes, StructIndexType, StructSize, DEFAULT_STRUCT_INDEX_TYPE,
	},
	variant_attributes::{variant_attributes, variant_ty},
};
use heck::*;
use quote::{format_ident, quote};
use std::convert::TryInto;
use syn::spanned::Spanned;

pub fn read(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
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
		#visibility struct #reader_ident<'a>(buffalo::DynamicStructReader<'a, #index_type>);
	};
	let read_impl = quote! {
		impl<'a> buffalo::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: buffalo::Position<Self>) -> Self::Output {
				Self(buffalo::DynamicStructReader::read(bytes, position.cast()))
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> buffalo::ReadType<'a> for #ident {
			type ReadType = buffalo::Pointer<#reader_ident<'a>>;
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
					#visibility fn #field_ident(self) -> <<#field_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output {
						let field_id = buffalo::DynamicStructIndexFieldId(#field_id);
						self.0.get_field_value::<<#field_ty as buffalo::ReadType<'a>>::ReadType>(field_id).unwrap()
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					#visibility fn #field_ident(self) -> Option<<<#field_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output> {
						let field_id = buffalo::DynamicStructIndexFieldId(#field_id);
						self.0.get_field_value::<<#field_ty as buffalo::ReadType<'a>>::ReadType>(field_id)
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
		#visibility struct #reader_ident<'a>(buffalo::StaticStructReader<'a>);
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
		impl<'a> buffalo::StaticSize for #reader_ident<'a> {
			const STATIC_SIZE: buffalo::PointerType = #(#size_exprs)+*;
		}
	};
	let read_impl = quote! {
		impl<'a> buffalo::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: buffalo::Position<Self>) -> Self::Output {
				Self(buffalo::StaticStructReader::read(bytes, position.cast()))
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> buffalo::ReadType<'a> for #ident {
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
					#visibility fn #field_ident(self) -> <<#field_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output {
						let mut field_offset = 0;
						#(field_offset += #previous_size_exprs;)*;
						self.0.get_field_value::<<#field_ty as buffalo::ReadType<'a>>::ReadType>(field_offset).unwrap()
					}
				};
				Ok(code)
			} else {
				let code = quote! {
					#visibility fn #field_ident(self) -> Option<<<#field_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output> {
						let mut field_offset = 0;
						#(field_offset += #previous_size_exprs;)*;
						self.0.get_field_value::<<#field_ty as buffalo::ReadType<'a>>::ReadType>(field_offset)
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
				#variant_ident(buffalo::VariantReader<'a, <#variant_ty as buffalo::ReadType<'a>>::ReadType>),
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
					Self::#variant_ident(buffalo::VariantReader::new(
						bytes,
						position.offset(#discriminant_size),
					))
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let read_impl = quote! {
		impl<'a> buffalo::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: buffalo::Position<Self>) -> Self::Output {
				let variant_id = #discriminant_type::read(bytes, position.cast());
				match variant_id {
					#(#read_match_arms)*
					_ => panic!("unknown variant"),
				}
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> buffalo::ReadType<'a> for #ident {
			type ReadType = buffalo::Pointer<#reader_ident<'a>>;
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
				#visibility fn #accessor_fn_name(self) -> Option<<<#variant_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output> {
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
				#variant_ident(buffalo::VariantReader<'a, <#variant_ty as buffalo::ReadType<'a>>::ReadType>),
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
		impl<'a> buffalo::StaticSize for #reader_ident<'a> {
			const STATIC_SIZE: buffalo::PointerType = #static_size;
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
					Self::#variant_ident(buffalo::VariantReader::new(
						bytes,
						position.offset(#discriminant_size),
					))
				}
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let read_impl = quote! {
		impl<'a> buffalo::Read<'a> for #reader_ident<'a> {
			type Output = Self;
			fn read(bytes: &'a [u8], position: buffalo::Position<Self>) -> Self::Output {
				let variant_id = #discriminant_type::read(bytes, position.cast());
				match variant_id {
					#(#read_match_arms)*
					_ => panic!("unknown variant"),
				}
			}
		}
	};
	let read_type_impl = quote! {
		impl<'a> buffalo::ReadType<'a> for #ident {
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
				#visibility fn #accessor_fn_name(self) -> Option<<<#variant_ty as buffalo::ReadType<'a>>::ReadType as buffalo::Read<'a>>::Output> {
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
