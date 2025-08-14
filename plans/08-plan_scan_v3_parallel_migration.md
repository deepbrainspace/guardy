# Plan: Scan v3 Migration to Parallel Module

## Status: IN PROGRESS (as of 2025-08-13)

## Background
The scan v3 implementation was initially using rayon's `par_iter` for parallelization, which caused:
1. Poor progress display (single bar instead of per-worker bars)
2. Performance issues (extremely slow on large directories)
3. Inability to show worker-specific progress like scan_v1

The decision was made to migrate scan v3 to use the same parallel module that scan_v1 uses successfully.

## Current State of Migration

### ✅ Completed
1. **Replaced rayon with ExecutionStrategy** - scan v3 now uses `ExecutionStrategy::execute()` instead of `par_iter()`
2. **Added parallel module's progress reporter** - Using `StatisticsProgressReporter` from parallel module
3. **Worker-aware processing** - Each file processor receives `worker_id: usize` parameter
4. **Progress bar styling** - Changed to use hash signs (`##-`) like scan_v1

### ⚠️ Partially Done
1. **ProgressTracker removal** - scan v3's custom ProgressTracker is partially removed but still referenced
2. **Type mismatches** - `max_threads` is `Option<usize>` in v3 but `usize` in parallel module

### ❌ Issues to Fix
1. **Compilation errors**:
   - `ProgressTracker` type not found in scope
   - `max_threads` type mismatch (`Option<usize>` vs `usize`)
2. **Duplicate progress systems** - Both scan v3's ProgressTracker and parallel's StatisticsProgressReporter exist
3. **Discovery phase progress** - Currently no progress shown during file discovery

## Architecture Analysis

### Scan v3 Strengths (to preserve)
- **Pipeline architecture**: DirectoryPipeline → FilePipeline
- **Filter system**: Modular filters for content, directory, binary detection
- **Static data module**: Global configuration management
- **Better separation of concerns**: Clear boundaries between components
- **Zero-copy optimizations**: Arc usage for shared data

### Parallel Module Benefits (why we're migrating)
- **Proven progress system**: Per-worker bars with file display
- **ExecutionStrategy**: Smart sequential/parallel decision based on workload
- **Worker management**: Explicit worker IDs and crossbeam channels
- **Statistics tracking**: Built-in counters for scanned/skipped/binary files

## Migration Plan

### Phase 1: Fix Immediate Compilation Errors ✅ URGENT
```rust
// 1. Fix max_threads type mismatch in core.rs:145
let max_workers_by_resources = ExecutionStrategy::calculate_optimal_workers(
    self.config.max_threads.unwrap_or(0),  // Handle Option<usize>
    self.config.max_cpu_percentage as u8,
);

// 2. Remove ProgressTracker import from core.rs:11-16
// Remove: tracking::ProgressTracker from imports

// 3. Update scan_with_progress signature in core.rs:120-124
pub fn scan_with_progress(
    &self,
    path: &Path,
    // Remove: external_progress: Option<Arc<ProgressTracker>>,
) -> Result<ScanResult> {
```

### Phase 2: Complete Progress System Migration
```rust
// 1. Remove all references to scan v3's ProgressTracker
// - Remove the external_progress parameter completely
// - Remove progress.start_discovery(), finish_discovery(), etc.

// 2. Keep using parallel module's progress exclusively
// Already done in core.rs:156-166

// 3. Optional: Add discovery progress to parallel module (future enhancement)
// Could extend StatisticsProgressReporter with discovery phase
```

### Phase 3: Configuration Alignment
```rust
// Option A: Keep Option<usize> and handle conversion
pub struct ScannerConfig {
    pub max_threads: Option<usize>,  // None = auto-detect
}

// Option B: Change to match scan_v1
pub struct ScannerConfig {
    pub max_threads: usize,  // 0 = auto-detect
}
```

### Phase 4: Testing & Verification
1. **Build successfully**: `cargo build --release`
2. **Test progress display**: 
   ```bash
   ./target/release/guardy scan ../../../ --stats
   ```
   Expected output:
   ```
   [Worker 01] ##########-----  2128/14000  📄 file1.rs
   [Worker 02] #########------  2505/14000  📄 file2.rs
   Overall: [00:00:21] #######--- 27290/178407 files (17%)
   ```
3. **Performance test**: Should complete in seconds, not minutes
4. **Compare with scan_v1**: Output should be visually identical

## File Changes Required

### Files to Modify
1. `/home/nsm/code/deepbrain/guardy/packages/guardy/src/scan/core.rs`
   - Remove ProgressTracker import
   - Fix max_threads type handling
   - Remove external_progress parameter
   - Remove progress tracking calls (already using parallel's)

2. `/home/nsm/code/deepbrain/guardy/packages/guardy/src/scan/pipeline/directory.rs`
   - Remove progress parameter from discover_files if not needed

3. `/home/nsm/code/deepbrain/guardy/packages/guardy/src/scan/tracking/` (entire directory)
   - Consider marking as deprecated or removing entirely
   - Could keep for future enhancements if needed

### Files to Keep Unchanged
- `/home/nsm/code/deepbrain/guardy/packages/guardy/src/parallel/` - Working perfectly
- All pipeline files - Good architecture to preserve
- All filter files - Good modular design

## Decision Points

### Q1: What to do with scan v3's ProgressTracker?
**Decision: REMOVE ENTIRELY**
- Reason: Parallel module's progress is proven and gives exact v1 output
- Alternative considered: Hybrid approach (keep for discovery) - too complex

### Q2: How to handle max_threads Option<usize> vs usize?
**Decision: Keep Option<usize> but handle conversion**
- Reason: More Rust-idiomatic, None = auto-detect is clearer than 0
- Implementation: Use `.unwrap_or(0)` when calling parallel module

### Q3: Discovery phase progress?
**Decision: Skip for now, add later if needed**
- Reason: Discovery is usually fast, not critical for MVP
- Future: Could extend parallel module with discovery support

## Success Criteria
1. ✅ Compilation successful without warnings
2. ✅ Per-worker progress bars display during scanning
3. ✅ Performance matches or exceeds scan_v1 
4. ✅ Output format identical to scan_v1
5. ✅ All tests pass

## Next Session Resume Points
If session is interrupted, resume from:
1. Check compilation: `cargo check`
2. Review this plan: `cat .claude/plans/plan_scan_v3_parallel_migration.md`
3. Check current issues: `cargo build 2>&1 | head -20`
4. Continue from the next uncompleted phase

## Commands for Quick Testing
```bash
# Build
cargo build --release --bin guardy

# Test small directory
./target/release/guardy scan ./src --stats

# Test large directory
./target/release/guardy scan ../../../ --stats

# Compare with v1
./target/release/guardy scan-v1 ../../../ --stats
```

## Notes
- The migration is ~70% complete
- Main blockers are simple compilation errors
- Architecture is sound, just need to complete the plumbing
- Performance should be excellent once compilation is fixed