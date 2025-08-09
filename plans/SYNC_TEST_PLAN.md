# Guardy Sync Test Plan

## Overview
Testing strategy for guardy sync functionality between repokit (template) and rusttoolkit (consumer).

## Test Phases

### Phase 1: Unit Tests (Isolated)
Run with: `cargo test --lib`

#### 1.1 Protection Manager Tests
- [ ] `test_mark_file_protected` - Mark a file as protected
- [ ] `test_is_file_protected` - Check protection status
- [ ] `test_unprotect_file` - Remove protection from file
- [ ] `test_clear_all_protections` - Clear all protections
- [ ] `test_persist_load_state` - Save and restore protection state

#### 1.2 Sync Config Tests
- [ ] `test_parse_valid_config` - Parse valid sync configuration
- [ ] `test_missing_config` - Handle missing configuration gracefully
- [ ] `test_invalid_repo_url` - Validate repository URLs
- [ ] `test_glob_patterns` - Test include/exclude glob patterns

#### 1.3 Git Operations Tests
- [ ] `test_clone_repo` - Clone repository successfully
- [ ] `test_checkout_version` - Checkout specific tag/branch/commit
- [ ] `test_fetch_updates` - Fetch latest changes
- [ ] `test_list_repo_files` - Get file list from repository

### Phase 2: Integration Tests
Run with: `cargo test --test sync_integration`

#### 2.1 Bootstrap Tests
```bash
# Manual test command:
cargo run -- sync update --repo=file:///path/to/repokit --version=main
```
- [ ] Bootstrap from scratch (no existing config)
- [ ] Create `.guardy/sync.yaml` configuration
- [ ] Download files to correct destinations
- [ ] Mark synced files as protected

#### 2.2 Sync Status Tests
```bash
# Manual test command:
cargo run -- sync check
```
- [ ] Detect in-sync state
- [ ] Detect out-of-sync files
- [ ] Show protected file indicators
- [ ] Handle missing sync config

#### 2.3 Update Tests
```bash
# Manual test commands:
cargo run -- sync update
cargo run -- sync update --force
```
- [ ] Update only changed files
- [ ] Preserve local changes in protected files
- [ ] Force update overwrites local changes
- [ ] Create backups before updates

#### 2.4 Protection Tests
```bash
# Manual test commands:
cargo run -- sync unprotect .eslintrc.json
cargo run -- sync unprotect --all
cargo run -- sync protected
```
- [ ] Block modifications to protected files
- [ ] Allow modifications after unprotect
- [ ] List all protected files
- [ ] Restore from backup

### Phase 3: End-to-End Tests (Real Repos)

#### 3.1 Setup Test Environment
```bash
# Create test repos
mkdir -p /tmp/guardy-test/{repokit,rusttoolkit}

# Setup repokit (template)
cd /tmp/guardy-test/repokit
git init
# Add template files (.github/workflows, .eslintrc.json, etc.)
git add .
git commit -m "Initial template"
git tag v1.0.0

# Setup rusttoolkit (consumer)
cd /tmp/guardy-test/rusttoolkit
git init
# Add Rust project files
git add .
git commit -m "Initial project"
```

#### 3.2 Test Scenarios

**Scenario 1: Initial Bootstrap**
```bash
cd /tmp/guardy-test/rusttoolkit
guardy sync update --repo=file:///tmp/guardy-test/repokit --version=v1.0.0

# Verify:
# - Files are synced correctly
# - .guardy/sync.yaml is created
# - Files are marked as protected
```

**Scenario 2: Update After Template Changes**
```bash
# Update repokit
cd /tmp/guardy-test/repokit
echo "updated" >> .eslintrc.json
git add . && git commit -m "Update eslint"
git tag v1.0.1

# Sync updates
cd /tmp/guardy-test/rusttoolkit
guardy sync check  # Should show out-of-sync
guardy sync update # Should update files
```

**Scenario 3: Local Modifications**
```bash
cd /tmp/guardy-test/rusttoolkit
# Modify protected file
echo "local change" >> .eslintrc.json
guardy sync update  # Should preserve local change

# Force update
guardy sync update --force  # Should overwrite with template
```

**Scenario 4: Unprotect and Modify**
```bash
cd /tmp/guardy-test/rusttoolkit
guardy sync unprotect .eslintrc.json
echo "custom config" > .eslintrc.json
guardy sync update  # Should skip unprotected file
```

### Phase 4: Publishing and Production Test

#### 4.1 Publish to crates.io
```bash
# Update Cargo.toml version
cd packages/guardy
cargo publish --dry-run  # Test first
cargo publish            # Actual publish
```

#### 4.2 Use Published Version
```bash
# In rusttoolkit
cargo install guardy
guardy sync update --repo=https://github.com/deepbrain/repokit --version=v1.0.0
```

## Test Automation

### GitHub Actions Workflow
```yaml
name: Sync Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run unit tests
        run: cargo test --lib
      - name: Run integration tests
        run: cargo test --test sync_integration
      - name: Run E2E tests
        run: ./scripts/test-sync-e2e.sh
```

## Success Criteria

1. **Reliability**: All tests pass consistently
2. **Protection**: Files cannot be accidentally modified when protected
3. **Sync Accuracy**: Files match source repository exactly
4. **Performance**: Sync completes in < 5 seconds for typical repos
5. **Error Handling**: Graceful handling of network issues, missing files, etc.

## Test Data Requirements

### Mock Repokit Structure
```
repokit/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── .eslintrc.json
├── .prettierrc
├── tsconfig.base.json
├── .gitignore
└── README.md
```

### Expected Rusttoolkit Structure After Sync
```
rusttoolkit/
├── .guardy/
│   ├── sync.yaml
│   └── protected_files.json
├── .github/           # Synced
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── .eslintrc.json     # Synced
├── .prettierrc        # Synced
├── tsconfig.base.json # Synced
├── .gitignore         # Synced (merged)
├── Cargo.toml         # Original
└── src/               # Original
    └── main.rs
```

## Next Steps

1. Implement missing sync functionality
2. Create automated test suite
3. Set up CI/CD for continuous testing
4. Document sync configuration format
5. Create user guide for sync feature