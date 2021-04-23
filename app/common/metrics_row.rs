use html::{component, html};

#[component]
pub fn MetricsRow() {
	html! {
		<div class="metrics-row">
		  {children}
	  </div>
	}
}
