//! File content processing pipeline

use crate::scan::{
    config::ScannerConfig,
    data::{FileResult, SecretMatch},
    filters::content::{ContextPrefilter, RegexExecutor, RegexInput},
    filters::Filter,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Pipeline for processing file contents with optimized two-stage detection
pub struct FilePipeline {
    config: Arc<ScannerConfig>,
    prefilter: ContextPrefilter,
    regex_executor: RegexExecutor,
}

impl FilePipeline {
    /// Create a new file pipeline
    pub fn new(config: Arc<ScannerConfig>) -> Result<Self> {
        let prefilter = ContextPrefilter::new();
        let regex_executor = RegexExecutor::new();
        
        Ok(Self { 
            config,
            prefilter,
            regex_executor,
        })
    }
    
    /// Process a single file through the content pipeline
    pub fn process_file(&self, path: &Path) -> Result<FileResult> {
        let start_time = Instant::now();
        let file_path = Arc::from(path.to_string_lossy().as_ref());
        
        // Read file contents with UTF-8 validation
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                // Handle read errors gracefully - could be binary file that passed filters
                // or permission issues, or actual I/O errors
                let error_msg = if e.kind() == std::io::ErrorKind::InvalidData {
                    "File contains invalid UTF-8 (likely binary)"
                } else {
                    "Failed to read file"
                };
                
                return Ok(FileResult::failure(
                    file_path,
                    format!("{}: {}", error_msg, e)
                ));
            }
        };
        
        // Get file metadata for statistics
        let metadata = fs::metadata(path)
            .context("Failed to read file metadata")?;
        let file_size = metadata.len();
        
        // Count lines for statistics
        let lines_processed = content.lines().count();
        
        // Stage 1: Aho-Corasick prefilter to eliminate ~85% of patterns
        let active_patterns = match self.prefilter.filter(&content) {
            Ok(patterns) => patterns,
            Err(e) => {
                return Ok(FileResult::failure(
                    file_path,
                    format!("Prefilter failed: {}", e)
                ));
            }
        };
        
        // If no patterns matched, file is clean
        if active_patterns.is_empty() {
            let scan_time_ms = start_time.elapsed().as_millis() as u64;
            return Ok(FileResult::success(
                file_path,
                Vec::new(),
                lines_processed,
                file_size,
                scan_time_ms,
            ));
        }
        
        // Stage 2: Run regex executor on filtered patterns only
        let regex_input = RegexInput {
            content,
            file_path: file_path.clone(),
            active_patterns,
        };
        
        let matches = match self.regex_executor.filter(&regex_input) {
            Ok(matches) => matches,
            Err(e) => {
                return Ok(FileResult::failure(
                    file_path,
                    format!("Regex execution failed: {}", e)
                ));
            }
        };
        
        let scan_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(FileResult::success(
            file_path,
            matches,
            lines_processed,
            file_size,
            scan_time_ms,
        ))
    }
}