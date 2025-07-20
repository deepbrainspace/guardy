use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of worker threads (0 = auto-detect)
    pub max_threads: usize,
    /// Percentage of CPU cores to use (1-100)
    pub thread_percentage: u8,
    /// Channel buffer size multiplier (buffer = workers * multiplier)
    pub channel_buffer_multiplier: usize,
    /// Progress update frequency (every N items)
    pub progress_update_frequency: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: 0,
            thread_percentage: 75,
            channel_buffer_multiplier: 2,
            progress_update_frequency: 5,
        }
    }
}

/// Result from processing a work item
#[derive(Debug)]
pub struct WorkResult<R> {
    pub result: R,
    pub success: bool,
    pub error: Option<String>,
}

/// Generic parallel processor for work distribution
pub struct ParallelProcessor {
    config: ParallelConfig,
}

impl ParallelProcessor {
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Calculate optimal number of worker threads
    pub fn calculate_optimal_workers(&self, work_count: usize) -> usize {
        let cpu_cores = num_cpus::get();
        
        // Apply thread percentage from config
        let max_by_percentage = std::cmp::max(1, (cpu_cores * self.config.thread_percentage as usize) / 100);
        
        // Apply max_threads limit if specified (0 means use percentage calculation)
        let max_workers = if self.config.max_threads > 0 {
            std::cmp::min(self.config.max_threads, max_by_percentage)
        } else {
            max_by_percentage
        };
        
        // Don't create more workers than work items
        std::cmp::min(max_workers, work_count.max(1))
    }

    /// Process work items in parallel using a worker function
    /// 
    /// # Arguments
    /// * `work_items` - Items to process
    /// * `worker_fn` - Function that processes each work item
    /// * `progress_label` - Label for progress display
    /// 
    /// # Returns
    /// Vector of successful results in original order
    pub fn process<T, R, F>(&self, work_items: Vec<T>, worker_fn: F, progress_label: &str) -> Result<Vec<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Result<R> + Send + Sync + 'static,
    {
        let work_count = work_items.len();
        if work_count == 0 {
            return Ok(Vec::new());
        }

        let optimal_workers = self.calculate_optimal_workers(work_count);
        
        // Create bounded channels
        let (work_tx, work_rx): (Sender<(usize, T)>, Receiver<(usize, T)>) = 
            bounded(optimal_workers * self.config.channel_buffer_multiplier);
        let (result_tx, result_rx): (Sender<(usize, WorkResult<R>)>, Receiver<(usize, WorkResult<R>)>) = 
            bounded(optimal_workers * self.config.channel_buffer_multiplier * 2);
        
        // Shared progress counter
        let progress_counter = Arc::new(AtomicUsize::new(0));
        
        // Use crossbeam::thread::scope for safe borrowing
        let indexed_results = crossbeam::thread::scope(|s| {
            let worker_fn = Arc::new(worker_fn);
            
            // Spawn worker threads
            for worker_id in 0..optimal_workers {
                let work_rx = work_rx.clone();
                let result_tx = result_tx.clone();
                let progress_counter = progress_counter.clone();
                let worker_fn = worker_fn.clone();
                
                s.spawn(move |_| {
                    while let Ok((index, work_item)) = work_rx.recv() {
                        // Process the work item
                        let work_result = match worker_fn(work_item) {
                            Ok(result) => WorkResult {
                                result,
                                success: true,
                                error: None,
                            },
                            Err(e) => WorkResult {
                                result: unsafe { std::mem::zeroed() }, // Placeholder for error case
                                success: false,
                                error: Some(e.to_string()),
                            },
                        };
                        
                        // Send result
                        if result_tx.send((index, work_result)).is_err() {
                            break; // Receiver dropped
                        }
                        
                        // Update progress
                        let current = progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
                        if current % self.config.progress_update_frequency == 0 || current == work_count {
                            print!("\r⚡ {}: {}/{} items ({:.1}%) [worker-{}]", 
                                   progress_label,
                                   current, work_count, 
                                   (current as f64 / work_count as f64 * 100.0),
                                   worker_id);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }
                    }
                });
            }
            
            // Producer thread: send work to workers
            let work_tx_clone = work_tx.clone();
            s.spawn(move |_| {
                for (index, work_item) in work_items.into_iter().enumerate() {
                    if work_tx_clone.send((index, work_item)).is_err() {
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
            self.collect_results(result_rx, work_count)
        }).map_err(|_| anyhow::anyhow!("Thread panic occurred during parallel processing"))?;
        
        // Clear progress line
        if work_count > 0 {
            print!("\r");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        
        // Sort results by original index and extract successful ones
        let mut sorted_results: Vec<_> = indexed_results.into_iter().collect();
        sorted_results.sort_by_key(|(index, _)| *index);
        
        let successful_results: Vec<R> = sorted_results
            .into_iter()
            .filter_map(|(_, work_result)| {
                if work_result.success {
                    Some(work_result.result)
                } else {
                    None // Could collect errors here if needed
                }
            })
            .collect();
            
        Ok(successful_results)
    }


    /// Collect results from workers
    fn collect_results<R>(
        &self,
        result_rx: Receiver<(usize, WorkResult<R>)>,
        total_work: usize,
    ) -> Vec<(usize, WorkResult<R>)> {
        let mut results = Vec::with_capacity(total_work);
        let mut work_processed = 0;
        
        while let Ok(result) = result_rx.recv() {
            results.push(result);
            work_processed += 1;
            
            // Break when all work is processed
            if work_processed >= total_work {
                break;
            }
        }
        
        results
    }
}

/// Convenience function for parallel processing with default config
pub fn process_parallel<T, R, F>(work_items: Vec<T>, worker_fn: F, progress_label: &str) -> Result<Vec<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Result<R> + Send + Sync + 'static,
{
    let processor = ParallelProcessor::new(ParallelConfig::default());
    processor.process(work_items, worker_fn, progress_label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parallel_processor_basic() {
        let items = vec![1, 2, 3, 4, 5];
        let results = process_parallel(items, |x| Ok(x * 2), "Test").unwrap();
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_processor_with_errors() {
        let items = vec![1, 2, 3, 4, 5];
        let results = process_parallel(items, |x| {
            if x == 3 {
                Err(anyhow::anyhow!("Error at 3"))
            } else {
                Ok(x * 2)
            }
        }, "Test with errors").unwrap();
        
        // Should return successful results only
        assert_eq!(results, vec![2, 4, 8, 10]);
    }

    #[test]
    fn test_optimal_workers_calculation() {
        let config = ParallelConfig::default();
        let processor = ParallelProcessor::new(config);
        
        // Should not exceed work count
        let workers = processor.calculate_optimal_workers(2);
        assert!(workers <= 2);
        assert!(workers >= 1);
    }
}