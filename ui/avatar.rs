use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Avatar {
	pub src: Option<String>,
}

impl Component for Avatar {
	fn into_node(self) -> Node {
		div()
			.class("avatar")
			.child({
				if let Some(src) = self.src {
					div()
						.class("avatar-img")
						.child(img().attribute("alt", "avatar").attribute("src", src))
				} else {
					div()
						.class("avatar-placeholder")
						.child(DefaultAvatar::new())
				}
			})
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct DefaultAvatar;

impl Component for DefaultAvatar {
	fn into_node(self) -> Node {
		svg()
			.attribute("height", "100%")
			.attribute("viewBox", "0 0 100 100")
			.attribute("width", "100%")
			.child(svg::desc().child("avatar"))
			.child(
				svg::circle()
					.attribute("cx", "50")
					.attribute("cy", "50")
					.attribute("fill", "var(--accent-color)")
					.attribute("r", "50"),
			)
			.child(
				svg::circle()
					.attribute("cx", "50")
					.attribute("cy", "40")
					.attribute("fill", "var(--surface-color)")
					.attribute("r", "16"),
			)
			.child(
				svg::circle()
					.attribute("cx", "50")
					.attribute("cy", "96")
					.attribute("fill", "var(--surface-color)")
					.attribute("r", "36"),
			)
			.into_node()
	}
}
