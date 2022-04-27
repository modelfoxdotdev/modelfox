use modelfox_ui as ui;
use pinwheel::prelude::*;

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

pub struct DatasetPreview;

impl Component for DatasetPreview {
	fn into_node(self) -> Node {
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
		ui::Table::new()
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("age"))
						.child(ui::TableHeaderCell::new().child("gender"))
						.child(ui::TableHeaderCell::new().child("chest_pain"))
						.child(ui::TableHeaderCell::new().child("resting_blood_pressure"))
						.child(ui::TableHeaderCell::new().child("cholesterol"))
						.child(ui::TableHeaderCell::new().child("fasting_blood_sugar"))
						.child(ui::TableHeaderCell::new().child("resting_ecg_result"))
						.child(ui::TableHeaderCell::new().child("exercise_max_heart_rate"))
						.child(ui::TableHeaderCell::new().child("exercise_induced_angina"))
						.child(ui::TableHeaderCell::new().child("exercise_st_depression"))
						.child(ui::TableHeaderCell::new().child("exercise_st_slope"))
						.child(ui::TableHeaderCell::new().child("fluoroscopy_vessels_colored"))
						.child(ui::TableHeaderCell::new().child("thal"))
						.child(ui::TableHeaderCell::new().child("diagnosis")),
				),
			)
			.child(ui::TableBody::new().children(data.into_iter().map(|entry| {
				ui::TableRow::new()
					.child(ui::TableCell::new().child(entry.age))
					.child(ui::TableCell::new().child(entry.gender))
					.child(ui::TableCell::new().child(entry.chest_pain))
					.child(ui::TableCell::new().child(entry.resting_blood_pressure))
					.child(ui::TableCell::new().child(entry.cholesterol))
					.child(ui::TableCell::new().child(entry.fasting_blood_sugar_greater_than120))
					.child(ui::TableCell::new().child(entry.resting_ecg_result))
					.child(ui::TableCell::new().child(entry.exercise_max_heart_rate))
					.child(ui::TableCell::new().child(entry.exercise_induced_angina))
					.child(ui::TableCell::new().child(entry.exercise_st_depression))
					.child(ui::TableCell::new().child(entry.exercise_st_slope))
					.child(ui::TableCell::new().child(entry.fluoroscopy_vessels_colored))
					.child(ui::TableCell::new().child(entry.thallium_stress_test))
					.child(ui::TableCell::new().child(entry.diagnosis))
			})))
			.into_node()
	}
}
