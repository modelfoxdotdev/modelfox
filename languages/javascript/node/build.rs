fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
	if target_os.as_str() == "macos" {
		println!("cargo:rustc-cdylib-link-arg=-undefined");
		println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
	}
}
