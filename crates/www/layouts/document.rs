use pinwheel::prelude::*;

#[derive(builder, Default, children, new)]
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
			.child(title().child("ModelFox"))
			.child(
				link()
					.attribute("href", "/styles.css")
					.attribute("rel", "stylesheet"),
			)
			.child(
				meta()
					.attribute("content", "ModelFox makes it easy for programmers to train, deploy, and monitor machine learning models.")
					.attribute("name", "description"),
			)
			.child(
				script().attribute("src", "https://cdn.usefathom.com/script.js")
				.attribute("data-site", "RDWWWHLA").attribute("defer", true)
			)
			.child(
				script()
					.attribute("async", true)
					.attribute("defer", true)
					.attribute("src", "https://buttons.github.io/buttons.js"),
			)
			.child(
				script()
					.attribute("defer", true)
					.attribute("data-domain","modelfox.dev")
					.attribute("src","https://plausible.io/js/plausible.js")
			);
		let client_script = self.client.map(|client| {
			let paths = sunfish::client_paths(client);
			script().attribute("type", "module").inner_html(format!(
				r#"import init from "{path_js}"; init("{path_wasm}")"#,
				path_js = paths.path_js,
				path_wasm = paths.path_wasm,
			))
		});
		let body = body().child(self.children).child(client_script);
		html::html()
			.attribute("lang", "en")
			.child(head)
			.child(body)
			.into_node()
	}
}
