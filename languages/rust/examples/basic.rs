fn main() {
	// Load the model from the path.
	let model: tangram::Model =
		tangram::Model::from_path("examples/heart_disease.tangram", None).unwrap();

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	let input = tangram::predict_input! {
		"age": 63.0,
		"gender": "male",
		"chest_pain": "typical angina",
		"resting_blood_pressure": 145.0,
		"cholesterol": 233.0,
		"fasting_blood_sugar_greater_than_120": "true",
		"resting_ecg_result": "probable or definite left ventricular hypertrophy",
		"exercise_max_heart_rate": 150.0,
		"exercise_induced_angina": "no",
		"exercise_st_depression": 2.3,
		"exercise_st_slope": "downsloping",
		"fluoroscopy_vessels_colored": 0.0,
		"thallium_stress_test": "fixed defect",
	};

	// Make the prediction!
	let output = model.predict_one(input, None);

	// Print the output.
	println!("Output: {:?}", output);
}
