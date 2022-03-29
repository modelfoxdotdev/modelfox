defmodule Example do
  def start(_type, _args) do
    # If you are running the ModelFox app locally or on your own server you can pass the URL to it with the MODELFOX_URL environment variable.
    modelfox_url = System.get_env("MODELFOX_URL")

    # Get the path to the .modelfox file.
    model_path = Path.join(:code.priv_dir(:example), "heart_disease.modelfox")
    # Load the model from the path.
    model = ModelFox.load_model_from_path(model_path, %{modelfox_url: modelfox_url})

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

    # Make the prediction using a custom threshold chosen on the "Tuning" page of the ModelFox app.
    predict_options = %ModelFox.PredictOptions{
      threshold: 0.5,
      compute_feature_contributions: false
    }

    output = ModelFox.predict(model, input, predict_options)

    # Print the output.
    IO.write("Output: ")
    IO.inspect(output)

    # Log the prediction.
    ModelFox.log_prediction(model, %ModelFox.LogPredictionArgs{
      identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
      options: predict_options,
      input: input,
      output: output
    })

    # Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
    ModelFox.log_true_value(model, %ModelFox.LogTrueValueArgs{
      identifier: "71762b29-2296-4bf9-a1d4-59144d74c9d9",
      true_value: "Positive"
    })

    Supervisor.start_link([], strategy: :one_for_one)
  end
end
