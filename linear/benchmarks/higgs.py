from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.metrics import mean_squared_error
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['sklearn', 'pytorch'], required=True)
args = parser.parse_args()

# Load the data.
# path_train = 'data/higgs_500k_train.csv'
# path_test = 'data/higgs_500k_test.csv'
path_train = 'data/higgs_train.csv'
path_test = 'data/higgs_test.csv'
target_column_name = "signal"
dtype = {
	'signal': np.bool,
	'lepton_pt': np.float64,
	'lepton_eta': np.float64,
	'lepton_phi': np.float64,
	'missing_energy_magnitude': np.float64,
	'missing_energy_phi': np.float64,
	'jet_1_pt': np.float64,
	'jet_1_eta': np.float64,
	'jet_1_phi': np.float64,
	'jet_1_b_tag': np.float64,
	'jet_2_pt': np.float64,
	'jet_2_eta': np.float64,
	'jet_2_phi': np.float64,
	'jet_2_b_tag': np.float64,
	'jet_3_pt': np.float64,
	'jet_3_eta': np.float64,
	'jet_3_phi': np.float64,
	'jet_3_b_tag': np.float64,
	'jet_4_pt': np.float64,
	'jet_4_eta': np.float64,
	'jet_4_phi': np.float64,
	'jet_4_b_tag': np.float64,
	'm_jj': np.float64,
	'm_jjj': np.float64,
	'm_lv': np.float64,
	'm_jlv': np.float64,
	'm_bb': np.float64,
	'm_wbb': np.float64,
	'm_wwbb': np.float64,
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

if args.library == 'pytorch' or args.library == 'sklearn':
	from sklearn.linear_model import SGDRegressor
	from sklearn.preprocessing import StandardScaler
	from sklearn.compose import ColumnTransformer
	from sklearn.pipeline import Pipeline
	from sklearn.impute import SimpleImputer
	from sklearn.preprocessing import StandardScaler, OneHotEncoder
	numeric_features = features_train.select_dtypes(
		include=[np.float64, np.int64]
	).columns
	numeric_transformer = Pipeline(steps=[
		('imputer', SimpleImputer(strategy='median')),
		('scaler', StandardScaler())
	])
	preprocessor = ColumnTransformer(
		transformers=[
			('num', numeric_transformer, numeric_features),
	])
	features_train = preprocessor.fit_transform(features_train)
	features_test = preprocessor.transform(features_test)

# Train the model.
if args.library == 'pytorch':
	from pytorch_linear import LinearBinaryClassifier
	model = LinearBinaryClassifier(batch_size=1000, n_epochs=1, learning_rate=0.01)
	model.fit(features_train, labels_train)
elif args.library == 'sklearn':
	from sklearn.linear_model import SGDClassifier
	model = SGDClassifier(
		max_iter=1,
		eta0=0.01,
		learning_rate='constant',
		tol=None,
		loss='log'
	)
	model.fit(features_train, labels_train)

# Make predictions on the test data.
if args.library == 'pytorch':
	predictions_proba = model.predict_proba(features_test)
else:
	predictions_proba = model.predict_proba(features_test)[:, 1]

# Compute metrics.
auc_roc = roc_auc_score(labels_test, predictions_proba)

# Compute memory usage.
f = open("/proc/self/status", "r")
for line in f.readlines():
	if line.startswith("VmHWM"):
		memory = line.split(":")[1].strip()

print(json.dumps({
	'auc_roc': auc_roc,
	'memory': memory,
}))