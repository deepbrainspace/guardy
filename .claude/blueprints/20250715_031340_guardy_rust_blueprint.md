# Guardy: Intelligent Git Workflows for Modern Developers

## Project Overview

**Guardy by DeepBrain** - A developer workflow intelligence tool written in pure Rust, designed to streamline git workflows with smart enforcement of quality, security, and team standards. Inspired by husky's simplicity and lint-staged's polish, but focused on comprehensive workflow automation.

**Domain**: guardy.dev  
**Repository**: github.com/deepbrainspace/guardy  
**Packages**: `guardy` (cargo) / `@deepbrainspace/guardy` (pnpm wrapper)  
**Sponsorship**: github.com/sponsors/wizardsupreme

## Reference Repositories

This project builds upon and aims to replace existing implementations:

### Primary References
- **Husky Fork**: `/home/nsm/code/forks/husky` - Reference implementation for git hooks
- **GoodieBag Implementation**: `/home/nsm/code/deepbrain/goodiebag` - Current sophisticated multi-hook system that Guardy will replace

### Current GoodieBag Implementation Features
- **Branch Protection**: Blocks direct commits to main branch
- **Working Tree Validation**: Ensures no unstaged/untracked files
- **Advanced Secret Detection**: Patterns for API keys, tokens, git-crypt exclusions
- **Git-Crypt Integration**: Validates encrypted file status
- **Code Formatting**: NX-based auto-formatting
- **Conventional Commits**: Enforces commit message standards
- **Lockfile Validation**: Ensures pnpm-lock.yaml sync
- **Professional UI**: Styled output similar to lint-staged

## Project Compatibility

**Universal Git Hook System** - Guardy works with any project structure:

### Supported Project Types
- **NX Monorepos**: Native support for `nx affected` commands
- **Node.js Projects**: npm, pnpm, yarn, bun compatibility
- **Rust Projects**: cargo integration
- **Python Projects**: pip, poetry, ruff support
- **Any Git Repository**: Language-agnostic git hooks

### Package Manager Detection
- **Auto-Detection**: Reads lockfiles (pnpm-lock.yaml, package-lock.json, yarn.lock)
- **Configurable**: Override detection with explicit configuration
- **Multi-Manager**: Support for mixed environments

## Installation Model

### Global Binary + Per-Repository Configuration
**Pattern**: Similar to NX, Husky, Prettier - global tool with local config

```bash
# 1. Global Installation (once per system)
cargo install guardy
brew install guardy
pnpm add -g guardy

# 2. Per-Repository Initialization
cd my-project/
guardy init                    # Creates .guardy.yml + installs git hooks

# 3. Repository-Specific Configuration
vim .guardy.yml               # Customize security rules per repo

# 4. Automatic Hook Execution
git commit                    # Triggers guardy pre-commit hook
```

### Benefits
- **Single Binary**: One global installation, no per-project overhead
- **Flexible Configuration**: Each repo can have different security levels
- **Team Consistency**: Share `.guardy.yml` via git for team standards
- **Easy Updates**: Update global binary, all repos benefit instantly

## Tool Integration

### Intelligent Auto-Detection System
**Smart Project Detection**: Automatically detects project type and configures appropriate tools

```rust
// Auto-detection logic
fn detect_project_type() -> ProjectType {
    if exists("nx.json") { 
        ProjectType::NxMonorepo 
    } else if exists("package.json") { 
        ProjectType::NodeJs 
    } else if exists("Cargo.toml") { 
        ProjectType::Rust 
    } else if exists("pyproject.toml") || exists("requirements.txt") { 
        ProjectType::Python 
    } else if exists("go.mod") { 
        ProjectType::Go 
    } else { 
        ProjectType::Generic 
    }
}
```

### Configuration Flexibility
```yaml
# .guardy.yml - Three levels of configuration
hooks:
  pre-commit:
    checks:
      code-formatting:
        enabled: true
        
        # Level 1: Auto-detection (default)
        auto-detect: true
        
        # Level 2: Manual override (single command)
        command: "my-custom-formatter"
        
        # Level 3: Multi-tool support (advanced)
        tools:
          - name: "prettier"
            command: "prettier --write ."
            files: ["*.js", "*.ts", "*.jsx", "*.tsx"]
          - name: "cargo-fmt"
            command: "cargo fmt"
            files: ["*.rs"]
          - name: "ruff"
            command: "ruff format ."
            files: ["*.py"]
```

