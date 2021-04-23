use std::path::PathBuf;
use tangram_error::Result;

fn main() -> Result<()> {
	let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let workspace_dir = crate_dir.parent().unwrap().to_owned();
	let crate_out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let cargo_target_wasm_dir = workspace_dir.join("target_wasm");
	let css_dirs = vec![
		workspace_dir.join("charts"),
		workspace_dir.join("ui"),
		workspace_dir.join("www"),
	];
	println!("cargo:rerun-if-changed=../charts");
	println!("cargo:rerun-if-changed=../ui");
	println!("cargo:rerun-if-changed=../www");
	tangram_serve::build(tangram_serve::BuildOptions {
		workspace_dir,
		crate_dir,
		crate_out_dir,
		cargo_target_wasm_dir,
		css_dirs,
	})?;
	Ok(())
}
