use pinwheel::prelude::*;
use tangram_app_common::alerts::{AlertHeuristics, AlertMethod, AlertModelType};
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub alert: AlertHeuristics,
	pub alert_id: String,
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
		let email = self
			.alert
			.methods
			.iter()
			.fold(String::new(), |acc, el| match el {
				AlertMethod::Email(e) => e.to_string(),
				_ => acc,
			});
		let lower = if let Some(l) = self.alert.threshold.variance_lower {
			l.to_string()
		} else {
			String::new()
		};
		let upper = if let Some(u) = self.alert.threshold.variance_upper {
			u.to_string()
		} else {
			String::new()
		};
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child(format!("Edit {} Alert", self.alert.title)))
						.child(
							ui::Form::new()
								.post(true)
								.child(
									input()
										.attribute("name", "action")
										.attribute("type", "hidden")
										.attribute("value", "update_alert"),
								)
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
										])
										.value(self.alert.cadence.to_string().to_lowercase()),
								)
								.child(
									ui::SelectField::new()
										.label("Alert Metric".to_owned())
										.name("metric".to_owned())
										.required(true)
										.options(metric_options)
										.value(self.alert.threshold.metric.short_name()),
								)
								.child(
									ui::TextField::new()
										.label("Lower Threshold Value".to_string())
										.name("threshold_lower".to_string())
										.required(false)
										.value(lower),
								)
								.child(
									ui::TextField::new()
										.label("Upper Threshold Value".to_string())
										.name("threshold_upper".to_string())
										.required(false)
										.value(upper),
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
										])
										.value(self.alert.threshold.mode.to_string()),
								)
								.child(
									ui::TextField::new()
										.label("Title (Optional)".to_string())
										.name("title".to_string())
										.required(false)
										.value(self.alert.title),
								)
								.child(
									ui::TextField::new()
										.label("Email Address".to_string())
										.name("email".to_string())
										.required(false)
										.value(email),
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
										.child("Update"),
								),
						)
						.child(DangerZone),
				),
			)
			.into_node()
	}
}

struct DangerZone;

impl Component for DangerZone {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Danger Zone"))
			.child(
				ui::Form::new()
					.post(true)
					.onsubmit("return confirm(\"Are you sure?\")".to_owned())
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "delete"),
					)
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.color(ui::colors::RED.to_owned())
							.child("Delete"),
					),
			)
			.into_node()
	}
}
