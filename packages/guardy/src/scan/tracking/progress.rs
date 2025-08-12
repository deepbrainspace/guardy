//! Progress tracking with indicatif

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// Progress tracker for scan operations
/// 
/// Will be enhanced with indicatif progress bars in later phase
pub struct ProgressTracker {
    files_processed: Arc<AtomicUsize>,
    current_stage: Arc<RwLock<String>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            files_processed: Arc::new(AtomicUsize::new(0)),
            current_stage: Arc::new(RwLock::new("Initializing".to_string())),
        }
    }
    
    /// Set the current stage
    pub fn set_stage(&self, stage: &str) {
        if let Ok(mut stage_lock) = self.current_stage.write() {
            *stage_lock = stage.to_string();
        }
    }
    
    /// Increment files processed counter
    pub fn increment_files_processed(&self) {
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current files processed count
    pub fn files_processed(&self) -> usize {
        self.files_processed.load(Ordering::Relaxed)
    }
}