# fast-config

[![Crates.io](https://img.shields.io/crates/v/fast-config.svg)](https://crates.io/crates/fast-config)
[![Documentation](https://docs.rs/fast-config/badge.svg)](https://docs.rs/fast-config)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance configuration library for Rust applications with intelligent caching and zero-copy access.

Fast-config provides blazing-fast configuration loading with automatic caching, supporting JSON and YAML formats while maintaining type safety through serde integration.

## Features

- 🚀 **Sub-microsecond config access** via `LazyLock` static instances
- 💾 **Intelligent caching** with bincode serialization (~1-3ms cached loads)
- 🔄 **Automatic cache invalidation** based on file timestamps
- 📄 **Multi-format support** for JSON and YAML
- 🏗️ **Procedural macro** for zero-boilerplate setup
- ⚡ **Optional runtime reloading** with feature gates
- 🛡️ **Type-safe** configuration with serde integration

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
fast-config = "0.1"
```

Create your config file (`myapp.json` or `myapp.yaml`):

```json
{
  "database_url": "postgres://localhost/myapp",
  "port": 8080,
  "debug": true,
  "features": {
    "auth": true,
    "metrics": false
  }
}
```

Use the procedural macro to auto-generate everything:

```rust
use fast_config::config;

// Auto-generates struct from myapp.json/yaml and creates LazyLock static
config!("myapp" => MyAppConfig);

fn main() {
    // Zero-copy access - sub-microsecond performance
    let config = MyAppConfig::global();
    println!("Server starting on port {}", config.port);
    println!("Database: {}", config.database_url);
    
    if config.features.auth {
        println!("Authentication enabled");
    }
}

// In other modules - zero-copy global access
fn start_server() {
    let config = MyAppConfig::global();
    let port = config.port;
    // Use port...
}
```

That's it! No manual struct definitions needed - the macro generates everything from your config file.

## Performance

| Load Type | Performance | Use Case |
|-----------|-------------|----------|
| **Static access** | `< 1μs` | Production (after first load) |
| **Cached load** | `1-3ms` | First load after restart |
| **JSON parse** | `~10ms` | Cold load from JSON |
| **YAML parse** | `~30ms` | Cold load from YAML |

## Configuration Search Paths

The library searches for config files in this order:

1. **Current directory**: `myapp.json`, `myapp.yaml`, `myapp.yml`
2. **Git repository root**: `myapp.json`, `myapp.yaml`, `myapp.yml`
3. **Git config directory**: `.config/myapp/config.{json,yaml,yml}`
4. **User config directory**: `~/.config/myapp/config.{json,yaml,yml}`

## Runtime Reloading (Optional)

Enable the `runtime-reload` feature for applications that need to reload config without restart:

```toml
[dependencies]
fast-config = { version = "0.1", features = ["runtime-reload"] }
```

```rust
// With runtime-reload feature
let config = AppConfig::global(); // Returns RwLockReadGuard
println!("Port: {}", config.port);

// Reload from disk
AppConfig::reload()?;
```

**Note**: Runtime reloading adds `RwLock` overhead. For maximum performance, use the default mode without runtime reloading.

## Advanced Usage

### Procedural Macro (Recommended Approach)

The [`config!`] macro automatically generates configuration structs from your config files:

```rust
use fast_config::config;

// Auto-generates complete struct hierarchy from myapp.json/yaml
config!("myapp" => MyAppConfig);

// Zero-copy access throughout your application
let config = MyAppConfig::global();
println!("Port: {}", config.port);
println!("Features: auth={}, metrics={}", config.features.auth, config.features.metrics);
```

**Benefits:**
- No manual struct definitions required
- Automatic type inference from config files
- Zero boilerplate code
- Compile-time validation

### Manual Configuration Loading

For more control over error handling and loading:

```rust
use fast_config::FastConfig;

// Explicit loading with error handling
let config = FastConfig::<AppConfig>::load("myapp")?;
println!("Loaded config: {}", config.name());

// Clone for ownership
let owned_config = config.clone_config();

// Reload from disk (bypasses cache)
let mut config = FastConfig::<AppConfig>::load("myapp")?;
config.reload()?;
```

### Concurrent Access

Fast-config includes high-performance concurrent containers powered by SCC:

```rust
use fast_config::concurrent::{HashMap, HashSet, PATTERN_CACHE};
use regex::Regex;

// Global concurrent pattern cache
let pattern = Regex::new(r"api_\w+").unwrap();
PATTERN_CACHE.insert("api_pattern".to_string(), pattern);

// Fast concurrent access
let matches = PATTERN_CACHE.read(&"api_pattern".to_string(), |_, pattern| {
    pattern.is_match("api_key")
});
```

### Features

- **json**: JSON format support (enabled by default)
- **yaml**: YAML format support (enabled by default) 
- **runtime-reload**: Enable runtime configuration reloading (adds RwLock overhead)

```toml
[dependencies]
# All features enabled
fast-config = { version = "0.1", features = ["json", "yaml", "runtime-reload"] }

# JSON only for maximum performance
fast-config = { version = "0.1", default-features = false, features = ["json"] }
```

## Cache Management

Configs are automatically cached using bincode serialization in:
- **Linux/macOS**: `~/.cache/fast-config/{app_name}/`
- **Windows**: `%LOCALAPPDATA%/fast-config/{app_name}/`

Cache files:
- `config.bin`: Serialized configuration data
- `meta.bin`: Cache metadata (timestamps, version, hash)

Cache is invalidated when:
- Config files are modified (timestamp check)
- Application version changes
- Cache format changes

### Manual Cache Management

```rust
use fast_config::CacheManager;

let cache = CacheManager::new("myapp")?;

// Clear cache
cache.clear_cache()?;

// Check cache directory
println!("Cache dir: {:?}", cache.cache_dir());
```

## Examples

### Web Server Configuration

```rust
use fast_config::static_config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    pub auth: bool,
    pub metrics: bool,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    pub level: String,
    pub structured: bool,
}

// Generate static configuration
static_config!(SERVER_CONFIG, ServerConfig, "server");

fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = &*SERVER_CONFIG;
    
    println!("Starting server on {}:{}", config.host, config.port);
    println!("Database: {}", config.database.url);
    
    if config.features.auth {
        println!("Authentication enabled");
    }
    
    Ok(())
}
```

### CLI Application with Environment Overrides

```rust
use fast_config::{FastConfig, static_config};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub verbose: bool,
    pub output_format: String,
    pub worker_threads: usize,
}

static_config!(CLI_CONFIG, CliConfig, "myapp");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start with file-based config
    let mut config = CLI_CONFIG.clone();
    
    // Override with environment variables
    if let Ok(threads) = env::var("MYAPP_THREADS") {
        config.worker_threads = threads.parse().unwrap_or(config.worker_threads);
    }
    
    if env::var("MYAPP_VERBOSE").is_ok() {
        config.verbose = true;
    }
    
    run_app(&config)
}

