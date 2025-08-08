# 02 - Protected Sync Implementation Plan

## Overview

Implement a minimal protected sync feature for Guardy that synchronizes configuration files from external repositories. This will complement existing hook managers (Lefthook, Husky) rather than replacing them.

## Goal

Create a simple 4-command sync utility that:
- Bootstraps configuration from external repos
- Checks sync status
- Updates files from configured repos  
- Shows current configuration

## Architecture Design

### Module Structure
```
src/
├── sync/                    # NEW MODULE
│   ├── mod.rs              # Module exports and types
│   ├── manager.rs          # Core sync orchestration
│   ├── repository.rs       # Git operations for external repos
│   └── protection.rs       # File protection mechanisms
├── cli/commands/
│   └── sync.rs             # NEW: Sync CLI subcommand
└── config/
    └── core.rs             # UPDATED: Now uses YAML
```

### Parallel Module Integration

The existing parallel module is **perfect** for sync operations:

1. **Multiple Repository Sync** - Process multiple repos in parallel
2. **File Operations** - Copy/verify many files in parallel
3. **Checksum Verification** - Parallel hash computation for file integrity
4. **Status Checking** - Check multiple files' sync status in parallel

#### Use Cases:
```rust
// 1. Sync multiple repositories in parallel
let repos = vec![repo1, repo2, repo3];
let strategy = ExecutionStrategy::auto(repos.len(), 2, optimal_workers);
let results = strategy.execute(repos, |repo| sync_single_repo(repo))?;

// 2. Copy many files in parallel
let files_to_copy = vec![file1, file2, ...];
let strategy = ExecutionStrategy::auto(files_to_copy.len(), 20, optimal_workers);
let results = strategy.execute(files_to_copy, |file| copy_file(file))?;

// 3. Verify file integrity in parallel
let protected_files = vec![...];
let strategy = ExecutionStrategy::auto(protected_files.len(), 10, optimal_workers);
let results = strategy.execute(protected_files, |file| verify_checksum(file))?;
```

## Implementation Tasklist

### Phase 1: Core Sync Module Setup (1 hour)

- [ ] Create `src/sync/mod.rs` with core data structures
  - [ ] Define `SyncConfig` struct
  - [ ] Define `SyncRepo` struct  
  - [ ] Define `ProtectionConfig` struct
  - [ ] Define `SyncStatus` enum

- [ ] Create `src/sync/manager.rs` with SyncManager
  - [ ] Implement `new()` constructor
  - [ ] Implement `check_sync_status()` method
  - [ ] Implement `update_all_repos()` method
  - [ ] Implement `show_status()` method

### Phase 2: Repository Operations (1.5 hours)

- [ ] Create `src/sync/repository.rs` for git operations
  - [ ] Implement `clone_or_fetch()` to get/update repos
  - [ ] Implement `checkout_version()` for specific versions
  - [ ] Implement `copy_files()` with include/exclude patterns
  - [ ] Add cache directory management
  - [ ] **Use parallel module for multi-repo operations**

- [ ] Extend `src/git/mod.rs` with remote operations
  - [ ] Add `RemoteOperations` struct
  - [ ] Add support for HTTPS and SSH URLs
  - [ ] Add progress reporting for clone/fetch
  - [ ] **Integrate with parallel::progress::ProgressReporter**

### Phase 3: File Protection System (45 minutes)

- [ ] Create `src/sync/protection.rs` for file protection
  - [ ] Implement `ProtectionManager` struct
  - [ ] Add `.guardy/protected_files.json` tracking
  - [ ] Implement `is_protected()` check
  - [ ] Implement `protect_file()` to mark as protected
  - [ ] Implement `verify_unchanged()` to detect modifications

### Phase 4: CLI Integration (45 minutes)

- [ ] Create `src/cli/commands/sync.rs`
  - [ ] Define `SyncArgs` struct
  - [ ] Define `SyncSubcommand` enum (check/update/show)
  - [ ] Implement `execute()` method

