//! Directory traversal and file discovery pipeline

use crate::scan::{
    config::ScannerConfig,
    data::FileResult,
    tracking::ProgressTracker,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Pipeline for directory traversal and file filtering
pub struct DirectoryPipeline {
    config: Arc<ScannerConfig>,
}

impl DirectoryPipeline {
    /// Create a new directory pipeline
    pub fn new(config: Arc<ScannerConfig>) -> Result<Self> {
        Ok(Self { config })
    }
    
    /// Discover all files to scan from a path
    pub fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        // Placeholder implementation
        Ok(vec![])
    }
    
    /// Process files in parallel using rayon
    pub fn process_files(
        &self,
        files: Vec<PathBuf>,
        file_pipeline: Arc<super::FilePipeline>,
        progress: Option<&ProgressTracker>,
    ) -> Result<Vec<FileResult>> {
        use rayon::prelude::*;
        
        // Decide whether to use parallel processing
        let use_parallel = files.len() >= self.config.min_files_for_parallel;
        
        if use_parallel {
            // Set rayon thread pool if configured
            if let Some(max_threads) = self.config.max_threads {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(max_threads)
                    .build_global()
                    .ok(); // Ignore if already set
            }
            
            // Process files in parallel
            Ok(files
                .par_iter()
                .map(|file_path| {
                    let file_path_str = Arc::from(file_path.to_string_lossy().as_ref());
                    
                    // Update progress if available
                    if let Some(p) = progress {
                        p.increment_files_processed();
                    }
                    
                    // Process the file
                    match file_pipeline.process_file(file_path) {
                        Ok(result) => result,
                        Err(e) => FileResult::failure(file_path_str, e.to_string()),
                    }
                })
                .collect())
        } else {
            // Process sequentially for small file counts
            files
                .iter()
                .map(|file_path| {
                    let file_path_str = Arc::from(file_path.to_string_lossy().as_ref());
                    
                    if let Some(p) = progress {
                        p.increment_files_processed();
                    }
                    
                    match file_pipeline.process_file(file_path) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(FileResult::failure(file_path_str, e.to_string())),
                    }
                })
                .collect()
        }
    }
}