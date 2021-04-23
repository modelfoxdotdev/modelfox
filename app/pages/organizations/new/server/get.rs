use crate::page::{Page, PageProps};
use html::html;
use tangram_app_common::Context;
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let app_layout_props = get_app_layout_props(context).await?;
	let props = PageProps {
		app_layout_props,
		error: None,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
