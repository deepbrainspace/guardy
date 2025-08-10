# Protected Sync Plan (Consolidated)

## Executive Summary

Deliver a minimal, fast "Protected Sync" feature that synchronizes and protects configuration files from external repositories. Guardy complements existing hook managers (Lefthook, Husky, pre-commit) instead of replacing them. Users wire Guardy via simple hook calls like `guardy sync check`.

- Focus: Bootstrap, Check, Update, Show, plus Protection (unprotect/restore) and Backups
- Keep simple: YAML config, no complex hook detection, single binary
- Performance: Leverage the parallel module where beneficial (multi-repo/file operations)

## Current Status (code)

- ✅ Module scaffolding: `packages/guardy/src/sync/{mod.rs,manager.rs,protection.rs}`
- ✅ Remote git ops: `packages/guardy/src/git/remote.rs` (clone/fetch/checkout)
- ✅ CLI subcommand: `packages/guardy/src/cli/commands/sync.rs` (Check/Update/Show/Unprotect/Protected/Restore)
- ✅ Protection: protect/unprotect list stored at `.guardy/protected_files.txt`
- ✅ Backup/restore: backup before force update, restore from backup
- ✅ Bootstrap: `SyncManager::bootstrap(repo, version)`
- ✅ Config integration: YAML supported via `GuardyConfig`; parsing with `SyncManager::parse_sync_config()`
- ⚠️ Pattern matching: include/exclude currently matched against file name only (needs path-relative matching)
- ⚠️ Status comparison: size-based; optional hashing not yet implemented
- ⚠️ Parallelization: not yet applied to sync/update or status checks
- ⚠️ Tests: protection unit tests exist; integration tests not finalized
- ⚠️ Docs: user guide and examples pending

## Architecture

- Sync module
  - `SyncManager`: orchestrates bootstrap/update/check/show and file copy
  - `ProtectionManager`: protect/unprotect, backup/restore, validation
- Git module
  - `RemoteOperations` in `git/remote.rs`: clone_or_fetch, checkout_version
- CLI
  - `guardy sync` with subcommands (see Command Interface)
- Config
  - YAML (`guardy.yaml` / `guardy.yml`) via `GuardyConfig` and `parse_sync_config`

## Command Interface

- `guardy sync check` — exit 0 if in sync; exit 1 with changed files listed
- `guardy sync update [--force] [--repo <url> --version <v>]` — regular update or bootstrap
- `guardy sync show` — print current configuration and summary
- `guardy sync unprotect [--all] <files...>` — remove protection
- `guardy sync protected` — list protected files
- `guardy sync restore <backup_path>` — restore from a backup directory

## Configuration (YAML)

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

## Implementation Checklist

- Phase 1: Core sync module
  - [x] `src/sync/mod.rs`: `SyncConfig`, `SyncRepo`, `ProtectionConfig`, `SyncStatus`
  - [x] `src/sync/manager.rs`: `with_config`, `bootstrap`, `check_sync_status`, `update_all_repos`, `show_status`
- Phase 2: Repository operations
  - [x] Remote git ops (`git/remote.rs`): clone/fetch/checkout
  - [x] File copy with include/exclude, backup-before-force
  - [ ] Apply parallel execution for multi-repo and large file sets
- Phase 3: File protection
  - [x] `ProtectionManager`: protect/unprotect/validate + persisted list
  - [x] Backup/restore of overwritten files
- Phase 4: CLI integration
  - [x] `sync.rs` (Check/Update/Show/Unprotect/Protected/Restore) and wiring
- Phase 5: Configuration integration
  - [x] YAML supported and parsed via `GuardyConfig` → `parse_sync_config`
  - [ ] Add schema validation and friendly errors for malformed `sync` config
- Phase 6: Bootstrap
  - [x] Bootstrap flow via `SyncManager::bootstrap()`
- Phase 7: Testing
  - [x] Unit tests: protection manager
  - [ ] Integration tests: bootstrap/update/check with temp repos
  - [ ] Performance checks and error scenarios
- Phase 8: Documentation
  - [ ] `docs/sync.md` with quick start and hook snippets
  - [ ] README updates

## Parallelization Plan (to implement)

Use `parallel::ExecutionStrategy` for:
- Multi-repo update/verification when `repos.len() >= 2`
- Large file verification during `check_sync_status` (e.g., threshold ≥ 20 files)
- Optional file copy concurrency (ensure safe directory creation and error aggregation)

## Quality Improvements (next)

- Pattern matching: switch include/exclude matching to use path relative to `source_path` (currently matches only base name)
- Hash-based comparison: add optional content hashing for more accurate status checks
- Progress: use `parallel::progress::ProgressReporter` during clone/fetch and long file ops
- Dry-run: `guardy sync update --dry-run` to preview changes
- Config validation: detect invalid patterns, missing `repo/version`, helpful messages

## Testing Plan (consolidated)

- Unit tests
  - Protection manager behaviors (done)
  - Sync config parsing and defaults
- Integration tests
  - Bootstrap from a local temp repo (file://)
  - `check` detects changes, respects protection; exit codes
  - `update` preserves local edits unless `--force`; creates backups in force mode
  - `unprotect` allows modification and update overwrites
  - HTTPS path happy-path with a small public repo (guarded by CI condition)
- Scenarios
  - Initial bootstrap
  - Update with local changes, then `--force`
  - Unprotect and restore from backup flows
  - Large file set performance sanity

## Success Criteria

- [x] Bootstrap from external repo
- [x] Check sync status with clear exit codes
- [x] Update from repos with protection and backups
- [x] Show configuration and status
- [x] Protect synced files; list/unprotect available
- [ ] Parallel execution where beneficial
- [ ] Docs and examples complete

## What’s next (implementation order)

1. Path-aware pattern matching
   - Use paths relative to `source_path` for include/exclude (`.github/**` etc.)
   - Add tests for nested directories
2. Parallelize update/check
   - Wrap repo loop in `update_all_repos` with `ExecutionStrategy::auto`
   - Parallelize check file verification with thresholds
3. Add progress reporting
   - Integrate `parallel::progress::ProgressReporter` for clone/fetch and long ops
4. Optional hashing mode
   - Add `--hash` flag or config option to compare file contents via hashes
5. Dry-run and config validation
   - `--dry-run` to preview changes; schema validation with friendly errors
6. Documentation and integration snippets
   - `docs/sync.md` and README updates (Lefthook/Husky/pre-commit snippets)

