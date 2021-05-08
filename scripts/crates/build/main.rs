use clap::Clap;
use duct::cmd;
use log::info;
use std::{fs::File, io::prelude::*};
use tangram_error::{err, Result};
use tangram_script_build::{target_file_names, Target};
use which::which;

#[derive(Clap)]
pub struct Args {
	#[clap(long)]
	target: Target,
}

pub fn main() -> Result<()> {
	let env = env_logger::Env::new().filter_or("LOG", "info");
	env_logger::Builder::from_env(env)
		.format(|buf, record| writeln!(buf, "{}", record.args()))
		.init();

	let args = Args::parse();
	let Args { target } = args;

	let tangram_path = std::env::current_dir()?;

	// Clean and create the dist target path.
	let dist_path = tangram_path.join("dist");
	let dist_target_path = dist_path.join(target.as_str());
	let dist_target_path_exists = std::fs::metadata(&dist_target_path)
		.map(|m| m.is_dir())
		.unwrap_or(false);
	if dist_target_path_exists {
		std::fs::remove_dir_all(&dist_target_path)?;
	}
	std::fs::create_dir_all(&dist_target_path)?;

	let target_file_names = target_file_names(target);

	info!("building");
	match target {
		Target::X8664UnknownLinuxMusl => {
			build_musl(target)?;
		}
		Target::X8664UnknownLinuxGnu => {
			build_local(target)?;
		}
		Target::X8664AppleDarwin => {
			build_local(target)?;
		}
		Target::AArch64AppleDarwin => {
			build_local(target)?;
		}
		Target::X8664PcWindowsMsvc => {
			build_local(target)?;
		}
		Target::X8664PcWindowsGnu => {
			build_local(target)?;
		}
	}

	info!("generating tangram.h");
	cbindgen::generate(tangram_path.join("languages/c"))?
		.write(File::create(dist_target_path.join("tangram.h"))?);

	info!("copying artifacts");
	let (
		tangram_cli_artifact_path,
		libtangram_dynamic_artifact_path,
		libtangram_static_artifact_path,
		tangram_elixir_artifact_path,
		tangram_node_artifact_path,
	) = match target {
		Target::X8664UnknownLinuxGnu
		| Target::X8664AppleDarwin
		| Target::AArch64AppleDarwin
		| Target::X8664PcWindowsMsvc
		| Target::X8664PcWindowsGnu => {
			let cargo_artifact_path = tangram_path
				.join("target")
				.join(target.as_str())
				.join("release");
			(
				cargo_artifact_path.join(target_file_names.tangram_cli_file_name),
				cargo_artifact_path.join(target_file_names.libtangram_dynamic_file_name),
				cargo_artifact_path.join(target_file_names.libtangram_static_file_name),
				cargo_artifact_path.join(target_file_names.tangram_elixir_file_name),
				cargo_artifact_path.join(target_file_names.tangram_node_file_name),
			)
		}
		Target::X8664UnknownLinuxMusl => {
			let cargo_artifact_path_dynamic = tangram_path
				.join("target_musl_dynamic")
				.join(target.as_str())
				.join("release");
			let cargo_artifact_path_static = tangram_path
				.join("target_musl_static")
				.join(target.as_str())
				.join("release");
			(
				cargo_artifact_path_static.join(target_file_names.tangram_cli_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.libtangram_dynamic_file_name),
				cargo_artifact_path_static.join(target_file_names.libtangram_static_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.tangram_elixir_file_name),
				cargo_artifact_path_dynamic.join(target_file_names.tangram_node_file_name),
			)
		}
	};
	std::fs::copy(
		tangram_cli_artifact_path,
		dist_target_path.join(target_file_names.tangram_cli_file_name),
	)?;
	std::fs::copy(
		libtangram_dynamic_artifact_path,
		dist_target_path.join(target_file_names.libtangram_dynamic_file_name),
	)?;
	std::fs::copy(
		libtangram_static_artifact_path,
		dist_target_path.join(target_file_names.libtangram_static_file_name),
	)?;
	std::fs::copy(
		tangram_elixir_artifact_path,
		dist_target_path.join(target_file_names.tangram_elixir_file_name),
	)?;
	std::fs::copy(
		tangram_node_artifact_path,
		dist_target_path.join(target_file_names.tangram_node_file_name),
	)?;

	// Build the python wheels.
	match target {
		Target::X8664UnknownLinuxGnu => build_python_manylinux()?,
		Target::X8664UnknownLinuxMusl => {}
		Target::X8664AppleDarwin => {}
		Target::AArch64AppleDarwin => build_python_macos()?,
		Target::X8664PcWindowsMsvc => build_python_windows()?,
		Target::X8664PcWindowsGnu => {}
	}

	// Move the python wheels to the dist target path.
	match target {
		Target::X8664UnknownLinuxGnu
		| Target::X8664AppleDarwin
		| Target::AArch64AppleDarwin
		| Target::X8664PcWindowsMsvc => {
			let python_dist_path = tangram_path.join("languages/python/dist");
			for wheel_entry in std::fs::read_dir(python_dist_path)? {
				let wheel_path = wheel_entry?;
				std::fs::copy(
					wheel_path.path(),
					dist_target_path.join(wheel_path.file_name()),
				)?;
				std::fs::remove_file(wheel_path.path())?;
			}
		}
		Target::X8664UnknownLinuxMusl | Target::X8664PcWindowsGnu => {}
	}

	Ok(())
}

