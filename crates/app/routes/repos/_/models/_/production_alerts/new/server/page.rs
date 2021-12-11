use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::page_heading::PageHeading;
use tangram_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_app_new_model_production_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new()
								.child(ui::H1::new().child("Specify Production Alert")),
						)
						.child(
							ui::Form::new()
								.post(true)
								.child(
									self.error.map(|error| {
										ui::Alert::new(ui::Level::Danger).child(error)
									}),
								)
								.child(
									ui::SelectField::new()
										.label("Alert Cadence".to_owned())
										.name("cadence".to_owned())
										.required(true)
										.options(vec![
											ui::SelectFieldOption {
												text: "Hourly".to_owned(),
												value: "hourly".to_owned(),
											},
											ui::SelectFieldOption {
												text: "Daily".to_owned(),
												value: "daily".to_owned(),
											},
											ui::SelectFieldOption {
												text: "Weekly".to_owned(),
												value: "weekly".to_owned(),
											},
											ui::SelectFieldOption {
												text: "Monthly".to_owned(),
												value: "monthly".to_owned(),
											},
										]),
								)
								.child(
									ui::SelectField::new()
										.label("Alert Metric".to_owned())
										.name("metric".to_owned())
										.required(true)
										.options(vec![
											ui::SelectFieldOption {
												text: "Accuracy".to_owned(),
												value: "accuracy".to_owned(),
											},
											ui::SelectFieldOption {
												text: "Root Mean Squared Error".to_owned(),
												value: "rmse".to_owned(),
											},
										]),
								)
								.child(
									ui::TextField::new()
										.label("Threshold Value".to_string())
										.name("threshold".to_string())
										.required(true),
								)
								.child(
									ui::TextField::new()
									.label("Email Address".to_string())
									.name("email".to_string())
									.required(false)
								)
								.child(
									ui::Button::new()
										.button_type(ui::ButtonType::Submit)
										.child("Create"),
								),
						),
				),
			)
			.into_node()
	}
}
