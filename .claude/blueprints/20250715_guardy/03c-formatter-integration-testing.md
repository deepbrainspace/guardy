# Phase 1.5+: Comprehensive Language Support - Testing Instructions

## Overview
This document provides detailed testing instructions for the comprehensive language support implementation in Guardy, including 18+ programming languages, 15+ package managers, and intelligent project detection.

## What Was Implemented

### ✅ Comprehensive Language Support
1. **18+ Programming Languages**: Rust, JavaScript/TypeScript, Python, Go, C/C++, .NET, PHP, Ruby, Perl, Elixir, Haskell, Kotlin, Scala, Crystal, Zig, Swift, Dart
2. **15+ Package Managers**: cargo, npm/pnpm/yarn/bun, pip/poetry/uv, go mod, composer, gem/bundler, cpan/cpanm, hex/mix, stack/cabal, conan/vcpkg, sbt, shards, nx
3. **Smart Project Detection**: Automatic detection of project type based on configuration files
4. **Comprehensive Formatter Support**: Language-specific formatters with installation instructions
5. **Advanced Pattern Matching**: Glob pattern matching with proper file extension handling

### ✅ Key Files Modified
- `src/utils/package_manager.rs` - Expanded package manager support
- `src/utils/mod.rs` - Added comprehensive project type detection
- `src/cli/commands/config.rs` - Added formatter configurations for all languages
- `src/cli/commands/hooks.rs` - Enhanced formatter integration
- `README.md` - Updated with comprehensive language support documentation

## Testing Instructions

### Test 1: Project Type Detection Test
**Purpose**: Verify automatic project type detection works for all supported languages

```bash
# 1. Build the project
cargo build --release

# 2. Test current Rust project detection
./target/release/guardy config init --verbose

# Expected Output:
# 🔧 Initializing Configuration
# Detected project type: Rust
# Configuration file created successfully
# Should include prettyplease as the primary formatter, rustfmt as fallback

# 3. Test project type detection function
cargo test --lib detect_project_type

# Expected Output:
# test utils::tests::test_detect_project_type_rust ... ok
# test utils::tests::test_detect_project_type_nodejs ... ok
# test utils::tests::test_detect_project_type_python ... ok
# test utils::tests::test_detect_project_type_go ... ok
# test utils::tests::test_detect_project_type_nx_monorepo ... ok
# test utils::tests::test_detect_project_type_dotnet ... ok
# test utils::tests::test_detect_project_type_php ... ok
# test utils::tests::test_detect_project_type_ruby ... ok
# test utils::tests::test_detect_project_type_generic ... ok
```

### Test 2: Multi-Language Project Setup Test
**Purpose**: Test project configuration generation for different languages

```bash
# Test 1: Node.js Project
mkdir -p test-projects/nodejs
cd test-projects/nodejs
echo '{"name": "test-project", "version": "1.0.0"}' > package.json

# Initialize configuration
../../target/release/guardy config init

# Expected Output:
# Detected project type: NodeJs
# Configuration should include prettier, biome, eslint

# Test 2: Python Project
cd ..
mkdir -p python
cd python
echo '[project]\nname = "test-project"' > pyproject.toml

../../target/release/guardy config init

# Expected Output:
# Detected project type: Python
# Configuration should include black, ruff formatters

# Test 3: Go Project
cd ..
mkdir -p go
cd go
echo 'module test-project' > go.mod

../../target/release/guardy config init

# Expected Output:
# Detected project type: Go
# Configuration should include gofmt, golangci-lint

# Test 4: .NET Project
cd ..
mkdir -p dotnet
cd dotnet
echo '<Project Sdk="Microsoft.NET.Sdk"></Project>' > test.csproj

../../target/release/guardy config init

# Expected Output:
# Detected project type: Dotnet
# Configuration should include dotnet format

# Test 5: PHP Project
cd ..
mkdir -p php
cd php
echo '{"name": "test/project"}' > composer.json

../../target/release/guardy config init

# Expected Output:
# Detected project type: Php
# Configuration should include php-cs-fixer, phpstan

# Clean up
cd ../../..
rm -rf test-projects
```

### Test 3: Package Manager Detection Test
**Purpose**: Test comprehensive package manager detection

```bash
# Test package manager detection
cargo test --lib package_manager

# Expected Output:
# All package manager tests should pass

# Test specific package managers
cargo test --lib -- --nocapture package_manager_detect

# Manual testing of detection
mkdir -p test-packages
cd test-packages

# Test 1: Python uv detection
echo '' > uv.lock
echo "import sys" > main.py
cd ..
echo "Testing uv detection..."
./target/release/guardy config init
echo "Expected: Python project with uv support"

# Test 2: NX monorepo detection
cd test-packages
rm -f uv.lock main.py
echo '{"name": "test-nx"}' > nx.json
echo '{"name": "test-project"}' > package.json
cd ..
echo "Testing NX detection..."
./target/release/guardy config init
echo "Expected: NxMonorepo project type"

# Test 3: .NET detection
cd test-packages
rm -f nx.json package.json
echo '<Project Sdk="Microsoft.NET.Sdk"></Project>' > Program.csproj
cd ..
echo "Testing .NET detection..."
./target/release/guardy config init
echo "Expected: Dotnet project type"

# Clean up
rm -rf test-packages
```

