# Testing Phase 1.3 Security Features

## Overview
Phase 1.3 implements comprehensive security features including secret detection, branch protection validation, staging area checks, and unified glob pattern functionality.

## Prerequisites
- Rust toolchain installed
- Git repository initialized
- Build the project: `cargo build`

## Test Commands

### 1. Build and Basic Setup
```bash
# Build the project
cargo build

# Check that guardy.yml exists and has security configuration
cat guardy.yml

# Verify the binary works
./target/debug/guardy --help
```

### 2. Status Command - Security Integration
```bash
# Test the enhanced status command
./target/debug/guardy status

# Expected output should show:
# - Security Status section
# - Secret detection enabled/disabled
# - Protected branches count
# - Git-crypt integration status
```

### 3. Security CLI Commands
```bash
# Test security command help
./target/debug/guardy security --help

# Test scan command help
./target/debug/guardy security scan --help

# Test validate command help
./target/debug/guardy security validate --help

# Test check command help
./target/debug/guardy security check --help
```

### 4. Secret Detection Testing

#### 4.1 Create Test File with Mock Secrets
```bash
# Create a test file with realistic fake secrets (FOR TESTING ONLY)
cat > demo-secrets.js << 'EOF'
// Test file with mock secrets (FOR TESTING ONLY - These are not real secrets)
const AWS_ACCESS_KEY_ID = "AKIAIOSFODNN7EXAMPLE";
const AWS_SECRET_ACCESS_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

// This is a private key
const privateKey = \`-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA1234567890abcdef1234567890abcdef1234567890abcdef
-----END RSA PRIVATE KEY-----\`;
EOF
```

#### 4.2 Test Secret Scanning
```bash
# Test scanning specific file (NOTE: use -i for input files)
./target/debug/guardy security scan -i demo-secrets.js

# Test with long form
./target/debug/guardy security scan --files demo-secrets.js

# Expected: Should find 3 security issues
# - AWS Access Key (AKIAIOSFODNN7EXAMPLE)
# - AWS Secret Key (wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY)
# - JSON Web Token (eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...)
# - Private Key (-----BEGIN RSA PRIVATE KEY-----)

# Test JSON output format (global flag)
./target/debug/guardy --format json security scan -i demo-secrets.js

# Test directory scanning
./target/debug/guardy security scan --directory .

# Test scanning without parameters (current directory)
./target/debug/guardy security scan

# Test verbose output (global flag)
./target/debug/guardy --verbose security scan -i demo-secrets.js

# Test quiet output (global flag)
./target/debug/guardy --quiet security scan -i demo-secrets.js
```

#### 4.3 Test Clean Scan
```bash
# Remove test file
rm demo-secrets.js

# Test clean scan
./target/debug/guardy security scan

# Expected: "No security issues found"
```

### 5. Glob Pattern Testing

#### 5.1 Test Glob Pattern Expansion
```bash
# Create multiple test files with different extensions
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > secret1.js
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > secret2.ts
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > secret3.py

# Test glob patterns
./target/debug/guardy security scan -i "*.js"    # Should find secret1.js
./target/debug/guardy security scan -i "*.ts"    # Should find secret2.ts
./target/debug/guardy security scan -i "secret*" # Should find all three
./target/debug/guardy security scan -i "*.{js,ts}" # Should find secret1.js and secret2.ts

# Test with verbose to see file discovery
./target/debug/guardy --verbose security scan -i "secret*"

# Clean up
rm secret1.js secret2.ts secret3.py
```

#### 5.2 Test Exclusion Patterns (.guardyignore)
```bash
# View current .guardyignore file
cat .guardyignore

# Create a test file that should be excluded
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > test_should_be_excluded.js

# Test that it gets excluded
./target/debug/guardy --verbose security scan -i "test_*"

# Expected: Should show "Excluded 1 files:" followed by "1 files (relative path match)"

# Clean up
rm test_should_be_excluded.js
```

#### 5.3 Test Gitignore Integration
```bash
# Create a file that matches gitignore pattern
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > debug.log

# Test that it gets excluded due to gitignore
./target/debug/guardy --verbose security scan -i "*.log"

# Expected: Should show exclusion message

# Clean up
rm debug.log
```

