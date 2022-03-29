use pinwheel::prelude::*;

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
		.class(self.class)
		.attribute("width", "100%")
		.attribute("height", "100%")
		.attribute("viewBox", "0 0 900 962")
		.attribute("fill", "none")
		.attribute("xmlns","http://www.w3.org/2000/svg")
		.child(
			svg::path()
			.attribute("d", "M860 620L449.277 930L40 622M860 620V33L539.196 326L860 620ZM40 622V30L363.803 327L40 622Z").attribute("stroke", "url(#paint0_linear_2_4)")
			.attribute("stroke-width", "80")
			.attribute("stroke-linejoin", "bevel")
		)
		.child(
			svg::defs()
			.child(
				svg::linearGradient()
				.attribute("id", "paint0_linear_2_4")
				.attribute("x1", "450")
				.attribute("y1", "30")
				.attribute("x2", "449.75")
				.attribute("y2", "1069.5")
				.attribute("gradientUnits", "userSpaceOnUse")
				.child(
					svg::stop().attribute("stop-color", "#0A84FF")
				).child(
					svg::stop().attribute("offset", "1").attribute("stop-color", "#4DD0E1")
				)
			)
		)
		.into_node()
	}
}
