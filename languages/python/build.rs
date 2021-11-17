fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
	match target_os.as_str() {
		"windows" => {
			println!("cargo:rustc-link-search=native={}", std::env::var("CARGO_MANIFEST_DIR").unwrap());
		}
		_ => {}
	};
}
