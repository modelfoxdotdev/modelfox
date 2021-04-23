use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub model_layout_props: ModelLayoutProps,
	pub pagination: Pagination,
	pub prediction_table: Option<PredictionTable>,
}

pub struct PredictionTable {
	pub rows: Vec<PredictionTableRow>,
}

pub struct PredictionTableRow {
	pub date: String,
	pub identifier: String,
	pub output: String,
}

pub struct Pagination {
	pub after: Option<usize>,
	pub before: Option<usize>,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Production Predictions"}</ui::H1>
					{if props.prediction_table.is_none() {
						html! { <ui::P>{"You have not yet logged any predictions."}</ui::P> }
					} else { html! {
						<>
							<ui::Form post?={Some(true)}>
								<div class="search-bar-wrapper">
									<ui::TextField
										autocomplete?="off"
										label?="Identifier"
										name?="identifier"
									/>
									<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
										{"Lookup"}
									</ui::Button>
								</div>
							</ui::Form>
							<ui::Table width?="100%">
								<ui::TableHeader>
									<ui::TableRow>
										<ui::TableHeaderCell>
											{"Identifier"}
										</ui::TableHeaderCell>
										<ui::TableHeaderCell>
											{"Date"}
										</ui::TableHeaderCell>
										<ui::TableHeaderCell>
											{"Output"}
										</ui::TableHeaderCell>
									</ui::TableRow>
								</ui::TableHeader>
								<ui::TableBody>
									{props.prediction_table.map(|prediction_table| prediction_table.rows.iter().map(|prediction| html! {
										<ui::TableRow>
											<ui::TableCell>
												<ui::Link href={format!("./predictions/{}", prediction.identifier)}>
													{prediction.identifier.clone()}
												</ui::Link>
											</ui::TableCell>
											<ui::TableCell>
											{prediction.date.clone()}
											</ui::TableCell>
											<ui::TableCell>
												{prediction.output.clone()}
											</ui::TableCell>
										</ui::TableRow>
									}).collect::<Vec<_>>())}
								</ui::TableBody>
							</ui::Table>
							<div class="pagination-buttons">
								<ui::Form>
									{props.pagination.after.map(|after| html! {
										<input
											name="after"
											type="hidden"
											value={after.to_string()}
										/>
									})}
									<ui::Button
										button_type?={Some(ui::ButtonType::Submit)}
										disabled?={Some(props.pagination.after.is_none())}
									>
										{"Newer"}
									</ui::Button>
								</ui::Form>
								<ui::Form>
									{props.pagination.before.map(|before| html! {
										<input
											name="before"
											type="hidden"
											value={before.to_string()}
										/>
									})}
									<ui::Button
										button_type?={Some(ui::ButtonType::Submit)}
										disabled?={Some(props.pagination.before.is_none())}
									>
										{"Older"}
									</ui::Button>
								</ui::Form>
							</div>
						</>
					}
				}}
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}
