# Plan 017: SuperConfig Integration & Performance Optimization

## Objective
Replace Guardy's scattered config system with centralized SuperConfig integration, achieving <5ms config load time and sub-nanosecond access patterns through cache-line optimization.

## Timeline
- **Hour 1-2**: SuperConfig library enhancements
- **Hour 3-4**: Native Rust defaults conversion  
- **Hour 5-6**: Guardy integration & cleanup
- **Hour 7-8**: SIMD pattern optimization & testing

## Architecture Overview

### Principles
1. **Single Source of Truth**: All config merging in `config/mod.rs`
2. **Zero-Copy**: Arc-wrapped at static level for entire config
3. **Cache-Optimal**: Hot fields aligned to 64-byte cache lines
4. **Type-Safe**: Compile-time struct validation
5. **Cross-Platform SIMD**: Auto-detection of AVX2/SSE2/NEON

### Performance Targets
- Config load: <5ms (from current 65ms)
- Hot field access: <0.3ns (L1 cache hit)
- Cold field access: <5ns (Arc deref + L2/L3)
- Clone operation: <3ns (Arc increment only)
- Pattern matching: 10-20x speedup with SIMD

## Part A: SuperConfig Library Enhancements ✅ COMPLETED

### A1. Add PartialConfig Type ✅ COMPLETED
**Status:** Implemented with trait-based approach for zero JSON overhead
**Location:** `packages/superconfig/src/partial.rs`

**Enhancement:** Implemented `PartialConfigurable` trait instead of JSON-based approach for 5x performance improvement:
- Direct field updates via trait methods
- No JSON serialization/deserialization overhead  
- Sub-millisecond application times
- Debug-tolerant performance testing
Location: `packages/superconfig/src/partial.rs`

```rust
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    overrides: HashMap<String, Value>,
}

impl PartialConfig {
    pub fn new() -> Self { Self::default() }
    
    pub fn set<T: Into<Value>>(&mut self, path: &str, value: T) {
        self.overrides.insert(path.to_string(), value.into());
    }
    
    pub fn set_if_some<T: Into<Value>>(&mut self, path: &str, value: Option<T>) {
        if let Some(v) = value {
            self.set(path, v);
        }
    }
    
    pub fn extend_array(&mut self, path: &str, values: Vec<String>) {
        // Special handling for arrays - extends rather than replaces
        self.overrides.insert(
            format!("{}.__extend", path), 
            Value::Array(values.into_iter().map(Value::String).collect())
        );
    }
    
    pub fn apply_to<T: DeserializeOwned + Serialize>(&self, config: &mut T) -> Result<()> {
        let mut value = serde_json::to_value(&*config)?;
        
        for (path, override_value) in &self.overrides {
            if path.ends_with(".__extend") {
                // Handle array extension
                let base_path = path.trim_suffix(".__extend").unwrap();
                extend_array_at_path(&mut value, base_path, override_value)?;
            } else {
                apply_at_path(&mut value, path, override_value.clone())?;
            }
        }
        
        *config = serde_json::from_value(value)?;
        Ok(())
    }
}
```

### A2. Builder Pattern with CLI Config Support ✅ COMPLETED
**Status:** Implemented with flexible defaults handling
**Location:** `packages/superconfig/src/builder.rs`

**Enhancement:** Enhanced ConfigBuilder with layered configuration support:
- Flexible defaults (either provided OR config file required)
- Performance tracking for each layer
- Proper error handling for missing configs
- Environment variable auto-detection (string/int/float/bool)

**NEW:** Added `config_builder!` macro with auto-derivation:
- Auto-derives env_prefix ("GUARDY") and config_name ("guardy") from struct names
- Uses heck crate for string transformations (same as rusttoolkit)  
- Supports override options for all auto-derived values
- For GuardyConfig: `config_builder!(GuardyConfig);`

