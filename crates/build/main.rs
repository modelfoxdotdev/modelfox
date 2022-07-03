#![warn(clippy::pedantic)]

use clap::Parser;
use digest::Digest;
use duct::cmd;
use indoc::formatdoc;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[derive(Parser)]
pub struct Args {
	#[clap(long)]
	version: String,
	#[clap(long, default_value = "https://pkgs.modelfox.dev")]
	pkgs_url: String,
}

fn main() {
	let Args { version, pkgs_url } = Args::parse();

	// Create the dist directory.
	let root_path = std::env::current_dir().unwrap();
	let dist_path = root_path.join("dist");
	clean_and_create(&dist_path);

	compile();
	build_containers(&version);
	build_debs(&version);
	build_rpms(&version);
	build_pkgs(&version, &pkgs_url);
	build_release(&version);
}

fn compile() {
	let root_path = std::env::current_dir().unwrap();
	let compile_path = root_path.join("dist").join("compile");
	clean_and_create(&compile_path);

	let mut args = vec!["build", "--release"];
	for target in TARGETS {
		args.extend(["--target", target.rust_target()]);
	}
	args.extend([
		"--package",
		"modelfox_cli",
		"--package",
		"libmodelfox",
		"--package",
		"modelfox_elixir",
		"--package",
		"modelfox_node",
		"--package",
		"modelfox_python",
	]);
	cmd("cargo", args).run().unwrap();
	cmd!(
		"cargo",
		"build",
		"--release",
		"--target",
		"wasm32-unknown-unknown",
		"--package",
		"modelfox_wasm",
	)
	.run()
	.unwrap();

	// Copy the artifacts to the compile directory.
	for target in TARGETS {
		let target_file_names = TargetFileNames::for_target(target);
		let cargo_artifact_path = root_path
			.join("target")
			.join(target.rust_target_name())
			.join("release");
		let dist_target_path = compile_path.join(target.target_name());
		std::fs::create_dir(&dist_target_path).unwrap();
		// cli
		let modelfox_cli_src = cargo_artifact_path.join(target_file_names.modelfox_cli_file_name);
		let modelfox_cli_dst = dist_target_path.join(target_file_names.modelfox_cli_file_name);
		std::fs::copy(modelfox_cli_src, modelfox_cli_dst).unwrap();
		// modelfox.h
		let modelfox_h_dst = dist_target_path.join(target_file_names.modelfox_h_file_name);
		cbindgen::generate(root_path.join("languages/c"))
			.unwrap()
			.write(std::fs::File::create(modelfox_h_dst).unwrap());
		// libmodelfox dynamic
		std::fs::copy(
			cargo_artifact_path.join(target_file_names.libmodelfox_dynamic_file_name),
			dist_target_path.join(target_file_names.libmodelfox_dynamic_file_name),
		)
		.unwrap();
		// libmodelfox static
		std::fs::copy(
			cargo_artifact_path.join(target_file_names.libmodelfox_static_file_name),
			dist_target_path.join(target_file_names.libmodelfox_static_file_name),
		)
		.unwrap();
		// modelfox_elixir
		std::fs::copy(
			cargo_artifact_path.join(target_file_names.modelfox_elixir_file_name),
			dist_target_path.join(target_file_names.modelfox_elixir_file_name),
		)
		.unwrap();
		// modelfox_node
		std::fs::copy(
			cargo_artifact_path.join(target_file_names.modelfox_node_file_name),
			dist_target_path.join(target_file_names.modelfox_node_file_name),
		)
		.unwrap();
		// modelfox_python
		std::fs::copy(
			cargo_artifact_path.join(target_file_names.modelfox_python_file_name),
			dist_target_path.join(target_file_names.modelfox_python_file_name),
		)
		.unwrap();
	}
	// modelfox_wasm
	let cargo_artifact_path = root_path
		.join("target")
		.join("wasm32-unknown-unknown")
		.join("release");
	let dist_target_path = compile_path.join("wasm32");
	std::fs::create_dir(&dist_target_path).unwrap();
	std::fs::copy(
		cargo_artifact_path.join("modelfox_wasm.wasm"),
		dist_target_path.join("modelfox_wasm.wasm"),
	)
	.unwrap();
}

