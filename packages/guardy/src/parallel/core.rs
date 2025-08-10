use anyhow::Result;
use crossbeam::channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Generic parallel execution framework for processing work items
/// This framework can be used by any module that needs parallel processing
pub struct ParallelExecutor<T, R> {
    max_workers: usize,
    buffer_size: usize,
    _phantom: std::marker::PhantomData<(T, R)>,
}

/// Context for worker threads to avoid too many function parameters
struct WorkerContext<T, R, F, P> {
    worker_id: usize,
    work_rx: Receiver<T>,
    result_tx: Sender<R>,
    progress_counter: Arc<AtomicUsize>,
    total_items: usize,
    processor: Arc<F>,
    progress_reporter: Option<Arc<P>>,
}

impl<T, R> ParallelExecutor<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers,
            buffer_size: max_workers * 2,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute work items in parallel using a producer-consumer pattern
    pub fn execute<F, P>(
        &self,
        work_items: Vec<T>,
        processor: F,
        progress_reporter: Option<P>,
    ) -> Result<Vec<R>>
    where
        F: Fn(&T, usize) -> R + Send + Sync + 'static, // Add worker_id parameter
        P: Fn(usize, usize, usize) + Send + Sync + 'static, // (current, total, worker_id)
    {
        if work_items.is_empty() {
            return Ok(Vec::new());
        }

        let actual_workers = std::cmp::min(self.max_workers, work_items.len());
        let (work_tx, work_rx): (Sender<T>, Receiver<T>) = bounded(self.buffer_size);
        let (result_tx, result_rx): (Sender<R>, Receiver<R>) = bounded(self.buffer_size);

        let progress_counter = Arc::new(AtomicUsize::new(0));
        let total_items = work_items.len();

        // Wrap processor and progress_reporter in Arc for sharing
        let processor = Arc::new(processor);
        let progress_reporter = progress_reporter.map(Arc::new);

        // Use crossbeam::thread::scope for safe borrowing

        crossbeam::thread::scope(|s| -> Result<Vec<R>> {
            // Spawn worker threads
            for worker_id in 0..actual_workers {
                let ctx = WorkerContext {
                    worker_id,
                    work_rx: work_rx.clone(),
                    result_tx: result_tx.clone(),
                    progress_counter: progress_counter.clone(),
                    total_items,
                    processor: processor.clone(),
                    progress_reporter: progress_reporter.clone(),
                };

                s.spawn(move |_| self.worker_thread(ctx));
            }

            // Producer thread: send work to workers
            let work_tx_clone = work_tx.clone();
            s.spawn(move |_| {
                for work_item in work_items {
                    if work_tx_clone.send(work_item).is_err() {
                        break; // Workers dropped
                    }
                }
            });

            // Drop senders so receivers know when work is done
            drop(work_tx);
            drop(result_tx);

            // Collector: gather results
            Ok(self.collect_results(result_rx, total_items))
        })
        .map_err(|_| anyhow::anyhow!("Thread panic occurred during parallel execution"))?
    }

    fn worker_thread<F, P>(&self, ctx: WorkerContext<T, R, F, P>)
    where
        F: Fn(&T, usize) -> R, // Add worker_id parameter
        P: Fn(usize, usize, usize),
    {
        while let Ok(work_item) = ctx.work_rx.recv() {
            let result = (ctx.processor)(&work_item, ctx.worker_id); // Pass worker_id

            if ctx.result_tx.send(result).is_err() {
                break; // Receiver dropped
            }

            // Update progress
            let current = ctx.progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if let Some(ref reporter) = ctx.progress_reporter {
                // Only report progress every 5 items to reduce contention
                if current % 5 == 0 || current == ctx.total_items {
                    reporter(current, ctx.total_items, ctx.worker_id);
                }
            }
        }
    }

    fn collect_results(&self, result_rx: Receiver<R>, total_items: usize) -> Vec<R> {
        let mut results = Vec::new();
        let mut items_processed = 0;

        while let Ok(result) = result_rx.recv() {
            results.push(result);
            items_processed += 1;

            if items_processed >= total_items {
                break;
            }
        }

        results
    }
}

/// Sequential execution strategy for comparison/fallback
pub struct SequentialExecutor;

