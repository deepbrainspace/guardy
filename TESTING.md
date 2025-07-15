# Guardy Testing Guide

This document provides comprehensive testing instructions for the Guardy project.

## Testing Framework

**Primary Framework**: Rust's built-in testing with `cargo test`

**Testing Dependencies:**
- `tokio-test` - Testing async functions
- `assert_cmd` - CLI command testing and integration tests  
- `predicates` - Complex assertion conditions
- `serial_test` - Tests that can't run in parallel
- `pretty_assertions` - Enhanced diff output for failures
- `tempfile` - Temporary files/directories for isolated tests

## Test Categories

### 1. Unit Tests (`cargo test`)

**Location**: `src/*/tests.rs` modules  
**Purpose**: Test individual components in isolation

```bash
# Run all unit tests
cargo test

# Run tests for specific module
cargo test config::tests
cargo test security::tests

# Run with verbose output
cargo test -- --nocapture

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1
```

### 2. Integration Tests (`cargo test --test`)

**Location**: `tests/integration_tests.rs`  
**Purpose**: Test CLI functionality end-to-end

```bash
# Run integration tests only
cargo test --test integration_tests

# Run specific integration test
cargo test --test integration_tests test_secret_detection
```

### 3. Manual Testing

**Purpose**: Verify real-world functionality

## Detailed Testing Instructions

### Prerequisites

1. **Build the project**:
   ```bash
   cargo build
   ```

2. **Install the binary locally** (for manual testing):
   ```bash
   cargo install --path .
   ```

### Configuration Testing

#### Test 1: Default Configuration Loading
```bash
# Test default config creation
cargo test test_default_config

# Verify gitignore integration
cargo test test_gitignore_patterns_loading
```

#### Test 2: Configuration File Operations
```bash
# Test YAML serialization/deserialization
cargo test test_config_serialization
cargo test test_config_deserialization

# Test file save/load operations
cargo test test_load_save_config
```

#### Test 3: Configuration Validation
```bash
# Test config validation rules
cargo test test_config_validation

# Manual validation test
guardy config validate --config .guardy.yml
```

### Security Pattern Testing

#### Test 1: Pattern Recognition
```bash
# Test pattern parsing and creation
cargo test test_security_pattern_creation
cargo test test_patterns_from_config

# Test severity levels
cargo test test_severity_parsing
```

#### Test 2: Secret Detection
```bash
# Create test file with secrets
echo 'let key = "sk_test_FAKE_KEY_FOR_TESTING";' > test_secrets.rs

# Run scanner tests
cargo test test_secret_scanner_scan_file

# Manual secret detection
guardy scan --file test_secrets.rs

# Clean up
rm test_secrets.rs
```

#### Test 3: File Exclusion
```bash
# Test gitignore pattern exclusion
cargo test test_secret_scanner_exclude_patterns
cargo test test_effective_exclude_patterns

# Test file type filtering
cargo test test_secret_scanner_should_scan_file
```

### CLI Integration Testing

#### Test 1: Basic CLI Functionality
```bash
# Test help and version commands
cargo test test_cli_help
cargo test test_cli_version

# Manual CLI tests
guardy --help
guardy --version
guardy scan --help
```

#### Test 2: Secret Scanning Integration
```bash
# Test complete scanning workflow
cargo test test_secret_detection

# Manual end-to-end test
mkdir test_project
cd test_project
echo 'const API_KEY = "sk_test_abcdef1234567890123456";' > main.js
guardy scan --file main.js
cd ..
rm -rf test_project
```

#### Test 3: Configuration Integration
```bash
# Test config operations
cargo test test_config_operations

# Manual config tests
guardy init --template
guardy config validate
guardy config show
```

### Gitignore Integration Testing

#### Test 1: Pattern Loading
```bash
# Test gitignore pattern discovery
cargo test test_gitignore_integration

# Manual gitignore test
mkdir test_gitignore
cd test_gitignore
echo "*.log" > .gitignore
echo "target/" >> .gitignore
echo 'secret in log file' > debug.log
echo 'let key = "sk_test_FAKE_KEY_FOR_TESTING_ONLY";' > main.rs

# Initialize guardy config with gitignore enabled
cat > .guardy.yml << EOF
security:
  secret_detection: true
  use_gitignore: true
  patterns:
    - name: "OpenAI API Key"
      regex: "sk_test_[a-zA-Z0-9]{26}"
hooks:
  pre_commit: false
  commit_msg: false
  pre_push: false
  timeout: 300
mcp:
  enabled: false
tools:
  auto_detect: false
  formatters: []
  linters: []
EOF

# Should scan main.rs but ignore debug.log
guardy scan --directory .

cd ..
rm -rf test_gitignore
```

### Tool Detection Testing

