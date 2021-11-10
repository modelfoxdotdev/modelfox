use chrono::Datelike;
use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_ui::logo::{Logo, LogoColorScheme};

pub struct Footer;

impl Component for Footer {
	fn into_node(self) -> Node {
		let company = div()
			.class("footer-section")
			.child(div().class("footer-section-title").child("Company"))
			.child(
				ui::Link::new()
					.href("/jobs".to_owned())
					.title("jobs".to_owned())
					.child("Jobs"),
			)
			.child(
				ui::Link::new()
					.href("/about".to_owned())
					.title("about".to_owned())
					.child("About"),
			)
			.child(
				ui::Link::new()
					.href("/pricing".to_owned())
					.title("pricing".to_owned())
					.child("Pricing"),
			)
			.child(
				ui::Link::new()
					.href("mailto:hello@tangram.dev".to_owned())
					.title("Email".to_owned())
					.child("Email Us"),
			);
		let app = div()
			.class("footer-section")
			.child(div().class("footer-section-title").child("App"))
			.child(
				ui::Link::new()
					.href("https://app.tangram.dev".to_owned())
					.title("Log In".to_owned())
					.child("Log In"),
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
					.href("https://github.com/tangramdotdev/tangram".to_owned())
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
					.child(
						Logo::new()
							.class("footer-logo".to_owned())
							.color_scheme(LogoColorScheme::Multi),
					)
					.child(div().class("footer-logo-title").child("tangram")),
			)
			.child(p().class("footer-copyright").child(format!(
				"Copyright © {} Tangram, Inc",
				chrono::Utc::now().year()
			)));
		let sections = div()
			.class("footer-sections")
			.child(logo)
			.child(community)
			.child(learn)
			.child(company)
			.child(app);
		div()
			.class("footer-wrapper")
			.child(
				div().class("footer-inner").child(sections).child(
					div().class("footer-rust").child(Crab).child(
						div()
							.child("This webpage was built with Rust. View the ")
							.child(
							ui::Link::new()
								.href(
									"https://github.com/tangramdotdev/tangram/tree/main/crates/www"
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

struct Crab;
impl Component for Crab {
	fn into_node(self) -> Node {
		div().style("width", "40px")
		.child(
		svg()
			.attribute("viewBox", "0 0 354 284")
			.attribute("fill", "none")
			.attribute("xmlns", "http://www.w3.org/2000/svg")
			.child(
				svg::path()
					.attribute(
						"d",
						"M141.42 71L211.777 71.3536L212.131 141.711L141.774 141.357L141.42 71Z",
					)
					.attribute("fill", "#FFD60A"),
			)
			.child(
				svg::path()
					.attribute("d", "M282.421 282.841L141 141.42H282.421V282.841Z")
					.attribute("fill", "#30D158"),
			)
			.child(
				svg::path()
					.attribute("d", "M353.551 70.7107H282.84V0L353.551 70.7107Z")
					.attribute("fill", "#BF5AF2"),
			)
			.child(
				svg::path()
					.attribute(
						"d",
						"M212.13 141.355H282.841L353.551 70.6447H282.841L212.13 141.355Z",
					)
					.attribute("fill", "#FF375F"),
			)
			.child(
				svg::path()
					.attribute(
						"d",
						"M70.71 212.13L141.421 212.13L70.71 282.841L70.71 212.13Z",
					)
					.attribute("fill", "#5E5CE6"),
			)
			.child(
				svg::path()
					.attribute("d", "M212.131 212.131H70.71V70.71L212.131 212.131Z")
					.attribute("fill", "#0A84FF"),
			)
			.child(
				svg::path()
					.attribute("d", "M70.7107 0V141.421L0 70.7107L70.7107 0Z")
					.attribute("fill", "#4DD0E1"),
			)
		)
			.into_node()
	}
}