impl SequentialExecutor {
    pub fn execute<T, R, F, P>(
        work_items: Vec<T>,
        processor: F,
        progress_reporter: Option<P>,
    ) -> Vec<R>
    where
        F: Fn(&T, usize) -> R,      // Add worker_id parameter
        P: Fn(usize, usize, usize), // (current, total, worker_id)
    {
        let total_items = work_items.len();
        let mut results = Vec::with_capacity(total_items);

        for (index, work_item) in work_items.iter().enumerate() {
            let result = processor(work_item, 0); // Sequential uses worker_id 0
            results.push(result);

            // Show progress
            let current = index + 1;
            if let Some(reporter) = &progress_reporter
                && (current % 5 == 0 || current == total_items)
            {
                reporter(current, total_items, 0); // worker_id = 0 for sequential
            }
        }

        results
    }
}

/// Execution strategy enum for choosing between parallel and sequential
#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    Sequential,
    Parallel { workers: usize },
}

impl ExecutionStrategy {
    pub fn execute<T, R, F, P>(
        &self,
        work_items: Vec<T>,
        processor: F,
        progress_reporter: Option<P>,
    ) -> Result<Vec<R>>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
        F: Fn(&T, usize) -> R + Send + Sync + 'static, // Add worker_id parameter
        P: Fn(usize, usize, usize) + Send + Sync + 'static,
    {
        match self {
            ExecutionStrategy::Sequential => Ok(SequentialExecutor::execute(
                work_items,
                processor,
                progress_reporter,
            )),
            ExecutionStrategy::Parallel { workers } => {
                let executor = ParallelExecutor::new(*workers);
                executor.execute(work_items, processor, progress_reporter)
            }
        }
    }

    /// Auto strategy selection based on workload size threshold
    ///
    /// This method provides a **threshold-based decision** between sequential and parallel execution.
    /// The client is responsible for providing pre-calculated optimal worker counts.
    ///
    /// # Parameters
    /// - `work_items_count`: Total number of work items to process
    /// - `min_items_for_parallel`: Minimum items needed to justify parallel overhead
    /// - `optimal_workers`: Pre-calculated optimal worker count (from domain-specific adaptation)
    ///
    /// # Returns
    /// ExecutionStrategy::Sequential or ExecutionStrategy::Parallel based on threshold
    ///
    /// # Decision Logic
    /// ```text
    /// if work_items_count >= min_items_for_parallel {
    ///     Parallel { workers: optimal_workers }  // Use provided worker count
    /// } else {
    ///     Sequential                              // Skip parallel overhead
    /// }
    /// ```
    ///
    /// # Design Principle
    /// This method only handles the **threshold decision**. All complex logic should be
    /// handled by the client before calling this method:
    ///
    /// **Client Responsibilities:**
    /// - Calculate system resource limits
    /// - Apply domain-specific worker adaptation  
    /// - Provide final optimal worker count
    ///
    /// **This Method's Responsibility:**
    /// - Simple threshold comparison
    /// - Strategy enum creation
    ///
    /// # Example
    /// ```rust
    /// use guardy::parallel::ExecutionStrategy;
    ///
    /// // Client handles complex calculations
    /// let max_workers = ExecutionStrategy::calculate_optimal_workers(0, 75);
    /// let optimal_workers = std::cmp::min(6, max_workers); // Domain adaptation
    ///
    /// // This method handles simple threshold decision
    /// let strategy = ExecutionStrategy::auto(36, 50, optimal_workers);
    /// // 36 < 50 → Sequential (threshold not met)
    /// assert!(matches!(strategy, ExecutionStrategy::Sequential));
    ///
    /// let strategy = ExecutionStrategy::auto(100, 50, optimal_workers);
    /// // 100 >= 50 → Parallel (threshold met)
    /// assert!(matches!(strategy, ExecutionStrategy::Parallel { .. }));
    /// ```
    pub fn auto(
        work_items_count: usize,
        min_items_for_parallel: usize,
        optimal_workers: usize,
    ) -> Self {
        if work_items_count >= min_items_for_parallel {
            ExecutionStrategy::Parallel {
                workers: optimal_workers,
            }
        } else {
            ExecutionStrategy::Sequential
        }
    }

