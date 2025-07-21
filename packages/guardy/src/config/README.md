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

## Enhanced Figment Providers

Guardy uses custom Figment providers from the `guardy-figment-providers` package for advanced configuration management:

### SmartFormat Provider
- **Auto-detects configuration file formats** from content (JSON, TOML, YAML)
- **No file extensions required** - `guardy` automatically detects format
- **Content-based detection** - examines file structure, not just filename

### SkipEmpty Provider  
- **Filters empty CLI values** to prevent overriding config files
- **Prevents accidental config masking** from empty arrays/strings
- **Smart value filtering** - preserves intentional falsy values (false, 0)

### NestedEnv Provider
- **Enhanced environment variable mapping** with nested object support
- **Automatic key transformation** - `GUARDY_SCANNER_MODE` → `scanner.mode`
- **Smart type parsing** - handles booleans, numbers, strings automatically

## Configuration Sources (Priority Order)

1. **CLI Overrides** (highest) - Filtered through SkipEmpty provider
2. **Environment Variables** - `GUARDY_*` with nested mapping via NestedEnv
3. **Custom Config** - Auto-detected format via SmartFormat provider
4. **Repository Config** - `guardy.{toml,json,yaml,yml}` with auto-detection
5. **User Config** - `~/.config/guardy/config.{toml,json,yaml,yml}` with auto-detection  
6. **Default Config** (lowest) - Embedded `default-config.toml`

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