fn build_debs(version: &str) {
	let root_path = std::env::current_dir().unwrap();
	let debs_path = root_path.join("dist").join("debs");
	clean_and_create(&debs_path);
	for target in [Target::AArch64LinuxGnu, Target::X8664LinuxGnu] {
		// Create the deb directory.
		let deb_tempdir = tempdir().unwrap();
		let deb_path = deb_tempdir.path();
		clean_and_create(deb_path);
		// Create /usr/bin in the deb directory.
		let bin_path = deb_path.join("usr").join("bin");
		std::fs::create_dir_all(&bin_path).unwrap();
		// Copy the modelfox cli to the deb's /usr/bin.
		let modelfox_cli_file_name = TargetFileNames::for_target(target).modelfox_cli_file_name;
		let modelfox_cli_path = root_path
			.join("dist")
			.join("compile")
			.join(target.target_name())
			.join(modelfox_cli_file_name);
		std::fs::copy(modelfox_cli_path, bin_path.join(modelfox_cli_file_name)).unwrap();
		// Create the control file.
		let debian_path = deb_path.join("DEBIAN");
		std::fs::create_dir_all(&debian_path).unwrap();
		let control_path = debian_path.join("control");
		let architecture = match target {
			Target::AArch64LinuxGnu => "arm64",
			Target::X8664LinuxGnu => "amd64",
			_ => unreachable!(),
		};
		let control = formatdoc!(
			r#"
				Package: modelfox
				Architecture: {architecture}
				Version: {version}
				Maintainer: ModelFox <root@modelfox.dev>
				Homepage: https://www.modelfox.dev
				Description: ModelFox makes it easy to train, deploy, and monitor machine learning models.
			"#,
			architecture = architecture,
			version = version,
		);
		std::fs::write(&control_path, &control).unwrap();
		// Run dpkg-deb
		let deb_file_name = format!(
			"modelfox_{version}_{architecture}.deb",
			version = version,
			architecture = architecture,
		);
		let deb_output_path = debs_path.join(&deb_file_name);
		cmd!("dpkg-deb", "--build", &deb_path, &deb_output_path)
			.run()
			.unwrap();
	}
}

fn build_rpms(version: &str) {
	let root_path = std::env::current_dir().unwrap();
	let compile_path = root_path.join("dist").join("compile");
	let rpms_path = root_path.join("dist").join("rpms");
	clean_and_create(&rpms_path);
	for target in [Target::AArch64LinuxGnu, Target::X8664LinuxGnu] {
		// Create the rpm directory.
		let rpm_tempdir = tempdir().unwrap();
		let rpm_path = rpm_tempdir.path();
		clean_and_create(rpm_path);
		for subdir in ["BUILD", "BUILDROOT", "RPMS", "SOURCES", "SPECS", "SRPMS"] {
			std::fs::create_dir(rpm_path.join(subdir)).unwrap();
		}
		// Make the tar.
		let modelfox_cli_file_name = TargetFileNames::for_target(target).modelfox_cli_file_name;
		let modelfox_cli_path = compile_path
			.join(target.target_name())
			.join(modelfox_cli_file_name);
		let modelfox_path_in_tar =
			PathBuf::from(format!("modelfox-{version}/modelfox", version = version));
		let sources_path = rpm_path.join("SOURCES");
		let tar_path = sources_path.join("modelfox.tar.gz");
		tar(&[(modelfox_cli_path, modelfox_path_in_tar)], &tar_path);
		// Write the spec file.
		let spec = formatdoc!(
			r#"
				%global __strip true

				Name: modelfox
				Version: {version}
				Release: 1
				Summary: ModelFox makes it easy to train, deploy, and monitor machine learning models.
				License: MIT
				Source0: modelfox.tar.gz

				%description
				%summary

				%prep
				%setup -q

				%install
				mkdir -p %buildroot/usr/bin
				install -m 755 modelfox %buildroot/usr/bin/modelfox

				%files
				%attr(0755, root, root) /usr/bin/modelfox
			"#,
			version = version,
		);
		let spec_path = rpm_path.join("SPECS/modelfox.spec");
		std::fs::write(&spec_path, spec).unwrap();
		// Run rpmbuild.
		let target = match target {
			Target::X8664LinuxGnu => "x86_64",
			Target::AArch64LinuxGnu => "aarch64",
			_ => unreachable!(),
		};
		let topdir = rpm_path.display();
		cmd!(
			"rpmbuild",
			"--quiet",
			"-D",
			format!("_topdir {topdir}", topdir = topdir),
			"--target",
			target,
			"-bb",
			spec_path,
		)
		.run()
		.unwrap();
		// Move the rpm to the release directory.
		let src_rpm_file_name = format!(
			"modelfox-{version}-1.{target}.rpm",
			version = version,
			target = target,
		);
		let dst_rpm_file_name = format!(
			"modelfox_{version}_{target}.rpm",
			version = version,
			target = target,
		);
		std::fs::copy(
			rpm_path.join("RPMS").join(target).join(&src_rpm_file_name),
			rpms_path.join(&dst_rpm_file_name),
		)
		.unwrap();
	}
}

