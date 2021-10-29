use crate as ui;
use pinwheel::prelude::*;
use std::borrow::Cow;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

#[derive(builder, Default, new)]
#[new(default)]
pub struct Code {
	#[builder]
	pub code: Option<Cow<'static, str>>,
	#[builder]
	pub language: Option<Language>,
	#[builder]
	pub line_numbers: Option<bool>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
	Elixir,
	Go,
	Javascript,
	PHP,
	Python,
	Ruby,
	Rust,
}

impl Component for Code {
	fn into_node(self) -> Node {
		let mut code = self.code.unwrap_or(Cow::Borrowed(""));
		if let Some(language) = self.language {
			code = highlight(code.as_ref(), language).into();
		}
		let line_numbers = self.line_numbers.unwrap_or(false);
		let line_numbers = if line_numbers {
			Some(LineNumbers {
				count: count_lines(&code),
			})
		} else {
			None
		};
		div()
			.class("code")
			.child(line_numbers)
			.child(div().class("code-inner").inner_html(code))
			.into_node()
	}
}

#[derive(builder, new)]
pub struct CodeSelect {
	pub code_for_language: CodeForLanguage,
	#[builder]
	#[new(default)]
	pub line_numbers: Option<bool>,
	#[builder]
	#[new(default)]
	pub language: Option<Language>,
}

pub struct CodeForLanguage {
	pub elixir: Cow<'static, str>,
	pub go: Cow<'static, str>,
	pub javascript: Cow<'static, str>,
	pub php: Cow<'static, str>,
	pub python: Cow<'static, str>,
	pub ruby: Cow<'static, str>,
	pub rust: Cow<'static, str>,
}

impl Component for CodeSelect {
	fn into_node(self) -> Node {
		let code_elixir = self.code_for_language.elixir;
		let code_go = self.code_for_language.go;
		let code_javascript = self.code_for_language.javascript;
		let code_php = self.code_for_language.php;
		let code_python = self.code_for_language.python;
		let code_ruby = self.code_for_language.ruby;
		let code_rust = self.code_for_language.rust;
		let options = vec![
			ui::SelectFieldOption {
				text: "elixir".to_owned(),
				value: "elixir".to_owned(),
			},
			ui::SelectFieldOption {
				text: "go".to_owned(),
				value: "go".to_owned(),
			},
			ui::SelectFieldOption {
				text: "javascript".to_owned(),
				value: "javascript".to_owned(),
			},
			ui::SelectFieldOption {
				text: "php".to_owned(),
				value: "php".to_owned(),
			},
			ui::SelectFieldOption {
				text: "python".to_owned(),
				value: "python".to_owned(),
			},
			ui::SelectFieldOption {
				text: "ruby".to_owned(),
				value: "ruby".to_owned(),
			},
			ui::SelectFieldOption {
				text: "rust".to_owned(),
				value: "rust".to_owned(),
			},
		];
		let language = self.language.unwrap_or(Language::Javascript);
		let language_str = match language {
			Language::Elixir => "elixir".to_owned(),
			Language::Go => "go".to_owned(),
			Language::Javascript => "javascript".to_owned(),
			Language::PHP => "php".to_owned(),
			Language::Python => "python".to_owned(),
			Language::Ruby => "ruby".to_owned(),
			Language::Rust => "rust".to_owned(),
		};
		div()
			.class("code-select-grid")
			.child(
				ui::SelectField::new()
					.class("code-select".to_owned())
					.options(options)
					.value(language_str),
			)
			.child(
				div()
					.class("code-select-body")
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Elixir {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "elixir")
							.child(
								Code::new()
									.code(code_elixir)
									.line_numbers(self.line_numbers),
							),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Go {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "go")
							.child(Code::new().code(code_go).line_numbers(self.line_numbers)),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Javascript {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "javascript")
							.child(
								Code::new()
									.code(code_javascript)
									.line_numbers(self.line_numbers),
							),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::PHP {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "php")
							.child(Code::new().code(code_php).line_numbers(self.line_numbers)),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Python {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "python")
							.child(
								Code::new()
									.code(code_python)
									.line_numbers(self.line_numbers),
							),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Ruby {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "ruby")
							.child(Code::new().code(code_ruby).line_numbers(self.line_numbers)),
					)
					.child(
						div()
							.style(
								style::DISPLAY,
								if language == Language::Rust {
									Some("block")
								} else {
									None
								},
							)
							.class("code-select-code-wrapper")
							.attribute("data-lang", "rust")
							.child(Code::new().code(code_rust).line_numbers(self.line_numbers)),
					),
			)
			.into_node()
	}
}