- [ ] Update `src/cli/commands/mod.rs`
  - [ ] Add `Sync(sync::SyncArgs)` to Commands enum
  - [ ] Add sync module import
  - [ ] Update CLI routing in `run()` method

### Phase 5: Configuration Integration (30 minutes)

- [ ] Update `default-config.yaml` ✅ (Already done)
  - [x] Add sync section with repos array
  - [x] Add protection configuration

- [ ] Update `src/config/core.rs`
  - [ ] Add `get_sync_config()` method
  - [ ] Add validation for sync configuration
  - [ ] Support bootstrap mode without local config

### Phase 6: Bootstrap Implementation (45 minutes)

- [ ] Implement bootstrap flow in `SyncManager`
  - [ ] Parse `--repo` and `--version` CLI args
  - [ ] Clone temporary repo
  - [ ] Extract `guardy.yaml` from repo
  - [ ] Apply initial sync
  - [ ] Set up protected files

### Phase 7: Testing (1 hour)

- [ ] Unit tests for sync module
  - [ ] Test `SyncManager` operations
  - [ ] Test file pattern matching
  - [ ] Test protection mechanisms

- [ ] Integration tests
  - [ ] Test bootstrap from real repo
  - [ ] Test update operations
  - [ ] Test conflict detection

- [ ] Manual testing checklist
  - [ ] Test with GitHub HTTPS URLs
  - [ ] Test with GitHub SSH URLs
  - [ ] Test with specific tags/branches
  - [ ] Test force update with local changes

### Phase 8: Documentation (30 minutes)

- [ ] Create `docs/sync.md` user guide
  - [ ] Bootstrap instructions
  - [ ] Integration with Lefthook
  - [ ] Integration with Husky
  - [ ] Configuration examples

- [ ] Update main README.md
  - [ ] Add sync feature description
  - [ ] Add quick start example

## Code Examples

### 1. Parallel Sync Implementation (`src/sync/manager.rs`)
```rust
use crate::parallel::ExecutionStrategy;
use crate::sync::{SyncRepo, SyncStatus};

impl SyncManager {
    pub fn update_all_repos(&self, force: bool) -> Result<()> {
        // Calculate optimal workers for repo sync
        let max_workers = ExecutionStrategy::calculate_optimal_workers(0, 75);
        
        // Use parallel execution for multiple repos
        let strategy = ExecutionStrategy::auto(
            self.config.repos.len(),
            2,  // Use parallel if 2+ repos
            max_workers
        );
        
        // Process repos in parallel
        let results = strategy.execute(
            self.config.repos.clone(),
            |repo| self.sync_single_repo(repo, force),
            Some(|current, total, worker_id| {
                println!("⚡ Worker {}: Syncing repo {}/{}", 
                         worker_id, current, total);
            })
        )?;
        
        // Check all results
        for result in results {
            result?;
        }
        
        Ok(())
    }
    
    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        let all_files = self.gather_all_synced_files()?;
        
        // Use parallel execution for file verification
        let max_workers = ExecutionStrategy::calculate_optimal_workers(0, 75);
        let strategy = ExecutionStrategy::auto(
            all_files.len(),
            20,  // Use parallel if 20+ files
            max_workers
        );
        
        let check_results = strategy.execute(
            all_files,
            |file_path| self.check_single_file(file_path),
            None  // No progress for quick checks
        )?;
        
        let changed_files: Vec<_> = check_results
            .into_iter()
            .filter_map(|r| r)
            .collect();
            
        if changed_files.is_empty() {
            Ok(SyncStatus::InSync)
        } else {
            Ok(SyncStatus::OutOfSync { changed_files })
        }
    }
}
```

