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
		.class(self.class)
		.attribute("viewBox", "0 0 1000 1000")
		.attribute("fill", "none")
		.attribute("xmlns", "http://www.w3.org/2000/svg")
		.child(
			svg::path()
			.attribute("fill-rule", "evenodd")
			.attribute("clip-rule", "evenodd")
			.attribute("d", "M0 0L466.762 313.062L105.322 616.089L500 900.946L894.678 616.089L533.238 313.062L1000 0V639.128L500 1000L0 639.128V0ZM920 532.551L666.762 320.238L920 150.388V532.551ZM80 532.551V150.388L333.238 320.238L80 532.551Z")
			.attribute("fill","url(#paint0_linear_33_10)")
		)
		.child(
			svg::defs().child(
				linearGradient()
				.attribute("id", "paint0_linear_33_10")
				.attribute("x1", "500")
				.attribute("y1", "13.8241")
				.attribute("x2", "500")
				.attribute("y2", "1019.89")
				.attribute("gradientUnits", "userSpaceOnUse")
				.child(
					svg::stop()
					.attribute("stop-color", "#0A84FF")
				)
				.child(
					svg::stop()
					.attribute("offset", "1")
					.attribute("stop-color", "#4DD0E1")
				)
			)
		)
		.into_node()
	}
}
