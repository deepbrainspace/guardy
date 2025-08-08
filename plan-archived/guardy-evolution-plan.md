# Guardy Evolution Plan: Lefthook Replacement + Protected Sync

## Executive Summary

Transform Guardy from a partial Husky replacement into a comprehensive **Lefthook replacement with Protected Sync capabilities**. This evolution will provide:

1. **Complete Git Hook Management** - Replace Lefthook as the primary git hooks tool
2. **Protected Sync System** - Automatically sync and protect configuration files from external repositories
3. **Enterprise-Grade File Protection** - Prevent modification of critical build/CI files

## Current State Analysis

### ✅ What's Implemented (Ready to Use)
- **CLI Framework** - Complete Clap-based CLI with professional styling
- **Configuration System** - Advanced SuperConfig-based configuration with smart format detection
- **Security Scanner** - Comprehensive secret detection with 40+ patterns
- **Git Integration** - Basic git operations and repository discovery
- **MCP Server** - Model Context Protocol server integration
- **Install Command** - Hook installation to `.git/hooks/`
- **Scan Command** - File security scanning functionality

### ❌ What's Missing (Core Gaps)
- **Hook Implementations** - All hook modules are TODO stubs **(2-3 hours)**
- **Pre-commit Logic** - Branch protection, staged file validation **(1-2 hours)**
- **Commit-msg Validation** - Conventional commits checking **(0.5-1 hour)**
- **Post-checkout Actions** - Dependency management **(1-2 hours)**
- **Pre-push Validation** - Lockfile sync, test execution **(1-2 hours)**
- **Protected Sync System** - The entire sync mechanism **(4-5 hours)**

**Total Current Implementation Gap: 10-15 hours**

### 📊 Implementation Status
- **Infrastructure**: 85% complete
- **Core Hook Logic**: 15% complete  
- **Protected Sync**: 0% complete

## Architecture Overview

### Phase 1: Complete Lefthook Replacement
```
src/
├── hooks/
│   ├── pre_commit.rs        # Branch protection + security scanning
│   ├── commit_msg.rs        # Conventional commits validation
│   ├── post_checkout.rs     # Dependency installation
│   └── pre_push.rs          # Lockfile sync + tests
├── git/
│   └── operations.rs        # Enhanced git operations
└── config/
    └── hooks_config.rs      # Hook-specific configuration
```

### Phase 2: Protected Sync System
```
src/
├── sync/
│   ├── manager.rs           # Sync orchestration
│   ├── repository.rs        # External repo operations
│   ├── protection.rs        # File protection enforcement
│   └── version.rs           # Version management
└── cli/commands/
    └── sync.rs              # Sync command implementation
```

## Critical Discovery: Lefthook Parity Analysis

**After comprehensive analysis of Lefthook's 6.4k+ star codebase and configuration system, Guardy needs significant enhancements to achieve competitive parity.**

### ❌ **Critical Feature Gaps Identified**

#### **1. Advanced Configuration System** - MISSING ENTIRELY
**Lefthook's flexibility** (what makes it industry standard):
```yaml
# Lefthook's rich configuration model Guardy lacks
pre-commit:
  parallel: true                    # ❌ Missing parallel execution
  jobs:                            # ❌ Missing jobs/groups model  
    - name: frontend-lint
      glob: "*.{js,ts}"             # ❌ Advanced glob patterns
      root: "frontend/"             # ❌ Directory context switching
      env: {NODE_ENV: test}         # ❌ Environment variable injection
      stage_fixed: true            # ❌ Auto-staging of fixes
      skip: [merge, rebase]        # ❌ Conditional execution
      tags: [frontend, linters]    # ❌ Tag-based grouping
      priority: 1                  # ❌ Execution ordering
```

#### **2. File Processing Variables** - CRITICAL GAP
```yaml
# Built-in file variables Guardy completely lacks
commands:
  lint: {run: "eslint {staged_files}"}     # ❌ Missing
  test: {run: "pytest {all_files}"}        # ❌ Missing  
  custom: 
    files: "git diff --name-only HEAD~1"   # ❌ Missing custom file commands
    run: "mypy {files}"                    # ❌ Missing
```

