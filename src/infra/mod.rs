//! Infrastructure - cross-cutting concerns
//!
//! This module contains:
//! - `config` - Configuration management
//! - `error` - Error type definitions
//! - `logging` - Logging infrastructure
//! - `pause_control` - Pause/resume control

pub mod config;
pub mod error;
pub mod logging;
mod pause_control;

pub use config::{CliArgs, Config};
pub use pause_control::PauseControl;
