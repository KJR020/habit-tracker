//! Data layer - persistence and reporting
//!
//! This module contains:
//! - `Database` - SQLite database operations
//! - `CaptureRecord` - Capture data transfer object
//! - `Report` - Report generation

mod database;
mod report;

pub use database::{CaptureRecord, Database};
pub use report::{AppSummary, Report, TimelineEntry};
