import * as path from "https://deno.land/std/node/path.ts"
import * as tangram from "https://js.tangram.dev/deno"
import * as url from "https://deno.land/std/node/url.ts"

// Get the path to the .tangram file.
let modelPath = path.join(
	path.dirname(url.fileURLToPath(import.meta.url)),
	"heart_disease.tangram",
)
// Load the model from the path.
let modelData = await Deno.readFile(modelPath)
let model = new tangram.Model(modelData.buffer)

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

// Make the prediction!
let output = model.predict(input)

// Print the output.
console.log("Output:", output)
