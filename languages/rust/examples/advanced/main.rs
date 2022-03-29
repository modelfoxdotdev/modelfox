use anyhow::Result;

#[derive(Clone, Debug, modelfox::PredictInput)]
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

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum Gender {
	#[modelfox(value = "male")]
	Male,
	#[modelfox(value = "female")]
	Female,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum ChestPain {
	#[modelfox(value = "asymptomatic")]
	Asymptomatic,
	#[modelfox(value = "non-angina pain")]
	NonAnginaPain,
	#[modelfox(value = "atypical angina")]
	AtypicalAngina,
	#[modelfox(value = "typical angina")]
	TypicalAngina,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum FastingBloodSugarGreaterThan120 {
	#[modelfox(value = "false")]
	False,
	#[modelfox(value = "true")]
	True,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum RestingEcgResult {
	#[modelfox(value = "normal")]
	Normal,
	#[modelfox(value = "probable or definite left ventricular hypertrophy")]
	Lvh,
	#[modelfox(value = "ST-T wave abnormality")]
	SttWaveAbnormality,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum ExerciseInducedAngina {
	#[modelfox(value = "no")]
	No,
	#[modelfox(value = "yes")]
	Yes,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum ExerciseStSlope {
	#[modelfox(value = "upsloping")]
	Upsloping,
	#[modelfox(value = "flat")]
	Flat,
	#[modelfox(value = "downsloping")]
	Downsloping,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum FluoroscopyVesselsColored {
	#[modelfox(value = "0")]
	Zero,
	#[modelfox(value = "1")]
	One,
	#[modelfox(value = "2")]
	Two,
	#[modelfox(value = "3")]
	Three,
}

#[derive(Clone, Debug, modelfox::PredictInputValue)]
pub enum ThalliumStressTest {
	#[modelfox(value = "normal")]
	Normal,
	#[modelfox(value = "reversible defect")]
	ReversibleDefect,
	#[modelfox(value = "fixed defect")]
	FixedDefect,
}

type Output = modelfox::BinaryClassificationPredictOutput<Diagnosis>;

#[derive(Clone, Debug, modelfox::ClassificationOutputValue)]
enum Diagnosis {
	#[modelfox(value = "Negative")]
	Negative,
	#[modelfox(value = "Positive")]
	Positive,
}

fn main() -> Result<()> {
	// If you are running the ModelFox app on your own server you can pass the URL to it with the MODELFOX_URL environment variable.
	let modelfox_url = if let Ok(url) = std::env::var("MODELFOX_URL") {
		Some(url.parse()?)
	} else {
		None
	};

	// Load the model from the path.
	let options = modelfox::LoadModelOptions { modelfox_url };
	let mut model =
		modelfox::Model::<Input, Output>::from_path("heart_disease.modelfox", Some(options))?;

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
		fluoroscopy_vessels_colored: FluoroscopyVesselsColored::Zero,
		thallium_stress_test: ThalliumStressTest::FixedDefect,
	};

	// Make the prediction using a custom threshold chosen on the "Tuning" page of the ModelFox app.
	let options = modelfox::PredictOptions {
		threshold: Some(0.25),
		compute_feature_contributions: Some(true),
	};
	let output = model.predict_one(input.clone(), Some(options.clone()));

	// Print the output.
	println!("{:?}", output);

	// Log the prediction.
	model.log_prediction(modelfox::LogPredictionArgs {
		identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9".into(),
		input,
		options: Some(options),
		output,
	})?;

	// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
	model.log_true_value(modelfox::LogTrueValueArgs {
		identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9".into(),
		true_value: "Positive".into(),
	})?;

	Ok(())
}