### Auto-Detection Examples
```yaml
# Example 1: NX Monorepo (auto-detected)
# No configuration needed - detects nx.json and uses:
# command: "nx format:write --uncommitted"

# Example 2: Mixed project with override
hooks:
  pre-commit:
    checks:
      code-formatting:
        enabled: true
        auto-detect: false  # Disable auto-detection
        command: "biome format --write ."  # Use Biome instead of Prettier

# Example 3: Multi-language project
hooks:
  pre-commit:
    checks:
      code-formatting:
        enabled: true
        tools:
          - name: "prettier"
            command: "prettier --write ."
            files: ["*.js", "*.ts", "*.json"]
          - name: "rustfmt"
            command: "cargo fmt"
            files: ["*.rs"]
          - name: "black"
            command: "black ."
            files: ["*.py"]
```

### Supported Development Tools
- **JavaScript/TypeScript**: Prettier, ESLint, Biome, Deno fmt
- **Rust**: cargo fmt, cargo clippy, rustfmt
- **Python**: ruff, black, pylint, isort
- **Go**: gofmt, golint, goimports
- **C/C++**: clang-format
- **Java**: google-java-format
- **Any Language**: Custom commands via configuration

## Technology Stack

- **Language**: Rust (2024 edition)
- **CLI Framework**: clap v4 with derive features
- **Configuration**: YAML with serde_yaml
- **Git Integration**: git2 crate
- **Progress/UI**: indicatif for progress bars
- **Async Runtime**: tokio for parallel operations
- **Error Handling**: anyhow for ergonomic error management
- **Package Manager**: pnpm (preferred over npm for stability and performance)
- **Website**: Cloudflare Pages for guardy.dev

## Testing Strategy

### Rust Testing Framework
```toml
# Cargo.toml dependencies
[dev-dependencies]
tokio-test = "0.4"          # Async testing utilities
tempfile = "3.0"            # Temporary file/directory creation
assert_cmd = "2.0"          # CLI testing framework
predicates = "3.0"          # Assertion predicates
serial_test = "3.0"         # Serial test execution
git2 = "0.18"              # Git repository manipulation
regex = "1.10"             # Pattern matching tests
```

### Test Structure
```
tests/
├── integration/
│   ├── cli_tests.rs           # CLI command testing
│   ├── init_tests.rs          # guardy init functionality
│   ├── hook_execution_tests.rs # Git hook execution
│   ├── config_validation_tests.rs # Configuration validation
│   └── security_tests.rs      # Secret detection tests
├── fixtures/
│   ├── sample_repos/          # Test git repositories
│   ├── sample_configs/        # Test .guardy.yml files
│   └── sample_secrets/        # Test files with secrets
└── unit/
    ├── config_tests.rs        # Configuration parsing
    ├── git_tests.rs           # Git operations
    ├── security_tests.rs      # Security pattern matching
    └── output_tests.rs        # UI output formatting
```

### Testing Approach
- **Unit Tests**: Each module with `#[cfg(test)]` for isolated component testing
- **Integration Tests**: CLI behavior testing with real git repositories
- **Property-Based Testing**: Security pattern validation with random inputs
- **Performance Tests**: Benchmark secret scanning and git operations
- **Cross-Platform Tests**: Windows, macOS, Linux compatibility

## MCP-First Architecture

### Primary Installation: MCP Server with CLI
**Revolutionary AI Integration**: Guardy is designed as an MCP-first tool where AI assistants are the primary interface, with CLI as a secondary option.

### Installation & Setup Flow
```bash
# 1. Single binary installation
cargo install guardy

# 2. Interactive setup wizard
guardy mcp setup
# - Detects AI assistants (Claude, VS Code, Cursor)
# - Registers MCP server automatically
# - Starts daemon mode
# - Analyzes current project (if in project directory)
# - Provides AI-assisted configuration
```

### CLI Architecture with Subcommands
```rust
// Organized subcommand structure
guardy mcp setup     # MCP setup wizard + registration
guardy mcp start     # Start MCP daemon
guardy mcp stop      # Stop MCP daemon
guardy mcp status    # Check MCP daemon status
guardy mcp logs      # View MCP daemon logs

guardy hooks install # Install git hooks
guardy hooks run pre-commit    # Run specific hook
guardy hooks list    # List available hooks
guardy hooks remove  # Remove git hooks

guardy config init   # Initialize .guardy.yml
guardy config validate # Validate configuration
guardy config show   # Show current config

guardy init          # Quick setup (hooks + config)
guardy status        # Overall status
```

### MCP Server Features (Built-in Rust)
```rust
// Built into guardy binary - no separate installation needed
impl GuardyMCPServer {
    // Real-time project analysis
    async fn analyze_project(&self, path: &str) -> Result<ProjectAnalysis>
    
    // Interactive configuration generation
    async fn generate_config(&self, project_type: ProjectType, preferences: UserPreferences) -> Result<GuardyConfig>
    
    // Live validation
    async fn validate_config(&self, config: &GuardyConfig) -> Result<ValidationResult>
    
    // Tool detection and recommendations
    async fn detect_tools(&self, path: &str) -> Result<ToolRecommendations>
    
    // Apply configuration and install hooks
    async fn apply_config(&self, config: &GuardyConfig, path: &str) -> Result<ApplyResult>
    
    // Installation wizard
    async fn setup_wizard(&self) -> Result<SetupResult>
}
```

