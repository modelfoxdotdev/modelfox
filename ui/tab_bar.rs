use html::{classes, component, html, Props};

#[component]
pub fn TabBar() {
	html! {
	  <div class="tab-bar">
			{children}
	  </div>
	}
}

#[derive(Props)]
pub struct TabProps {
	pub selected: bool,
	#[optional]
	pub disabled: Option<bool>,
}

#[component]
pub fn Tab(props: TabProps) {
	let selected = if props.selected {
		Some("tab-bar-tab-selected")
	} else {
		None
	};
	let disabled = if props.disabled.unwrap_or(false) {
		Some("tab-bar-tab-disabled")
	} else {
		None
	};
	let class = classes!("tab-bar-tab", selected, disabled);
	html! {
		<div class={class}>
			{children}
		</div>
	}
}

#[derive(Props)]
pub struct TabLinkProps {
	pub href: String,
	pub selected: bool,
	#[optional]
	pub disabled: Option<bool>,
}

#[component]
pub fn TabLink(props: TabLinkProps) {
	html! {
		<Tab selected={props.selected} disabled?={props.disabled}>
			<a class="tab-bar-tab-link" href={props.href}>
				{children}
			</a>
		</Tab>
	}
}
