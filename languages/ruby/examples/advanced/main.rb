require 'tangram'

# If you are running the Tangram app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
tangram_url = ENV['TANGRAM_URL'] || 'https://app.tangram.dev'

# Get the path to the `.tangram` file.
model_path = File.join(File.dirname(__FILE__), 'heart_disease.tangram')
# Load the model from the path and set the url where the tangram app is running.
options = Tangram::LoadModelOptions.new(tangram_url: tangram_url)
model = Tangram::Model.from_path(model_path, options: options)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = {
  age: 63.0,
  gender: 'male',
  chest_pain: 'typical angina',
  resting_blood_pressure: 145.0,
  cholesterol: 233.0,
  fasting_blood_sugar_greater_than_120: 'true',
  resting_ecg_result: 'probable or definite left ventricular hypertrophy',
  exercise_max_heart_rate: 150.0,
  exercise_induced_angina: 'no',
  exercise_st_depression: 2.3,
  exercise_st_slope: 'downsloping',
  fluoroscopy_vessels_colored: '0',
  thallium_stress_test: 'fixed defect'
}

# Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
options = Tangram::PredictOptions.new(threshold: 0.5, compute_feature_contributions: true)
output = model.predict(input, options: options)

# Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram app.
puts('Input:', input)
puts('Output:', output)

# Log the prediction.
model.log_prediction(
  identifier: '71762b29-2296-4bf9-a1d4-59144d74c9d9',
  input: input,
  output: output,
  options: options
)

# Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
model.log_true_value(
  identifier: '71762b29-2296-4bf9-a1d4-59144d74c9d9',
  true_value: 'Positive'
)
