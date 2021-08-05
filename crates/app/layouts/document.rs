use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Document {
	#[builder]
	pub client: Option<&'static str>,
	pub children: Vec<Node>,
}

impl Component for Document {
	fn into_node(self) -> Node {
		let head = head()
			.child(meta().attribute("charset", "utf-8"))
			.child(
				meta()
					.attribute("content", "width=device-width, initial-scale=1")
					.attribute("name", "viewport"),
			)
			.child(
				link()
					.attribute("href", "/favicon.png")
					.attribute("rel", "icon")
					.attribute("type", "image/png"),
			)
			.child(title().child("Tangram"))
			.child(
				link()
					.attribute("href", "/styles.css")
					.attribute("rel", "stylesheet"),
			)
			.child(
				meta()
					.attribute("content", "Tangram makes machine learning easy.")
					.attribute("name", "description"),
			);
		let timezone_script = script().child(
			"document.cookie = `tangram_timezone=${Intl.DateTimeFormat().resolvedOptions().timeZone};max-age=31536000;path=/;samesite=lax`");
		let client_script = self.client.map(|client| {
			let paths = sunfish::client_paths(client);
			script().attribute("type", "module").inner_html(format!(
				r#"import init from "{path_js}"; init("{path_wasm}")"#,
				path_js = paths.path_js,
				path_wasm = paths.path_wasm,
			))
		});
		let body = body()
			.child(self.children)
			.child(timezone_script)
			.child(client_script);
		html::html().child(head).child(body).into_node()
	}
}
