# Git Module Organization

This module handles all git repository operations and file discovery. It provides a clean abstraction over git2 operations with clear separation of concerns.

## File Structure & Responsibilities

### Core Files
- **`mod.rs`** - Main `GitRepo` struct and basic repository operations
- **`operations.rs`** - File discovery operations (staged, unstaged, diff files)

### Future Extensions
- **`hooks.rs`** - Git hook installation and management (planned)
- **`commit.rs`** - Commit-related operations (planned)

## Current API

### GitRepo (mod.rs)
- `discover()` - Find git repo from current directory
- `open(path)` - Open git repo at specific path  
- `current_branch()` - Get current branch name
- `workdir()` - Get working directory path

### File Operations (operations.rs)
- `get_staged_files()` - Files staged for commit (pre-commit hook use case)
- `get_unstaged_files()` - Modified but unstaged files
- `get_uncommitted_files()` - All uncommitted files (staged + unstaged)
- `get_diff_files(commit1, commit2)` - Files changed between commits

## When to Add New Code

### Add to `mod.rs` when:
- Adding basic repository operations
- Adding repository discovery logic
- Adding fundamental GitRepo methods

### Add to `operations.rs` when:
- Adding file discovery operations
- Adding status/diff operations
- Adding file listing functionality

### Add to `hooks.rs` (future) when:
- Installing/uninstalling git hooks
- Managing hook configuration
- Hook execution logic

### Add to `commit.rs` (future) when:
- Commit creation/manipulation
- Commit message processing
- Commit history operations

## Design Principles

1. **Pure Git Operations** - No knowledge of scanning or other business logic
2. **Path-Based Results** - Return file paths for other modules to process
3. **Repository Abstraction** - Clean wrapper around git2 complexity
4. **Error Handling** - Proper error context for git operations
5. **Performance** - Efficient file discovery for large repositories

## Integration with Other Modules

The git module should:
- **Provide file lists** to scanner module
- **Handle repository discovery** for CLI commands
- **Manage hooks** for pre-commit integration
- **Stay independent** of business logic (scanning, analysis, etc.)

Other modules should:
- **Use GitRepo for file discovery** instead of implementing git operations
- **Pass file lists to scanner** instead of scanner knowing about git
- **Handle git errors appropriately** with proper user feedback