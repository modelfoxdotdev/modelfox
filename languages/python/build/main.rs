//! Build the python wheel from a prebuilt artifact.
use build_wheel::{Arch, WheelBuilder};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
	/// The architecture the library is compiled for
	#[clap(short, long)]
	architecture: Arch,
	/// The directory to run the build and output the wheel.
	#[clap(short, long)]
	build_dir: Option<PathBuf>,
	/// The .pyi interface file
	#[clap(short, long)]
	interface_path: Option<PathBuf>,
	/// The shared object library (.so, .dylib, .dll)
	#[clap(short, long)]
	lib_path: PathBuf,
	/// The location of the TOML file defining the metadata.
	#[clap(short, long)]
	metadata_path: PathBuf,
}

fn main() {
	let args = Args::parse();
	let mut builder = WheelBuilder::new(args.architecture, args.lib_path, args.metadata_path);
	if let Some(p) = args.interface_path {
		builder = builder.type_interface(p);
	}
	if let Some(d) = args.build_dir {
		builder = builder.output_dir(d);
	}
	if let Err(e) = builder.build() {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
