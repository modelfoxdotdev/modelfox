use quote::quote;
use std::path::PathBuf;
use walkdir::WalkDir;

#[proc_macro]
pub fn include_out_dir(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let out_dir = std::env::var("OUT_DIR").unwrap();
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
	let ast = quote! {{
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
	ast.into()
}
