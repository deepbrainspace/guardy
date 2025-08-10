# Guardy 🛡️

Fast, secure git hooks in Rust with secret scanning and repository synchronization.

## Overview

Guardy provides high-performance Rust implementations for three essential repository management needs:
- **Git Hooks** - Prevent bad commits with automated checks
- **Secret Scanning** - Detect 40+ types of secrets before they're committed
- **Repository Sync** - Keep shared configurations synchronized across multiple repositories

## Features

- 🚀 **Native Rust Performance** - Faster than bash equivalents
- 🔒 **Comprehensive Security Scanning** - Detects 40+ types of secrets with entropy analysis
- 🔄 **Repository Synchronization** - Sync shared configs, CI/CD, and build tools across repos
- 🪝 **Complete Git Hook Support** - Pre-commit, pre-push, commit-msg, and post-checkout hooks
- ⚙️ **Flexible Configuration** - YAML, TOML, and JSON support with smart defaults
- 🎯 **Parallel Processing** - Multi-threaded scanning with automatic optimization
- 🌊 **Professional CLI** - Modern terminal output with progress bars and styled output

## Quick Start

```bash
# Install hooks in your repository
guardy install

# Scan files for secrets
guardy scan src/ --stats

# Show what files have drifted from protected sync
guardy sync diff

# Update files interactively
guardy sync

# Check installation status
guardy status

# Test a hook manually
guardy run pre-commit
```

## Security Scanning

Guardy includes comprehensive secret detection with 40+ built-in patterns:

### Private Keys & Certificates
- SSH private keys (RSA, DSA, EC, OpenSSH, SSH2)
- PGP/GPG private keys
- PKCS private keys
- PuTTY private keys
- Age encryption keys

### Cloud Provider Credentials
- **AWS**: Access keys, secret keys, session tokens
- **Azure**: Client secrets, storage keys
- **Google Cloud**: API keys, service account keys

### API Keys & Tokens
- **AI/ML**: OpenAI, Anthropic Claude, Hugging Face, Cohere, Replicate, Mistral
- **Development**: GitHub, GitLab, npm tokens
- **Services**: Slack, SendGrid, Twilio, Mailchimp, Stripe, Square
- **JWT/JWE**: JSON Web Tokens and encryption

### Database Credentials
- MongoDB connection strings
- PostgreSQL connection strings  
- MySQL connection strings
- Redis connection URLs

### Generic Detection
- Context-based secret detection (high-entropy strings near keywords like "password", "token", "key")
- URL credentials (`https://user:pass@host`)
- Custom configurable patterns

### Scanning Examples

```bash
# Scan current directory with statistics
guardy scan --stats

# Scan specific files and directories
guardy scan src/ config/ --parallel

# Interactive scanning mode
guardy scan --interactive

# Generate HTML report
guardy scan --output-format html --output report.html

# Include binary files and set custom limits
guardy scan --include-binary --max-file-size 50
```

## Repository Sync

Keep shared configurations, CI/CD workflows, and build tools synchronized across multiple repositories:

### Use Cases

- **Shared CI/CD**: Maintain consistent GitHub Actions across all repositories
- **Build Configurations**: Sync ESLint, Prettier, TypeScript configs
- **Documentation Templates**: Keep README structures and contributing guides aligned
- **Security Policies**: Ensure all repos have latest security configurations

### Configuration

Add sync configuration to your `guardy.yaml`:

```yaml
sync:
  repos:
    - name: "shared-config"
      repo: "https://github.com/yourorg/shared-configs"
      version: "v1.2.0"  # Use fixed versions for stability
      source_path: "."
      dest_path: "."
      include: ["*.yml", "*.json", ".editorconfig"]
      exclude: [".git", "node_modules"]
    
    - name: "ci-workflows"
      repo: "https://github.com/yourorg/ci-templates"
      version: "v2.0.1"
      source_path: ".github"
      dest_path: ".github"
      include: ["workflows/*.yml"]
```

### Sync Commands

```bash
# Show what has changed (diff mode)
guardy sync diff

# Update files interactively (default behavior)
guardy sync

# Force update all changes without prompts
guardy sync --force

# Show sync status for all repositories
guardy sync status
```

### Features

- **Version Pinning**: Use specific versions for reproducible syncs
- **Diff Visualization**: See exactly what changed with colored output
- **Interactive Updates**: Choose which files to update with y/n prompts
- **Multiple Sources**: Sync from multiple template repositories
- **Selective Sync**: Include/exclude patterns for fine-grained control

