# Guardy Benchmarks

Comprehensive performance benchmarks comparing guardy against lefthook and other git hook tools.

## Overview

This benchmark suite measures:

1. **Hook Execution Performance** - Comparing guardy vs lefthook for git hooks
2. **Secret Scanning Performance** - Guardy's core strength vs other tools  
3. **Parallel Execution Efficiency** - Worker scaling and resource utilization
4. **Memory Usage** - Resource consumption patterns
5. **Cold vs Warm Cache** - OS filesystem caching benefits

## Benchmark Categories

### 1. Hook Execution Benchmarks (`hooks/`)
- **Small projects** (10-100 files): Startup overhead comparison
- **Medium projects** (1k-10k files): Balanced workload performance  
- **Large projects** (10k+ files): Scalability and parallelism benefits
- **Custom commands**: Shell command execution overhead
- **Built-in actions**: Secret scanning, commit validation

### 2. Secret Scanning Benchmarks (`scanning/`)
- **File processing speed**: Files per second across different sizes
- **Pattern matching efficiency**: Regex performance vs competitors
- **Entropy analysis**: Statistical secret detection overhead
- **False positive rates**: Accuracy vs speed trade-offs

### 3. System Resource Benchmarks (`resources/`)
- **Memory usage**: Peak and average memory consumption
- **CPU utilization**: Core usage patterns and efficiency
- **I/O performance**: Disk read patterns and caching benefits
- **Startup time**: Cold start vs warm start performance

### 4. Real-World Scenarios (`scenarios/`)
- **Monorepo**: Large codebases with multiple projects
- **Microservices**: Many small repositories
- **Legacy projects**: Large files, binary content, deep directories
- **CI/CD**: Repeated scans in containerized environments

## Running Benchmarks

### Prerequisites
```bash
# Install required tools
cargo install hyperfine  # For timing benchmarks
cargo install valgrind   # For memory profiling (Linux)

# Install lefthook for comparison
curl -1sLf 'https://dl.cloudsmith.io/public/evilmartians/lefthook/setup.deb.sh' | sudo -E bash
sudo apt install lefthook
```

### Quick Start
```bash
# Run all benchmarks
cargo run --release --bin benchmark-runner

# Run specific category
cargo run --release --bin benchmark-runner -- hooks
cargo run --release --bin benchmark-runner -- scanning
cargo run --release --bin benchmark-runner -- resources

# Compare specific tools
cargo run --release --bin benchmark-runner -- --tools guardy,lefthook

# Generate detailed reports
cargo run --release --bin benchmark-runner -- --output-format json --output results.json
```

### Individual Benchmarks
```bash
# Hook execution performance
hyperfine --warmup 3 'guardy run pre-commit' 'lefthook run pre-commit'

# Secret scanning speed
hyperfine --warmup 1 --parameter-list size small,medium,large \
  'guardy scan test-repos/{size} --quiet' \
  'gitleaks detect --source test-repos/{size} --no-git'

# Memory usage comparison
valgrind --tool=massif --massif-out-file=guardy.out guardy scan large-repo/
valgrind --tool=massif --massif-out-file=lefthook.out lefthook run pre-commit
```

## Expected Performance Results

Based on architecture and real-world testing:

**Guardy Advantages:**
- **OS Cache Optimization**: 2.7x performance improvement (1,900 → 5,200 files/sec)
- **Parallel execution**: Better CPU utilization on multi-core systems  
- **Memory efficiency**: <200MB for 100k+ file repositories
- **Rust performance**: Native speed vs interpreted/VM languages

**Comparison Metrics:**
- **Hook execution**: Startup time, command coordination, parallel efficiency
- **Secret scanning**: Files/second, accuracy, memory usage
- **Resource usage**: CPU utilization, memory consumption, I/O patterns