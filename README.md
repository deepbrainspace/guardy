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
