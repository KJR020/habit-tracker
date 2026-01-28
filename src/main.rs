//! Habit Tracker - macOS向け個人作業トラッキングツール
//!
//! ## Module Organization
//! - `capture/` - Screen capture domain (capture_loop, image_store, metadata, ocr)
//! - `data/` - Data persistence and reporting (database, report)
//! - `infra/` - Infrastructure concerns (config, error, logging, pause_control)
//! - `cli` - Command-line interface

mod capture;
mod cli;
mod data;
mod infra;

use anyhow::Result;

fn main() -> Result<()> {
    infra::logging::init();
    cli::run()
}
