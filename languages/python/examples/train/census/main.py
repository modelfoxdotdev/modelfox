import os
import json
import pandas as pd
import pyarrow as pa
from pyarrow import csv
import modelfox
from typing import cast

# Load the csv into a pandas dataframe
table_train = pd.read_csv("./adult_train.csv")

# Load the csv into a pandas dataframe
table_test = pd.read_csv("./adult_test.csv")


# Train a model!
model = modelfox.train(
  table_train,
  "income",
  table_test,
  autogrid=None,
	grid=[
    {
      "type": "tree",
      "learning_rate": 0.01,
      "max_rounds": 1500,
      "model": "tree"
    }
  ]
)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = {
  "age": 37,
  "workclass": "Private",
  "fnlwgt": "284582",
  "education": "Masters",
  "education-num": 14,
  "marital-status": "Married-civ-spouse",
  "occupation": "Exec-managerial",
  "relationship": "Wife",
  "race": "White",
  "sex": "Female",
  "capital-gain": 0,
  "capital-loss": 0,
  "hours-per-week": 40,
  "native-country": "United-States",
}

# Make the prediction!
output = model.predict(input)

# Print the output.
print("Output:", output)

# Print the output.
print("Metrics:", model.test_metrics().default_threshold.accuracy)