    /// Calculate optimal workers based on available system resources and configuration limits
    ///
    /// This method implements **resource-aware worker calculation** that respects system capabilities
    /// and user configuration without any domain-specific knowledge.
    ///
    /// # Parameters
    /// - `max_threads_config`: User-specified maximum threads (0 = no limit)
    /// - `thread_percentage`: Percentage of CPU cores to utilize (e.g., 75 for 75%)
    ///
    /// # Returns
    /// Maximum number of workers that can be used based on system resources and configuration
    ///
    /// # Algorithm
    /// ```text
    /// 1. Detect available CPU cores: num_cpus::get()
    /// 2. Apply percentage: cores * thread_percentage / 100
    /// 3. Apply config limit: min(max_threads_config, percentage_result) if max_threads_config > 0
    /// 4. Ensure minimum: max(1, final_result)
    /// ```
    ///
    /// # Examples
    /// ```rust
    /// use guardy::parallel::ExecutionStrategy;
    ///
    /// // These examples show the calculation logic but results depend on actual system
    /// let workers = ExecutionStrategy::calculate_optimal_workers(0, 75);
    /// assert!(workers >= 1); // Always at least 1 worker
    ///
    /// let workers = ExecutionStrategy::calculate_optimal_workers(8, 75);
    /// assert!(workers <= 8); // Respects max limit
    ///
    /// let workers = ExecutionStrategy::calculate_optimal_workers(0, 50);
    /// assert!(workers >= 1); // Always at least 1 worker
    /// ```
    ///
    /// # Design Principle
    /// This method focuses solely on **system resources** and **user preferences**.
    /// It does NOT consider:
    /// - Workload characteristics (file counts, task types)
    /// - Domain-specific optimization
    /// - Application logic
    ///
    /// Domain-specific adaptations should be handled by the calling module.
    pub fn calculate_optimal_workers(max_threads_config: usize, thread_percentage: u8) -> usize {
        let available_cores = num_cpus::get();

        // Calculate workers based on percentage of available cores
        let workers_by_percentage =
            std::cmp::max(1, (available_cores * thread_percentage as usize) / 100);

        // Apply config limit if specified (0 means use percentage calculation only)
        if max_threads_config > 0 {
            std::cmp::min(max_threads_config, workers_by_percentage)
        } else {
            workers_by_percentage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_executor() {
        let work_items = vec![1, 2, 3, 4, 5];
        let results = SequentialExecutor::execute(
            work_items,
            |x, _worker_id| x * 2,
            None::<fn(usize, usize, usize)>,
        );
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_executor() {
        let executor = ParallelExecutor::new(2);
        let work_items = vec![1, 2, 3, 4, 5];
        let results = executor
            .execute(
                work_items,
                |x, _worker_id| x * 2,
                None::<fn(usize, usize, usize)>,
            )
            .unwrap();

        // Results may be in different order due to parallel execution
        let mut sorted_results = results;
        sorted_results.sort();
        assert_eq!(sorted_results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_execution_strategy() {
        let work_items = vec![1, 2, 3];

        // Test sequential strategy
        let sequential = ExecutionStrategy::Sequential;
        let seq_results = sequential
            .execute(
                work_items.clone(),
                |x, _worker_id| x * 3,
                None::<fn(usize, usize, usize)>,
            )
            .unwrap();
        assert_eq!(seq_results, vec![3, 6, 9]);

        // Test parallel strategy
        let parallel = ExecutionStrategy::Parallel { workers: 2 };
        let par_results = parallel
            .execute(
                work_items,
                |x, _worker_id| x * 3,
                None::<fn(usize, usize, usize)>,
            )
            .unwrap();
        let mut sorted_par_results = par_results;
        sorted_par_results.sort();
        assert_eq!(sorted_par_results, vec![3, 6, 9]);
    }

    #[test]
    fn test_auto_strategy() {
        // Small workload should be sequential
        let strategy = ExecutionStrategy::auto(5, 10, 8);
        assert!(matches!(strategy, ExecutionStrategy::Sequential));

        // Large workload should be parallel
        let strategy = ExecutionStrategy::auto(50, 10, 8);
        assert!(matches!(strategy, ExecutionStrategy::Parallel { .. }));
    }
}
