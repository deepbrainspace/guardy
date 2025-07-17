# guardy
Guardy is a Git Workflow Management and Automation Tool written in Rust.

## Quick Setup

1. **Install Guardy**:
   ```bash
   cargo install guardy
   ```

2. **Initialize in your project**:
   ```bash
   guardy init
   ```

3. **Configuration**:
   Guardy uses `guardy.yml` or `.guardy.yml` for configuration.

## Global Options

Guardy supports several global flags that work with all commands:

### Output Control
- `--verbose, -v`: Show detailed output with extra information
- `--quiet, -q`: Minimal output (only errors and essential results)
- `--format <FORMAT>`: Output format (text, json, yaml) - default: text

### Behavior Control
- `--force, -f`: Skip confirmations and overwrite without prompting
- `--dry-run`: Show what would be done without executing
- `--auto-install`: Automatically install missing tools instead of failing
- `--config, -c <FILE>`: Use custom configuration file path

### Examples
```bash
# Verbose security scan
guardy --verbose security scan

# Quiet operation for scripts
guardy --quiet security scan

# Force install hooks without prompts
guardy --force hooks install

# JSON output for processing
guardy --format json security scan

# Dry run to see what would happen
guardy --dry-run init
```

## File Exclusions

Guardy respects multiple exclusion mechanisms:

- **`.guardyignore`**: Guardy-specific exclusions (always loaded)
- **`.gitignore`**: Git exclusions (when `use_gitignore: true`)
- **Configuration**: Manual patterns in `guardy.yml`

### .guardyignore File

Create a `.guardyignore` file to exclude files that contain test secrets or sensitive content:

```
# Testing files
TESTING.md
tests/
*.test.rs
test_*

# Documentation
docs/
*.example
README*
```

This ensures test files with intentional secrets are not flagged during security scans.

## Commands

### Security Scanning
```bash
# Scan current directory
guardy security scan

# Scan specific files (note: use -i for input files)
guardy security scan -i src/main.rs src/lib.rs

# Scan specific directory
guardy security scan -d src/

# Scan with verbose output
guardy --verbose security scan

# Scan with JSON output
guardy --format json security scan
```

### Git Hooks Management
```bash
# Install hooks
guardy hooks install

# Force install (overwrite existing)
guardy --force hooks install

# List available hooks
guardy hooks list

# Remove hooks
guardy hooks remove
```

### Configuration
```bash
# Initialize configuration
guardy config init

# Show current configuration
guardy config show

# Validate configuration
guardy config validate
```
