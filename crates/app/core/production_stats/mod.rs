use std::borrow::BorrowMut;

pub use self::{column_stats::*, number_stats::*, prediction_stats::*};
use crate::monitor_event::PredictionMonitorEvent;
use anyhow::Result;
use num::ToPrimitive;
use sqlx::prelude::*;
use tangram_app_ui::date_window::{DateWindow, DateWindowInterval};
use tangram_zip::zip;

mod column_stats;
mod number_stats;
mod prediction_stats;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ProductionStats {
	pub start_date: time::OffsetDateTime,
	pub end_date: time::OffsetDateTime,
	pub row_count: u64,
	pub column_stats: Vec<ProductionColumnStats>,
	pub prediction_stats: ProductionPredictionStats,
}

#[derive(Debug)]
pub struct ProductionStatsOutput {
	pub start_date: time::OffsetDateTime,
	pub end_date: time::OffsetDateTime,
	pub row_count: u64,
	pub column_stats: Vec<ProductionColumnStatsOutput>,
	pub prediction_stats: ProductionPredictionStatsOutput,
}

impl ProductionStats {
	pub fn new(
		model: tangram_model::ModelReader,
		start_date: time::OffsetDateTime,
		end_date: time::OffsetDateTime,
	) -> ProductionStats {
		let train_column_stats = match model.inner() {
			tangram_model::ModelInnerReader::Regressor(regressor) => {
				regressor.read().train_column_stats()
			}
			tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
				binary_classifier.read().train_column_stats()
			}
			tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
				multiclass_classifier.read().train_column_stats()
			}
		};
		let column_stats = train_column_stats
			.iter()
			.map(|column_stats| ProductionColumnStats::new(&column_stats))
			.collect();
		let prediction_stats = ProductionPredictionStats::new(model);
		ProductionStats {
			start_date,
			end_date,
			row_count: 0,
			column_stats,
			prediction_stats,
		}
	}

	pub fn update(&mut self, model: tangram_model::ModelReader, value: PredictionMonitorEvent) {
		self.row_count += 1;
		for column_stats in self.column_stats.iter_mut() {
			let value = value.input.get(column_stats.column_name());
			column_stats.update(model, value);
		}
		self.prediction_stats.update(value.output);
	}

	pub fn merge(&mut self, other: ProductionStats) {
		self.start_date = self.start_date.min(other.start_date);
		self.end_date = self.end_date.max(other.end_date);
		self.row_count += other.row_count;
		for (this, other) in zip!(self.column_stats.iter_mut(), other.column_stats) {
			this.merge(other)
		}
		self.prediction_stats.merge(other.prediction_stats);
	}

	pub fn finalize(self) -> ProductionStatsOutput {
		ProductionStatsOutput {
			start_date: self.start_date,
			end_date: self.end_date,
			row_count: self.row_count,
			column_stats: self
				.column_stats
				.into_iter()
				.map(|c| c.finalize())
				.collect(),
			prediction_stats: self.prediction_stats.finalize(),
		}
	}
}

#[derive(Debug)]
pub struct GetProductionStatsOutput {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionStatsOutput,
	pub intervals: Vec<ProductionStatsOutput>,
}

pub async fn get_production_stats(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	model: tangram_model::ModelReader<'_>,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	offset: time::UtcOffset,
) -> Result<GetProductionStatsOutput> {
	// Compute the start date given the date window.
	// * For today, use the start of this day in this timezone.
	// * For this month, use the start of this month in this timezone.
	// * For this year, use the start of this year in this timezone.
	let now = time::OffsetDateTime::now_utc().to_offset(offset);
	let start_date = match date_window {
		DateWindow::Today => timezone
			.ymd(now.year(), now.month(), now.day())
			.and_hms(0, 0, 0),
		DateWindow::ThisMonth => timezone.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0),
		DateWindow::ThisYear => timezone.ymd(now.year(), 1, 1).and_hms(0, 0, 0),
	};
	let end_date = match date_window {
		DateWindow::Today => start_date + time::Duration::days(1),
		DateWindow::ThisMonth => {
			let days_in_month =
				time::util::days_in_year_month(start_date.year(), start_date.month());
			start_date + time::Duration::days(days_in_month.into())
		}
		DateWindow::ThisYear => timezone.ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0),
	};
	// Retrieve the production stats for the date window.
	let rows = sqlx::query(
		"
			select
				data,
				hour
			from production_stats
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
	.fetch_all(txn.borrow_mut())
	.await?;
	// Compute the number of intervals.
	// * For today, use 24.
	// * For this month, use the number of days in this month.
	// * For this year, use 12.
	let n_intervals: usize = match date_window_interval {
		DateWindowInterval::Hourly => 24,
		DateWindowInterval::Daily => n_days_in_month(start_date.year(), start_date.month())
			.to_usize()
			.unwrap(),
		DateWindowInterval::Monthly => 12,
	};
	// Initialize the intervals with start and end dates.
	let mut intervals: Vec<ProductionStats> = (0..n_intervals.to_u32().unwrap())
		.map(|i| {
			// Determine the start and end dates for the interval.
			let start = match date_window_interval {
				DateWindowInterval::Hourly => start_date.with_hour(i).unwrap(),
				DateWindowInterval::Daily => start_date.with_day0(i).unwrap(),
				DateWindowInterval::Monthly => start_date.with_month0(i).unwrap(),
			};
			let end = match date_window_interval {
				DateWindowInterval::Hourly => start + time::Duration::hours(1),
				DateWindowInterval::Daily => start + time::Duration::days(1),
				DateWindowInterval::Monthly => {
					let days_in_month = time::util::days_in_year_month(start.year(), start.month());
					start + time::Duration::days(days_in_month.into())
				}
			};
			ProductionStats::new(
				model,
				start.to_offset(time::UtcOffset::UTC),
				end.to_offset(time::UtcOffset::UTC),
			)
		})
		.collect();
	// Merge each hourly production stats entry into its corresponding interval.
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
		let hourly_production_stats = serde_json::from_str(&data)?;
		interval.merge(hourly_production_stats);
	}
	// Compute the overall production stats by merging all the intervals together.
	let overall = intervals
		.iter()
		.fold(
			ProductionStats::new(
				model,
				start_date.to_offset(time::UtcOffset::UTC),
				end_date.to_offset(time::UtcOffset::UTC),
			),
			|mut stats, next| {
				stats.merge(next.clone());
				stats
			},
		)
		.finalize();
	// Finalize the intervals.
	let intervals: Vec<ProductionStatsOutput> = intervals
		.into_iter()
		.map(|stats| stats.finalize())
		.collect();
	Ok(GetProductionStatsOutput {
		date_window,
		date_window_interval,
		overall,
		intervals,
	})
}
