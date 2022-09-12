import os
import modelfox

# Get the path to the CSV file.
csv_path = os.path.join(os.path.dirname(__file__), "heart_disease.csv")
# Get the path to the .modelfox file.
model_path = os.path.join(os.path.dirname(__file__), "heart_disease.modelfox")
# Train a model.
model = modelfox.Model.train(csv_path, "diagnosis", model_path)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
specimen = {
    "age": 63,
    "gender": "male",
    "chest_pain": "typical angina",
    "resting_blood_pressure": 145,
    "cholesterol": 233,
    "fasting_blood_sugar_greater_than_120": "true",
    "resting_ecg_result": "probable or definite left ventricular hypertrophy",
    "exercise_max_heart_rate": 150,
    "exercise_induced_angina": "no",
    "exercise_st_depression": 2.3,
    "exercise_st_slope": "downsloping",
    "fluoroscopy_vessels_colored": "0",
    "thallium_stress_test": "fixed defect",
}

# Make the prediction!
output = model.predict(specimen)

# Print the output.
print("Output.class_name:", output.class_name)
print("Output.probability:", output.probability)
