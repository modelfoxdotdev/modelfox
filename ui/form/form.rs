use html::{component, html, Props};

#[derive(Props)]
pub struct FormProps {
	#[optional]
	pub action: Option<String>,
	#[optional]
	pub autocomplete: Option<String>,
	#[optional]
	pub enc_type: Option<String>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub onsubmit: Option<String>,
	#[optional]
	pub post: Option<bool>,
}

#[component]
pub fn Form(props: FormProps) {
	let method = if props.post.unwrap_or(false) {
		Some("post".to_owned())
	} else {
		None
	};
	html! {
		<form
			action={props.action}
			autocomplete={props.autocomplete}
			class="form"
			enctype={props.enc_type}
			id={props.id}
			onsubmit={props.onsubmit}
			method={method}
		>
			{children}
		</form>
	}
}
