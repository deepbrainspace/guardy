# SuperFigment

**Supercharged configuration management for Rust** - 100% Figment compatible with powerful enhancements.

## ✨ What SuperFigment Provides

### 🔧 Enhanced Providers (Additional superpowers)
- **Universal** - Smart format detection (.toml/.yaml/.json)
- **Nested** - Advanced environment variable parsing with JSON arrays
- **Empty** - Automatic empty value filtering
- **Hierarchical** - Cascading config files across directory hierarchy

### 🚀 Extension Traits (Add methods to regular Figment)
- **ExtendExt** - Array merging with `_add`/`_remove` patterns
- **FluentExt** - Builder methods (`.with_file()`, `.with_env()`, etc.)
- **AccessExt** - Convenience methods (`.as_json()`, `.get_string()`, etc.)

### 💫 SuperFigment Builder (All-in-one solution)
- Built-in methods combining all enhancements
- Zero import complexity for new projects
- Use existing Figment functionalities from within SuperFigment

## 🎯 Quick Start

```rust
use superfigment::SuperFigment;  // Recommended: clean all-in-one API
// or
use superfigment::prelude::*;    // For existing Figment users: add superpowers to current setup
```

## 🔗 100% Figment Compatibility

SuperFigment is fully compatible with existing Figment code:
- All Figment methods work seamlessly 
- Existing Figment configurations can be enhanced without changes
- SuperFigment can be converted to/from regular Figment instances
- No breaking changes to your existing Figment workflow

## Two Ways to Use SuperFigment

Choose the approach that best fits your project:

### Approach A: Enhance Existing Figment Setup (Extension Pattern)

**For teams with existing Figment code** - Add SuperFigment powers without changing your setup:

```rust
use figment::Figment;
use superfigment::prelude::*;  // Everything: traits + providers
use serde::Serialize;

#[derive(Serialize)]
struct Config { name: String }

let cli_args = Config { name: "test".to_string() };

let config = Figment::new()                     // Keep existing Figment code
    .merge(Universal::file("config"))           // Enhanced provider
    .merge_extend(Nested::prefixed("APP_"))     // Extension trait method
    .merge(Empty::new(figment::providers::Serialized::defaults(cli_args))); // Enhanced provider
```

### Approach B: Pure SuperFigment (All-in-One Pattern)

**For new projects or clean rewrites** - Use SuperFigment's fluent builder directly:

```rust
use superfigment::SuperFigment;  // Only import you need
use serde::{Deserialize, Serialize};
// No prelude needed - SuperFigment has built-in methods

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    name: String,
    port: u16,
}

let cli_args = AppConfig {
    name: "myapp".to_string(),
    port: 3000,
};
let args = Some(figment::providers::Serialized::defaults(cli_args));

let config: AppConfig = SuperFigment::new()
    .with_file("config")        // Auto-detects .toml/.json/.yaml
    .with_env("APP_")          // Enhanced env parsing with JSON arrays
    .with_cli_opt(args)        // Filtered CLI args (if Some)
    .extract()?;               // Direct extraction with auto array merging

# Ok::<(), figment::Error>(())
```

## Features

- **Smart Format Detection**: Automatically detect and parse .toml, .yaml, .json files
- **Array Merging**: Merge arrays across configuration sources with `_add` and `_remove` patterns
- **Environment Variable Nesting**: Parse `APP_DATABASE_HOST=localhost` into `database.host`
- **JSON Array Support**: Parse `APP_FEATURES=["auth","cache"]` from environment variables
- **Empty Value Filtering**: Automatically filter out empty strings and arrays
- **Hierarchical Configuration**: Search and merge config files across directory hierarchy
- **100% Figment Compatible**: All existing Figment code works without changes

## Environment Variable Handling

SuperFigment provides two options for environment variable processing:

### Standard Environment Variables
```rust
let config = SuperFigment::new()
    .with_env("APP_");  // Preserves all values, including empty ones
```

### Environment Variables with Empty Filtering
```rust  
let config = SuperFigment::new()
    .with_env_ignore_empty("APP_");  // Filters out empty strings, arrays, objects
```

**When to use each:**
- Use `with_env()` when you want maximum flexibility and explicit empty values
- Use `with_env_ignore_empty()` when you want clean config overrides without empty noise

**Example:**
```bash
# These environment variables:
export APP_DEBUG=""           # Empty string
export APP_HOST="localhost"   # Valid value  
export APP_FEATURES="[]"      # Empty array
```

```rust
// with_env() result:
// { debug: "", host: "localhost", features: [] }

// with_env_ignore_empty() result:  
// { host: "localhost" }  # Empty values filtered out
```

## License

Licensed under the MIT License.