pub struct LineNumbers {
	pub count: usize,
}

impl Component for LineNumbers {
	fn into_node(self) -> Node {
		div()
			.class("code-line-numbers-wrapper")
			.children((0..self.count).map(|index| {
				div()
					.class("code-line-numbers")
					.child((index + 1).to_string())
			}))
			.into_node()
	}
}

pub struct InlineCode {
	pub code: Cow<'static, str>,
}

impl InlineCode {
	pub fn new(code: impl Into<Cow<'static, str>>) -> InlineCode {
		InlineCode { code: code.into() }
	}
}

impl Component for InlineCode {
	fn into_node(self) -> Node {
		span().class("inline-code").child(self.code).into_node()
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
		php: highlight(&code_for_language.php, Language::PHP).into(),
		python: highlight(&code_for_language.python, Language::Python).into(),
		ruby: highlight(&code_for_language.ruby, Language::Ruby).into(),
		rust: highlight(&code_for_language.rust, Language::Rust).into(),
	}
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! highlight_configuration {
	($i:ident, $c:ident) => {
		static $i: once_cell::sync::Lazy<tree_sitter_highlight::HighlightConfiguration> =
			once_cell::sync::Lazy::new(|| {
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

#[cfg(target_arch = "wasm32")]
pub fn highlight(_code: &str, _language: Language) -> String {
	unimplemented!()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn highlight(code: &str, language: Language) -> String {
	static NAMES: once_cell::sync::Lazy<Vec<String>> = once_cell::sync::Lazy::new(|| {
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
	highlight_configuration!(ELIXIR, tree_sitter_javascript);
	highlight_configuration!(GO, tree_sitter_javascript);
	highlight_configuration!(JAVASCRIPT, tree_sitter_javascript);
	highlight_configuration!(PHP, tree_sitter_javascript);
	highlight_configuration!(PYTHON, tree_sitter_javascript);
	highlight_configuration!(RUBY, tree_sitter_javascript);
	highlight_configuration!(RUST, tree_sitter_javascript);
	let highlight_configuration = match language {
		Language::Elixir => &ELIXIR,
		Language::Go => &GO,
		Language::Javascript => &JAVASCRIPT,
		Language::PHP => &PHP,
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
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: dom::Event| {
		let document = dom::window().unwrap().document().unwrap();
		let select_node = event
			.current_target()
			.unwrap()
			.dyn_into::<dom::Node>()
			.unwrap();
		let select_element = select_node.dyn_ref::<dom::HtmlSelectElement>().unwrap();
		let select_element_language = select_element.value();
		let code_select_elements = document.query_selector_all(".code-select").unwrap();
		for index in 0..code_select_elements.length() {
			let code_select_element = code_select_elements
				.get(index)
				.unwrap()
				.dyn_into::<dom::HtmlSelectElement>()
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
				.dyn_into::<dom::HtmlElement>()
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
				.dyn_into::<dom::HtmlElement>()
				.unwrap();
			code_element_for_lang
				.style()
				.set_property("display", "block")
				.unwrap();
		}
	}));
	let document = dom::window().unwrap().document().unwrap();
	let code_select_elements = document.query_selector_all(".code-select").unwrap();
	for index in 0..code_select_elements.length() {
		let code_select_element = code_select_elements
			.get(index)
			.unwrap()
			.dyn_into::<dom::HtmlElement>()
			.unwrap();
		code_select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
