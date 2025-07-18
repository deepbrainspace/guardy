# How to Add New Languages to Guardy

Guardy uses a **preset system** to define language-specific formatters and linters. This makes it easy to add support for new languages without modifying the core code.

## 🎯 Overview

**Presets are automatically loaded** - no configuration needed! Guardy comes with built-in presets for popular languages, and you can add custom presets by creating `.yml` files.

## 📁 Built-in vs Custom Presets

Guardy has **two types of presets**:

1. **Built-in presets** (embedded in the binary):
   - Rust, JavaScript/TypeScript, Python, Go
   - Always available, no setup required
   - Updated with new Guardy releases

2. **Custom presets** (user-defined):
   - Create `.yml` files in your config directory
   - Automatically discovered and loaded
   - Perfect for new languages or custom configurations

## 📂 Where to Add Custom Presets

Guardy looks for presets in these locations (in order):

1. **Development**: `./presets/` (for contributing to Guardy)
2. **User config**: `~/.config/guardy/presets/` (your custom presets)
3. **Built-in**: Embedded in the binary (fallback)

### Setting Up Custom Presets

```bash
# Create your presets directory
mkdir -p ~/.config/guardy/presets

# Copy the template
cp TEMPLATE.yml ~/.config/guardy/presets/mylang.yml

# Edit your preset
nano ~/.config/guardy/presets/mylang.yml
```

## 🚀 Quick Start

1. **Copy the template**:
   ```bash
   cp presets/TEMPLATE.yml presets/mylang.yml
   ```

2. **Edit the preset** (see template structure below)

3. **Test it**:
   ```bash
   guardy list  # Should show your new language
   ```

That's it! **No restart or configuration needed** - presets are loaded automatically.

## 📋 Preset Structure

```yaml
# Basic Information
name: "My Language"
description: "Brief description of the language"
file_patterns:
  - "**/*.mylang"      # Files that identify this language
  - "project.toml"     # Project configuration files

# Formatters (in order of preference)
formatters:
  - name: "preferred-formatter"
    command: "myformat --write"
    patterns: ["**/*.mylang"]
    check_command: "myformat --version"
    install:
      cargo: "cargo install myformat"
      npm: "npm install -g myformat"
      brew: "brew install myformat"
      manual: "Installation instructions"
    priority: 1

# Linters (in order of preference)  
linters:
  - name: "my-linter"
    command: "mylint"
    patterns: ["**/*.mylang"]
    check_command: "mylint --version"
    install:
      manual: "Installation instructions"
    priority: 1
```

## 🔧 Installation Methods

Guardy supports multiple installation methods:

| Method | Usage | Example |
|--------|-------|---------|
| `cargo` | Rust packages | `cargo install myformat` |
| `npm` | Node.js packages | `npm install -g myformat` |
| `pip` | Python packages | `pip install myformat` |
| `go` | Go packages | `go install myformat` |
| `brew` | Homebrew | `brew install myformat` |
| `apt` | APT (Debian/Ubuntu) | `apt install myformat` |
| `manual` | **Required** - fallback instructions | `"Download from https://..."` |

**Note**: `manual` is always required as a fallback.

## 🏆 Priority System

Lower numbers = higher priority (tried first):

- **Priority 1**: Most preferred tool
- **Priority 2**: Alternative tool
- **Priority 3+**: Additional alternatives

## 📝 Real Example: Adding Ruby Support

```yaml
# presets/ruby.yml
name: "Ruby"
description: "Ruby programming language"
file_patterns:
  - "**/*.rb"
  - "Gemfile"
  - "Rakefile"

formatters:
  - name: "rubocop"
    command: "rubocop --auto-correct"
    patterns: ["**/*.rb"]
    check_command: "rubocop --version"
    install:
      gem: "gem install rubocop"
      brew: "brew install rubocop"
      manual: "Install Ruby then: gem install rubocop"
    priority: 1

linters:
  - name: "rubocop-lint"
    command: "rubocop"
    patterns: ["**/*.rb"]
    check_command: "rubocop --version"
    install:
      gem: "gem install rubocop"
      manual: "Install Ruby then: gem install rubocop"
    priority: 1
```

## 🧪 Testing Your Preset

1. **Check if it loads**:
   ```bash
   guardy list
   # Should show your language in the output
   ```

2. **Test in a project**:
   ```bash
   # In a project with your language files
   guardy status
   # Should detect your language
   ```

3. **Test initialization**:
   ```bash
   guardy init
   # Should include your formatters in generated config
   ```

## 🎨 Advanced Features

### Multiple File Patterns
```yaml
file_patterns:
  - "**/*.mylang"
  - "**/*.ml"           # Alternative extension
  - "project.toml"
  - "*.config.mylang"
```

### Conditional Commands
```yaml
formatters:
  - name: "formatter-with-options"
    command: "myformat --write --config .myformat.toml"
    patterns: ["**/*.mylang"]
    priority: 1
```

### Language-Specific Patterns
```yaml
formatters:
  - name: "html-formatter"
    command: "prettier --write"
    patterns: 
      - "**/*.html"
      - "**/*.htm"
    priority: 1
  
  - name: "css-formatter"  
    command: "prettier --write"
    patterns: ["**/*.css"]
    priority: 1
```

## 🔍 How Guardy Uses Presets

1. **Auto-Discovery**: Guardy automatically scans `presets/` for `.yml` files
2. **Project Detection**: Uses `file_patterns` to identify project types
3. **Tool Selection**: Follows priority order for formatters/linters
4. **Installation**: Tries installation methods in order until one succeeds

## 📚 Built-in Examples

Study these existing presets for inspiration:

- **`rust.yml`** - Simple language with 2 formatters
- **`javascript.yml`** - Complex language with multiple tools
- **`python.yml`** - Language with many formatter options
- **`go.yml`** - Language with built-in tools

## 🤝 Contributing Presets

If you create a preset for a popular language, consider contributing it back to the project! This helps other users who work with that language.

## 🆘 Troubleshooting

**Preset not loading?**
- Check YAML syntax: `yamllint presets/mylang.yml`
- Ensure file extension is `.yml` or `.yaml`
- Check file permissions

**Language not detected?**
- Verify `file_patterns` match your project files
- Use `guardy status` to see what's detected

**Tools not working?**
- Check `check_command` is correct
- Verify installation instructions in `manual`
- Test tools manually first

## 📖 Next Steps

1. Create your first preset using the template
2. Test it with `guardy list`
3. Use it in a project with `guardy init`
4. Share your preset with the community!

---

**Need help?** Check the existing presets in the `presets/` directory for real examples!