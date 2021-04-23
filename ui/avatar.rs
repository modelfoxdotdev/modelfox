use html::{component, html, Props};

#[derive(Props)]
pub struct AvatarProps {
	pub src: Option<String>,
}

#[component]
pub fn Avatar(props: AvatarProps) {
	html! {
		<div class="avatar">
			{if let Some(src) = props.src {
				html! {
					<div class="avatar-img">
						<img alt="avatar" src={src} />
					</div>
				}
			} else {
				html! {
					<div class="avatar-placeholder">
						<DefaultAvatar />
					</div>
				}
			}}
		</div>
	}
}

#[component]
pub fn DefaultAvatar() {
	html! {
		<svg height="100%" viewBox="0 0 100 100" width="100%">
			<desc>{"avatar"}</desc>
			<circle cx="50" cy="50" fill="var(--accent-color)" r="50"></circle>
			<circle cx="50" cy="40" fill="var(--surface-color)" r="16"></circle>
			<circle cx="50" cy="96" fill="var(--surface-color)" r="36"></circle>
		</svg>
	}
}
