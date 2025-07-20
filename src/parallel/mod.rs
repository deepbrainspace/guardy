pub mod processor;

// Re-export main types for easier access
pub use processor::{ParallelProcessor, ParallelConfig, WorkResult, process_parallel};