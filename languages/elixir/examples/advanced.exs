# If you are running the Tangram app locally or on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
tangram_url = System.get_env("TANGRAM_URL")

# Get the path to the `.tangram` file. In your application, you will probably want to put your `.tangram` file in your mix package's `priv` directory and read it like this: `Path.join(:code.priv_dir(:your_app_name), "model.tangram")`.
model_path = Path.join(Path.dirname(__ENV__.file), "heart_disease.tangram")
# Load the model from the path.
model = Tangram.load_model_from_path(model_path, %{tangram_url: tangram_url})

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = %{
  "age" => 63.0,
  "gender" => "male",
  "chest_pain" => "typical angina",
  "resting_blood_pressure" => 145.0,
  "cholesterol" => 233.0,
  "fasting_blood_sugar_greater_than_120" => "true",
  "resting_ecg_result" => "probable or definite left ventricular hypertrophy",
  "exercise_max_heart_rate" => 150.0,
  "exercise_induced_angina" => "no",
  "exercise_st_depression" => 2.3,
  "exercise_st_slope" => "downsloping",
  "fluoroscopy_vessels_colored" => "0",
  "thallium_stress_test" => "fixed defect"
}

# Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
predict_options = %Tangram.PredictOptions{
  threshold: 0.5,
  compute_feature_contributions: false
}
output = Tangram.predict(model, input, predict_options)

# Print the output.
IO.write("Output: ")
IO.inspect(output)

# Log the prediction.
Tangram.log_prediction(model, %Tangram.LogPredictionArgs{
  identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
  options: predict_options,
  input: input,
  output: output,
})

# Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
Tangram.log_true_value(model, %Tangram.LogTrueValueArgs{
  identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
  true_value: "Positive",
})
