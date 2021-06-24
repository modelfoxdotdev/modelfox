use anyhow::Result;

#[derive(Clone, Debug, tangram::PredictInput)]
pub struct Input {
	pub age: f32,
	pub gender: Gender,
	pub chest_pain: ChestPain,
	pub resting_blood_pressure: f32,
	pub cholesterol: f32,
	pub fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120,
	pub resting_ecg_result: RestingEcgResult,
	pub exercise_max_heart_rate: f32,
	pub exercise_induced_angina: ExerciseInducedAngina,
	pub exercise_st_depression: f32,
	pub exercise_st_slope: ExerciseStSlope,
	pub fluoroscopy_vessels_colored: FluoroscopyVesselsColored,
	pub thallium_stress_test: ThalliumStressTest,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum Gender {
	#[tangram(value = "male")]
	Male,
	#[tangram(value = "female")]
	Female,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ChestPain {
	#[tangram(value = "asymptomatic")]
	Asymptomatic,
	#[tangram(value = "non-angina pain")]
	NonAnginaPain,
	#[tangram(value = "atypical angina")]
	AtypicalAngina,
	#[tangram(value = "typical angina")]
	TypicalAngina,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum FastingBloodSugarGreaterThan120 {
	#[tangram(value = "false")]
	False,
	#[tangram(value = "true")]
	True,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum RestingEcgResult {
	#[tangram(value = "normal")]
	Normal,
	#[tangram(value = "probable or definite left ventricular hypertrophy")]
	Lvh,
	#[tangram(value = "ST-T wave abnormality")]
	SttWaveAbnormality,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ExerciseInducedAngina {
	#[tangram(value = "no")]
	No,
	#[tangram(value = "yes")]
	Yes,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ExerciseStSlope {
	#[tangram(value = "upsloping")]
	Upsloping,
	#[tangram(value = "flat")]
	Flat,
	#[tangram(value = "downsloping")]
	Downsloping,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum FluoroscopyVesselsColored {
	#[tangram(value = "0")]
	Zero,
	#[tangram(value = "1")]
	One,
	#[tangram(value = "2")]
	Two,
	#[tangram(value = "3")]
	Three,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ThalliumStressTest {
	#[tangram(value = "normal")]
	Normal,
	#[tangram(value = "reversible defect")]
	ReversibleDefect,
	#[tangram(value = "fixed defect")]
	FixedDefect,
}

// type Output = tangram_core::ClassificationOutput<Diagnosis>;

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
// enum Diagnosis {
// 	#[tangram(value = "negative")]
// 	Negative,
// 	#[tangram(value = "positive")]
// 	Positive,
// }

fn main() -> Result<()> {
	// If you are running the Tangram app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
	let tangram_url = if let Ok(url) = std::env::var("TANGRAM_URL") {
		Some(url.parse()?)
	} else {
		None
	};

	// Load the model from the path.
	let options = tangram::LoadModelOptions { tangram_url };
	let mut model = tangram::Model::<Input, tangram::BinaryClassificationPredictOutput>::from_path(
		"examples/heart_disease.tangram",
		Some(options),
	)?;

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	let input = Input {
		age: 63.0,
		gender: Gender::Male,
		chest_pain: ChestPain::TypicalAngina,
		resting_blood_pressure: 145.0,
		cholesterol: 233.0,
		fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120::True,
		resting_ecg_result: RestingEcgResult::Lvh,
		exercise_max_heart_rate: 150.0,
		exercise_induced_angina: ExerciseInducedAngina::No,
		exercise_st_depression: 2.3,
		exercise_st_slope: ExerciseStSlope::Downsloping,
		fluoroscopy_vessels_colored: FluoroscopyVesselsColored::One,
		thallium_stress_test: ThalliumStressTest::FixedDefect,
	};

	// Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
	let options = tangram::PredictOptions {
		threshold: Some(0.25),
		compute_feature_contributions: Some(true),
	};
	let output = model.predict_one(input.clone(), Some(options.clone()));

	// Print the output.
	println!("{:?}", output);

	// Log the prediction.
	model.log_prediction(tangram::LogPredictionArgs {
		identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9".into(),
		input,
		options: Some(options),
		output,
	})?;

	// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
	model.log_true_value(tangram::LogTrueValueArgs {
		identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9".into(),
		true_value: "Positive".into(),
	})?;

	Ok(())
}
