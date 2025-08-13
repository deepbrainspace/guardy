//! Advanced progress tracking with indicatif
//!
//! Provides multi-level progress tracking with:
//! - File discovery progress
//! - Scanning progress with speed metrics
//! - Real-time statistics updates
//! - Optional indicatif progress bars for TTY environments

use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, LazyLock};
use std::time::{Duration, Instant};

/// Style templates for progress bars
static DISCOVERY_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos}/{len}")
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
});

static SCANNING_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos}/{len} ({per_sec}) | {prefix}")
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
});

static SPINNER_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg} [{elapsed_precise}]")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
});

/// Progress tracking modes
#[derive(Debug, Clone, Copy)]
pub enum ProgressMode {
    /// No progress bars (for non-TTY or quiet mode)
    Silent,
    /// Show progress bars with detailed stats
    Visible,
}

/// Progress tracker with indicatif integration
///
/// # Design Patterns
/// - **Observer Pattern**: Real-time updates to progress bars
/// - **Facade Pattern**: Simple API hiding indicatif complexity
/// - **Builder Pattern**: Configurable progress modes
///
/// # Performance Optimizations
/// - Atomic counters for lock-free updates
/// - LazyLock for one-time style compilation
/// - Buffered updates to reduce rendering overhead
#[derive(Clone)]
pub struct ProgressTracker {
    /// Multi-progress container for multiple bars
    multi_progress: Option<Arc<MultiProgress>>,
    /// Discovery phase progress bar
    discovery_bar: Option<Arc<ProgressBar>>,
    /// Scanning phase progress bar
    scanning_bar: Option<Arc<ProgressBar>>,
    /// Aggregation phase spinner
    aggregation_spinner: Option<Arc<ProgressBar>>,
    
    /// Atomic counters for lock-free updates
    files_discovered: Arc<AtomicUsize>,
    files_processed: Arc<AtomicUsize>,
    matches_found: Arc<AtomicUsize>,
    bytes_processed: Arc<AtomicU64>,
    
    /// Timing information
    start_time: Arc<RwLock<Option<Instant>>>,
    discovery_duration: Arc<RwLock<Option<Duration>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker with automatic mode detection
    pub fn new() -> Self {
        Self::new_with_mode(if atty::is(atty::Stream::Stdout) {
            ProgressMode::Visible
        } else {
            ProgressMode::Silent
        })
    }
    
    /// Create with explicit indicatif enable/disable
    pub fn new_with_indicatif(enabled: bool) -> Self {
        Self::new_with_mode(if enabled {
            ProgressMode::Visible
        } else {
            ProgressMode::Silent
        })
    }
    
