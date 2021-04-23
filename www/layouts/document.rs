use html::{component, html, raw, Props};

#[derive(Props)]
pub struct DocumentProps {
	pub client_wasm_js_src: Option<String>,
}

#[component]
pub fn Document(props: DocumentProps) {
	html! {
		<>
			{raw!("<!doctype html>")}
			<html lang="en">
				<head>
					<meta charset="utf-8" />
					<meta content="width=device-width, initial-scale=1" name="viewport" />
					<link href="/favicon.png" rel="icon" type="image/png" />
					<title>{"Tangram"}</title>
					<link href="/styles.css" rel="stylesheet" />
					<meta
						content="All-In-One Machine Learning Toolkit Designed for Programmers"
						name="description"
					/>
				</head>
				<body>
					{children}
					{props.client_wasm_js_src.map(|client_wasm_js_src| html! {
						<script type="module">
							{raw!(format!(r#"import init from "{}"; init()"#, client_wasm_js_src))}
						</script>
					})}
				</body>
			</html>
		</>
	}
}
