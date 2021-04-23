# Get the path to the .tangram file. In your application, you will probably want to put your .tangram file in your mix package's `priv` directory and read it like this: `Path.join(:code.priv_dir(:your_app_name), "model.tangram")`.
model_path = Path.join(Path.dirname(__ENV__.file), "heart_disease.tangram")
# Load the model from the path.
model = Tangram.load_model_from_path(model_path)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = %{
  :age => 63.0,
  :gender => "male",
  :chest_pain => "typical angina",
  :resting_blood_pressure => 145.0,
  :cholesterol => 233.0,
  :fasting_blood_sugar_greater_than_120 => "true",
  :resting_ecg_result => "probable or definite left ventricular hypertrophy",
  :exercise_max_heart_rate => 150.0,
  :exercise_induced_angina => "no",
  :exercise_st_depression => 2.3,
  :exercise_st_slope => "downsloping",
  :fluoroscopy_vessels_colored => "0",
  :thallium_stress_test => "fixed defect"
}

# Make the prediction!
output = Tangram.predict(model, input)

# Print the output.
IO.write("Output: ")
IO.inspect(output)
