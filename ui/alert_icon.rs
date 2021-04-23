use super::alert::Level;
use html::{classes, component, html, Props};

#[derive(Props)]
pub struct AlertIconProps {
	pub alert: String,
	pub level: Level,
}

#[component]
pub fn AlertIcon(props: AlertIconProps) {
	let level_class = match props.level {
		Level::Info => "alert-icon-level-info",
		Level::Success => "alert-icon-level-success",
		Level::Warning => "alert-icon-level-warning",
		Level::Danger => "alert-icon-level-danger",
	};
	let alert_message_class = classes!("alert-icon-message", level_class);
	let alert_icon_class = classes!("alert-icon", level_class);
	html! {
		<div class="alert-icon-container">
			<div class={alert_message_class}>{props.alert}</div>
			<div class={alert_icon_class}>{children}</div>
		</div>
	}
}
