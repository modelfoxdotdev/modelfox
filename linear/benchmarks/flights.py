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
# path_train = 'data/flights_100k_train.csv'
# path_test = 'data/flights_test.csv'
# path_train = 'data/flights_1m_train.csv'
# path_test = 'data/flights_test.csv'
path_train = 'data/flights_10m_train.csv'
path_test = 'data/flights_test.csv'
target_column_name = "dep_delayed_15min"
month_options = [
	"c-1", "c-10", "c-11", "c-12", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
]
day_of_week_options = ["c-1", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7"]
day_of_month_options = [
	"c-1", "c-10", "c-11", "c-12", "c-13", "c-14", "c-15", "c-16", "c-17", "c-18", "c-19",
	"c-2", "c-20", "c-21", "c-22", "c-23", "c-24", "c-25", "c-26", "c-27", "c-28", "c-29",
	"c-3", "c-30", "c-31", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
]
carrier_options = [
	"AA", "AQ", "AS", "B6", "CO", "DH", "DL", "EV", "F9", "FL", "HA", "HP", "MQ", "NW", "OH",
	"OO", "TZ", "UA", "US", "WN", "XE", "YV",
]
origin_options = [
	"ABE", "ABI", "ABQ", "ABY", "ACK", "ACT", "ACV", "ACY", "ADK", "ADQ", "AEX", "AGS", "AKN",
	"ALB", "AMA", "ANC", "APF", "ASE", "ATL", "ATW", "AUS", "AVL", "AVP", "AZO", "BDL", "BET",
	"BFL", "BGM", "BGR", "BHM", "BIL", "BIS", "BLI", "BMI", "BNA", "BOI", "BOS", "BPT", "BQK",
	"BQN", "BRO", "BRW", "BTM", "BTR", "BTV", "BUF", "BUR", "BWI", "BZN", "CAE", "CAK", "CDC",
	"CDV", "CEC", "CHA", "CHO", "CHS", "CIC", "CID", "CLD", "CLE", "CLL", "CLT", "CMH", "CMI",
	"COD", "COS", "CPR", "CRP", "CRW", "CSG", "CVG", "CWA", "DAB", "DAL", "DAY", "DBQ", "DCA",
	"DEN", "DFW", "DHN", "DLG", "DLH", "DRO", "DSM", "DTW", "EGE", "EKO", "ELP", "ERI", "EUG",
	"EVV", "EWR", "EYW", "FAI", "FAR", "FAT", "FAY", "FCA", "FLG", "FLL", "FLO", "FNT", "FSD",
	"FSM", "FWA", "GEG", "GFK", "GGG", "GJT", "GNV", "GPT", "GRB", "GRK", "GRR", "GSO", "GSP",
	"GST", "GTF", "GTR", "GUC", "HDN", "HKY", "HLN", "HNL", "HOU", "HPN", "HRL", "HSV", "HTS",
	"HVN", "IAD", "IAH", "ICT", "IDA", "ILG", "ILM", "IND", "IPL", "ISO", "ISP", "ITO", "IYK",
	"JAC", "JAN", "JAX", "JFK", "JNU", "KOA", "KTN", "LAN", "LAS", "LAW", "LAX", "LBB", "LCH",
	"LEX", "LFT", "LGA", "LGB", "LIH", "LIT", "LNK", "LRD", "LSE", "LWB", "LWS", "LYH", "MAF",
	"MBS", "MCI", "MCN", "MCO", "MDT", "MDW", "MEI", "MEM", "MFE", "MFR", "MGM", "MHT", "MIA",
	"MKE", "MLB", "MLI", "MLU", "MOB", "MOD", "MOT", "MQT", "MRY", "MSN", "MSO", "MSP", "MSY",
	"MTJ", "MYR", "OAJ", "OAK", "OGG", "OKC", "OMA", "OME", "ONT", "ORD", "ORF", "OTZ", "OXR",
	"PBI", "PDX", "PFN", "PHF", "PHL", "PHX", "PIA", "PIE", "PIH", "PIT", "PNS", "PSC", "PSE",
	"PSG", "PSP", "PVD", "PWM", "RAP", "RDD", "RDM", "RDU", "RFD", "RIC", "RNO", "ROA", "ROC",
	"RST", "RSW", "SAN", "SAT", "SAV", "SBA", "SBN", "SBP", "SCC", "SCE", "SDF", "SEA", "SFO",
	"SGF", "SGU", "SHV", "SIT", "SJC", "SJT", "SJU", "SLC", "SMF", "SMX", "SNA", "SOP", "SPI",
	"SPS", "SRQ", "STL", "STT", "STX", "SUN", "SWF", "SYR", "TEX", "TLH", "TOL", "TPA", "TRI",
	"TTN", "TUL", "TUP", "TUS", "TVC", "TWF", "TXK", "TYR", "TYS", "VCT", "VIS", "VLD", "VPS",
	"WRG", "WYS", "XNA", "YAK", "YUM",
]
dest_options= [
	"ABE", "ABI", "ABQ", "ABY", "ACK", "ACT", "ACV", "ACY", "ADK", "ADQ", "AEX", "AGS", "AKN",
	"ALB", "AMA", "ANC", "APF", "ASE", "ATL", "ATW", "AUS", "AVL", "AVP", "AZO", "BDL", "BET",
	"BFL", "BGM", "BGR", "BHM", "BIL", "BIS", "BLI", "BMI", "BNA", "BOI", "BOS", "BPT", "BQK",
	"BQN", "BRO", "BRW", "BTM", "BTR", "BTV", "BUF", "BUR", "BWI", "BZN", "CAE", "CAK", "CDC",
	"CDV", "CEC", "CHA", "CHO", "CHS", "CIC", "CID", "CLD", "CLE", "CLL", "CLT", "CMH", "CMI",
	"COD", "COS", "CPR", "CRP", "CRW", "CSG", "CVG", "CWA", "DAB", "DAL", "DAY", "DBQ", "DCA",
	"DEN", "DFW", "DHN", "DLG", "DLH", "DRO", "DSM", "DTW", "EGE", "EKO", "ELP", "ERI", "EUG",
	"EVV", "EWR", "EYW", "FAI", "FAR", "FAT", "FAY", "FCA", "FLG", "FLL", "FLO", "FNT", "FSD",
	"FSM", "FWA", "GEG", "GFK", "GGG", "GJT", "GNV", "GPT", "GRB", "GRK", "GRR", "GSO", "GSP",
	"GST", "GTF", "GTR", "GUC", "HDN", "HKY", "HLN", "HNL", "HOU", "HPN", "HRL", "HSV", "HTS",
	"HVN", "IAD", "IAH", "ICT", "IDA", "ILG", "ILM", "IND", "IPL", "ISO", "ISP", "ITO", "IYK",
	"JAC", "JAN", "JAX", "JFK", "JNU", "KOA", "KTN", "LAN", "LAS", "LAW", "LAX", "LBB", "LBF",
	"LCH", "LEX", "LFT", "LGA", "LGB", "LIH", "LIT", "LNK", "LRD", "LSE", "LWB", "LWS", "LYH",
	"MAF", "MBS", "MCI", "MCN", "MCO", "MDT", "MDW", "MEI", "MEM", "MFE", "MFR", "MGM", "MHT",
	"MIA", "MKE", "MLB", "MLI", "MLU", "MOB", "MOD", "MOT", "MQT", "MRY", "MSN", "MSO", "MSP",
	"MSY", "MTJ", "MYR", "OAJ", "OAK", "OGG", "OKC", "OMA", "OME", "ONT", "ORD", "ORF", "OTZ",
	"OXR", "PBI", "PDX", "PFN", "PHF", "PHL", "PHX", "PIA", "PIE", "PIH", "PIT", "PNS", "PSC",
	"PSE", "PSG", "PSP", "PVD", "PWM", "RAP", "RDD", "RDM", "RDU", "RFD", "RIC", "RNO", "ROA",
	"ROC", "RST", "RSW", "SAN", "SAT", "SAV", "SBA", "SBN", "SBP", "SCC", "SCE", "SDF", "SEA",
	"SFO", "SGF", "SGU", "SHV", "SIT", "SJC", "SJT", "SJU", "SLC", "SMF", "SMX", "SNA", "SOP",
	"SPI", "SPS", "SRQ", "STL", "STT", "STX", "SUN", "SWF", "SYR", "TEX", "TLH", "TOL", "TPA",
	"TRI", "TTN", "TUL", "TUP", "TUS", "TVC", "TWF", "TXK", "TYR", "TYS", "VCT", "VIS", "VLD",
	"VPS", "WRG", "WYS", "XNA", "YAK", "YUM",
]
dtype = {
	'month': CategoricalDtype(categories=month_options) ,
	'day_of_month': CategoricalDtype(categories=day_of_month_options),
	'day_of_week': CategoricalDtype(categories=day_of_week_options),
	'dep_time': np.int64,
	'unique_carrier': CategoricalDtype(categories=carrier_options),
	'origin': CategoricalDtype(categories=origin_options),
	'dest': CategoricalDtype(categories=origin_options),
	'distance': np.int64,
	'dep_delayed_15min': CategoricalDtype(categories=['N','Y']),
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
if args.library == 'pytorch' or args.library == 'sklearn':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
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
	categorical_features = features_train.select_dtypes(
		include=['category']
	).columns
	categorical_transformer = Pipeline(
		steps=[
			('imputer', SimpleImputer(strategy='constant', fill_value='missing')),
			('onehot', OneHotEncoder(handle_unknown='ignore'))
	])
	preprocessor = ColumnTransformer(
		transformers=[
			('num', numeric_transformer, numeric_features),
			('cat', categorical_transformer, categorical_features)
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
elif args.library == 'tensorflow':
	pass

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
	'memory': memory
}))
