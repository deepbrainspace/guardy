# Guardy 🛡️

Fast, secure git hooks in Rust with MCP server integration.

## Overview

Guardy provides native Rust implementations of git hooks with security scanning, code formatting, and MCP server capabilities for AI integration. It replaces bash-based git hooks with high-performance, type-safe alternatives.

## Features

- 🚀 **Native Rust Performance** - Faster than bash equivalents
- 🔒 **Security Scanning** - Secret detection with configurable patterns
- 🎨 **Code Formatting** - Automatic formatting with NX integration
- 🔧 **Git-crypt Support** - Encrypted file handling
- 🤖 **MCP Server** - Model Context Protocol integration for AI tools
- ⚙️ **Flexible Configuration** - Multiple format support (TOML, JSON, YAML, Environment Variables)
- 🌊 **Professional CLI** - Claude Code-style output with symbols and colors

## Quick Start

```bash
# Install hooks in your repository
guardy install

# Check installation status
guardy status

# Test a hook manually
guardy run pre-commit

# Start MCP server for AI integration
guardy mcp start
```

## Configuration

Guardy supports flexible configuration through multiple formats and sources with automatic merging:

### Configuration Priority (highest to lowest)
1. **Environment Variables** - `GUARDY_*` prefixed variables
2. **Repository Config** - Project-specific settings
3. **User Config** - Personal defaults  
4. **Built-in Defaults** - Embedded fallback values

### Supported Configuration Formats

You can use any of these formats for your configuration:

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

### Prerequisites
- Rust 1.70+ (uses 2024 edition)
- Git 2.0+

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test                    # Run all tests (unit + integration)
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