### 6. Branch Protection Testing
```bash
# Test branch protection validation
./target/debug/guardy security validate

# Expected output should show:
# - Current branch protection status
# - List of protected branches (main, master, develop)
# - Git-crypt integration status
# - Installation checks for git-crypt
```

### 7. Staging Area Testing

#### 7.1 Test with Clean Staging Area
```bash
# Test with no staged files
./target/debug/guardy security check

# Expected: "No files staged for commit"
```

#### 7.2 Test with Staged Files
```bash
# Create test file with real pattern that will be detected
echo 'const aws_key = "AKIAIOSFODNN7EXAMPLE";' > staged-demo.js

# Stage the file
git add staged-demo.js

# Test staging area check
./target/debug/guardy security check

# Expected: Should find security issues in staged files

# Clean up
git reset staged-demo.js
rm staged-demo.js
```

### 8. Configuration Testing

#### 8.1 Test with Missing Configuration
```bash
# Backup current config
cp guardy.yml guardy.yml.bak

# Remove config file
rm guardy.yml

# Test commands without config
./target/debug/guardy security scan
./target/debug/guardy security validate
./target/debug/guardy security check

# Expected: Should use default configuration and show warnings

# Restore config
mv guardy.yml.bak guardy.yml
```

#### 8.2 Test with Disabled Secret Detection
```bash
# Temporarily disable secret detection
sed -i 's/secret_detection: true/secret_detection: false/' guardy.yml

# Test scan command
./target/debug/guardy security scan

# Expected: Should show "Secret detection is disabled"

# Restore config
sed -i 's/secret_detection: false/secret_detection: true/' guardy.yml
```

### 9. Global Flags Testing
```bash
# Test global flags work in any position
./target/debug/guardy --verbose security scan  # Before subcommand
./target/debug/guardy security scan --verbose  # After subcommand

# Test multiple global flags
./target/debug/guardy --verbose --format json security scan -i "*.js"

# Test force flag with hooks
./target/debug/guardy --force hooks install

# Test force flag with init
./target/debug/guardy --force init

# Test quiet mode across different commands
./target/debug/guardy --quiet security scan
./target/debug/guardy --quiet status
./target/debug/guardy --quiet hooks list

# Test help shows all global flags
./target/debug/guardy --help
./target/debug/guardy security scan --help
```

### 10. Error Handling Testing
```bash
# Test with non-existent file (use -i for input files)
./target/debug/guardy security scan -i non-existent.js

# Test with non-existent directory
./target/debug/guardy security scan --directory /non/existent/path

# Test with invalid format (global flag)
./target/debug/guardy --format invalid security scan
```

### 11. Unit Tests Verification
```bash
# Run all unit tests
cargo test

# Run specific test modules
cargo test config     # Configuration tests
cargo test security   # Security pattern tests
cargo test utils      # Utility function tests (including glob)

# Run tests with verbose output
cargo test -- --nocapture

# Run clippy to check code quality
cargo clippy --all-targets --all-features -- -D warnings
```

## Expected Results

### 1. Status Command
- Should show "Security Status" section
- Should indicate if secret detection is enabled
- Should show count of protected branches
- Should show git-crypt integration status

### 2. Security Scan
- Should detect all realistic security patterns in test files
- Should provide detailed output with file, line, and column information
- Should support both text and JSON output formats
- Should handle file and directory scanning
- Should properly exclude files based on .guardyignore and .gitignore

### 3. Glob Pattern Functionality
- Should expand glob patterns correctly (*.js, *.{js,ts}, secret*)
- Should exclude files matching patterns in .guardyignore
- Should exclude files matching patterns in .gitignore
- Should show excluded files summary in verbose mode (counts grouped by exclusion reason)
- Should handle complex glob patterns with proper file discovery

### 4. Branch Protection
- Should show current branch protection status
- Should list configured protected branches
- Should check git-crypt installation and setup

### 5. Staging Area Check
- Should scan only staged files
- Should provide appropriate feedback for empty staging area
- Should detect secrets in staged files

### 6. Global Flags
- Should work in any position (before or after subcommands)
- Verbose mode should show compact summaries for exclusions and scanning
- Quiet mode should show minimal output (only errors and final results)
- Force mode should skip confirmations without prompting
- Format flag should affect output format (json, yaml, text)
- Help commands should show all global flags

