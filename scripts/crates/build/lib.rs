use tangram_error::{err, Error, Result};

#[derive(Clone, Copy, Debug)]
pub enum Target {
	X8664UnknownLinuxGnu,
	X8664UnknownLinuxMusl,
	X8664AppleDarwin,
	AArch64AppleDarwin,
	X8664PcWindowsMsvc,
	X8664PcWindowsGnu,
}

impl std::str::FromStr for Target {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"x86_64-unknown-linux-gnu" => Ok(Target::X8664UnknownLinuxGnu),
			"x86_64-unknown-linux-musl" => Ok(Target::X8664UnknownLinuxMusl),
			"x86_64-apple-darwin" => Ok(Target::X8664AppleDarwin),
			"aarch64-apple-darwin" => Ok(Target::AArch64AppleDarwin),
			"x86_64-pc-windows-msvc" => Ok(Target::X8664PcWindowsMsvc),
			"x86_64-pc-windows-gnu" => Ok(Target::X8664PcWindowsGnu),
			_ => Err(err!("invalid target")),
		}
	}
}

impl Target {
	pub fn as_str(&self) -> &str {
		match self {
			Target::X8664UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
			Target::X8664UnknownLinuxMusl => "x86_64-unknown-linux-musl",
			Target::X8664AppleDarwin => "x86_64-apple-darwin",
			Target::AArch64AppleDarwin => "aarch64-apple-darwin",
			Target::X8664PcWindowsMsvc => "x86_64-pc-windows-msvc",
			Target::X8664PcWindowsGnu => "x86_64-pc-windows-gnu",
		}
	}
}

impl std::fmt::Display for Target {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

pub struct TargetFileNames {
	pub tangram_cli_file_name: &'static str,
	pub tangram_h_file_name: &'static str,
	pub libtangram_dynamic_file_name: &'static str,
	pub libtangram_static_file_name: &'static str,
	pub tangram_elixir_file_name: &'static str,
	pub tangram_node_file_name: &'static str,
}

pub fn target_file_names(target: Target) -> TargetFileNames {
	match target {
		Target::X8664UnknownLinuxGnu | Target::X8664UnknownLinuxMusl => TargetFileNames {
			tangram_cli_file_name: "tangram",
			tangram_h_file_name: "tangram.h",
			libtangram_dynamic_file_name: "libtangram.so",
			libtangram_static_file_name: "libtangram.a",
			tangram_elixir_file_name: "libtangram_elixir.so",
			tangram_node_file_name: "libtangram_node.so",
		},
		Target::X8664AppleDarwin | Target::AArch64AppleDarwin => TargetFileNames {
			tangram_cli_file_name: "tangram",
			tangram_h_file_name: "tangram.h",
			libtangram_dynamic_file_name: "libtangram.dylib",
			libtangram_static_file_name: "libtangram.a",
			tangram_elixir_file_name: "libtangram_elixir.dylib",
			tangram_node_file_name: "libtangram_node.dylib",
		},
		Target::X8664PcWindowsMsvc => TargetFileNames {
			tangram_cli_file_name: "tangram.exe",
			tangram_h_file_name: "tangram.h",
			libtangram_dynamic_file_name: "tangram.dll",
			libtangram_static_file_name: "tangram.lib",
			tangram_elixir_file_name: "tangram_elixir.dll",
			tangram_node_file_name: "tangram_node.dll",
		},
		Target::X8664PcWindowsGnu => TargetFileNames {
			tangram_cli_file_name: "tangram.exe",
			tangram_h_file_name: "tangram.h",
			libtangram_dynamic_file_name: "tangram.dll",
			libtangram_static_file_name: "libtangram.a",
			tangram_elixir_file_name: "tangram_elixir.dll",
			tangram_node_file_name: "tangram_node.dll",
		},
	}
}
