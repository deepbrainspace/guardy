# Rust Git Hooks Implementation Plan

## Analysis Complete ✅
- Analyzed goodiebag husky hooks structure
- Identified 4 main hooks: pre-commit, commit-msg, post-checkout, pre-push
- Documented current functionality and requirements

## Current Hook Analysis

### Existing Hooks Functionality:
1. **pre-commit**: Branch protection, unstaged changes check, secret detection, git-crypt validation, code formatting
2. **commit-msg**: Conventional commit format validation  
3. **post-checkout**: Automatic dependency installation when package files change
4. **pre-push**: Lockfile synchronization validation
5. **utils.sh**: Shared utility functions for colored output and status messages

## Proposed Rust Architecture

### Project Structure (with MCP Server)
```
src/
├── main.rs                 # CLI entry point
├── lib.rs                 # Library exports
├── hooks/
│   ├── mod.rs             # Hook module exports
│   ├── pre_commit.rs      # Pre-commit hook implementation
│   ├── commit_msg.rs      # Commit message validation
│   ├── post_checkout.rs   # Post-checkout dependency management
│   └── pre_push.rs        # Pre-push validation
├── git/
│   ├── mod.rs             # Git operations module
│   ├── operations.rs      # Git command wrappers
│   └── commit.rs          # Commit message parsing
├── security/
│   ├── mod.rs             # Security module exports
│   ├── scanner.rs         # Secret detection engine
│   └── patterns.rs        # Secret pattern definitions
├── external/
│   ├── mod.rs             # External tool integrations
│   ├── formatters.rs      # Code formatting (NX, etc.)
│   └── package_managers.rs # Package manager operations
├── config/
│   ├── mod.rs             # Configuration management
│   └── languages.rs       # Language-specific configs
├── cli/
│   ├── mod.rs             # CLI module
│   ├── commands/          # Command implementations
│   │   ├── install.rs     # Hook installation
│   │   ├── run.rs         # Manual hook execution
│   │   ├── config.rs      # Configuration management
│   │   ├── status.rs      # Status reporting
│   │   ├── mcp.rs         # MCP server management
│   │   └── mod.rs         # Command exports
│   └── output.rs          # Colored output utilities
├── mcp/
│   ├── mod.rs             # MCP module exports
│   ├── server.rs          # MCP server implementation
│   ├── tools.rs           # MCP tool definitions
│   ├── types.rs           # MCP protocol types
│   └── utils.rs           # MCP utilities
└── shared/
    ├── mod.rs             # Shared utilities
    ├── patterns.rs        # Regex patterns
    └── glob.rs            # File pattern matching
```


  Proposed Rust implementation follows a modular architecture with:
  - Separate modules for hooks, git ops, security, external tools, config
  - Native Rust performance with async operations
  - TOML-based configuration system
  - Full CLI with install/run/config/status commands

## 4-6 Hour Implementation Plan (AI-Assisted)

### Hour 1: Project Foundation & Core Structure ✅ COMPLETED
- [x] **Setup Cargo.toml with latest dependencies** ✅ TESTED
  - ✅ **Test**: `cargo check` compiles without errors
  - ✅ **Test**: `cargo run -- --version` shows version "guardy 0.1.0"
  - ✅ **Test**: `cargo run -- --help` shows help with all commands
- [x] **Create module structure with empty files** ✅ TESTED
  - ✅ **Test**: `cargo check` compiles all modules without errors
  - ✅ **Test**: All module files exist and are properly organized
- [x] **Implement Claude Code-style CLI output utilities** ✅ TESTED
  - ✅ **Test**: `cargo run -- status` shows colored output with symbols (ℹ, ✔)
  - ✅ **Test**: Success, warning, info functions display proper colors and symbols
  - ✅ Professional symbols (✔, ✖, ⚠, ℹ, ❯)
  - ✅ Color-coded messaging