## Configuration

Guardy features an advanced configuration system with **smart format detection**, **empty value filtering**, and **nested environment variables**:

### Enhanced Configuration Features
- 🎯 **Smart Format Detection** - No file extensions needed, auto-detects JSON/YAML/TOML from content
- 🚫 **Empty Value Filtering** - CLI empty values don't override config files  
- 🌳 **Nested Environment Variables** - `GUARDY_SCANNER_MODE` → `scanner.mode`
- 🔄 **Intelligent Merging** - Advanced priority system with proper value preservation

### Configuration Priority (highest to lowest)
1. **CLI Overrides** - Command line arguments (filtered for empty values)
2. **Environment Variables** - `GUARDY_*` prefixed with automatic nesting
3. **Custom Config** - Via `--config` flag (format auto-detected)
4. **Repository Config** - Project-specific settings (format auto-detected)
5. **User Config** - `~/.config/guardy/config.*` (format auto-detected)  
6. **Built-in Defaults** - Embedded fallback values

### Supported Configuration Formats

All formats are **automatically detected** - no file extensions required:

#### Repository Level (project-specific)
- `guardy.toml` ⭐ **Recommended**
- `guardy.json`
- `guardy.yaml` or `guardy.yml`

#### User Level (personal defaults)
- `~/.config/guardy/config.toml` ⭐ **Recommended**  
- `~/.config/guardy/config.json`
- `~/.config/guardy/config.yaml` or `~/.config/guardy/config.yml`

#### Environment Variables
```bash
export GUARDY_GENERAL_DEBUG=true
export GUARDY_SECURITY_PATTERNS='["custom-[a-zA-Z0-9]{20,}"]'
export GUARDY_MCP_PORT=8080
```

### Configuration Examples

<details>
<summary><strong>TOML (Recommended)</strong></summary>

```toml
# guardy.toml
[general]
debug = false
color = true

[security]
patterns = [
    "sk-[a-zA-Z0-9]{48}",           # OpenAI API keys
    "ghp_[a-zA-Z0-9]{36}",          # GitHub tokens
    "custom-[a-zA-Z0-9]{20,}",      # Your custom pattern
]

[hooks]
pre_commit = true
commit_msg = true

[mcp]
enabled = true
port = 8080
```
</details>

<details>
<summary><strong>JSON</strong></summary>

```json
{
  "general": {
    "debug": false,
    "color": true
  },
  "security": {
    "patterns": [
      "sk-[a-zA-Z0-9]{48}",
      "ghp_[a-zA-Z0-9]{36}",
      "custom-[a-zA-Z0-9]{20,}"
    ]
  },
  "hooks": {
    "pre_commit": true,
    "commit_msg": true
  },
  "mcp": {
    "enabled": true,
    "port": 8080
  }
}
```
</details>

<details>
<summary><strong>YAML</strong></summary>

```yaml
# guardy.yaml
general:
  debug: false
  color: true

security:
  patterns:
    - "sk-[a-zA-Z0-9]{48}"           # OpenAI API keys
    - "ghp_[a-zA-Z0-9]{36}"          # GitHub tokens  
    - "custom-[a-zA-Z0-9]{20,}"      # Your custom pattern

hooks:
  pre_commit: true
  commit_msg: true

mcp:
  enabled: true
  port: 8080
```
</details>

### Dynamic Configuration

Guardy's configuration system is completely dynamic - you can add new sections and settings without code changes:

```toml
# Add your own custom sections
[my_custom_tool]
enabled = true
settings = ["value1", "value2"]

[experimental]
new_feature = "beta"
```

Access in extensions or plugins:
```rust
let config = GuardyConfig::load()?;
let enabled = config.get_bool("my_custom_tool.enabled")?;
let settings = config.get_vec("my_custom_tool.settings")?;
```

## Installation

### From Crates.io (Coming Soon)
```bash
cargo install guardy
```

### From Source
```bash
git clone https://github.com/deepbrainspace/guardy.git
cd guardy
cargo build --release
sudo cp target/release/guardy /usr/local/bin/
```

### Initialize in Repository
```bash
cd your-project
guardy install
```

## CLI Commands

### Core Commands

- `guardy install` - Install git hooks into the current repository
- `guardy scan [paths]` - Scan files for secrets and security issues
- `guardy sync [command]` - Synchronize shared configurations
- `guardy run <hook>` - Manually execute a specific hook for testing
- `guardy status` - Show current installation and configuration status
- `guardy config` - Configuration management commands
- `guardy uninstall` - Remove all installed hooks

