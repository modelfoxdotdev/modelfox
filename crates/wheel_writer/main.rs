//! This example demonstrates how to use `build_wheel` via a small command-line tool.
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use wheel_writer::WheelWriter;

/// CLI argument parser.  Accepts parameters to build Python wheels.
/// See <https://www.python.org/dev/peps/pep-0425/#platform-tag> for option specifications.
/// See <https://www.python.org/dev/peps/pep-0491/> for wheel format specification.
#[derive(Parser)]
struct Args {
	#[clap(long, help = "The supported ABI version")]
	abi: Option<String>,
	#[clap(long, help = "Optional build number. Must start with a digit.")]
	build_tag: Option<String>,
	#[clap(long, help = "The name of the distribution to package")]
	distribution: String,
	#[clap(long, help = "The location of the TOML file defining the metadata.")]
	metadata: PathBuf,
	#[clap(long, help = "The destination directory for the wheel file output.")]
	output: PathBuf,
	#[clap(
		long,
		alias = "package",
		help = "A directory containing the Python package to include in the wheel."
	)]
	packages: Vec<PathBuf>,
	#[clap(long, help = "The platform string this wheel supports.")]
	platform: Option<String>,
	#[clap(long, help = "The python environment and supported version tag")]
	python: String,
	#[clap(long, help = "The distribution version.")]
	version: String,
}

fn main() -> Result<()> {
	let args = Args::parse();
	WheelWriter {
		abi: args.abi,
		build_tag: args.build_tag,
		distribution: args.distribution,
		metadata_toml_path: args.metadata,
		packages: args.packages,
		platform: args.platform,
		python_tag: args.python,
		version: args.version,
	}
	.write(args.output)?;
	Ok(())
}
