# Scanner Module

Comprehensive secret detection and file content analysis using pattern matching, entropy analysis, and intelligent filtering. Detects 40+ types of secrets including private keys, API tokens, database credentials, and more.

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
- **Contains**: `SecretPatterns`, `SecretPattern`, 40+ predefined patterns for comprehensive secret detection
- **Built-in Detection**: Private keys (SSH, PGP, RSA, etc.), API keys (OpenAI, GitHub, AWS, etc.), database credentials, JWT tokens
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