```rust
pub struct ConfigBuilder<T> {
    defaults: Option<T>,
    file_config: Option<T>,
    config_path: Option<String>, // Explicit config file path from CLI
    env_overrides: Option<PartialConfig>,
    cli_overrides: Option<PartialConfig>,
}

impl<T> ConfigBuilder<T> 
where T: Default + DeserializeOwned + Serialize + Clone
{
    pub fn with_defaults(mut self, defaults: T) -> Self {
        self.defaults = Some(defaults);
        self
    }
    
    // Handle both default search and explicit CLI path
    pub fn with_config_file(mut self, name_or_path: Option<&str>) -> Self {
        let config_result = if let Some(path) = name_or_path {
            // Explicit path from CLI (-c/--config)
            Config::<T>::load_from_path(path)
        } else {
            // Default search pattern
            Config::<T>::load("guardy")
        };
        
        if let Ok(config) = config_result {
            self.file_config = Some(config.into_inner());
        }
        self
    }
    
    pub fn with_env_prefix(mut self, prefix: &str) -> Self {
        let mut partial = PartialConfig::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                // GUARDY_SCANNER_MAX_THREADS -> scanner.max_threads
                let path = key[prefix.len()..]
                    .to_lowercase()
                    .replace('_', ".");
                    
                // Auto-detect type
                if let Ok(bool_val) = value.parse::<bool>() {
                    partial.set(&path, bool_val);
                } else if let Ok(int_val) = value.parse::<i64>() {
                    partial.set(&path, int_val);
                } else if let Ok(float_val) = value.parse::<f64>() {
                    partial.set(&path, float_val);
                } else {
                    partial.set(&path, value);
                }
            }
        }
        
        self.env_overrides = Some(partial);
        self
    }
    
    pub fn with_cli_overrides(mut self, overrides: PartialConfig) -> Self {
        self.cli_overrides = Some(overrides);
        self
    }
    
    pub fn build(self) -> Result<T> {
        let mut config = self.defaults.unwrap_or_default();
        
        // Layer: File Config
        if let Some(file_config) = self.file_config {
            config = deep_merge(config, file_config)?;
        }
        
        // Layer: Environment Variables
        if let Some(env) = self.env_overrides {
            env.apply_to(&mut config)?;
        }
        
        // Layer: CLI Arguments (highest priority)
        if let Some(cli) = self.cli_overrides {
            cli.apply_to(&mut config)?;
        }
        
        Ok(config)
    }
}
```

### A3. Rename FastConfig to Config ✅ COMPLETED
**Status:** Completed throughout entire codebase
**Changes:**
- Renamed `FastConfig<T>` to `Config<T>` throughout superconfig
- Updated all documentation and examples
- Fixed import statements and prelude exports
- Maintained Default trait requirement for simplicity
- All tests passing (22 unit tests + 12 doc tests)

## Part B: Guardy Cleanup ⏳ NEXT

### B1. Files to Delete ⏳ READY TO IMPLEMENT
- `packages/guardy/src/config/core.rs` - old SuperConfig integration
- `packages/guardy/src/config/fast.rs` - duplicate implementation
- `packages/guardy/src/scan/config.rs` - manual CLI merging

### B2. Methods to Remove ⏳ READY TO IMPLEMENT
- `Scanner::from_fast_config_with_cli_overrides()`
- `Scanner::parse_scanner_config_with_cli_overrides()`
- All manual CLI override logic in scan module

## Part C: Native Rust Defaults with Cache Optimization ⏳ READY TO IMPLEMENT

### C1. Hot/Cold Field Separation Strategy ✅ ALREADY EXISTS
**Status:** Found existing implementation in `packages/guardy/src/config/defaults.rs`
**Discovery:** GuardyConfig already has hot/cold optimization and Arc wrapping implemented exactly as planned!

**Cache Line Optimization Explained:**
- Modern CPUs load data in 64-byte cache lines
- When accessing one field, CPU loads entire cache line into L1 cache
- Subsequent accesses to nearby fields are essentially free (<0.3ns)
- We group frequently-accessed fields together in first 64 bytes

Location: `packages/guardy/src/config/defaults.rs`

