//! This example demonstrates how to use `build_wheel` via a small command-line tool.
use build_wheel::WheelBuilder;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
	/// The directory to run the build and output the wheel.  MUST BE EMPTY/NONEXISTENT
	#[clap(short, long)]
	build_path: Option<PathBuf>,
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
	tag: Option<String>,
}

fn main() {
	let args = Args::parse();
	let mut builder = WheelBuilder::new(args.lib_path, args.metadata_path);
	if let Some(p) = args.interface_path {
		builder = builder.type_interface(p);
	}
	if let Some(p) = args.build_path {
		builder = builder.output_dir(p);
	}
	if let Some(t) = args.tag {
		builder = builder.tag(&t).expect("Could not parse tag");
	}
	if let Err(e) = builder.build() {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
