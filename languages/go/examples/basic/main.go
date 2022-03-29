package main

import (
	"fmt"
	"log"

	"github.com/modelfoxdotdev/modelfox-go"
)

func main() {
	// Load the model from the path.
	model, err := modelfox.LoadModelFromPath("heart_disease.modelfox", nil)
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

	// Create the options
	options := modelfox.PredictOptions{
		Threshold: 0.5,
	}

	// Make the prediction!
	output := model.PredictOne(input, &options)

	// Print the output.
	fmt.Println("Output:", output)
}
