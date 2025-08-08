# Guardy 🛡️

Fast, secure git hooks in Rust with MCP server integration.

## Overview

Guardy provides native Rust implementations of git hooks with security scanning, code formatting, and MCP server capabilities for AI integration. It replaces bash-based git hooks with high-performance, type-safe alternatives.

## Features

- 🚀 **Native Rust Performance** - Faster than bash equivalents
- 🔒 **Comprehensive Security Scanning** - Detects 40+ types of secrets and credentials
- 🎨 **Code Formatting** - Automatic formatting with NX integration
- 🔧 **Git-crypt Support** - Encrypted file handling
- 🤖 **MCP Server** - Model Context Protocol integration for AI tools
- ⚙️ **Advanced Configuration** - Smart format detection, empty value filtering, nested environment variables
- 🌊 **Professional CLI** - Claude Code-style output with symbols and colors

## Quick Start

```bash
# Install hooks in your repository
guardy install

# Scan files for secrets
guardy scan src/ --stats

# Check installation status
guardy status

# Test a hook manually
guardy run pre-commit

# Start MCP server for AI integration
guardy mcp start
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

### Usage Examples

```bash
# Scan current directory
guardy scan

# Scan specific files with statistics
guardy scan src/ config/ --stats

# Include binary files in scan
guardy scan --include-binary

# Set custom file size limit
guardy scan --max-file-size 50
```

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
- `guardy run <hook>` - Manually execute a specific hook for testing
- `guardy status` - Show current installation and configuration status
- `guardy config` - Configuration management commands
- `guardy uninstall` - Remove all installed hooks

### MCP Server Commands

- `guardy mcp start` - Start MCP server for AI integration
- `guardy mcp status` - Show MCP server status
- `guardy mcp tools` - List available MCP tools

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

## MCP Integration

Guardy includes an MCP (Model Context Protocol) server for seamless AI integration:

```bash
# Start MCP server
guardy mcp start --port 8080

# Available MCP tools
guardy mcp tools
```

### Available MCP Tools
- `git-status` - Get repository status
- `hook-run` - Execute hooks programmatically  
- `security-scan` - Run security scans on demand

## Module Organization

Guardy follows a clean modular architecture with clear separation of concerns:

```
src/
├── config/         # Configuration management (Figment-based)
│   ├── core.rs     # GuardyConfig struct and loading logic + tests
│   ├── formats.rs  # Export functionality and syntax highlighting + tests
│   ├── languages.rs # Language detection + tests
│   └── README.md   # Config module documentation
├── scanner/        # Secret detection and file analysis
│   ├── core.rs     # Main Scanner struct and file processing + tests
│   ├── patterns.rs # Secret patterns and regex compilation + tests
│   ├── entropy.rs  # Statistical entropy analysis + tests
│   ├── ignore_intel.rs # Project type detection + tests
│   └── README.md   # Scanner module documentation
├── git/           # Git operations and repository management
├── cli/           # Command-line interface and output formatting
├── hooks/         # Git hook implementations
├── mcp/           # Model Context Protocol server
└── tests/         # Integration tests (cross-module functionality)
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
│   ├── guardy/                          # Main application
│   │   ├── src/                         # Core guardy implementation
│   │   └── Cargo.toml                   # Package dependencies
│   └── guardy-figment-providers/        # Custom Figment providers  
│       ├── src/                         # SmartFormat, SkipEmpty, NestedEnv
│       ├── tests/                       # Comprehensive provider tests
│       └── README.md                    # Provider documentation
├── Cargo.toml                           # Workspace root
└── target/                              # Shared build artifacts
```

### Custom Figment Providers Package

The `guardy-figment-providers` package contains reusable configuration providers:
- **Independently testable** with 18 comprehensive tests
- **Reusable** across other Rust projects
- **Well-documented** with examples and usage patterns
- **Future crates.io distribution** ready

### Prerequisites
- Rust 1.70+ (uses 2024 edition)
- Git 2.0+

### Building
```bash
# Build entire workspace
cargo build --release

# Build specific package
cargo build -p guardy --release
cargo build -p guardy-figment-providers --release
```

### Testing
```bash
# Test entire workspace
cargo test

# Test specific package  
cargo test -p guardy-figment-providers
cargo test -p guardy

# Test specific provider
cargo test -p guardy-figment-providers smart_format

# Test specific modules
cargo test --lib config      # Test only config module
cargo test --lib scanner     # Test only scanner module  
cargo test integration_      # Run only integration tests
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
- Uses [Figment](https://github.com/SergioBenitez/Figment) for configuration management
- Built with [Clap](https://github.com/clap-rs/clap) for CLI interface