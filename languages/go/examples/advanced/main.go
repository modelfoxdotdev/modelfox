package main

import (
	"fmt"
	"log"
	"os"

	"github.com/modelfoxdotdev/modelfox-go"
)

func main() {
	// If you are running the ModelFox app on your own server you can pass the URL to it with the MODELFOX_URL environment variable.
	modelfoxURL, present := os.LookupEnv("MODELFOX_URL")
	if !present {
		modelfoxURL = "https://app.modelfox.dev"
	}

	// Load the model from the path.
	options := modelfox.LoadModelOptions{
		ModelFoxURL: modelfoxURL,
	}
	model, err := modelfox.LoadModelFromPath("heart_disease.modelfox", &options)
	if err != nil {
		log.Fatal(err)
	}
	// Destroy the model when it is no longer needed to free up memory.
	defer model.Destroy()

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	input := modelfox.PredictInput{
		"age":                                  63,
		"gender":                               "male",
		"chest_pain":                           "typical angina",
		"resting_blood_pressure":               145,
		"cholesterol":                          233,
		"fasting_blood_sugar_greater_than_120": "true",
		"resting_ecg_result":                   "probable or definite left ventricular hypertrophy",
		"exercise_max_heart_rate":              150,
		"exercise_induced_angina":              "no",
		"exercise_st_depression":               2.3,
		"exercise_st_slope":                    "downsloping",
		"fluoroscopy_vessels_colored":          "0",
		"thallium_stress_test":                 "fixed defect",
	}

	// Make the prediction using a custom threshold chosen on the "Tuning" page of the ModelFox app.
	predictOptions := modelfox.PredictOptions{
		Threshold:                   0.5,
		ComputeFeatureContributions: true,
	}
	output := model.PredictOne(input, &predictOptions).(modelfox.BinaryClassificationPredictOutput)

	// Print the output.
	fmt.Println("Output:", output)

	// Log the prediction.
	err = model.LogPrediction(modelfox.LogPredictionArgs{
		Identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
		Input:      input,
		Options:    predictOptions,
		Output:     output,
	})
	if err != nil {
		log.Fatal(err)
	}

	// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
	err = model.LogTrueValue(modelfox.LogTrueValueArgs{
		Identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
		TrueValue:  "Positive",
	})
	if err != nil {
		log.Fatal(err)
	}
}