fn build_local(target: Target) -> Result<()> {
	cmd!(
		which("cargo")?,
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
	.run()?;
	Ok(())
}

fn build_musl(target: Target) -> Result<()> {
	let cwd = std::env::current_dir()?;
	let home_dir = dirs::home_dir().ok_or_else(|| err!("could not get home dir"))?;
	let script = format!(
		r#"
			apk add build-base curl
			curl -sSf https://sh.rustup.rs | sh -s -- -y
			export PATH="$HOME/.cargo/bin:$PATH"
			# build dynamic libraries with -crt-static
			CARGO_TARGET_DIR=target_musl_dynamic \
			RUSTFLAGS="-C target-feature=-crt-static" \
			cargo build \
				--release \
				--target {target} \
				--package libtangram \
				--package tangram_elixir \
				--package tangram_node
			# build tangram_cli and libtangram static with +crt-static
			CARGO_TARGET_DIR=target_musl_static \
			cargo build \
				--release \
				--target {target} \
				--package tangram_cli \
				--package libtangram
		"#,
		target = target,
	);
	cmd!(
		"podman",
		"run",
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
		"alpine",
		"sh",
	)
	.stdin_bytes(script)
	.run()?;
	Ok(())
}

fn build_python_manylinux() -> Result<()> {
	let cwd = std::env::current_dir()?;
	let home_dir = dirs::home_dir().ok_or_else(|| err!("could not get home dir"))?;
	let script = r#"
		set -ex
		curl -sSf https://sh.rustup.rs | sh -s -- -y
		export PATH="$HOME/.cargo/bin:$PATH"
		/opt/python/cp36-cp36m/bin/pip install -U setuptools wheel setuptools-rust
		rm -rf dist
		CARGO_TARGET_DIR="../../target_python" /opt/python/cp36-cp36m/bin/python setup.py bdist_wheel --py-limited-api=cp36
		for WHEEL in dist/*.whl; do
			auditwheel repair $WHEEL -w dist
			rm $WHEEL
		done
		rm -rf build tangram.egg-info
	"#;
	cmd!(
		"podman",
		"run",
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
		"quay.io/pypa/manylinux_2_24_x86_64",
		"bash",
	)
	.stdin_bytes(script)
	.run()?;
	Ok(())
}

fn build_python_macos() -> Result<()> {
	cmd!(
		"pip3",
		"install",
		"-U",
		"setuptools",
		"wheel",
		"setuptools-rust"
	)
	.run()?;
	std::fs::remove_dir_all("languages/python/dist")?;
	cmd!(
		"python3",
		"setup.py",
		"bdist_wheel",
		"--py-limited-api",
		"cp36"
	)
	.env("CARGO_TARGET_DIR", "../../target_python")
	.env("ARCHFLAGS", "-arch x86_64 -arch arm64")
	.env("PYO3_NO_PYTHON", "")
	.dir("languages/python")
	.run()?;
	Ok(())
}

fn build_python_windows() -> Result<()> {
	cmd!(
		"pip",
		"install",
		"-U",
		"setuptools",
		"wheel",
		"setuptools-rust"
	)
	.run()?;
	std::fs::remove_dir_all("languages/python/dist")?;
	cmd!(
		"python",
		"setup.py",
		"bdist_wheel",
		"--py-limited-api",
		"cp36"
	)
	.env("CARGO_TARGET_DIR", "../../target_python")
	.dir("languages/python")
	.run()?;
	Ok(())
}
