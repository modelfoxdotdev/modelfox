use html::{component, html};

#[component]
pub fn Card() {
	html! {
		<div class="card">
			{children}
		</div>
	}
}
