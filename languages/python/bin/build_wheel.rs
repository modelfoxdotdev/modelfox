use anyhow::{Context, Result};
use const_format::concatcp;
use maturin::{CargoToml, Metadata21, WheelWriter};
use std::{io, path::Path, str::FromStr};

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
	Help,
	Release,
	Testing,
}

impl FromStr for Mode {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_ascii_lowercase().as_str() {
			"help" => Ok(Mode::Help),
			"release" => Ok(Mode::Release),
			"testing" => Ok(Mode::Testing),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Unrecognized operating mode {}", s),
			)),
		}
	}
}

fn build_wheel<T, U, V>(so_path: T, cargo_toml_path: U, output_path: V) -> Result<()>
where
	T: AsRef<Path>,
	U: AsRef<Path>,
	V: AsRef<Path>,
{
	let cargo_toml_path = cargo_toml_path.as_ref();
	let so_path = so_path.as_ref();
	let output_path = output_path.as_ref();

	// Estabish clean output dir
	if std::fs::metadata(&output_path)
		.map(|m| m.is_dir())
		.unwrap_or(false)
	{
		std::fs::remove_dir_all(&output_path).unwrap();
	}
	std::fs::create_dir_all(&output_path).unwrap();

	let cargo_toml = CargoToml::from_path(cargo_toml_path).with_context(|| {
		format!(
			"Could not read Cargo.toml from {}",
			cargo_toml_path.display()
		)
	})?;
	let metadata = Metadata21::from_cargo_toml(&cargo_toml, output_path)
		.with_context(|| "Could not build Python metadata")?;
	let wheel_writer = WheelWriter::new(TAG, output_path, &metadata, &[])?;

	let result_path = wheel_writer
		.finish()
		.with_context(|| "Could not create wheel")?;
	println!("Wrote wheel to {}", result_path.display());

	Ok(())
}

fn main() -> Result<()> {
	let mut args = std::env::args().skip(2);
	let mode = Mode::from_str(&args.next().unwrap_or_else(|| {
		eprintln!("{}", HELP);
		std::process::exit(1)
	}))
	.unwrap_or_else(|e| {
		eprintln!("{}", e);
		std::process::exit(2);
	});

	if mode == Mode::Help {
		eprintln!("{}", HELP);
		std::process::exit(0);
	}

	// FIXME - this assumes we're running from tangram/languages/python - might be okay?
	let root = std::env::current_dir()?;
	let tangram_root = root.parent().unwrap().parent().unwrap();
	let lib_name = "libtangram_python.so";
	let so_path = match mode {
		Mode::Release => tangram_root.join("dist").join("x86_64-linux-gnu_2_24"),
		Mode::Testing => tangram_root.join("target").join("release"),
		_ => unreachable!(),
	}
	.join(lib_name);
	let output_path = root.join("build");
	let cargo_toml_path = root.join("Cargo.toml");

	println!("mode: {:?}", mode);
	println!("so_path: {}", so_path.display());
	println!("output_path: {}", output_path.display());

	build_wheel(so_path, cargo_toml_path, output_path)?;

	Ok(())
}
