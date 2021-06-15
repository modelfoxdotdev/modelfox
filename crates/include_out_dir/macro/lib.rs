mod include_out_dir;

#[proc_macro]
pub fn include_out_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	include_out_dir::include_out_dir(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}
