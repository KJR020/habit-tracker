//! Habit Tracker - macOS向け個人作業トラッキングツール

mod capture;
mod cli;
mod config;
mod database;
mod error;
mod image_store;
mod logging;
mod metadata;
mod pause_control;
mod report;

use anyhow::Result;

fn main() -> Result<()> {
    logging::init();
    cli::run()
}
