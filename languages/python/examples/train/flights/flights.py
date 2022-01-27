from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost', 'catboost'], required=True)
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
if args.library == 'xgboost' or args.library == 'sklearn' or args.library == 'catboost':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
labels_train = data_train.pop(target_column_name)
features_train = data_train
labels_test = data_test.pop(target_column_name)
features_test = data_test

# Train the model.
if args.library == 'h2o':
	import h2o
	from h2o.estimators import H2OGradientBoostingEstimator
	h2o.init(max_mem_size=20480000 * 1000)
	data_train = pd.concat([features_train, labels_train], axis=1)
	data_test = pd.concat([features_test, labels_test], axis=1)
	data_train = h2o.H2OFrame(python_obj=data_train)
	data_test = h2o.H2OFrame(python_obj=data_test)
	feature_column_names = [column for column in data_train.columns if column != target_column_name]
	model = H2OGradientBoostingEstimator(
		distribution="bernoulli",
		learn_rate=0.1,
		nbins=255,
		ntrees=100,
	)
	model.train(
		training_frame=data_train,
		x=feature_column_names,
		y=target_column_name,
	)
elif args.library == 'lightgbm':
	import lightgbm as lgb
	model = lgb.LGBMClassifier(
		force_row_wise=True,
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(
		features_train,
		labels_train,
	)
elif args.library == 'sklearn':
	from sklearn.experimental import enable_hist_gradient_boosting
	from sklearn.ensemble import HistGradientBoostingClassifier
	model = HistGradientBoostingClassifier(
		learning_rate=0.1,
		max_iter=100,
		max_leaf_nodes=255,
		validation_fraction=None,
	)
	model.fit(features_train, labels_train)
elif args.library == 'xgboost':
	import xgboost as xgb
	model = xgb.XGBClassifier(
		eta=0.1,
		eval_metric='logloss',
		grow_policy='lossguide',
		n_estimators=100,
		tree_method='hist',
		max_depth=0,
		max_leaves=255,
		use_label_encoder=False
	)
	model.fit(features_train, labels_train)
elif args.library == 'catboost':
	from catboost import CatBoostClassifier
	categorical_columns = [column for column in categorical_columns if column != target_column_name]
	model = CatBoostClassifier(
		cat_features=categorical_columns,
		grow_policy='Lossguide',
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
		train_dir='data/catboost_info',
		verbose=False
	)
	model.fit(features_train, labels_train, silent=True)

# Make predictions on the test data.
if args.library == 'h2o':
	predictions_proba = model.predict(data_test).as_data_frame()['Y']
else:
	predictions_proba = model.predict_proba(features_test)[:, 1]

# Compute metrics.
auc_roc = roc_auc_score(labels_test, predictions_proba)

print(json.dumps({
	'auc_roc': auc_roc,
}))