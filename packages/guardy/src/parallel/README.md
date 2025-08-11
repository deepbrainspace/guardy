# Parallel Module

A generic parallel execution framework for processing work items across multiple threads.

## Overview

This module provides reusable parallel processing infrastructure that can be used by any part of the codebase needing parallel execution. It separates **system resource management** from **domain-specific logic**.

## Architecture Responsibilities

### What This Module Does:
- **Resource Discovery**: Detects available CPU cores using `num_cpus::get()`
- **Resource Calculation**: Applies user configuration (thread percentage, max threads) to available resources
- **Execution Strategy**: Provides Sequential vs Parallel execution with worker management
- **Thread Safety**: Manages crossbeam channels and worker coordination

### What This Module Does NOT Do:
- **Domain Logic**: Does not understand file counts, scanning workloads, or application-specific constraints
- **Workload Analysis**: Does not make decisions about when to use parallel vs sequential based on work characteristics
- **Adaptive Scaling**: Does not adjust worker counts based on work item counts or types

## Key Components

### `ExecutionStrategy` Enum
- `Sequential`: Single-threaded execution
- `Parallel { workers: usize }`: Multi-threaded execution with specified worker count

### Key Methods

#### `calculate_optimal_workers(max_threads_config, thread_percentage) -> usize`
Calculates maximum workers based on system resources and user configuration:
- Detects CPU cores
- Applies thread percentage (e.g., 75% of cores)
- Respects user-defined maximum thread limits
- Always returns at least 1 worker

#### `auto(work_items_count, min_threshold, optimal_workers) -> ExecutionStrategy`
Threshold-based strategy selection:
- If `work_items_count >= min_threshold`: Returns `Parallel { workers: optimal_workers }`
- Otherwise: Returns `Sequential`
- Client must provide pre-calculated optimal worker count

### Progress Reporting
The module includes configurable progress reporting with:
- Frequency control (report every N items)
- Custom icons (⚡ for parallel, ⏳ for sequential)
- Worker ID tracking
- Thread-safe progress updates

## Design Principles

### Separation of Concerns
```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client        │    │   Parallel       │    │   System        │
│   (Scanner)     │───▶│   Module         │───▶│   Resources     │
│                 │    │                  │    │                 │
│ • File counts   │    │ • CPU cores      │    │ • Hardware      │
│ • Domain logic  │    │ • Thread mgmt    │    │ • OS limits     │
│ • Workload      │    │ • Execution      │    │ • Availability  │
│   adaptation    │    │   strategy       │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Resource-First Approach
1. **System Resource Calculation**: `calculate_optimal_workers()` determines maximum possible workers
2. **Domain Adaptation**: Client applies domain-specific constraints
3. **Strategy Selection**: `auto()` makes threshold-based decision using adapted worker count

## Usage Examples

### Basic Usage
```rust
use guardy::parallel::ExecutionStrategy;

// Calculate system resource limits
let max_workers = ExecutionStrategy::calculate_optimal_workers(0, 75);

// Client applies domain adaptation (e.g., based on file count)
let optimal_workers = std::cmp::min(6, max_workers);

// Choose strategy based on workload threshold
let strategy = ExecutionStrategy::auto(100, 50, optimal_workers);

// Execute work items
let results = strategy.execute(
    work_items,
    |item| process_item(item),
    Some(progress_reporter)
)?;
```

### Manual Strategy Selection
```rust
// Force sequential execution
let strategy = ExecutionStrategy::Sequential;

// Force parallel execution with specific worker count
let strategy = ExecutionStrategy::Parallel { workers: 4 };
```

## Notes for AI Assistants and Developers

### 🤖 AI Assistant Guidelines

#### When to Use This Module:
- Any task requiring parallel processing of work items
- When you need resource-aware worker calculation
- When implementing threshold-based parallel/sequential decisions

#### What NOT to Put Here:
- **Domain-specific logic** (file scanning, business rules, etc.)
- **Application-specific worker adaptation** (based on file counts, etc.)
- **Custom thresholds** (let clients decide when to go parallel)

#### Key Design Patterns:
1. **Resource Calculation First**: Always call `calculate_optimal_workers()` before domain adaptation
2. **Client Adaptation**: Let calling modules handle domain-specific worker adjustments
3. **Threshold Decision Last**: Use `auto()` for simple threshold-based strategy selection

#### Common Integration Pattern:
```rust
// 1. Calculate system resources (in parallel module)
let max_workers = ExecutionStrategy::calculate_optimal_workers(config.max_threads, config.thread_percentage);

// 2. Apply domain adaptation (in client module)
let optimal_workers = MyModule::adapt_workers_for_domain(workload_size, max_workers);

// 3. Strategy selection (in parallel module)
let strategy = ExecutionStrategy::auto(workload_size, threshold, optimal_workers);
```

### 🔧 Development Guidelines

#### Adding New Features:
- **Progress Reporting**: Extend `ProgressReporter` trait for new display types
- **Execution Strategies**: Add new variants to `ExecutionStrategy` enum
- **Resource Detection**: Enhance `calculate_optimal_workers()` for new resource types

#### Performance Considerations:
- Worker count is automatically capped by work item count in `ParallelExecutor`
- Progress reporting frequency is configurable to reduce contention
- Uses crossbeam channels for efficient producer-consumer pattern

#### Testing Strategy:
- Unit tests for `ExecutionStrategy` logic
- Integration tests with mock work items
- Doctest examples for all public APIs
- Performance benchmarks for worker scaling

### 🚨 Common Pitfalls to Avoid

1. **Don't put domain logic here**: File scanning logic belongs in scanner module
2. **Don't hardcode thresholds**: Let clients provide their own decision criteria
3. **Don't bypass resource calculation**: Always respect system limitations
4. **Don't assume optimal worker counts**: Different workloads need different strategies

### 🔄 Related Modules

- **Scanner Module**: Primary consumer, handles file scanning workloads
- **Config Module**: Provides thread percentage and max thread configuration
- **Progress Module**: Handles user-facing progress display

## Future Enhancements

- [ ] Support for priority-based work item ordering
- [ ] Dynamic worker scaling based on work item processing time
- [ ] Memory-aware worker calculation
- [ ] Integration with async/await patterns
- [ ] Custom channel buffer size configuration