fn build_containers(version: &str) {
	let root_path = std::env::current_dir().unwrap();
	let compile_path = root_path.join("dist").join("compile");
	for target in [Target::AArch64LinuxMusl, Target::X8664LinuxMusl] {
		let dockerfile_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
		let modelfox_cli_file_name = TargetFileNames::for_target(target).modelfox_cli_file_name;
		let modelfox_cli_path = compile_path
			.strip_prefix(&root_path)
			.unwrap()
			.join(target.target_name())
			.join(modelfox_cli_file_name);
		let modelfox_cli_path = modelfox_cli_path.display();
		let dockerfile = formatdoc!(
			r#"
				FROM docker.io/alpine
				WORKDIR /
				COPY {modelfox_cli_path} .
				ENTRYPOINT ["/modelfox"]
			"#,
			modelfox_cli_path = modelfox_cli_path,
		);
		std::fs::write(&dockerfile_path, &dockerfile).unwrap();
		let platform = match target.arch() {
			Arch::AArch64 => "linux/arm64",
			Arch::X8664 => "linux/amd64",
		};
		let tag = format!(
			"docker.io/modelfoxdotdev/modelfox:{version}",
			version = version,
		);
		cmd!(
			"podman",
			"build",
			"--platform",
			platform,
			"-f",
			&dockerfile_path,
			"--tag",
			tag,
			&root_path,
		)
		.run()
		.unwrap();
	}
}

/// # Panics
///
/// This function will panic if shelling out to `gpg --decrypt` fails.
pub fn build_pkgs(version: &str, pkgs_url: &str) {
	let root_path = std::env::current_dir().unwrap();
	let pkgs_path = root_path.join("dist").join("pkgs");
	clean_and_create(&pkgs_path);

	// Retrieve the keys from the password store.
	let alpine_public_key = cmd!("gpg", "--decrypt", "secrets/keys/alpine.public.rsa.gpg")
		.read()
		.unwrap();
	let alpine_private_key = cmd!("gpg", "--decrypt", "secrets/keys/alpine.private.rsa.gpg")
		.read()
		.unwrap();
	let deb_public_key = cmd!("gpg", "--decrypt", "secrets/keys/deb.public.gpg.gpg")
		.read()
		.unwrap();
	let deb_private_key = cmd!("gpg", "--decrypt", "secrets/keys/deb.private.gpg.gpg")
		.read()
		.unwrap();
	let rpm_public_key = cmd!("gpg", "--decrypt", "secrets/keys/rpm.public.gpg.gpg")
		.read()
		.unwrap();
	let rpm_private_key = cmd!("gpg", "--decrypt", "secrets/keys/rpm.private.gpg.gpg")
		.read()
		.unwrap();

	let alpine_public_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&alpine_public_key_path, alpine_public_key).unwrap();
	let alpine_private_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&alpine_private_key_path, alpine_private_key).unwrap();
	let deb_public_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&deb_public_key_path, deb_public_key).unwrap();
	let deb_private_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&deb_private_key_path, deb_private_key).unwrap();
	let rpm_public_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&rpm_public_key_path, rpm_public_key).unwrap();
	let rpm_private_key_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
	std::fs::write(&rpm_private_key_path, rpm_private_key).unwrap();

	alpine(
		pkgs_url,
		version,
		&alpine_public_key_path,
		&alpine_private_key_path,
	);
	deb(
		pkgs_url,
		version,
		&deb_public_key_path,
		&deb_private_key_path,
	);
	rpm(
		pkgs_url,
		version,
		&rpm_public_key_path,
		&rpm_private_key_path,
	);
}

