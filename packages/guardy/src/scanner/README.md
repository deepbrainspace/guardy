# Scanner Module

Comprehensive secret detection and file content analysis using pattern matching, entropy analysis, and intelligent filtering. Detects 40+ types of secrets including private keys, API tokens, database credentials, and more.

## Architecture

```
src/scanner/
├── mod.rs           # Module routing and re-exports only
├── core.rs          # Main Scanner struct and scanning logic
├── directory.rs     # DirectoryHandler and parallel coordination
├── patterns.rs      # Secret pattern definitions and regex compilation
├── entropy.rs       # Statistical entropy analysis algorithms
├── types.rs         # Core types (ScanResult, ScanStats, etc.)
├── test_detection.rs # Intelligent test code block detection
└── README.md        # This documentation
```

## Files and Responsibilities

### `core.rs`
- **Purpose**: Core scanning logic and individual file processing
- **Contains**: `Scanner` struct, individual file scan methods, pattern matching orchestration
- **Tests**: Scanner creation, single file scanning, pattern matching accuracy

### `directory.rs`
- **Purpose**: Directory scanning coordination and parallel execution
- **Contains**: `DirectoryHandler`, worker adaptation, execution strategy coordination, gitignore analysis
- **Tests**: Directory filtering, parallel execution, worker adaptation strategies

### `patterns.rs`
- **Purpose**: Secret pattern definitions and regex management  
- **Contains**: `SecretPatterns`, `SecretPattern`, 40+ predefined patterns for comprehensive secret detection
- **Built-in Detection**: Private keys (SSH, PGP, RSA, etc.), API keys (OpenAI, GitHub, AWS, etc.), database credentials, JWT tokens
- **Tests**: Pattern compilation, pattern matching, coverage of AI/cloud service patterns

### `entropy.rs`
- **Purpose**: Statistical analysis for randomness detection
- **Contains**: `is_likely_secret()` function, entropy calculation algorithms
- **Tests**: Entropy analysis accuracy, threshold validation, realistic vs fake secrets

### `types.rs`
- **Purpose**: Core data structures and type definitions
- **Contains**: `ScanResult`, `ScanStats`, `ScanMode`, `SecretMatch`, `Warning`, etc.
- **Tests**: Type serialization, result aggregation, statistics calculation

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
- **Parallel**: Coordinates parallel execution strategies and resource management

### Parallel Module Integration

The scanner module integrates tightly with the parallel module for efficient file processing:

#### Execution Strategies
- **Sequential**: Single-threaded scanning for small workloads
- **Parallel**: Multi-threaded scanning with domain-adapted worker counts
- **Auto**: Threshold-based automatic strategy selection

#### Resource Management Flow
```text
1. Scanner Config       →  2. Resource Calculation      →  3. Domain Adaptation
   ┌─────────────────┐      ┌──────────────────────────┐     ┌─────────────────────┐
   │ • max_threads   │      │ CPU cores: 16            │     │ File count: 36      │
   │ • thread_%: 75% │  ──▶ │ 16 * 75% = 12 workers    │ ──▶ │ ≤50 → 12/2 = 6     │
   │ • mode: auto    │      │ (system resource limit)  │     │ (domain adaptation)  │
   └─────────────────┘      └──────────────────────────┘     └─────────────────────┘
                                                                       │
4. Strategy Decision                          ← ← ← ← ← ← ← ← ← ← ← ← ← ← 
   ┌─────────────────────────────────────────┐
   │ auto(file_count=36, threshold=50, workers=6) │
   │ → 36 < 50 → ExecutionStrategy::Sequential    │  
   └─────────────────────────────────────────────┘
```

#### Worker Adaptation Strategy
The scanner implements domain-specific worker adaptation in `DirectoryHandler::adapt_workers_for_file_count()`:
- **≤10 files**: Minimal parallelism (overhead exceeds benefits)
- **≤50 files**: Conservative parallelism (50% of max workers)
- **≤100 files**: Moderate parallelism (75% of max workers)
- **>100 files**: Full parallelism (all available workers)