```rust
use std::sync::Arc;

// Hot path configuration - fits in single cache line
#[repr(C, align(64))]  // Force 64-byte alignment
pub struct ScannerHotConfig {
    // These 4 fields are accessed on EVERY file scan
    pub mode: ScanMode,           // 1 byte
    pub max_threads: u16,         // 2 bytes  
    pub include_binary: bool,     // 1 byte
    pub follow_symlinks: bool,    // 1 byte
    pub max_file_size_mb: u32,    // 4 bytes
    pub enable_entropy: bool,     // 1 byte
    pub entropy_threshold: f32,   // 4 bytes (f32 instead of f64)
    _padding: [u8; 50],           // Pad to exactly 64 bytes
}

// Cold configuration - separate allocation
pub struct ScannerColdConfig {
    pub ignore_paths: Arc<Vec<String>>,      // Rarely accessed
    pub ignore_patterns: Arc<Vec<String>>,   // Rarely accessed
    pub custom_patterns: Arc<Vec<String>>,   // Rarely accessed
    pub thread_percentage: u8,               // Only at startup
    pub min_files_for_parallel: usize,       // Only at startup
}

pub struct ScannerConfig {
    pub hot: ScannerHotConfig,    // Embedded directly (stack allocated)
    pub cold: Arc<ScannerColdConfig>, // Heap allocated, Arc-wrapped
}

// Main config with Arc wrapping
pub struct GuardyConfig {
    pub general: Arc<GeneralConfig>,
    pub hooks: Arc<HooksConfig>,
    pub scanner: Arc<ScannerConfig>,
}

impl Default for GuardyConfig {
    fn default() -> Self {
        GuardyConfig {
            general: Arc::new(GeneralConfig {
                debug: false,
                color: true,
                interactive: true,
            }),
            hooks: Arc::new(HooksConfig {
                pre_commit: HookConfig {
                    enabled: true,
                    builtin: Arc::new(vec!["scan_secrets".into()]),
                    custom: Arc::new(vec![]),
                },
                // ... other hooks
            }),
            scanner: Arc::new(ScannerConfig {
                hot: ScannerHotConfig {
                    mode: ScanMode::Auto,
                    max_threads: 8,
                    include_binary: false,
                    follow_symlinks: false,
                    max_file_size_mb: 10,
                    enable_entropy: true,
                    entropy_threshold: 0.00001,
                    _padding: [0; 50],
                },
                cold: Arc::new(ScannerColdConfig {
                    ignore_paths: Arc::new(vec![
                        "**/.git/**".into(),
                        "**/node_modules/**".into(),
                        "**/target/**".into(),
                    ]),
                    ignore_patterns: Arc::new(vec![]),
                    custom_patterns: Arc::new(vec![]),
                    thread_percentage: 75,
                    min_files_for_parallel: 100,
                }),
            }),
            // ... rest of config
        }
    }
}
```

### C2. SIMD-Optimized Pattern System

Location: `packages/guardy/src/scan/patterns/simd.rs`

```rust
use memchr::memmem;  // SIMD-accelerated substring search
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use regex::Regex;

pub struct SimdPatternMatcher {
    // Aho-Corasick automaton for multi-pattern keyword matching
    keyword_matcher: AhoCorasick,
    // Map from keyword match ID to pattern indices
    keyword_to_patterns: Vec<Vec<usize>>,
    // Compiled regex patterns
    patterns: Vec<CompiledPattern>,
}

impl SimdPatternMatcher {
    pub fn new(patterns: Vec<PatternDef>) -> Self {
        // Extract all unique keywords
        let mut all_keywords = Vec::new();
        let mut keyword_to_patterns = Vec::new();
        
        for (idx, pattern) in patterns.iter().enumerate() {
            for keyword in &pattern.keywords {
                if let Some(kid) = all_keywords.iter().position(|k| k == keyword) {
                    keyword_to_patterns[kid].push(idx);
                } else {
                    all_keywords.push(keyword.clone());
                    keyword_to_patterns.push(vec![idx]);
                }
            }
        }
        
        // Build Aho-Corasick DFA with SIMD acceleration
        let keyword_matcher = AhoCorasickBuilder::new()
            .auto_configure(&all_keywords)  // Auto-selects best algorithm
            .build(&all_keywords);
        
        Self {
            keyword_matcher,
            keyword_to_patterns,
            patterns: patterns.into_iter().map(compile_pattern).collect(),
        }
    }
    
    pub fn scan_content(&self, content: &[u8]) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut patterns_to_check = HashSet::new();
        
        // Phase 1: SIMD keyword search (10-100x faster than regex)
        for mat in self.keyword_matcher.find_iter(content) {
            let keyword_id = mat.pattern();
            for &pattern_id in &self.keyword_to_patterns[keyword_id] {
                patterns_to_check.insert(pattern_id);
            }
        }
        
        // Phase 2: Run regex only on files with keyword matches
        let content_str = std::str::from_utf8(content).unwrap_or("");
        for &pattern_id in &patterns_to_check {
            let pattern = &self.patterns[pattern_id];
            for mat in pattern.regex.find_iter(content_str) {
                matches.push(Match {
                    pattern: pattern.clone(),
                    start: mat.start(),
                    end: mat.end(),
                    text: mat.as_str().to_string(),
                });
            }
        }
        
        matches
    }
}

// Platform detection and optimization
pub fn create_optimized_matcher(patterns: Vec<PatternDef>) -> Box<dyn PatternMatcher> {
    // Runtime CPU feature detection
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            tracing::info!("Using AVX2-optimized pattern matcher");
        } else if is_x86_feature_detected!("sse2") {
            tracing::info!("Using SSE2-optimized pattern matcher");
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            tracing::info!("Using NEON-optimized pattern matcher");
        }
    }
    
    // memchr and aho-corasick automatically use best SIMD available
    Box::new(SimdPatternMatcher::new(patterns))
}
```

