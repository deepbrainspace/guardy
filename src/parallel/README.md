# Parallel Processing Framework

This module provides a reusable parallel processing framework for distributing work across multiple CPU cores using crossbeam channels and worker threads.

## Features

- **Generic Work Distribution**: Process any type of work items with any result type
- **Optimal Thread Management**: Automatically calculates optimal worker threads based on CPU cores and configuration
- **Progress Tracking**: Real-time progress updates with atomic counters
- **Error Handling**: Graceful error collection and recovery
- **Ordered Results**: Maintains original order of work items in results
- **Resource Control**: Configurable thread limits and resource usage

## Usage

### Basic Usage

```rust
use crate::parallel::process_parallel;

let items = vec![1, 2, 3, 4, 5];
let results = process_parallel(items, |x| Ok(x * 2), "Processing numbers")?;
// Results: [2, 4, 6, 8, 10]
```

### Advanced Configuration

```rust
use crate::parallel::{ParallelProcessor, ParallelConfig};

let config = ParallelConfig {
    max_threads: 4,
    thread_percentage: 50,
    ..Default::default()
};

let processor = ParallelProcessor::new(config);
let results = processor.process(work_items, worker_fn, "Custom processing")?;
```

## Configuration Options

- `max_threads`: Maximum worker threads (0 = auto-detect)
- `thread_percentage`: Percentage of CPU cores to use (1-100)
- `channel_buffer_multiplier`: Channel buffer size multiplier
- `progress_update_frequency`: Progress update frequency (every N items)

## Performance Characteristics

- **Overhead**: ~10-50ms setup cost for thread spawning
- **Optimal for**: >50 work items with CPU-intensive operations
- **Memory**: Bounded channels prevent memory blowup
- **Scalability**: Linear speedup up to CPU core limit

## Use Cases

1. **File Processing**: Parallel file scanning and analysis
2. **Pattern Matching**: Distribute regex patterns across threads  
3. **Data Transformation**: Parallel data processing pipelines
4. **I/O Operations**: Concurrent file operations with backpressure

## Thread Safety

- Uses `crossbeam::thread::scope` for safe borrowing
- Atomic counters for progress tracking
- Bounded channels prevent deadlocks
- Graceful error propagation and recovery