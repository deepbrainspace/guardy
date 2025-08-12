# SuperCLI

[![Crates.io](https://img.shields.io/crates/v/supercli.svg)](https://crates.io/crates/supercli)
[![Documentation](https://docs.rs/supercli/badge.svg)](https://docs.rs/supercli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Universal CLI output wrapper around starbase-styles for consistent CLI theming across tools.

## Overview

SuperCLI wraps [starbase-styles](https://github.com/moonrepo/starbase) to provide consistent, semantic CLI output patterns across all your command-line tools while maintaining full compatibility with the underlying starbase styling system.

## Features

- 🎨 **Semantic CLI Output Macros** - success!, warning!, info!, error!
- 🎯 **Fine-Grained Styling Control** - styled! macro for mixing styles
- 🎛️ **Output Mode Management** - color/monochrome/none with environment variables
- 🌈 **Theme-Aware Output** - automatically adapts to light/dark terminals
- ✅ **100% starbase-styles compatibility** - enhanced convenience methods
- 🚀 **Zero-dependency core** - only requires starbase-styles
- 🔧 **Optional clap integration** - enhanced help styling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
supercli = "0.1.0"

# With clap integration
supercli = { version = "0.1.0", features = ["clap"] }
```

## Quick Start

### Basic Usage

```rust
use supercli::prelude::*;

// Semantic output macros
success!("Operation completed successfully!");
warning!("This action cannot be undone");
info!("Processing files...");
error!("Configuration file not found");

// Fine-grained styling control
styled!("Processing {} files in {}",
    ("150", "number"),
    ("/home/user", "file_path")
);

// Use starbase-styles functions directly
println!("Found {}", file("config.toml"));
```

### Advanced Styling

```rust
use supercli::prelude::*;

// Mix multiple styled components in one line
styled!("{} Found {} secrets in {} files ({})",
    ("🔍", "info_symbol"),
    ("5", "error_count"),
    ("127", "file_count"),
    ("2.3s", "duration")
);

// Chain different style types
styled!("Status: {} | Progress: {} | ETA: {}",
    ("✅ Complete", "success"),
    ("87%", "progress"),
    ("1m 23s", "muted")
);
```

## Environment Control

SuperCLI respects standard environment variables and adds its own:

```bash
# Disable colors completely (NO_COLOR standard)
export NO_COLOR=1

# Force monochrome output
export GUARDY_OUTPUT_STYLE=monochrome

# Disable all output styling
export GUARDY_OUTPUT_STYLE=none

# Force color output (override detection)
export GUARDY_OUTPUT_STYLE=color
```

## Clap Integration

When using the `clap` feature, SuperCLI enhances command-line argument parsing:

```rust
use supercli::clap::create_help_styles;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "my-tool",
    about = "My awesome CLI tool",
    styles = create_help_styles()  // Enhanced help styling
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        info!("Verbose mode enabled");
    }

    styled!("Using config file: {}", (cli.config, "file_path"));
}
```

## Semantic Style Types

SuperCLI supports all starbase-styles semantic types:

### Status Styles
- `success` - Success messages (green)
- `warning` - Warning messages (yellow)
- `error` - Error messages (red)
- `info` - Informational messages (blue)

### Data Styles
- `number` - Numeric values
- `file_path` - File and directory paths
- `url` - Web URLs
- `email` - Email addresses
- `id` - Identifiers and IDs
- `hash` - Hash values
- `token` - Tokens and keys

### UI Styles
- `property` - Property names
- `value` - Property values
- `symbol` - Icons and symbols
- `muted` - Dimmed text
- `highlight` - Highlighted text

## Output Modes

SuperCLI automatically detects the best output mode:

1. **Color Mode**: Full color output when terminal supports it
2. **Monochrome Mode**: Black and white styling for limited terminals
3. **None Mode**: Plain text with no styling

Detection considers:
- Terminal capabilities
- Environment variables (NO_COLOR, TERM, etc.)
- User preferences (GUARDY_OUTPUT_STYLE)

## API Reference

### Macros

#### `success!(message, ...)`
Display success message with checkmark symbol.

```rust
success!("Build completed successfully!");
success!("Processed {} files", 42);
```

#### `warning!(message, ...)`
Display warning message with warning symbol.

```rust
warning!("This action cannot be undone");
warning!("Found {} deprecated functions", count);
```

#### `info!(message, ...)`
Display informational message with info symbol.

```rust
info!("Starting deployment process...");
info!("Using {} workers", num_workers);
```

#### `error!(message, ...)`
Display error message with error symbol.

```rust
error!("Configuration file not found");
error!("Failed to connect to {}", server);
```

#### `styled!(format, (value, style), ...)`
Fine-grained styling control with multiple style parameters.

```rust
styled!("Found {} issues in {} files",
    ("3", "error_count"),
    ("src/main.rs", "file_path")
);
```

## Examples

### CLI Progress Output

```rust
use supercli::prelude::*;

info!("Starting file scan...");

styled!("📁 Scanning {} ({} files)",
    ("src/", "file_path"),
    ("1,247", "number")
);

styled!("🔍 Found {} secrets in {} files",
    ("5", "error_count"),
    ("3", "file_count")
);

warning!("2 files contain potential secrets");
success!("Scan completed in 1.3s");
```

### Configuration Display

```rust
use supercli::prelude::*;

info!("Configuration loaded:");

styled!("  Database: {}", ("postgresql://localhost", "url"));
styled!("  Max connections: {}", ("100", "number"));
styled!("  Debug mode: {}", ("enabled", "success"));
styled!("  Log level: {}", ("info", "property"));
```

### Error Reporting

```rust
use supercli::prelude::*;

error!("Validation failed");

styled!("  File: {}", ("config.yaml:23", "file_path"));
styled!("  Error: {}", ("Invalid email format", "error"));
styled!("  Value: {}", ("not-an-email", "value"));

info!("Tip: Use format user@domain.com");
```

## Integration with Other Tools

SuperCLI works well with popular CLI libraries:

- **clap**: Enhanced help styling (built-in feature)
- **indicatif**: Progress bars with consistent theming
- **dialoguer**: Prompts that match your CLI style
- **console**: Terminal utilities with color coordination

## Performance

SuperCLI is designed for minimal overhead:

- Zero-cost abstractions where possible
- Lazy evaluation of styling
- Minimal allocations
- Fast terminal capability detection

Typical overhead is <1ms per styled output operation.

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Support

- 📚 [Documentation](https://docs.rs/supercli)
- 🐛 [Issues](https://github.com/deepbrainspace/guardy/issues)
- 💬 [Discussions](https://github.com/deepbrainspace/guardy/discussions)
