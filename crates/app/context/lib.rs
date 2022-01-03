use std::sync::Arc;
use tangram_app_core::App;

pub struct Context {
	pub app: Arc<App>,
	pub sunfish: sunfish::Sunfish,
	pub feature_flags: Vec<FeatureFlag>,
}

impl Context {
	pub fn new(app: Arc<App>, sunfish: sunfish::Sunfish) -> Self {
		Context {
			app,
			sunfish,
			feature_flags: Vec::new(),
		}
	}

	pub fn add_feature_flag(&mut self, flag: FeatureFlag) {
		self.feature_flags.push(flag);
	}
}

/// FIXME - this will be an enum of feature flags
pub struct FeatureFlag;