### Global Options

- `-C, --directory <DIR>` - Run in different directory (like `git -C`)
- `-v, --verbose` - Increase verbosity (can be repeated)
- `-q, --quiet` - Suppress non-error output
- `--config <FILE>` - Use custom configuration file

## Git Hooks

Guardy implements these git hooks:

### Pre-commit Hook
- ✅ Branch protection (prevents direct commits to main/master)
- ✅ Working tree validation (no unstaged changes)
- ✅ Secret detection with configurable patterns
- ✅ Git-crypt encrypted file handling
- ✅ Code formatting with NX integration

### Commit-msg Hook
- ✅ Conventional commit validation
- ✅ Message length limits
- ✅ Clear error messages with examples

### Post-checkout Hook
- ✅ Automatic dependency installation
- ✅ Package manager detection (pnpm/npm/yarn)
- ✅ Smart change detection

### Pre-push Hook
- ✅ Lockfile synchronization validation
- ✅ Test suite execution (if configured)

## Module Organization

Guardy follows a clean modular architecture with clear separation of concerns:

```
src/
├── cli/           # Command-line interface and output formatting
├── config/        # Configuration management (superconfig-based)
├── git/           # Git operations and repository management
├── hooks/         # Git hook implementations
│   ├── config.rs  # Hook configuration
│   └── executor.rs # Hook execution logic
├── scanner/       # Secret detection and file analysis
│   ├── core.rs    # Main Scanner struct and file processing
│   ├── patterns.rs # Secret patterns and regex compilation
│   ├── entropy.rs # Statistical entropy analysis
│   └── types.rs   # Scanner types and configuration
├── sync/          # Repository synchronization
│   ├── manager.rs # Sync orchestration
│   └── status.rs  # Sync status tracking
├── parallel/      # Parallel execution framework
│   ├── core.rs    # Execution strategies
│   └── progress.rs # Progress bars and reporting
├── reports/       # Output formatting (JSON, HTML, etc.)
└── shared/        # Shared utilities
```

### Test Organization

**✅ Proper Test Structure:**
- **Unit Tests**: Inline with `#[cfg(test)] mod tests` in each implementation file
- **Integration Tests**: Separate `/tests/` directory for cross-module functionality
- **Module Tests**: Specific to the file they test (e.g., config tests in `config/core.rs`)

**❌ Avoid:**
- Separate `tests.rs` files (use inline tests instead)
- Tests in `mod.rs` files (routing only)
- Mixing tests from different modules

**Finding Module Information:**
- Each module has its own `README.md` with architecture and usage guidelines
- Check module README for specific test placement and contribution guidelines

## Development

### Monorepo Structure

This project uses a Cargo workspace with multiple packages:

```
guardy/
├── packages/
│   ├── guardy/                # Main application
│   │   ├── src/               # Core guardy implementation
│   │   ├── examples/          # Usage examples
│   │   └── Cargo.toml         # Package dependencies
│   └── supercli/              # Universal CLI styling (to be extracted)
│       ├── src/               # Styling and output utilities
│       └── Cargo.toml         # Package dependencies
├── Cargo.toml                 # Workspace root
└── target/                    # Shared build artifacts
```

### Related Packages

Guardy integrates with these external packages:
- **[superconfig](https://github.com/deepbrainspace/superconfig)** - Advanced configuration management
- **supercli** - Professional CLI output styling (currently bundled, to be extracted)

### Prerequisites
- Rust 1.88+ (uses 2024 edition)
- Git 2.0+

### Building
```bash
# Build entire workspace
cargo build --release

# Build specific package
cargo build -p guardy --release
cargo build -p supercli --release
```

### Testing
```bash
# Test entire workspace
cargo test

# Test specific package  
cargo test -p guardy
cargo test -p supercli

# Run tests with output
cargo test -- --nocapture

# Test specific modules
cargo test --lib scanner     # Test only scanner module
cargo test --lib sync        # Test only sync module  
cargo test --lib hooks       # Test only hooks module
```

### Contributing
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Husky](https://github.com/typicode/husky) for JavaScript
- Built with [Clap](https://github.com/clap-rs/clap) for CLI interface
- Uses [starbase-styles](https://github.com/moonrepo/starbase) for terminal styling
- Configuration powered by [superconfig](https://github.com/deepbrainspace/superconfig)