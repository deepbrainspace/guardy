use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scanning statistics tracked atomically across threads
#[derive(Debug, Default)]
pub struct ScanningStats {
    pub scanned: AtomicUsize,
    pub with_secrets: AtomicUsize,
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

    pub fn increment_with_secrets(&self) {
        self.with_secrets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_skipped(&self) {
        self.skipped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_binary(&self) {
        self.binary.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.scanned.load(Ordering::Relaxed),
            self.with_secrets.load(Ordering::Relaxed),
            self.skipped.load(Ordering::Relaxed),
            self.binary.load(Ordering::Relaxed),
        )
    }
}

/// Enhanced progress reporter with live statistics using indicatif
#[derive(Clone)]
pub struct StatisticsProgressReporter {
    #[allow(dead_code)]
    // Required for indicatif coordination - see https://docs.rs/indicatif/latest/indicatif/struct.MultiProgress.html
    multi_progress: MultiProgress,
    overall_bar: Option<ProgressBar>,
    pub worker_bars: Vec<ProgressBar>,
    stats: Arc<ScanningStats>,
    update_frequency: usize,
    pub is_parallel: bool,
    worker_counts: Arc<Vec<AtomicUsize>>, // Per-worker file counts
}

impl StatisticsProgressReporter {
    /// Create a sequential progress reporter with live statistics
    pub fn sequential(total_files: usize) -> Self {
        let multi_progress = MultiProgress::new();

        let style = ProgressStyle::with_template(
            "🔍 [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} files {spinner} {msg}",
        )
        .unwrap()
        .progress_chars("##-")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let main_bar = multi_progress.add(ProgressBar::new(total_files as u64));
        main_bar.set_style(style);
        main_bar.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            multi_progress,
            overall_bar: Some(main_bar),
            worker_bars: Vec::new(),
            stats: Arc::new(ScanningStats::new()),
            update_frequency: 100, // Update every 100 files to reduce spam
            is_parallel: false,
            worker_counts: Arc::new(Vec::new()),
        }
    }

    /// Create a parallel progress reporter with per-worker bars and live statistics
    pub fn parallel(total_files: usize, worker_count: usize) -> Self {
        let multi_progress = MultiProgress::new();
        let mut worker_bars = Vec::new();

        // Simple color scheme that should work in most terminals
        let worker_colors = ["green/black", "blue/black", "red/black", "yellow/black"];

        // Estimate files per worker for progress bar lengths
        let estimated_files_per_worker = total_files.div_ceil(worker_count); // Round up division

        // Create worker bars with different colors and styles
        for worker_id in 0..worker_count {
            let color = worker_colors[worker_id % worker_colors.len()];
            let style = ProgressStyle::with_template(&format!(
                "[Worker {:02}] {{bar:40.{}}} {{pos:>7}}/{{len:7}} {{msg}}",
                worker_id + 1,
                color
            ))
            .unwrap()
            .progress_chars("##-");

            // Set reasonable length based on estimated files per worker
            let worker_bar =
                multi_progress.add(ProgressBar::new(estimated_files_per_worker as u64));
            worker_bar.set_style(style);
            worker_bar.enable_steady_tick(std::time::Duration::from_millis(100)); // Need tick for visibility
            worker_bars.push(worker_bar);
        }

        // Overall progress bar
        let overall_style = ProgressStyle::with_template(
            "Overall:\n[{elapsed_precise}] {bar:40.green/red} {pos:>7}/{len:7} files ({percent}%)\n{msg}"
        )
        .unwrap()
        .progress_chars("##-");

        let overall_bar = multi_progress.add(ProgressBar::new(total_files as u64));
        overall_bar.set_style(overall_style);

        // Initialize per-worker counters
        let worker_counts: Vec<AtomicUsize> =
            (0..worker_count).map(|_| AtomicUsize::new(0)).collect();

        Self {
            multi_progress,
            overall_bar: Some(overall_bar),
            worker_bars,
            stats: Arc::new(ScanningStats::new()),
            update_frequency: 100, // Update every 100 files to reduce spam
            is_parallel: true,
            worker_counts: Arc::new(worker_counts),
        }
    }

    /// Update worker bar with current file being processed
    pub fn update_worker_file(&self, worker_id: usize, file_path: &str) {
        if let Some(worker_bar) = self.worker_bars.get(worker_id) {
            // Increment this worker's file count
            if let Some(worker_count) = self.worker_counts.get(worker_id) {
                let current_count = worker_count.fetch_add(1, Ordering::Relaxed) + 1;
                worker_bar.set_position(current_count as u64);
            }

            // Show current file being scanned (truncate if too long)
            let display_path = if file_path.len() > 40 {
                format!("...{}", &file_path[file_path.len() - 37..])
            } else {
                file_path.to_string()
            };
            worker_bar.set_message(format!("📄 {display_path}"));
        }
    }

    /// Update overall progress and statistics display
    pub fn update_overall(&self, completed: usize, total: usize) {
        if let Some(ref overall_bar) = self.overall_bar {
            overall_bar.set_position(completed as u64);

            // Update statistics in the progress bar message instead of printing

            // Only update message every few files to reduce flicker
            if completed % self.update_frequency == 0 || completed == total {
                let (scanned, with_secrets, skipped, binary) = self.stats.get_counts();
                let stats_msg = format!(
                    "📊 Scanned: {scanned} | With Secrets: {with_secrets} | Skipped: {skipped} | Binary: {binary}"
                );

                overall_bar.set_message(stats_msg);
            }
        }
    }

    /// Get shared statistics for external updates
    pub fn stats(&self) -> Arc<ScanningStats> {
        self.stats.clone()
    }

    /// Finish all progress bars properly but keep them visible
    pub fn finish(&self) {
        // Finish all worker bars and keep them visible with final state
        for worker_bar in &self.worker_bars {
            worker_bar.finish_with_message("");
        }

        // Finish overall bar
        if let Some(ref overall_bar) = self.overall_bar {
            overall_bar.finish_with_message("🎯 Scan completed");
        }

        // DON'T clear the multi-progress display - keep it visible
    }
}

/// Factory functions for common progress reporters
pub mod factories {
    use super::*;

    /// Create an enhanced statistics progress reporter for sequential scanning
    pub fn enhanced_sequential_reporter(total_files: usize) -> StatisticsProgressReporter {
        StatisticsProgressReporter::sequential(total_files)
    }

    /// Create an enhanced statistics progress reporter for parallel scanning
    pub fn enhanced_parallel_reporter(
        total_files: usize,
        worker_count: usize,
    ) -> StatisticsProgressReporter {
        StatisticsProgressReporter::parallel(total_files, worker_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_progress_reporter_sequential() {
        let reporter = StatisticsProgressReporter::sequential(100);

        // Test basic functionality
        reporter.update_overall(50, 100);
        reporter.finish();
    }

    #[test]
    fn test_statistics_progress_reporter_parallel() {
        let reporter = StatisticsProgressReporter::parallel(100, 4);

        // Test worker updates
        reporter.update_worker_file(0, "/test/file.rs");
        reporter.update_overall(50, 100);
        reporter.finish();
    }

    #[test]
    fn test_factory_functions() {
        let _sequential = factories::enhanced_sequential_reporter(100);
        let _parallel = factories::enhanced_parallel_reporter(100, 4);
    }
}
