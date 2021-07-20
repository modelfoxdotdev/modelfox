use anyhow::anyhow;
use clap::Clap;
use duct::cmd;
use indoc::formatdoc;
use std::{fs::File, path::Path};
use tangram_build::{Arch, Target, TargetFileNames};
use which::which;

#[derive(Clap)]
pub struct Args {
	#[clap(long)]
	target: Target,
}

pub fn run(args: Args) {
	let Args { target } = args;

	let tangram_path = std::env::current_dir().unwrap();
	let target_path = tangram_path.join(format!("target_{}", target.as_str()));

	// Clean and create the dist target path.
	let dist_path = tangram_path.join("dist");
	let dist_target_path = dist_path.join(target.as_str());
	if std::fs::metadata(&dist_target_path)
		.map(|m| m.is_dir())
		.unwrap_or(false)
	{
		std::fs::remove_dir_all(&dist_target_path).unwrap();
	}
	std::fs::create_dir_all(&dist_target_path).unwrap();

	eprintln!("compiling");
	match target {
		Target::X8664UnknownLinuxGnu => {
			build_gnu(target, None);
		}
		Target::AArch64UnknownLinuxGnu => {
			build_gnu(target, None);
		}
		Target::X8664UnknownLinuxMusl => {
			build_musl(target);
		}
		Target::AArch64UnknownLinuxMusl => {
			build_musl(target);
		}
		Target::X8664AppleDarwin => {
			build_local(target);
		}
		Target::AArch64AppleDarwin => {
			build_local(target);
		}
		Target::X8664PcWindowsMsvc => {
			build_local(target);
		}
		Target::X8664PcWindowsGnu => {
			build_gnu(target, Some(vec!["g++-mingw-w64-x86-64".to_owned()]));
		}
		Target::Wasm32UnknownUnknown => {
			build_wasm(target);
		}
	}

	if matches!(target, Target::Wasm32UnknownUnknown) {
		let cargo_artifact_path = target_path.join(target.as_str()).join("release");
		let tangram_wasm_artifact_path = cargo_artifact_path.join("tangram_wasm.wasm");
		std::fs::copy(
			tangram_wasm_artifact_path,
			dist_target_path.join("tangram_wasm.wasm"),
		)
		.unwrap();
		return;
	}

	let target_file_names = TargetFileNames::for_target(target);

	eprintln!("generating tangram.h");
	cbindgen::generate(tangram_path.join("languages/c"))
		.unwrap()
		.write(File::create(dist_target_path.join("tangram.h")).unwrap());

	eprintln!("copying artifacts");
	let (
		tangram_cli_artifact_path,
		libtangram_dynamic_artifact_path,
		libtangram_static_artifact_path,
		tangram_elixir_artifact_path,
		tangram_node_artifact_path,
	) = match target {
		Target::X8664UnknownLinuxGnu
		| Target::AArch64UnknownLinuxGnu
		| Target::X8664AppleDarwin
		| Target::AArch64AppleDarwin
		| Target::X8664PcWindowsMsvc
		| Target::X8664PcWindowsGnu => {
			let cargo_artifact_path = target_path.join(target.as_str()).join("release");
			(
				cargo_artifact_path.join(target_file_names.tangram_cli_file_name),
				cargo_artifact_path.join(target_file_names.libtangram_dynamic_file_name),
				cargo_artifact_path.join(target_file_names.libtangram_static_file_name),
				cargo_artifact_path.join(target_file_names.tangram_elixir_src_file_name),
				cargo_artifact_path.join(target_file_names.tangram_node_src_file_name),
			)
		}
		Target::X8664UnknownLinuxMusl | Target::AArch64UnknownLinuxMusl => {
			let cargo_artifact_path_dynamic = tangram_path
				.join(format!("target_{}_dynamic", target.as_str()))
				.join(target.as_str())
				.join("release");
			let cargo_artifact_path_static = tangram_path
				.join(format!("target_{}_static", target.as_str()))
				.join(target.as_str())
				.join("release");
			(
				cargo_artifact_path_static.join(target_file_names.tangram_cli_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.libtangram_dynamic_file_name),
				cargo_artifact_path_static.join(target_file_names.libtangram_static_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.tangram_elixir_src_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.tangram_node_src_file_name),
			)
		}
		Target::Wasm32UnknownUnknown => unreachable!(),
	};
	std::fs::copy(
		tangram_cli_artifact_path,
		dist_target_path.join(target_file_names.tangram_cli_file_name),
	)
	.unwrap();
	std::fs::copy(
		libtangram_dynamic_artifact_path,
		dist_target_path.join(target_file_names.libtangram_dynamic_file_name),
	)
	.unwrap();
	std::fs::copy(
		libtangram_static_artifact_path,
		dist_target_path.join(target_file_names.libtangram_static_file_name),
	)
	.unwrap();
	std::fs::copy(
		tangram_elixir_artifact_path,
		dist_target_path.join(target_file_names.tangram_elixir_src_file_name),
	)
	.unwrap();
	std::fs::copy(
		tangram_node_artifact_path,
		dist_target_path.join(target_file_names.tangram_node_src_file_name),
	)
	.unwrap();

	// Build the python wheels.
	match target {
		Target::X8664UnknownLinuxGnu => build_python_manylinux(target),
		Target::AArch64UnknownLinuxGnu => build_python_manylinux(target),
		Target::X8664UnknownLinuxMusl => {}
		Target::AArch64UnknownLinuxMusl => {}
		Target::X8664AppleDarwin => {}
		Target::AArch64AppleDarwin => build_python_macos(target),
		Target::X8664PcWindowsMsvc => build_python_windows(target),
		Target::X8664PcWindowsGnu => {}
		Target::Wasm32UnknownUnknown => unreachable!(),
	}

	// Move the python wheels to the dist target path.
	match target {
		Target::X8664UnknownLinuxGnu
		| Target::AArch64UnknownLinuxGnu
		| Target::X8664AppleDarwin
		| Target::AArch64AppleDarwin
		| Target::X8664PcWindowsMsvc => {
			let python_dist_path = tangram_path.join("languages/python/dist");
			for wheel_entry in std::fs::read_dir(python_dist_path).unwrap() {
				let wheel_path = wheel_entry.unwrap();
				std::fs::copy(
					wheel_path.path(),
					dist_target_path.join(wheel_path.file_name()),
				)
				.unwrap();
				std::fs::remove_file(wheel_path.path()).unwrap();
			}
		}
		Target::X8664UnknownLinuxMusl
		| Target::AArch64UnknownLinuxMusl
		| Target::X8664PcWindowsGnu => {}
		Target::Wasm32UnknownUnknown => unreachable!(),
	}
}

