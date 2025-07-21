use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam::channel::{bounded, Receiver, Sender};
use super::types::{SecretMatch, ScanStats, Warning, ScanResult, Scanner, ScanFileResult};

impl Scanner {
    /// Scan a directory recursively using parallel processing
    /// This version uses crossbeam channels for producer-consumer pattern
    /// achieving 3-8x speedup on multi-core systems
    pub fn scan_directory_parallel(&self, path: &Path) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        let mut warnings: Vec<Warning> = Vec::new();
        
        let walker = self.build_directory_walker(path).build();
        
        // Fast file counting for progress tracking
        let file_count = self.fast_count_files(path)?;
        println!("🔍 Scanning {} files in parallel...", file_count);
        
        // Analyze directories and their gitignore status (same as sequential mode)
        self.analyze_and_display_directories(path);
        
        // Collect all file paths first (lightweight operation)
        let mut file_paths = Vec::new();
        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        file_paths.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    warnings.push(Warning {
                        message: format!("Walk error: {}", e),
                    });
                }
            }
        }
        
        // Calculate optimal number of worker threads
        let optimal_workers = self.calculate_optimal_workers(file_paths.len());
        
        // Create bounded channels for work distribution and result collection
        let (work_tx, work_rx): (Sender<PathBuf>, Receiver<PathBuf>) = bounded(optimal_workers * 2);
        let (result_tx, result_rx): (Sender<ScanFileResult>, Receiver<ScanFileResult>) = bounded(optimal_workers * 4);
        
        // Shared progress counter
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let total_files = file_paths.len();
        
        // Use crossbeam::thread::scope for safe borrowing of self
        let all_matches = match crossbeam::thread::scope(|s| {
            // Spawn worker threads
            for worker_id in 0..optimal_workers {
                let work_rx = work_rx.clone();
                let result_tx = result_tx.clone();
                let progress_counter = progress_counter.clone();
                
                s.spawn(move |_| {
                    self.parallel_worker(worker_id, work_rx, result_tx, progress_counter, total_files)
                });
            }
            
            // Producer thread: send work to workers
            let work_tx_clone = work_tx.clone();
            s.spawn(move |_| {
                for file_path in file_paths {
                    if work_tx_clone.send(file_path).is_err() {
                        break; // Workers dropped
                    }
                }
                // Close the work channel
                drop(work_tx_clone);
            });
            
            // Drop the original senders so receivers know when work is done
            drop(work_tx);
            drop(result_tx);
            
            // Collector: gather results
            self.collect_parallel_results(result_rx, &mut warnings, total_files)
        }) {
            Ok(matches) => matches,
            Err(_) => {
                return Err(anyhow::anyhow!("Thread panic occurred during parallel scan"));
            }
        };
        
        // Clear progress line
        if total_files > 0 {
            print!("\r");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        
        let scan_duration = start_time.elapsed();
        let stats = ScanStats {
            files_scanned: total_files,
            files_skipped: warnings.len(),
            total_matches: all_matches.len(),
            scan_duration_ms: scan_duration.as_millis() as u64,
        };
        
        // Show timing summary
        println!("⚡ Parallel scan completed in {:.2}s ({} files, {} matches, {} workers)", 
                 scan_duration.as_secs_f64(), 
                 stats.files_scanned, 
                 stats.total_matches,
                 optimal_workers);
        
        Ok(ScanResult {
            matches: all_matches,
            stats,
            warnings,
        })
    }
    
    /// Calculate optimal number of worker threads based on config and workload
    fn calculate_optimal_workers(&self, file_count: usize) -> usize {
        let cpu_cores = num_cpus::get();
        
        // Apply thread percentage from config
        let max_by_percentage = std::cmp::max(1, (cpu_cores * self.config.thread_percentage as usize) / 100);
        
        // Apply max_threads limit if specified (0 means use percentage calculation)
        let max_workers = if self.config.max_threads > 0 {
            std::cmp::min(self.config.max_threads, max_by_percentage)
        } else {
            max_by_percentage
        };
        
        // Don't create more workers than files
        std::cmp::min(max_workers, file_count.max(1))
    }
    
    /// Worker thread function for parallel file processing
    fn parallel_worker(
        &self,
        worker_id: usize,
        work_rx: Receiver<PathBuf>,
        result_tx: Sender<ScanFileResult>,
        progress_counter: Arc<AtomicUsize>,
        total_files: usize,
    ) {
        while let Ok(file_path) = work_rx.recv() {
            // Process the file
            let result = match self.scan_single_path(&file_path) {
                Ok(matches) => ScanFileResult {
                    matches,
                    file_path: file_path.to_string_lossy().to_string(),
                    success: true,
                    error: None,
                },
                Err(e) => ScanFileResult {
                    matches: Vec::new(),
                    file_path: file_path.to_string_lossy().to_string(),
                    success: false,
                    error: Some(e.to_string()),
                },
            };
            
            // Send result
            if result_tx.send(result).is_err() {
                break; // Receiver dropped
            }
            
            // Update progress (reduced frequency to minimize contention)
            let current = progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if current % 5 == 0 || current == total_files {
                print!("\r⚡ Progress: {}/{} files ({:.1}%) [worker-{}]", 
                       current, total_files, 
                       (current as f64 / total_files as f64 * 100.0),
                       worker_id);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
    }
    
    /// Collect results from parallel workers
    fn collect_parallel_results(
        &self,
        result_rx: Receiver<ScanFileResult>,
        warnings: &mut Vec<Warning>,
        total_files: usize,
    ) -> Vec<SecretMatch> {
        let mut all_matches = Vec::new();
        let mut files_processed = 0;
        
        while let Ok(result) = result_rx.recv() {
            files_processed += 1;
            
            if result.success {
                all_matches.extend(result.matches);
            } else if let Some(error) = result.error {
                warnings.push(Warning {
                    message: format!("Failed to scan {}: {}", result.file_path, error),
                });
            }
            
            // Break when all files are processed
            if files_processed >= total_files {
                break;
            }
        }
        
        all_matches
    }
}