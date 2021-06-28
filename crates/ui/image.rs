use pinwheel::prelude::*;

#[derive(builder, Default, new)]
#[new(default)]
pub struct Img {
	#[builder]
	pub alt: Option<String>,
	#[builder]
	pub src: Option<String>,
}

impl Component for Img {
	fn into_node(self) -> Node {
		details()
			.class("image-details")
			.child(
				summary().class("image-details-summary").child(
					img()
						.class("image-img")
						.attribute("alt", self.alt.clone())
						.attribute("src", self.src.clone()),
				),
			)
			.child(
				div().class("image-viewer").child(
					img()
						.class("image-viewer-img")
						.attribute("alt", self.alt)
						.attribute("src", self.src),
				),
			)
			.into_node()
	}
}
