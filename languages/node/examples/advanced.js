let fs = require("fs")
let path = require("path")

// In your app this should be `let tangram = require('@tangramxyz/tangram-node')`.
let tangram = require("../")

// If you are running the Tangram app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
let tangramUrl = process.env.TANGRAM_URL || "https://app.tangram.xyz"

// Get the path to the .tangram file.
let modelPath = path.join(__dirname, "heart_disease.tangram")
// Load the model from the path and set the url where the tangram app is running.
let model = new tangram.Model(modelPath, { tangramUrl })

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
let input = {
	age: 63,
	chest_pain: "typical angina",
	cholesterol: 233,
	exercise_induced_angina: "no",
	exercise_max_heart_rate: 150,
	exercise_st_depression: 2.3,
	exercise_st_slope: "downsloping",
	fasting_blood_sugar_greater_than_120: "true",
	fluoroscopy_vessels_colored: "0",
	gender: "male",
	resting_blood_pressure: 145,
	resting_ecg_result: "probable or definite left ventricular hypertrophy",
	thallium_stress_test: "fixed defect",
}

// Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
options = { threshold: 0.5, computeFeatureContributions: true }
let output = model.predictSync(input, options)

// Print the output.
console.log("Output:", output)

// Log the prediction.
model.logPrediction({
	identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
	input,
	options,
	output,
})

// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
model.logTrueValue({
	identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
	trueValue: "Positive",
})