#### **3. Local Configuration Override** - HIGH PRIORITY
```toml
# guardy-local.toml (missing entirely)
[sync]
enabled = false  # Disable sync for local development

[hooks.pre-commit]
skip_commands = ["slow-linter", "security-scan"]  # Skip slow checks locally
```

### ✅ **Guardy's Unique Advantages** (Differentiators)
- **Security-First**: Built-in secret detection (vs Lefthook requiring external tools)
- **Protected Sync**: File protection system (completely unique)
- **Modern Architecture**: Rust performance + SuperConfig system
- **MCP Integration**: AI tooling capabilities

### **Revised Implementation Strategy**

## Revised Implementation Plan (MVP + Future Features)

### **Configuration Format Decision** ✅

#### **YAML as Primary Format (Recommended)**
**YAML is optimal for Guardy's use case:**
- ✅ **Migration-friendly**: Lefthook users already familiar with YAML
- ✅ **More concise**: Better for complex nested structures (jobs, repos arrays)  
- ✅ **DevOps standard**: CI/CD, Docker, Kubernetes all use YAML
- ✅ **SuperConfig support**: Full YAML support with smart format detection

```yaml
# guardy.yml - Cleaner for complex structures
hooks:
  pre-commit:
    parallel: true
    jobs:
      - name: rust-format
        run: cargo fmt {staged_files}
        glob: "*.rs"
        stage_fixed: true
    scripts:
      - path: .guardy/check-secrets.sh
        runner: bash

sync:
  repos:
    - name: build-toolkit
      repo: org/build-toolkit
      version: v1.2.3
```

#### **TOML as Alternative** 
**Support both formats** via SuperConfig's smart format detection:
- `guardy.yml` / `guardy.yaml` (primary)
- `guardy.toml` (alternative for Rust ecosystem preference)

### **SuperConfig Array Support Confirmation** ✅
**Both YAML arrays and TOML `[[]]` syntax fully supported** by SuperConfig's Figment foundation.

### **MVP Strategy: Essential Lefthook Features Only**

## Phase 1: Essential Lefthook Replacement (6-8 AI hours)

### ✅ **Essential Features (MVP Implementation)**
1. **Parallel Execution**: `parallel: true` - Critical for CI/CD performance
2. **File Variables**: `{staged_files}`, `{all_files}`, `{push_files}` - Core hook functionality  
3. **Basic Job Configuration**: `run`, `glob`, `env` - Essential filtering and execution
4. **Stage Fixed**: `stage_fixed: true` - Auto-stage formatted files (essential for formatters)
5. **Custom Script Support**: External script execution - **Lefthook's killer feature**
6. **Local Config Override**: `guardy-local.toml` - Essential for developer productivity

### **MVP Configuration Structure**
```toml
# guardy.toml - Essential features only
[hooks.pre-commit]
parallel = true

# Job-based execution
[[hooks.pre-commit.jobs]]
name = "rust-format" 
run = "cargo fmt {staged_files}"
glob = "*.rs"
stage_fixed = true
env = {RUST_LOG = "warn"}

[[hooks.pre-commit.jobs]]
name = "rust-lint"
run = "cargo clippy {staged_files}"
glob = "*.rs"

# Custom script support - Lefthook's major advantage
[[hooks.pre-commit.scripts]]
path = ".guardy/check-secrets.sh"
runner = "bash"

[[hooks.pre-commit.scripts]]
path = "custom-lint.py"  
runner = "python3"

# Protected sync configuration
[[sync.repos]]
name = "build-toolkit"
repo = "org/build-toolkit"
version = "v1.2.3"
target_path = "."
```

### **MVP Implementation Timeline**

#### Hour 1-2: Core Configuration & Job System
- **Enhanced Configuration** (`src/config/jobs.rs`)
```rust
pub struct JobConfig {
    pub name: String,
    pub run: Option<String>,
    pub glob: Option<String>,
    pub env: HashMap<String, String>,
    pub stage_fixed: bool,
}

pub struct ScriptConfig {
    pub path: PathBuf,
    pub runner: String,  // bash, python3, node, etc.
}

pub struct HookConfig {
    pub parallel: bool,
    pub jobs: Vec<JobConfig>,
    pub scripts: Vec<ScriptConfig>,
}
```

