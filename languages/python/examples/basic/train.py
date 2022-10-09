import os
import pyarrow as pa
from pyarrow.cffi import ffi as arrow_c
import polars as pl
import modelfox

# Get the path to the CSV file.
csv_path = os.path.join(os.path.dirname(__file__), "heart_disease.csv")
# Get the path to the .modelfox file.
model_path = os.path.join(os.path.dirname(__file__), "heart_disease.modelfox")

# Read the CSV file into a PyArrow.
df = pl.read_csv(csv_path)
arrow = df.to_arrow()
with arrow_c.new("struct ArrowArray*") as c_array, \
     arrow_c.new("struct ArrowSchema*") as c_schema:
    c_array_ptr = int(arrow_c.cast("uintptr_t", c_array))
    c_schema_ptr = int(arrow_c.cast("uintptr_t", c_schema))

    # Export the Array and its schema to the C Data structures.
    print(type(arrow[1]), arrow[1])
    arrow[1].combine_chunks()._export_to_c(c_array_ptr)
    arrow[1].combine_chunks().type._export_to_c(c_schema_ptr)

    # Train a model.
    model = modelfox.Model.train((c_array_ptr, c_schema_ptr), "diagnosis", model_path)

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