## Part D: Guardy Config Module with Arc-wrapped Static

### D1. Main Config with Arc at Static Level
Location: `packages/guardy/src/config/mod.rs`

```rust
use superconfig::prelude::*;
use std::sync::{Arc, LazyLock};
use once_cell::sync::OnceCell;

// Global merged config - Arc wrapped for zero-copy cloning
static CONFIG: LazyLock<Arc<GuardyConfig>> = LazyLock::new(|| {
    Arc::new(load_merged_config())
});

// CLI args stored once (not Arc-wrapped, only used once)
static CLI_ARGS: OnceCell<crate::cli::CliArgs> = OnceCell::new();

pub fn init_with_cli(args: crate::cli::CliArgs) -> Result<()> {
    // Store CLI args for config building
    CLI_ARGS.set(args).map_err(|_| anyhow!("Already initialized"))?;
    // Force config load
    let _ = &*CONFIG;
    Ok(())
}

fn load_merged_config() -> GuardyConfig {
    let cli = CLI_ARGS.get().cloned().unwrap_or_default();
    
    // Handle explicit config file from CLI
    let config_path = cli.config.as_deref();
    
    Config::builder()
        .with_defaults(GuardyConfig::default())      // 0ms (native Rust)
        .with_config_file(config_path)               // 5ms if exists
        .with_env_prefix("GUARDY_")                  // 0.1ms
        .with_cli_overrides(cli.to_partial_config()) // 0.01ms
        .build()
        .unwrap_or_default()
}

// Return Arc clone for zero-copy access
pub fn config() -> Arc<GuardyConfig> {
    CONFIG.clone()  // Just increments Arc refcount, ~3ns
}
```

### D2. CLI Integration (No Arc Needed)
Location: `packages/guardy/src/cli/args.rs`

```rust
use clap::{Parser, Args, Subcommand};
use superconfig::PartialConfig;

// CLI args don't need Arc - they're only parsed once
#[derive(Parser, Clone)]
#[command(name = "guardy", version, about)]
pub struct CliArgs {
    /// Enable debug output
    #[arg(short = 'd', long, global = true)]
    pub debug: Option<bool>,
    
    /// Disable colored output  
    #[arg(long, global = true)]
    pub no_color: bool,
    
    /// Config file path (overrides default search)
    #[arg(short = 'c', long, global = true)]
    pub config: Option<PathBuf>,
    
    /// Override config values (can be repeated)
    /// Example: --set scanner.max_threads=4
    #[arg(long = "set", global = true, value_parser = parse_key_value)]
    pub overrides: Vec<(String, String)>,
    
    /// Verbose output (-v, -vv, -vvv)
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    
    #[command(subcommand)]
    pub command: Command,
}

impl CliArgs {
    pub fn to_partial_config(&self) -> PartialConfig {
        let mut partial = PartialConfig::new();
        
        // Note: config file path handled separately in builder
        
        // Global flags
        partial.set_if_some("general.debug", self.debug);
        if self.no_color {
            partial.set("general.color", false);
        }
        
        // Generic overrides from --set
        for (key, value) in &self.overrides {
            // Auto-detect type and set
            if let Ok(bool_val) = value.parse::<bool>() {
                partial.set(key, bool_val);
            } else if let Ok(int_val) = value.parse::<i64>() {
                partial.set(key, int_val);
            } else if let Ok(float_val) = value.parse::<f64>() {
                partial.set(key, float_val);
            } else {
                partial.set(key, value.clone());
            }
        }
        
        // Command-specific overrides...
        partial
    }
}
```

## Part E: Clean Consumers

### E1. Scanner with Hot Path Optimization
```rust
impl Scanner {
    pub fn new() -> Result<Self> {
        let config = config::config();  // Arc clone, ~3ns
        
        // Hot path: Access frequently-used fields
        if config.scanner.hot.mode == ScanMode::Sequential {
            // This access is <0.3ns due to L1 cache
        }
        
        let patterns = create_optimized_matcher(DEFAULT_PATTERNS);
        
        Ok(Scanner { config, patterns })
    }
    
    pub fn should_scan_file(&self, size: u64) -> bool {
        // Hot path - all these fields in same cache line
        let hot = &self.config.scanner.hot;
        
        if !hot.include_binary { return false; }
        if size > hot.max_file_size_mb as u64 * 1024 * 1024 { return false; }
        if hot.enable_entropy && self.check_entropy() > hot.entropy_threshold {
            return false;
        }
        true
    }
}
```

