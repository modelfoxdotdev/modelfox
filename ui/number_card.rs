use crate as ui;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct NumberCard {
	pub title: String,
	pub value: BoxSignal<String>,
}

impl Component for NumberCard {
	fn into_node(self) -> Node {
		ui::Card::new()
			.child(
				div()
					.class("number-wrapper")
					.child(
						div()
							.class("number-value")
							.child_signal(self.value.signal_cloned()),
					)
					.child(div().class("number-title").child(self.title)),
			)
			.into_node()
	}
}
