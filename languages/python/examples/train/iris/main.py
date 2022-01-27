import pandas as pd
import modelfox

# Load the data.

dataset_url = "https://datasets.modelfox.dev/"

path_train = dataset_url + "iris_train.csv"
path_test = dataset_url + "iris_test.csv"
target_column_name = "species"
data_train = pd.read_csv(path_train)
data_test = pd.read_csv(path_test)

# Train a model!
model = modelfox.train(
  data_train,
  "species",
  data_test
)

input = {
"sepal_length": 5.0,
"sepal_width": 3.0,
"petal_length": 1.6,
"petal_width": 0.2
}

# Make the prediction!
output = model.predict(input)

# Print the output.
print("Output:", output)

# Print the output.
print("Metrics:", model.test_metrics().accuracy)
