use html::{component, html, style, Props};
use sqlx::prelude::*;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	repos::get_model_version_ids,
	topbar::{Topbar, TopbarAvatar},
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_error::Result;
use tangram_id::Id;
use tangram_ui as ui;

#[derive(Props)]
pub struct ModelLayoutProps {
	pub model_id: Id,
	pub model_tag: Option<String>,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub topbar_avatar: Option<TopbarAvatar>,
	pub selected_item: ModelNavItem,
}

pub enum Owner {
	User { id: Id, email: String },
	Organization { id: Id, name: String },
}

pub async fn get_model_layout_props(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	model_id: Id,
	selected_item: ModelNavItem,
) -> Result<ModelLayoutProps> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				models.tag,
				repos.user_id,
				users.email,
				repos.organization_id,
				organizations.name
			from repos
			join models
				on models.repo_id = repos.id
			left join users
				on users.id = repos.user_id
			left join organizations
				on organizations.id = repos.organization_id
			where models.id = $1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let repo_id: String = row.get(0);
	let repo_id: Id = repo_id.parse()?;
	let repo_title: String = row.get(1);
	let model_tag: Option<String> = row.get(2);
	let model_version_ids = get_model_version_ids(&mut db, repo_id).await?;
	let owner_organization_id: Option<String> = row.get(3);
	let owner_organization_name: Option<String> = row.get(4);
	let owner_user_id: Option<String> = row.get(5);
	let owner_user_email: Option<String> = row.get(6);
	#[allow(clippy::manual_map)]
	let owner = if let Some(owner_user_id) = owner_user_id {
		Some(Owner::User {
			id: owner_user_id.parse().unwrap(),
			email: owner_user_email.unwrap(),
		})
	} else if let Some(owner_organization_id) = owner_organization_id {
		Some(Owner::Organization {
			id: owner_organization_id.parse().unwrap(),
			name: owner_organization_name.unwrap(),
		})
	} else {
		None
	};
	let topbar_avatar = if context.options.auth_enabled {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(ModelLayoutProps {
		model_id,
		model_tag,
		model_version_ids,
		owner,
		repo_id: repo_id.to_string(),
		repo_title,
		topbar_avatar,
		selected_item,
	})
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel,
	#[serde(rename = "download_model")]
	DownloadModel,
}

