use tangram_error::{err, Error, Result};

pub enum Arch {
	X8664,
	AArch64,
}

pub const ARCHS: [Arch; 2] = [Arch::X8664, Arch::AArch64];

impl std::str::FromStr for Arch {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"x86_64" => Ok(Arch::X8664),
			"aarch64" => Ok(Arch::AArch64),
			_ => Err(err!("invalid arch")),
		}
	}
}

impl Arch {
	pub fn as_str(&self) -> &str {
		match self {
			Arch::X8664 => "x86_64",
			Arch::AArch64 => "aarch64",
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
	X8664UnknownLinuxGnu,
	AArch64UnknownLinuxGnu,
	X8664UnknownLinuxMusl,
	AArch64UnknownLinuxMusl,
	X8664AppleDarwin,
	AArch64AppleDarwin,
	X8664PcWindowsMsvc,
	X8664PcWindowsGnu,
}

pub const TARGETS: [Target; 8] = [
	Target::X8664UnknownLinuxGnu,
	Target::AArch64UnknownLinuxGnu,
	Target::X8664UnknownLinuxMusl,
	Target::AArch64UnknownLinuxMusl,
	Target::X8664AppleDarwin,
	Target::AArch64AppleDarwin,
	Target::X8664PcWindowsMsvc,
	Target::X8664PcWindowsGnu,
];

impl std::str::FromStr for Target {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"x86_64-unknown-linux-gnu" => Ok(Target::X8664UnknownLinuxGnu),
			"aarch64-unknown-linux-gnu" => Ok(Target::AArch64UnknownLinuxGnu),
			"x86_64-unknown-linux-musl" => Ok(Target::X8664UnknownLinuxMusl),
			"aarch64-unknown-linux-musl" => Ok(Target::AArch64UnknownLinuxMusl),
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
			Target::AArch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu",
			Target::X8664UnknownLinuxMusl => "x86_64-unknown-linux-musl",
			Target::AArch64UnknownLinuxMusl => "aarch64-unknown-linux-musl",
			Target::X8664AppleDarwin => "x86_64-apple-darwin",
			Target::AArch64AppleDarwin => "aarch64-apple-darwin",
			Target::X8664PcWindowsMsvc => "x86_64-pc-windows-msvc",
			Target::X8664PcWindowsGnu => "x86_64-pc-windows-gnu",
		}
	}

	pub fn arch(&self) -> Arch {
		match self {
			Target::X8664UnknownLinuxGnu
			| Target::X8664UnknownLinuxMusl
			| Target::X8664AppleDarwin
			| Target::X8664PcWindowsMsvc
			| Target::X8664PcWindowsGnu => Arch::X8664,
			Target::AArch64UnknownLinuxGnu
			| Target::AArch64UnknownLinuxMusl
			| Target::AArch64AppleDarwin => Arch::AArch64,
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
	pub tangram_elixir_src_file_name: &'static str,
	pub tangram_elixir_dst_file_name: &'static str,
	pub tangram_node_src_file_name: &'static str,
	pub tangram_node_dst_file_name: &'static str,
}

impl TargetFileNames {
	pub fn for_target(target: Target) -> TargetFileNames {
		match target {
			Target::X8664UnknownLinuxGnu
			| Target::AArch64UnknownLinuxGnu
			| Target::X8664UnknownLinuxMusl
			| Target::AArch64UnknownLinuxMusl => TargetFileNames {
				tangram_cli_file_name: "tangram",
				tangram_h_file_name: "tangram.h",
				libtangram_dynamic_file_name: "libtangram.so",
				libtangram_static_file_name: "libtangram.a",
				tangram_elixir_src_file_name: "libtangram_elixir.so",
				tangram_elixir_dst_file_name: "libtangram_elixir.so",
				tangram_node_src_file_name: "libtangram_node.so",
				tangram_node_dst_file_name: "tangram.node",
			},
			Target::X8664AppleDarwin | Target::AArch64AppleDarwin => TargetFileNames {
				tangram_cli_file_name: "tangram",
				tangram_h_file_name: "tangram.h",
				libtangram_dynamic_file_name: "libtangram.dylib",
				libtangram_static_file_name: "libtangram.a",
				tangram_elixir_src_file_name: "libtangram_elixir.dylib",
				tangram_elixir_dst_file_name: "libtangram_elixir.so",
				tangram_node_src_file_name: "libtangram_node.dylib",
				tangram_node_dst_file_name: "tangram.node",
			},
			Target::X8664PcWindowsMsvc => TargetFileNames {
				tangram_cli_file_name: "tangram.exe",
				tangram_h_file_name: "tangram.h",
				libtangram_dynamic_file_name: "tangram.dll",
				libtangram_static_file_name: "tangram.lib",
				tangram_elixir_src_file_name: "tangram_elixir.dll",
				tangram_elixir_dst_file_name: "tangram_elixir.dll",
				tangram_node_src_file_name: "tangram_node.dll",
				tangram_node_dst_file_name: "tangram.node",
			},
			Target::X8664PcWindowsGnu => TargetFileNames {
				tangram_cli_file_name: "tangram.exe",
				tangram_h_file_name: "tangram.h",
				libtangram_dynamic_file_name: "tangram.dll",
				libtangram_static_file_name: "libtangram.a",
				tangram_elixir_src_file_name: "tangram_elixir.dll",
				tangram_elixir_dst_file_name: "tangram_elixir.dll",
				tangram_node_src_file_name: "tangram_node.dll",
				tangram_node_dst_file_name: "tangram.node",
			},
		}
	}
}
