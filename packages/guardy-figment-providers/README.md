# Guardy Figment Providers

Custom Figment providers for enhanced configuration management in Rust applications.

## Overview

This package provides three specialized Figment providers that solve common configuration challenges:

1. **SmartFormat** - Auto-detects configuration file formats (JSON, TOML, YAML)
2. **SkipEmpty** - Filters empty values to prevent CLI overrides from masking config files
3. **NestedEnv** - Enhanced environment variable provider with nested object support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
guardy-figment-providers = { path = "../guardy-figment-providers" }
figment = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

## Usage

```rust
use figment::Figment;
use guardy_figment_providers::{SmartFormat, SkipEmpty, NestedEnv};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    app: AppConfig,
    database: DatabaseConfig,
}

// Load configuration with all three providers
let config: Config = Figment::new()
    // 1. Load from auto-detected format files
    .merge(SmartFormat::file("~/.config/app"))        // No extension needed!
    .merge(SmartFormat::file("./app-config"))         // Detects JSON/YAML/TOML from content
    
    // 2. Environment variables with nested support  
    .merge(NestedEnv::prefixed("APP_"))               // APP_DATABASE_HOST → database.host
    
    // 3. CLI overrides with empty value filtering
    .merge(SkipEmpty::new(cli_args))                  // Filters out empty arrays/strings
    .extract()?;
```

## Providers

### SmartFormat

Auto-detects configuration file formats from content, not file extensions.

```rust
use guardy_figment_providers::SmartFormat;

// These all work without specifying extensions
let config = Figment::new()
    .merge(SmartFormat::file("config"))           // Detects format from content
    .merge(SmartFormat::file("app-settings"))     // JSON, YAML, or TOML - auto-detected
    .merge(SmartFormat::string(&content));        // Works with string content too
```

**Detection Logic:**
- **JSON**: Starts with `{` or `[`, ends with `}` or `]`  
- **YAML**: Contains `---` or `key: value` patterns
- **TOML**: Contains `[section]` or `key = value` patterns

### SkipEmpty

Filters out empty values from CLI overrides to prevent config masking.

```rust
use guardy_figment_providers::SkipEmpty;

// Without SkipEmpty: empty CLI arrays override config files
let bad_config = Figment::new()
    .merge(Toml::file("config.toml"))           // ignore_patterns = ["test", "demo"]
    .merge(Serialized::defaults(cli_args));     // ignore_patterns = [] (empty array!)
    // Result: ignore_patterns = [] (config overridden!)

// With SkipEmpty: empty values are filtered out
let good_config = Figment::new()
    .merge(Toml::file("config.toml"))           // ignore_patterns = ["test", "demo"] 
    .merge(SkipEmpty::new(cli_args));           // Empty arrays filtered out
    // Result: ignore_patterns = ["test", "demo"] (config preserved!)
```

**Filters out:**
- Empty strings (`""`)
- Empty arrays (`[]`)
- Null values
- Nested empty structures

**Preserves:**
- Intentional falsy values (`false`, `0`)
- Non-empty strings and arrays
- Valid nested objects

### NestedEnv

Enhanced environment variable provider with automatic nested object creation.

```rust
use guardy_figment_providers::NestedEnv;

// Set environment variables:
// APP_DATABASE_HOST=localhost  
// APP_DATABASE_PORT=5432
// APP_SCANNER_MODE=parallel

let config = Figment::new()
    .merge(NestedEnv::prefixed("APP_"));

// Results in nested structure:
// {
//   "database": {
//     "host": "localhost",
//     "port": 5432
//   },
//   "scanner": {
//     "mode": "parallel"  
//   }
// }
```

**Features:**
- **Automatic nesting**: `PREFIX_KEY_SUBKEY` → `key.subkey`
- **Smart type parsing**: Detects booleans, numbers, strings
- **Custom separators**: Configure beyond underscore if needed
- **Prefix filtering**: Only processes variables with specified prefix

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific provider
cargo test smart_format
cargo test skip_empty  
cargo test nested_env
```

## Integration Examples

### With Serde and Complex Types

```rust
#[derive(Deserialize)]
struct AppConfig {
    database: DatabaseConfig,
    scanner: ScannerConfig, 
    features: Vec<String>,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    ssl: bool,
}

// Environment: APP_DATABASE_HOST=localhost, APP_DATABASE_PORT=5432, APP_DATABASE_SSL=true
// CLI: --features=[] (empty array that should be filtered)
// Config file: features = ["auth", "cache"] 

let config: AppConfig = Figment::new()
    .merge(SmartFormat::file("app.toml"))               // features = ["auth", "cache"]
    .merge(NestedEnv::prefixed("APP_"))                 // database = {host, port, ssl}  
    .merge(SkipEmpty::new(cli_overrides))               // Empty features array filtered
    .extract()?;

// Result: 
// - database.host = "localhost" (from env)
// - database.port = 5432 (from env)  
// - database.ssl = true (from env)
// - features = ["auth", "cache"] (from file, CLI empty array ignored)
```

### With Custom Configuration Layers

```rust
// Complex configuration pipeline
let config = Figment::new()
    // 1. Defaults
    .merge(Toml::string(DEFAULT_CONFIG))
    
    // 2. System config (auto-format detection)
    .merge(SmartFormat::file("/etc/myapp/config"))
    
    // 3. User config (auto-format detection)  
    .merge(SmartFormat::file("~/.config/myapp/config"))
    
    // 4. Environment variables (nested)
    .merge(NestedEnv::prefixed("MYAPP_"))
    
    // 5. CLI overrides (filtered)
    .merge(SkipEmpty::new(cli_args))
    
    .extract::<MyConfig>()?;
```

## Error Handling

All providers implement proper Figment error handling:

```rust
match figment.extract::<Config>() {
    Ok(config) => println!("Config loaded successfully"),
    Err(e) => {
        eprintln!("Configuration error: {}", e);
        eprintln!("Kind: {:?}", e.kind);
        eprintln!("Path: {:?}", e.path); 
        eprintln!("Profile: {:?}", e.profile);
    }
}
```

## Performance

- **SmartFormat**: Content detection is fast (single pass)
- **SkipEmpty**: Minimal overhead (only processes during merge)
- **NestedEnv**: Efficient key transformation with caching

All providers are designed for zero-allocation where possible and minimal performance impact.

## License

MIT