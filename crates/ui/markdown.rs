use crate as ui;
use convert_case::Casing;
use pinwheel::prelude::*;
use pulldown_cmark::{escape::escape_html, Alignment, CodeBlockKind, Event, Options, Parser, Tag};
use std::{borrow::Cow, fmt::Write};

#[derive(builder, new)]
pub struct Markdown {
	string: Cow<'static, str>,
}

enum State {
	Ground,
	Code {
		code: Option<String>,
		language: Option<ui::Language>,
	},
	Table {
		part: TablePart,
		alignments: Vec<Alignment>,
		column_index: usize,
	},
	Heading {
		heading: Option<String>,
	},
}

enum TablePart {
	Head,
	Body,
}

impl Component for Markdown {
	fn into_node(self) -> Node {
		let markdown = div().class("markdown");
		let mut state = State::Ground;
		let parser = Parser::new_ext(&self.string, Options::all());
		let mut html = String::new();
		for event in parser {
			match event {
				Event::Start(tag) => match tag {
					Tag::Paragraph => {
						html.push_str("<p>");
					}
					Tag::Heading(_, _, _) => {
						state = State::Heading { heading: None };
					}
					Tag::BlockQuote => {
						write!(&mut html, "<blockquote>").unwrap();
					}
					Tag::CodeBlock(kind) => {
						let language = match kind {
							CodeBlockKind::Indented => None,
							CodeBlockKind::Fenced(language) => match language.as_ref() {
								"elixir" => Some(ui::Language::Elixir),
								"go" => Some(ui::Language::Go),
								"javscript" => Some(ui::Language::Javascript),
								"php" => Some(ui::Language::Php),
								"python" => Some(ui::Language::Python),
								"ruby" => Some(ui::Language::Ruby),
								"rust" => Some(ui::Language::Rust),
								_ => None,
							},
						};
						state = State::Code {
							code: None,
							language,
						};
					}
					Tag::List(start) => {
						if let Some(start) = start {
							write!(&mut html, "<ol start=\"{}\">", start).unwrap();
						} else {
							html.push_str("<ul>");
						}
					}
					Tag::Item => {
						html.push_str("<li>");
					}
					Tag::FootnoteDefinition(_) => {
						unimplemented!();
					}
					Tag::Table(alignments) => {
						html.push_str("<div class=\"table\"><table>");
						state = State::Table {
							part: TablePart::Head,
							alignments,
							column_index: 0,
						};
					}
					Tag::TableHead => {
						html.push_str("<thead>");
					}
					Tag::TableRow => {
						html.push_str("<tr>");
						match &mut state {
							State::Table { column_index, .. } => *column_index = 0,
							_ => unreachable!(),
						}
					}
					Tag::TableCell => {
						match &state {
							State::Table {
								part: TablePart::Head,
								..
							} => html.push_str("<th"),
							State::Table {
								part: TablePart::Body,
								..
							} => html.push_str("<td"),
							_ => unreachable!(),
						};
						match &mut state {
							State::Table {
								alignments,
								column_index,
								..
							} => match alignments[*column_index] {
								Alignment::None => {}
								Alignment::Left => {
									html.push_str(" style=\"text-align: left;\"");
								}
								Alignment::Center => {
									html.push_str(" style=\"text-align: center;\"");
								}
								Alignment::Right => {
									html.push_str(" style=\"text-align: right;\"");
								}
							},
							_ => unreachable!(),
						}
						html.push('>');
					}
					Tag::Emphasis => {
						html.push_str("<em>");
					}
					Tag::Strong => {
						html.push_str("<strong>");
					}
					Tag::Strikethrough => {
						html.push_str("<del>");
					}
					Tag::Link(_, href, _) => {
						write!(&mut html, "<a href=\"{}\">", href).unwrap();
					}
					Tag::Image(_, src, alt) => {
						let node = ui::Img::new()
							.alt(alt.into_string())
							.src(src.into_string())
							.into_node();
						write!(&mut html, "{}", node).unwrap();
					}
				},
				Event::End(tag) => match tag {
					Tag::Paragraph => {
						html.push_str("</p>");
					}
					Tag::Heading(level, id, _) => match &state {
						State::Heading { heading, .. } => {
							write!(&mut html, "<h{}", level).unwrap();
							let id = id.map(|id| id.to_owned()).or_else(|| {
								heading.as_ref().map(|heading| {
									heading.to_lowercase().to_case(convert_case::Case::Snake)
								})
							});
							if let Some(id) = id {
								write!(&mut html, " id=\"{}\"", id).unwrap();
							}
							write!(&mut html, ">").unwrap();
							if let Some(heading) = heading {
								write!(&mut html, "{}", heading).unwrap();
							}
							write!(&mut html, "</h{}>", level).unwrap();
							state = State::Ground;
						}
						_ => unreachable!(),
					},
					Tag::BlockQuote => {
						write!(&mut html, "</blockquote>").unwrap();
					}
					Tag::CodeBlock(_) => match &state {
						State::Code { language, code } => {
							let code = code.as_ref().unwrap().to_owned();
							let code = ui::Code::new().code(Cow::Owned(code)).language(*language);
							let node = ui::Card::new().child(code).into_node();
							write!(&mut html, "{}", node).unwrap();
							state = State::Ground;
						}
						_ => unreachable!(),
					},
					Tag::List(start) => {
						if start.is_some() {
							html.push_str("</ol>");
						} else {
							html.push_str("</ul>");
						}
					}
					Tag::Item => {
						html.push_str("</li>");
					}
					Tag::FootnoteDefinition(_) => {
						unimplemented!();
					}
					Tag::Table(_) => {
						html.push_str("</tbody></table></div>");
						state = State::Ground;
					}
					Tag::TableHead => {
						html.push_str("</thead><tbody>");
						match &mut state {
							State::Table { part, .. } => *part = TablePart::Body,
							_ => unreachable!(),
						};
					}
					Tag::TableRow => {
						html.push_str("</tr>");
					}
					Tag::TableCell => {
						match &state {
							State::Table {
								part: TablePart::Head,
								..
							} => html.push_str("</th>"),
							State::Table {
								part: TablePart::Body,
								..
							} => html.push_str("</td>"),
							_ => unreachable!(),
						};
						match &mut state {
							State::Table { column_index, .. } => *column_index += 1,
							_ => unreachable!(),
						};
					}
					Tag::Emphasis => {
						html.push_str("</em>");
					}
					Tag::Strong => {
						html.push_str("</strong>");
					}
					Tag::Strikethrough => {
						html.push_str("</del>");
					}
					Tag::Link(_, _, _) => {
						html.push_str("</a>");
					}
					Tag::Image(_, _, _) => {}
				},
				Event::Text(text) => {
					match &mut state {
						State::Code { code, .. } => *code = Some(text.into_string()),
						State::Heading { heading, .. } => *heading = Some(text.into_string()),
						_ => escape_html(&mut html, &text).unwrap(),
					};
				}
				Event::Code(code) => {
					html.push_str(r#"<span class="inline-code">"#);
					escape_html(&mut html, &code).unwrap();
					html.push_str(r#"</span>"#);
				}
				Event::Html(raw) => {
					html.push_str(&raw);
				}
				Event::FootnoteReference(_reference) => unimplemented!(),
				Event::SoftBreak => {
					html.push(' ');
				}
				Event::HardBreak => {
					html.push('\n');
				}
				Event::Rule => {
					html.push_str("<hr />");
				}
				Event::TaskListMarker(checked) => {
					html.push_str("<input type=\"checkbox\" disabled=\"\"");
					if checked {
						html.push_str("checked=\"\"");
					}
					html.push_str(" />");
				}
			};
		}
		markdown.inner_html(html).into_node()
	}
}