fn alpine(
	_pkgs_url: &str,
	version: &str,
	alpine_public_key_path: &Path,
	alpine_private_key_path: &Path,
) {
	let root_path = std::env::current_dir().unwrap();
	let compile_path = root_path.join("dist").join("compile");
	let pkgs_path = root_path.join("dist").join("pkgs");
	let repo_path = pkgs_path.join("stable").join("alpine");
	clean_and_create(&repo_path);
	for target in [Target::AArch64LinuxMusl, Target::X8664LinuxMusl] {
		let src_tempdir = tempdir().unwrap();
		let src_path = src_tempdir.path();
		let dst_tempdir = tempdir().unwrap();
		let dst_path = dst_tempdir.path();
		std::fs::copy(alpine_public_key_path, src_path.join("modelfox.rsa")).unwrap();
		let apkbuild_path = src_path.join("APKBUILD");
		let arch = match target.arch() {
			Arch::AArch64 => "aarch64",
			Arch::X8664 => "x86_64",
		};
		let apkbuild = formatdoc!(
			r#"
				# Contributor: ModelFox <root@modelfox.dev>
				# Maintainer: ModelFox <root@modelfox.dev>
				pkgname=modelfox
				pkgver={version}
				pkgrel=1
				pkgdesc="ModelFox makes it easy to train, deploy, and monitor machine learning models."
				url="https://www.modelfox.dev"
				arch={arch}
				license="MIT"
				source="modelfox"
				options="!strip"

				check() {{
					:
				}}

				package() {{
					install -D -m 755 "$srcdir"/modelfox "$pkgdir"/usr/bin/modelfox
				}}
			"#,
			version = version,
			arch = arch,
		);
		std::fs::write(&apkbuild_path, &apkbuild).unwrap();
		let modelfox_cli_file_name = TargetFileNames::for_target(target).modelfox_cli_file_name;
		let modelfox_cli_dst_path = src_path.join("modelfox");
		let modelfox_cli_path = compile_path
			.join(target.target_name())
			.join(modelfox_cli_file_name);
		std::fs::copy(modelfox_cli_path, &modelfox_cli_dst_path).unwrap();
		cmd!("abuild", "-d", "-P", &dst_path, "checksum", "all")
			.dir(&src_path)
			.env("CBUILD", arch)
			.env("PACKAGER_PUBKEY", alpine_public_key_path)
			.env("PACKAGER_PRIVKEY", alpine_private_key_path)
			.run()
			.unwrap();
		cmd!("mv", dst_path.join("tmp").join(arch), &repo_path)
			.run()
			.unwrap();
	}
}