## Notes for AI Assistants and Developers

### 🤖 AI Assistant Guidelines

#### When Working with Scanner Module:
- **Use `DirectoryHandler::scan()`** as the primary entry point for directory scanning
- **Let the module handle strategy decisions** unless explicit override needed
- **Trust the domain adaptation logic** for worker scaling based on file counts
- **Respect the filtered directory patterns** for optimal performance

#### Key Integration Points:
1. **File Discovery**: Use built-in directory filtering and walking logic
2. **Parallel Coordination**: Integrate with parallel module for resource management
3. **Progress Reporting**: Use configured progress reporters with appropriate icons
4. **Result Aggregation**: Collect and combine scan results with statistics

#### Common Usage Patterns:
```rust
use guardy::scanner::directory::DirectoryHandler;
use guardy::scanner::Scanner;
use std::sync::Arc;

// Primary scanning workflow
let config = GuardyConfig::load(None, None::<&()>)?;
let scanner = Arc::new(Scanner::new(&config)?);
let directory_handler = DirectoryHandler::default();

// Automatic strategy selection
let result = directory_handler.scan(scanner, path, None)?;

// Explicit strategy override
let strategy = ExecutionStrategy::Parallel { workers: 4 };
let result = directory_handler.scan(scanner, path, Some(strategy))?;
```

### 🔧 Development Guidelines

#### File Architecture Updates:
The current file structure reflects the parallel integration:
```
src/scanner/
├── mod.rs           # Module routing and re-exports
├── core.rs          # Main Scanner struct and scanning logic
├── directory.rs     # DirectoryHandler and parallel coordination
├── patterns.rs      # Secret pattern definitions and regex compilation
├── entropy.rs       # Statistical entropy analysis algorithms
├── types.rs         # Core types (ScanResult, ScanStats, etc.)
├── test_detection.rs # Intelligent test code block detection
└── README.md        # This documentation
```

#### Key Responsibilities by File:

##### `directory.rs` (New/Enhanced)
- **Purpose**: Directory scanning coordination and parallel execution
- **Contains**: `DirectoryHandler`, worker adaptation, execution strategy coordination
- **Integration**: Primary interface between scanner and parallel modules

##### `core.rs` (Updated)
- **Purpose**: Core scanning logic and file processing
- **Contains**: `Scanner` struct, individual file scanning methods
- **Focus**: Single-file processing, pattern matching orchestration

##### `types.rs` (Updated)
- **Purpose**: Core data structures and enums
- **Contains**: `ScanResult`, `ScanStats`, `ScanMode`, `Warning`, etc.
- **Usage**: Shared types across scanner modules

#### Adding New Features:
- **Directory Filtering**: Extend `DirectoryHandler::default()` with new patterns
- **Worker Adaptation**: Modify thresholds in `adapt_workers_for_file_count()`
- **Progress Reporting**: Customize icons and frequency in execution strategies
- **File Processing**: Add new scan methods to `Scanner` in `core.rs`

### 🎯 Performance Optimization

#### Directory Filtering Impact:
- Reduces scan time by 60-80% by skipping build/cache directories
- Automatic gitignore analysis provides optimization suggestions
- Language-specific patterns (node_modules, target, __pycache__, etc.)

#### Parallel Execution Benefits:
- File-count-aware worker scaling
- Resource-aware execution strategy selection
- Automatic threshold-based parallel/sequential decisions

#### Memory Management:
- Arc<Scanner> enables thread-safe sharing across workers
- Bounded channels prevent memory overflow in large directories
- Progress reporting optimized for minimal contention

### 🚨 Common Pitfalls to Avoid

1. **Don't bypass DirectoryHandler**: Use the coordinated scanning approach
2. **Don't hardcode execution strategies**: Let auto mode optimize for workload
3. **Don't ignore filtered directories**: They're essential for performance
4. **Don't mix scanning and parallel logic**: Keep separation of concerns

