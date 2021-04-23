use html::{component, html, style, Props};
use num::ToPrimitive;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

#[derive(Props)]
pub struct SliderProps {
	pub id: Option<String>,
	pub max: f32,
	pub min: f32,
	pub value: usize,
}

#[component]
pub fn Slider(props: SliderProps) {
	let percent = ((props.value.to_f32().unwrap() - props.min) / (props.max - props.min)) * 100.0;
	let progress_style = style! {
	  "width" =>  format!("{}%", percent),
	};
	let tooltip_style = style! {
	  "margin-left" =>  format!("{}%", percent),
	};
	html! {
		<div class="slider-wrapper">
			<input
				autocomplete="off"
				id={props.id}
				class="slider-input"
				max={props.max.to_string()}
				min={props.min.to_string()}
				type="range"
			/>
			<div class="slider-progress" style={progress_style}></div>
			<div class="slider-tooltip" style={tooltip_style}>
				{props.value.to_string()}
			</div>
		</div>
	}
}

pub fn boot_slider(id: &str, slider_formatter: Option<Box<dyn Fn(usize) -> String>>) {
	let document = window().unwrap().document().unwrap();
	let slider = document
		.get_element_by_id(&id)
		.unwrap()
		.dyn_into::<HtmlInputElement>()
		.unwrap();
	let number_formatter = slider_formatter.unwrap_or_else(|| Box::new(|value| value.to_string()));
	let value = slider.value().parse().unwrap();
	set_slider_tooltip_value(id.to_owned(), number_formatter(value));
	let id = id.to_owned();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		if let Some(current_target) = event.current_target() {
			let current_target = &current_target.dyn_into::<HtmlInputElement>().unwrap();
			let value = current_target.value();
			let min: f32 = current_target.min().parse().unwrap();
			let max: f32 = current_target.max().parse().unwrap();
			let percent = ((value.parse::<f32>().unwrap() - min) / (max - min)) * 100.0;
			let parent_element = current_target
				.parent_element()
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			let slider_progress = parent_element
				.get_elements_by_class_name("slider-progress")
				.item(0)
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			slider_progress
				.style()
				.set_property("width", &format!("{}%", &percent))
				.unwrap();
			let slider_tooltip = parent_element
				.get_elements_by_class_name("slider-tooltip")
				.item(0)
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			slider_tooltip
				.style()
				.set_property("margin-left", &format!("{}%", &percent))
				.unwrap();
			set_slider_tooltip_value(
				id.clone(),
				number_formatter(current_target.value().parse::<usize>().unwrap()),
			);
		}
	}));
	if let Some(slider) = slider.dyn_ref::<HtmlInputElement>() {
		slider
			.add_event_listener_with_callback("input", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}

pub fn set_slider_tooltip_value(id: String, value: String) {
	let document = window().unwrap().document().unwrap();
	let slider_tooltip = document
		.query_selector(&format!("#{}~.slider-tooltip", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	slider_tooltip.set_inner_html(&value);
}