#### Hour 3-4: File Variable System (Essential Only)
- **Built-in Variables** (`src/execution/file_vars.rs`)
```rust
pub enum FileVariable {
    StagedFiles,     // {staged_files}
    AllFiles,        // {all_files} 
    PushFiles,       // {push_files}
    // Skip custom file commands for MVP
}

impl FileVariable {
    pub fn resolve(&self, repo: &GitRepo) -> Result<Vec<PathBuf>>;
}
```

#### Hour 5-6: Parallel Execution & Script Runner
- **Simple Parallel Execution** (`src/execution/orchestrator.rs`)
```rust
pub struct JobOrchestrator {
    parallel: bool,
    jobs: Vec<JobConfig>,
    scripts: Vec<ScriptConfig>,
}

impl JobOrchestrator {
    pub async fn execute(&self) -> Result<ExecutionResult> {
        if self.parallel {
            // tokio::task::spawn for parallel jobs
            self.execute_parallel().await
        } else {
            self.execute_sequential().await
        }
    }
}
```

- **Script Runner System** (`src/execution/runners.rs`)
```rust
pub enum Runner {
    Bash,
    Python3, 
    Node,
    Ruby,
    Custom(String),
}
```

#### Hour 7-8: Local Configuration & Integration
- **Local Config Override** (`src/config/local.rs`)
- `guardy-local.toml` loading and merging
- Hook integration with existing security scanner
- Basic CLI enhancements for script execution

## Phase 2: Protected Sync System (4-5 hours AI time)
*[Unchanged from previous plan]*

## **Deferred Features (Future Implementation)**

### ❌ **Phase 3: Advanced Lefthook Features** (6-8 AI hours estimate)

#### **3A: Advanced Execution Models** (3-4 hours)
- **Piped Execution**: `piped: true` with stdout/stdin chaining
- **Job Groups**: Nested job organization with independent parallel/piped settings  
- **Priority Ordering**: `priority: 1` for fine-grained execution control
- **Custom File Commands**: `files: "git diff --name-only HEAD~1"`

#### **3B: Organization & Control Features** (3-4 hours)  
- **Tag System**: `tags: [frontend, backend]` for command grouping
- **Advanced Skip Logic**: Complex conditional execution with `skip`/`only`
- **Root Directory**: `root: "frontend/"` for execution context switching
- **Interactive Mode**: `interactive: true` for user input commands

### ❌ **Phase 4: Enterprise Features** (4-6 hours estimate)
- **Docker Integration**: `runner: docker run -it --rm {container}`
- **Remote Configs**: Fetch configurations from external repositories
- **Template System**: Reusable configuration templates
- **Advanced Output Control**: Detailed formatting and verbosity options

### **Future Features Summary**
| Feature Category | Implementation Time | Business Priority |
|------------------|-------------------|-------------------|
| **Advanced Execution** | 3-4 hours | Medium |
| **Organization & Control** | 3-4 hours | Medium |  
| **Enterprise Features** | 4-6 hours | Low |
| **Total Deferred** | **10-14 hours** | *Post-MVP* |

### **Migration Support** (2 hours)
- **Configuration Migration** (`src/cli/commands/migrate.rs`) 
```bash
guardy migrate lefthook.yml
# Converts lefthook.yml to guardy.toml with feature mapping
```
- **Migration validation** and **compatibility testing**

### Phase 2: Protected Sync System (4-5 hours AI time)

### Phase 2: Protected Sync System (4-5 hours AI time)

#### Hour 1: Sync Configuration & Core Types
- **Configuration extension** (`src/config/sync_config.rs`)
```toml
[[sync.repos]]
name = "build-toolkit"
repo = "org/build-toolkit"
version = "v1.2.3"
target_path = "."
include_patterns = ["*.yml", "*.toml"]
exclude_patterns = ["secrets/*"]
```
- **Core sync types** (`src/sync/types.rs`)
  - `SyncRepo`, `SyncConfig`, `ProtectedFile` structs
  - Version management types