### MCP Tools Available to AI
- **`analyze_project`**: Analyze current repository structure and recommend configuration
- **`generate_config`**: Generate `.guardy.yml` based on project type and preferences
- **`validate_config`**: Validate configuration before applying
- **`detect_tools`**: Detect available formatters, linters, and tools
- **`apply_config`**: Apply configuration and install git hooks
- **`troubleshoot`**: Diagnose and fix configuration issues

### AI Workflow Example
```bash
# 1. AI Assistant connects to Guardy MCP server
# 2. AI can now do:

# Analyze current project
> analyze_project("/path/to/project")
Returns: {
  "project_type": "nx_monorepo",
  "languages": ["typescript", "rust"],
  "tools_detected": ["prettier", "eslint", "cargo"],
  "recommendations": {
    "security_level": "strict",
    "formatters": ["nx format:write --uncommitted"],
    "additional_checks": ["lockfile-validation"]
  }
}

# Generate configuration
> generate_config(project_type="nx_monorepo", security_level="strict")
Returns: Complete .guardy.yml configuration

# Validate and apply
> validate_config(config)
> apply_config(config, "/path/to/project")
```

### Installation & AI Integration
```bash
# 1. Install guardy binary
cargo install guardy

# 2. Setup MCP integration (interactive wizard)
guardy mcp setup
# - Detects AI assistants (Claude Desktop, VS Code, Cursor)
# - Registers MCP server automatically
# - Starts daemon mode on port 3001
# - Ready for AI-assisted configuration

# 3. AI assistants can now connect to:
# - Server: guardy daemon
# - Port: 3001 (configurable)
# - Tools: analyze_project, generate_config, validate_config, apply_config, setup_wizard
```

### Benefits Over Static Documentation
- **Real-time Analysis**: Understands current project state
- **Interactive Configuration**: Step-by-step setup with validation
- **Live Validation**: Test configurations before applying
- **Tool Detection**: Automatically finds available tools
- **Personalized Recommendations**: Based on actual project structure
- **Error Prevention**: Validates configurations before application

### Built-in MCP Server Architecture
```rust
// src/mcp/ - Built into guardy binary
├── mod.rs                     # MCP server module
├── server.rs                  # MCP server implementation
├── daemon.rs                  # Background daemon mode
├── tools/
│   ├── analyze.rs             # Project analysis
│   ├── generate.rs            # Configuration generation
│   ├── validate.rs            # Configuration validation
│   ├── detect.rs              # Tool detection
│   ├── apply.rs               # Configuration application
│   └── setup.rs               # Installation wizard
├── types/
│   ├── project.rs             # Project type definitions
│   ├── config.rs              # Configuration types
│   └── tools.rs               # Tool definitions
└── utils/
    ├── filesystem.rs          # File system operations
    ├── git.rs                 # Git operations
    └── validation.rs          # Validation utilities
```

### User Experience Flow
```bash
# 1. Install & Setup
cargo install guardy
guardy mcp setup

# 2. AI-Assisted Configuration
# User: "Configure guardy for my NX monorepo with strict security"
# AI connects to MCP server, analyzes project, generates config, applies it

# 3. Traditional CLI (optional)
guardy init
guardy run pre-commit

# 4. Daemon Management
guardy mcp status    # Check daemon status
guardy mcp stop      # Stop daemon
guardy mcp start     # Start daemon
guardy mcp logs      # View daemon logs
```

## Project Structure

**Architecture Decision**: Single Rust crate for simplicity and focused distribution

