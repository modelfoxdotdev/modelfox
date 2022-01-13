use tangram_app_core::App;

pub struct Context {
	pub app: App,
	pub sunfish: sunfish::Sunfish,
	pub feature_flags: Vec<FeatureFlag>,
}

impl Context {
	pub fn new(app: App, sunfish: sunfish::Sunfish) -> Self {
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

pub struct FeatureFlag;