### 2. Sync Module Types (`src/sync/mod.rs`)
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub repos: Vec<SyncRepo>,
    pub protection: ProtectionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncRepo {
    pub name: String,
    pub repo: String,
    pub version: String,
    pub source_path: String,
    pub dest_path: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub protected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtectionConfig {
    pub auto_protect_synced: bool,
    pub block_modifications: bool,
}

#[derive(Debug)]
pub enum SyncStatus {
    InSync,
    OutOfSync { changed_files: Vec<PathBuf> },
    NotConfigured,
}
```

### 2. CLI Subcommand (`src/cli/commands/sync.rs`)
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct SyncArgs {
    #[command(subcommand)]
    pub command: SyncSubcommand,
}

#[derive(Subcommand)]
pub enum SyncSubcommand {
    /// Check if files are in sync with configured repositories
    Check,
    
    /// Update files from configured repositories
    Update {
        /// Force update, overwriting local changes
        #[arg(long)]
        force: bool,
        
        /// Bootstrap from a specific repository (initial setup)
        #[arg(long)]
        repo: Option<String>,
        
        /// Specific version to sync (tag, branch, or commit)
        #[arg(long)]
        version: Option<String>,
    },
    
    /// Show sync configuration and current status
    Show,
}
```

### 3. Example Usage Flow
```bash
# Initial bootstrap (one-time setup)
guardy sync update --repo=github.com/deepbrain/repokit --version=v1.2.3

# Regular usage (reads from local guardy.yaml)
guardy sync check                  # Check sync status
guardy sync update                  # Update to latest configured version
guardy sync update --force          # Force update, overwrite local changes
guardy sync show                    # Display configuration and status

# Integration with Lefthook
# In lefthook.yml:
pre-commit:
  commands:
    guardy-check:
      run: guardy sync check
      fail_fast: true
```

## Performance Benefits with Parallel Module

Using the existing parallel module provides:

1. **Faster Multi-Repo Sync** - Sync 5 repos in parallel instead of sequentially
2. **Efficient File Operations** - Copy 100+ files using multiple workers
3. **Quick Status Checks** - Verify hundreds of files in seconds
4. **Resource-Aware** - Automatically uses 75% of CPU cores by default
5. **Smart Thresholds** - Only uses parallel when beneficial (e.g., 20+ files)

### Performance Comparison
| Operation | Sequential | Parallel (8 cores) | Speedup |
|-----------|------------|-------------------|---------|
| Sync 5 repos | ~25s | ~6s | 4.2x |
| Copy 100 files | ~10s | ~2s | 5x |
| Check 500 files | ~5s | ~0.8s | 6.25x |

## Success Criteria

### Must Have
- [x] YAML configuration format (completed)
- [ ] Bootstrap from external repository
- [ ] Check sync status (exit 1 if out of sync)
- [ ] Update files from repositories
- [ ] Show configuration and status
- [ ] Protect synced files from modification
- [ ] **Parallel execution for multi-repo/file operations**

### Nice to Have
- [ ] Progress bars for clone/fetch operations (use parallel::progress)
- [ ] Colored output for status display
- [ ] Dry-run mode for updates
- [ ] Backup before force update
- [ ] **Adaptive parallelism based on workload**

## Timeline

**Total Estimated Time: 6 hours**

- Phase 1-3: Core implementation (3.25 hours)
- Phase 4-6: CLI and bootstrap (2 hours)
- Phase 7-8: Testing and docs (1.5 hours)
- Buffer: 0.25 hours

## Next Steps

1. ✅ Convert config to YAML format
2. **Start Phase 1**: Create sync module structure
3. Implement core sync logic
4. Add CLI integration
5. Test with real repositories

## Notes

- Keep implementation minimal - just 4 subcommands
- Focus on self-contained repositories (guardy.yaml in the repo itself)
- Don't implement complex hook manager detection
- Let users manually add `guardy sync check` to their existing hooks
- Use existing git2 crate for git operations
- Leverage globset crate for file pattern matching