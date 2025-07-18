# Guardy 🛡️

**Intelligent Git Workflows for Modern Developers**

Guardy is a revolutionary MCP-first developer workflow intelligence tool that provides comprehensive language support, automated formatting, security scanning, and Git hook management. Built with Rust for maximum performance and reliability.

## ✨ Key Features

- **🌍 Comprehensive Language Support**: 18+ programming languages with automatic detection
- **🔧 Smart Formatting**: Auto-detects and configures formatters for your project type
- **🔒 Security Scanning**: Advanced pattern matching for secrets and vulnerabilities
- **🪝 Git Hook Management**: Automated pre-commit, commit-msg, and pre-push hooks
- **📦 Package Manager Integration**: Supports 15+ package managers with intelligent detection
- **🤖 MCP Integration**: Model Context Protocol support for AI-powered workflows
- **⚡ High Performance**: Built with Rust for speed and reliability

## 🚀 Supported Languages & Ecosystems

### Programming Languages
- **Rust** - `prettyplease`, `rustfmt`, `clippy`
- **JavaScript/TypeScript** - `prettier`, `biome`, `eslint`
- **Python** - `black`, `ruff`, `ruff-lint`
- **Go** - `gofmt`, `golangci-lint`
- **C/C++** - `clang-format`, `clang-tidy`
- **.NET (C#/F#/VB)** - `dotnet format`, `dotnet analyzers`
- **PHP** - `php-cs-fixer`, `phpstan`
- **Ruby** - `rubocop`
- **Perl** - `perltidy`, `perlcritic`
- **Elixir** - `mix format`, `credo`
- **Haskell** - `ormolu`, `hlint`
- **Kotlin** - `ktlint`
- **Scala** - `scalafmt`, `scalafix`
- **Crystal** - `crystal tool format`
- **Zig** - `zig fmt`
- **Swift** - `swift-format`, `swiftlint`
- **Dart** - `dart format`, `dart analyze`

### Package Managers
- **JavaScript**: npm, pnpm, yarn, bun
- **Python**: pip, poetry, uv
- **Rust**: cargo
- **Go**: go mod
- **PHP**: composer
- **Ruby**: gem, bundler
- **Elixir**: hex, mix
- **Haskell**: stack, cabal
- **C/C++**: conan, vcpkg
- **Scala**: sbt
- **Crystal**: shards
- **Zig**: zig build
- **NX**: nx (monorepo)

### Project Types Auto-Detection
Guardy automatically detects and configures support for:
- **Rust** (`Cargo.toml`)
- **Node.js** (`package.json`)
- **Python** (`pyproject.toml`, `requirements.txt`, `uv.lock`, `poetry.lock`)
- **Go** (`go.mod`)
- **NX Monorepo** (`nx.json`)
- **.NET** (`*.csproj`, `*.sln`, `global.json`)
- **PHP** (`composer.json`)
- **Ruby** (`Gemfile`, `*.gemspec`)
- **Perl** (`cpanfile`, `Makefile.PL`)
- **Elixir** (`mix.exs`)
- **Haskell** (`stack.yaml`, `*.cabal`)
- **C/C++** (`CMakeLists.txt`, `Makefile`, `conanfile.txt`)
- **Kotlin** (`build.gradle.kts` + `*.kt` files)
- **Scala** (`build.sbt`)
- **Crystal** (`shard.yml`)
- **Zig** (`build.zig`)
- **Swift** (`Package.swift`, `*.xcodeproj`)
- **Dart** (`pubspec.yaml`)

## Quick Setup

1. **Install Guardy**:
   ```bash
   cargo install guardy
   ```

2. **Initialize in your project**:
   ```bash
   guardy init
   ```

3. **Configuration**:
   Guardy uses `guardy.yml` or `.guardy.yml` for configuration.

## Global Options

Guardy supports several global flags that work with all commands:

### Output Control
- `--verbose, -v`: Show detailed output with extra information
- `--quiet, -q`: Minimal output (only errors and essential results)
- `--format <FORMAT>`: Output format (text, json, yaml) - default: text

### Behavior Control
- `--force, -f`: Skip confirmations and overwrite without prompting
- `--dry-run`: Show what would be done without executing
- `--auto-install`: Automatically install missing tools instead of failing
- `--config, -c <FILE>`: Use custom configuration file path

### Examples
```bash
# Verbose security scan
guardy --verbose security scan

# Quiet operation for scripts
guardy --quiet security scan

# Force install hooks without prompts
guardy --force hooks install

# JSON output for processing
guardy --format json security scan

# Dry run to see what would happen
guardy --dry-run init
```

## File Exclusions

Guardy respects multiple exclusion mechanisms:

- **`.guardyignore`**: Guardy-specific exclusions (always loaded)
- **`.gitignore`**: Git exclusions (when `use_gitignore: true`)
- **Configuration**: Manual patterns in `guardy.yml`

### .guardyignore File

Create a `.guardyignore` file to exclude files that contain test secrets or sensitive content:

```
# Testing files
TESTING.md
tests/
*.test.rs
test_*

# Documentation
docs/
*.example
README*
```

This ensures test files with intentional secrets are not flagged during security scans.

## Commands

### Security Scanning
```bash
# Scan current directory
guardy security scan

# Scan specific files (note: use -i for input files)
guardy security scan -i src/main.rs src/lib.rs

# Scan specific directory
guardy security scan -d src/

# Scan with verbose output
guardy --verbose security scan

# Scan with JSON output
guardy --format json security scan
```

### Git Hooks Management
```bash
# Install hooks
guardy hooks install

# Force install (overwrite existing)
guardy --force hooks install

# List available hooks
guardy hooks list

# Remove hooks
guardy hooks remove
```

### Configuration
```bash
# Initialize configuration
guardy config init

# Show current configuration
guardy config show

# Validate configuration
guardy config validate
```