```
guardy/
├── Cargo.toml                 # Project configuration
├── README.md                  # Project documentation
├── LICENSE                    # MIT license (public domain)
├── .github/
│   ├── workflows/
│   │   ├── ci.yml            # Test, lint, format
│   │   ├── release.yml       # Cross-compile and release
│   │   └── security.yml      # Security audits
│   └── FUNDING.yml           # GitHub sponsors
├── src/
│   ├── main.rs               # CLI entry point
│   ├── lib.rs                # Library exports
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── init.rs           # Initialize command
│   │   ├── run.rs            # Run hook command
│   │   ├── config.rs         # Config commands
│   │   ├── setup.rs          # MCP setup wizard
│   │   ├── daemon.rs         # Daemon management
│   │   └── status.rs         # Status commands
│   ├── mcp/                  # Built-in MCP server
│   │   ├── mod.rs
│   │   ├── server.rs         # MCP server implementation
│   │   ├── daemon.rs         # Background daemon mode
│   │   ├── tools/
│   │   │   ├── analyze.rs    # Project analysis
│   │   │   ├── generate.rs   # Configuration generation
│   │   │   ├── validate.rs   # Configuration validation
│   │   │   ├── detect.rs     # Tool detection
│   │   │   ├── apply.rs      # Configuration application
│   │   │   └── setup.rs      # Installation wizard
│   │   ├── types/
│   │   │   ├── project.rs    # Project type definitions
│   │   │   ├── config.rs     # Configuration types
│   │   │   └── tools.rs      # Tool definitions
│   │   └── utils/
│   │       ├── filesystem.rs # File system operations
│   │       ├── git.rs        # Git operations
│   │       └── validation.rs # Validation utilities
│   ├── config/
│   │   ├── mod.rs
│   │   ├── types.rs          # Configuration types
│   │   ├── loader.rs         # Config file loading
│   │   └── validator.rs      # Config validation
│   ├── hooks/
│   │   ├── mod.rs
│   │   ├── pre_commit.rs     # Pre-commit security checks
│   │   ├── commit_msg.rs     # Commit message validation
│   │   ├── pre_push.rs       # Pre-push validation
│   │   └── common.rs         # Shared hook utilities
│   ├── security/
│   │   ├── mod.rs
│   │   ├── secrets.rs        # Secret detection
│   │   ├── git_crypt.rs      # Git-crypt integration
│   │   └── patterns.rs       # Security pattern matching
│   ├── git/
│   │   ├── mod.rs
│   │   ├── operations.rs     # Git operations
│   │   ├── staging.rs        # Staging area checks
│   │   └── branch.rs         # Branch protection
│   └── utils/
│       ├── mod.rs
│       ├── output.rs         # Styled output/symbols
│       ├── parallel.rs       # Parallel execution
│       └── fs.rs             # File system utilities
├── templates/
│   ├── .guardy.yml           # Default configuration
│   └── hooks/
│       ├── pre-commit        # Generated hook scripts
│       ├── commit-msg
│       └── pre-push
├── tests/
│   ├── integration/
│   │   ├── init_test.rs
│   │   ├── hooks_test.rs
│   │   └── config_test.rs
│   └── fixtures/
│       ├── repos/            # Test git repositories
│       └── configs/          # Test configurations
├── website/                   # Cloudflare Pages website
│   ├── index.html
│   ├── docs/
│   │   ├── installation.md
│   │   ├── configuration.md
│   │   ├── security-checks.md
│   │   ├── examples/
│   │   └── ai-docs/          # AI-friendly documentation
│   │       ├── guardy-schema.json
│   │       ├── configuration-guide.md
│   │       ├── examples/
│   │       │   ├── javascript-project.yml
│   │       │   ├── rust-project.yml
│   │       │   ├── python-project.yml
│   │       │   ├── nx-monorepo.yml
│   │       │   └── multi-language.yml
│   │       ├── patterns/
│   │       │   ├── security-patterns.md
│   │       │   ├── tool-integrations.md
│   │       │   └── troubleshooting.md
│   │       └── ai-prompts/
│   │           ├── configuration-prompt.md
│   │           ├── security-setup-prompt.md
│   │           └── tool-integration-prompt.md
│   └── install.sh            # Installation script
├── npm-wrapper/               # pnpm-compatible npm wrapper
│   ├── package.json
│   ├── index.js
│   └── bin/
│       └── guardy
```

## Configuration Schema

### `.guardy.yml`