pub async fn post(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let response = delete_model(&mut db, model_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn delete_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<http::Response<hyper::Body>> {
	sqlx::query(
		"
		delete from models
		where
			models.id = $1
	",
	)
	.bind(&model_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/")
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}

pub async fn download(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(&context.options.data_storage, model_id).await?;
	let bytes = bytes.to_owned();
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(bytes))
		.unwrap();
	Ok(response)
}

#[derive(PartialEq)]
pub enum ModelNavItem {
	Overview,
	TrainingGrid,
	TrainingStats,
	TrainingMetrics,
	Playground,
	Tuning,
	ProductionPredictions,
	ProductionStats,
	ProductionMetrics,
}

#[component]
pub fn ModelLayout(props: ModelLayoutProps) {
	let selected_model_version_id = props
		.model_version_ids
		.iter()
		.find(|model_version_id| *model_version_id == &props.model_id)
		.unwrap();
	let topbar_avatar = props
		.topbar_avatar
		.as_ref()
		.map(|topbar_avatar| TopbarAvatar {
			avatar_url: topbar_avatar.avatar_url.to_owned(),
		});
	html! {
		<div class="model-layout">
			<div style={style! { "grid-area" => "topbar" }}>
				<Topbar topbar_avatar={topbar_avatar} />
			</div>
			<ModelLayoutTop
				model_id={props.model_id}
				model_tag={props.model_tag}
				model_version_ids={props.model_version_ids.clone()}
				owner={props.owner}
				repo_id={props.repo_id.clone()}
				repo_title={props.repo_title}
				selected_model_version_id={selected_model_version_id.to_string()}
			/>
			<div class="model-layout-left">
				<ModelNav
					repo_id={props.repo_id.to_string()}
					model_id={props.model_id.to_string()}
					selected_item={props.selected_item}
				/>
			</div>
			<div class="model-layout-center">{children}</div>
			<div class="model-layout-right"></div>
		</div>
	}
}

#[derive(Props)]
pub struct ModelLayoutTopProps {
	pub model_id: Id,
	pub model_tag: Option<String>,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub selected_model_version_id: String,
}

#[component]
pub fn ModelLayoutTop(props: ModelLayoutTopProps) {
	let repo_id = props.repo_id;
	let model_id = props.model_id;
	let model_heading = props.model_tag.unwrap_or_else(|| model_id.to_string());
	struct OwnerInfo {
		title: String,
		url: String,
	}
	let owner_info = props.owner.map(|owner| match owner {
		Owner::Organization { name, id } => OwnerInfo {
			title: name,
			url: format!("/organizations/{}", id),
		},
		Owner::User { email, .. } => OwnerInfo {
			title: email,
			url: "/user".to_owned(),
		},
	});
	html! {
		<div class="model-layout-top">
			<div class="model-layout-top-title-wrapper">
				{owner_info.map(|owner_info| {
					html! {
						<>
							<a
								class="model-layout-top-title-segment"
								href={owner_info.url}
								title="owner"
							>
								{owner_info.title}
							</a>
							<span class="model-layout-top-title-slash">{"/"}</span>
						</>
					}
				})}
				<a
					class="model-layout-top-title-segment"
					href={format!("/repos/{}/", repo_id)}
					title="repo"
				>
					{props.repo_title.clone()}
				</a>
				<span class="model-layout-top-title-slash">{"/"}</span>
				<a
					class="model-layout-top-title-segment"
					href={format!("/repos/{}/models/{}/", repo_id, props.model_id.to_string())}
					title="repo"
				>
					{model_heading}
				</a>
			</div>
			<div class="model-layout-top-buttons-wrapper">
				<ui::Button
					color?={Some(ui::colors::GRAY.to_owned())}
					href?={Some(format!("/repos/{}/models/{}/edit", repo_id, props.model_id))}
				>
					{"Edit"}
				</ui::Button>
				<ui::Button
					download?={Some(format!("{}.tangram", props.repo_title))}
					href?={Some(format!("/repos/{}/models/{}/download", repo_id, props.model_id))}
				>
					{"Download"}
				</ui::Button>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct ModelNavItemProps {
	repo_id: String,
	model_id: String,
	selected_item: ModelNavItem,
}

#[component]
fn ModelNav(props: ModelNavItemProps) {
	html! {
		<ui::Nav title?="Pages">
			<ui::NavSection title="Overview">
				<ui::NavItem
					title="Overview"
					href={Some(format!("/repos/{}/models/{}/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::Overview)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Training">
				<ui::NavItem
					title="Grid"
					href={Some(format!("/repos/{}/models/{}/training_grid/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::TrainingGrid)}
				/>
				<ui::NavItem
					title="Stats"
					href={Some(format!("/repos/{}/models/{}/training_stats/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::TrainingStats)}
				/>
				<ui::NavItem
					title="Metrics"
					href={Some(format!("/repos/{}/models/{}/training_metrics/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::TrainingMetrics)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Playground">
				<ui::NavItem
					title="Playground"
					href={Some(format!("/repos/{}/models/{}/playground", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::Playground)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Tuning">
				<ui::NavItem
					title="Tuning"
					href={Some(format!("/repos/{}/models/{}/tuning", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::Tuning)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Production">
				<ui::NavItem
					title="Predictions"
					href={Some(format!("/repos/{}/models/{}/production_predictions/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::ProductionPredictions)}
				/>
				<ui::NavItem
					title="Stats"
					href={Some(format!("/repos/{}/models/{}/production_stats/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::ProductionStats)}
				/>
				<ui::NavItem
					title="Metrics"
					href={Some(format!("/repos/{}/models/{}/production_metrics/", props.repo_id, props.model_id))}
					selected={Some(props.selected_item == ModelNavItem::ProductionMetrics)}
				/>
			</ui::NavSection>
		</ui::Nav>
	}
}