#### Test 1: Project Type Detection
```bash
# Test tool auto-detection
cargo test test_tool_detection

# Manual tool detection
guardy tools detect
```

### Error Handling Testing

#### Test 1: Invalid Configurations
```bash
# Test invalid regex patterns
cargo test test_invalid_regex_pattern

# Test invalid MCP config
cargo test test_mcp_config_validation
```

#### Test 2: Invalid CLI Usage
```bash
# Test invalid subcommands
cargo test test_invalid_subcommand

# Manual error testing
guardy invalid-command
guardy scan --file nonexistent.rs
```

### Performance Testing

#### Test 1: Large File Scanning
```bash
# Test directory scanning performance
cargo test test_batch_scanning

# Manual performance test
mkdir large_project
for i in {1..100}; do
  echo "fn test_$i() {}" > large_project/file_$i.rs
done

time guardy scan --directory large_project

rm -rf large_project
```

### Security Testing

#### Test 1: Secret Pattern Coverage
```bash
# Test all built-in patterns
mkdir pattern_test
cd pattern_test

# Create test files with various secret types
echo 'openai_key = "sk_test_FAKE_OPENAI_KEY_FOR_TESTING"' > openai.py
echo 'github_pat = "ghp_FAKE_GITHUB_PAT_FOR_TESTING"' > github.js
echo 'aws_key = "AKIA_FAKE_AWS_KEY_FOR_TESTING"' > aws.yml
echo 'jwt = "eyJ_FAKE_JWT_TOKEN_FOR_TESTING"' > jwt.json

# Copy template config
cp ../templates/guardy.yml.template .guardy.yml

# Scan for all secret types
guardy scan --directory .

cd ..
rm -rf pattern_test
```

## Test Data Management

### Creating Test Fixtures

For tests requiring specific file structures:

```bash
# Create test fixture directory
mkdir -p test_fixtures/rust_project
cd test_fixtures/rust_project

# Create minimal Rust project
cat > Cargo.toml << EOF
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
EOF

cat > src/main.rs << EOF
fn main() {
    println!("Hello, world!");
}
EOF

# Create gitignore
cat > .gitignore << EOF
target/
*.log
.env
EOF
```

### Cleaning Test Data

```bash
# Clean up test fixtures
rm -rf test_fixtures/

# Clean up any temporary test files
find . -name "*.test.*" -delete
find . -name "test_*" -type f -delete
```

## Continuous Integration

### GitHub Actions Testing

The CI pipeline runs:
1. `cargo test` - All unit tests
2. `cargo test --test integration_tests` - Integration tests  
3. `cargo clippy` - Linting
4. `cargo fmt -- --check` - Code formatting
5. Security audits

### Local CI Simulation

```bash
# Run full test suite like CI
cargo test --all-features
cargo clippy --all-targets --all-features
cargo fmt -- --check
cargo audit
```

## Test Coverage

### Generate Coverage Report

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage report
open coverage/tarpaulin-report.html
```

## Debugging Test Failures

### Common Issues

1. **Tests fail due to missing dependencies**:
   ```bash
   cargo build --all-features
   ```

2. **Integration tests can't find binary**:
   ```bash
   cargo build --bin guardy
   ```

3. **Temporary file cleanup issues**:
   ```bash
   cargo test -- --test-threads=1
   ```

4. **Git-related test failures**:
   ```bash
   git config --global user.email "test@example.com"
   git config --global user.name "Test User"
   ```

### Verbose Test Output

```bash
# Show all test output
cargo test -- --nocapture

# Show only failing test output
cargo test -- --nocapture --test-threads=1 test_name

# Run with debug logging
RUST_LOG=debug cargo test
```

## Test Checklist

Before submitting code, verify:

- [ ] All unit tests pass: `cargo test`
- [ ] All integration tests pass: `cargo test --test integration_tests`
- [ ] Code formatting is correct: `cargo fmt -- --check`
- [ ] Linting passes: `cargo clippy`
- [ ] No security vulnerabilities: `cargo audit`
- [ ] Manual testing of new features completed
- [ ] Test coverage is adequate for new code
- [ ] Documentation is updated for new functionality

## Writing New Tests

### Unit Test Template

```rust
#[test]
fn test_new_functionality() {
    // Arrange
    let input = "test_input";
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result.unwrap(), "expected_output");
}
```

### Integration Test Template

```rust
#[test]
fn test_cli_new_command() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("new-command")
        .arg("--option")
        .arg("value")
        .assert()
        .success()
        .stdout(predicate::str::contains("expected_output"));
}
```

## Test Environment Variables

Set these for consistent test behavior:

```bash
export RUST_TEST_NOCAPTURE=1      # Show all output
export RUST_TEST_THREADS=1        # Single-threaded for debugging
export RUST_LOG=debug             # Enable debug logging
export GUARDY_CONFIG_PATH=test_config.yml  # Use test config
```