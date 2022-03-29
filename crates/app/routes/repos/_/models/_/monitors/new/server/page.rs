use pinwheel::prelude::*;
use modelfox_app_core::monitor::AlertModelType;
use modelfox_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use modelfox_app_ui::page_heading::PageHeading;
use modelfox_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub model_type: AlertModelType,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let metric_options = match self.model_type {
			AlertModelType::Classifier => vec![ui::SelectFieldOption {
				text: "Accuracy".to_owned(),
				value: "accuracy".to_owned(),
			}],
			AlertModelType::Regressor => vec![
				ui::SelectFieldOption {
					text: "Mean Squared Error".to_owned(),
					value: "mse".to_owned(),
				},
				ui::SelectFieldOption {
					text: "Root Mean Squared Error".to_owned(),
					value: "rmse".to_owned(),
				},
			],
		};
		Document::new()
			.client("modelfox_app_new_monitor_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new()
								.child(ui::H1::new("Specify Production Alert Monitor")),
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
										.options(metric_options),
								)
								.child(
									ui::TextField::new()
										.label("Lower Threshold Value".to_string())
										.name("threshold_lower".to_string())
										.required(false),
								)
								.child(
									ui::TextField::new()
										.label("Upper Threshold Value".to_string())
										.name("threshold_upper".to_string())
										.required(false),
								)
								.child(
									ui::SelectField::new()
										.label("Threshold Mode".to_string())
										.name("mode".to_string())
										.required(true)
										.options(vec![
											ui::SelectFieldOption {
												text: "Absolute".to_owned(),
												value: "absolute".to_owned(),
											},
											ui::SelectFieldOption {
												text: "Percentage".to_owned(),
												value: "percentage".to_owned(),
											},
										]),
								)
								.child(
									ui::TextField::new()
										.label("Title (Optional)".to_string())
										.name("title".to_string())
										.required(false),
								)
								.child(
									ui::TextField::new()
										.label("Email Address".to_string())
										.name("email".to_string())
										.required(false),
								)
								.child(
									ui::TextField::new()
										.label("Webhook URL".to_string())
										.name("webhook".to_string())
										.required(false),
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
