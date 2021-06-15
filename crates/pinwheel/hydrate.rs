use crate::{app::App, component::Component};
use wasm_bindgen::JsCast;
use web_sys as dom;

pub fn hydrate<T>(id: &str)
where
	T: Component + serde::Serialize + serde::de::DeserializeOwned,
{
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	let root = document.get_element_by_id(id).unwrap();
	let component = root
		.dyn_ref::<dom::HtmlElement>()
		.unwrap()
		.dataset()
		.get("component")
		.unwrap();
	let component: T = serde_json::from_str(&component).unwrap();
	App::new(root.into(), component.into_node()).forget();
}
