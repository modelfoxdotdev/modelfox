use html::{component, html, Props};
use tangram_app_common::tokens::{BASELINE_COLOR, TRAINING_COLOR};
use tangram_ui as ui;
use tangram_zip::zip;

#[derive(Props)]
pub struct MulticlassClassifierProps {
	pub warning: Option<String>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<ClassMetrics>,
	pub classes: Vec<String>,
}

pub struct ClassMetrics {
	pub precision: f32,
	pub recall: f32,
}

#[component]
pub fn MulticlassClassifier(props: MulticlassClassifierProps) {
	html! {
	<ui::S1>
		<ui::H1>{"Training Metrics"}</ui::H1>
		<ui::TabBar>
			<ui::TabLink
				href=""
				selected={true}
			>
				{"Overview"}
			</ui::TabLink>
			<ui::TabLink
				href="class_metrics"
				selected={false}
			>
				{"Class Metrics"}
			</ui::TabLink>
		</ui::TabBar>
		<ui::S2>
			<ui::H2>{"Accuracy"}</ui::H2>
			<ui::P>{"Accuracy is the percentage of predictions that were correct."}</ui::P>
			<ui::NumberComparisonCard
				color_a={Some(BASELINE_COLOR.to_owned())}
				color_b={Some(TRAINING_COLOR.to_owned())}
				title="Accuracy"
				value_a={Some(props.baseline_accuracy)}
				value_a_title="Baseline"
				value_b={Some(props.accuracy)}
				value_b_title="Training"
				number_formatter={ui::NumberFormatter::Percent(Default::default())}
			/>
		</ui::S2>
		<ui::S2>
			<ui::H2>{"Precision and Recall"}</ui::H2>
			<ui::P>{"Precision is the percentage of examples that were labeled as this class that are actually this class. Recall is the percentage of examples that are of this class that were labeled as this class."}</ui::P>
			<ui::Table width?="100%">
				<ui::TableHeader>
					<ui::TableRow>
						<ui::TableHeaderCell>
							{"Class"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell>
							{"Precision"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell>
							{"Recall"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
				{zip!(props.class_metrics, props.classes).map(|(class_metrics, class_name)| html! {
					<ui::TableRow>
						<ui::TableCell>{class_name}</ui::TableCell>
						<ui::TableCell>
							{ui::format_percent(class_metrics.precision)}
						</ui::TableCell>
						<ui::TableCell>
							{ui::format_percent(class_metrics.recall)}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
		</ui::S2>
	</ui::S1>
	}
}
