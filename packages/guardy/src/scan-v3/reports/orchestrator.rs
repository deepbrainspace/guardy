//! Report orchestration and file management

use super::{ReportGenerator, ReportConfig, ReportFormat, ReportMetadata};
use super::{JsonReportGenerator, HtmlReportGenerator};
use crate::scan_v3::data::ScanResult;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// High-level report orchestrator - main entry point for report generation
pub struct ReportOrchestrator;

impl ReportOrchestrator {
    /// Generate a report and save it to the specified location
    pub fn generate_report(
        result: &ScanResult,
        format: ReportFormat,
        output_path: Option<PathBuf>,
        output_dir: Option<PathBuf>,
        config: ReportConfig,
    ) -> Result<PathBuf> {
        let metadata = ReportMetadata::new(result.stats.scan_duration_ms);
        
        let generator: Box<dyn ReportGenerator> = match format {
            ReportFormat::Json => Box::new(JsonReportGenerator),
            ReportFormat::Html => Box::new(HtmlReportGenerator),
        };
        
        let content = generator.generate(result, &config, &metadata)?;
        
        // Resolve output path
        let report_path = Self::resolve_output_path(output_path, output_dir, format, config.display_secrets)?;
        
        // Ensure parent directory exists
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            
            // Ensure .gitignore exists in report directory
            Self::ensure_gitignored(parent)?;
        }
        
        // Write report
        fs::write(&report_path, content)
            .with_context(|| format!("Failed to write report: {}", report_path.display()))?;
        
        Ok(report_path)
    }
    
    /// Resolve the final output path based on user preferences
    fn resolve_output_path(
        report_output: Option<PathBuf>,    // --report-output
        report_dir: Option<PathBuf>,       // --report-dir
        format: ReportFormat,
        display_secrets: bool,
    ) -> Result<PathBuf> {
        match (report_output, report_dir) {
            // Explicit file path
            (Some(path), _) => Ok(path),
            
            // Explicit directory
            (None, Some(dir)) => Ok(dir.join(Self::generate_filename(format, display_secrets))),
            
            // Default: safe location
            (None, None) => {
                let default_dir = Self::get_default_report_dir()?;
                Ok(default_dir.join(Self::generate_filename(format, display_secrets)))
            }
        }
    }
    
    /// Generate filename with timestamp and safety indicator
    fn generate_filename(format: ReportFormat, display_secrets: bool) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let safety_suffix = if display_secrets { "sensitive" } else { "redacted" };
        let extension = format.extension();
        
        format!("guardy-report-{timestamp}.{safety_suffix}.{extension}")
    }
    
    /// Get default report directory with fallback options
    fn get_default_report_dir() -> Result<PathBuf> {
        // Try .guardy/reports/ first
        let guardy_dir = PathBuf::from(".guardy").join("reports");
        if guardy_dir.parent().is_none() || guardy_dir.parent().is_some_and(|p| p.exists() || fs::create_dir_all(p).is_ok()) {
            return Ok(guardy_dir);
        }
        
        // Fallback to guardy-reports/
        let fallback_dir = PathBuf::from("guardy-reports");
        if fs::create_dir_all(&fallback_dir).is_ok() {
            return Ok(fallback_dir);
        }
        
        // Final fallback to current directory
        Ok(PathBuf::from("."))
    }
    
    /// Ensure .gitignore exists in the report directory
    fn ensure_gitignored(report_dir: &Path) -> Result<()> {
        let gitignore_path = report_dir.join(".gitignore");
        if !gitignore_path.exists() {
            let content = "# Guardy reports - auto-generated\n*.sensitive.*\n";
            fs::write(&gitignore_path, content)
                .with_context(|| format!("Failed to create .gitignore: {}", gitignore_path.display()))?;
        }
        Ok(())
    }
}