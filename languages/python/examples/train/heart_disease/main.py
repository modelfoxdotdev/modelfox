import os
import json
import pandas as pd
import pyarrow as pa
from pyarrow import csv
import modelfox
from typing import cast

# # Load the csv into a pandas dataframe
# table = pd.read_csv("./heart_disease.csv")

# # Load the csv into a pyarrow dataframe
table = pa.csv.read_csv("./heart_disease.csv")

# Train a model!
model = modelfox.train(
  table,
  "diagnosis",
  column_types=[
    {
      "name": "age",
      "type": "number",
    },
    {
      "name": "gender",
      "type": "enum",
      "variants": ["female", "male"]
    },
    {
      "name": "chest_pain",
      "type": "enum",
      "variants": ["asymptomatic", "atypical angina", "non-angina pain", "typical angina"]
    },
    {
      "name": "resting_blood_pressure",
      "type": "number",
    },
    {
      "name": "cholesterol",
      "type": "number",
    },
    {
      "name": "fasting_blood_sugar_greater_than_120",
      "type": "enum",
      "variants": ["false", "true"]
    },
    {
      "name": "resting_ecg_result",
      "type": "enum",
      "variants": ["ST-T wave abnormality", "normal", "probable or definite left ventricular hypertrophy"]
    },
    {
      "name": "exercise_max_heart_rate",
      "type": "number",
    },
    {
      "name": "exercise_induced_angina",
      "type": "enum",
      "variants": ["no", "yes"]
    },
    {
      "name": "exercise_st_depression",
      "type": "number",
    },
    {
      "name": "exercise_st_slope",
      "type": "enum",
      "variants": ["downsloping", "flat", "upsloping"]
    },
    {
      "name": "fluoroscopy_vessels_colored",
      "type": "number",
    },
    {
      "name": "thallium_stress_test",
      "type": "enum",
      "variants": ["fixed defect", "normal", "reversible defect"]
    },
  ],
  comparison_fraction=0.05,
  autogrid=None,
	grid=[
    {
      "type": "linear"
    },
    {
      "type": "tree"
    }
  ],
	comparison_metric="accuracy"
)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = {
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
output = model.predict(input)

# Print the output.
print("Output:", output)

# Print the accuracy.
print("Accuracy:", model.test_metrics().default_threshold.accuracy)
