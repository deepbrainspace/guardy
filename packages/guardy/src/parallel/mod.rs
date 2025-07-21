//! Generic parallel execution framework
//! 
//! This module provides a reusable parallel processing infrastructure that can be used
//! by any module in the codebase that needs to process work items in parallel.
//! 
//! # Architecture Responsibilities
//! 
//! The parallel module focuses exclusively on **system resource management** and **execution strategy**:
//! 
//! ## What This Module Does:
//! - **Resource Discovery**: Detects available CPU cores using `num_cpus::get()`
//! - **Resource Calculation**: Applies user configuration (thread percentage, max threads) to available resources
//! - **Execution Strategy**: Provides Sequential vs Parallel execution with worker management
//! - **Thread Safety**: Manages crossbeam channels and worker coordination
//! 
//! ## What This Module Does NOT Do:
//! - **Domain Logic**: Does not understand file counts, scanning workloads, or application-specific constraints
//! - **Workload Analysis**: Does not make decisions about when to use parallel vs sequential based on work characteristics
//! - **Adaptive Scaling**: Does not adjust worker counts based on work item counts or types
//! 
//! # Separation of Concerns
//! 
//! This design follows clear separation of concerns:
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   Client        │    │   Parallel       │    │   System        │
//! │   (Scanner)     │───▶│   Module         │───▶│   Resources     │
//! │                 │    │                  │    │                 │
//! │ • File counts   │    │ • CPU cores      │    │ • Hardware      │
//! │ • Domain logic  │    │ • Thread mgmt    │    │ • OS limits     │
//! │ • Workload      │    │ • Execution      │    │ • Availability  │
//! │   adaptation    │    │   strategy       │    │                 │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//! ```
//! 
//! # Key Methods
//! 
//! ## Resource Calculation
//! ```rust,no_run
//! use guardy::parallel::ExecutionStrategy;
//! 
//! // Calculate maximum workers based on system resources and config
//! let max_threads_config = 0; // User's thread limit (0 = no limit)
//! let thread_percentage = 75; // Percentage of CPU cores to use (e.g., 75)
//! let max_workers = ExecutionStrategy::calculate_optimal_workers(
//!     max_threads_config,
//!     thread_percentage,
//! );
//! ```
//! 
//! ## Strategy Selection
//! ```rust,no_run
//! use guardy::parallel::ExecutionStrategy;
//! 
//! // Client provides the optimal worker count (after domain-specific adaptation)
//! let work_item_count = 100; // Total items to process
//! let min_threshold = 50; // Minimum items needed for parallel to be worthwhile
//! let optimal_workers = 8; // Pre-calculated optimal workers for this workload
//! let strategy = ExecutionStrategy::auto(
//!     work_item_count,
//!     min_threshold,
//!     optimal_workers,
//! );
//! ```
//! 
//! # Features
//! - Generic parallel and sequential execution strategies
//! - Resource-aware worker calculation
//! - Configurable progress reporting
//! - Worker thread management with crossbeam channels
//! - Threshold-based strategy selection
//! 
//! # Example Usage
//! 
//! ```rust
//! use guardy::parallel::ExecutionStrategy;
//! 
//! // Auto strategy selects parallel vs sequential based on workload  
//! let strategy = ExecutionStrategy::auto(100, 50, 8);
//! 
//! // Or explicitly choose parallel with specific worker count
//! let parallel_strategy = ExecutionStrategy::Parallel { workers: 4 };
//! 
//! // Or force sequential processing
//! let sequential_strategy = ExecutionStrategy::Sequential;
//! ```

pub mod core;
pub mod progress;

// Re-export main types for easier access
pub use core::ExecutionStrategy;