fn build_local(target: Target) {
	cmd!(
		which("cargo").unwrap(),
		"build",
		"--release",
		"--target",
		target.as_str(),
		"--package",
		"tangram_cli",
		"--package",
		"libtangram",
		"--package",
		"tangram_elixir",
		"--package",
		"tangram_node",
	)
	.env("CARGO_TARGET_DIR", format!("target_{}", target.as_str()))
	.run()
	.unwrap();
}

fn build_gnu(target: Target, apt_packages: Option<Vec<String>>) {
	let cwd = std::env::current_dir().unwrap();
	let home_dir = dirs::home_dir()
		.ok_or_else(|| anyhow!("could not get home dir"))
		.unwrap();
	let arch = target.arch();
	let docker_platform = match arch {
		Arch::X8664 => "linux/amd64",
		Arch::AArch64 => "linux/arm64",
		Arch::Wasm32 => unreachable!(),
	};
	let apt_packages = apt_packages
		.map(|apt_packages| apt_packages.join(" "))
		.unwrap_or_else(|| "".to_owned());
	let script = format!(
		r#"
			apt update
			apt install -y curl build-essential {apt_packages}
			curl -sSf https://sh.rustup.rs | sh -s -- -y
			export PATH="$HOME/.cargo/bin:$PATH"
			rustup target add {target}
			CARGO_TARGET_DIR=target_{target} \
			cargo build \
				--release \
				--target {target} \
				--package tangram_cli \
				--package libtangram \
				--package tangram_elixir \
				--package tangram_node
		"#,
		apt_packages = apt_packages,
		target = target,
	);
	cmd!(
		"docker",
		"run",
		"--platform",
		docker_platform,
		"-i",
		"--rm",
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/registry").display(),
			"/root/.cargo/registry",
		),
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/git").display(),
			"/root/.cargo/git",
		),
		"-v",
		format!("{}:{}", cwd.display(), "/tangram"),
		"-w",
		"/tangram",
		"debian:stretch",
		"sh",
	)
	.stdin_bytes(script)
	.run()
	.unwrap();
}

