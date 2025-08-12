# Static Data Refactor Plan for Scan v3

## Issues Identified

1. **Naming**: `scanner.rs` should be `core.rs` for consistency
2. **Static Data Architecture**: Need base + custom pattern like v2
3. **Zero-Copy Optimization**: Need to review all data structures
4. **Configuration Handling**: Should be part of static classes

## Static Classes Architecture (from v2 analysis)

### Pattern from v2:
```rust
static STATIC_DATA: LazyLock<Arc<DataType>> = LazyLock::new(|| {
    // 1. Load base/default data (always available)
    let base_data = load_defaults();
    
    // 2. Load custom/user data (optional, may fail)
    let custom_data = load_custom().unwrap_or_default();
    
    // 3. Merge base + custom
    let merged = merge(base_data, custom_data);
    
    // 4. Return Arc for zero-copy sharing
    Arc::new(merged)
});
```

## Proposed Static Classes

### 1. PatternLibrary
```rust
// static_data/pattern_library.rs
static PATTERN_LIBRARY: LazyLock<Arc<PatternLibrary>> = LazyLock::new(|| {
    // Base patterns (embedded YAML)
    // Custom patterns (from config)
    // Compiled regex patterns
    // Keywords for Aho-Corasick
});

pub struct PatternLibrary {
    patterns: Vec<CompiledPattern>,
    keywords: Vec<String>,           // For Aho-Corasick
    pattern_map: HashMap<usize, Arc<CompiledPattern>>, // Index -> Pattern
}
```

### 2. BinaryExtensions
```rust
// static_data/binary_extensions.rs
static BINARY_EXTENSIONS: LazyLock<Arc<HashSet<String>>> = LazyLock::new(|| {
    // Base extensions (hardcoded)
    // Custom extensions (from config)
    // Merged into HashSet for O(1) lookup
});
```

### 3. EntropyPatterns
```rust
// static_data/entropy_patterns.rs
static ENTROPY_PATTERNS: LazyLock<Arc<EntropyConfig>> = LazyLock::new(|| {
    // Pattern-specific entropy thresholds
    // Base64/Hex detection patterns
    // Statistical thresholds
});

pub struct EntropyConfig {
    thresholds: HashMap<String, f64>,  // Pattern type -> threshold
    base64_pattern: Regex,
    hex_pattern: Regex,
}
```

### 4. Configuration
```rust
// static_data/configuration.rs
static SCANNER_CONFIG: LazyLock<Arc<ScannerConfig>> = LazyLock::new(|| {
    // Global scanner configuration
    // Loaded once, shared everywhere
});
```

## Zero-Copy Data Structure Optimizations

### Current Issues & Fixes:

1. **SecretMatch**
   - ✅ Already uses `Arc<str>` for file_path, secret_type, pattern_description
   - ❌ `line_content` is `String` (copies data)
   - **Fix**: Use `Arc<str>` or consider storing line range instead

2. **FileResult**
   - ✅ Uses `Arc<str>` for file_path
   - ❌ `matches: Vec<SecretMatch>` could be expensive to clone
   - **Fix**: Consider `Arc<[SecretMatch]>` if results are shared

3. **ScanResult**
   - ❌ `matches: Vec<SecretMatch>` - potentially large
   - ❌ `file_results: Vec<FileResult>` - potentially large
   - **Fix**: Use `Arc<[SecretMatch]>` and `Arc<[FileResult]>` for immutable results

4. **PathBuf vs Arc<Path>**
   - Current: Using `PathBuf` which allocates
   - **Fix**: Use `Arc<Path>` or `Arc<str>` for paths

5. **Pattern Strings**
   - Current: Cloning strings for patterns
   - **Fix**: Use `Arc<str>` or `&'static str` where possible

## Implementation Steps

### Step 1: Rename scanner.rs to core.rs
```bash
mv src/scan/scanner.rs src/scan/core.rs
# Update mod.rs imports
```

### Step 2: Implement Static Data Classes
1. PatternLibrary with base + custom patterns
2. BinaryExtensions with base + custom extensions  
3. EntropyPatterns with configurable thresholds
4. Configuration as static class

### Step 3: Zero-Copy Optimizations
1. Change `String` to `Arc<str>` where data is shared
2. Use `Arc<[T]>` for immutable arrays
3. Replace `PathBuf` with `Arc<Path>` where appropriate
4. Use `Cow<'a, str>` for data that might be borrowed or owned

### Step 4: Update Pipelines
1. DirectoryPipeline to use static BinaryExtensions
2. FilePipeline to use static PatternLibrary
3. Filters to reference static data via Arc

## Memory Layout After Optimization

```
STATIC DATA (shared via Arc, never copied):
├── PatternLibrary (Arc<PatternLibrary>)
│   ├── patterns: Vec<CompiledPattern>
│   ├── keywords: Vec<String>
│   └── aho_corasick: AhoCorasick
├── BinaryExtensions (Arc<HashSet<String>>)
├── EntropyPatterns (Arc<EntropyConfig>)
└── Configuration (Arc<ScannerConfig>)

PER-SCAN DATA (minimal copying):
├── FileResult
│   ├── file_path: Arc<str>  // Shared
│   ├── matches: Vec<SecretMatch>
│   └── stats: FileStats
└── SecretMatch
    ├── file_path: Arc<str>  // Shared with FileResult
    ├── line_content: Arc<str>  // Shared if same line
    ├── secret_type: Arc<str>  // Shared from PatternLibrary
    └── pattern_description: Arc<str>  // Shared from PatternLibrary
```

## Benefits

1. **Memory Efficiency**: 
   - Static data compiled once, shared everywhere
   - Arc prevents copying large data structures
   - String interning for repeated values

2. **Performance**:
   - O(1) HashSet lookups for binary extensions
   - Compiled patterns cached globally
   - Zero-copy access to static data

3. **Maintainability**:
   - Clear separation of static vs dynamic data
   - Base + custom pattern is extensible
   - Consistent architecture across all static classes

## Testing Strategy

1. Verify Arc reference counting works correctly
2. Benchmark memory usage before/after
3. Ensure custom patterns/extensions load correctly
4. Test thread safety of static data access