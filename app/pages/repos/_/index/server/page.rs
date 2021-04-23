use html::{component, html, Props};
use tangram_app_common::page_heading::{PageHeading, PageHeadingButtons};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub models_table_props: Option<ModelsTableProps>,
	pub title: String,
}

#[component]
pub fn Page(props: PageProps) {
	let models_table_or_empty_message = if let Some(models_table_props) = props.models_table_props {
		html! {
			<ModelsTable {models_table_props} />
		}
	} else {
		html! {
			<ui::Card>
				<ui::P>{"This repository has no models."}</ui::P>
			</ui::Card>
		}
	};
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				<ui::S1>
					<PageHeading>
						<ui::H1>{props.title}</ui::H1>
						<PageHeadingButtons>
							<ui::Button
								color?={Some(ui::colors::GRAY.to_owned())}
								href?="edit"
							>
								{"Edit"}
							</ui::Button>
							<ui::Button href?="models/new">
								{"Upload New Version"}
							</ui::Button>
						</PageHeadingButtons>
					</PageHeading>
					{models_table_or_empty_message}
				</ui::S1>
			</AppLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct ModelsTableProps {
	pub rows: Vec<ModelsTableRow>,
}

pub struct ModelsTableRow {
	pub id: String,
	pub created_at: String,
	pub tag: Option<String>,
}

#[component]
pub fn ModelsTable(props: ModelsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Id"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Tag"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Uploaded"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.into_iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
							<ui::Link href={format!("./models/{}/", row.id)}>
								{row.id.clone()}
							</ui::Link>
						</ui::TableCell>
						<ui::TableCell>
							{row.tag.clone()}
						</ui::TableCell>
						<ui::TableCell>
							{row.created_at}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
