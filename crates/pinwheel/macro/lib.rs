mod component_builder;

#[proc_macro_derive(ComponentBuilder, attributes(optional, children))]
pub fn component_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	component_builder::component_builder(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}
