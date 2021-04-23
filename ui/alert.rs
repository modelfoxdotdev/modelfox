use html::{classes, component, html, Props};

#[derive(Props)]
pub struct AlertProps {
	pub level: Level,
	#[optional]
	pub title: Option<String>,
}

pub enum Level {
	Info,
	Success,
	Warning,
	Danger,
}

#[component]
pub fn Alert(props: AlertProps) {
	let level_class = match props.level {
		Level::Info => "alert-level-info",
		Level::Success => "alert-level-success",
		Level::Warning => "alert-level-warning",
		Level::Danger => "alert-level-danger",
	};
	let class = classes!("alert-wrapper", level_class);
	html! {
		<div class={class}>
			{props.title.map(|title| {
				html! {
					<div class="alert-title">
						{title}
					</div>
				}
			})}
			{children}
		</div>
	}
}
