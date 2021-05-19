mod element;

#[proc_macro]
pub fn element(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	element::element(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}
