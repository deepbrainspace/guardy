# Config Module

The configuration system for Guardy using Figment for flexible, multi-source configuration management.

## Architecture

```
src/config/
├── mod.rs          # Module routing and re-exports only
├── core.rs         # GuardyConfig struct and core loading logic
├── formats.rs      # ConfigFormat enum and export functionality  
├── languages.rs    # Language detection for project types
└── README.md       # This documentation
```

## Files and Responsibilities

### `core.rs`
- **Purpose**: Core configuration loading and data access
- **Contains**: `GuardyConfig` struct, loading methods, getter methods
- **Tests**: Configuration loading, environment variables, default values

### `formats.rs` 
- **Purpose**: Configuration export and formatting
- **Contains**: `ConfigFormat` enum, export methods, syntax highlighting
- **Tests**: Export functionality, format conversion, syntax highlighting

### `languages.rs`
- **Purpose**: Project language detection logic
- **Contains**: Language detection utilities for project types
- **Tests**: Language detection accuracy and coverage

### `mod.rs`
- **Purpose**: Module organization only
- **Contains**: Module declarations and re-exports
- **Tests**: None (routing only)

## Test Organization Guidelines

**✅ DO:**
- Put tests inline with `#[cfg(test)] mod tests` in each implementation file
- Test the specific functionality in the same file where it's implemented
- Keep tests focused on the module's specific responsibilities

**❌ DON'T:**
- Put tests in `mod.rs` (routing only)
- Create separate `tests.rs` files (use inline tests)
- Mix tests from different modules in one file

## Configuration Sources (Priority Order)

1. **Environment Variables** (highest) - `GUARDY_*`
2. **Repository Config** - `guardy.{toml,json,yaml,yml}`
3. **User Config** - `~/.config/guardy/config.{toml,json,yaml,yml}`
4. **Default Config** (lowest) - Embedded `default-config.toml`

## Usage Examples

```rust
use crate::config::{GuardyConfig, ConfigFormat};

// Load configuration
let config = GuardyConfig::load()?;

// Access values
let debug_mode = config.get_bool("general.debug")?;
let port = config.get_u16("mcp.port")?;

// Export configuration
let json_output = config.export_config(ConfigFormat::Json)?;
```