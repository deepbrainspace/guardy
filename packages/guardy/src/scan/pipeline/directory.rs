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
    /// Rayon automatically handles work distribution - setting RAYON_NUM_THREADS=1 
    /// or configuring max_threads=1 will effectively make this sequential
    pub fn process_files(
        &self,
        files: Vec<PathBuf>,
        file_pipeline: Arc<super::FilePipeline>,
        progress: Option<&ProgressTracker>,
    ) -> Result<Vec<FileResult>> {
        use rayon::prelude::*;
        
        // Configure rayon thread pool if specified
        // This is a one-time setup that affects the global thread pool
        if let Some(max_threads) = self.config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build_global()
                .ok(); // Ignore if already set
        }
        
        // Always use par_iter - rayon handles optimization
        // With 1 thread, this effectively becomes sequential
        // With multiple threads, rayon's work-stealing provides optimal distribution
        Ok(files
            .par_iter()
            .map(|file_path| {
                // Use Arc::from for zero-copy string sharing across threads
                let file_path_str = Arc::from(file_path.to_string_lossy().as_ref());
                
                // Update progress atomically if available
                if let Some(p) = progress {
                    p.increment_files_processed();
                }
                
                // Process the file and handle errors gracefully
                match file_pipeline.process_file(file_path) {
                    Ok(result) => result,
                    Err(e) => FileResult::failure(file_path_str, e.to_string()),
                }
            })
            .collect())
    }
}