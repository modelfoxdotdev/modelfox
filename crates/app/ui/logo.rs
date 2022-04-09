use pinwheel::prelude::{svg::linearGradient, *};

#[derive(builder, Default, new)]
#[new(default)]
pub struct Logo {
	#[builder]
	pub class: Option<String>,
	#[builder]
	pub color: Option<String>,
}

impl Component for Logo {
	fn into_node(self) -> Node {
		svg()
		.attribute("width", "1000")
		.attribute("height","1000")
		.attribute("viewBox", "0 0 1000 1000")
		.attribute("fill", "none")
		.attribute("xmlns", "http://www.w3.org/2000/svg")
		.child(
			svg::path()
			.attribute("d", "M960 620L500 950L40 620M960 620V80L600 320L960 620ZM40 620V80L400 320L40 620Z")
			.attribute("stroke", "url(#paint0_linear_33_10)")
			.attribute("stroke-width","80")
		)
		.child(
			svg::defs()
			.child(
				linearGradient()
				.attribute("id", "paint0_linear_33_10")
				.attribute("x1", "500")
				.attribute("y1", "19")
				.attribute("x2", "500")
				.attribute("y2", "1019")
				.attribute("gradientUnits", "userSpaceOnUse")
				.child(
					svg::stop()
					.attribute("stop-color","#0A84FF")
				)
				.child(
					svg::stop()
					.attribute("offset", "1")
					.attribute("stop-color", "#4DD0E1"))
				)
			)
		.into_node()
	}
}
