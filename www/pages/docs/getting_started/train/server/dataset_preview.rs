use html::{component, html};
use tangram_ui as ui;

struct HeartDiseaseRow {
	age: String,
	chest_pain: String,
	cholesterol: String,
	diagnosis: String,
	exercise_induced_angina: String,
	exercise_max_heart_rate: String,
	exercise_st_depression: String,
	exercise_st_slope: String,
	fasting_blood_sugar_greater_than120: String,
	fluoroscopy_vessels_colored: String,
	gender: String,
	resting_blood_pressure: String,
	resting_ecg_result: String,
	thallium_stress_test: String,
}

#[component]
pub fn DatasetPreview() {
	let data = vec![
		HeartDiseaseRow {
			age: "63".to_owned(),
			chest_pain: "typical angina".to_owned(),
			cholesterol: "233".to_owned(),
			diagnosis: "Negative".to_owned(),
			exercise_induced_angina: "no".to_owned(),
			exercise_max_heart_rate: "150".to_owned(),
			exercise_st_depression: "2.3".to_owned(),
			exercise_st_slope: "downsloping".to_owned(),
			fasting_blood_sugar_greater_than120: "true".to_owned(),
			fluoroscopy_vessels_colored: "0.0".to_owned(),
			gender: "male".to_owned(),
			resting_blood_pressure: "145".to_owned(),
			resting_ecg_result: "probable or definite left ventricular hypertrophy".to_owned(),
			thallium_stress_test: "fixed defect".to_owned(),
		},
		HeartDiseaseRow {
			age: "67".to_owned(),
			chest_pain: "asymptomatic".to_owned(),
			cholesterol: "286".to_owned(),
			diagnosis: "Positive".to_owned(),
			exercise_induced_angina: "yes".to_owned(),
			exercise_max_heart_rate: "108".to_owned(),
			exercise_st_depression: "1.5".to_owned(),
			exercise_st_slope: "flat".to_owned(),
			fasting_blood_sugar_greater_than120: "false".to_owned(),
			fluoroscopy_vessels_colored: "3.0".to_owned(),
			gender: "male".to_owned(),
			resting_blood_pressure: "160".to_owned(),
			resting_ecg_result: "probable or definite left ventricular hypertrophy".to_owned(),
			thallium_stress_test: "normal".to_owned(),
		},
		HeartDiseaseRow {
			age: "67".to_owned(),
			chest_pain: "asymptomatic".to_owned(),
			cholesterol: "229".to_owned(),
			diagnosis: "Positive".to_owned(),
			exercise_induced_angina: "yes".to_owned(),
			exercise_max_heart_rate: "129".to_owned(),
			exercise_st_depression: "2.6".to_owned(),
			exercise_st_slope: "flat".to_owned(),
			fasting_blood_sugar_greater_than120: "false".to_owned(),
			fluoroscopy_vessels_colored: "2.0".to_owned(),
			gender: "male".to_owned(),
			resting_blood_pressure: "120".to_owned(),
			resting_ecg_result: "probable or definite left ventricular hypertrophy".to_owned(),
			thallium_stress_test: "reversible defect".to_owned(),
		},
	];
	html! {
		<ui::Table>
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"age"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"gender"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"chest_pain"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"resting_blood_pressure"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"cholesterol"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"fasting_blood_sugar"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"resting_ecg_result"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"exercise_max_heart_rate"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"exercise_induced_angina"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"exercise_st_depression"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"exercise_st_slope"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"fluoroscopy_vessels_colored"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"thal"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"diagnosis"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{data.into_iter().map(|entry| html! {
					<ui::TableRow>
						<ui::TableCell>{entry.age}</ui::TableCell>
						<ui::TableCell>{entry.gender}</ui::TableCell>
						<ui::TableCell>{entry.chest_pain}</ui::TableCell>
						<ui::TableCell>{entry.resting_blood_pressure}</ui::TableCell>
						<ui::TableCell>{entry.cholesterol}</ui::TableCell>
						<ui::TableCell>{entry.fasting_blood_sugar_greater_than120}</ui::TableCell>
						<ui::TableCell>{entry.resting_ecg_result}</ui::TableCell>
						<ui::TableCell>{entry.exercise_max_heart_rate}</ui::TableCell>
						<ui::TableCell>{entry.exercise_induced_angina}</ui::TableCell>
						<ui::TableCell>{entry.exercise_st_depression}</ui::TableCell>
						<ui::TableCell>{entry.exercise_st_slope}</ui::TableCell>
						<ui::TableCell>{entry.fluoroscopy_vessels_colored}</ui::TableCell>
						<ui::TableCell>{entry.thallium_stress_test}</ui::TableCell>
						<ui::TableCell>{entry.diagnosis}</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
