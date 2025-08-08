# Guardy Protected Sync Implementation Plan

## Executive Summary

Implement a minimal, focused protected sync feature for Guardy that allows teams to synchronize configuration files from external repositories. This will be a complementary tool to existing hook managers (Lefthook, Husky), not a replacement.

## Current Code Analysis

### ✅ Strengths of Current Implementation
1. **Well-architected modular structure** - Clear separation of concerns with distinct modules
2. **Advanced parallel execution framework** - Sophisticated resource management and execution strategies
3. **Robust configuration system** - Uses SuperConfig with hierarchical loading and environment variable support
4. **Strong CLI foundation** - Well-structured CLI with subcommands using clap
5. **Git integration** - Basic git2 operations already in place
6. **Security scanning** - Comprehensive secret detection with entropy analysis

### ⚠️ Areas for Improvement
1. **External module is empty** - Placeholder for external tool integrations
2. **No sync functionality** - Core protected sync feature missing
3. **No repository management** - No code for cloning/fetching external repos
4. **No file protection mechanism** - No way to mark files as protected

### 🔍 Key Observations
- The codebase is **optimally structured** for adding new features
- The parallel module shows **excellent separation of concerns** with detailed documentation
- Config system is **flexible enough** to add sync configuration
- Git module provides **foundation** but needs extension for remote operations

## Implementation Strategy

### Phase 1: Core Sync Module (3 hours)

#### 1.1 Create Sync Module Structure
```
src/sync/
├── mod.rs           # Module exports
├── manager.rs       # Main sync orchestration
├── repository.rs    # External repo operations
└── protection.rs    # File protection enforcement
```

#### 1.2 Core Data Structures
```rust
// src/sync/mod.rs
pub struct SyncConfig {
    repos: Vec<SyncRepo>,
    protection: ProtectionConfig,
}

pub struct SyncRepo {
    name: String,
    repo: String,
    version: String,
    source_path: String,
    dest_path: String,
    include: Vec<String>,
    exclude: Vec<String>,
    protected: bool,
}

pub struct ProtectionConfig {
    auto_protect_synced: bool,
    block_modifications: bool,
}
```

#### 1.3 Manager Implementation
```rust
// src/sync/manager.rs
pub struct SyncManager {
    config: SyncConfig,
    git_ops: GitOperations,
}

impl SyncManager {
    pub fn check_sync_status(&self) -> Result<SyncStatus>;
    pub fn update_all_repos(&self, force: bool) -> Result<()>;
    pub fn show_status(&self) -> Result<String>;
}
```

### Phase 2: CLI Integration (1 hour)

#### 2.1 Add Sync Subcommand
```rust
// src/cli/commands/sync.rs
#[derive(Subcommand)]
pub enum SyncSubcommand {
    /// Check if files are in sync
    Check,
    /// Update files from configured repositories
    Update { 
        #[arg(long)]
        force: bool,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        version: Option<String>,
    },
    /// Show sync configuration and status
    Show,
}
```

#### 2.2 Update Main CLI
```rust
// src/cli/commands/mod.rs
pub enum Commands {
    // ... existing commands ...
    /// Protected file synchronization
    Sync(sync::SyncArgs),
}
```

### Phase 3: Configuration Integration (30 minutes)

#### 3.1 Update Default Config
```toml
# default-config.toml
[sync]
repos = []

[sync.protection]
auto_protect_synced = true
block_modifications = true
```

#### 3.2 Bootstrap Configuration
Support initial bootstrap without local config:
```rust
impl SyncManager {
    pub fn bootstrap(repo: &str, version: &str) -> Result<()> {
        // Clone repo, extract guardy.yml, apply sync
    }
}
```

### Phase 4: Git Operations Extension (1 hour)

#### 4.1 Extend Git Module
```rust
// src/git/remote.rs
pub struct RemoteOperations {
    cache_dir: PathBuf,
}

impl RemoteOperations {
    pub fn clone_or_fetch(&self, repo: &str, version: &str) -> Result<PathBuf>;
    pub fn checkout_version(&self, repo_path: &Path, version: &str) -> Result<()>;
}
```

### Phase 5: File Protection (30 minutes)

#### 5.1 Protection Mechanism
```rust
// src/sync/protection.rs
pub struct ProtectionManager {
    protected_files: HashSet<PathBuf>,
}

impl ProtectionManager {
    pub fn is_protected(&self, path: &Path) -> bool;
    pub fn protect_file(&mut self, path: &Path) -> Result<()>;
    pub fn verify_unchanged(&self, path: &Path) -> Result<bool>;
}
```

## Implementation Gaps to Fill

### Required New Components
1. **Sync module** - Complete new module for sync functionality
2. **Remote git operations** - Clone, fetch, checkout specific versions
3. **File sync logic** - Copy files with include/exclude patterns
4. **Protection tracking** - Store and check protected file status
5. **CLI sync command** - New subcommand with check/update/show

### Configuration Changes
1. Add sync section to default-config.toml
2. Support bootstrap mode for initial setup
3. Add sync configuration to GuardyConfig

### Integration Points
1. Hook into existing git module for repository operations
2. Use existing config system for sync configuration
3. Leverage CLI framework for new sync command
4. Potentially use parallel module for multi-repo sync

## Task Breakdown

### Priority 1: Core Implementation (4 hours)
- [ ] Create sync module structure
- [ ] Implement SyncManager with check/update/show
- [ ] Add remote git operations
- [ ] Implement file copying with patterns
- [ ] Add basic protection tracking

### Priority 2: CLI Integration (1 hour)
- [ ] Add sync subcommand to CLI
- [ ] Implement check, update, show subcommands
- [ ] Add bootstrap support for initial setup
- [ ] Update help text and documentation

### Priority 3: Testing & Polish (1 hour)
- [ ] Unit tests for sync operations
- [ ] Integration tests with test repos
- [ ] Error handling and recovery
- [ ] Progress reporting for long operations

## Risk Mitigation

### Technical Risks
1. **Git operations complexity** → Use git2 crate's high-level APIs
2. **File permission issues** → Check permissions before operations
3. **Network failures** → Add retry logic with exponential backoff
4. **Config conflicts** → Clear precedence rules and validation

### Design Decisions
1. **Keep it minimal** - Just sync functionality, no hook management
2. **Self-contained repos** - guardy.yml lives in synced repo
3. **Simple CLI** - Only 4 subcommands (check/update/show + bootstrap)
4. **No complex integration** - Just document how to add to existing hooks

## Success Criteria

### Functional Requirements
- ✅ Can bootstrap from external repo with single command
- ✅ Can check if local files match synced versions
- ✅ Can update files from configured repositories
- ✅ Can show current sync configuration and status
- ✅ Prevents modification of protected files

### Non-Functional Requirements
- ✅ Completes sync operations in < 5 seconds for typical repo
- ✅ Clear error messages for common failures
- ✅ Works with existing hook managers without conflicts
- ✅ Single binary, no external dependencies

## Next Steps

1. **Review and approve this plan** - Confirm approach aligns with requirements
2. **Create sync module structure** - Set up basic module files
3. **Implement core sync logic** - Start with SyncManager
4. **Add CLI integration** - Wire up sync subcommand
5. **Test with real repositories** - Validate with actual use cases

## Conclusion

The current Guardy codebase is **well-architected and optimal** for its current feature set. The modular structure, separation of concerns, and robust foundation make it straightforward to add the protected sync feature without major refactoring.

The implementation should take approximately **6 hours total**, delivering a minimal but powerful sync utility that complements existing hook managers rather than competing with them.