### 7. Error Handling
- Should gracefully handle missing files/directories
- Should provide helpful error messages
- Should fallback to defaults when configuration is missing

## Manual Verification Steps

1. **Build Success**: `cargo build` should complete without errors
2. **Test Success**: `cargo test` should pass all tests
3. **Clippy Clean**: `cargo clippy` should show no warnings except for unused code
4. **Security Patterns**: All default security patterns should be detected correctly
5. **JSON Format**: JSON output should be valid and well-formatted
6. **Branch Detection**: Current branch should be correctly identified
7. **File Scanning**: Both individual files and directories should be scannable
8. **Configuration Loading**: Should handle both present and missing configuration files
9. **Glob Expansion**: Should properly expand glob patterns and exclude files
10. **Pattern Exclusion**: Should respect both .guardyignore and .gitignore patterns

## Cleanup
```bash
# Remove any test files
rm -f demo-secrets.js staged-demo.js secret*.js secret*.ts secret*.py debug.log test_*.js

# Ensure no files are staged
git status

# Ensure configuration is restored
cp guardy.yml.bak guardy.yml 2>/dev/null || true
rm -f guardy.yml.bak
```

## Success Criteria
- All commands execute without errors
- Security scanning detects all test secrets using realistic patterns
- Branch protection validation works correctly
- Staging area checks function properly
- Error handling is graceful and informative
- JSON output is properly formatted
- Configuration loading works with defaults and custom files
- Global flags work in any position (before/after subcommands)
- Verbose mode shows detailed debugging information
- Quiet mode shows minimal output
- Force mode skips confirmations appropriately
- Help output shows all global flags consistently
- **Glob patterns expand correctly and find matching files**
- **Exclusion patterns work properly (.guardyignore and .gitignore)**
- **File discovery handles complex patterns and large directories**
- **All unit tests pass (cargo test)**

## Important CLI Changes

### New Global Flags
- `--verbose, -v`: Detailed output (now global)
- `--quiet, -q`: Minimal output (new)
- `--force, -f`: Skip confirmations (now global, replaces --yes)
- `--format <FORMAT>`: Output format (now global)
- `--config, -c <FILE>`: Custom config file (now global)
- `--auto-install`: Auto-install tools (now global)
- `--dry-run`: Show what would be done (new)

### Changed Flags
- **Security scan files**: Changed from `-f, --files` to `-i, --files` (input files)
- **Init command**: `--yes` flag removed, use global `--force` instead
- **Hooks install**: `--force` flag removed, use global `--force` instead

### All Global Flags Work With All Commands
```bash
# Examples of new global flag usage:
guardy --verbose security scan -i file.js
guardy --quiet --force hooks install
guardy --format json security scan
guardy --dry-run init
```

## Implemented Features Summary

### ✅ Phase 1.3 Complete Features
1. **Security Pattern Detection**: 6 default patterns for AWS keys, JWT tokens, private keys
2. **Glob Pattern Support**: Full glob expansion with `*.js`, `**/*.rs`, `{js,ts}` patterns
3. **File Exclusion System**: Unified .guardyignore and .gitignore pattern processing
4. **CLI Global Flags**: Verbose, quiet, force, format flags work across all commands
5. **Configuration System**: YAML-based configuration with validation and defaults
6. **Branch Protection**: Validation of protected branches and git-crypt integration
7. **Staging Area Checks**: Security scanning of staged files only
8. **Professional Output**: Consistent formatting with colors, icons, and structured output
9. **Error Handling**: Graceful handling of missing files, invalid patterns, and configuration issues
10. **Testing Framework**: Comprehensive unit tests for all modules

### 🔧 Integration Points
- **Glob Utility**: `src/utils/glob.rs` provides unified pattern matching
- **Security Scanner**: Uses glob patterns for file exclusion and discovery
- **Configuration**: Loads and processes ignore patterns from multiple sources
- **CLI Commands**: Support glob patterns in file specifications
- **Output System**: Consistent formatting across all commands and modes

This testing document covers all implemented features and verifies the complete functionality of Phase 1.3 security features with glob pattern support.