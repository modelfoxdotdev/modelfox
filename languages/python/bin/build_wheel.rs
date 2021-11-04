use maturin::WheelWriter;
use std::{fs, io, path::PathBuf, str::FromStr};

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

const HELP: &str = "usage: build_wheel [help|release|testing]";

fn main() {
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
		std::process::exit(3);
	}

	let root = std::env::current_dir().unwrap();
	let tangram_root = root.parent().unwrap().parent().unwrap();
	let so_path = match mode {
		Mode::Release => tangram_root.join("dist").join("x86_64-linux-gnu_2_24"),
		Mode::Testing => tangram_root.join("target").join("release"),
		_ => unreachable!(),
	};

	println!("mode: {:?}", mode);
	println!("so_path: {}", so_path.display());
}
