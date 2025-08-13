//! File content processing pipeline

use crate::scan::{
    config::ScannerConfig,
    data::{FileResult, StatsCollector},
    filters::content::{CommentFilter, CommentFilterInput, ContextPrefilter, EntropyFilter, RegexExecutor, RegexInput},
    filters::Filter,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tracing;

/// Pipeline for processing file contents with optimized four-stage detection
pub struct FilePipeline {
    config: Arc<ScannerConfig>,
    prefilter: ContextPrefilter,
    regex_executor: RegexExecutor,
    comment_filter: CommentFilter,
    entropy_filter: EntropyFilter,
}

impl FilePipeline {
    /// Create a new file pipeline
    pub fn new(config: Arc<ScannerConfig>) -> Result<Self> {
        let prefilter = ContextPrefilter::new();
        let regex_executor = RegexExecutor::new();
        let comment_filter = CommentFilter::new();
        let entropy_filter = EntropyFilter::new(config.min_entropy_threshold);
        
        Ok(Self { 
            config,
            prefilter,
            regex_executor,
            comment_filter,
            entropy_filter,
        })
    }
    
    /// Process a single file through the content pipeline
    pub fn process_file(&self, path: &Path, stats: Arc<StatsCollector>) -> Result<FileResult> {
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
                
                stats.increment_files_failed();
                return Ok(FileResult::failure(
                    file_path,
                    format!("{error_msg}: {e}")
                ));
            }
        };
        
        // Get file metadata for statistics
        let metadata = fs::metadata(path)
            .context("Failed to read file metadata")?;
        let file_size = metadata.len();
        
        // Count lines for statistics and update stats
        let lines_processed = content.lines().count();
        stats.increment_files_scanned();
        stats.add_bytes_processed(file_size);
        stats.add_lines_processed(lines_processed);
        
        // Stage 1: Aho-Corasick prefilter to eliminate ~85% of patterns
        let active_patterns = match self.prefilter.filter(&content) {
            Ok(patterns) => {
                tracing::debug!("Filter '{}' found {} active patterns", self.prefilter.name(), patterns.len());
                patterns
            },
            Err(e) => {
                stats.increment_files_failed();
                tracing::warn!("Filter '{}' failed: {}", self.prefilter.name(), e);
                return Ok(FileResult::failure(
                    file_path,
                    format!("{} failed: {}", self.prefilter.name(), e)
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
        
        let mut matches = match self.regex_executor.filter(&regex_input) {
            Ok(matches) => {
                tracing::debug!("Filter '{}' found {} potential matches", self.regex_executor.name(), matches.len());
                matches
            },
            Err(e) => {
                tracing::warn!("Filter '{}' failed: {}", self.regex_executor.name(), e);
                return Ok(FileResult::failure(
                    file_path,
                    format!("{} failed: {}", self.regex_executor.name(), e)
                ));
            }
        };
        
        // Stage 3: Comment filtering - remove matches ignored by comments
        if !matches.is_empty() && !self.config.no_entropy {
            let original_count = matches.len();
            let comment_input = CommentFilterInput {
                file_content: regex_input.content.clone(),
                matches: matches.clone(),
            };
            
            matches = match self.comment_filter.filter(&comment_input) {
                Ok(filtered_matches) => {
                    let filtered_count = original_count - filtered_matches.len();
                    if filtered_count > 0 {
                        tracing::debug!("Filter '{}' removed {} matches via comments", self.comment_filter.name(), filtered_count);
                        stats.add_matches_filtered_by_comments(filtered_count);
                    }
                    filtered_matches
                },
                Err(e) => {
                    stats.increment_files_failed();
                    tracing::warn!("Filter '{}' failed: {}", self.comment_filter.name(), e);
                    return Ok(FileResult::failure(
                        file_path,
                        format!("{} failed: {}", self.comment_filter.name(), e)
                    ));
                }
            };
        }
        
        // Stage 4: Entropy filtering - validate randomness of potential secrets
        if !matches.is_empty() && !self.config.no_entropy {
            let original_count = matches.len();
            let mut entropy_filtered = Vec::new();
            for secret_match in matches {
                if self.entropy_filter.is_likely_secret(secret_match.matched_text.as_bytes()) {
                    entropy_filtered.push(secret_match);
                }
            }
            let filtered_count = original_count - entropy_filtered.len();
            if filtered_count > 0 {
                tracing::debug!("Entropy filter removed {} low-entropy matches", filtered_count);
                stats.add_matches_filtered_by_entropy(filtered_count);
            }
            matches = entropy_filtered;
        }
        
        // Update final match count
        stats.add_matches(matches.len());
        
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