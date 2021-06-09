use quote::quote;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn include_out_dir(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let path: Option<syn::LitStr> = syn::parse2(input).ok();
	let mut out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	if let Some(path) = path {
		out_dir = out_dir.join(path.value());
	}
	let absolute_paths: Vec<PathBuf> = WalkDir::new(&out_dir)
		.into_iter()
		.filter_map(|entry| {
			let entry = entry.unwrap();
			let path = entry.path().to_owned();
			let metadata = std::fs::metadata(&path).unwrap();
			if metadata.is_file() {
				Some(path)
			} else {
				None
			}
		})
		.collect();
	let relative_paths: Vec<PathBuf> = absolute_paths
		.iter()
		.map(|absolute_path| absolute_path.strip_prefix(&out_dir).unwrap().to_owned())
		.collect();
	let absolute_paths: Vec<String> = absolute_paths
		.into_iter()
		.map(|path| path.to_str().unwrap().to_owned())
		.collect();
	let relative_paths: Vec<String> = relative_paths
		.into_iter()
		.map(|path| path.to_str().unwrap().to_owned())
		.collect();
	let code = quote! {{
		let mut map = ::std::collections::HashMap::new();
		#({
			let path = ::std::path::Path::new(#relative_paths);
			let data = include_bytes!(#absolute_paths);
			let hash = include_out_dir::hash(data);
			let file = include_out_dir::File {
				data: data.as_ref(),
				hash,
			};
			map.insert(path, file);
		})*
		include_out_dir::IncludeOutDir::new(map)
	}};
	Ok(code)
}
