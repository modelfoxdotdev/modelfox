use crate::{SelectField, SelectFieldOption};
use html::{component, html, raw, style, Props};
use once_cell::sync::Lazy;
use std::borrow::Cow;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

#[derive(Props)]
pub struct CodeProps {
	pub code: String,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub hide_line_numbers: Option<bool>,
}

#[derive(PartialEq)]
pub enum Language {
	Elixir,
	Go,
	Javascript,
	Python,
	Ruby,
	Rust,
}

#[component]
pub fn Code(props: CodeProps) {
	let code = props.code;
	let hide_line_numbers = props.hide_line_numbers.unwrap_or(false);
	let line_numbers = if !hide_line_numbers {
		Some(html! {
			<LineNumbers count={count_lines(&code)} />
		})
	} else {
		None
	};
	html! {
		<div id={props.id} class="code">
			{line_numbers}
			<div class="code-inner">
				{raw!(code)}
			</div>
		</div>
	}
}

pub fn update_code(id: &str, code: String) {
	let document = window().unwrap().document().unwrap();
	let value_element = document
		.query_selector(&format!(".code#{} .code-inner", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	value_element.set_inner_html(&code);
}

#[derive(Props)]
pub struct CodeSelectProps {
	pub id: String,
	pub code_for_language: CodeForLanguage,
	#[optional]
	pub hide_line_numbers: Option<bool>,
	#[optional]
	pub language: Option<Language>,
}

pub struct CodeForLanguage {
	pub elixir: Cow<'static, str>,
	pub go: Cow<'static, str>,
	pub javascript: Cow<'static, str>,
	pub python: Cow<'static, str>,
	pub ruby: Cow<'static, str>,
	pub rust: Cow<'static, str>,
}

#[component]
pub fn CodeSelect(props: CodeSelectProps) {
	let code_elixir = props.code_for_language.elixir;
	let code_go = props.code_for_language.go;
	let code_javascript = props.code_for_language.javascript;
	let code_python = props.code_for_language.python;
	let code_ruby = props.code_for_language.ruby;
	let code_rust = props.code_for_language.rust;
	let options = vec![
		SelectFieldOption {
			text: "elixir".to_owned(),
			value: "elixir".to_owned(),
		},
		SelectFieldOption {
			text: "go".to_owned(),
			value: "go".to_owned(),
		},
		SelectFieldOption {
			text: "javascript".to_owned(),
			value: "javascript".to_owned(),
		},
		SelectFieldOption {
			text: "python".to_owned(),
			value: "python".to_owned(),
		},
		SelectFieldOption {
			text: "ruby".to_owned(),
			value: "ruby".to_owned(),
		},
		SelectFieldOption {
			text: "rust".to_owned(),
			value: "rust".to_owned(),
		},
	];
	let selected_style = style! {
		"display" => "block",
	};
	let value = props.language.unwrap_or(Language::Javascript);
	let elixir_style = if value == Language::Elixir {
		Some(selected_style.to_owned())
	} else {
		None
	};
	let go_style = if value == Language::Go {
		Some(selected_style.to_owned())
	} else {
		None
	};
	let javascript_style = if value == Language::Javascript {
		Some(selected_style.to_owned())
	} else {
		None
	};
	let python_style = if value == Language::Python {
		Some(selected_style.to_owned())
	} else {
		None
	};
	let ruby_style = if value == Language::Ruby {
		Some(selected_style.to_owned())
	} else {
		None
	};
	let rust_style = if value == Language::Rust {
		Some(selected_style)
	} else {
		None
	};
	let value = match value {
		Language::Elixir => "elixir".to_owned(),
		Language::Go => "go".to_owned(),
		Language::Javascript => "javascript".to_owned(),
		Language::Python => "python".to_owned(),
		Language::Ruby => "ruby".to_owned(),
		Language::Rust => "rust".to_owned(),
	};
	html! {
		<div class="code-select-grid">
			<SelectField
				id?="code_select"
				options?={Some(options)}
				value?={Some(value)}
			/>
			<div class="code-select-body">
				<div
					style={elixir_style}
					class="code-select-code-wrapper"
					data-lang="elixir"
				>
					<Code
						code={code_elixir.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
				<div
					style={go_style}
					class="code-select-code-wrapper"
					data-lang="go"
				>
					<Code
						code={code_go.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
				<div
					style={javascript_style}
					class="code-select-code-wrapper"
					data-lang="javascript"
				>
					<Code
						code={code_javascript.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
				<div
					style={python_style}
					class="code-select-code-wrapper"
					data-lang="python"
				>
					<Code
						code={code_python.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
				<div
					style={ruby_style}
					class="code-select-code-wrapper"
					data-lang="ruby"
				>
					<Code
						code={code_ruby.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
				<div
					style={rust_style}
					class="code-select-code-wrapper"
					data-lang="rust"
				>
					<Code
						code={code_rust.into_owned()}
						hide_line_numbers?={props.hide_line_numbers}
					/>
				</div>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct LineNumbersProps {
	pub count: usize,
}

#[component]
pub fn LineNumbers(props: LineNumbersProps) {
	html! {
		<div class="code-line-numbers-wrapper">
			{(0..props.count).map(|index| html! {
				<div class="code-line-numbers">
					{(index + 1).to_string()}
				</div>
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[component]
pub fn InlineCode() {
	html! {
		<span class="inline-code-wrapper">
			{children}
		</span>
	}
}

fn count_lines(text: &str) -> usize {
	let n_lines = text.split('\n').count();
	if text.ends_with('\n') {
		n_lines - 1
	} else {
		n_lines
	}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn highlight_code_for_language(code_for_language: CodeForLanguage) -> CodeForLanguage {
	CodeForLanguage {
		elixir: highlight(&code_for_language.elixir, Language::Elixir).into(),
		go: highlight(&code_for_language.go, Language::Go).into(),
		javascript: highlight(&code_for_language.javascript, Language::Javascript).into(),
		python: highlight(&code_for_language.python, Language::Python).into(),
		ruby: highlight(&code_for_language.ruby, Language::Ruby).into(),
		rust: highlight(&code_for_language.rust, Language::Rust).into(),
	}
}

macro_rules! highlight_configuration {
	($i:ident, $c:ident) => {
		static $i: Lazy<tree_sitter_highlight::HighlightConfiguration> = Lazy::new(|| {
			let language = $c::language();
			let query = $c::HIGHLIGHT_QUERY;
			let mut config =
				tree_sitter_highlight::HighlightConfiguration::new(language, query, "", "")
					.unwrap();
			config.configure(&NAMES);
			config
		});
	};
}

#[cfg(not(target_arch = "wasm32"))]
pub fn highlight(code: &str, language: Language) -> String {
	static NAMES: Lazy<Vec<String>> = Lazy::new(|| {
		[
			"comment",
			"function",
			"keyword",
			"operator",
			"punctuation",
			"string",
			"type",
			"variable",
		]
		.iter()
		.cloned()
		.map(String::from)
		.collect()
	});
	highlight_configuration!(ELIXIR, tree_sitter_python);
	highlight_configuration!(GO, tree_sitter_go);
	highlight_configuration!(JAVASCRIPT, tree_sitter_javascript);
	highlight_configuration!(PYTHON, tree_sitter_python);
	highlight_configuration!(RUBY, tree_sitter_python);
	highlight_configuration!(RUST, tree_sitter_rust);
	let highlight_configuration = match language {
		Language::Elixir => &ELIXIR,
		Language::Go => &GO,
		Language::Javascript => &JAVASCRIPT,
		Language::Python => &PYTHON,
		Language::Ruby => &RUBY,
		Language::Rust => &RUST,
	};
	let mut highlighter = tree_sitter_highlight::Highlighter::new();
	let highlights = highlighter
		.highlight(highlight_configuration, code.as_bytes(), None, |_| None)
		.unwrap();
	let mut highlighted_code = String::new();
	for event in highlights {
		match event.unwrap() {
			tree_sitter_highlight::HighlightEvent::Source { start, end } => {
				highlighted_code.push_str(&code[start..end]);
			}
			tree_sitter_highlight::HighlightEvent::HighlightStart(highlight) => {
				highlighted_code.push_str(&format!(
					"<span class=\"{}\">",
					NAMES.get(highlight.0).unwrap()
				));
			}
			tree_sitter_highlight::HighlightEvent::HighlightEnd => {
				highlighted_code.push_str("</span>");
			}
		}
	}
	highlighted_code
}

pub fn boot_code_select() {
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		let document = window().unwrap().document().unwrap();
		let select_node = event.current_target().unwrap().dyn_into::<Node>().unwrap();
		let select_element = select_node.dyn_ref::<HtmlSelectElement>().unwrap();
		let select_element_language = select_element.value();
		let code_select_elements = document.query_selector_all("#code_select").unwrap();
		// update all code select elements
		for index in 0..code_select_elements.length() {
			let code_select_element = code_select_elements
				.get(index)
				.unwrap()
				.dyn_into::<HtmlSelectElement>()
				.unwrap();
			code_select_element.set_value(&select_element_language);
		}
		let code_elements_not_selected_lang = document
			.query_selector_all(&format!(
				".code-select-code-wrapper:not([data-lang={}]",
				select_element_language
			))
			.unwrap();
		for index in 0..code_elements_not_selected_lang.length() {
			let code_element = code_elements_not_selected_lang
				.get(index)
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			code_element
				.style()
				.set_property("display", "none")
				.unwrap();
		}
		let code_elements_selected_lang = document
			.query_selector_all(&format!(
				".code-select-code-wrapper[data-lang={}]",
				select_element_language
			))
			.unwrap();
		for index in 0..code_elements_selected_lang.length() {
			let code_element_for_lang = code_elements_selected_lang
				.get(index)
				.unwrap()
				.dyn_into::<HtmlElement>()
				.unwrap();
			code_element_for_lang
				.style()
				.set_property("display", "block")
				.unwrap();
		}
	}));
	let document = window().unwrap().document().unwrap();
	let code_select_elements = document.query_selector_all("#code_select").unwrap();
	for index in 0..code_select_elements.length() {
		let code_select_element = code_select_elements
			.get(index)
			.unwrap()
			.dyn_into::<HtmlElement>()
			.unwrap();
		code_select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
