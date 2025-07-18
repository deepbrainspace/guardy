# Phase 1.5: Code Formatting Integration - Testing Instructions

## Overview
This document provides detailed testing instructions for the newly implemented code formatting integration in Guardy's pre-commit hooks.

## What Was Implemented

### ✅ Core Features
1. **Formatter Integration**: Real formatter execution during pre-commit hooks
2. **Package Auto-Detection**: Automatic detection of available tools and package managers
3. **Multi-Language Support**: Support for Rust, JavaScript/TypeScript, Python, Go formatters
4. **Smart Pattern Matching**: Glob pattern matching to find files that need formatting
5. **Check Mode Execution**: Dry-run mode to detect formatting issues without making changes

### ✅ Key Files Modified
- `src/cli/commands/hooks.rs` - Main integration code
- `src/main.rs` - Added tools module declaration
- Added comprehensive tests in `hooks.rs`

## Testing Instructions

### Test 1: Basic Functionality Test
**Purpose**: Verify the formatter integration works with auto-detection

```bash
# 1. Build the project
cargo build --release

# 2. Run pre-commit hook with verbose output
./target/release/guardy hooks run pre-commit --verbose

# Expected Output:
# 🏃 Running pre-commit Hook
# [1/3] 🔍 Running security scans
# [2/3] 🎨 Running formatting checks
# ℹ️  Auto-detected tools: Cargo (Cargo.toml found), rustfmt (available), Clippy (available)
# ℹ️  Auto-detection enabled but no formatters configured yet
# ℹ️  Consider adding detected tools to your guardy.yml configuration
# [3/3] 🔧 Running linting validation
# ✔ pre-commit hook completed successfully
```

### Test 2: Formatter Configuration Test
**Purpose**: Test formatter execution with configured tools

```bash
# 1. Create a test configuration file
cat > guardy.yml << 'EOF'
security:
  secret_detection: true
  patterns:
    - name: AWS Access Key
      regex: AKIA[0-9A-Z]{16}
      severity: Critical
      description: AWS Access Key ID
      enabled: true
  exclude_patterns:
    - "*.tmp"
  use_gitignore: true
  protected_branches:
    - main
  git_crypt: false

hooks:
  pre_commit: true
  commit_msg: true
  pre_push: true
  timeout: 300

mcp:
  enabled: false
  port: 8080
  host: localhost
  daemon: false

tools:
  auto_detect: true
  auto_install: false
  formatters:
    - name: rustfmt
      command: cargo fmt
      patterns:
        - "**/*.rs"
      check_command: rustfmt --version
      install:
        cargo: rustup component add rustfmt
        manual: "Install Rust toolchain: https://rustup.rs/"
  linters: []
EOF

# 2. Run the hook again
./target/release/guardy hooks run pre-commit --verbose

# Expected Output:
# Should show auto-detected tools AND configured formatters
```

### Test 3: Real Formatting Issue Test
**Purpose**: Test detection of actual formatting issues

```bash
# 1. Create a poorly formatted Rust file
mkdir -p test-src
cat > test-src/bad_format.rs << 'EOF'
fn main(){
let  x   =    5;
    println!("Hello, world! The value is: {}", x);
let mut y=10;
y+=1;
println!("Y is: {}",y);
}
EOF

# 2. Stage the file
git add test-src/bad_format.rs

# 3. Run the pre-commit hook
./target/release/guardy hooks run pre-commit --verbose

# Expected Output:
# Should detect formatting issues and fail the hook
# Look for: "❌ Code formatting issues found"
# Should show: "Files need formatting with rustfmt: test-src/bad_format.rs"
```

### Test 4: Auto-Detection Test
**Purpose**: Verify auto-detection works across different project types

```bash
# 1. Create JavaScript files to test JS detection
cat > package.json << 'EOF'
{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test project"
}
EOF

cat > index.js << 'EOF'
function   hello(){
console.log("Hello,    world!");
}
EOF

# 2. Run auto-detection
./target/release/guardy hooks run pre-commit --verbose

# Expected Output:
# Should detect both Rust and JavaScript tools
# Look for: "Auto-detected tools: Cargo (...), rustfmt (...), Clippy (...), NPM (...)"
```

### Test 5: Unit Tests
**Purpose**: Verify all unit tests pass

```bash
# Run all tests
cargo test --lib

# Run specific formatter tests
cargo test --lib hooks::tests

# Expected Output:
# All tests should pass, including:
# - test_glob_match
# - test_find_matching_files
# - test_is_conventional_commit
# - test_pre_commit_hook_with_formatters
```

### Test 6: Pattern Matching Test
**Purpose**: Test glob pattern matching functionality

```bash
# Test glob patterns manually
cargo test --lib hooks::tests::test_glob_match -- --nocapture

# Expected Output:
# test cli::commands::hooks::tests::test_glob_match ... ok
```

### Test 7: Edge Cases Test
**Purpose**: Test error handling and edge cases

```bash
# 1. Test with missing formatter
# Create guardy.yml with non-existent formatter
cat > guardy.yml << 'EOF'
tools:
  auto_detect: true
  auto_install: false
  formatters:
    - name: non-existent-formatter
      command: non-existent-command
      patterns:
        - "**/*.rs"
      check_command: non-existent-command --version
EOF

# 2. Run the hook
./target/release/guardy hooks run pre-commit --verbose

# Expected Output:
# Should gracefully handle missing formatter
# Look for: "Formatter 'non-existent-formatter' not available"
```

## Expected Behavior Summary

### ✅ Success Cases
1. **Auto-detection works**: Shows detected tools in verbose output
2. **Formatter execution**: Runs configured formatters in check mode
3. **Pattern matching**: Correctly matches files to formatter patterns
4. **Error reporting**: Clear feedback when formatting issues are found

### ✅ Error Cases
1. **Missing formatters**: Graceful handling with helpful error messages
2. **No staged files**: Appropriate message when no files to format
3. **Invalid configuration**: Proper error handling for malformed config

### ✅ Performance
1. **Staged files only**: Only processes files staged for commit
2. **Pattern filtering**: Only runs formatters on matching files
3. **Efficient execution**: Fast execution with minimal overhead

## Testing Checklist

- [ ] Test 1: Basic functionality with auto-detection
- [ ] Test 2: Formatter configuration and execution
- [ ] Test 3: Real formatting issue detection
- [ ] Test 4: Multi-language auto-detection
- [ ] Test 5: Unit tests pass
- [ ] Test 6: Pattern matching works correctly
- [ ] Test 7: Error handling for edge cases

## Known Issues / Limitations

1. **Formatter Installation**: Formatters must be pre-installed (no auto-install during hooks)
2. **Pattern Complexity**: Simple glob patterns only (no advanced regex patterns)
3. **Platform Support**: Some formatters may behave differently on different platforms

## Next Steps

After testing approval, the next phases would be:
1. **Parallel Execution**: Run formatters in parallel for better performance
2. **Error Aggregation**: Collect and display all errors together
3. **Lint Integration**: Add linter support similar to formatter integration
4. **Timeout Handling**: Add timeout controls for long-running formatters
5. **Configurable Checks**: Allow enabling/disabling specific checks

## Notes for Reviewer

- The implementation prioritizes security (no auto-install during hooks)
- All formatter execution is in check mode (dry-run) to avoid unexpected changes
- Auto-detection provides intelligent suggestions for configuration
- Pattern matching uses glob patterns for flexibility
- Error messages are actionable and user-friendly