```yaml
# Security-first git hooks configuration
guardy:
  version: "1.0"
  
  # Global settings
  global:
    debug: false
    colors: true
    parallel: true
    timeout: 30 # seconds
    
  # Hook-specific configuration
  hooks:
    pre-commit:
      enabled: true
      checks:
        branch-protection:
          enabled: true
          protected-branches: ["main", "master", "production"]
          
        staging-validation:
          enabled: true
          require-staged: true
          allow-untracked: false
          
        secret-detection:
          enabled: true
          patterns:
            # OpenAI API keys
            - pattern: "sk-[a-zA-Z0-9]{20,}"
              name: "OpenAI API Key"
              severity: "high"
            # GitHub Personal Access Tokens
            - pattern: "ghp_[a-zA-Z0-9]{20,}"
              name: "GitHub PAT"
              severity: "high"
            # AWS Access Keys
            - pattern: "AKIA[0-9A-Z]{16}"
              name: "AWS Access Key"
              severity: "critical"
            # JWT tokens
            - pattern: "ey[a-zA-Z0-9]{20,}"
              name: "JWT Token"
              severity: "medium"
            # Generic base64 secrets (32+ chars)
            - pattern: "['\"][a-zA-Z0-9+/]{32,}[=]*['\"]"
              name: "Base64 Secret"
              severity: "medium"
          exclude-files: 
            - "*.test.*"
            - "*.spec.*"
            - "test/**/*"
            - "tests/**/*"
          exclude-patterns:
            - "example.*"
            - "demo.*"
            
        git-crypt:
          enabled: true
          require-encryption: true
          check-gitattributes: true
          
        code-formatting:
          enabled: true
          command: "nx format:write --uncommitted"  # NX monorepos
          # command: "pnpm run format"              # Standard projects
          # command: "npm run format"               # npm projects
          # command: "yarn format"                  # Yarn projects
          auto-fix: true
          
    commit-msg:
      enabled: true
      conventional-commits:
        enabled: true
        types: 
          - "feat"      # New feature
          - "fix"       # Bug fix
          - "docs"      # Documentation
          - "style"     # Code style changes
          - "refactor"  # Code refactoring
          - "test"      # Adding tests
          - "chore"     # Maintenance
          - "perf"      # Performance improvements
          - "ci"        # CI/CD changes
          - "build"     # Build system changes
          - "revert"    # Reverting changes
        scopes: 
          - "api"
          - "ui" 
          - "auth"
          - "db"
          - "ci"
        max-length: 50
        require-description: true
        
    pre-push:
      enabled: true
      checks:
        lockfile-validation:
          enabled: true
          package-managers: ["pnpm", "npm", "yarn", "bun"]
          
        test-execution:
          enabled: false
          command: "pnpm test"     # Auto-detects: pnpm, npm, yarn
          timeout: 300 # 5 minutes
          
        lint-checks:
          enabled: false
          command: "nx affected --target=lint"  # NX monorepos
          # command: "pnpm run lint"            # Standard projects
```

## Core Components

### 1. CLI Interface (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "guardy")]
#[command(about = "Intelligent git workflows")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, global = true)]
    config: Option<String>,
    
    #[arg(long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize guardy in repository
    Init {
        #[arg(long)]
        security_level: Option<SecurityLevel>,
        
        #[arg(long)]
        force: bool,
    },
    
    /// Run specific hook
    Run {
        hook: HookType,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Validate configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Install git hooks
    Install {
        #[arg(long)]
        force: bool,
    },
    
    /// Uninstall git hooks
    Uninstall,
}

#[derive(clap::ValueEnum, Clone)]
enum SecurityLevel {
    Basic,
    Standard,
    Strict,
}

#[derive(clap::ValueEnum, Clone)]
enum HookType {
    PreCommit,
    CommitMsg,
    PrePush,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Validate configuration file
    Validate,
    /// Show current configuration
    Show,
    /// Initialize default configuration
    Init,
}
```

### 2. Configuration System (`src/config/`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GuardyConfig {
    #[serde(rename = "guardy")]
    pub guardy: GuardyHooks,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuardyHooks {
    pub version: String,
    pub global: GlobalConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub debug: bool,
    #[serde(default = "default_colors")]
    pub colors: bool,
    #[serde(default = "default_parallel")]
    pub parallel: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HooksConfig {
    #[serde(rename = "pre-commit")]
    pub pre_commit: Option<PreCommitConfig>,
    #[serde(rename = "commit-msg")]
    pub commit_msg: Option<CommitMsgConfig>,
    #[serde(rename = "pre-push")]
    pub pre_push: Option<PrePushConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreCommitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub checks: PreCommitChecks,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreCommitChecks {
    #[serde(rename = "branch-protection")]
    pub branch_protection: Option<BranchProtectionConfig>,
    #[serde(rename = "staging-validation")]
    pub staging_validation: Option<StagingValidationConfig>,
    #[serde(rename = "secret-detection")]
    pub secret_detection: Option<SecretDetectionConfig>,
    #[serde(rename = "git-crypt")]
    pub git_crypt: Option<GitCryptConfig>,
    #[serde(rename = "code-formatting")]
    pub code_formatting: Option<CodeFormattingConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecretDetectionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub patterns: Vec<SecretPattern>,
    #[serde(rename = "exclude-files", default)]
    pub exclude_files: Vec<String>,
    #[serde(rename = "exclude-patterns", default)]
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecretPattern {
    pub pattern: String,
    pub name: String,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

### 3. Security Checks (`src/security/`)

```rust
use regex::Regex;
use anyhow::{Result, Context};
use git2::Repository;

pub struct SecretScanner {
    patterns: Vec<CompiledPattern>,
    exclude_files: Vec<Regex>,
    exclude_patterns: Vec<Regex>,
}

#[derive(Debug)]
pub struct CompiledPattern {
    pub regex: Regex,
    pub name: String,
    pub severity: SecuritySeverity,
}

