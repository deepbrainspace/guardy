# Guardy 

[![Crates.io](https://img.shields.io/crates/v/guardy.svg)](https://crates.io/crates/guardy)
[![Documentation](https://docs.rs/guardy/badge.svg)](https://docs.rs/guardy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast, secure git hooks in Rust with secret scanning and protected file synchronization.

## Features

- 🚀 **Fast Security Scanning**: Multi-threaded secret detection with entropy analysis
- 🔄 **Protected File Synchronization**: Keep configuration files in sync across repositories
- 🪝 **Git Hook Support**: Pre-commit, pre-push, and other git hooks
- ⚙️ **Flexible Configuration**: YAML, TOML, and JSON configuration support
- 📊 **Multiple Output Formats**: JSON, HTML, and plain text reporting
- 🔍 **Comprehensive Scanning**: Detect secrets, credentials, and sensitive data

## Installation

### From crates.io

```bash
cargo install guardy
```

### From source

```bash
git clone https://github.com/deepbrainspace/guardy
cd guardy
cargo build --release
```

## Quick Start

### 1. Initialize in your repository

```bash
cd your-repo/
guardy install
```

This installs git hooks and creates a default configuration.

### 2. Configure hooks

Guardy supports both custom commands and built-in actions in hooks:

```yaml
# guardy.yaml
hooks:
  pre-commit:
    enabled: true
    # Built-in actions
    builtin: ["scan_secrets"]
    # Custom commands
    custom:
      - command: "echo 'Running pre-commit checks...'"
        description: "Pre-commit header"
        fail_on_error: false

  pre-push:
    enabled: true
    custom:
      - command: "guardy sync update --force --config ./guardy.yaml"
        description: "Sync protected files before push"
        fail_on_error: true
```

### 3. Configure repository sync (optional)

Keep files synchronized from upstream repositories:

```yaml
# guardy.yaml
sync:
  repos:
    - name: "shared-configs"
      repo: "https://github.com/your-org/shared-configs"
      version: "v1.0.0"  # Can be tag, branch, or commit
      source_path: ".github"
      dest_path: "./.github"
      include: ["**/*"]
      exclude: ["*.md"]
```

### 4. Configure scanning (optional)

```yaml
# guardy.yaml
scanner:
  file_extensions:
    - "*.rs"
    - "*.js" 
    - "*.py"
  ignore_patterns:
    - "target/"
    - "node_modules/"
  entropy_threshold: 3.0

hooks:
  pre_commit:
    enabled: true
    commands:
      - scan
```

### 3. Use the core features

```bash
# Scan files for secrets
guardy scan src/

# Check installation status
guardy status

# Sync configuration files
guardy sync
```

## Commands

### Core Commands

- `guardy install` - Install git hooks in the current repository
- `guardy scan <PATH>` - Scan files/directories for secrets and sensitive data
- `guardy status` - Show installation and configuration status
- `guardy config` - Manage configuration settings
- `guardy uninstall` - Remove all installed git hooks

### File Synchronization

- `guardy sync` - Interactively update files from remote repositories
- `guardy sync diff` - Show differences without making changes
- `guardy sync --force` - Update all changes without prompting
- `guardy sync status` - Show sync configuration and status

### Advanced

- `guardy run <HOOK>` - Manually run a specific git hook for testing

## Configuration

Guardy supports multiple configuration formats (YAML, TOML, JSON):

### Basic Configuration (guardy.yaml)

```yaml
# Scanner settings
scanner:
  file_extensions:
    - "*.rs"
    - "*.js"
    - "*.py"
    - "*.go"
  ignore_patterns:
    - "target/"
    - "node_modules/"
    - "*.log"
  max_file_size: 1048576  # 1MB
  entropy_threshold: 3.5
  
# Git hooks configuration
hooks:
  pre-commit:
    enabled: true
    builtin: ["scan_secrets"]  # Built-in secret scanning
    custom: []  # Add custom commands here
  pre-push:
    enabled: true
    custom:
      - command: "guardy sync update --force --config ./guardy.yaml"
        description: "Sync protected files"
        fail_on_error: true

# File synchronization
sync:
  repos:
    - name: "shared-configs"
      repo: "https://github.com/yourorg/shared-configs"
      version: "main"
      source_path: "."
      dest_path: "."
      include: ["*.yml", "*.json", ".gitignore"]
      exclude: [".git", "target/"]
```

## Library Usage

Guardy can be used as a library for building custom security tools:

```rust
use guardy::scanner::ScannerConfig;
use guardy::config::GuardyConfig;

// Load configuration
let config = GuardyConfig::load("guardy.yaml", None, 0)?;
let scanner_config = ScannerConfig::from_config(&config)?;

// Scan for secrets
let results = scanner_config.scan_path("src/")?;

// Process findings
for finding in results.findings {
    println!(
        "Secret found in {}: {} (confidence: {:.2})", 
        finding.file_path,
        finding.secret_type,
        finding.confidence
    );
}
```

## Git Hooks Integration

Guardy provides flexible git hook management with both built-in actions and custom commands:

### Built-in Actions
- `scan_secrets` - Scan staged files for secrets and credentials
- `validate_commit_msg` - Validate commit message format (placeholder)

### Hook Configuration
```yaml
hooks:
  pre-commit:
    enabled: true
    builtin: ["scan_secrets"]
    custom:
      - command: "cargo fmt --check"
        description: "Check code formatting"
        fail_on_error: true
      
  pre-push:
    enabled: true
    custom:
      - command: "cargo test"
        description: "Run tests"
        fail_on_error: true
      - command: "guardy sync update --force --config ./guardy.yaml"
        description: "Sync protected files"
        fail_on_error: true
```

### Installing Specific Hooks
```bash
# Install all hooks
guardy install

# Install specific hooks
guardy install --hooks pre-commit,pre-push

# Force overwrite existing hooks
guardy install --force
```

## Protected File Synchronization

Keep configuration files synchronized across multiple repositories:

```bash
# Configure sync in guardy.yaml
guardy sync status          # Show sync configuration

guardy sync diff            # Preview changes without applying
guardy sync                 # Interactive update with diffs  
guardy sync --force         # Apply all changes automatically

# Bootstrap from a repository
guardy sync --repo=https://github.com/org/configs --version=main
```

### Automating Sync with Hooks

Integrate sync into your git workflow to ensure files stay synchronized:

```yaml
# guardy.yaml
sync:
  repos:
    - name: "shared-configs"
      repo: "https://github.com/org/shared-configs"
      version: "v1.0.0"
      source_path: ".github"
      dest_path: "./.github"
      include: ["**/*"]

hooks:
  pre-push:
    enabled: true
    custom:
      - command: "guardy sync update --force --config ./guardy.yaml"
        description: "Ensure configs are synchronized before push"
        fail_on_error: true
```

This ensures protected files are always synchronized before pushing changes.

Features:
- **Diff visualization** with syntax highlighting
- **Interactive updates** with per-file control
- **Selective sync** with include/exclude patterns
- **Version pinning** to specific tags or commits
- **Multi-repository** configuration support
- **Automatic restoration** of modified protected files

## Examples

### Scanning specific file types

```bash
# Scan only Rust files
guardy scan --include="*.rs" src/

# Scan excluding test files  
guardy scan --exclude="*test*" .

# Output as JSON
guardy scan --format=json src/ > scan-results.json
```

### Custom git hooks

```yaml
# guardy.yaml
hooks:
  pre-commit:
    enabled: true
    builtin: ["scan_secrets"]
    custom:
      - command: "cargo fmt -- --check"
        description: "Check formatting"
        fail_on_error: true
      - command: "cargo clippy -- -D warnings"
        description: "Run clippy"
        fail_on_error: true
```

### File sync with filters

```yaml
sync:
  repos:
    - name: "eslint-config"
      repo: "https://github.com/company/eslint-configs"  
      version: "v2.1.0"
      source_path: "configs"
      dest_path: "."
      include: [".eslintrc*", "prettier.config.js"]
      exclude: ["*.local.*"]
```

## Performance

- **Multi-threaded**: Utilizes all CPU cores for scanning
- **Memory efficient**: Processes large repositories without high memory usage
- **Fast I/O**: Optimized file reading with memory-mapped files
- **Smart filtering**: Skips binary files and respects .gitignore patterns

Typical performance on a modern machine:
- ~50,000 files/second for secret scanning
- ~1GB/second throughput for large codebases
- <100ms startup time for git hooks

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Support

- 📚 [Documentation](https://docs.rs/guardy)
- 🐛 [Issues](https://github.com/deepbrainspace/guardy/issues)
- 💬 [Discussions](https://github.com/deepbrainspace/guardy/discussions)