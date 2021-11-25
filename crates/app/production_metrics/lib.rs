pub use self::{
	binary_classification_production_metrics::{
		BinaryClassificationProductionPredictionMetrics,
		BinaryClassificationProductionPredictionMetricsOutput,
	},
	multiclass_classification_production_metrics::{
		MulticlassClassificationProductionPredictionMetrics,
		MulticlassClassificationProductionPredictionMetricsOutput,
	},
	regression_production_metrics::{
		RegressionProductionPredictionMetrics, RegressionProductionPredictionMetricsOutput,
	},
};
use anyhow::Result;
use chrono::prelude::*;
use chrono_tz::Tz;
use num::ToPrimitive;
use sqlx::prelude::*;
use tangram_app_common::monitor_event::NumberOrString;
use tangram_app_ui::date_window::{DateWindow, DateWindowInterval};

mod binary_classification_production_metrics;
mod multiclass_classification_production_metrics;
mod regression_production_metrics;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: ProductionPredictionMetrics,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ProductionPredictionMetrics {
	#[serde(rename = "regression")]
	Regression(RegressionProductionPredictionMetrics),
	#[serde(rename = "binary_classification")]
	BinaryClassification(BinaryClassificationProductionPredictionMetrics),
	#[serde(rename = "multiclass_classification")]
	MulticlassClassification(MulticlassClassificationProductionPredictionMetrics),
}

pub struct ProductionMetricsOutput {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: Option<ProductionPredictionMetricsOutput>,
}

pub enum ProductionPredictionMetricsOutput {
	Regression(RegressionProductionPredictionMetricsOutput),
	BinaryClassification(BinaryClassificationProductionPredictionMetricsOutput),
	MulticlassClassification(MulticlassClassificationProductionPredictionMetricsOutput),
}

impl ProductionMetrics {
	pub fn new(
		model: tangram_model::ModelReader,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> ProductionMetrics {
		let prediction_metrics = ProductionPredictionMetrics::new(model);
		ProductionMetrics {
			start_date,
			end_date,
			true_values_count: 0,
			prediction_metrics,
		}
	}

	pub fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		self.true_values_count += 1;
		self.prediction_metrics.update(value);
	}

	pub fn merge(&mut self, other: ProductionMetrics) {
		self.start_date = self.start_date.min(other.start_date);
		self.end_date = self.end_date.max(other.end_date);
		self.prediction_metrics.merge(other.prediction_metrics);
		self.true_values_count += other.true_values_count;
	}

	pub fn finalize(self) -> ProductionMetricsOutput {
		ProductionMetricsOutput {
			start_date: self.start_date,
			end_date: self.end_date,
			true_values_count: self.true_values_count,
			prediction_metrics: self.prediction_metrics.finalize(),
		}
	}
}

impl ProductionPredictionMetrics {
	pub fn new(model: tangram_model::ModelReader) -> ProductionPredictionMetrics {
		match model.inner() {
			tangram_model::ModelInnerReader::Regressor(_) => {
				ProductionPredictionMetrics::Regression(RegressionProductionPredictionMetrics::new())
			}
			tangram_model::ModelInnerReader::BinaryClassifier(model) => {
				let model = model.read();
				ProductionPredictionMetrics::BinaryClassification(
					BinaryClassificationProductionPredictionMetrics::new(
						model.negative_class().to_owned(),
						model.positive_class().to_owned(),
					),
				)
			}
			tangram_model::ModelInnerReader::MulticlassClassifier(model) => {
				let model = model.read();
				ProductionPredictionMetrics::MulticlassClassification(
					MulticlassClassificationProductionPredictionMetrics::new(
						model.classes().iter().map(ToOwned::to_owned).collect(),
					),
				)
			}
		}
	}

	pub fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		match self {
			ProductionPredictionMetrics::Regression(s) => s.update(value),
			ProductionPredictionMetrics::BinaryClassification(s) => s.update(value),
			ProductionPredictionMetrics::MulticlassClassification(s) => s.update(value),
		}
	}

	pub fn merge(&mut self, other: ProductionPredictionMetrics) {
		match self {
			ProductionPredictionMetrics::Regression(s) => {
				if let ProductionPredictionMetrics::Regression(other) = other {
					s.merge(other)
				}
			}
			ProductionPredictionMetrics::BinaryClassification(s) => {
				if let ProductionPredictionMetrics::BinaryClassification(other) = other {
					s.merge(other)
				}
			}
			ProductionPredictionMetrics::MulticlassClassification(s) => {
				if let ProductionPredictionMetrics::MulticlassClassification(other) = other {
					s.merge(other)
				}
			}
		}
	}

	pub fn finalize(self) -> Option<ProductionPredictionMetricsOutput> {
		match self {
			ProductionPredictionMetrics::Regression(s) => s
				.finalize()
				.map(ProductionPredictionMetricsOutput::Regression),
			ProductionPredictionMetrics::BinaryClassification(s) => s
				.finalize()
				.map(ProductionPredictionMetricsOutput::BinaryClassification),
			ProductionPredictionMetrics::MulticlassClassification(s) => s
				.finalize()
				.map(ProductionPredictionMetricsOutput::MulticlassClassification),
		}
	}
}