#[allow(clippy::too_many_lines)]
fn deb(pkgs_url: &str, version: &str, deb_public_key_path: &Path, deb_private_key_path: &Path) {
	struct Deb<'a> {
		arch: DebArch,
		version: &'a str,
		path: PathBuf,
	}
	#[derive(Clone, Copy, PartialEq)]
	enum DebArch {
		Amd64,
		Arm64,
	}
	impl std::fmt::Display for DebArch {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				DebArch::Amd64 => write!(f, "amd64"),
				DebArch::Arm64 => write!(f, "arm64"),
			}
		}
	}
	let root_path = std::env::current_dir().unwrap();
	let pkgs_path = root_path.join("dist").join("pkgs");
	let debs_path = root_path.join("dist").join("debs");
	let archs = [DebArch::Amd64, DebArch::Arm64];
	let debs: Vec<Deb> = archs
		.iter()
		.copied()
		.map(|arch| {
			let path = debs_path.join(format!(
				"modelfox_{version}_{arch}.deb",
				version = version,
				arch = arch,
			));
			Deb {
				arch,
				version,
				path,
			}
		})
		.collect();
	let distributions = ["debian", "ubuntu"];
	let debian_versions = ["sid", "bullseye", "buster", "stretch"];
	let ubuntu_versions = ["hirsute", "focal", "bionic"];
	for distribution in distributions {
		let repo_path = pkgs_path.join("stable").join(distribution);
		std::fs::create_dir_all(&repo_path).unwrap();
		// Create the list files.
		let distribution_versions = match distribution {
			"debian" => &debian_versions[..],
			"ubuntu" => &ubuntu_versions[..],
			_ => unreachable!(),
		};
		for distribution_version in distribution_versions {
			// Write the .list file.
			let list_path = repo_path.join(format!(
				"{distribution_version}.list",
				distribution_version = distribution_version,
			));
			let list_file = formatdoc!(
				r#"
					# modelfox packages for {distribution} {distribution_version}
					deb {pkgs_url}/stable/{distribution} {distribution_version} main
				"#,
				distribution = distribution,
				distribution_version = distribution_version,
				pkgs_url = pkgs_url,
			);
			std::fs::write(list_path, list_file).unwrap();
			// Copy the public key.
			let public_key_path = repo_path.join(format!(
				"{distribution_version}.gpg",
				distribution_version = distribution_version,
			));
			std::fs::copy(deb_public_key_path, public_key_path).unwrap();
		}
		let pool_path = repo_path.join("pool");
		std::fs::create_dir_all(&pool_path).unwrap();
		// Copy all the .debs into the pool.
		for deb in &debs {
			std::fs::copy(&deb.path, pool_path.join(&deb.path.file_name().unwrap())).unwrap();
		}
		let dists_path = repo_path.join("dists");
		std::fs::create_dir_all(&dists_path).unwrap();
		for distribution_version in distribution_versions {
			let distribution_path = dists_path.join(distribution_version);
			std::fs::create_dir_all(&distribution_path).unwrap();
			let mut md5_lines = Vec::new();
			let mut sha1_lines = Vec::new();
			let mut sha256_lines = Vec::new();
			for arch in archs {
				let component_path = distribution_path.join("main");
				std::fs::create_dir_all(&component_path).unwrap();
				let binary_arch_path = component_path.join(format!("binary-{arch}", arch = arch));
				std::fs::create_dir_all(&binary_arch_path).unwrap();
				let packages_path = binary_arch_path.join("Packages");
				let mut packages_file = String::new();
				for deb in debs.iter().filter(|deb| deb.arch == arch) {
					let deb_bytes = std::fs::read(&deb.path).unwrap();
					let deb_version = deb.version;
					let size = deb_bytes.len();
					let md5 = hex::encode(Md5::digest(&deb_bytes));
					let sha1 = hex::encode(Sha1::digest(&deb_bytes));
					let sha256 = hex::encode(Sha256::digest(&deb_bytes));
					let packages_entry = formatdoc!(
						r#"
							Package: modelfox
							Version: {deb_version}
							Architecture: {arch}
							Maintainer: ModelFox <root@modelfox.dev>
							Filename: pool/modelfox_{version}_{arch}.deb
							Size: {size}
							MD5sum: {md5}
							SHA1: {sha1}
							SHA256: {sha256}
							Homepage: https://www.modelfox.dev
							Description: ModelFox makes it easy to train, deploy, and monitor machine learning models.
						"#,
						deb_version = deb_version,
						arch = arch,
						version = version,
						size = size,
						md5 = md5,
						sha1 = sha1,
						sha256 = sha256,
					);
					packages_file.push_str(&packages_entry);
					packages_file.push('\n');
				}
				std::fs::write(&packages_path, &packages_file).unwrap();
				let packages_file_len = packages_file.len();
				let packages_relative_path = packages_path
					.strip_prefix(&distribution_path)
					.unwrap()
					.display();
				let md5 = hex::encode(Md5::digest(packages_file.as_bytes()));
				let md5_line = format!(
					" {md5} {packages_file_len} {packages_relative_path}",
					md5 = md5,
					packages_file_len = packages_file_len,
					packages_relative_path = packages_relative_path,
				);
				md5_lines.push(md5_line);
				let sha1 = hex::encode(Sha1::digest(packages_file.as_bytes()));
				let sha1_line = format!(
					" {sha1} {packages_file_len} {packages_relative_path}",
					sha1 = sha1,
					packages_file_len = packages_file_len,
					packages_relative_path = packages_relative_path,
				);
				sha1_lines.push(sha1_line);
				let sha256 = hex::encode(Sha256::digest(packages_file.as_bytes()));
				let sha256_line = format!(
					" {sha256} {packages_file_len} {packages_relative_path}",
					sha256 = sha256,
					packages_file_len = packages_file_len,
					packages_relative_path = packages_relative_path,
				);
				sha256_lines.push(sha256_line);
			}
			// Write the Release file.
			let release_file_path = distribution_path.join("Release");
			let date = chrono::Utc::now().to_rfc2822();
			let md5 = md5_lines.join("\n");
			let sha1 = sha1_lines.join("\n");
			let sha256 = sha256_lines.join("\n");
			let release_file = formatdoc!(
				r#"
					Codename: {distribution_version}
					Architectures: amd64 arm64
					Components: main
					Date: {date}
					Description: Packages from ModelFox, Inc. (https://www.modelfox.dev)
					MD5Sum:
					{md5}
					SHA1:
					{sha1}
					SHA256:
					{sha256}
				"#,
				distribution_version = distribution_version,
				date = date,
				md5 = md5,
				sha1 = sha1,
				sha256 = sha256,
			);
			std::fs::write(&release_file_path, &release_file).unwrap();
			// Write the Release.gpg file.
			sign(
				SignatureType::Detached,
				deb_private_key_path,
				&release_file_path,
				&distribution_path.join("Release.gpg"),
			);
			// Write the InRelease file.
			sign(
				SignatureType::Cleartext,
				deb_private_key_path,
				&release_file_path,
				&distribution_path.join("InRelease"),
			);
		}
	}
}

