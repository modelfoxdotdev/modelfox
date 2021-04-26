use crate as html;
use html::{component, html, Props};

#[derive(Props)]
pub struct TestProps {
	greeting: String,
}

#[component]
fn Test(props: TestProps) {
	html! {
		<div>{props.greeting}</div>
	}
}

#[test]
fn test() {
	let html = html! { <Test greeting="Hello, World!" /> };
	assert_eq!(html.render_to_string(), "<div>Hello, World!</div>");
}

#[test]
fn test_rest() {
	let props = TestProps {
		greeting: "Hello, World!".to_owned(),
	};
	let html = html! { <Test {props} /> };
	assert_eq!(html.render_to_string(), "<div>Hello, World!</div>");
}
