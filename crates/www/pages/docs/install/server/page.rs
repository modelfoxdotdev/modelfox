use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				DocsLayout::new().selected_page(DocsPage::Install).child(
					ui::S1::new()
						.child(ui::H1::new().child("Install"))
						.child(Homebrew)
						.child(Deb {
							distribution: "ubuntu".to_owned(),
							version: "hirsute".to_owned(),
							title: "Ubuntu 21.04 (Hirsute Hippo)".to_owned(),
						})
						.child(Deb {
							distribution: "ubuntu".to_owned(),
							version: "focal".to_owned(),
							title: "Ubuntu 20.04 LTS (Focal Fossa)".to_owned(),
						})
						.child(Deb {
							distribution: "ubuntu".to_owned(),
							version: "bionic".to_owned(),
							title: "Ubuntu 18.04 LTS (Bionic Beaver)".to_owned(),
						})
						.child(Deb {
							distribution: "debian".to_owned(),
							version: "sid".to_owned(),
							title: "Debian Sid (unstable)".to_owned(),
						})
						.child(Deb {
							distribution: "debian".to_owned(),
							version: "bullseye".to_owned(),
							title: "Debian Bullseye (testing)".to_owned(),
						})
						.child(Deb {
							distribution: "debian".to_owned(),
							version: "buster".to_owned(),
							title: "Debian Buster (stable)".to_owned(),
						})
						.child(Deb {
							distribution: "debian".to_owned(),
							version: "stretch".to_owned(),
							title: "Debian Stretch (oldstable)".to_owned(),
						})
						.child(Alpine)
						.child(Arch)
						.child(AmazonLinux2)
						.child(Centos7)
						.child(Centos8)
						.child(Fedora)
						.child(Nix)
						.child(Rhel)
						.child(Scoop)
						.child(Docker)
						.child(Manual),
				),
			)
			.into_node()
	}
}

struct Homebrew;

impl Component for Homebrew {
	fn into_node(self) -> Node {
		let code = "brew install tangramxyz/tap/tangram";
		ui::S2::new()
			.child(ui::H2::new().child("Homebrew"))
			.child(
				ui::P::new()
					.child("Install the tangram package from the ")
					.child(
						ui::Link::new()
							.href("https://github.com/tangramxyz/homebrew-tap".to_owned())
							.child("homebrew tap"),
					)
					.child(":"),
			)
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Alpine;

impl Component for Alpine {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram rsa key.
				curl -fsSL https://pkgs.tangram.xyz/stable/alpine/tangram.rsa | tee /etc/apk/keys/tangram.rsa
				# Add the tangram repository.
				echo "https://pkgs.tangram.xyz/stable/alpine" | tee /etc/apk/repositories
				# Install!
				apk add tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Alpine"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

pub struct Deb {
	distribution: String,
	version: String,
	title: String,
}

impl Component for Deb {
	fn into_node(self) -> Node {
		let code = ui::formatdoc!(
			r#"
				# Add the tangram gpg key.
				curl -fsSL https://pkgs.tangram.xyz/stable/{distribution}/{version}.gpg | sudo apt-key add -
				# Add the tangram repository.
				curl -fsSL https://pkgs.tangram.xyz/stable/{distribution}/{version}.list | sudo tee /etc/apt/sources.list.d/tangram.list
				# Install!
				sudo apt-get update && sudo apt-get install tangram
			"#,
			distribution = self.distribution,
			version = self.version,
		);
		ui::S2::new()
			.child(ui::H2::new().child(self.title))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Owned(code))))
			.into_node()
	}
}

struct AmazonLinux2;

impl Component for AmazonLinux2 {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram repository.
				sudo yum-config-manager --add-repo https://pkgs.tangram.xyz/stable/amazon-linux/2/tangram.repo
				# Install!
				sudo yum install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Amazon Linux 2"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Centos7;

impl Component for Centos7 {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram repository.
				sudo yum-config-manager --add-repo https://pkgs.tangram.xyz/stable/centos/7/tangram.repo
				# Install!
				sudo yum install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Centos 7"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Fedora;

impl Component for Fedora {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram repository.
				sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/fedora/tangram.repo
				# Install!
				sudo dnf install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Fedora"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Nix;

impl Component for Nix {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# To avoid having to build from scratch, use the tangram cachix cache:
				# https://tangram.cachix.org
				# tangram.cachix.org-1:NQ5Uzhhbrgi4R6A0JoljrMg8X4a2doTv3WrSnajJANs=
				nix run github:tangramxyz/tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Nix"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Rhel;

impl Component for Rhel {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram repository.
				sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/rhel/8/tangram.repo
				# Install!
				sudo dnf install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("RHEL 8"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Centos8;

impl Component for Centos8 {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				# Add the tangram repository.
				sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/centos/8/tangram.repo
				# Install!
				sudo dnf install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Centos 8"))
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Arch;

impl Component for Arch {
	fn into_node(self) -> Node {
		let code = "yay -S tangram-bin";
		ui::S2::new()
			.child(ui::H2::new().child("Arch"))
			.child(
				ui::P::new()
					.child("Install the tangram package from the ")
					.child(
						ui::Link::new()
							.href("https://aur.archlinux.org/packages/tangram-bin".to_owned())
							.child("AUR"),
					)
					.child(":"),
			)
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Scoop;

impl Component for Scoop {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				scoop bucket add tangram https://github.com/tangramxyz/scoop.git
				scoop install tangram
			"#
		);
		ui::S2::new()
			.child(ui::H2::new().child("Scoop"))
			.child(
				ui::P::new()
					.child("Install the tangram package from the ")
					.child(
						ui::Link::new()
							.href("https://aur.archlinux.org/packages/tangram".to_owned())
							.child("scoop bucket"),
					)
					.child(":"),
			)
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Docker;

impl Component for Docker {
	fn into_node(self) -> Node {
		let code =
		"docker run --rm -it tangramxyz/tangram train --file heart_disease.csv --target diagnosis";
		ui::S2::new()
			.child(ui::H2::new().child("Docker"))
			.child(
				ui::P::new()
					.child("Run the tangramxyz/tangram docker image from ")
					.child(
						ui::Link::new()
							.href("https://hub.docker.com/tangramxyz/tangram".to_owned())
							.child("Docker Hub"),
					)
					.child(":"),
			)
			.child(ui::Window::new().child(ui::Code::new().code(Cow::Borrowed(code))))
			.into_node()
	}
}

struct Manual;

impl Component for Manual {
	fn into_node(self) -> Node {
		let p = ui::P::new()
			.child("If none of the above methods works for you, you can download the tarball for your CPU architecture and operating system from ")
			.child(ui::Link::new().href("https://github.com/tangramxyz/tangram/releases/".to_owned()).child("GitHub Releases"))
			.child(". Untar the file and place the tangram executable somewhere on your ")
			.child(ui::InlineCode::new("PATH"))
			.child(". If you do this, please email us at ")
			.child(ui::Link::new().href("mailto:hello@tangram.xyz".to_owned())
			.child("hello@tangram.xyz"))
			.child(" so we can consider supporting your preferred installation method.");
		ui::S2::new()
			.child(ui::H2::new().child("Install Manually"))
			.child(p)
			.into_node()
	}
}
