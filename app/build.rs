use std::path::PathBuf;
use tangram_error::Result;

fn main() -> Result<()> {
	let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let workspace_dir = crate_dir.parent().unwrap().to_owned();
	let crate_out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let cargo_target_wasm_dir = workspace_dir.join("target_wasm");
	let css_dirs = vec![
		workspace_dir.join("app"),
		workspace_dir.join("charts"),
		workspace_dir.join("ui"),
	];
	println!("cargo:rerun-if-changed=../app");
	println!("cargo:rerun-if-changed=../charts");
	println!("cargo:rerun-if-changed=../ui");
	tangram_serve::build::build(tangram_serve::build::BuildOptions {
		workspace_dir,
		crate_dir,
		crate_out_dir,
		cargo_target_wasm_dir,
		css_dirs,
	})?;
	Ok(())
}
