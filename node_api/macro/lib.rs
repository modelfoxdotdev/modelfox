use quote::quote;
use syn::spanned::Spanned;

#[proc_macro]
pub fn init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	init_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn init_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::Ident = syn::parse2(input)?;
	let code = quote! {
		#[no_mangle]
		pub unsafe extern "C" fn napi_register_module_v1(env: node_api::sys::napi_env, exports: node_api::sys::napi_value) -> node_api::sys::napi_value {
			let env = node_api::Env::from_raw(env);
			let exports = node_api::Value::from_raw(env, exports);
			let result = std::panic::catch_unwind(|| #input(env, exports));
			let result = match result {
				Ok(result) => result,
				Err(panic_info) => {
					env.throw_error("A panic occurred.");
					return std::ptr::null_mut();
				},
			};
			let exports = match result {
				Ok(exports) => exports,
				Err(error) => {
					if !env.is_exception_pending() {
						env.throw_error(&format!("{}", error));
					}
					return std::ptr::null_mut();
				}
			};
			exports.raw()
		}
	};
	Ok(code)
}

#[proc_macro_attribute]
pub fn function(
	_attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	function_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn function_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::ItemFn = syn::parse2(input)?;
	let visibility = &input.vis;
	let ident = &input.sig.ident;
	let impl_inputs = input.sig.inputs.iter().skip(1);
	let impl_output = &input.sig.output;
	let impl_block = &input.block;
	let args = input
		.sig
		.inputs
		.iter()
		.skip(1)
		.map(|input| {
			let input = match input {
				syn::FnArg::Typed(arg) => arg,
				syn::FnArg::Receiver(_) => {
					return Err(syn::Error::new(
						input.span(),
						"receiver arg is not allowed here",
					))
				}
			};
			let ident = match &*input.pat {
				syn::Pat::Ident(pat_ident) => &pat_ident.ident,
				_ => return Err(syn::Error::new(input.pat.span(), "invalid pattern")),
			};
			Ok(ident)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let args_count = args.len();
	let from_node_api_statements = args
		.iter()
		.enumerate()
		.map(|(i, ident)| {
			let code = quote! {
				let #ident = argv[#i];
				let #ident = node_api::Value::from_raw(env, #ident);
				let #ident = node_api::FromNodeAPI::from_node_api(#ident)?;
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let code = quote! {
		#visibility unsafe extern "C" fn #ident(env: node_api::sys::napi_env, info: node_api::sys::napi_callback_info) -> node_api::sys::napi_value {
			fn function_impl<'a>(env: node_api::Env<'a>, #(#impl_inputs),*) #impl_output #impl_block
			let env = node_api::Env::from_raw(env);
			let result = std::panic::catch_unwind(|| -> node_api::Result<_> {
				let mut argc = #args_count;
				let mut argv: [node_api::sys::napi_value; #args_count] = [std::ptr::null_mut(); #args_count];
				let status = node_api::sys::napi_get_cb_info(
					env.raw(),
					info,
					&mut argc as *mut usize,
					argv.as_mut_ptr() as *mut node_api::sys::napi_value,
					std::ptr::null_mut(),
					std::ptr::null_mut()
				);
				if status != node_api::sys::napi_status::napi_ok {
					return Err(node_api::Error::from_last_node_api_error(env.raw(), status).into());
				}
				if argc != #args_count {
					return Err(node_api::Error::message("invalid number of arguments").into());
				}
				#(#from_node_api_statements)*
				let output = function_impl(env, #(#args),*).map_err(|error| node_api::Error::message(error.to_string()))?;
				let output = node_api::ToNodeAPI::to_node_api(output, env)?;
				Ok(output)
			});
			let result = match result {
				Ok(result) => result,
				Err(_) => {
					env.throw_error("A panic occurred.");
					return std::ptr::null_mut();
				},
			};
			let output = match result {
				Ok(output) => output,
				Err(error) => {
					if !env.is_exception_pending() {
						env.throw_error(&format!("{}", error));
					}
					return std::ptr::null_mut();
				}
			};
			output.raw()
		}
	};
	Ok(code)
}
