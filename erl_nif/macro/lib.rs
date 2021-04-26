use quote::quote;
use syn::spanned::Spanned;
use tangram_zip::zip;

#[proc_macro]
pub fn init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	init_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

struct InitArgs {
	name: syn::LitStr,
	funcs: syn::ExprArray,
	load: syn::Ident,
}

impl syn::parse::Parse for InitArgs {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut name = None;
		let mut funcs = None;
		let mut load = None;
		while !input.is_empty() {
			let key: syn::Ident = input.parse()?;
			input.parse::<syn::Token![:]>()?;
			match key.to_string().as_str() {
				"name" => name = Some(input.parse()?),
				"funcs" => funcs = Some(input.parse()?),
				"load" => load = Some(input.parse()?),
				_ => return Err(syn::Error::new(key.span(), "unknown key")),
			}
			input.parse::<syn::Token![,]>()?;
		}
		let name = name.ok_or_else(|| syn::Error::new(input.span(), "\"name\" is required"))?;
		let funcs = funcs.ok_or_else(|| syn::Error::new(input.span(), "\"funcs\" is required"))?;
		let load = load.ok_or_else(|| syn::Error::new(input.span(), "\"load\" is required"))?;
		Ok(InitArgs { name, funcs, load })
	}
}

fn init_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: InitArgs = syn::parse2(input)?;
	let name = &input.name;
	let funcs = &input.funcs;
	let num_of_funcs = funcs.elems.len();
	let load = &input.load;
	let load = quote! {{
		unsafe extern "C" fn _load(env: *mut erl_nif::sys::ErlNifEnv, priv_data: *mut *mut std::os::raw::c_void, load_info: erl_nif::sys::ERL_NIF_TERM) -> std::os::raw::c_int {
			let env = erl_nif::Env::from_raw(env);
			let load_info = erl_nif::Term::from_raw(env, load_info);
			let result = std::panic::catch_unwind(|| {
				#load(env, load_info)
			});
			let result = match result {
				Ok(result) => result,
				Err(_) => {
					env.raise_exception("A panic occurred.");
					return -1;
				}
			};
			match result {
				Ok(_) => {},
				Err(error) => {
					env.raise_exception(&error.to_string());
					return -1;
				},
			};
			0
		}
		Some(_load)
	}};
	let entry = quote! {
		static ENTRY: erl_nif::Entry = erl_nif::Entry::new(erl_nif::sys::ErlNifEntry {
			major: erl_nif::sys::ERL_NIF_MAJOR_VERSION,
			minor: erl_nif::sys::ERL_NIF_MINOR_VERSION,
			name: concat!(#name, "\0").as_ptr() as *const std::os::raw::c_char,
			num_of_funcs: #num_of_funcs as std::os::raw::c_int,
			funcs: #funcs.as_ptr(),
			load: #load,
			reload: None,
			upgrade: None,
			unload: None,
			vm_variant: erl_nif::sys::ERL_NIF_VM_VARIANT.as_ptr() as *const std::os::raw::c_char,
			options: 0,
			sizeof_ErlNifResourceTypeInit: std::mem::size_of::<erl_nif::sys::ErlNifResourceTypeInit>(),
			min_erts: erl_nif::sys::ERL_NIF_MIN_ERTS_VERSION.as_ptr() as *const std::os::raw::c_char,
		});
		ENTRY.raw()
	};
	let code = quote! {
		#[cfg(unix)]
		#[no_mangle]
		pub unsafe extern "C" fn nif_init() -> *const erl_nif::sys::ErlNifEntry {
			#entry
		}
		#[cfg(windows)]
		#[no_mangle]
		pub unsafe extern "C" fn nif_init(callbacks: *const erl_nif::sys::TWinDynNifCallbacks) -> *const erl_nif::sys::ErlNifEntry {
			unsafe { erl_nif::sys::WinDynNifCallbacks = Some(callbacks) };
			#entry
		}
	};
	Ok(code)
}

