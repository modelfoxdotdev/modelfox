use build_wheel::{build_wheel, Paths};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
	/// The directory to run the build and output the wheel.  MUST BE EMPTY/NONEXISTENT
	#[clap(short, long)]
	build_path: PathBuf,
	/// The shared object library (.so, .dylib, .dll)
	#[clap(short, long)]
	lib_path: PathBuf,
	/// The location of the TOML file defining the metadata.
	#[clap(short, long)]
	manifest_path: PathBuf,
}

fn main() {
	let args = Args::parse();
	let paths = Paths::new(args.lib_path, args.manifest_path, args.build_path);
	if let Err(e) = build_wheel(paths) {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
