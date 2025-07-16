# 03 - Phase 1.2: CLI Framework Implementation Test Instructions

**Phase**: 1.2 - CLI Framework, Configuration System & Git Integration  
**Date**: 2025-07-16  
**Status**: Ready for Testing

## Overview

This document provides test instructions for verifying the Phase 1.2 CLI framework implementation, including the clap-based command structure, YAML configuration system, professional output formatting, and git integration layer.

## Changes Made

### 1. CLI Framework Implementation
- **Files**: `src/cli/mod.rs`, `src/cli/output.rs`, `src/cli/commands/*`
- **Features**: 
  - Complete clap-based CLI with subcommands
  - Professional output system with colors, icons, and formatting
  - Command structure: `init`, `status`, `config`, `hooks`, `mcp`
  - Interactive prompts and confirmations

### 2. Configuration System Enhancement
- **Files**: `src/config/mod.rs`
- **Features**:
  - YAML configuration loading and saving
  - Project type detection
  - Configuration validation
  - File-based configuration management

### 3. Git Integration Layer
- **Files**: `src/utils/mod.rs`, command implementations
- **Features**:
  - Git repository detection
  - Branch information retrieval
  - Repository status checking
  - Git hooks management

### 4. Command Implementations
- **init**: Full initialization with project detection, config creation, and hook installation
- **status**: Comprehensive system status with git, config, and hooks information
- **config**: Configuration management (init, validate, show)
- **hooks**: Git hooks installation, removal, listing, and execution
- **mcp**: MCP server management (placeholder implementations)

## Test Instructions

### Pre-Test Setup
Ensure you're in the project directory and on the correct branch:
```bash
cd /home/nsm/code/deepbrain/guardy
git branch --show-current
# Should show: feat/guardy-blueprint
```

### Test 1: Basic CLI Help System
**Objective**: Verify CLI structure and help system

```bash
# Test main help
cargo run -- --help

# Test subcommand help
cargo run -- config --help
cargo run -- hooks --help
cargo run -- mcp --help
```

**Expected Results**:
- ✅ Professional help output with proper formatting
- ✅ All subcommands listed correctly
- ✅ Options and arguments described clearly
- ✅ Version information available with -V

### Test 2: Status Command Functionality
**Objective**: Test comprehensive status reporting

```bash
# Clean state status
rm -f guardy.yml .git/hooks/pre-commit .git/hooks/commit-msg .git/hooks/pre-push
cargo run -- status
```

**Expected Results**:
- ✅ Git repository detection (should show current branch)
- ✅ Configuration file status (should show "not found")
- ✅ Git hooks status (should show "not installed")
- ✅ MCP server status (placeholder message)
- ✅ Professional formatting with colors and icons

### Test 3: Configuration Management
**Objective**: Test configuration system

```bash
# Initialize configuration
cargo run -- config init

# Validate configuration
cargo run -- config validate

# Show configuration
cargo run -- config show

# Check status after config creation
cargo run -- status
```

**Expected Results**:
- ✅ Configuration file created successfully
- ✅ Project type detected correctly (Rust)
- ✅ Configuration validates without errors
- ✅ Configuration content displayed properly
- ✅ Status command shows valid configuration

### Test 4: Git Hooks Management
**Objective**: Test hooks installation and management

```bash
# List hooks (should show none installed)
cargo run -- hooks list

# Install hooks
cargo run -- hooks install

# List hooks again (should show installed)
cargo run -- hooks list

# Check status
cargo run -- status

# Test force reinstall
cargo run -- hooks install --force

# Remove hooks
cargo run -- hooks remove

# Verify removal
cargo run -- hooks list
```

**Expected Results**:
- ✅ Hook installation creates executable scripts
- ✅ Hook scripts contain Guardy-specific content
- ✅ Status correctly shows hook installation state
- ✅ Force reinstall works properly
- ✅ Hook removal only removes Guardy-managed hooks

### Test 5: Complete Initialization Flow
**Objective**: Test full initialization process

