 Implementation Plan for Guardy Hook Enhancements

  Phase 1: Quick Wins from Lefthook (Easy to Implement)

  1. Parallel Execution ⚡

  - Current: Guardy runs hooks sequentially
  - Add: parallel: true option for hooks and individual commands
  - Implementation: Use tokio tasks to run commands concurrently
  - Config:
  hooks:
    pre-commit:
      parallel: true  # Run all commands in parallel
      custom:
        - command: "cargo fmt --check"
        - command: "cargo clippy"

  2. Glob Pattern Support 📁

  - Current: scan_secrets only works on staged files
  - Add: glob and files patterns for filtering
  - Implementation: Use existing glob crate
  - Config:
  hooks:
    pre-commit:
      custom:
        - command: "eslint {files}"
          glob: ["*.js", "*.jsx"]
          files: "git diff --cached --name-only"  # Or custom command

  3. Stage Fixed Files 🔧

  - Current: No auto-staging of fixed files
  - Add: stage_fixed: true to auto-stage corrected files
  - Implementation: Run git add on modified files after command
  - Config:
  custom:
    - command: "cargo fmt"
      stage_fixed: true  # Auto-stage formatted files

  4. Skip Conditions ⏭️

  - Current: Basic skip functionality
  - Add: Conditional skipping based on branch, CI, etc.
  - Config:
  hooks:
    pre-push:
      skip:
        - merge  # Skip on merge commits
        - rebase # Skip during rebase
        - ci     # Skip in CI environment

  5. File Type Filtering 🗂️

  - Current: No file type filtering
  - Add: file_types for extension-based filtering
  - Config:
  custom:
    - command: "prettier --write {files}"
      file_types: [".js", ".ts", ".jsx", ".tsx"]

  Phase 2: Conventional Commits Validation ✅

  Implementation using git-conventional crate:
  // Add to Cargo.toml
  git-conventional = "0.12"

  // In executor.rs
  async fn validate_commit_msg(&self, commit_file: &str) -> Result<()> {
      let msg = std::fs::read_to_string(commit_file)?;
      match git_conventional::Commit::parse(&msg) {
          Ok(commit) => {
              // Optional: Additional validation rules
              if commit.type_() == git_conventional::Type::FEAT && commit.scope().is_none() {
                  return Err(anyhow!("feat commits require a scope"));
              }
              Ok(())
          }
          Err(e) => Err(anyhow!("Invalid conventional commit: {}", e))
      }
  }

  Phase 3: Advanced Features

  6. Priority Execution 🎯

  - Add: priority field to control execution order
  - Config:
  custom:
    - command: "security-scan"
      priority: 1  # Run first
    - command: "format"
      priority: 10  # Run later

  7. Interactive Mode 💬

  - Add: interactive: true for commands needing TTY
  - Config:
  custom:
    - command: "npm test"
      interactive: true  # Keep TTY for interactive tests

  8. Piped Commands 🔗

  - Add: piped: true to pipe output between commands
  - Config:
  hooks:
    pre-commit:
      piped: true  # Pipe output from one command to next

  9. Environment Variables 🌍

  - Add: Per-command environment variables
  - Config:
  custom:
    - command: "npm test"
      env:
        NODE_ENV: "test"
        CI: "true"

  Phase 4: Pre-commit Compatibility Layer 🔄

  To support pre-commit hooks, we'd need:

  1. YAML Parser for .pre-commit-config.yaml:
  struct PreCommitConfig {
      repos: Vec<PreCommitRepo>,
  }

  struct PreCommitRepo {
      repo: String,
      rev: String,
      hooks: Vec<PreCommitHook>,
  }

  2. Hook Repository Management:
  - Clone hook repos to .guardy/pre-commit-repos/
  - Cache by revision
  - Support local repos

  3. Language Runners:
  # Support pre-commit's language system
  - id: black
    language: python
    entry: black
    files: \.py$

  4. Translation Layer:
  fn translate_precommit_to_guardy(config: PreCommitConfig) -> GuardyHookConfig {
      // Convert pre-commit hooks to guardy format
      // Map language -> command runner
      // Convert patterns to glob
  }

  Implementation Complexity Assessment

  Easy (1-2 hours each):
  - ✅ Parallel execution
  - ✅ Glob patterns
  - ✅ Stage fixed
  - ✅ File type filtering
  - ✅ Skip conditions
  - ✅ Conventional commits validation
  - ✅ Priority execution
  - ✅ Environment variables

  Medium (2-4 hours):
  - 🔶 Interactive mode
  - 🔶 Piped commands
  - 🔶 Files command substitution

  Complex (1-2 days):
  - 🔴 Full pre-commit compatibility layer
  - 🔴 Hook repository management
  - 🔴 Multi-language runner support

  Recommended Implementation Order

  1. Immediate: Conventional commits (missing feature)
  2. Quick wins: Parallel, glob, stage_fixed, file_types
  3. Nice to have: Skip conditions, priority, env vars
  4. Future: Pre-commit compatibility layer

  This plan would make guardy a strong competitor to lefthook with better performance (Rust vs Go) and integrated secret scanning. The pre-commit compatibility would be a game-changer but requires significant work.


-------------

 Summary

  I've successfully implemented the three requested features for guardy:

  1. Conventional Commits Validation ✅

  - Added git-conventional crate for parsing and validating commit messages
  - The validate_commit_msg builtin hook now properly validates commit messages against the conventional commits specification
  - Provides helpful error messages when commits don't follow the format
  - Successfully tested with both valid and invalid commit messages

  2. Parallel Execution Support ✅

  - Added parallel: true/false flag to HookDefinition
  - Custom commands can now run concurrently using tokio tasks
  - Refactored to use standalone functions to avoid Sync trait issues
  - Errors are collected from all parallel tasks and reported together

  3. Glob Patterns with all_files Flag ✅

  - Added glob: [] field for file pattern matching using the globset crate
  - Added all_files: true/false flag to process all repository files instead of just staged files
  - When all_files is true, requires glob patterns to be specified
  - Added stage_fixed: true/false to automatically stage modified files after processing
  - File lists are substituted into commands using {files} placeholder

  Key Implementation Details:

  - Default behavior: Hooks run on staged files only (for pre-commit)
  - With glob patterns: Filters the staged files to match patterns
  - With all_files + glob: Searches entire repository for matching files
  - Parallel execution: Commands run concurrently when enabled
  - Error handling: Respects fail_on_error flag per command

  The implementation makes guardy a strong alternative to lefthook with:
  - Better performance (Rust vs Go)
  - Conventional commits validation built-in
  - Compatible configuration structure
  - Flexible file filtering options

