fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
	if target_os == "windows" {
		windows_link_node_import_library();
	}
}

fn windows_link_node_import_library() {
	let version = "v15.9.0";
	let arch = "x64";
	let url = format!(
		"https://nodejs.org/dist/{version}/win-{arch}/node.lib",
		version = version,
		arch = arch
	);
	let node_lib_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
	let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let node_lib_path = out_dir.join("node.lib");
	std::fs::write(node_lib_path, node_lib_bytes).unwrap();
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rustc-link-search={}", out_dir.to_str().unwrap());
	println!("cargo:rustc-link-lib=node");
}