fn rpm(pkgs_url: &str, version: &str, rpm_public_key_path: &Path, rpm_private_key_path: &Path) {
	struct Rpm {
		target: RpmTarget,
		path: PathBuf,
	}
	#[derive(Clone, Copy, PartialEq)]
	enum RpmTarget {
		X8664,
		AArch64,
	}
	impl std::fmt::Display for RpmTarget {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				RpmTarget::X8664 => write!(f, "x86_64"),
				RpmTarget::AArch64 => write!(f, "aarch64"),
			}
		}
	}
	let root_path = std::env::current_dir().unwrap();
	let pkgs_path = root_path.join("dist").join("pkgs");
	let rpms_path = root_path.join("dist").join("rpms");
	// Find all the .rpms in args.rpms.
	let targets = [RpmTarget::X8664, RpmTarget::AArch64];
	let rpms: Vec<Rpm> = targets
		.iter()
		.copied()
		.map(|target| {
			let path = rpms_path.join(format!(
				"modelfox_{version}_{target}.rpm",
				version = version,
				target = target,
			));
			Rpm { target, path }
		})
		.collect();
	for (distribution, distribution_version) in [
		("amazon-linux", Some("2")),
		("centos", Some("8")),
		("centos", Some("7")),
		("fedora", None),
		("rhel", Some("8")),
	] {
		let repo_path = pkgs_path
			.join("stable")
			.join(distribution)
			.join(distribution_version.unwrap_or(""));
		std::fs::create_dir_all(&repo_path).unwrap();
		// Create the .repo file.
		let repo_file_path = repo_path.join("modelfox.repo");
		let distribution_version_with_leading_slash =
			distribution_version.map_or_else(|| "".to_owned(), |v| format!("/{v}", v = v));
		let repo_file = formatdoc!(
			r#"
				[modelfox]
				name=ModelFox
				baseurl={pkgs_url}/stable/{distribution}{distribution_version_with_leading_slash}/$basearch
				enabled=1
				type=rpm
				repo_gpgcheck=1
				gpgcheck=0
				gpgkey={pkgs_url}/stable/{distribution}{distribution_version_with_leading_slash}/repo.gpg
			"#,
			pkgs_url = pkgs_url,
			distribution = distribution,
			distribution_version_with_leading_slash = distribution_version_with_leading_slash,
		);
		std::fs::write(repo_file_path, repo_file).unwrap();
		// Copy the rpm public key.
		std::fs::copy(rpm_public_key_path, repo_path.join("repo.gpg")).unwrap();
		#[allow(clippy::single_element_loop)]
		for target in targets {
			// Create the target dir.
			let repo_target_path = repo_path.join(target.to_string());
			std::fs::create_dir_all(&repo_target_path).unwrap();
			// Copy the .rpm.
			for rpm in &rpms {
				if rpm.target == target {
					std::fs::copy(
						&rpm.path,
						repo_target_path.join(rpm.path.file_name().unwrap()),
					)
					.unwrap();
				}
			}
			// Run createrepo.
			cmd!("createrepo_c", &repo_target_path).run().unwrap();
			// Write the signature.
			sign(
				SignatureType::Detached,
				rpm_private_key_path,
				&repo_target_path.join("repodata/repomd.xml"),
				&repo_target_path.join("repodata/repomd.xml.asc"),
			);
		}
	}
}

