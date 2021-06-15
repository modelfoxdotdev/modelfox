mod enum_attributes;
mod field_attributes;
mod read;
mod struct_attributes;
mod variant_attributes;
mod write;

#[proc_macro_derive(Read, attributes(tangram_serialize))]
pub fn read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	self::read::read(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

#[proc_macro_derive(Write, attributes(tangram_serialize))]
pub fn write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	self::write::write(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}
