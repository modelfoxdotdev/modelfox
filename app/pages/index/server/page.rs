use html::{component, html, text, Props};
use tangram_app_common::page_heading::{PageHeading, PageHeadingButtons};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub repos_table_props: Option<ReposTableProps>,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				<ui::S1>
					<PageHeading>
						<ui::H1>{"Suppositories"}</ui::H1>
						<PageHeadingButtons>
							<ui::Button href?="/repos/new">
								{"Create Repo"}
							</ui::Button>
						</PageHeadingButtons>
					</PageHeading>
					{if let Some(repos_table_props) = props.repos_table_props {
						html! {
							<ReposTable {repos_table_props} />
						}
					} else {
						html! {
							<ui::Card>
								<ui::P>
									{"You do not have any repositories."}
								</ui::P>
							</ui::Card>
						}
					}}
				</ui::S1>
			</AppLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct ReposTableProps {
	pub rows: Vec<ReposTableRow>,
}

pub struct ReposTableRow {
	pub id: String,
	pub title: String,
	pub owner_name: Option<String>,
}

#[component]
fn ReposTable(props: ReposTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell expand?={Some(true)}>
						{"Name"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.into_iter().map(|repo| html! {
					<ui::TableRow>
						<ui::TableCell>
							<ui::Link href={format!("/repos/{}/", repo.id)}>
								{repo.owner_name.map(|owner_name| text!(format!("{}/", owner_name)))}
								{repo.title}
							</ui::Link>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
