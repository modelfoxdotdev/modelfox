import * as fs from "fs"
import * as path from "path"

// In your app this should be `import * as tangram from "../"`.
import * as tangram from "../"

// Define the type for the input to the model.
type Input = {
	age: number | null | undefined
	chest_pain:
		| "asymptomatic"
		| "non-angina pain"
		| "atypical angina"
		| "typical angina"
		| null
		| undefined
	cholesterol: number | null | undefined
	exercise_induced_angina: "no" | "yes" | null | undefined
	exercise_max_heart_rate: number | null | undefined
	exercise_st_depression: number | null | undefined
	exercise_st_slope: "upsloping" | "flat" | "downsloping" | null | undefined
	fasting_blood_sugar_greater_than_120: false | true | null | undefined
	fluoroscopy_vessels_colored: "0" | "1" | "2" | "3" | null | undefined
	gender: "female" | "male" | null | undefined
	resting_blood_pressure: number | null | undefined
	resting_ecg_result:
		| "normal"
		| "probable or definite left ventricular hypertrophy"
		| "ST-T wave abnormality"
		| null
		| undefined
	thallium_stress_test:
		| "normal"
		| "reversible defect"
		| "fixed defect"
		| null
		| undefined
}

// Define the type for the output of the model.
type Output = tangram.MulticlassClassificationOutput<"Positive" | "Negative">

// If you are running the Tangram app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
let tangramUrl = process.env.TANGRAM_URL || "https://app.tangram.xyz"

// Get the path to the .tangram file.
let modelPath = path.join(__dirname, "heart_disease.tangram")
// Load the model from the path and set the url where the tangram app is running.
let modelData = fs.readFileSync(modelPath)
let model = new tangram.Model<Input, Output>(modelData, {
	tangramUrl,
})

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
let input: Input = {
	age: 63,
	chest_pain: "typical angina",
	cholesterol: 233,
	exercise_induced_angina: "no",
	exercise_max_heart_rate: 150,
	exercise_st_depression: 2.3,
	exercise_st_slope: "downsloping",
	fasting_blood_sugar_greater_than_120: true,
	fluoroscopy_vessels_colored: "0",
	gender: "male",
	resting_blood_pressure: 145,
	resting_ecg_result: "probable or definite left ventricular hypertrophy",
	thallium_stress_test: "fixed defect",
}

// Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
let options = { threshold: 0.25 }
let output = model.predictSync(input, options)

// Print the output.
console.log("Output", output)

// Log the prediction.
model.logPrediction({
	identifier: "John Doe",
	input,
	options,
	output,
})

// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
model.logTrueValue({
	identifier: "John Doe",
	trueValue: "Positive",
})
