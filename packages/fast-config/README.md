# fast-config

A high-performance configuration library for Rust applications with intelligent caching and zero-copy access.

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
serde = { version = "1.0", features = ["derive"] }
```

Define your configuration struct:

```rust
use fast_config::static_config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub debug: bool,
    pub features: Features,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Features {
    pub auth: bool,
    pub metrics: bool,
}

// Generate static LazyLock instance (zero-copy access)
static_config!(CONFIG, AppConfig, "myapp");
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

Use throughout your application:

```rust
fn main() {
    // Zero-copy access - sub-microsecond performance
    println!("Server starting on port {}", CONFIG.port);
    println!("Database: {}", CONFIG.database_url);
    
    if CONFIG.features.auth {
        println!("Authentication enabled");
    }
}

// In other modules - zero-copy global access
fn start_server() {
    let port = CONFIG.port;
    // Use port...
}

// Alternative: Load explicitly if you need error handling
fn load_config_explicitly() -> Result<(), Box<dyn std::error::Error>> {
    let config = fast_config::FastConfig::<AppConfig>::load("myapp")?;
    println!("Port: {}", config.get().port);
    Ok(())
}
```

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

## Cache Management

Configs are automatically cached in:
- **Linux/macOS**: `~/.cache/fast-config/{app_name}/`
- **Windows**: `%LOCALAPPDATA%/fast-config/{app_name}/`

Cache is invalidated when:
- Config files are modified (timestamp check)
- Application version changes
- Cache format changes

## License

MIT