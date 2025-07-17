# Testing Phase 1.3 Security Features

## Overview
Phase 1.3 implements comprehensive security features including secret detection, branch protection validation, and staging area checks.

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
# Create a test file with fake secrets
cat > test-secrets.js << 'EOF'
// Test file with mock secrets (FOR TESTING ONLY)
const AWS_ACCESS_KEY_ID = "AKIAIOSFODNN7EXAMPLE";
const AWS_SECRET_ACCESS_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const API_KEY = "sk-1234567890abcdef1234567890abcdef";
const secret = "my-super-secret-password-123";
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
./target/debug/guardy security scan -i test-secrets.js

# Test with long form
./target/debug/guardy security scan --files test-secrets.js

# Expected: Should find 6 security issues
# - AWS Access Key
# - AWS Secret Key (multiple matches)
# - JSON Web Token
# - Private Key
# - Generic Secret

# Test JSON output format (global flag)
./target/debug/guardy --format json security scan -i test-secrets.js

# Test directory scanning
./target/debug/guardy security scan --directory .

# Test scanning without parameters (current directory)
./target/debug/guardy security scan

# Test verbose output (global flag)
./target/debug/guardy --verbose security scan -i test-secrets.js

# Test quiet output (global flag)
./target/debug/guardy --quiet security scan -i test-secrets.js
```

#### 4.3 Test Clean Scan
```bash
# Remove test file
rm test-secrets.js

# Test clean scan
./target/debug/guardy security scan

# Expected: "No security issues found"
```

### 5. Branch Protection Testing
```bash
# Test branch protection validation
./target/debug/guardy security validate

# Expected output should show:
# - Current branch protection status
# - List of protected branches (main, master, develop)
# - Git-crypt integration status
# - Installation checks for git-crypt
```

### 6. Staging Area Testing

#### 6.1 Test with Clean Staging Area
```bash
# Test with no staged files
./target/debug/guardy security check

# Expected: "No files staged for commit"
```

#### 6.2 Test with Staged Files
```bash
# Create test file again
echo 'const api_key = "sk-1234567890abcdef1234567890abcdef";' > staged-test.js

# Stage the file
git add staged-test.js

# Test staging area check
./target/debug/guardy security check

# Expected: Should find security issues in staged files

# Clean up
git reset staged-test.js
rm staged-test.js
```

### 7. Configuration Testing

#### 7.1 Test with Missing Configuration
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

#### 7.2 Test with Disabled Secret Detection
```bash
# Temporarily disable secret detection
sed -i 's/secret_detection: true/secret_detection: false/' guardy.yml

# Test scan command
./target/debug/guardy security scan

# Expected: Should show "Secret detection is disabled"

# Restore config
sed -i 's/secret_detection: false/secret_detection: true/' guardy.yml
```

### 8. Global Flags Testing
```bash
# Test global flags work in any position
./target/debug/guardy --verbose security scan  # Before subcommand
./target/debug/guardy security scan --verbose  # After subcommand

# Test multiple global flags
./target/debug/guardy --verbose --format json security scan -i test-secrets.js

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

### 9. Error Handling Testing
```bash
# Test with non-existent file (use -i for input files)
./target/debug/guardy security scan -i non-existent.js

# Test with non-existent directory
./target/debug/guardy security scan --directory /non/existent/path

# Test with invalid format (global flag)
./target/debug/guardy --format invalid security scan
```

## Expected Results

### 1. Status Command
- Should show "Security Status" section
- Should indicate if secret detection is enabled
- Should show count of protected branches
- Should show git-crypt integration status

### 2. Security Scan
- Should detect all 6 types of secrets in test file
- Should provide detailed output with file, line, and column information
- Should support both text and JSON output formats
- Should handle file and directory scanning

### 3. Branch Protection
- Should show current branch protection status
- Should list configured protected branches
- Should check git-crypt installation and setup

### 4. Staging Area Check
- Should scan only staged files
- Should provide appropriate feedback for empty staging area
- Should detect secrets in staged files

### 5. Global Flags
- Should work in any position (before or after subcommands)
- Verbose mode should show detailed output with file checking info
- Quiet mode should show minimal output (only errors and final results)
- Force mode should skip confirmations without prompting
- Format flag should affect output format (json, yaml, text)
- Help commands should show all global flags

### 6. Error Handling
- Should gracefully handle missing files/directories
- Should provide helpful error messages
- Should fallback to defaults when configuration is missing

## Manual Verification Steps

1. **Build Success**: `cargo build` should complete without errors
2. **Clippy Clean**: `cargo clippy` should show no warnings except for unused code
3. **Security Patterns**: All 6 default security patterns should be detected
4. **JSON Format**: JSON output should be valid and well-formatted
5. **Branch Detection**: Current branch should be correctly identified
6. **File Scanning**: Both individual files and directories should be scannable
7. **Configuration Loading**: Should handle both present and missing configuration files

## Cleanup
```bash
# Remove any test files
rm -f test-secrets.js staged-test.js

# Ensure no files are staged
git status
```

## Success Criteria
- All commands execute without errors
- Security scanning detects all test secrets
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