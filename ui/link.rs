use html::{classes, component, html, Props};

#[derive(Props)]
pub struct LinkProps {
	pub href: String,
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub title: Option<String>,
}

#[component]
pub fn Link(props: LinkProps) {
	let class = classes!("link", props.class);
	html! {
		<a class={class} href={props.href} title={props.title}>
			{children}
		</a>
	}
}
