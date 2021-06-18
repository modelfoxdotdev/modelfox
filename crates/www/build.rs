use std::path::PathBuf;
use tangram_error::Result;

fn main() -> Result<()> {
	let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let workspace_path = crate_path.parent().unwrap().parent().unwrap().to_owned();
	let crate_out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let css_paths = vec![workspace_path.to_owned()];
	println!("cargo:rerun-if-changed=../../Cargo.lock");
	println!("cargo:rerun-if-changed=../");
	sunfish::build(sunfish::BuildOptions {
		workspace_path,
		crate_path,
		crate_out_dir,
		css_paths,
	})?;
	Ok(())
}