#[derive(Debug)]
pub struct SecretMatch {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub pattern_name: String,
    pub severity: SecuritySeverity,
    pub snippet: String,
}

impl SecretScanner {
    pub fn new(config: &SecretDetectionConfig) -> Result<Self> {
        let patterns = config.patterns
            .iter()
            .map(|p| {
                let regex = Regex::new(&p.pattern)
                    .with_context(|| format!("Invalid regex pattern: {}", p.pattern))?;
                Ok(CompiledPattern {
                    regex,
                    name: p.name.clone(),
                    severity: p.severity.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
            
        let exclude_files = config.exclude_files
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid exclude file pattern")?;
            
        let exclude_patterns = config.exclude_patterns
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid exclude pattern")?;
            
        Ok(Self {
            patterns,
            exclude_files,
            exclude_patterns,
        })
    }
    
    pub async fn scan_staged_files(&self, repo: &Repository) -> Result<Vec<SecretMatch>> {
        let staged_files = crate::git::operations::get_staged_files(repo)?;
        let mut matches = Vec::new();
        
        for file_path in staged_files {
            if self.should_exclude_file(&file_path) {
                continue;
            }
            
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read file: {}", file_path))?;
                
            let file_matches = self.scan_content(&file_path, &content)?;
            matches.extend(file_matches);
        }
        
        Ok(matches)
    }
    
    fn scan_content(&self, file_path: &str, content: &str) -> Result<Vec<SecretMatch>> {
        let mut matches = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            // Skip if line matches exclude patterns
            if self.exclude_patterns.iter().any(|regex| regex.is_match(line)) {
                continue;
            }
            
            for pattern in &self.patterns {
                for mat in pattern.regex.find_iter(line) {
                    matches.push(SecretMatch {
                        file: file_path.to_string(),
                        line: line_num + 1,
                        column: mat.start() + 1,
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                        snippet: line.to_string(),
                    });
                }
            }
        }
        
        Ok(matches)
    }
    
    fn should_exclude_file(&self, file_path: &str) -> bool {
        self.exclude_files.iter().any(|regex| regex.is_match(file_path))
    }
}
```

### 4. Git Integration (`src/git/`)

```rust
use git2::{Repository, StatusFlags, StatusOptions};
use anyhow::{Result, Context};

pub struct GitOperations {
    repo: Repository,
}

impl GitOperations {
    pub fn new() -> Result<Self> {
        let repo = Repository::open_from_env()
            .context("Not in a git repository")?;
        Ok(Self { repo })
    }
    
    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;
            
        let branch_name = head.shorthand()
            .context("Failed to get branch name")?;
            
        Ok(branch_name.to_string())
    }
    
    pub fn get_staged_files(&self) -> Result<Vec<String>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);
        
        let statuses = self.repo.statuses(Some(&mut opts))
            .context("Failed to get git status")?;
            
        let staged_files: Vec<String> = statuses
            .iter()
            .filter_map(|entry| {
                let flags = entry.status();
                if flags.contains(StatusFlags::INDEX_NEW) ||
                   flags.contains(StatusFlags::INDEX_MODIFIED) ||
                   flags.contains(StatusFlags::INDEX_DELETED) ||
                   flags.contains(StatusFlags::INDEX_RENAMED) ||
                   flags.contains(StatusFlags::INDEX_TYPECHANGE) {
                    entry.path().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
            
        Ok(staged_files)
    }
    
    pub fn has_unstaged_changes(&self) -> Result<bool> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut opts))
            .context("Failed to get git status")?;
            
        for entry in statuses.iter() {
            let flags = entry.status();
            if flags.contains(StatusFlags::WT_MODIFIED) ||
               flags.contains(StatusFlags::WT_DELETED) ||
               flags.contains(StatusFlags::WT_NEW) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

### 5. Output/UI (`src/utils/output.rs`)

```rust
use console::{style, Style};
use indicatif::{ProgressBar, ProgressStyle};

pub struct Output {
    colors_enabled: bool,
}

impl Output {
    pub fn new(colors_enabled: bool) -> Self {
        Self { colors_enabled }
    }
    
    pub fn info(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", style("ℹ").cyan(), message);
        } else {
            println!("ℹ {}", message);
        }
    }
    
    pub fn success(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", style("✔").green(), message);
        } else {
            println!("✔ {}", message);
        }
    }
    
    pub fn error(&self, message: &str) {
        if self.colors_enabled {
            eprintln!("{} {}", style("✖").red(), message);
        } else {
            eprintln!("✖ {}", message);
        }
    }
    
    pub fn warning(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", style("⚠").yellow(), message);
        } else {
            println!("⚠ {}", message);
        }
    }
    
    pub fn pointer(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", style("❯").yellow(), message);
        } else {
            println!("❯ {}", message);
        }
    }
    
    pub fn create_progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        pb.set_message(message.to_string());
        pb
    }
}
```

## Installation Methods

### 1. Cargo Install (Primary)
```bash
cargo install guardy
```

### 2. Binary Downloads
```bash
# Universal installer (Linux/macOS/Windows)
curl -fsSL https://install.guardy.dev | sh

# Direct download from GitHub releases
# Available at: https://github.com/deepbrainspace/guardy/releases
```

### 3. Package Managers
```bash
# Homebrew (macOS + Linux)
brew install deepbrainspace/tap/guardy

# Windows (Scoop)
scoop bucket add deepbrainspace https://github.com/deepbrainspace/scoop-bucket
scoop install guardy

# Arch Linux (AUR)
yay -S guardy-bin

# Tea.xyz (Blockchain-based)
tea install guardy
```

### 4. Node.js Projects (pnpm wrapper)
```bash
# For Node.js projects preferring package.json
pnpm add -D @deepbrainspace/guardy
npx guardy init

# Also compatible with npm/yarn
npm install -D @deepbrainspace/guardy
yarn add -D @deepbrainspace/guardy
```

## Usage

### Initialize in Repository
```bash
# Basic initialization
guardy init

# With security level
guardy init --security-level=strict

# Force overwrite existing hooks
guardy init --force
```

### Manual Hook Execution
```bash
# Run pre-commit hook
guardy run pre-commit

# Run commit-msg hook with message file
guardy run commit-msg .git/COMMIT_EDITMSG

# Run pre-push hook
guardy run pre-push
```

### Configuration Management
```bash
# Validate configuration
guardy config validate

# Show current configuration
guardy config show

# Create default configuration
guardy config init
```

## Development Workflow

### 1. Setup Development Environment
```bash
# Clone repository
git clone https://github.com/deepbrainspace/guardy.git
cd guardy

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --help
```

### 2. Testing Strategy
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# Test with different configurations
cargo test --test config_validation

# Test hook execution
cargo test --test hook_execution

# Security pattern tests
cargo test --test security_patterns
```

### 3. Cross-Platform Builds
```bash
# Install cross-compilation targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for all platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Security Features

### 1. Secret Detection Patterns
- **OpenAI API Keys**: `sk-[a-zA-Z0-9]{20,}`
- **GitHub Personal Access Tokens**: `ghp_[a-zA-Z0-9]{20,}`
- **AWS Access Keys**: `AKIA[0-9A-Z]{16}`
- **JWT Tokens**: `ey[a-zA-Z0-9]{20,}`
- **Generic Base64 Secrets**: `['\"][a-zA-Z0-9+/]{32,}[=]*['\"]`
- **Custom Patterns**: User-configurable via YAML

### 2. Git-Crypt Integration
- Validates encrypted file status
- Checks `.gitattributes` configuration
- Prevents unencrypted commits of sensitive files
- Supports custom encryption rules

### 3. Branch Protection
- Prevents direct commits to protected branches
- Configurable branch patterns
- Supports multiple protection levels

### 4. Staging Validation
- Ensures clean working tree
- Detects unstaged changes
- Handles untracked files
- Validates commit completeness

## Performance Targets

- **Cold Start**: < 50ms
- **Hot Path**: < 10ms for cached operations
- **Memory Usage**: < 10MB peak
- **Binary Size**: < 5MB compressed
- **Secret Scanning**: > 1000 files/second

## Distribution Strategy

### Multi-Channel Approach
```yaml
distribution:
  primary:
    - cargo install guardy
    - GitHub releases (cross-platform binaries)
    
  package_managers:
    - homebrew: macOS + Linux (official support)
    - scoop: Windows users
    - tea.xyz: blockchain rewards + decentralized distribution
    - aur: Arch Linux (guardy-bin)
    - pnpm: @deepbrainspace/guardy (Node.js wrapper)
    
  website:
    - guardy.dev (Cloudflare Pages)
    - Installation scripts
    - Documentation and examples
```

### Tea.xyz Integration
- **Automatic Rewards**: Developers get compensated via "Proof of Contribution"
- **Decentralized Storage**: Packages stored in `~/.tea` (similar to `~/.cargo`)
- **Virtual Environments**: Isolated per-project environments
- **Cross-Platform**: Unified package manager experience

## Business Model

### Community-First Approach
```yaml
business_model:
  core_product:
    - Open source (MIT License)
    - Complete git hook functionality
    - All security features included
    - Community support via GitHub
    
  revenue_streams:
    - Tea.xyz blockchain rewards (automatic)
    - GitHub Sponsors (individual support)
    - Future: Enterprise features (when needed)
    
  website_monetization:
    - guardy.dev hosted on Cloudflare Pages
    - Install scripts and documentation
    - Community showcase
```

## Release Strategy

### 1. GitHub Actions CI/CD
- **Testing**: All platforms, all Rust versions
- **Security**: Cargo audit, dependency scanning
- **Building**: Cross-compilation for all targets
- **Publishing**: Automated crate and binary releases

### 2. Versioning
- **Semantic Versioning**: Major.Minor.Patch
- **Git Tags**: Automated from GitHub releases
- **Changelog**: Auto-generated from conventional commits

### 3. Distribution Channels
- **Crates.io**: Primary Rust distribution
- **GitHub Releases**: Binary downloads
- **Package Managers**: Homebrew, Scoop, AUR, Tea.xyz
- **Install Scripts**: Cross-platform installers
- **Website**: Cloudflare Pages for guardy.dev

## Implementation Checklist

### Phase 1: Core Development (Priority 1)
- [ ] **Project Setup**
  - [ ] Initialize Rust project with Cargo.toml
  - [ ] Setup GitHub repository structure
  - [ ] Configure GitHub Actions CI/CD
  - [ ] Setup basic CLI with clap framework

- [ ] **Core Features**
  - [ ] Implement YAML configuration system
  - [ ] Build git integration layer (git2 crate)
  - [ ] Create professional output system (styled like lint-staged)
  - [ ] Add secret detection with regex patterns
  - [ ] Implement git-crypt integration
  - [ ] Build branch protection checks
  - [ ] Add staging validation
  - [ ] Create conventional commit validation
  - [ ] Tool integration system (prettier, cargo fmt, etc.)
  - [ ] Package manager auto-detection
  - [ ] AI-friendly documentation system for LLM-assisted configuration

- [ ] **Hook Implementation**
  - [ ] Pre-commit hook (security, formatting, validation)
  - [ ] Commit-msg hook (conventional commits)
  - [ ] Pre-push hook (lockfile validation)

### Phase 2: Distribution (Priority 2)
- [ ] **Release Automation**
  - [ ] Cross-platform binary builds
  - [ ] Automated cargo publish
  - [ ] GitHub releases with binaries
  - [ ] Version management system

- [ ] **Package Managers**
  - [ ] Homebrew formula creation
  - [ ] Scoop manifest setup
  - [ ] AUR package (guardy-bin)
  - [ ] Tea.xyz integration
  - [ ] pnpm wrapper package
  - [ ] Guardy MCP server for AI integration

- [ ] **Website & Documentation**
  - [ ] Setup Cloudflare Pages for guardy.dev
  - [ ] Create installation scripts
  - [ ] Write comprehensive documentation
  - [ ] Add configuration examples
  - [ ] Implement MCP server for AI assistant integration

### Phase 3: Community & Growth (Priority 3)
- [ ] **Community Features**
  - [ ] GitHub Sponsors setup
  - [ ] Community guidelines
  - [ ] Issue templates
  - [ ] Contributing guidelines

- [ ] **Advanced Features**
  - [ ] Plugin system architecture
  - [ ] Custom security pattern support
  - [ ] Performance optimizations
  - [ ] Extended configuration options

## Future Enhancements

### Phase 4: Advanced Features (Future)
- **Plugin System**: Custom security checks via WASM
- **IDE Integration**: VS Code extension, JetBrains plugin
- **Dashboard**: Web UI for team security metrics
- **Advanced Security**: ML-based pattern detection
- **Enterprise Features**: Team collaboration, audit logging (when needed)

## Contributing

1. **Fork the repository**
2. **Create feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes with tests**
4. **Run test suite**: `cargo test`
5. **Submit pull request**

## License

MIT License - see LICENSE file for details.

## Sponsorship Model

**Personal Sponsorship for Organization Project**: While Guardy is part of the DeepBrain.space organization, individual sponsorship provides direct support to the maintainer and ensures sustainable development.

### How It Works:
- **Repository**: `github.com/deepbrainspace/guardy` (org ownership)
- **Sponsorship**: `github.com/sponsors/wizardsupreme` (personal maintainer)
- **Benefits**: 
  - Direct maintainer support ensures rapid issue resolution
  - Personal investment in project success
  - Flexible sponsorship tiers for individuals and companies
  - Priority support for sponsors

### Sponsorship Tiers:
- **Individual Developer**: $5/month - Priority issue responses
- **Small Team**: $25/month - Feature requests consideration
- **Enterprise**: $100/month - Direct support channel, custom integrations

### Support Channels:
- **GitHub Sponsors**: https://github.com/sponsors/wizardsupreme
- **Buy Me a Coffee**: https://buymeacoffee.com/wizardsupreme
- **Corporate Sponsorship**: Contact for custom arrangements

Your support helps maintain and improve this open-source developer workflow tool for the entire community while ensuring dedicated maintainer availability.