#### Hour 2: Repository Operations
- **External repository handling** (`src/sync/repository.rs`)
  - Git clone/fetch operations for external repos
  - Version/tag resolution
  - File filtering and extraction
- **Local file operations** (`src/sync/filesystem.rs`)
  - Safe file copying with backup
  - Directory structure management
  - Conflict detection

#### Hour 3: Protection System
- **Protection enforcement** (`src/sync/protection.rs`)
  - Protected file registry
  - Pre-commit protection checks
  - File modification detection
- **Protection integration** - Modify existing pre-commit hook
  - Check if staged files are protected
  - Block commits that modify protected files

#### Hour 4: Sync Commands
- **Sync command implementation** (`src/cli/commands/sync.rs`)
  - `guardy sync` - Sync all configured repos
  - `guardy sync <repo-name>` - Sync specific repo
  - `guardy protected list` - Show protected files
- **Status integration** - Show sync status in `guardy status`

#### Hour 5: Advanced Features & Testing
- **Conflict resolution** - Handle merge conflicts gracefully
- **Backup/restore** - Rollback failed syncs
- **Comprehensive testing** - Integration tests for sync system
- **Performance optimization** - Parallel syncing, caching

## Configuration Evolution

### Current Configuration (guardy.toml)
```toml
[general]
debug = false
color = true

[security]
patterns = ["sk-[a-zA-Z0-9]{48}"]

[hooks]
pre_commit = true
commit_msg = true
post_checkout = true
pre_push = true
```

### Enhanced Configuration (Phase 2)
```toml
[general]
debug = false
color = true

[security]
patterns = ["sk-[a-zA-Z0-9]{48}"]

[hooks]
pre_commit = true
commit_msg = true
post_checkout = true
pre_push = true

[sync]
enabled = true
auto_protect = true  # Automatically protect synced files

[[sync.repos]]
name = "build-toolkit"
repo = "org/build-toolkit"
version = "v1.2.3"
target_path = "."
include_patterns = ["*.yml", "*.toml", "Makefile"]
exclude_patterns = ["secrets/*", "local-*"]

[[sync.repos]]
name = "shared-configs"
repo = "org/shared-configs" 
version = "v2.1.0"
target_path = ".configs/"

[protection]
auto_protect_synced = true
allow_override = false  # Strict protection mode
```

## Command Interface

### Phase 1: Enhanced Hook Commands
```bash
# Existing commands (enhanced)
guardy install                    # Install all hooks with Lefthook compatibility
guardy run pre-commit            # Run pre-commit with full validation
guardy status                    # Show hook status + protected files

# New hook-specific options
guardy install --hooks pre-commit,commit-msg  # Install specific hooks
guardy run pre-push --skip-tests             # Skip test execution
```

### Phase 2: Protected Sync Commands
```bash
# Sync management
guardy sync                      # Sync all configured repositories
guardy sync build-toolkit        # Sync specific repository
guardy sync --version v1.3.0     # Upgrade to specific version
guardy sync --dry-run            # Show what would be synced

# Protection management  
guardy protected list            # List all protected files
guardy protected status          # Show protection status
guardy protected check           # Validate protection integrity

# Advanced operations
guardy sync rollback             # Rollback last sync operation
guardy sync backup               # Create backup before sync
```

## Success Criteria

### Phase 1 (Lefthook Replacement)
- ✅ **Complete hook implementation** - All 4 hooks fully functional
- ✅ **Lefthook feature parity** - Matches core Lefthook capabilities
- ✅ **Performance superiority** - Faster than bash-based alternatives
- ✅ **Easy migration** - Drop-in replacement for Lefthook
- ✅ **Professional UX** - Claude Code style output and experience

### Phase 2 (Protected Sync)
- ✅ **Automatic file protection** - Synced files cannot be modified
- ✅ **Multi-repository sync** - Support multiple external repositories
- ✅ **Version management** - Pin and upgrade repository versions
- ✅ **Conflict resolution** - Handle sync conflicts gracefully
- ✅ **Integration testing** - Comprehensive test coverage

