# Scan v3 Implementation Plan

## Executive Summary

Based on analysis of v1 scanner and v2 scan modules, and incorporating user feedback, this document outlines the complete implementation plan for Scan v3. The plan eliminates unnecessary complexity while maintaining essential functionality.

## Key Architectural Decisions

### 1. Strategy Class - ELIMINATED ✅
**Rationale:** User's analysis is correct. The Strategy class adds unnecessary abstraction:
- Worker calculation belongs in `system-profile` crate (already exists)
- Single parallel execution path eliminates code duplication
- Rayon automatically handles worker distribution based on file sizes
- Setting workers=1 achieves sequential execution when needed

### 2. Parallel Processing - FILE LEVEL ONLY ✅
**Implementation:** 
```rust
files.par_iter().map(|file| {
    let active_patterns = aho_corasick_prefilter(file);  // Single pass
    for pattern in active_patterns {                      // Sequential
        pattern.find_matches(file)
    }
})
```
**Rationale:** v1 proved pattern-level parallelism is 10x slower (line 554 scanner/core.rs)

### 3. Test Detection - EXCLUDED ✅
**Rationale:** Per user requirement - clients need to know about secrets in tests
- Users can override with `guardy:ignore` comments if needed
- Eliminates 484 lines of complex test detection code

### 4. Ignore System - SIMPLIFIED ✅
**Implementation:** Comment-based only (`guardy:ignore`, `guardy:ignore-next`)
- No complex pattern ignores (DEMO_KEY_, FAKE_)
- Handled entirely within CommentFilter

### 5. File Streaming - NOT NEEDED ✅
**Implementation:** Simple size limit with configurable max (default 50MB)
- Files > limit return error
- User can increase limit in config if needed
- Eliminates streaming complexity

## Core Architecture

### Trait System (3 Main Interfaces)

```rust
// 1. Base Filter trait
pub trait Filter {
    type Input;
    type Output;
    
    fn filter(&self, input: Self::Input) -> Result<Self::Output>;
    fn name(&self) -> &'static str;
}

// 2. Directory-level filter trait
pub trait DirectoryFilter: Filter {
    type Input = PathBuf;
    type Output = FilterDecision;
}

// 3. Content-level filter trait  
pub trait ContentFilter: Filter {
    type Input = FileContent;
    type Output = Vec<SecretMatch>;
}
```

### Module Structure

```
src/scan/
├── mod.rs                    // Public API exports
├── scanner.rs                // Main Scanner struct
├── config.rs                 // ScannerConfig
│
├── pipeline/
│   ├── mod.rs
│   ├── directory.rs          // DirectoryPipeline (orchestrates directory filters)
│   └── file.rs              // FilePipeline (orchestrates content filters)
│
├── filters/
│   ├── mod.rs
│   ├── traits.rs            // Filter, DirectoryFilter, ContentFilter traits
│   ├── directory/
│   │   ├── mod.rs
│   │   ├── path.rs         // PathFilter (gitignore, custom patterns)
│   │   ├── size.rs         // SizeFilter (max file size)
│   │   └── binary.rs       // BinaryFilter (skip binary files)
│   └── content/
│       ├── mod.rs
│       ├── prefilter.rs    // ContextPrefilter (Aho-Corasick)
│       ├── regex.rs        // RegexExecutor (actual pattern matching)
│       ├── comment.rs      // CommentFilter (guardy:ignore directives)
│       └── entropy.rs      // EntropyFilter (Shannon entropy validation)
│
├── static/                  // Shared immutable data
│   ├── mod.rs
│   ├── pattern_library.rs  // Arc<PatternLibrary> - compiled patterns
│   └── binary_extensions.rs // Arc<HashSet<String>> - binary file extensions
│
├── data/                    // Data structures
│   ├── mod.rs
│   ├── scan_result.rs      // ScanResult with hierarchical stats
│   ├── file_result.rs      // FileResult
│   ├── secret_match.rs     // SecretMatch
│   └── stats.rs            // ScanStats, DirectoryStats, FileStats
│
└── tracking/
    ├── mod.rs
    └── progress.rs          // ProgressTracker with indicatif
```

