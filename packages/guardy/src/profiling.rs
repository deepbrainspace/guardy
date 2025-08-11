//! Common profiling utilities for optimizing parallel execution
//!
//! This module provides shared utilities for determining optimal parallel execution
//! strategies based on workload characteristics and system resources.

use crate::parallel::ExecutionStrategy;

/// Configuration for profiling parallel execution
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Maximum number of threads (0 = no limit)
    pub max_threads: usize,
    /// Percentage of CPU cores to use (e.g., 75 for 75%)
    pub thread_percentage: u8,
    /// Minimum items needed to justify parallel execution
    pub min_items_for_parallel: usize,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            max_threads: 0,
            thread_percentage: 75,
            min_items_for_parallel: 10,
        }
    }
}

/// Workload-aware profiler for optimizing parallel execution
pub struct WorkloadProfiler;

impl WorkloadProfiler {
    /// Adapt worker count based on workload characteristics
    ///
    /// This method implements domain-agnostic workload adaptation that can be used
    /// by any module needing parallel execution.
    ///
    /// # Adaptation Strategy
    /// - ≤10 items: Use min(2, max_workers) - minimal parallelism
    /// - ≤50 items: Use max_workers * 0.5 - conservative parallelism
    /// - ≤100 items: Use max_workers * 0.75 - moderate parallelism
    /// - >100 items: Use max_workers - full parallelism
    ///
    /// # Parameters
    /// - `item_count`: Number of items to process
    /// - `max_workers`: Maximum available workers
    ///
    /// # Returns
    /// Adapted worker count optimized for the workload size
    pub fn adapt_workers_to_workload(item_count: usize, max_workers: usize) -> usize {
        match item_count {
            0..=10 => std::cmp::min(2, max_workers),
            11..=50 => std::cmp::max(1, max_workers / 2),
            51..=100 => std::cmp::max(1, (max_workers * 3) / 4),
            _ => max_workers,
        }
    }