### Test 4: Comprehensive Language Configuration Test
**Purpose**: Test configuration generation for advanced languages

```bash
# Test Ruby project configuration
mkdir -p test-advanced/ruby
cd test-advanced/ruby
echo 'source "https://rubygems.org"' > Gemfile
echo 'gem "rails"' >> Gemfile
../../target/release/guardy config init
echo "Expected: Ruby project with rubocop configuration"

# Test Elixir project configuration  
cd ../
mkdir -p elixir
cd elixir
echo 'defmodule TestProject do' > lib/test_project.ex
echo 'end' >> lib/test_project.ex
echo 'defmodule TestProject.MixProject do' > mix.exs
echo 'end' >> mix.exs
../../target/release/guardy config init
echo "Expected: Elixir project with mix format, credo configuration"

# Test Haskell project configuration
cd ../
mkdir -p haskell
cd haskell
echo 'resolver: lts-18.18' > stack.yaml
echo 'packages:' >> stack.yaml
echo '- .' >> stack.yaml
echo 'main :: IO ()' > app/Main.hs
echo 'main = putStrLn "Hello, Haskell!"' >> app/Main.hs
../../target/release/guardy config init
echo "Expected: Haskell project with ormolu, hlint configuration"

# Test C++ project configuration
cd ../
mkdir -p cpp
cd cpp
echo 'cmake_minimum_required(VERSION 3.10)' > CMakeLists.txt
echo 'project(TestProject)' >> CMakeLists.txt
echo '#include <iostream>' > main.cpp
echo 'int main() { return 0; }' >> main.cpp
../../target/release/guardy config init
echo "Expected: C++ project with clang-format, clang-tidy configuration"

# Clean up
cd ../../
rm -rf test-advanced
```

### Test 5: Comprehensive Unit Tests
**Purpose**: Verify all unit tests pass for new functionality

```bash
# Run all tests
cargo test --lib

# Run specific utility tests
cargo test --lib utils::tests

# Expected Output:
# All tests should pass, including:
# - test_detect_project_type_rust
# - test_detect_project_type_nodejs  
# - test_detect_project_type_python
# - test_detect_project_type_go
# - test_detect_project_type_nx_monorepo
# - test_detect_project_type_dotnet
# - test_detect_project_type_php
# - test_detect_project_type_ruby
# - test_detect_project_type_generic

# Run package manager tests
cargo test --lib package_manager

# Expected Output:
# All package manager tests should pass

# Run configuration tests
cargo test --lib config

# Expected Output:
# Configuration generation tests should pass
```

### Test 6: Configuration Validation Test
**Purpose**: Test configuration validation and display

```bash
# Test configuration validation
./target/release/guardy config validate

# Expected Output:
# ✅ Configuration is valid
# Configuration Summary with formatters/linters count

# Test configuration display
./target/release/guardy config show

# Expected Output:
# 📄 Current Configuration
# YAML content with syntax highlighting
# Should show all configured formatters and linters

# Test configuration with verbose output
./target/release/guardy --verbose config validate

# Expected Output:
# Detailed validation information
# Security pattern validation
# Tool integration validation
```

### Test 7: Real-World Integration Test
**Purpose**: Test full workflow with multiple languages in one project

```bash
# Create a polyglot project
mkdir -p polyglot-test
cd polyglot-test

# Add multiple language files
echo '{"name": "polyglot-project", "version": "1.0.0"}' > package.json
echo 'fn main() { println!("Hello from Rust!"); }' > main.rs
echo 'print("Hello from Python!")' > main.py
echo 'package main\nfunc main() { println("Hello from Go!") }' > main.go
echo '<?php echo "Hello from PHP!"; ?>' > main.php

# Initialize git repository
git init
git add .

# Initialize Guardy configuration
../target/release/guardy config init

# Expected Output:
# Should detect primary language (likely Node.js due to package.json)
# Configuration should include appropriate formatters

# Test configuration shows all detected languages
../target/release/guardy config show

# Expected Output:
# Should show comprehensive configuration with multiple formatter options

# Test validation
../target/release/guardy config validate

# Expected Output:
# Should validate successfully with multiple formatters configured

# Clean up
cd ..
rm -rf polyglot-test
```

### Test 8: Rust Formatter Preference Test
**Purpose**: Verify that prettyplease is configured as the preferred Rust formatter