### Hour 2: Git Operations & CLI Framework ✅ COMPLETED
- [x] **Implement GitRepo struct with git2** ✅ TESTED
  - ✅ **Test**: `cargo run -- status` detects git repository correctly
  - ✅ **Test**: Shows current branch name "feat/guardy-revised"
  - ✅ **Test**: GitRepo.discover() and current_branch() methods working
  - ✅ current_branch() implemented, staged_files(), unstaged_files() ready for implementation
- [x] **Basic CLI structure with clap derive** ✅ TESTED
  - ✅ **Test**: All commands show in `cargo run -- --help` (install, run, config, status, uninstall, mcp, version)
  - ✅ **Test**: `cargo run -- version` (detailed) and `cargo run -- --version` (brief) both work
  - ✅ **Test**: `cargo run -- -C /path/to/dir status` changes directory correctly
  - ✅ Main commands: install, run, config, status, uninstall, mcp, version
  - ✅ Professional help text matching Claude Code style
- [x] **Error handling with anyhow** ✅ IMPLEMENTED

### Hour 3: Security Scanner & Secret Detection
- [ ] **SecretScanner implementation**
  - Configurable regex patterns for API keys, tokens, JWT
  - File exclusion logic for git-crypt
  - Interactive confirmation prompts
- [ ] **Git-crypt integration**
  - Status checking for encrypted files
  - Proper error handling when git-crypt missing

### Hour 4: Hook Implementations (Core Functionality)
- [ ] **Pre-commit Hook (Priority 1)**
  - Branch protection logic
  - Working tree cleanliness check
  - Secret detection integration
  - Code formatting trigger (NX integration)
- [ ] **Commit-msg Hook**
  - Conventional commit validation
  - Clear error messages with examples

### Hour 5: Remaining Hooks & External Integration
- [ ] **Post-checkout Hook**
  - Package.json change detection
  - Auto pnpm install execution
- [ ] **Pre-push Hook**
  - Lockfile sync validation
- [ ] **External tool integration**
  - Process execution with tokio
  - Package manager detection
  - NX command execution

### Hour 6: CLI Commands & Basic MCP Server
- [ ] **Complete CLI implementation**
  - `guardy install` - Install hooks to .git/hooks/
  - `guardy run <hook>` - Manual hook execution
  - `guardy status` - Show installation status
  - `guardy mcp start` - Start MCP server
- [ ] **Configuration system**
  - Basic TOML config loading
  - Environment variable support (GUARDY_DEBUG)
- [ ] **Basic MCP server setup**
  - MCP protocol types and server structure
  - Basic tool definitions for git operations
- [ ] **Testing & validation**
  - Basic integration test
  - Manual testing with real git repo

### Post-MVP: Advanced MCP Server (Additional 2-3 Hours)
- [ ] **MCP Server Tools**
  - Git status and branch operations
  - Hook execution and validation
  - Security scanning as a service
  - Configuration management
- [ ] **MCP Integration Features**
  - Real-time git repository monitoring
  - AI-assisted commit message generation
  - Automated security policy enforcement
  - Integration with Claude Code workflow

### Rapid Implementation Strategy

**AI-Assisted Development Approach:**
1. **Parallel Development**: Implement multiple modules simultaneously 
2. **Template-Driven**: Use existing patterns from goodiebag hooks
3. **Incremental Testing**: Test each component as it's built
4. **Copy-Adapt Pattern**: Translate bash logic directly to Rust
5. **Minimum Viable Product**: Focus on core functionality first

**Key Time-Savers:**
- Use Clap's derive macros for instant CLI generation
- Leverage git2's high-level APIs
- Copy exact regex patterns from existing hooks
- Use tokio for async process execution
- Reuse output formatting patterns from Claude Code

## Technical Decisions