### 📊 Configuration Integration

#### Scanner-Specific Settings:
```toml
[scanner]
mode = "auto"                    # Sequential/Parallel/Auto
max_threads = 0                  # 0 = no limit
thread_percentage = 75           # Use 75% of CPU cores
min_files_for_parallel = 50      # Threshold for auto mode

# Ignore mechanisms
ignore_test_code = true
ignore_paths = ["tests/*", "*_test.rs"]
ignore_patterns = ["# TEST_SECRET:", "DEMO_KEY_"]
ignore_comments = ["guardy:ignore", "guardy:ignore-line"]
```

#### Progress Reporting Configuration:
- **Sequential**: ⏳ icon, 10-item frequency
- **Parallel**: ⚡ icon, 5-item frequency  
- **Custom**: Configurable via progress reporter factories

## Supported Secret Types

The scanner includes 40+ built-in patterns for comprehensive secret detection:

### Private Keys & Certificates
- SSH private keys (RSA, DSA, EC, OpenSSH, SSH2)
- PGP/GPG private keys (`-----BEGIN PGP PRIVATE KEY BLOCK-----`)
- PKCS private keys (`-----BEGIN PRIVATE KEY-----`)
- PuTTY private keys (`PuTTY-User-Key-File-2`)
- Age encryption keys (`AGE-SECRET-KEY-1...`)

### Cloud Provider Credentials
- **AWS**: Access keys (`AKIA...`), secret keys, session tokens
- **Azure**: Client secrets, storage keys (`AccountKey=...`)
- **Google Cloud**: API keys (`AIzaSy...`), service account keys

### API Keys & Tokens
- **AI/ML**: OpenAI (`sk-proj-...`, `sk-...`), Anthropic Claude (`sk-ant-api...`), Hugging Face (`hf_...`), Cohere (`co....`), Replicate (`r8_...`), Mistral (UUID format)
- **Development**: GitHub (`ghp_...`, `gho_...`), GitLab (`glpat-...`), npm (`npm_...`)
- **Services**: Slack (`xox[aboprs]-...`), SendGrid (`SG....`), Twilio (`AC...`, `SK...`), Mailchimp, Stripe (`[rs]k_live_...`), Square
- **JWT/JWE**: JSON Web Tokens (`eyJ...`)

### Database Credentials
- MongoDB connection strings (`mongodb://user:pass@host`)
- PostgreSQL connection strings (`postgres://user:pass@host`)
- MySQL connection strings (`mysql://user:pass@host`)

### Generic Detection
- **Context-based patterns**: High-entropy strings near keywords like "password", "token", "key", "secret", "api"
- **URL credentials**: `https://user:pass@host` patterns
- **Custom configurable patterns**: Add your own regex patterns via configuration

### Pattern Matching Strategy
1. **Specific patterns**: Known formats for popular services (high precision)
2. **Generic context patterns**: Detect unknown secrets using contextual keywords + high entropy
3. **Entropy analysis**: Statistical validation of randomness for suspected secrets
4. **Intelligent filtering**: Skip test code, demo data, and false positives

## Usage Examples

```rust
use crate::scanner::{Scanner, SecretPatterns};
use crate::config::GuardyConfig;

// Create scanner
let config = GuardyConfig::load()?;
let scanner = Scanner::new(&config)?;

// Scan individual file
let matches = scanner.scan_file(&path)?;
for secret_match in matches {
    println!("Found {} at {}:{}", 
        secret_match.secret_type,
        secret_match.file_path, 
        secret_match.line_number);
}

// Scan directory with full results
let result = scanner.scan_directory(&dir_path)?;
println!("Found {} secrets in {} files", 
    result.stats.total_matches, 
    result.stats.files_scanned);

// CLI usage examples
// guardy scan src/ --stats
// guardy scan config.json --include-binary  
// guardy scan . --max-file-size 50
```