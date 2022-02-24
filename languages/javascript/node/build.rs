use std::path::PathBuf;

const NODE_VERSION: &str = "v16.4.0";

fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
	if target_os == "macos" {
		println!("cargo:rustc-cdylib-link-arg=-undefined");
		println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
	} else if target_os == "windows" {
		let path = match std::env::var("X64_64_WINDOWS_NODE_API_LINK_SEARCH_PATH") {
			Ok(path) => PathBuf::from(path),
			Err(std::env::VarError::NotPresent) => {
				let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
				let arch = match target_arch.as_str() {
					"x86_64" => "x64",
					_ => panic!("unsupported architecture"),
				};
				let url = format!("https://nodejs.org/dist/{NODE_VERSION}/win-{arch}/node.lib");
				let node_lib_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
				let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
				let node_lib_path = out_dir.join("node.lib");
				std::fs::write(node_lib_path, node_lib_bytes).unwrap();
				out_dir
			}
			Err(e) => {
				panic!("{}", e);
			}
		};
		println!("cargo:rustc-link-search={}", path.to_str().unwrap());
		println!("cargo:rustc-link-lib=node");
	}
}
