import * as fs from "fs/promises"
import * as path from "path"
import * as modelfox from "@modelfoxdotdev/modelfox"
import * as url from "url"

// Define the type for the input to the model.
type Input = {
	age?: number
	chest_pain?: ChestPain
	cholesterol?: number
	exercise_induced_angina?: ExerciseInducedAngina
	exercise_max_heart_rate?: number
	exercise_st_depression?: number
	exercise_st_slope?: ExerciseStSlope
	fasting_blood_sugar_greater_than_120?: FastingBloodSugarGreaterThan120
	fluoroscopy_vessels_colored?: FluoroscopyVesselsColored
	gender?: Gender
	resting_blood_pressure?: number
	resting_ecg_result?: RestingEcgResult
	thallium_stress_test?: ThalliumStressTest
}

enum Gender {
	Male = "male",
	Female = "female",
}

enum ChestPain {
	Asymptomatic = "asymptomatic",
	NonAnginaPain = "non-angina pain",
	AtypicalAngina = "atypical angina",
	TypicalAngina = "typical angina",
}

enum FastingBloodSugarGreaterThan120 {
	False = "false",
	True = "true",
}

enum RestingEcgResult {
	Normal = "normal",
	Lvh = "probable or definite left ventricular hypertrophy",
	SttWaveAbnormality = "ST-T wave abnormality",
}

enum ExerciseInducedAngina {
	No = "no",
	Yes = "yes",
}

enum ExerciseStSlope {
	Upsloping = "upsloping",
	Flat = "flat",
	Downsloping = "downsloping",
}

enum FluoroscopyVesselsColored {
	Zero = "0",
	One = "1",
	Two = "2",
	Three = "3",
}

enum ThalliumStressTest {
	Normal = "normal",
	ReversibleDefect = "reversible defect",
	FixedDefect = "fixed defect",
}

// Define the type for the output of the model.
type Output = modelfox.BinaryClassificationPredictOutput<Diagnosis>

enum Diagnosis {
	Negative = "Negative",
	Positive = "Positive",
}

// If you are running the ModelFox app on your own server you can pass the URL to it with the MODELFOX_URL environment variable.
let modelfoxUrl = process.env.MODELFOX_URL

// Get the path to the .modelfox file.
let modelPath = path.join(
	path.dirname(url.fileURLToPath(import.meta.url)),
	"heart_disease.modelfox",
)
// Load the model from the path.
let modelData = await fs.readFile(modelPath)
let model = new modelfox.Model<modelfox.Task.BinaryClassification, Input, Output>(
	modelData.buffer,
	{
		modelfoxUrl,
	},
)

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
let input: Input = {
	age: 63,
	chest_pain: ChestPain.TypicalAngina,
	cholesterol: 233,
	exercise_induced_angina: ExerciseInducedAngina.No,
	exercise_max_heart_rate: 150,
	exercise_st_depression: 2.3,
	exercise_st_slope: ExerciseStSlope.Downsloping,
	fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120.True,
	fluoroscopy_vessels_colored: FluoroscopyVesselsColored.Zero,
	gender: Gender.Male,
	resting_blood_pressure: 145,
	resting_ecg_result: RestingEcgResult.Lvh,
	thallium_stress_test: ThalliumStressTest.FixedDefect,
}

// Make the prediction using a custom threshold chosen on the "Tuning" page of the ModelFox app.
let options = { threshold: 0.25 }
let output = model.predict(input, options)

// Print the output.
console.log("Output", output)
