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
    
    /// Get the file extension for this format
    fn file_extension(&self) -> &'static str;
    
    /// Get the MIME type for this format
    fn mime_type(&self) -> &'static str;
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
mod utils;