## Parallel Processing Decision

### Current v1 Analysis:
- Uses custom `parallel` module with crossbeam channels
- 200+ lines of complex producer-consumer pattern
- No rayon dependency currently

### Decision: Use Rayon Directly ✅
**Rationale:**
- Rayon's `par_iter()` is simpler and more idiomatic
- Eliminates 200+ lines of custom parallel code
- Better work-stealing algorithm than manual channels
- Automatic worker management based on system resources
- Can still use `RAYON_NUM_THREADS=1` for sequential execution

**Implementation:**
```rust
// Simple rayon usage - no custom parallel module needed
use rayon::prelude::*;

files.par_iter()
    .map(|file| scanner.scan_file(file))
    .collect()
```

## Test Migration Strategy

### Current State:
- Integration tests in `tests/integration/`
- Unit tests scattered throughout modules

### Migration Plan:
1. **Rename existing tests to v2:**
   - `tests/integration/scanner_test.rs` → `tests/integration/scanner_v2_test.rs`
   - Keep v2 tests running during transition

2. **Create new v3 tests:**
   - `tests/integration/scan_v3_test.rs` - new comprehensive tests
   - Start fresh with clean test structure
   - Focus on behavior, not implementation

3. **Cleanup after v3 stable:**
   - Remove v2 tests once v3 is validated
   - Remove old scanner and scan-v2 modules

## Implementation Phases (AI Agent Timeline)

### Phase 1: Foundation (2-3 hours)
**Goal:** Establish core structure and traits

1. **Create base module structure**
   - [ ] Create all directories and mod.rs files
   - [ ] Define public API in root mod.rs

2. **Define core traits**
   - [ ] Implement Filter trait in `filters/traits.rs`
   - [ ] Implement DirectoryFilter trait
   - [ ] Implement ContentFilter trait

3. **Create data structures**
   - [ ] Port ScanResult from v2
   - [ ] Port FileResult from v2
   - [ ] Port SecretMatch from v1
   - [ ] Create hierarchical stats structures

**Validation:** `cargo check` passes

### Phase 2: Static Data & Config (1-2 hours)
**Goal:** Setup shared immutable data and configuration

1. **Static data structures**
   - [ ] Implement PatternLibrary with Arc<LazyLock>
   - [ ] Create binary_extensions with common extensions
   - [ ] Setup pattern compilation from TOML

2. **Configuration**
   - [ ] Port ScannerConfig from v2
   - [ ] Add new config options (thread limit, file size limit)
   - [ ] Create default configuration

**Validation:** Unit tests for pattern compilation

### Phase 3: Directory Pipeline (2-3 hours)
**Goal:** Implement directory traversal and filtering

1. **Directory filters**
   - [ ] Implement PathFilter (gitignore + custom patterns)
   - [ ] Implement SizeFilter (configurable max size)
   - [ ] Implement BinaryFilter (extension-based)

2. **Directory pipeline**
   - [ ] Create DirectoryPipeline orchestrator
   - [ ] Integrate with ignore crate for gitignore
   - [ ] Add parallel file discovery using rayon

**Validation:** Integration test scanning test directory structure

### Phase 4: Content Pipeline - Core (3-4 hours)
**Goal:** Implement core content analysis

1. **Aho-Corasick prefilter**
   - [ ] Build Aho-Corasick automaton from patterns
   - [ ] Return pattern indices for active patterns
   - [ ] Optimize for ~85% pattern elimination

2. **Regex executor**
   - [ ] Sequential pattern matching on active patterns
   - [ ] Capture group extraction
   - [ ] Line number and position tracking

**Validation:** Unit tests with known secrets

### Phase 5: Content Pipeline - Filters (2 hours)
**Goal:** Implement validation filters

1. **Comment filter**
   - [ ] Parse guardy:ignore directives
   - [ ] Handle ignore-next and ignore-line
   - [ ] Track ignore ranges

