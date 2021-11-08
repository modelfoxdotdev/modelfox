use build_wheel::{build_wheel, Config, Tag};
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser)]
struct Args {
	/// The directory to run the build and output the wheel.  MUST BE EMPTY/NONEXISTENT
	#[clap(short, long)]
	build_path: PathBuf,
	/// The .pyi interface file
	#[clap(short, long)]
	interface_path: Option<PathBuf>,
	/// The shared object library (.so, .dylib, .dll)
	#[clap(short, long)]
	lib_path: PathBuf,
	/// The location of the TOML file defining the metadata.
	#[clap(short, long)]
	metadata_path: PathBuf,
    /// The tag to build for.  Default is "cp36-abi3-manylinux_2_24_x86_64".
    tag: Option<String>
}

fn main() {
	let args = Args::parse();
    let tag = args.tag.map(|s| Tag::from_str(&s).expect("Could not parse tag"));
	let paths = Config::new(
		args.interface_path,
		args.lib_path,
		args.metadata_path,
		args.build_path,
        tag
	);
	if let Err(e) = build_wheel(paths) {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
