use html::{classes, component, html, Props};

#[component]
pub fn S1() {
	html! {
		<div class="s1">
			{children}
		</div>
	}
}

#[component]
pub fn S2() {
	html! {
		<div class="s2">
			{children}
		</div>
	}
}

#[component]
pub fn SpaceBetween() {
	html! {
		<div class="space-between">
			{children}
		</div>
	}
}

#[derive(Props)]
pub struct H1Props {
	#[optional]
	pub center: Option<bool>,
}

#[component]
pub fn H1(props: H1Props) {
	let center = if props.center.unwrap_or(false) {
		Some("center")
	} else {
		None
	};
	let class = classes!(Some("h1"), center);
	html! {
		<h1 class={class}>
			{children}
		</h1>
	}
}

#[derive(Props)]
pub struct H2Props {
	#[optional]
	pub center: Option<bool>,
}

#[component]
pub fn H2(props: H2Props) {
	let center = if props.center.unwrap_or(false) {
		Some("center")
	} else {
		None
	};
	let class = classes!(Some("h2"), center);
	html! {
		<h2 class={class}>
			{children}
		</h2>
	}
}

#[component]
pub fn P() {
	html! {
		<p class="p">
			{children}
		</p>
	}
}

#[component]
pub fn List() {
	html! {
		<ul class="list">
			{children}
		</ul>
	}
}

#[component]
pub fn OrderedList() {
	html! {
		<ol class="ordered-list">
			{children}
		</ol>
	}
}

#[component]
pub fn ListItem() {
	html! {
		<li>
			{children}
		</li>
	}
}
