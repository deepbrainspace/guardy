# Scanner Module

Secret detection and file content analysis using pattern matching and entropy analysis.

## Architecture

```
src/scanner/
├── mod.rs          # Module routing and re-exports only
├── core.rs         # Main Scanner struct and scanning logic
├── patterns.rs     # Secret pattern definitions and regex compilation
├── entropy.rs      # Statistical entropy analysis algorithms
├── ignore_intel.rs # Gitignore analysis and project type detection
└── README.md       # This documentation
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