//! Capture domain - screen capture and metadata collection
//!
//! This module contains:
//! - `CaptureLoop` - Main capture loop orchestration
//! - `ImageStore` - Screenshot capture and storage
//! - `Metadata` - macOS metadata collection
//! - `ocr` - OCR text recognition

mod capture_loop;
mod image_store;
mod metadata;
pub mod ocr;

pub use capture_loop::CaptureLoop;
pub use image_store::ImageStore;
pub use metadata::Metadata;
