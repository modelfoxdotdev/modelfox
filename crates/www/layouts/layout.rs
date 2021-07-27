use crate::footer::Footer;
use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_ui::logo::{Logo, LogoColorScheme};

#[derive(children, Default, new)]
#[new(default)]
pub struct Layout {
	pub children: Vec<Node>,
}

impl Component for Layout {
	fn into_node(self) -> Node {
		div()
			.class("layout")
			.child(header().child(Topbar))
			.child(main().child(self.children))
			.child(footer().child(Footer))
			.into_node()
	}
}

struct Topbar;

impl Component for Topbar {
	fn into_node(self) -> Node {
		let topbar_items = vec![
			ui::TopbarItem {
				element: None,
				href: "https://github.com/tangramdotdev/tangram".to_owned(),
				title: "GitHub".to_owned(),
			},
			ui::TopbarItem {
				element: None,
				href: "/pricing".to_owned(),
				title: "Pricing".to_owned(),
			},
			ui::TopbarItem {
				element: None,
				href: "/docs/".to_owned(),
				title: "Docs".to_owned(),
			},
			ui::TopbarItem {
				element: None,
				href: "/benchmarks".to_owned(),
				title: "Benchmarks".to_owned(),
			},
			ui::TopbarItem {
				element: None,
				href: "https://app.tangram.dev".to_owned(),
				title: "Sign In".to_owned(),
			},
			ui::TopbarItem {
				element: Some({
					ui::Button::new()
						.color(ui::colors::GREEN.to_owned())
						.href("https://cozycal.com/tangram/demo".to_owned())
						.open_new_window(true)
						.child("Schedule a Demo")
						.into()
				}),
				href: "/docs/install".to_owned(),
				title: "Install the CLI".to_owned(),
			},
		];
		ui::Topbar::new()
			.background_color(ui::colors::BACKGROUND.to_owned())
			.dropdown_background_color(ui::colors::BACKGROUND.to_owned())
			.items(topbar_items)
			.logo(Logo::new().color_scheme(LogoColorScheme::Multi).into_node())
			.title("tangram".to_owned())
			.into_node()
	}
}

pub struct GithubIconLink {
	color: String,
}

impl Component for GithubIconLink {
	fn into_node(self) -> Node {
		let path = "M 8 0 C 3.58 0 0 3.58 0 8 C 0 11.54 2.29 14.53 5.47 15.59 C 5.87 15.66 6.02 15.42 6.02 15.21 C 6.02 15.02 6.01 14.39 6.01 13.72 C 4 14.09 3.48 13.23 3.32 12.78 C 3.23 12.55 2.84 11.84 2.5 11.65 C 2.22 11.5 1.82 11.13 2.49 11.12 C 3.12 11.11 3.57 11.7 3.72 11.94 C 4.44 13.15 5.59 12.81 6.05 12.6 C 6.12 12.08 6.33 11.73 6.56 11.53 C 4.78 11.33 2.92 10.64 2.92 7.58 C 2.92 6.71 3.23 5.99 3.74 5.43 C 3.66 5.23 3.38 4.41 3.82 3.31 C 3.82 3.31 4.49 3.1 6.02 4.13 C 6.66 3.95 7.34 3.86 8.02 3.86 C 8.7 3.86 9.38 3.95 10.02 4.13 C 11.55 3.09 12.22 3.31 12.22 3.31 C 12.66 4.41 12.38 5.23 12.3 5.43 C 12.81 5.99 13.12 6.7 13.12 7.58 C 13.12 10.65 11.25 11.33 9.47 11.53 C 9.76 11.78 10.01 12.26 10.01 13.01 C 10.01 14.08 10 14.94 10 15.21 C 10 15.42 10.15 15.67 10.55 15.59 C 13.71 14.53 16 11.53 16 8 C 16 3.58 12.42 0 8 0 Z";
		let icon = svg()
			.class("github-icon")
			.attribute("fill", self.color)
			.attribute("height", "24")
			.attribute("viewBox", "0 0 16 16")
			.attribute("width", "24")
			.child(svg::desc().child("github"))
			.child(svg::path().attribute("d", path));
		ui::Link::new()
			.href("https://github.com/tangramdotdev".to_owned())
			.child(icon)
			.into_node()
	}
}
