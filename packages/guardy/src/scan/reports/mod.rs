//! V3 Reporting System
//!
//! Generates reports from scan results with pluggable formats

use anyhow::Result;
use crate::scan::data::ScanResult;

/// Core reporting trait - allows pluggable report formats
pub trait ReportGenerator {
    /// Generate report content as a string
    fn generate(
        &self,
        result: &ScanResult,
        config: &ReportConfig,
        metadata: &ReportMetadata,
    ) -> Result<String>;
}

// Public re-exports
pub use orchestrator::ReportOrchestrator;
pub use config::{ReportConfig, ReportFormat, ReportMetadata, RedactionStyle};
pub use json::JsonReportGenerator;
pub use html::HtmlReportGenerator;

// Module declarations
mod orchestrator;
mod config;
mod aggregator;
mod json;
mod html;
pub mod utils;