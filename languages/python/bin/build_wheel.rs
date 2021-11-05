use anyhow::{Context, Result};
use build_wheel::{build_wheel, Paths};
use clap::Parser;
use const_format::concatcp;
use std::{io, path::PathBuf, str::FromStr};

const HELP: &str = "usage: build_wheel [help|release|testing]";
const MIN_PY: &str = "cp36";
const ABI: &str = "abi3";
const MANYLINUX: &str = "manylinux_2_24";
const ARCH: &str = "x86_64";
const TAG: &str = concatcp!(MIN_PY, "-", ABI, "-", MANYLINUX, "_", ARCH);
const PNAME: &str = "tangram";
const VERSION: &str = "0.7.0";
const NAME: &str = concatcp!(PNAME, "-", VERSION);
const WHEEL_NAME: &str = concatcp!(NAME, "-", TAG, ".whl");

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

fn main() -> Result<()> {
	let args = Args::parse();
	let paths = Paths::new(args.lib_path, args.manifest_path, args.build_path);
	build_wheel(paths)?;
	Ok(())
}
