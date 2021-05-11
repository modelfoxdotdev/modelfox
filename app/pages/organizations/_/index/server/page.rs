use html::{component, html, Props};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_id::Id;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub details_props: DetailsSectionProps,
	pub id: String,
	pub members_props: MembersSectionProps,
	pub name: String,
	pub repos_props: ReposSectionProps,
	pub can_delete: bool,
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
					<ui::H1>{props.name}</ui::H1>
					<DetailsSection {props.details_props} />
					<MembersSection {props.members_props} />
					<ReposSection {props.repos_props} />
					{if props.can_delete {
						Some(html! { <DangerZoneSection /> })
					} else {
						None
					}}
				</ui::S1>
			</AppLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct DetailsSectionProps {
	pub organization_id: String,
	pub organization_name: String,
	pub can_edit: bool,
}

#[component]
fn DetailsSection(props: DetailsSectionProps) {
	html! {
		<ui::S2>
			<ui::SpaceBetween>
				<ui::H2>{"Details"}</ui::H2>
				<ui::Button
					color?="var(--gray)"
					disabled?={Some(false)}
					href?={Some(format!("/organizations/{}/edit", props.organization_id))}
				>
					{"Edit"}
				</ui::Button>
			</ui::SpaceBetween>
			<ui::TextField
				disabled?={Some(true)}
				value?={Some(props.organization_name)}
				label?="Organization Name"
				readonly?={Some(true)}
			/>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct MembersSectionProps {
	pub organization_id: Id,
	pub members_table_props: MembersTableProps,
}

pub struct MembersTableRow {
	pub id: Id,
	pub email: String,
	pub is_admin: bool,
}

#[component]
fn MembersSection(props: MembersSectionProps) {
	html! {
		<ui::S2>
			<ui::SpaceBetween>
				<ui::H2>{"Members"}</ui::H2>
				<ui::Button href?={Some(format!("/organizations/{}/members/new", props.organization_id))}>
					{"Invite Team Member"}
				</ui::Button>
			</ui::SpaceBetween>
			<MembersTable {props.members_table_props} />
		</ui::S2>
	}
}

#[derive(Props)]
pub struct MembersTableProps {
	pub user_id: String,
	pub can_edit: bool,
	pub rows: Vec<MembersTableRow>,
}

#[component]
fn MembersTable(props: MembersTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Email"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Role"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
						{if props.can_edit {
							html! {
								<ui::Link href={format!("members/{}", row.id)}>
									{row.email.clone()}
								</ui::Link>
							}
						} else {
							html! { <>{row.email.clone()}</> }
						}}
						</ui::TableCell>
						<ui::TableCell>
							{if row.is_admin {
								"Admin"
							} else {
								"Member"
							}}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct MemberDeleteFormProps {
	member_id: String,
}

#[component]
fn MemberDeleteForm(props: MemberDeleteFormProps) {
	html! {
		<ui::Form post?={Some(true)}>
			<input
				name="action"
				type="hidden"
				value="delete_member"
			/>
			<input
				name="member_id"
				type="hidden"
				value={props.member_id}
			/>
			<ui::Button button_type?={Some(ui::ButtonType::Submit)} color?="var(--red)">
				{"Remove"}
			</ui::Button>
		</ui::Form>
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
				<ui::P>{"This organization does not have any repos."}</ui::P>
			</ui::Card>
		}
	};
	html! {
		<ui::S2>
			<ui::SpaceBetween>
				<ui::H2>{"Repos"}</ui::H2>
				<ui::Button href?="/repos/new">
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

#[component]
fn DangerZoneSection() {
	html! {
		<ui::S2>
			<ui::H2>{"Danger Zone"}</ui::H2>
			<ui::Form post?={Some(true)} onsubmit?="return confirm(\"Are you sure?\")">
				<input
					name="action"
					type="hidden"
					value="delete_organization"
				/>
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					color?="var(--red)"
				>
					{"Delete Organization"}
				</ui::Button>
			</ui::Form>
		</ui::S2>
	}
}
