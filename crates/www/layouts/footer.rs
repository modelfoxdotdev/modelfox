use chrono::Datelike;
use modelfox_ui as ui;
use modelfox_www_ui::logo::Logo;
use pinwheel::prelude::*;

pub struct Footer;

impl Component for Footer {
	fn into_node(self) -> Node {
		let company = div()
			.class("footer-section")
			.child(div().class("footer-section-title").child("Company"))
			.child(
				ui::Link::new()
					.href("/pricing".to_owned())
					.title("pricing".to_owned())
					.child("Pricing"),
			)
			.child(
				ui::Link::new()
					.href("mailto:hello@modelfox.dev".to_owned())
					.title("Email".to_owned())
					.child("Email Us"),
			);
		let community = div()
			.class("footer-section")
			.child(div().class("footer-section-title").child("Community"))
			.child(
				ui::Link::new()
					.href("https://discord.gg/jT9ZGp3TK2".to_owned())
					.title("Discord".to_owned())
					.child("Discord"),
			)
			.child(
				ui::Link::new()
					.href("https://github.com/modelfoxdotdev/modelfox".to_owned())
					.title("GitHub".to_owned())
					.child("GitHub"),
			);
		let learn = div()
			.class("footer-section")
			.child(div().class("footer-section-title").child("Learn"))
			.child(
				ui::Link::new()
					.href("/docs/".to_owned())
					.title("Docs".to_owned())
					.child("Docs"),
			)
			.child(
				ui::Link::new()
					.href("/benchmarks".to_owned())
					.title("Benchmarks".to_owned())
					.child("Benchmarks"),
			);
		let logo = div()
			.class("footer-logo-section")
			.child(
				div()
					.class("footer-logo-wrapper")
					.child(Logo::new().class("footer-logo".to_owned()))
					.child(div().class("footer-logo-title").child("ModelFox")),
			)
			.child(p().class("footer-copyright").child(format!(
				"Copyright © {} ModelFox, Inc",
				chrono::Utc::now().year()
			)));
		let sections = div()
			.class("footer-sections")
			.child(logo)
			.child(community)
			.child(learn)
			.child(company);
		div()
			.class("footer-wrapper")
			.child(
				div().class("footer-inner").child(sections).child(
					div().class("footer-rust").child(
						div()
							.child("This webpage was built with Rust. View the ")
							.child(
							ui::Link::new()
								.href(
									"https://github.com/modelfoxdotdev/modelfox/tree/main/crates/www"
										.to_owned(),
								)
								.child("source."),
						),
					),
				),
			)
			.into_node()
	}
}