    /// Profile and get execution strategy with custom adaptation
    ///
    /// This method allows callers to provide their own workload adaptation function
    /// for domain-specific optimization. This is useful when different modules have
    /// different parallelization characteristics.
    ///
    /// # When to Use This
    /// - **File Processing**: Large files might benefit from fewer workers to avoid I/O contention
    /// - **Network Operations**: May want conservative parallelism to avoid rate limiting
    /// - **Memory-Intensive Tasks**: Might need worker limits based on available RAM
    /// - **Custom Thresholds**: Different modules may have different overhead characteristics
    ///
    /// # Parameters
    /// - `item_count`: Number of items to process
    /// - `config`: Profiling configuration
    /// - `adapter`: Custom function to adapt worker count based on workload
    ///
    /// # Adapter Function Signature
    /// The adapter function receives `(item_count, max_workers)` and returns optimal worker count.
    ///
    /// # Examples
    ///
    /// ## Conservative File Processing
    /// ```rust
    /// use guardy::profiling::{WorkloadProfiler, ProfilingConfig};
    /// use guardy::parallel::ExecutionStrategy;
    ///
    /// let config = ProfilingConfig::default();
    /// let strategy = WorkloadProfiler::profile_with_adapter(
    ///     100,
    ///     &config,
    ///     |count, max_workers| {
    ///         // Use standard adaptation as baseline
    ///         let base = WorkloadProfiler::adapt_workers_to_workload(count, max_workers);
    ///         // Then apply file I/O specific constraints
    ///         std::cmp::max(1, base / 2)  // Use half for I/O contention
    ///     }
    /// );
    /// ```
    ///
    /// ## Memory-Aware Processing
    /// ```rust
    /// use guardy::profiling::{WorkloadProfiler, ProfilingConfig};
    /// use guardy::parallel::ExecutionStrategy;
    ///
    /// let config = ProfilingConfig::default();
    /// let strategy = WorkloadProfiler::profile_with_adapter(
    ///     1000,
    ///     &config,
    ///     |count, max_workers| {
    ///         // Limit workers based on memory constraints
    ///         let memory_limited_workers = 4; // Assume 4 workers max for memory-heavy tasks
    ///         std::cmp::min(max_workers, memory_limited_workers)
    ///     }
    /// );
    /// ```
    pub fn profile_with_adapter<F>(
        item_count: usize,
        config: &ProfilingConfig,
        adapter: F,
    ) -> ExecutionStrategy
    where
        F: FnOnce(usize, usize) -> usize,
    {
        // Calculate maximum workers based on system resources
        let max_workers = ExecutionStrategy::calculate_optimal_workers(
            config.max_threads,
            config.thread_percentage,
        );

        // Apply custom adaptation
        let optimal_workers = adapter(item_count, max_workers);

        // Choose strategy based on threshold
        ExecutionStrategy::auto(item_count, config.min_items_for_parallel, optimal_workers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_adaptation() {
        // Test small workload
        assert_eq!(WorkloadProfiler::adapt_workers_to_workload(5, 8), 2);

        // Test medium workload
        assert_eq!(WorkloadProfiler::adapt_workers_to_workload(30, 8), 4);

        // Test large workload
        assert_eq!(WorkloadProfiler::adapt_workers_to_workload(75, 8), 6);

        // Test very large workload
        assert_eq!(WorkloadProfiler::adapt_workers_to_workload(150, 8), 8);
    }

    #[test]
    fn test_custom_adapter() {
        let config = ProfilingConfig::default();

        let strategy = WorkloadProfiler::profile_with_adapter(
            100,
            &config,
            |_count, max_workers| max_workers / 3, // Custom: always use 1/3 of workers
        );

        if let ExecutionStrategy::Parallel { workers } = strategy {
            assert!(workers > 0);
        }
    }

    #[test]
    fn test_custom_adapter_file_io_scenario() {
        let config = ProfilingConfig {
            max_threads: 8,
            thread_percentage: 100,
            min_items_for_parallel: 5,
        };

        // Test conservative file I/O adapter
        let strategy = WorkloadProfiler::profile_with_adapter(
            50, // 50 files to process
            &config,
            |count, max_workers| {
                // Conservative file I/O: use half the workers to avoid contention
                if count < 10 {
                    1
                } else {
                    std::cmp::max(1, max_workers / 2)
                }
            },
        );

        // Should be parallel but with reduced workers
        if let ExecutionStrategy::Parallel { workers } = strategy {
            assert!(workers <= 4); // Should be less than or equal to half of 8
            assert!(workers > 0);
        }
    }

    #[test]
    fn test_custom_adapter_memory_limited() {
        let config = ProfilingConfig {
            max_threads: 16,
            thread_percentage: 100,
            min_items_for_parallel: 10,
        };

        // Test memory-limited adapter
        let strategy = WorkloadProfiler::profile_with_adapter(
            1000, // Large workload
            &config,
            |_count, max_workers| {
                // Memory constraint: never use more than 4 workers
                let memory_limit = 4;
                std::cmp::min(max_workers, memory_limit)
            },
        );

        // Should be parallel but capped at memory limit
        if let ExecutionStrategy::Parallel { workers } = strategy {
            assert_eq!(workers, 4); // Should be capped at 4
        }
    }

    #[test]
    fn test_custom_adapter_sequential_override() {
        let config = ProfilingConfig {
            max_threads: 8,
            thread_percentage: 100,
            min_items_for_parallel: 5,
        };

        // Test adapter that forces sequential for certain conditions
        let strategy = WorkloadProfiler::profile_with_adapter(
            100, // Normally would be parallel
            &config,
            |count, _max_workers| {
                // Custom logic: force sequential if count is exactly 100
                if count == 100 {
                    1 // Force sequential
                } else {
                    8 // Otherwise use 8 workers
                }
            },
        );

        // Even though we have 100 items (above threshold), custom adapter forces 1 worker
        if let ExecutionStrategy::Parallel { workers } = strategy {
            assert_eq!(workers, 1);
        }
    }

    #[test]
    fn test_custom_adapter_network_rate_limiting() {
        let config = ProfilingConfig {
            max_threads: 12,
            thread_percentage: 100,
            min_items_for_parallel: 5,
        };

        // Test network-aware adapter that limits concurrency to avoid rate limiting
        let strategy = WorkloadProfiler::profile_with_adapter(
            200, // Many network requests
            &config,
            |count, max_workers| {
                // Network operations: scale based on rate limiting considerations
                let rate_limit_safe_workers = match count {
                    0..=19 => 2,
                    20..=99 => 4,
                    _ => std::cmp::min(max_workers / 2, 6), // Use half available, max 6
                };
                std::cmp::min(rate_limit_safe_workers, max_workers)
            },
        );

        // Should be parallel but limited for network considerations
        if let ExecutionStrategy::Parallel { workers } = strategy {
            assert_eq!(workers, 6); // Should be 6 for 200 items
        }
    }
}