fn build_release(version: &str) {
	let root_path = std::env::current_dir().unwrap();
	let compile_path = root_path.join("dist").join("compile");
	let release_path = root_path.join("dist").join("release");
	clean_and_create(&release_path);
	// modelfox_cli
	for target in TARGETS {
		let modelfox_cli_file_name = TargetFileNames::for_target(target).modelfox_cli_file_name;
		let modelfox_cli_path = compile_path
			.join(target.target_name())
			.join(modelfox_cli_file_name);
		let target_name = target.target_name();
		let output_path = release_path.join(format!(
			"modelfox_cli_{version}_{target_name}.tar.gz",
			version = version,
			target_name = target_name,
		));
		let inputs = vec![(
			modelfox_cli_path.clone(),
			PathBuf::from(modelfox_cli_file_name),
		)];
		tar(&inputs, &output_path);
	}
	// libmodelfox
	for target in TARGETS {
		let target_file_names = TargetFileNames::for_target(target);
		let target_path = compile_path.join(target.target_name());
		let target_name = target.target_name();
		let output_path = release_path.join(format!(
			"libmodelfox_{version}_{target_name}.tar.gz",
			version = version,
			target_name = target_name,
		));
		let inputs = vec![
			(
				target_path.join(target_file_names.modelfox_h_file_name),
				PathBuf::from(target_file_names.modelfox_h_file_name),
			),
			(
				target_path.join(target_file_names.libmodelfox_dynamic_file_name),
				PathBuf::from(target_file_names.libmodelfox_dynamic_file_name),
			),
			(
				target_path.join(target_file_names.libmodelfox_static_file_name),
				PathBuf::from(target_file_names.libmodelfox_static_file_name),
			),
		];
		tar(&inputs, &output_path);
	}
}

const TARGETS: [Target; 8] = [
	Target::AArch64LinuxGnu,
	Target::AArch64LinuxMusl,
	Target::AArch64MacOs,
	Target::X8664LinuxGnu,
	Target::X8664LinuxMusl,
	Target::X8664MacOs,
	Target::X8664WindowsGnu,
	Target::X8664WindowsMsvc,
];

pub enum Arch {
	AArch64,
	X8664,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
	AArch64LinuxGnu,
	AArch64LinuxMusl,
	AArch64MacOs,
	X8664LinuxGnu,
	X8664LinuxMusl,
	X8664MacOs,
	X8664WindowsGnu,
	X8664WindowsMsvc,
}

impl Target {
	#[must_use]
	pub fn arch(&self) -> Arch {
		match self {
			Target::AArch64LinuxGnu | Target::AArch64LinuxMusl | Target::AArch64MacOs => {
				Arch::AArch64
			}
			Target::X8664LinuxGnu
			| Target::X8664LinuxMusl
			| Target::X8664MacOs
			| Target::X8664WindowsGnu
			| Target::X8664WindowsMsvc => Arch::X8664,
		}
	}

	#[must_use]
	pub fn target_name(&self) -> &'static str {
		match self {
			Target::AArch64LinuxGnu => "aarch64-linux-gnu",
			Target::AArch64LinuxMusl => "aarch64-linux-musl",
			Target::AArch64MacOs => "aarch64-macos",
			Target::X8664LinuxGnu => "x86_64-linux-gnu",
			Target::X8664LinuxMusl => "x86_64-linux-musl",
			Target::X8664MacOs => "x86_64-macos",
			Target::X8664WindowsGnu => "x86_64-windows-gnu",
			Target::X8664WindowsMsvc => "x86_64-windows-msvc",
		}
	}

	#[must_use]
	pub fn rust_target_name(&self) -> &'static str {
		match self {
			Target::AArch64LinuxGnu => "aarch64-unknown-linux-gnu",
			Target::AArch64LinuxMusl => "aarch64-unknown-linux-musl",
			Target::AArch64MacOs => "aarch64-apple-darwin",
			Target::X8664LinuxGnu => "x86_64-unknown-linux-gnu",
			Target::X8664LinuxMusl => "x86_64-unknown-linux-musl",
			Target::X8664MacOs => "x86_64-apple-darwin",
			Target::X8664WindowsGnu => "x86_64-pc-windows-gnu",
			Target::X8664WindowsMsvc => "x86_64-pc-windows-msvc",
		}
	}

	#[must_use]
	pub fn rust_target(&self) -> &'static str {
		match self {
			Target::AArch64LinuxGnu => "aarch64-unknown-linux-gnu",
			Target::AArch64LinuxMusl => "aarch64-unknown-linux-musl",
			Target::AArch64MacOs => "aarch64-apple-darwin",
			Target::X8664LinuxGnu => "x86_64-unknown-linux-gnu",
			Target::X8664LinuxMusl => "x86_64-unknown-linux-musl",
			Target::X8664MacOs => "x86_64-apple-darwin",
			Target::X8664WindowsGnu => "x86_64-pc-windows-gnu",
			Target::X8664WindowsMsvc => "x86_64-pc-windows-msvc",
		}
	}
}

