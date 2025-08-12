# Scan2 Refactoring Plan

## Issues Identified During Code Review

After reviewing the complete scan module implementation, several critical refactoring issues have been identified that need to be addressed to achieve the optimal, efficient design we're aiming for.

## Critical Refactoring Tasks

### 1. **Inefficient Filter Creation Pattern**
**Problem**: Creating new filter instances multiple times instead of reusing them
- `strategy.rs:104` - Creating BinaryFilter for each file processed
- `file.rs:48` - Creating BinaryFilter for each file processed  
- `file.rs:198` - Creating BinaryFilter for each file processed

**Solution**: Create filters once in Core scanner and pass references

### 2. **Redundant Binary Filtering**
**Problem**: Binary filtering happens in TWO places:
- Directory-level filtering in `core.rs:249` (correct location)  
- File-level filtering in `strategy.rs:103-120` (redundant)

**Solution**: Remove redundant binary filtering from strategy.rs since it's already handled in directory filters

### 3. **Inconsistent Method Naming and API**
**Problem**: Mix of static vs instance methods creates confusion
- `Pattern::filter_by_keywords()` - static method but unclear API
- Context filter uses wrong method name in file.rs
- Missing proper method implementations

**Solution**: Standardize on instance-based API with clear method names

### 4. **Unimplemented Placeholder Functions**
**Problem**: Critical functions are TODOs that break functionality
- `directory.rs:28` - `fast_count_files()` returns hardcoded 100
- `directory.rs:36` - `analyze_paths()` returns empty vec
- Several other TODO comments in filters

**Solution**: Either implement properly or remove unused functions

### 5. **Type Mismatches and Compilation Errors**
**Problem**: Various type issues preventing compilation
- `file.rs:93` - Comparing f64 to usize for file size
- Missing imports and unused imports
- Lifetime issues in filter methods (partially fixed)

**Solution**: Fix all type mismatches and clean up imports

### 6. **Inefficient Pattern Filtering Logic**
**Problem**: Current context prefiltering logic is inefficient
- Creating context filter for each file instead of reusing
- Complex index-based pattern selection instead of direct references
- Missing proper integration between Aho-Corasick and regex patterns

**Solution**: Optimize pattern filtering with proper caching and direct references

### 7. **Missing Progress Integration**
**Problem**: Progress system exists but isn't properly connected
- Progress updates not happening in correct places
- Statistics tracking incomplete
- Worker progress not properly coordinated

**Solution**: Integrate progress system properly throughout pipeline

### 8. **Architecture Violations**
**Problem**: Code violates the clean architecture we designed
- Direct module-to-module calls instead of dependency injection
- Tight coupling between modules
- Missing proper error handling patterns

**Solution**: Refactor to follow dependency injection and clean architecture

## Refactoring Strategy

### Phase 1: Fix Compilation Errors
1. Fix all type mismatches and imports
2. Remove unused functions and TODOs
3. Standardize method signatures
4. Ensure clean compilation

### Phase 2: Optimize Filter Patterns
1. Implement proper filter reuse in Core scanner
2. Remove redundant binary filtering from strategy.rs
3. Fix context prefiltering integration
4. Optimize pattern selection logic

### Phase 3: Architecture Cleanup
1. Implement dependency injection for filters
2. Standardize error handling patterns
3. Clean up module coupling
4. Remove legacy compatibility code

### Phase 4: Progress Integration
1. Connect progress system properly
2. Implement proper statistics tracking
3. Fix worker coordination
4. Test progress reporting

## Success Criteria

- [ ] Clean compilation with no warnings
- [ ] All filters created once and reused efficiently
- [ ] No redundant processing anywhere in pipeline
- [ ] Proper progress reporting throughout scan
- [ ] 5x performance improvement over original scanner
- [ ] Clean, maintainable architecture following OOP principles

## Next Steps

1. Complete Phase 1 compilation fixes
2. Test each refactoring incrementally
3. Validate performance improvements
4. Ensure no regressions in detection accuracy