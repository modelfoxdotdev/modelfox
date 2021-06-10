use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::Document,
};

#[derive(ComponentBuilder)]
pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				DocsLayout::new(DocsPage::Install, None).child(
					ui::S1::new()
						.child(ui::H1::new().child("Install"))
						.child(Homebrew::new())
						.child(Deb::new(
							"ubuntu".to_owned(),
							"groovy".to_owned(),
							"Ubuntu 20.10 (Groovy Gorilla)".to_owned(),
						))
						.child(Deb::new(
							"ubuntu".to_owned(),
							"focal".to_owned(),
							"Ubuntu 20.04 LTS (Focal Fossa)".to_owned(),
						))
						.child(Deb::new(
							"ubuntu".to_owned(),
							"bionic".to_owned(),
							"Ubuntu 18.04 LTS (Bionic Beaver)".to_owned(),
						))
						.child(Deb::new(
							"debian".to_owned(),
							"sid".to_owned(),
							"Debian Sid (unstable)".to_owned(),
						))
						.child(Deb::new(
							"debian".to_owned(),
							"bullseye".to_owned(),
							"Debian Bullseye (testing)".to_owned(),
						))
						.child(Deb::new(
							"debian".to_owned(),
							"buster".to_owned(),
							"Debian Buster (stable)".to_owned(),
						))
						.child(Deb::new(
							"debian".to_owned(),
							"stretch".to_owned(),
							"Debian Stretch (oldstable)".to_owned(),
						))
						.child(Alpine::new())
						.child(Arch::new())
						.child(AmazonLinux2::new())
						.child(Centos7::new())
						.child(Centos8::new())
						.child(Fedora::new())
						.child(Rhel::new())
						.child(Scoop::new())
						.child(Docker::new())
						.child(Manual::new()),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Homebrew {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Alpine {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Owned(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct AmazonLinux2 {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Centos7 {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Fedora {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Rhel {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Centos8 {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Arch {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Scoop {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Docker {
	#[children]
	pub children: Vec<Node>,
}

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
			.child(
				ui::Window::new().child(
					ui::Code::new()
						.code(Cow::Borrowed(code))
						.hide_line_numbers(Some(true)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct Manual {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Manual {
	fn into_node(self) -> Node {
		let p = ui::P::new()
			.child("If none of the above methods works for you, you can download the tarball for your CPU architecture and operating system from ")
			.child(ui::Link::new().href("https://github.com/tangramxyz/tangram/releases/".to_owned()).child("GitHub Releases"))
			.child(". Untar the file and place the tangram executable somewhere on your ")
			.child(ui::InlineCode::new().child("PATH"))
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
