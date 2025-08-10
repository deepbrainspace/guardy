# Plan: Guardy Sync Testing & Implementation

## Overview
Implement and test the guardy sync functionality that allows rusttoolkit to sync template files from repokit while maintaining local customizations through file protection.

## Current Status
- ✅ Basic sync module structure exists (`src/sync/`)
- ✅ CLI commands implemented (`src/cli/commands/sync.rs`)
- ✅ Protection manager implemented (`src/sync/protection.rs`)
- ⚠️ Sync manager partially implemented (`src/sync/manager.rs`)
- ❌ No test coverage for sync functionality
- ❌ Not yet tested with real repos (repokit/rusttoolkit)
- ❌ Not published to crates.io

## Architecture & Design

### Core Components
1. **SyncManager** - Orchestrates sync operations
2. **ProtectionManager** - Manages file protection state
3. **GitOperations** - Handles git clone/fetch/checkout
4. **FileSync** - Copies files with pattern matching

### Data Flow
```
repokit (source) → Git Clone/Fetch → File Filtering → Copy to rusttoolkit → Mark as Protected
```

### Configuration Format
```yaml
sync:
  repos:
    - name: repokit
      repo: github.com/deepbrain/repokit
      version: v1.0.0
      source_path: "."
      dest_path: "."
      include: [".github/**", ".eslintrc.json", "tsconfig.base.json"]
      exclude: ["*.log", "*.tmp"]
      protected: true
  protection:
    auto_protect_synced: true
    block_modifications: true
```

## Task List

### Phase 1: Complete Core Implementation
- [ ] Fix SyncManager::bootstrap() method
- [ ] Implement SyncManager::update_all_repos()
- [ ] Add git operations (clone, fetch, checkout)
- [ ] Implement file filtering with glob patterns
- [ ] Add file copy with merge strategy
- [ ] Create backup before updates
- [ ] Add restore from backup functionality

### Phase 2: Create Test Infrastructure
- [ ] Set up unit tests for ProtectionManager
- [ ] Set up unit tests for SyncConfig parsing
- [ ] Create integration test harness with temp directories
- [ ] Add mock git repositories for testing
- [ ] Implement test scenarios from SYNC_TEST_PLAN.md

### Phase 3: Test with Mock Repositories
- [ ] Create mock repokit with template files
- [ ] Create mock rusttoolkit with Rust project
- [ ] Test bootstrap command
- [ ] Test sync check command
- [ ] Test update with local changes
- [ ] Test force update
- [ ] Test unprotect functionality
- [ ] Test restore from backup

### Phase 4: Real Repository Testing
- [ ] Set up real test repos in /tmp/guardy-test/
- [ ] Test with actual git operations
- [ ] Test with file:// protocol
- [ ] Test with https:// protocol
- [ ] Test version checkout (tags/branches)
- [ ] Test large file sync performance
- [ ] Test error scenarios (network, permissions)

### Phase 5: Production Preparation
- [ ] Add comprehensive error handling
- [ ] Add progress indicators for long operations
- [ ] Add dry-run mode
- [ ] Add verbose logging option
- [ ] Update documentation
- [ ] Add examples to README

### Phase 6: Publishing
- [ ] Update Cargo.toml metadata
- [ ] Run cargo fmt and cargo clippy
- [ ] Run full test suite
- [ ] Test with cargo publish --dry-run
- [ ] Publish to crates.io
- [ ] Create GitHub release

### Phase 7: Integration Testing
- [ ] Install published version in rusttoolkit
- [ ] Configure rusttoolkit to sync from repokit
- [ ] Test initial bootstrap
- [ ] Test ongoing sync updates
- [ ] Monitor for issues

## Test Scenarios

### Scenario 1: Initial Bootstrap
```bash
cd rusttoolkit
guardy sync update --repo=https://github.com/deepbrain/repokit --version=main
# Should create .guardy/sync.yaml and sync files
```

### Scenario 2: Check Sync Status
```bash
guardy sync check
# Should show if files are in sync or modified
```

### Scenario 3: Update with Protection
```bash
# Modify a protected file
echo "custom" > .eslintrc.json
guardy sync update
# Should preserve local change

guardy sync update --force
# Should overwrite with template version
```

### Scenario 4: Unprotect Files
```bash
guardy sync unprotect .eslintrc.json
# File can now be modified without sync overwriting it
```

## Success Criteria

1. **Functionality**
   - Bootstrap works from scratch
   - Sync correctly identifies changed files
   - Protection prevents accidental overwrites
   - Force update works when needed

2. **Reliability**
   - All tests pass consistently
   - Handles network errors gracefully
   - Handles file permission issues
   - Creates backups before destructive operations

3. **Performance**
   - Sync completes in < 5 seconds for typical repos
   - Minimal memory usage
   - Efficient git operations

4. **Usability**
   - Clear error messages
   - Intuitive command structure
   - Good documentation
   - Progress indicators for long operations

## Known Issues & Risks

1. **Git Operations**: Need to handle various git scenarios (auth, submodules, large files)
2. **File Permissions**: Cross-platform compatibility (Windows/Linux/Mac)
3. **Merge Conflicts**: When syncing .gitignore and other merged files
4. **Version Management**: Handling different version formats (tags, branches, commits)

## Next Immediate Steps

1. **Review existing code** - Check what's already implemented in `src/sync/manager.rs`
2. **Fix compilation issues** - Ensure all modules compile
3. **Create minimal test** - Start with simplest bootstrap test
4. **Iterate** - Add features incrementally with tests

## Files to Create/Modify

- `src/sync/manager.rs` - Complete implementation
- `src/sync/git.rs` - Add git operations module
- `src/sync/file_ops.rs` - Add file operations module
- `tests/sync_unit.rs` - Unit tests
- `tests/sync_integration.rs` - Integration tests
- `scripts/test-sync-e2e.sh` - End-to-end test script
- `.github/workflows/sync-tests.yml` - CI workflow

## Commands for Testing

```bash
# Build and test locally
cargo build --release
cargo test --test sync_integration

# Manual testing
cd /tmp/test-repo
cargo run -- sync update --repo=file:///tmp/repokit --version=main
cargo run -- sync check
cargo run -- sync show
cargo run -- sync protected
cargo run -- sync unprotect --all
```

## Timeline Estimate

- Phase 1-2: 2-3 hours (implementation)
- Phase 3-4: 2-3 hours (testing)
- Phase 5-6: 1-2 hours (publishing)
- Phase 7: 1 hour (integration)

Total: ~8-10 hours of focused work

## Dependencies

- Git must be installed
- Network access for remote repos
- File system permissions for local operations
- Cargo/Rust for building and testing