fn build_musl(target: Target) {
	let cwd = std::env::current_dir().unwrap();
	let home_dir = dirs::home_dir()
		.ok_or_else(|| anyhow!("could not get home dir"))
		.unwrap();
	let arch = target.arch();
	let docker_platform = match arch {
		Arch::X8664 => "linux/amd64",
		Arch::AArch64 => "linux/arm64",
		Arch::Wasm32 => unreachable!(),
	};
	let script = format!(
		r#"
			apk add build-base curl
			curl -sSf https://sh.rustup.rs | sh -s -- -y
			export PATH="$HOME/.cargo/bin:$PATH"
			# build dynamic libraries with -crt-static
			CARGO_TARGET_DIR=target_{target}_dynamic \
			RUSTFLAGS="-C target-feature=-crt-static" \
			cargo build \
				--release \
				--target {target} \
				--package libtangram \
				--package tangram_elixir \
				--package tangram_node
			# build tangram_cli and libtangram static with +crt-static
			CARGO_TARGET_DIR=target_{target}_static \
			cargo build \
				--release \
				--target {target} \
				--package tangram_cli \
				--package libtangram
		"#,
		target = target,
	);
	cmd!(
		"docker",
		"run",
		"--platform",
		docker_platform,
		"-i",
		"--rm",
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/registry").display(),
			"/root/.cargo/registry",
		),
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/git").display(),
			"/root/.cargo/git",
		),
		"-v",
		format!("{}:{}", cwd.display(), "/tangram"),
		"-w",
		"/tangram",
		"alpine:3.13",
		"sh",
	)
	.stdin_bytes(script)
	.run()
	.unwrap();
}

fn build_python_manylinux(target: Target) {
	let cwd = std::env::current_dir().unwrap();
	let home_dir = dirs::home_dir()
		.ok_or_else(|| anyhow!("could not get home dir"))
		.unwrap();
	let arch = target.arch();
	let docker_platform = match arch {
		Arch::X8664 => "linux/amd64",
		Arch::AArch64 => "linux/arm64",
		Arch::Wasm32 => unreachable!(),
	};
	let script = formatdoc!(
		r#"
			set -e
			curl -sSf https://sh.rustup.rs | sh -s -- -y
			export PATH="$HOME/.cargo/bin:$PATH"
			/opt/python/cp36-cp36m/bin/pip install -U setuptools wheel setuptools-rust
			rm -rf dist
			CARGO_TARGET_DIR="../../target_{target}_python" /opt/python/cp36-cp36m/bin/python setup.py bdist_wheel --py-limited-api=cp36
			for WHEEL in dist/*.whl; do
				auditwheel repair $WHEEL -w dist
				rm $WHEEL
			done
			rm -rf build tangram.egg-info
		"#,
		target = target,
	);
	cmd!(
		"docker",
		"run",
		"--platform",
		docker_platform,
		"-i",
		"--rm",
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/registry").display(),
			"/root/.cargo/registry",
		),
		"-v",
		format!(
			"{}:{}",
			home_dir.join(".cargo/git").display(),
			"/root/.cargo/git",
		),
		"-v",
		format!("{}:{}", cwd.display(), "/tangram"),
		"-w",
		"/tangram/languages/python",
		format!("quay.io/pypa/manylinux_2_24_{}", arch.as_str()),
		"bash",
	)
	.stdin_bytes(script)
	.run()
	.unwrap();
}

fn build_python_macos(target: Target) {
	cmd!(
		"pip3",
		"install",
		"-U",
		"setuptools",
		"wheel",
		"setuptools-rust"
	)
	.run()
	.unwrap();
	let dist_path = Path::new("languages/python/dist");
	if std::fs::metadata(&dist_path)
		.map(|m| m.is_dir())
		.unwrap_or(false)
	{
		std::fs::remove_dir_all(&dist_path).unwrap();
	}
	cmd!(
		"python3",
		"setup.py",
		"bdist_wheel",
		"--py-limited-api",
		"cp36"
	)
	.env(
		"CARGO_TARGET_DIR",
		format!("../../target_{}_python", target.as_str()),
	)
	.env("ARCHFLAGS", "-arch x86_64 -arch arm64")
	.env("PYO3_NO_PYTHON", "")
	.dir("languages/python")
	.run()
	.unwrap();
}

fn build_python_windows(target: Target) {
	cmd!(
		"pip",
		"install",
		"-U",
		"setuptools",
		"wheel",
		"setuptools-rust"
	)
	.run()
	.unwrap();
	let dist_path = Path::new("languages/python/dist");
	if std::fs::metadata(&dist_path)
		.map(|m| m.is_dir())
		.unwrap_or(false)
	{
		std::fs::remove_dir_all(&dist_path).unwrap();
	}
	cmd!(
		"python",
		"setup.py",
		"bdist_wheel",
		"--py-limited-api",
		"cp36"
	)
	.env(
		"CARGO_TARGET_DIR",
		format!("../../target_{}_python", target.as_str()),
	)
	.dir("languages/python")
	.run()
	.unwrap();
}

fn build_wasm(target: Target) {
	cmd!(
		"cargo",
		"build",
		"--release",
		"--target",
		target.as_str(),
		"--package",
		"tangram_wasm",
	)
	.env("CARGO_TARGET_DIR", format!("target_{}", target.as_str()))
	.run()
	.unwrap();
}
