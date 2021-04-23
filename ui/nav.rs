use html::{classes, component, html, Props};

#[derive(Props)]
pub struct NavProps {
	#[optional]
	title: Option<String>,
}

#[component]
pub fn Nav(props: NavProps) {
	html! {
		<>
			<details class="nav-details">
				<summary>
					{props.title}
				</summary>
			</details>
			<nav class="nav">{children}</nav>
		</>
	}
}

#[derive(Props)]
pub struct NavItemProps {
	pub title: String,
	pub href: Option<String>,
	pub selected: Option<bool>,
}

#[component]
pub fn NavItem(props: NavItemProps) {
	let selected = props.selected.unwrap_or(false);
	let class = classes!(
		"nav-item",
		if selected {
			Some("nav-item-selected")
		} else {
			None
		},
		if props.href.is_some() {
			Some("nav-item-clickable")
		} else {
			None
		}
	);
	html! {
		<div class={class}>
			<a href={props.href}>{props.title}</a>
			{children}
		</div>
	}
}

#[derive(Props)]
pub struct NavSectionProps {
	pub title: String,
}

#[component]
pub fn NavSection(props: NavSectionProps) {
	html! {
		<div class="nav-section">
			<div class="nav-section-title">{props.title}</div>
			{children}
		</div>
	}
}
