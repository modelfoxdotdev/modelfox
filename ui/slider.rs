use pinwheel::{prelude::*, signal::Broadcaster};
use wasm_bindgen::JsCast;
use web_sys as dom;

#[derive(ComponentBuilder)]
pub struct Slider {
	pub min: f32,
	pub max: f32,
	pub step: f32,
	pub value: BoxSignal<f32>,
	#[optional]
	pub on_change: Option<Box<dyn Fn(f32)>>,
	#[optional]
	pub tooltip_number_formatter: Option<Box<dyn Fn(f32) -> String>>,
}

impl Component for Slider {
	fn into_node(self) -> Node {
		let Slider {
			min,
			max,
			on_change,
			tooltip_number_formatter,
			value,
			..
		} = self;
		let value = Broadcaster::new(value);
		let percent = value
			.signal()
			.map(move |value| ((value - min) / (max - min)) * 100.0);
		let percent = Broadcaster::new(percent);
		let tooltip_value = value
			.signal()
			.map(move |value| match &tooltip_number_formatter {
				Some(tooltip_number_formatter) => tooltip_number_formatter(value),
				None => value.to_string(),
			});
		let oninput = move |event: dom::InputEvent| {
			let current_target = event
				.current_target()
				.unwrap()
				.dyn_into::<dom::HtmlInputElement>()
				.unwrap();
			let value = current_target.value();
			let value: f32 = value.parse::<f32>().unwrap();
			if let Some(on_change) = on_change.as_ref() {
				on_change(value);
			}
		};
		div()
			.class("slider-wrapper")
			.child(
				input()
					.class("slider-input")
					.attribute("type", "range")
					.min(self.min.to_string())
					.max(self.max.to_string())
					.value_signal(value.signal().map(|value| value.to_string()))
					.step("1")
					.autocomplete("off")
					.oninput(oninput),
			)
			.child(
				div()
					.class("slider-inner-wrapper")
					.child(div().class("slider-progress").style_signal(
						style::WIDTH,
						percent.signal().map(|percent| format!("{}%", percent)),
					))
					.child(
						div()
							.class("slider-tooltip")
							.style_signal(
								style::MARGIN_LEFT,
								percent.signal().map(|percent| format!("{}%", percent)),
							)
							.child_signal(tooltip_value),
					),
			)
			.into_node()
	}
}
