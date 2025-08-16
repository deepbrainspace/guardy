# Plan 21: Config Module Integration Fixes

## Analysis Summary

After investigating the config module integration issues, the problems are much simpler than initially thought. The new optimized config module uses static `CONFIG` (LazyLock) but code is still trying to use old methods that don't exist.

**Key Insight**: Most CLI config commands are unnecessary since static CONFIG with serde handles everything automatically.

## Issues Found

### 1. Missing Methods (11 call sites)
- `GuardyConfig::load()` - Used in 11 places where `CONFIG` direct access should be used
- `config.get_section()` - Used in 5 places where `CONFIG.section` should be used

### 2. Serde Arc<Vec<T>> Issues  
- `Arc<Vec<String>>` fields need `"rc"` feature flag in serde
- Simple fix: Add to workspace Cargo.toml

### 3. Type Mismatches
- `config.max_threads` is `u16` but some functions expect `usize`

### 4. Unnecessary CLI Commands
- **Get**: Not needed - use `CONFIG.section.field` directly in code
- **Set**: Not needed - done at initialization via CLI args/config files  
- **Init**: Not needed - default config embedded in code
- **Validate**: Not needed - serde validates automatically
- **Show**: Simplify to basic serialization + optional highlighting

## Fix Strategy

### Phase 1: Add Serde RC Feature
```toml
# Cargo.toml (workspace)
serde = { version = "1.0.219", features = ["derive", "rc"] }
```

### Phase 2: Replace Method Calls with Direct Access

#### Replace `GuardyConfig::load()` calls:
```rust
// OLD: let config = GuardyConfig::load(...)?;
// NEW: // Just use CONFIG directly
```

#### Replace `config.get_section()` calls:
```rust
// OLD: config.get_section("hooks")?
// NEW: &CONFIG.hooks

// OLD: config.get_section("sync")?  
// NEW: &CONFIG.sync
```

### Phase 3: Simplify CLI Config Commands

Remove unnecessary subcommands, keep only Show:
```rust
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Display current merged configuration  
    Show {
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
}

// Implementation:
match format {
    "json" => println!("{}", serde_json::to_string_pretty(&*CONFIG)?),
    "yaml" => println!("{}", serde_yml::to_string(&*CONFIG)?),
    "toml" => println!("{}", toml::to_string_pretty(&*CONFIG)?),
}
```

### Phase 4: Fix Type Issues
```rust
// OLD: config.max_threads  
// NEW: CONFIG.scanner.max_threads as usize
```

## Files to Modify

### Core Fixes:
1. `Cargo.toml` - Add "rc" feature
2. `src/cli/commands/config.rs` - Simplify to Show only
3. `src/profiling.rs` - Fix type casting

### Replace load() calls:
4. `src/cli/commands/run.rs`
5. `src/cli/commands/install.rs` 
6. `src/cli/commands/status.rs`
7. `src/cli/commands/sync.rs`

### Replace get_section() calls:
8. `src/hooks/executor.rs`
9. `src/sync/manager.rs`
10. `src/sync/sync_backup/manager.rs`

### Fix tests:
11. `src/config/formats.rs`

### Cleanup:
12. `src/scan/types.rs` - Remove unused import

## Benefits

✅ **Massive Simplification**: Remove complex CLI config logic  
✅ **Zero-Copy Performance**: Direct field access instead of method calls  
✅ **Type Safety**: Compile-time checked field access  
✅ **Consistency**: All code uses static CONFIG the same way  
✅ **Less Code**: Remove hundreds of lines of unnecessary CLI logic  

## Implementation Order

1. Add serde "rc" feature
2. Fix the 16 method call sites  
3. Simplify config CLI to Show only
4. Fix type casting issues
5. Test and verify all modules work

This approach makes the config system much cleaner and more performant by embracing the static CONFIG design.