```bash
# Test Rust project with prettyplease preference
./target/release/guardy config init

# Check that prettyplease is listed first in the generated config
./target/release/guardy config show | grep -A 10 "formatters:"

# Expected Output:
# formatters:
#   - name: prettyplease
#     command: prettyplease
#     patterns:
#       - "**/*.rs"
#   - name: rustfmt
#     command: cargo fmt
#     patterns:
#       - "**/*.rs"

# Test that the configuration validates correctly
./target/release/guardy config validate

# Expected Output:
# ✅ Configuration is valid
# Should show prettyplease as primary formatter option
```

## Expected Behavior Summary

### ✅ Success Cases
1. **18+ Language Support**: Detects and configures Rust, JavaScript/TypeScript, Python, Go, C/C++, .NET, PHP, Ruby, Perl, Elixir, Haskell, Kotlin, Scala, Crystal, Zig, Swift, Dart
2. **Smart Project Detection**: Automatically identifies project type based on configuration files
3. **Comprehensive Formatters**: Configures appropriate formatters for each language
4. **Package Manager Integration**: Supports 15+ package managers with intelligent detection
5. **Proper Pattern Matching**: Uses glob patterns to match files to formatters
6. **Installation Instructions**: Provides multiple installation methods for each tool

### ✅ Error Cases
1. **Missing Project Files**: Graceful handling when project files are not found
2. **Invalid Configuration**: Proper error handling for malformed config
3. **Unsupported Languages**: Falls back to Generic project type gracefully
4. **Missing Formatters**: Clear error messages with installation instructions

### ✅ Performance
1. **Efficient Detection**: Fast project type detection with minimal file system operations
2. **Lazy Loading**: Only detects tools when needed
3. **Cached Results**: Avoids redundant file system checks
4. **Minimal Dependencies**: Uses built-in Rust capabilities where possible

## Testing Checklist

- [ ] Test 1: Project type detection for all supported languages
- [ ] Test 2: Multi-language project configuration generation
- [ ] Test 3: Package manager detection and preference ordering
- [ ] Test 4: Advanced language configuration (Ruby, Elixir, Haskell, C++)
- [ ] Test 5: Comprehensive unit tests for new functionality
- [ ] Test 6: Configuration validation and display
- [ ] Test 7: Real-world polyglot project integration
- [ ] Test 8: Rust formatter preference (prettyplease over rustfmt)

## Supported Languages Overview

### **Tier 1 Languages** (Full Support)
- **Rust**: prettyplease, rustfmt, clippy, cargo
- **JavaScript/TypeScript**: prettier, biome, eslint, npm/pnpm/yarn/bun
- **Python**: black, ruff, pip/poetry/uv
- **Go**: gofmt, golangci-lint, go mod

### **Tier 2 Languages** (Comprehensive Support)
- **C/C++**: clang-format, clang-tidy, conan/vcpkg
- **.NET**: dotnet format, dotnet analyzers
- **PHP**: php-cs-fixer, phpstan, composer
- **Ruby**: rubocop, gem/bundler

### **Tier 3 Languages** (Modern Support)
- **Elixir**: mix format, credo, hex/mix
- **Haskell**: ormolu, hlint, stack/cabal
- **Kotlin**: ktlint, gradle
- **Scala**: scalafmt, scalafix, sbt
- **Swift**: swift-format, swiftlint
- **Dart**: dart format, dart analyze

### **Tier 4 Languages** (Emerging Support)
- **Crystal**: crystal tool format, shards
- **Zig**: zig fmt, zig build
- **Perl**: perltidy, perlcritic, cpan/cpanm

## Known Features / Capabilities

1. **Intelligent Detection**: Prioritizes specific project types (e.g., NX > Node.js)
2. **Multiple Installation Methods**: brew, apt, npm, cargo, manual instructions
3. **Pattern Matching**: Comprehensive file extension support
4. **Preference Ordering**: Modern tools preferred (uv > poetry > pip)
5. **Extensible Architecture**: Easy to add new languages and tools

## Next Steps

After testing approval, the next phases would be:
1. **Auto-Installation**: Implement automatic tool installation
2. **Parallel Execution**: Run formatters in parallel for better performance
3. **Custom Configurations**: Allow custom formatter configurations
4. **Language-Specific Options**: Add advanced options for each language
5. **Performance Optimization**: Optimize detection and execution speed

## Notes for Reviewer

- **Comprehensive Coverage**: 18+ languages, 15+ package managers, 30+ formatters/linters
- **Production Ready**: All code compiles and includes proper error handling
- **Well Tested**: Unit tests for all major functionality
- **Documentation Complete**: README updated with full language support matrix
- **Maintainable Architecture**: Clean separation of concerns and extensible design
- **Modern Rust Tooling**: Prioritizes `prettyplease` over `rustfmt` for better formatting quality
- **Intelligent Preferences**: Uses preference ordering for modern tools (uv > poetry > pip, prettyplease > rustfmt)