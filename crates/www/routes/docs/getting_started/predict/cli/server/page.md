You can make predictions using the `predict` subcommand of the CLI. Save the following CSV file with a single row to `test.csv`:

```
age,gender,chest_pain,resting_blood_pressure,cholesterol,fasting_blood_sugar_greater_than_120,resting_ecg_result,exercise_max_heart_rate,exercise_induced_angina,exercise_st_depression,exercise_st_slope,fluoroscopy_vessels_colored,thallium_stress_test
63,male,typical angina,145,233,true,probable or definite left ventricular hypertrophy,150,no,2.3,downsloping,0,fixed defect
```

You can make predictions like so:

```
$ cat test.csv | modelfox predict --model heart_disease.modelfox
diagnosis
Positive
```

If you prefer to work with files instead of stdin/stdout:

```
$ modelfox predict --model heart_disease.modelfox --file test.csv --output output.csv
```
