use html::{component, html, Props};
use indoc::{formatdoc, indoc};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::Install} headings={None}>
				<ui::S1>
					<ui::H1>{"Install"}</ui::H1>
					<Homebrew />
					<Deb
						distribution="ubuntu"
						version="groovy"
						title="Ubuntu 20.10 (Groovy Gorilla)"
					/>
					<Deb
						distribution="ubuntu"
						version="focal"
						title="Ubuntu 20.04 LTS (Focal Fossa)"
					/>
					<Deb
						distribution="ubuntu"
						version="bionic"
						title="Ubuntu 18.04 LTS (Bionic Beaver)"
					/>
					<Deb
						distribution="debian"
						version="sid"
						title="Debian Sid (unstable)"
					/>
					<Deb
						distribution="debian"
						version="bullseye"
						title="Debian Bullseye (testing)"
					/>
					<Deb
						distribution="debian"
						version="buster"
						title="Debian Buster (stable)"
					/>
					<Deb
						distribution="debian"
						version="stretch"
						title="Debian Stretch (oldstable)"
					/>
					<Alpine />
					<Arch />
					<AmazonLinux2 />
					<Centos7 />
					<Centos8 />
					<Fedora />
					<Rhel />
					<Scoop />
					<Docker />
					<Manual />
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}

#[component]
fn Homebrew() {
	let code = "brew install tangramxyz/tap/tangram";
	html! {
		<ui::S2>
			<ui::H2>{"Homebrew"}</ui::H2>
			<ui::P>
				{"Install the tangram package from the "}
				<ui::Link href="https://github.com/tangramxyz/homebrew-tap">
					{"homebrew tap"}
				</ui::Link>
				{":"}
			</ui::P>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code.to_owned()} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Alpine() {
	let code = formatdoc! {
		r#"
			# Add the tangram rsa key.
			curl -fsSL https://pkgs.tangram.xyz/stable/alpine/tangram.rsa | tee /etc/apk/keys/tangram.rsa
			# Add the tangram repository.
			echo "https://pkgs.tangram.xyz/stable/alpine" | tee /etc/apk/repositories
			# Install!
			apk add tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Alpine"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct DebProps {
	distribution: String,
	version: String,
	title: String,
}

#[component]
fn Deb(props: DebProps) {
	let code = formatdoc! {
		r#"
			# Add the tangram gpg key.
			curl -fsSL https://pkgs.tangram.xyz/stable/{distribution}/{version}.gpg | sudo apt-key add -
			# Add the tangram repository.
			curl -fsSL https://pkgs.tangram.xyz/stable/{distribution}/{version}.list | sudo tee /etc/apt/sources.list.d/tangram.list
			# Install!
			sudo apt-get update && sudo apt-get install tangram
		"#,
		distribution = props.distribution,
		version = props.version,
	};
	html! {
		<ui::S2>
			<ui::H2>{props.title}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn AmazonLinux2() {
	let code = indoc! {
		r#"
			# Add the tangram repository.
			sudo yum-config-manager --add-repo https://pkgs.tangram.xyz/stable/amazon-linux/2/tangram.repo
			# Install!
			sudo yum install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Amazon Linux 2"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code.to_owned()} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Centos7() {
	let code = formatdoc! {
		r#"
			# Add the tangram repository.
			sudo yum-config-manager --add-repo https://pkgs.tangram.xyz/stable/centos/7/tangram.repo
			# Install!
			sudo yum install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Centos 7"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Fedora() {
	let code = formatdoc! {
		r#"
			# Add the tangram repository.
			sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/fedora/tangram.repo
			# Install!
			sudo dnf install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Fedora"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Rhel() {
	let code = formatdoc! {
		r#"
			# Add the tangram repository.
			sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/rhel/8/tangram.repo
			# Install!
			sudo dnf install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"RHEL 8"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Centos8() {
	let code = formatdoc! {
		r#"
			# Add the tangram repository.
			sudo dnf config-manager --add-repo https://pkgs.tangram.xyz/stable/centos/8/tangram.repo
			# Install!
			sudo dnf install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Centos 8"}</ui::H2>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Arch() {
	let code = "yay -S tangram";
	html! {
		<ui::S2>
			<ui::H2>{"Arch"}</ui::H2>
			<ui::P>
				{"Install the tangram package from the "}
				<ui::Link
					href="https://aur.archlinux.org/packages/tangram"
				>
					{"AUR"}
				</ui::Link>
				{":"}
			</ui::P>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code.to_owned()} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Scoop() {
	let code = indoc! {
		r#"
			scoop bucket add tangram https://github.com/tangramxyz/scoop.git
			scoop install tangram
		"#
	};
	html! {
		<ui::S2>
			<ui::H2>{"Windows Scoop"}</ui::H2>
			<ui::P>
				{"Install the tangram package from the "}
				<ui::Link href="https://aur.archlinux.org/packages/tangram">
					{"scoop bucket"}
				</ui::Link>
				{":"}
			</ui::P>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code.to_owned()} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Docker() {
	let code =
		"docker run --rm -it tangramxyz/tangram train --file heart_disease.csv --target diagnosis";
	html! {
		<ui::S2>
			<ui::H2>{"Docker"}</ui::H2>
			<ui::P>
				{"Run the tangramxyz/tangram docker image from "}
				<ui::Link href="https://hub.docker.com/tangramxyz/tangram">
					{"Docker Hub"}
				</ui::Link>
				{":"}
			</ui::P>
			<ui::Window padding={Some(true)}>
				<ui::Code hide_line_numbers?={Some(true)} code={code.to_owned()} />
			</ui::Window>
		</ui::S2>
	}
}

#[component]
fn Manual() {
	html! {
		<ui::S2>
			<ui::H2>{"Install Manually"}</ui::H2>
			<ui::P>
				{"If none of the above methods works for you, you can download the tarball for your CPU architecture and operating system from "}
				<ui::Link href="https://github.com/tangramxyz/tangram/releases/">
					{"GitHub Releases"}
				</ui::Link>
				{". Untar the file and place the tangram executable somewhere on your "}
				<ui::InlineCode>{"PATH"}</ui::InlineCode>
				{". If you do this, please email us at "}
				<ui::Link href="mailto:hello@tangram.xyz">
					{"hello@tangram.xyz"}
				</ui::Link>
				{" so we can consider supporting your preferred installation method."}
			</ui::P>
		</ui::S2>
	}
}
