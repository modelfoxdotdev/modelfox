use crate::alert::Level;
use html::{classes, component, html, Props};

#[derive(Props)]
pub struct CalloutProps {
	pub level: Level,
	pub title: Option<String>,
}

#[component]
pub fn Callout(props: CalloutProps) {
	let level_class = match props.level {
		Level::Danger => "callout-wrapper-danger",
		Level::Info => "callout-wrapper-info",
		Level::Warning => "callout-wrapper-warning",
		Level::Success => "callout-wrapper-success",
	};
	let class = classes!("callout-wrapper", level_class);
	html! {
		<div class={class}>
			{props.title.map(|title| html! {
				<div class="callout-title">{title}</div>
			})}
			<div class="callout-inner">{children}</div>
		</div>
	}
}