## Migration Strategy

### From Lefthook
1. **Side-by-side installation** - Guardy can coexist with Lefthook initially
2. **Configuration migration** - Tool to convert `lefthook.yml` to `guardy.toml`
3. **Feature flags** - Gradual adoption of Guardy features
4. **Compatibility mode** - Support Lefthook-style configuration initially

### Adoption Path
1. **Install Guardy** - `cargo install guardy` or prebuilt binaries
2. **Replace hooks** - `guardy install` replaces Lefthook hooks
3. **Add protected sync** - Configure sync repositories in `guardy.toml`
4. **Remove Lefthook** - Clean removal once migration complete

## Technical Implementation Details

### Dependencies Required
```toml
# Additional dependencies for Phase 1
git2 = "0.20"           # Enhanced git operations
regex = "1.11"          # Conventional commit validation
which = "8.0"           # External tool detection
tokio = "1.46"          # Async process execution

# Additional dependencies for Phase 2  
tempfile = "3.20"       # Temporary directories for cloning
tar = "0.4"             # Archive handling
flate2 = "1.0"          # Compression support
semver = "1.0"          # Version comparison
```

### Performance Optimizations
- **Parallel scanning** - Concurrent file processing
- **Async operations** - Non-blocking git and network operations
- **Caching** - Cache external repository clones
- **Incremental updates** - Only sync changed files

### Error Handling Strategy
- **Graceful degradation** - Continue operation if non-critical features fail
- **Detailed error messages** - Clear actionable error descriptions
- **Rollback capabilities** - Automatic rollback on sync failures
- **User confirmation** - Interactive prompts for destructive operations

## Risks & Mitigations

### Technical Risks
- **Git complexity** - Mitigated by using battle-tested git2 crate
- **Sync conflicts** - Comprehensive conflict detection and resolution
- **Performance** - Extensive benchmarking and optimization

### User Experience Risks
- **Migration complexity** - Automated migration tools and clear documentation
- **Configuration complexity** - Smart defaults and validation
- **Lock-in concerns** - Export capabilities and standard formats

## Revised Timeline Summary (MVP Focus)

### **Optimized AI Coding Agent Estimates**
- **Phase 1 (Essential Lefthook MVP)**: 6-8 hours
- **Phase 2 (Protected Sync)**: 4-5 hours  
- **Migration Support**: 2 hours
- **Testing & Integration**: 2-3 hours
- **Documentation**: 1-2 hours

**Total MVP Implementation Time: 15-20 hours AI agent time**

### **Future Enhancement Estimates**
- **Phase 3 (Advanced Lefthook Features)**: 6-8 hours *(deferred)*
- **Phase 4 (Enterprise Features)**: 4-6 hours *(deferred)*
- **Total Future Features**: 10-14 hours *(post-MVP)*

### Human Equivalent
- **MVP**: 6-8 weeks
- **Full Feature Parity**: 12-15 weeks
- **Total with Future Features**: 18-23 weeks

### **Strategic MVP Approach**
**Focused on 80% of commercial use cases** rather than 100% feature parity:
- **Essential Features**: Parallel execution, file variables, custom scripts, stage_fixed
- **Deferred Features**: Piped execution, tags, priority ordering, Docker integration
- **Key Insight**: Custom script support is Lefthook's killer feature and much simpler to implement than complex orchestration

**MVP delivers commercial-grade git hook management while building toward the unique Protected Sync differentiator.**

## Next Steps

1. **Review and approve this plan** - Ensure alignment with requirements
2. **Phase 1 implementation** - Complete Lefthook replacement
3. **Testing and validation** - Comprehensive testing of hook system
4. **Phase 2 implementation** - Add Protected Sync capabilities
5. **Integration testing** - End-to-end testing with real repositories
6. **Documentation and release** - Prepare for broader adoption

---

*This plan transforms Guardy into a comprehensive repository management tool that combines the best of Lefthook's git hook management with enterprise-grade configuration protection and synchronization capabilities.*