fn run_app(config: &CliConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.verbose {
        println!("Running with {} worker threads", config.worker_threads);
        println!("Output format: {}", config.output_format);
    }
    
    Ok(())
}
```

## Best Practices

1. **Use static_config! for globals**: Provides zero-copy access and maximum performance
2. **Prefer JSON over YAML**: JSON parsing is ~3x faster than YAML
3. **Keep config structs simple**: Avoid complex nested enums that hurt cache performance
4. **Use Default derive**: Enables graceful fallback when config files are missing
5. **Validate config on startup**: Use custom validation after loading
6. **Group related settings**: Organize config into logical sub-structs

## Performance Tips

- Use `LazyLock` static instances for zero-copy access after first load
- JSON format is faster than YAML for parsing
- Cache invalidation is cheap (timestamp comparison)
- Bincode serialization provides 5-10x faster loads than JSON/YAML parsing
- SCC concurrent containers outperform standard HashMap/HashSet in multi-threaded scenarios

## Error Handling

```rust
use fast_config::{FastConfig, Error};

fn load_config() -> Result<(), Error> {
    match FastConfig::<AppConfig>::load("myapp") {
        Ok(config) => {
            println!("Loaded config: {}", config.name());
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            // Fall back to defaults
            let default_config = AppConfig::default();
            println!("Using default config");
            Ok(())
        }
    }
}
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.