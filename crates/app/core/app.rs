//! The App presents a Rust API for interacting with Tangram.

use crate::{options, storage};
use anyhow::Result;

pub struct App {
	pub database_pool: sqlx::AnyPool,
	pub options: options::Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: self::storage::Storage,
}

impl App {
	pub fn create_monitor() -> Result<()> {
		todo!()
	}
}
