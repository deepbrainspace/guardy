use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Progress - Visual feedback system with indicatif integration
///
/// Responsibilities:
/// - Visual progress tracking with bars and statistics
/// - Coordinate between sequential and parallel execution progress display
/// - Bridge between scan module and existing indicatif progress system
/// - Provide clean OOP interfaces for progress updates
pub struct Progress {
    pub is_parallel: bool,
    multi_progress: Arc<MultiProgress>,
    overall_bar: ProgressBar,
    worker_bars: Vec<ProgressBar>,
    stats: Arc<ProgressStats>,
    start_time: Instant,
}

/// Statistics tracked during scanning progress
#[derive(Debug, Default)]
pub struct ProgressStats {
    files_scanned: Mutex<usize>,
    files_skipped: Mutex<usize>,
    files_with_secrets: Mutex<usize>,
    binary_files: Mutex<usize>,
}

impl ProgressStats {
    /// Increment the count of successfully scanned files
    pub fn increment_scanned(&self) {
        let mut count = self.files_scanned.lock().unwrap();
        *count += 1;
    }

    /// Increment the count of skipped files (due to errors)
    pub fn increment_skipped(&self) {
        let mut count = self.files_skipped.lock().unwrap();
        *count += 1;
    }

    /// Increment the count of files containing secrets
    pub fn increment_with_secrets(&self) {
        let mut count = self.files_with_secrets.lock().unwrap();
        *count += 1;
    }

    /// Increment the count of binary files detected
    pub fn increment_binary(&self) {
        let mut count = self.binary_files.lock().unwrap();
        *count += 1;
    }

    /// Get current statistics snapshot
    pub fn get_snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            files_scanned: *self.files_scanned.lock().unwrap(),
            files_skipped: *self.files_skipped.lock().unwrap(),
            files_with_secrets: *self.files_with_secrets.lock().unwrap(),
            binary_files: *self.binary_files.lock().unwrap(),
        }
    }
}

/// Snapshot of progress statistics at a point in time
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub files_with_secrets: usize,
    pub binary_files: usize,
}

impl Progress {
    /// Create a new sequential progress reporter
    pub fn new_sequential(total_files: usize) -> Result<Self> {
        let multi_progress = Arc::new(MultiProgress::new());
        
        // Create overall progress bar for sequential mode
        let overall_bar = ProgressBar::new(total_files as u64);
        overall_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        
        let overall_bar = multi_progress.add(overall_bar);

        Ok(Self {
            is_parallel: false,
            multi_progress,
            overall_bar,
            worker_bars: Vec::new(),
            stats: Arc::new(ProgressStats::default()),
            start_time: Instant::now(),
        })
    }

    /// Create a new parallel progress reporter with worker bars
    pub fn new_parallel(total_files: usize, worker_count: usize) -> Result<Self> {
        let multi_progress = Arc::new(MultiProgress::new());
        
        // Create overall progress bar
        let overall_bar = ProgressBar::new(total_files as u64);
        overall_bar.set_style(
            ProgressStyle::default_bar()
                .template("⚡ [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta}) - {workers} workers")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        overall_bar.set_message(format!("{}", worker_count));
        let overall_bar = multi_progress.add(overall_bar);

        // Create worker progress bars
        let mut worker_bars = Vec::with_capacity(worker_count);
        for worker_id in 0..worker_count {
            let worker_bar = ProgressBar::new_spinner();
            worker_bar.set_style(
                ProgressStyle::default_spinner()
                    .template(&format!("  Worker {}: {{spinner:.dim}} {{wide_msg}}", worker_id + 1))
                    .unwrap()
                    .tick_strings(&["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃"]),
            );
            worker_bar.set_message("Waiting for files...");
            
            let worker_bar = multi_progress.add(worker_bar);
            worker_bars.push(worker_bar);
        }

        Ok(Self {
            is_parallel: true,
            multi_progress,
            overall_bar,
            worker_bars,
            stats: Arc::new(ProgressStats::default()),
            start_time: Instant::now(),
        })
    }

    /// Update overall progress (called every 5 files or at completion)
    pub fn update_overall(&self, current: usize, total: usize) {
        self.overall_bar.set_position(current as u64);
        self.overall_bar.set_length(total as u64);
        
        // Update statistics display every 100 files or at completion
        if current % 100 == 0 || current == total {
            let snapshot = self.stats.get_snapshot();
            let elapsed = self.start_time.elapsed();
            let rate = current as f64 / elapsed.as_secs_f64();
            
            if self.is_parallel {
                self.overall_bar.set_message(format!(
                    "{} workers | {:.1} files/s | {} scanned, {} with secrets", 
                    self.worker_bars.len(),
                    rate,
                    snapshot.files_scanned,
                    snapshot.files_with_secrets
                ));
            } else {
                self.overall_bar.set_message(format!(
                    "{:.1} files/s | {} scanned, {} with secrets", 
                    rate,
                    snapshot.files_scanned,
                    snapshot.files_with_secrets
                ));
            }
        }
    }

    /// Update worker file display (called for every file in parallel mode)
    pub fn update_worker_file(&self, worker_id: usize, file_path: &str) {
        if let Some(worker_bar) = self.worker_bars.get(worker_id) {
            // Show just the filename, not the full path
            let filename = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);
            
            worker_bar.set_message(format!("Scanning {}", filename));
            worker_bar.tick();
        }
    }

    /// Get reference to statistics for external updates
    pub fn stats(&self) -> Arc<ProgressStats> {
        Arc::clone(&self.stats)
    }

    /// Finish all progress bars and clean up display
    pub fn finish(&self) {
        let snapshot = self.stats.get_snapshot();
        let elapsed = self.start_time.elapsed();
        
        // Finish overall bar with final message
        let final_message = format!(
            "Completed in {:.2}s | {} scanned, {} with secrets",
            elapsed.as_secs_f64(),
            snapshot.files_scanned,
            snapshot.files_with_secrets
        );
        
        self.overall_bar.finish_with_message(final_message);
        
        // Finish worker bars
        for (i, worker_bar) in self.worker_bars.iter().enumerate() {
            worker_bar.finish_with_message(format!("Worker {} completed", i + 1));
        }
        
        // Give UI time to update
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Factory functions for creating progress reporters
/// (Maintains compatibility with existing codebase structure)
pub mod factories {
    use super::*;
    
    /// Create enhanced sequential progress reporter
    pub fn enhanced_sequential_reporter(total_files: usize) -> Arc<Progress> {
        Arc::new(
            Progress::new_sequential(total_files)
                .expect("Failed to create sequential progress reporter")
        )
    }
    
    /// Create enhanced parallel progress reporter  
    pub fn enhanced_parallel_reporter(total_files: usize, worker_count: usize) -> Arc<Progress> {
        Arc::new(
            Progress::new_parallel(total_files, worker_count)
                .expect("Failed to create parallel progress reporter")
        )
    }
}