### Dependencies (Latest Versions)
```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }  # CLI framework
git2 = "0.19"                                             # Git operations  
regex = "1.10"                                            # Pattern matching
serde = { version = "1.0", features = ["derive"] }       # Serialization
serde_json = "1.0"                                        # JSON serialization for MCP
anyhow = "1.0"                                            # Error handling
tokio = { version = "1.0", features = ["full"] }         # Async runtime
console = "0.15"                                          # Claude Code style terminal
globset = "0.4"                                           # File pattern matching
which = "6.0"                                             # Binary detection
toml = "0.8"                                              # Configuration parsing

# MCP Server dependencies (Latest Versions)
# Official Rust SDK from Model Context Protocol team
rmcp = { version = "0.2.0", features = ["server"] }      # Official MCP Rust SDK
# Alternative community implementation
rust-mcp-sdk = { version = "0.4", features = ["server", "macros"] }  # Community Rust MCP SDK
uuid = { version = "1.0", features = ["v4"] }            # Unique identifiers
tracing = "0.1"                                           # Structured logging
tracing-subscriber = "0.3"                               # Logging subscriber
```

### Architecture Principles
1. **Modular Design**: Each hook is a separate module with clear responsibilities
2. **Async by Default**: Use tokio for parallel operations where beneficial
3. **Configuration-Driven**: All behavior configurable via guardy.toml
4. **Error Recovery**: Graceful handling of missing external tools
5. **Performance First**: Minimize I/O and leverage Rust's performance
6. **Cross-Platform**: Support Windows, macOS, and Linux

### Migration Strategy
1. Side-by-side installation with existing Husky hooks
2. Feature flags for gradual adoption
3. Compatibility mode preserving existing behavior
4. Migration tool to convert .husky/ to guardy.toml

## Success Criteria (4-6 Hours)
- [ ] **Core functionality working** - All 4 hooks operational
- [ ] **Professional CLI** - Claude Code style output and UX  
- [ ] **Performance baseline** - Faster than bash equivalents
- [ ] **Easy installation** - Single `guardy install` command
- [ ] **Debug support** - GUARDY_DEBUG environment variable
- [ ] **Basic testing** - Manual validation with real git operations

## CLI Functionality Overview

### Core Commands

#### `guardy install [OPTIONS]`
Install git hooks into the current repository
- `--hooks <HOOKS>` - Specify which hooks to install (default: all)
- `--force` - Overwrite existing hooks
- `--config <FILE>` - Use custom configuration file
- Creates executable scripts in `.git/hooks/` that call `guardy run <hook>`

#### `guardy run <HOOK> [ARGS...]`
Manually execute a specific hook for testing
- `<HOOK>` - Hook name: pre-commit, commit-msg, post-checkout, pre-push
- `[ARGS...]` - Hook-specific arguments (e.g., commit message file for commit-msg)
- Useful for debugging and testing hook behavior

#### `guardy status`
Show current installation and configuration status
- Lists installed hooks and their status
- Shows configuration file location and settings
- Displays git repository information
- Reports any issues or warnings

#### `guardy config <SUBCOMMAND>`
Configuration management
- `guardy config init` - Create default guardy.toml
- `guardy config show` - Display current configuration
- `guardy config set <KEY> <VALUE>` - Set configuration value
- `guardy config get <KEY>` - Get configuration value
- `guardy config validate` - Validate configuration file

#### `guardy uninstall`
Remove all installed hooks
- Removes guardy scripts from `.git/hooks/`
- Preserves any existing non-guardy hooks
- Safe cleanup with confirmation prompt

#### `guardy mcp <SUBCOMMAND>`
MCP (Model Context Protocol) server management
- `guardy mcp start [OPTIONS]` - Start MCP server
  - `--port <PORT>` - Server port (default: auto-assign)
  - `--host <HOST>` - Bind address (default: localhost)
  - `--config <FILE>` - MCP server configuration
- `guardy mcp stop` - Stop running MCP server
- `guardy mcp status` - Show MCP server status
- `guardy mcp tools` - List available MCP tools

### Global Options
- `--directory, -C <DIR>` - Run as if started in `<DIR>` instead of current working directory
- `--verbose, -v` - Increase verbosity (can be repeated)
- `--quiet, -q` - Suppress non-error output
- `--config <FILE>` - Use custom configuration file
- `--help, -h` - Show help information
- `--version, -V` - Show version information