    /// Create with specific progress mode
    pub fn new_with_mode(mode: ProgressMode) -> Self {
        let (multi_progress, discovery_bar, scanning_bar, aggregation_spinner) = match mode {
            ProgressMode::Silent => (None, None, None, None),
            ProgressMode::Visible => {
                let mp = Arc::new(MultiProgress::new());
                
                // Set draw target based on TTY availability
                if !atty::is(atty::Stream::Stdout) {
                    mp.set_draw_target(ProgressDrawTarget::hidden());
                }
                
                let discovery = Arc::new(mp.add(ProgressBar::new(0)));
                discovery.set_style(DISCOVERY_STYLE.clone());
                discovery.set_message("Discovering files");
                
                let scanning = Arc::new(mp.add(ProgressBar::new(0)));
                scanning.set_style(SCANNING_STYLE.clone());
                scanning.set_message("Scanning files");
                
                let aggregation = Arc::new(mp.add(ProgressBar::new_spinner()));
                aggregation.set_style(SPINNER_STYLE.clone());
                aggregation.set_message("Aggregating results");
                
                (Some(mp), Some(discovery), Some(scanning), Some(aggregation))
            }
        };
        
        Self {
            multi_progress,
            discovery_bar,
            scanning_bar,
            aggregation_spinner,
            files_discovered: Arc::new(AtomicUsize::new(0)),
            files_processed: Arc::new(AtomicUsize::new(0)),
            matches_found: Arc::new(AtomicUsize::new(0)),
            bytes_processed: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(RwLock::new(None)),
            discovery_duration: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Start file discovery phase
    pub fn start_discovery(&self) {
        if let Ok(mut start) = self.start_time.write() {
            *start = Some(Instant::now());
        }
        
        if let Some(ref bar) = self.discovery_bar {
            bar.reset();
            bar.set_message("Discovering files");
            bar.enable_steady_tick(Duration::from_millis(100));
        }
    }
    
    /// Finish discovery phase
    pub fn finish_discovery(&self, total_files: usize) {
        self.files_discovered.store(total_files, Ordering::Relaxed);
        
        if let Ok(start) = self.start_time.read() {
            if let Some(start_instant) = *start {
                if let Ok(mut duration) = self.discovery_duration.write() {
                    *duration = Some(start_instant.elapsed());
                }
            }
        }
        
        if let Some(ref bar) = self.discovery_bar {
            bar.set_length(total_files as u64);
            bar.set_position(total_files as u64);
            bar.finish_with_message(format!("Discovered {} files", total_files));
        }
    }
    
    /// Start scanning phase
    pub fn start_scanning(&self, total_files: usize) {
        if let Some(ref bar) = self.scanning_bar {
            bar.reset();
            bar.set_length(total_files as u64);
            bar.set_message("Scanning files");
            bar.set_prefix(format!("0 matches"));
            bar.enable_steady_tick(Duration::from_millis(100));
        }
    }
    
    /// Increment files processed counter
    pub fn increment_files_processed(&self) {
        let processed = self.files_processed.fetch_add(1, Ordering::Relaxed) + 1;
        
        if let Some(ref bar) = self.scanning_bar {
            bar.set_position(processed as u64);
        }
    }
    
    /// Update discovery progress with current counts
    pub fn update_discovery_progress(&self, files_found: usize, dirs_found: usize) {
        if let Some(ref bar) = self.discovery_bar {
            bar.set_message(format!("{} files, {} dirs found", files_found, dirs_found));
        }
    }
    
    /// Update scan details (matches and bytes)
    pub fn update_scan_details(&self, new_matches: usize, bytes: u64) {
        let total_matches = self.matches_found.fetch_add(new_matches, Ordering::Relaxed) + new_matches;
        let total_bytes = self.bytes_processed.fetch_add(bytes, Ordering::Relaxed) + bytes;
        
        if let Some(ref bar) = self.scanning_bar {
            bar.set_prefix(format!(
                "{} matches | {} MB",
                total_matches,
                total_bytes / 1_000_000
            ));
        }
    }
    
    /// Finish scanning phase
    pub fn finish_scanning(&self) {
        if let Some(ref bar) = self.scanning_bar {
            let files = self.files_processed.load(Ordering::Relaxed);
            let matches = self.matches_found.load(Ordering::Relaxed);
            let bytes = self.bytes_processed.load(Ordering::Relaxed);
            
            bar.finish_with_message(format!(
                "Scanned {} files | Found {} matches | Processed {} MB",
                files,
                matches,
                bytes / 1_000_000
            ));
        }
    }
    
    /// Start aggregation phase
    pub fn start_aggregation(&self) {
        if let Some(ref spinner) = self.aggregation_spinner {
            spinner.reset();
            spinner.set_message("Aggregating results");
            spinner.enable_steady_tick(Duration::from_millis(100));
        }
    }
    
    /// Finish aggregation phase
    pub fn finish_aggregation(&self) {
        if let Some(ref spinner) = self.aggregation_spinner {
            spinner.finish_with_message("Results aggregated");
        }
    }
    
    /// Clear all progress bars (for clean exit)
    pub fn clear(&self) {
        if let Some(ref mp) = self.multi_progress {
            mp.clear().ok();
        }
    }
}


impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}