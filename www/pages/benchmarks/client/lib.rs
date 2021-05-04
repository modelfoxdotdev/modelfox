use tangram_charts::{bar_chart::BarChart, components::hydrate_chart};
use wasm_bindgen::{self, prelude::*, JsCast};
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate_charts();
	init_cpu_select();
	init_dataset_select();
}

fn hydrate_charts() {
	let cpus = ["m1", "ryzen"];
	let datasets = ["allstate", "flights", "higgs"];
	let metrics = ["duration", "memory", "metric"];
	let document = window().unwrap().document().unwrap();
	for cpu in cpus.iter() {
		for dataset in datasets.iter() {
			for metric in metrics.iter() {
				let id = format!("{}_{}_{}_chart", cpu, dataset, metric);
				if document.get_element_by_id(&id).is_some() {
					hydrate_chart::<BarChart>(&id);
				}
			}
		}
	}
}

fn set_dataset_hidden_states(selected_dataset: &str) {
	let document = window().unwrap().document().unwrap();
	let datasets = ["allstate", "higgs", "flights"];
	let cpu_selected_option = document
		.get_element_by_id(&"cpu-select")
		.unwrap()
		.dyn_ref::<HtmlSelectElement>()
		.unwrap()
		.value();
	for dataset in datasets.iter() {
		let dataset_selected_section_id = format!("{}-{}", cpu_selected_option, dataset);
		let dataset_selected_section = document
			.get_element_by_id(&dataset_selected_section_id)
			.unwrap()
			.dyn_into::<HtmlElement>()
			.unwrap();
		if dataset == &selected_dataset {
			dataset_selected_section.set_hidden(false);
		} else {
			dataset_selected_section.set_hidden(true);
		}
	}
}

fn init_dataset_select() {
	let document = window().unwrap().document().unwrap();
	let dataset_select_element = document.get_element_by_id(&"dataset-select").unwrap();

	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		if let Some(event) = event.current_target() {
			let dataset_selected_option = event.dyn_ref::<HtmlSelectElement>().unwrap().value();
			set_dataset_hidden_states(dataset_selected_option.as_str());
		}
	}));

	if let Some(select_element) = dataset_select_element.dyn_ref::<HtmlSelectElement>() {
		set_dataset_hidden_states(select_element.value().as_str());
		select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}

fn init_cpu_select() {
	let document = window().unwrap().document().unwrap();
	let cpu_select_element = document.get_element_by_id(&"cpu-select").unwrap();

	fn set_cpu_hidden_states(cpu_selected_option: &str) {
		let document = window().unwrap().document().unwrap();
		let cpus = ["m1", "ryzen"];
		for cpu in cpus.iter() {
			let cpu_selected_section = document
				.get_element_by_id(&cpu)
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			if cpu == &cpu_selected_option {
				cpu_selected_section.set_hidden(false);
			} else {
				cpu_selected_section.set_hidden(true);
			}
		}
	}

	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		if let Some(event) = event.current_target() {
			let cpu_selected_option = event.dyn_ref::<HtmlSelectElement>().unwrap().value();
			set_cpu_hidden_states(cpu_selected_option.as_str());
			let dataset_select_element = document
				.get_element_by_id(&"dataset-select")
				.unwrap()
				.dyn_into::<HtmlSelectElement>()
				.unwrap();
			let dataset_selected_option = dataset_select_element.value();
			set_dataset_hidden_states(dataset_selected_option.as_str());
		}
	}));

	if let Some(select_element) = cpu_select_element.dyn_ref::<HtmlSelectElement>() {
		set_cpu_hidden_states(select_element.value().as_str());
		select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