pub struct GetProductionMetricsOutput {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionMetricsOutput,
	pub intervals: Vec<ProductionMetricsOutput>,
}

pub async fn get_production_metrics(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: tangram_model::ModelReader<'_>,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Result<GetProductionMetricsOutput> {
	// Compute the start date given the date window.
	// * For today, use the start of this day in this timezone.
	// * For this month, use the start of this month in this timezone.
	// * For this year, use the start of this year in this timezone.
	let now: DateTime<Tz> = Utc::now().with_timezone(&timezone);
	let start_date = match date_window {
		DateWindow::Today => timezone
			.ymd(now.year(), now.month(), now.day())
			.and_hms(0, 0, 0),
		DateWindow::ThisMonth => timezone.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0),
		DateWindow::ThisYear => timezone.ymd(now.year(), 1, 1).and_hms(0, 0, 0),
	};
	let end_date = match date_window {
		DateWindow::Today => start_date + chrono::Duration::days(1),
		DateWindow::ThisMonth => {
			start_date
				+ chrono::Duration::days(n_days_in_month(start_date.year(), start_date.month()))
		}
		DateWindow::ThisYear => timezone.ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0),
	};
	let rows = sqlx::query(
		"
			select
				data,
				hour
			from production_metrics
			where
				model_id = $1 and
				hour >= $2 and
				hour < $3
			order by hour
		",
	)
	.bind(&model.id().to_string())
	.bind(&start_date.timestamp())
	.bind(&end_date.timestamp())
	.fetch_all(&mut *db)
	.await?;
	/*
	 Compute the number of intervals.
	 * For today, use 24.
	 * For this month, use the number of days in this month.
	 * For this year, use 12.
	*/
	let n_intervals: usize = match date_window_interval {
		DateWindowInterval::Hourly => 24,
		DateWindowInterval::Daily => n_days_in_month(start_date.year(), start_date.month())
			.to_usize()
			.unwrap(),
		DateWindowInterval::Monthly => 12,
	};
	let mut intervals: Vec<ProductionMetrics> = (0..n_intervals.to_u32().unwrap())
		.map(|i| {
			// Determine the start and end dates for the interval.
			let start = match date_window_interval {
				DateWindowInterval::Hourly => start_date.with_hour(i).unwrap(),
				DateWindowInterval::Daily => start_date.with_day0(i).unwrap(),
				DateWindowInterval::Monthly => start_date.with_month0(i).unwrap(),
			};
			let end = match date_window_interval {
				DateWindowInterval::Hourly => start + chrono::Duration::hours(1),
				DateWindowInterval::Daily => start + chrono::Duration::days(1),
				DateWindowInterval::Monthly => {
					start + chrono::Duration::days(n_days_in_month(start.year(), start.month()))
				}
			};
			ProductionMetrics::new(model, start.with_timezone(&Utc), end.with_timezone(&Utc))
		})
		.collect();
	// Merge each hourly production metrics entry into its corresponding interval.
	for row in rows {
		let data: String = row.get(0);
		let hour: i64 = row.get(1);
		let hour = timezone.timestamp(hour, 0);
		let interval = match date_window_interval {
			DateWindowInterval::Hourly => {
				let hour = hour.hour().to_usize().unwrap();
				intervals.get_mut(hour).unwrap()
			}
			DateWindowInterval::Daily => {
				let day = hour.day0().to_usize().unwrap();
				intervals.get_mut(day).unwrap()
			}
			DateWindowInterval::Monthly => {
				let month = hour.month0().to_usize().unwrap();
				intervals.get_mut(month).unwrap()
			}
		};
		let hourly_production_metrics: ProductionMetrics = serde_json::from_str(&data).unwrap();
		interval.merge(hourly_production_metrics);
	}
	let overall = intervals
		.iter()
		.fold(
			ProductionMetrics::new(
				model,
				start_date.with_timezone(&Utc),
				end_date.with_timezone(&Utc),
			),
			|mut metrics, next| {
				metrics.merge(next.clone());
				metrics
			},
		)
		.finalize();
	let intervals: Vec<ProductionMetricsOutput> = intervals
		.into_iter()
		.map(|metrics| metrics.finalize())
		.collect();
	Ok(GetProductionMetricsOutput {
		date_window,
		date_window_interval,
		overall,
		intervals,
	})
}

fn n_days_in_month(year: i32, month: u32) -> i64 {
	let (end_year, end_month) = if month == 12 {
		(year + 1, 1)
	} else {
		(year, month + 1)
	};
	let start = NaiveDate::from_ymd(year, month, 1);
	let end = NaiveDate::from_ymd(end_year, end_month, 1);
	(end - start).num_days()
}