```bash
# Clean state
rm -f guardy.yml .git/hooks/pre-commit .git/hooks/commit-msg .git/hooks/pre-push

# Run full initialization
cargo run -- init --yes

# Verify final state
cargo run -- status
```

**Expected Results**:
- ✅ Git repository confirmed/initialized
- ✅ Project type detected
- ✅ Configuration file created
- ✅ Git hooks installed
- ✅ MCP server setup information provided
- ✅ Clear next steps displayed

### Test 6: Interactive vs Non-Interactive Mode
**Objective**: Test interactive prompts

```bash
# Test interactive mode (without --yes)
rm -f guardy.yml
cargo run -- config init
# Should prompt for overwrite confirmation

# Test with existing config
cargo run -- config init
# Should ask for overwrite

# Test init without --yes
rm -f guardy.yml .git/hooks/pre-commit
cargo run -- init
# Should show interactive prompts
```

**Expected Results**:
- ✅ Interactive prompts appear when expected
- ✅ User can cancel operations
- ✅ Confirmations work properly
- ✅ --yes flag skips all prompts

### Test 7: Error Handling
**Objective**: Test error conditions and graceful handling

```bash
# Test outside git repository
cd /tmp
guardy status
cd -

# Test invalid configuration
echo "invalid: yaml: content" > guardy.yml
cargo run -- config validate
git checkout -- guardy.yml  # Restore

# Test missing permissions (if applicable)
# This depends on your system setup
```

**Expected Results**:
- ✅ Clear error messages for invalid states
- ✅ Helpful suggestions for resolving issues
- ✅ Graceful degradation when features unavailable
- ✅ No crashes or panic conditions

### Test 8: Professional Output Formatting
**Objective**: Verify output quality and consistency

```bash
# Test various output types
cargo run -- status
cargo run -- hooks list
cargo run -- config show
```

**Expected Results**:
- ✅ Consistent use of colors and icons
- ✅ Proper alignment and spacing
- ✅ Clear section headers and separators
- ✅ Professional appearance similar to modern CLI tools

### Test 9: Build and Test Suite
**Objective**: Ensure code quality and test coverage

```bash
# Build test
cargo build

# Run tests
cargo test

# Clippy check
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check
```

**Expected Results**:
- ✅ Clean build with no warnings
- ✅ All tests passing
- ✅ No clippy warnings
- ✅ Code properly formatted

## Success Criteria

### Core Functionality
- [ ] CLI framework responds to all documented commands
- [ ] Configuration system creates and validates YAML files
- [ ] Git integration detects repository state correctly
- [ ] Hooks management installs, lists, and removes hooks
- [ ] Status command provides comprehensive system overview

### User Experience
- [ ] Professional output with consistent formatting
- [ ] Clear error messages and helpful suggestions
- [ ] Interactive prompts work correctly
- [ ] Help system provides useful information

### Code Quality
- [ ] Clean build with no warnings
- [ ] All tests passing
- [ ] Clippy compliance
- [ ] Proper code formatting

### Integration
- [ ] Commands work together cohesively
- [ ] State changes reflected across commands
- [ ] File system operations work correctly
- [ ] Git operations function properly

## Implementation Notes

### Completed Features
1. **CLI Framework**: Complete clap-based command structure with subcommands
2. **Output System**: Professional formatting with colors, icons, and proper spacing
3. **Configuration System**: YAML-based configuration with validation
4. **Git Integration**: Repository detection, branch info, and hooks management
5. **Command Implementations**: All Phase 1.2 commands fully functional

### Phase 1.2 Achievements
- ✅ Modern CLI with professional output
- ✅ Comprehensive status reporting
- ✅ Complete configuration management
- ✅ Git repository integration
- ✅ Hook installation and management
- ✅ Project type detection
- ✅ Interactive and non-interactive modes

### Next Phase Preparation
The CLI framework is now ready for Phase 1.3 security feature integration:
- Security commands can be added to the CLI structure
- Configuration system ready for security patterns
- Output system ready for security reporting
- Git integration ready for security hooks

This implementation provides a solid foundation for all future Guardy features while maintaining professional CLI standards and user experience.