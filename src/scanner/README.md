# Scanner Module

Secret detection and file content analysis using pattern matching and entropy analysis.

## Architecture

```
src/scanner/
├── mod.rs           # Module routing and re-exports only
├── core.rs          # Main Scanner struct and scanning logic
├── patterns.rs      # Secret pattern definitions and regex compilation
├── entropy.rs       # Statistical entropy analysis algorithms
├── ignore_intel.rs  # Gitignore analysis and project type detection
├── test_detection.rs # Intelligent test code block detection
└── README.md        # This documentation
```

## Files and Responsibilities

### `core.rs`
- **Purpose**: Main scanning orchestration and file processing
- **Contains**: `Scanner` struct, scan methods, `SecretMatch`, `ScanResult` types
- **Tests**: Scanner creation, file scanning, directory scanning, scan statistics

### `patterns.rs`
- **Purpose**: Secret pattern definitions and regex management
- **Contains**: `SecretPatterns`, `SecretPattern`, predefined patterns for modern APIs
- **Tests**: Pattern compilation, pattern matching, coverage of AI/cloud service patterns

### `entropy.rs`
- **Purpose**: Statistical analysis for randomness detection
- **Contains**: `is_likely_secret()` function, entropy calculation algorithms
- **Tests**: Entropy analysis accuracy, threshold validation, realistic vs fake secrets

### `ignore_intel.rs`
- **Purpose**: Project type detection and gitignore intelligence
- **Contains**: `GitignoreIntelligence`, `ProjectType`, gitignore suggestions
- **Tests**: Project type detection, gitignore suggestions, pattern recommendations

### `test_detection.rs`
- **Purpose**: Intelligent test code block detection across multiple languages
- **Contains**: `TestDetector`, block boundary detection, language-specific parsing
- **Tests**: Rust test blocks, TypeScript/JavaScript test suites, Python test functions

### `mod.rs`
- **Purpose**: Module organization only
- **Contains**: Module declarations and re-exports
- **Tests**: None (routing only)

## Test Organization Guidelines

**✅ DO:**
- Put tests inline with `#[cfg(test)] mod tests` in each implementation file
- Test the specific functionality in the same file where it's implemented
- Keep scanner tests in `core.rs`, pattern tests in `patterns.rs`, etc.

**❌ DON'T:**
- Put tests in `mod.rs` (routing only)
- Create separate `tests.rs` files (use inline tests)
- Mix tests from different components in one file

## Data Flow

```
Scanner (core.rs)
    ↓
File Reading → Line Scanning → Pattern Matching (patterns.rs)
    ↓                              ↓
SecretMatch ← Entropy Analysis (entropy.rs)
    ↓
ScanResult with Statistics
```

## Scanner Ignore Mechanisms

The scanner provides four intelligent ignore mechanisms to prevent false positives:

### 1. **Path-based Ignoring** (`ignore_paths`)
Uses glob patterns to ignore entire files and directories:
```toml
[scanner]
ignore_paths = [
    "tests/*",        # All test directories
    "testdata/*",     # Test data directories  
    "*_test.rs",      # Test files
    "test_*.rs"       # Test files
]
```

### 2. **Pattern-based Ignoring** (`ignore_patterns`)
Ignores lines containing specific patterns:
```toml
[scanner]
ignore_patterns = [
    "# TEST_SECRET:",  # Lines marked as test secrets
    "DEMO_KEY_",       # Demo/fake keys
    "FAKE_"            # Fake credentials
]
```

### 3. **Comment-based Ignoring** (`ignore_comments`)
Inline comments to suppress scanning:
```toml
[scanner]
ignore_comments = [
    "guardy:ignore",      # Ignore this line
    "guardy:ignore-line", # Ignore this line  
    "guardy:ignore-next"  # Ignore next line
]
```

**Usage:**
```rust
let secret = "sk_live_real_key"; // guardy:ignore-line
// guardy:ignore-next
let another_secret = "sk_test_fake_key";
```

### 4. **Intelligent Test Code Detection** (`ignore_test_code`)
Automatically detects and ignores test code across multiple languages:
```toml
[scanner]
ignore_test_code = true
test_attributes = [
    # Rust test patterns
    "#[*test]",      # Matches #[test], #[tokio::test], etc.
    "#[bench]",      # Benchmark functions
    "#[cfg(test)]",  # Test configuration
    # Python test patterns  
    "def test_*",    # Test functions
    "class Test*",   # Test classes
    "@pytest.*",     # Pytest decorators
    # TypeScript/JavaScript test patterns
    "it(*",          # Jest/Mocha it() blocks
    "test(*",        # Jest test() blocks  
    "describe(*"     # Jest/Mocha describe() blocks
]
test_modules = [
    # Rust
    "mod tests {",   # Test modules
    "mod test {",    # Test modules
    # Python
    "class Test",    # Test classes
    # TypeScript/JavaScript  
    "describe(",     # Test suites
    "__tests__"      # Test directories
]
```

**Detected patterns by language:**

**Rust:**
- `#[test]`, `#[tokio::test]`, `#[async_test]`, `#[wasm_bindgen_test]`
- `#[bench]` benchmark functions
- `#[cfg(test)]` conditional compilation
- `mod tests {` and `mod test {` test modules

**Python:**
- `def test_*` test functions
- `class Test*` test classes  
- `@pytest.*` pytest decorators
- `class Test` test class declarations

**TypeScript/JavaScript:**
- `it(` Jest/Mocha test cases
- `test(` Jest test cases
- `describe(` Jest/Mocha test suites

## Configuration

All ignore mechanisms are configurable via `guardy.toml`:

```toml
[scanner]
# Enable/disable each mechanism
ignore_test_code = true

# Customize patterns for your project
ignore_patterns = [
    "# DEMO:",
    "EXAMPLE_", 
    "YOUR_CUSTOM_PATTERN"
]

# Add custom test attributes
test_attributes = [
    "#[*test]",
    "#[custom::test]"
]
```

## Integration with Other Modules

- **Config**: Gets scanner configuration and pattern customization
- **Git**: Integrates with git file discovery for targeted scanning
- **CLI**: Provides scan results for command-line output
- **MCP**: Exposes scanning capabilities via MCP server interface

## Usage Examples

```rust
use crate::scanner::{Scanner, SecretPatterns};
use crate::config::GuardyConfig;

// Create scanner
let config = GuardyConfig::load()?;
let scanner = Scanner::new(&config)?;

// Scan individual file
let matches = scanner.scan_file(&path)?;

// Scan directory
let result = scanner.scan_directory(&dir_path)?;
println!("Found {} secrets in {} files", 
    result.stats.total_matches, 
    result.stats.files_scanned);
```