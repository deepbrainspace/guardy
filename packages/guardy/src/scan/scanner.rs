//! Main Scanner implementation

use crate::scan::{
    config::ScannerConfig,
    data::{ScanResult, ScanStats},
    pipeline::{DirectoryPipeline, FilePipeline},
    tracking::ProgressTracker,
};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Main scanner orchestrator
pub struct Scanner {
    config: Arc<ScannerConfig>,
    directory_pipeline: DirectoryPipeline,
    file_pipeline: Arc<FilePipeline>,
}

impl Scanner {
    /// Create a new scanner with the given configuration
    pub fn new(config: ScannerConfig) -> Result<Self> {
        let config = Arc::new(config);
        
        // Initialize pipelines
        let directory_pipeline = DirectoryPipeline::new(config.clone())?;
        let file_pipeline = Arc::new(FilePipeline::new(config.clone())?);
        
        Ok(Self {
            config,
            directory_pipeline,
            file_pipeline,
        })
    }
    
    /// Scan a path (file or directory) for secrets
    pub fn scan(&self, path: &Path) -> Result<ScanResult> {
        let start = Instant::now();
        
        // Create progress tracker
        let progress = ProgressTracker::new();
        
        // Discover files
        progress.set_stage("Discovering files");
        let files = self.directory_pipeline.discover_files(path)?;
        
        // Process files
        progress.set_stage("Scanning files");
        let file_results = self.directory_pipeline.process_files(
            files,
            self.file_pipeline.clone(),
            Some(&progress),
        )?;
        
        // Aggregate results
        let mut all_matches = Vec::new();
        let mut warnings = Vec::new();
        let mut stats = ScanStats::new();
        
        for result in &file_results {
            if result.success {
                all_matches.extend(result.matches.clone());
                stats.files_scanned += 1;
                stats.total_bytes_processed += result.file_size;
                stats.total_lines_processed += result.lines_processed;
            } else {
                stats.files_failed += 1;
                if let Some(ref error) = result.error {
                    warnings.push(format!("{}: {}", result.file_path, error));
                }
            }
        }
        
        stats.total_matches = all_matches.len();
        stats.scan_duration_ms = start.elapsed().as_millis() as u64;
        
        // Count severity levels
        for match_ in &all_matches {
            match match_.severity {
                crate::scan::data::MatchSeverity::Critical => stats.critical_matches += 1,
                crate::scan::data::MatchSeverity::High => stats.high_severity_matches += 1,
                _ => {}
            }
        }
        
        Ok(ScanResult::new(
            all_matches,
            stats,
            file_results,
            warnings,
        ))
    }
}