### Environment Variables
- `GUARDY_DEBUG=1` - Enable debug output (equivalent to multiple -v flags)
- `GUARDY_CONFIG` - Default configuration file path
- `GUARDY_NO_COLOR` - Disable colored output
- `GUARDY_MCP_PORT` - Default MCP server port

### Usage Examples

```bash
# Install all hooks
guardy install

# Install only pre-commit and commit-msg hooks
guardy install --hooks pre-commit,commit-msg

# Test pre-commit hook manually
guardy run pre-commit

# Start MCP server for Claude integration
guardy mcp start --port 8080

# Check status
guardy status

# Configure secret detection patterns
guardy config set security.patterns '["sk-[a-zA-Z0-9]{48}"]'

# Remove all hooks
guardy uninstall
```

### Professional CLI Features (Claude Code Style)
- **Rich help text** with examples and context
- **Colored output** with professional symbols (✔, ✖, ⚠, ℹ, ❯)
- **Progress indicators** for long-running operations
- **Interactive prompts** for confirmations and selections
- **Consistent error handling** with actionable suggestions
- **Tab completion** support (bash/zsh/fish)
- **Self-updating** capabilities

## Configuration System (guardy.toml)

### Why TOML?
- **Rust ecosystem standard** (same as Cargo.toml)
- **Human-readable** with comment support
- **Type-safe** parsing with serde
- **Familiar** to Rust developers
- **Less verbose** than JSON for configuration

### Configuration Structure

```toml
# guardy.toml - Main configuration file

[general]
# Global settings
debug = false
color = true
interactive = true

[hooks]
# Enable/disable specific hooks
pre_commit = true
commit_msg = true  
post_checkout = true
pre_push = true

[security]
# Secret detection patterns (regex)
patterns = [
    "sk-[a-zA-Z0-9]{48}",           # OpenAI API keys
    "ghp_[a-zA-Z0-9]{36}",          # GitHub personal access tokens
    "ey[a-zA-Z0-9]{20,}",           # JWT tokens
    "['\"][a-zA-Z0-9+/]{32,}['\"]", # Base64 encoded secrets
]
# Files to exclude from secret scanning
exclude_files = [
    "*.lock",
    "*.log", 
    ".husky/*",
]

[branch_protection]
# Protected branches
protected_branches = ["main", "master", "develop"]
allow_direct_commits = false

[git_crypt]
# Git-crypt integration
enabled = true
required_files = []  # Files that must be encrypted

[formatting]
# Code formatting settings
enabled = true
command = "nx format:write --uncommitted"
auto_fix = false  # Whether to auto-stage formatted files

[package_manager]
# Package manager preferences
preferred = "pnpm"  # pnpm, npm, yarn
auto_install = true  # Auto-install on checkout

[mcp]
# MCP server settings
port = 8080
host = "127.0.0.1"
enabled = false
tools = ["git-status", "hook-run", "security-scan"]

[external_tools]
# External tool paths (auto-detected if not specified)
git_crypt = "git-crypt"
nx = "nx"
pnpm = "pnpm"
```

### Configuration Priority (highest to lowest)
1. Command line arguments (`--config`, `--verbose`, etc.)
2. Environment variables (`GUARDY_CONFIG`, `GUARDY_DEBUG`)
3. Project-level `guardy.toml` 
4. User-level `~/.config/guardy/config.toml`
5. Built-in defaults

### Configuration Management Commands
- `guardy config init` - Create default guardy.toml
- `guardy config show` - Display merged configuration
- `guardy config set security.patterns '["new-pattern"]'` - Set values
- `guardy config get hooks.pre_commit` - Get specific values
- `guardy config validate` - Validate configuration file

## Next Steps (Post-MVP)
- Advanced configuration options
- Comprehensive test suite  
- Performance benchmarking
- Migration tooling from Husky
- Documentation and examples