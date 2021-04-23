use html::{component, html, Props};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub inner: Inner,
}

pub enum Inner {
	Auth(AuthProps),
	NoAuth(NoAuthProps),
}

#[derive(Props)]
pub struct AuthProps {
	pub details_section_props: DetailsSectionProps,
	pub organizations_section_props: OrganizationsSectionProps,
	pub repos_section_props: ReposSectionProps,
}

#[derive(Props)]
pub struct NoAuthProps {
	pub repos_section_props: ReposSectionProps,
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Auth(inner) => {
			html! {
				<Auth {inner} />
			}
		}
		Inner::NoAuth(inner) => {
			html! {
				<NoAuth {inner} />
			}
		}
	};
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				{inner}
			</AppLayout>
		</Document>
	}
}

#[component]
pub fn NoAuth(props: NoAuthProps) {
	html! {
		<ui::S1>
			<ui::P>
				{"You are using the free version of tangram that does not support user accounts or organizations. Checkout out the different plans that allow you to collaborate with your team."}
			</ui::P>
		</ui::S1>
	}
}

#[component]
pub fn Auth(props: AuthProps) {
	let AuthProps {
		organizations_section_props,
		repos_section_props,
		details_section_props,
	} = props;
	html! {
		<ui::S1>
			<Header />
			<DetailsSection {details_section_props} />
			<OrganizationsSection {organizations_section_props} />
			<ReposSection {repos_section_props} />
		</ui::S1>
	}
}

#[component]
fn Header() {
	html! {
		<ui::SpaceBetween>
			<ui::H1>{"User"}</ui::H1>
			<ui::Form post?={Some(true)}>
				<input
					name="action"
					type="hidden"
					value="logout"
				/>
				<ui::Button color?="var(--red)">
					{"Logout"}
				</ui::Button>
			</ui::Form>
		</ui::SpaceBetween>
	}
}

#[derive(Props)]
pub struct DetailsSectionProps {
	pub email: String,
}

#[component]
fn DetailsSection(props: DetailsSectionProps) {
	html! {
		<ui::S2>
			<ui::Form>
				<ui::TextField
					label?="Email"
					readonly?={Some(true)}
					value?={Some(props.email)}
				/>
			</ui::Form>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct ReposSectionProps {
	pub repos_table_props: Option<ReposTableProps>,
}

#[component]
fn ReposSection(props: ReposSectionProps) {
	let repos_table_or_empty_message = if let Some(repos_table_props) = props.repos_table_props {
		html! {
			<ReposTable {repos_table_props} />
		}
	} else {
		html! {
			<ui::Card>
				<ui::P>{"You do not have any repos."}</ui::P>
			</ui::Card>
		}
	};
	html! {
		<ui::S2>
			<ui::SpaceBetween>
				<ui::H2>{"User Repos"}</ui::H2>
				<ui::Button
					href?="/repos/new"
					id?={None}
				>
					{"Create New Repo"}
				</ui::Button>
			</ui::SpaceBetween>
			{repos_table_or_empty_message}
		</ui::S2>
	}
}

#[derive(Props)]
pub struct ReposTableProps {
	pub rows: Vec<ReposTableRow>,
}

pub struct ReposTableRow {
	pub id: String,
	pub title: String,
}

#[component]
fn ReposTable(props: ReposTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Repo Title"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
			{props.rows.into_iter().map(|row| html! {
				<ui::TableRow>
					<ui::TableCell>
						<ui::Link href={format!("/repos/{}/", row.id)}>
							{row.title}
						</ui::Link>
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct OrganizationsSectionProps {
	pub organizations_table_props: Option<OrganizationsTableProps>,
}

#[component]
fn OrganizationsSection(props: OrganizationsSectionProps) {
	let organizations_table_or_empty_message =
		if let Some(organizations_table_props) = props.organizations_table_props {
			html! {
				<OrganizationsTable {organizations_table_props} />
			}
		} else {
			html! {
				<ui::Card>
					<ui::P>{"You do not have any organizations."}</ui::P>
				</ui::Card>
			}
		};
	html! {
		<ui::S2>
			<ui::SpaceBetween>
				<ui::H2>{"Organizations"}</ui::H2>
				<ui::Button href?="/organizations/new">
					{"Create New Organization"}
				</ui::Button>
			</ui::SpaceBetween>
			{organizations_table_or_empty_message}
		</ui::S2>
	}
}

#[derive(Props)]
pub struct OrganizationsTableProps {
	pub rows: Vec<OrganizationsTableRow>,
}

pub struct OrganizationsTableRow {
	pub id: String,
	pub name: String,
}

#[component]
fn OrganizationsTable(props: OrganizationsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Organization Name"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
							<ui::Link href={format!("/organizations/{}/", row.id)}>
								{row.name.clone()}
							</ui::Link>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