#[proc_macro_attribute]
pub fn nif(
	_attrs: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	nif_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

fn nif_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::ItemFn = syn::parse2(input)?;
	let name = input.sig.ident.to_string();
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
		.map(|arg| {
			let arg = match arg {
				syn::FnArg::Typed(arg) => arg,
				syn::FnArg::Receiver(_) => {
					return Err(syn::Error::new(
						arg.span(),
						"receiver arg is not allowed here",
					))
				}
			};
			let ident = match &*arg.pat {
				syn::Pat::Ident(pat_ident) => &pat_ident.ident,
				_ => return Err(syn::Error::new(arg.pat.span(), "invalid pattern")),
			};
			Ok(ident)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let args_count = args.len();
	let arity = args_count as u32;
	let from_erl_nif_statements = args
		.iter()
		.enumerate()
		.map(|(i, ident)| {
			let code = quote! {
				let #ident = argv[#i];
				let #ident = erl_nif::Term::from_raw(env, #ident);
				let #ident = erl_nif::FromErlNif::from_erl_nif(#ident)?;
			};
			Ok(code)
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let fptr = quote! {{
		unsafe extern "C" fn nif(
			env: *mut erl_nif::sys::ErlNifEnv,
			argc: std::os::raw::c_int,
			argv: *const erl_nif::sys::ERL_NIF_TERM
		) -> erl_nif::sys::ERL_NIF_TERM {
			fn nif_impl<'a>(env: erl_nif::Env<'a>, #(#impl_inputs),*) #impl_output #impl_block
			let env = erl_nif::Env::from_raw(env);
			let result = std::panic::catch_unwind(|| -> erl_nif::Result<_> {
				let argv = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
				#(#from_erl_nif_statements)*
				let output = nif_impl(env, #(#args),*).map_err(|error| erl_nif::Error::message(error.to_string()))?;
				let output = erl_nif::ToErlNif::to_erl_nif(output, env)?;
				Ok(output)
			});
			let result = match result {
				Ok(result) => result,
				Err(_) => {
					return env.raise_exception("A panic occurred.");
				},
			};
			let output = match result {
				Ok(output) => output,
				Err(error) => {
					return env.raise_exception(&error.to_string());
				}
			};
			output.raw()
		}
		Some(nif)
	}};
	let code = quote! {
		#[allow(non_upper_case_globals)]
		#visibility const #ident: erl_nif::sys::ErlNifFunc = erl_nif::sys::ErlNifFunc {
			name: concat!(#name, "\0").as_ptr() as *const std::os::raw::c_char,
			arity: #arity,
			flags: 0,
			fptr: #fptr,
		};
	};
	Ok(code)
}

#[proc_macro]
pub fn api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	api_impl(input.into())
		.unwrap_or_else(|e| e.to_compile_error())
		.into()
}

struct ApiInput {
	functions: Vec<syn::ForeignItemFn>,
}

impl syn::parse::Parse for ApiInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut functions = Vec::new();
		while !input.is_empty() {
			functions.push(input.parse()?);
		}
		Ok(ApiInput { functions })
	}
}

fn api_impl(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: ApiInput = syn::parse2(input)?;
	let input = &input.functions;
	let attrs = input
		.iter()
		.map(|function| &function.attrs)
		.collect::<Vec<_>>();
	let idents = input
		.iter()
		.map(|function| &function.sig.ident)
		.collect::<Vec<_>>();
	let inputs = input
		.iter()
		.map(|function| &function.sig.inputs)
		.collect::<Vec<_>>();
	let outputs = input
		.iter()
		.map(|function| &function.sig.output)
		.collect::<Vec<_>>();
	let args = input
		.iter()
		.map(|input| {
			input
				.sig
				.inputs
				.iter()
				.map(|input| {
					let arg = match input {
						syn::FnArg::Typed(arg) => arg,
						syn::FnArg::Receiver(_) => {
							return Err(syn::Error::new(
								input.span(),
								"receiver arg is not allowed here",
							))
						}
					};
					let ident = match &*arg.pat {
						syn::Pat::Ident(pat_ident) => &pat_ident.ident,
						_ => return Err(syn::Error::new(arg.pat.span(), "invalid pattern")),
					};
					Ok(ident)
				})
				.collect::<syn::Result<Vec<_>>>()
		})
		.collect::<syn::Result<Vec<_>>>()?;
	let functions_not_windows = zip!(attrs.iter(), idents.iter(), inputs.iter(), outputs.iter(),)
		.map(|(attrs, ident, inputs, output)| {
			let attrs = quote! { #(#attrs)* };
			quote! {
				#attrs pub fn #ident(#inputs) #output;
			}
		});
	let functions_windows = zip!(
		attrs.iter(),
		idents.iter(),
		inputs.iter(),
		outputs.iter(),
		args.iter()
	)
	.map(|(attrs, ident, inputs, output, args)| {
		let attrs = quote! { #(#attrs)* };
		let args = quote! { #(#args,)* };
		quote! {
			#[cfg(windows)]
			#attrs
			pub unsafe fn #ident(#inputs) #output {
				((*WinDynNifCallbacks.unwrap()).#ident)(#args)
			}
		}
	});
	let code = quote! {
		#[cfg(unix)]
		extern "C" {
			#(#functions_not_windows)*
		}
		#[cfg(windows)]
		pub static mut WinDynNifCallbacks: Option<*const TWinDynNifCallbacks> = None;
		#[cfg(windows)]
		#[repr(C)]
		pub struct TWinDynNifCallbacks {
			#(#idents: extern "C" fn (#inputs) #outputs,)*
		}
		#(#functions_windows)*
	};
	Ok(code)
}
