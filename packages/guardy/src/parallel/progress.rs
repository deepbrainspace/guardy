use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Scanning statistics tracked atomically across threads
#[derive(Debug, Default)]
pub struct ScanningStats {
    pub scanned: AtomicUsize,
    pub skipped: AtomicUsize,
    pub binary: AtomicUsize,
}

impl ScanningStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment_scanned(&self) {
        self.scanned.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_skipped(&self) {
        self.skipped.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_binary(&self) {
        self.binary.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_counts(&self) -> (usize, usize, usize) {
        (
            self.scanned.load(Ordering::Relaxed),
            self.skipped.load(Ordering::Relaxed),
            self.binary.load(Ordering::Relaxed),
        )
    }
}

/// Progress reporting strategies for parallel execution
pub trait ProgressReporter: Send + Sync {
    fn report(&self, current: usize, total: usize, worker_id: usize);
    fn clear(&self);
}

/// Enhanced progress reporter with live statistics using indicatif
#[derive(Clone)]
pub struct StatisticsProgressReporter {
    multi_progress: MultiProgress,
    overall_bar: Option<ProgressBar>,
    pub worker_bars: Vec<ProgressBar>,
    stats: Arc<ScanningStats>,
    update_frequency: usize,
    pub is_parallel: bool,
}

impl StatisticsProgressReporter {
    /// Create a sequential progress reporter with live statistics
    pub fn sequential(total_files: usize) -> Self {
        let multi_progress = MultiProgress::new();
        
        let style = ProgressStyle::with_template(
            "🔍 [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} files {spinner}"
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
        
        let main_bar = multi_progress.add(ProgressBar::new(total_files as u64));
        main_bar.set_style(style);
        main_bar.enable_steady_tick(std::time::Duration::from_millis(100));
        
        Self {
            multi_progress,
            overall_bar: Some(main_bar),
            worker_bars: Vec::new(),
            stats: Arc::new(ScanningStats::new()),
            update_frequency: 10, // Update every 10 files for smooth animation
            is_parallel: false,
        }
    }
    
    /// Create a parallel progress reporter with per-worker bars and live statistics
    pub fn parallel(total_files: usize, worker_count: usize) -> Self {
        let multi_progress = MultiProgress::new();
        let mut worker_bars = Vec::new();
        
        // Colors for different workers
        let worker_colors = ["cyan/blue", "green/yellow", "magenta/red", "yellow/blue"];
        
        // Create worker bars with different colors and styles
        for worker_id in 0..worker_count {
            let color = worker_colors[worker_id % worker_colors.len()];
            let style = ProgressStyle::with_template(
                &format!("[Worker {}] [{{elapsed_precise}}] {{bar:40.{}}} {{pos:>7}}/{{len:7}} {{spinner}} {{msg}}", 
                        worker_id + 1, color)
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  ")
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
            
            let worker_bar = multi_progress.add(ProgressBar::new(0)); // Will be set when files are assigned
            worker_bar.set_style(style);
            worker_bar.enable_steady_tick(std::time::Duration::from_millis(120)); // Slightly different timing for visual variety
            worker_bars.push(worker_bar);
        }
        
        // Overall progress bar
        let overall_style = ProgressStyle::with_template(
            "Overall:   [{elapsed_precise}] {bar:40.bright_white/dim} {pos:>7}/{len:7} files ({percent}%)"
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ");
        
        let overall_bar = multi_progress.add(ProgressBar::new(total_files as u64));
        overall_bar.set_style(overall_style);
        
        Self {
            multi_progress,
            overall_bar: Some(overall_bar),
            worker_bars,
            stats: Arc::new(ScanningStats::new()),
            update_frequency: 5, // More frequent updates for parallel (smoother)
            is_parallel: true,
        }
    }
    
    /// Update worker bar with current file being processed
    pub fn update_worker_file(&self, worker_id: usize, file_path: &str, progress: usize, total: usize) {
        if let Some(worker_bar) = self.worker_bars.get(worker_id) {
            worker_bar.set_length(total as u64);
            worker_bar.set_position(progress as u64);
            
            // Show current file being scanned (truncate if too long)
            let display_path = if file_path.len() > 40 {
                format!("...{}", &file_path[file_path.len()-37..])
            } else {
                file_path.to_string()
            };
            worker_bar.set_message(format!("📄 {}", display_path));
        }
    }
    
    /// Mark worker as complete
    pub fn finish_worker(&self, worker_id: usize) {
        if let Some(worker_bar) = self.worker_bars.get(worker_id) {
            worker_bar.finish_with_message("✓ Complete");
        }
    }
    
    /// Update overall progress and statistics display
    pub fn update_overall(&self, completed: usize, total: usize) {
        if let Some(ref overall_bar) = self.overall_bar {
            overall_bar.set_position(completed as u64);
        }
        
        // Update statistics display using println (appears above progress bars)
        let (scanned, skipped, binary) = self.stats.get_counts();
        let active_workers = if self.is_parallel {
            self.worker_bars.iter().filter(|bar| !bar.is_finished()).count()
        } else {
            0
        };
        
        let stats_msg = if self.is_parallel {
            format!("📊 Scanned: {} | Skipped: {} | Binary: {} | Active: {} threads", 
                   scanned, skipped, binary, active_workers)
        } else {
            format!("📊 Scanned: {} | Skipped: {} | Binary: {}", 
                   scanned, skipped, binary)
        };
        
        // Use println to show stats above progress bars (only every N updates to avoid spam)
        if completed % (self.update_frequency * 5) == 0 || completed == total {
            let _ = self.multi_progress.println(stats_msg);
        }
    }
    
    /// Get shared statistics for external updates
    pub fn stats(&self) -> Arc<ScanningStats> {
        self.stats.clone()
    }
    
    /// Clear all progress bars
    pub fn finish(&self) {
        let _ = self.multi_progress.clear();
    }
}

/// Console progress reporter with configurable display
#[derive(Clone)]
pub struct ConsoleProgressReporter {
    show_worker_id: bool,
    update_frequency: usize,
    progress_icon: &'static str,
    item_name: String,
}

impl ConsoleProgressReporter {
    pub fn new(item_name: &str) -> Self {
        Self {
            show_worker_id: false,
            update_frequency: 5,
            progress_icon: "⏳",
            item_name: item_name.to_string(),
        }
    }

    pub fn with_worker_id(mut self) -> Self {
        self.show_worker_id = true;
        self.progress_icon = "⚡";
        self
    }

    pub fn with_frequency(mut self, frequency: usize) -> Self {
        self.update_frequency = frequency;
        self
    }

    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.progress_icon = icon;
        self
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    fn report(&self, current: usize, total: usize, worker_id: usize) {
        // Only report at specified frequency to reduce console spam
        if current % self.update_frequency == 0 || current == total {
            let percentage = current as f64 / total as f64 * 100.0;
            
            if self.show_worker_id {
                print!("\r{} Progress: {}/{} {} ({:.1}%) [worker-{}]", 
                       self.progress_icon, current, total, self.item_name, percentage, worker_id);
            } else {
                print!("\r{} Progress: {}/{} {} ({:.1}%)", 
                       self.progress_icon, current, total, self.item_name, percentage);
            }
            
            std::io::stdout().flush().ok();
        }
    }

    fn clear(&self) {
        print!("\r");
        std::io::stdout().flush().ok();
    }
}



/// Factory functions for common progress reporters
pub mod factories {
    use super::*;

    /// Create a parallel progress reporter for any type of items (legacy)
    pub fn parallel_reporter(item_name: &str) -> ConsoleProgressReporter {
        ConsoleProgressReporter::new(item_name)
            .with_worker_id()
            .with_frequency(5)
    }

    /// Create a sequential progress reporter for any type of items (legacy)
    pub fn sequential_reporter(item_name: &str) -> ConsoleProgressReporter {
        ConsoleProgressReporter::new(item_name)
            .with_frequency(5)
    }

    /// Create an enhanced statistics progress reporter for sequential scanning
    pub fn enhanced_sequential_reporter(total_files: usize) -> StatisticsProgressReporter {
        StatisticsProgressReporter::sequential(total_files)
    }

    /// Create an enhanced statistics progress reporter for parallel scanning  
    pub fn enhanced_parallel_reporter(total_files: usize, worker_count: usize) -> StatisticsProgressReporter {
        StatisticsProgressReporter::parallel(total_files, worker_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_progress_reporter() {
        let reporter = ConsoleProgressReporter::new("items");
        
        // This test mainly ensures the reporter doesn't panic
        reporter.report(5, 10, 0);
        reporter.clear();
    }

    #[test]
    fn test_console_progress_reporter_with_options() {
        let reporter = ConsoleProgressReporter::new("files")
            .with_worker_id()
            .with_frequency(10)
            .with_icon("🔍");
        
        // This test mainly ensures the reporter doesn't panic
        reporter.report(10, 100, 1);
        reporter.clear();
    }

    #[test]
    fn test_factory_functions() {
        let _parallel = factories::parallel_reporter("tasks");
        let _sequential = factories::sequential_reporter("items");
    }
}