2. **Entropy filter**
   - [ ] Port Shannon entropy calculation from v1
   - [ ] Configurable thresholds per pattern type
   - [ ] Base64/hex detection optimization

**Validation:** Test with false positive cases

### Phase 6: Scanner Integration (2-3 hours)
**Goal:** Wire everything together

1. **Scanner implementation**
   - [ ] Create main Scanner struct
   - [ ] Wire DirectoryPipeline and FilePipeline
   - [ ] Implement parallel execution with rayon

2. **Progress tracking**
   - [ ] Integrate indicatif progress bars
   - [ ] Multi-level progress (scan/directory/file)
   - [ ] Real-time statistics

**Validation:** End-to-end scanning test

### Phase 7: Testing & Migration (2-3 hours)
**Goal:** Comprehensive testing and migration

1. **Test Migration**
   - [ ] Rename existing tests to v2 suffix
   - [ ] Create new v3 integration tests
   - [ ] Ensure all v3 tests pass

2. **Module Migration**
   - [ ] Update main.rs to use v3
   - [ ] Verify CLI still works
   - [ ] Performance comparison

**Validation:** Full application works with v3

### Phase 8: Cleanup (1 hour)
**Goal:** Remove old code

1. **Cleanup**
   - [ ] Remove scanner (v1) module
   - [ ] Remove scan-v2 module
   - [ ] Remove custom parallel module (if not used elsewhere)
   - [ ] Remove v2 tests
   - [ ] Update documentation

**Validation:** Clean build, all tests pass

## Total Estimated Time: 16-22 hours (2-3 days for AI agent)

## Critical Implementation Notes

### Memory Optimization
- Use `Arc<str>` for file paths to avoid cloning
- Stream large files line-by-line (don't load entire file)
- Clear pattern match vectors between files
- Use SmallVec for small collections

### Performance Targets
- File discovery: < 1ms per 1000 files
- Aho-Corasick prefilter: < 0.1ms per MB
- Full scan: > 100 MB/s on modern hardware
- Memory usage: < 100MB for 10,000 files

### Error Handling
- Use anyhow::Result throughout
- Graceful handling of permission errors
- Continue scanning on individual file errors
- Collect warnings for non-fatal issues

### Testing Strategy
- Unit tests for each filter
- Integration tests for pipelines
- Property-based tests for pattern matching
- Benchmarks for performance regression

## Dependencies

### Required Crates
- `rayon`: Parallel iteration (NEW - replaces custom parallel module)
- `aho-corasick`: Pattern prefiltering  
- `regex`: Pattern matching
- `ignore`: Gitignore handling
- `globset`: Path pattern matching
- `indicatif`: Progress bars
- `arc-swap`: Atomic Arc updates
- `smallvec`: Small vector optimization

### System Dependencies
- `system-profile` crate: CPU/memory detection only

## Success Metrics

1. **Performance**
   - 2x faster than v1 scanner
   - < 100MB memory for large scans
   - Linear scaling with CPU cores

2. **Accuracy**
   - < 5% false positive rate
   - Zero false negatives for known patterns
   - Correct gitignore handling

3. **Maintainability**
   - Clean trait boundaries
   - < 3000 lines total code
   - 80% test coverage

## Risk Mitigation

### Risk: Aho-Corasick construction overhead
**Mitigation:** Cache compiled automaton with LazyLock

### Risk: Memory usage with many files
**Mitigation:** Process in batches, clear intermediate results

### Risk: Pattern regex compilation time
**Mitigation:** Compile once at startup, share with Arc

### Risk: Progress bar overhead
**Mitigation:** Update at intervals, not every file

## Conclusion

This implementation plan provides a clean, efficient v3 scanner that:
- Eliminates unnecessary complexity (Strategy, test detection, streaming)
- Maintains essential functionality (pattern matching, filtering, parallel execution)
- Improves performance through Aho-Corasick prefiltering
- Provides clean trait-based architecture for maintainability

The phased approach ensures each component is properly tested before integration, minimizing risk and ensuring a smooth migration from v1/v2.