## Performance Analysis

### Memory Layout & Access Times
```rust
// Cache line boundary visualization
// |<------------ 64 bytes (1 cache line) ------------>|
// [mode][threads][binary][symlink][size][entropy][pad]
//   1      2        1       1        4      5      50

// Access patterns:
config.scanner.hot.mode           // 0.3ns - L1 hit (same line)
config.scanner.hot.max_threads    // 0.3ns - L1 hit (same line)
config.scanner.hot.include_binary // 0.3ns - L1 hit (same line)

config.scanner.cold.ignore_paths  // 5ns - Arc deref + L2/L3
config.clone()                     // 3ns - Arc increment only
```

### SIMD Performance Gains
| Operation | Without SIMD | With SIMD | Speedup |
|-----------|-------------|-----------|---------|
| Keyword search | 500ns/KB | 50ns/KB | 10x |
| Multi-pattern | 2000ns/KB | 100ns/KB | 20x |
| Pattern prefilter | N/A | 20ns/KB | ∞ |
| Total scan | 100ms/MB | 5ms/MB | 20x |

### Platform SIMD Support
| Platform | SIMD | Auto-Detection | Fallback |
|----------|------|----------------|----------|
| x86_64 | AVX2/SSE2 | ✓ Runtime | Scalar |
| ARM64 | NEON | ✓ Runtime | Scalar |
| Mac M1/M2 | NEON | ✓ Built-in | Scalar |
| WASM | SIMD128 | ✓ Feature | Scalar |

## Testing Strategy

1. **Unit Tests**: Each config layer (defaults, file, env, CLI)
2. **Integration Tests**: Full merge scenarios with CLI config paths
3. **Benchmarks**: Cache hit rates, SIMD speedup verification
4. **Cross-platform**: Test on x86_64, ARM64, Mac M-series

## Success Criteria

- [ ] Config load <5ms
- [ ] Hot field access <0.3ns (L1 cache)
- [ ] Cold field access <5ns (Arc deref)
- [ ] Pattern matching 10-20x faster with SIMD
- [ ] All tests pass on Mac and PC
- [ ] CLI config path override works

## PROGRESS UPDATE

### ✅ COMPLETED (Part A - SuperConfig Enhancements)
1. **Cache removal** - 5x performance improvement (5.28μs vs 26.39μs cached)
2. **PartialConfigurable trait** - Zero JSON overhead for env/CLI overrides
3. **ConfigBuilder enhancement** - Flexible defaults, layered configuration
4. **NEW config_builder! macro** - Auto-derivation with heck crate
5. **FastConfig → Config rename** - Throughout entire codebase
6. **Enhanced config! macro** - Better error handling, fallback to Default
7. **All tests passing** - 22 unit tests + 12 doc tests + integration tests

### ⏳ NEXT UP (Part B - Guardy Integration)
1. **Switch GuardyConfig to use config_builder! macro**
2. **Remove old config files** (core.rs, fast.rs, scan/config.rs)
3. **Clean up Scanner methods** (remove manual CLI override logic)
4. **Implement Arc-wrapped LazyLock static** in config/mod.rs
5. **CLI integration** with PartialConfig conversion

### 🎯 READY TO IMPLEMENT (Part C & D)
- GuardyConfig already has hot/cold optimization ✅
- SIMD pattern matching (Part C2)
- Full guardy integration (Part D)

## Implementation Order (Updated)

1. ✅ Move this plan to `guardy/plans/` folder
2. ✅ SuperConfig library enhancements (Part A)
3. ⏳ **NEXT:** Integrate GuardyConfig with config_builder! macro
4. ⏳ Remove old FastConfig from guardy completely  
5. 🎯 Add SIMD pattern matching
6. 🎯 Set up Arc-wrapped LazyLock static
7. 🎯 Test performance meets targets

## Notes

- Arc wrapping entire config at static level enables zero-copy for all consumers
- Hot/cold separation achieves <1ns access for critical path
- SIMD automatically selects best instruction set at runtime
- No backward compatibility needed - clean slate approach
- CLI config path overrides default search behavior