pub struct TargetFileNames {
	pub modelfox_cli_file_name: &'static str,
	pub modelfox_h_file_name: &'static str,
	pub libmodelfox_dynamic_file_name: &'static str,
	pub libmodelfox_static_file_name: &'static str,
	pub modelfox_elixir_file_name: &'static str,
	pub modelfox_node_file_name: &'static str,
	pub modelfox_python_file_name: &'static str,
}

impl TargetFileNames {
	#[must_use]
	pub fn for_target(target: Target) -> TargetFileNames {
		match target {
			Target::X8664LinuxGnu
			| Target::AArch64LinuxGnu
			| Target::X8664LinuxMusl
			| Target::AArch64LinuxMusl => TargetFileNames {
				modelfox_cli_file_name: "modelfox",
				modelfox_h_file_name: "modelfox.h",
				libmodelfox_dynamic_file_name: "libmodelfox.so",
				libmodelfox_static_file_name: "libmodelfox.a",
				modelfox_elixir_file_name: "libmodelfox_elixir.so",
				modelfox_node_file_name: "libmodelfox_node.so",
				modelfox_python_file_name: "libmodelfox_python.so",
			},
			Target::X8664MacOs | Target::AArch64MacOs => TargetFileNames {
				modelfox_cli_file_name: "modelfox",
				modelfox_h_file_name: "modelfox.h",
				libmodelfox_dynamic_file_name: "libmodelfox.dylib",
				libmodelfox_static_file_name: "libmodelfox.a",
				modelfox_elixir_file_name: "libmodelfox_elixir.dylib",
				modelfox_node_file_name: "libmodelfox_node.dylib",
				modelfox_python_file_name: "libmodelfox_python.dylib",
			},
			Target::X8664WindowsMsvc => TargetFileNames {
				modelfox_cli_file_name: "modelfox.exe",
				modelfox_h_file_name: "modelfox.h",
				libmodelfox_dynamic_file_name: "modelfox.dll",
				libmodelfox_static_file_name: "modelfox.lib",
				modelfox_elixir_file_name: "modelfox_elixir.dll",
				modelfox_node_file_name: "modelfox_node.dll",
				modelfox_python_file_name: "modelfox_python.dll",
			},
			Target::X8664WindowsGnu => TargetFileNames {
				modelfox_cli_file_name: "modelfox.exe",
				modelfox_h_file_name: "modelfox.h",
				libmodelfox_dynamic_file_name: "modelfox.dll",
				libmodelfox_static_file_name: "libmodelfox.a",
				modelfox_elixir_file_name: "modelfox_elixir.dll",
				modelfox_node_file_name: "modelfox_node.dll",
				modelfox_python_file_name: "modelfox_python.dll",
			},
		}
	}
}

fn clean_and_create(path: &Path) {
	if std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false) {
		std::fs::remove_dir_all(path).unwrap();
	}
	std::fs::create_dir_all(path).unwrap();
}

fn tar(input_paths: &[(PathBuf, PathBuf)], output_path: &Path) {
	let output_file = std::fs::File::create(output_path).unwrap();
	let gz = flate2::write::GzEncoder::new(output_file, flate2::Compression::default());
	let mut tar = tar::Builder::new(gz);
	for (path, name) in input_paths {
		tar.append_path_with_name(path, name).unwrap();
	}
	tar.finish().unwrap();
}

#[derive(Clone, Copy)]
enum SignatureType {
	Cleartext,
	Detached,
}

fn sign(signature_type: SignatureType, key: &Path, input: &Path, output: &Path) {
	let action = match signature_type {
		SignatureType::Cleartext => "--clear-sign",
		SignatureType::Detached => "--detach-sign",
	};
	let gnupghome_tempdir = tempdir().unwrap();
	let gnupghome = gnupghome_tempdir.path().display();
	let key = key.display();
	let input = input.display();
	let output = output.display();
	let script = formatdoc!(
		r#"
			export GNUPGHOME={gnupghome}
			gpg --import {key}
			cat {input} | gpg --armor {action} > {output}
		"#,
		gnupghome = gnupghome,
		key = key,
		input = input,
		action = action,
		output = output,
	);
	cmd!("sh", "-c", script).read().unwrap();
}
