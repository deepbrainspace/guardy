# Figment Enhanced Providers Implementation Plan

## Overview
Create a reusable `guardy-figment-providers` crate that provides three specialized Figment providers to solve common configuration management challenges:

1. **SmartFormat**: Automatically detects configuration file formats (JSON, TOML, YAML) from content
2. **NonEmpty**: Filters out empty values to prevent CLI overrides from masking config files
3. **SmartEnv**: Enhanced environment variable provider with better prefix/separator handling

## Architecture Decision
Using **Wrapper Provider Pattern** for maximum compatibility and idiomatic Figment usage:
- Each provider implements the `Provider` trait properly
- Drop-in replacement for existing Figment providers
- Maintains type safety and composability
- Can be distributed as standalone crate

## Implementation Plan

### Phase 1: SmartFormat Provider ✅ Completed
- [x] Study existing Figment provider patterns from source code
- [x] Design SmartFormat struct with Provider implementation
- [x] Implement robust content detection algorithm
  - [x] JSON detection: `{...}` and `[...]` patterns
  - [x] YAML detection: `key: value` and `---` separator patterns  
  - [x] TOML detection: `[section]` and `key = value` patterns
  - [x] Handle edge cases and mixed content
- [x] Create comprehensive test suite for format detection
- [x] Implement `SmartFormat::file()` and `SmartFormat::string()` constructors
- [ ] Test integration with Figment merge chains (pending)

### Phase 2: SkipEmpty Provider ✅ Completed
- [x] Analyze CLI override problem in current codebase
- [x] Design SkipEmpty wrapper that filters empty values
- [x] Implement recursive empty value detection
  - [x] Handle empty strings, empty arrays, null values  
  - [x] Preserve intentional empty values where needed (false, 0)
- [x] Create test cases for various empty value scenarios
- [x] Integrate with Figment chain patterns
- [x] Implement proper Provider trait with type conversions
- [x] Create comprehensive doc tests and integration tests

### Phase 3: NestedEnv Provider ✅ Completed
- [x] Study current environment variable mapping in core.rs
- [x] Design enhanced key transformation logic
- [x] Implement smart prefix/separator handling
  - [x] `GUARDY_DB_HOST` → `db.host` mapping
  - [x] Support nested object creation
  - [x] Smart type parsing (bool, int, float, string)
- [x] Create comprehensive environment variable test suite
- [x] Document environment variable conventions with doc tests
- [x] Implement proper Provider trait with error handling
- [x] Support custom separators beyond underscore

### Phase 4: Integration & Testing
- [ ] Update guardy's core.rs to use new providers
- [ ] Replace `super::smart_load::auto()` with `SmartFormat::file()`
- [ ] Replace manual env mapping with `SmartEnv::prefixed()`
- [ ] Replace manual CLI filtering with `NonEmpty::new()`
- [ ] Create integration tests with real config scenarios
- [ ] Test the full configuration loading pipeline

### Phase 5: Documentation & Distribution
- [ ] Write comprehensive crate documentation
- [ ] Create usage examples for each provider
- [ ] Document migration guide from manual approaches
- [ ] Prepare crate for publication to crates.io
- [ ] Version and release the crate

### Phase 6: Cleanup
- [ ] Remove deprecated smart_load.rs module
- [ ] Update guardy to use the published crate version
- [ ] Clean up any remaining manual configuration code

## Success Criteria
- [ ] All three providers work independently and together
- [ ] Drop-in replacement for existing configuration loading
- [ ] No breaking changes to guardy's configuration API
- [ ] Comprehensive test coverage (>90%)
- [ ] Published crate usable by other projects
- [ ] Performance equal or better than current implementation

## Usage Example (Target API)
```rust
use guardy_figment_providers::{SmartFormat, NonEmpty, SmartEnv};
use figment::Figment;

let config = Figment::new()
    .merge(SmartFormat::file("~/.config/app.toml"))    // Auto-detects format
    .merge(SmartFormat::file("./project-config"))      // No extension needed  
    .merge(SmartEnv::prefixed("APP_"))                 // Smart env mapping
    .merge(NonEmpty::new(cli_args))                    // Filtered CLI overrides
    .extract::<AppConfig>()?;
```

## Current Status
- [x] Monorepo structure created
- [x] Basic crate scaffolding in place
- [x] Figment source code analyzed
- [ ] SmartFormat implementation (In Progress)
- [ ] Content detection algorithm
- [ ] Provider trait implementation
- [ ] Test suite creation

## Notes
- Prioritizing SmartFormat as it solves the immediate bug in smart_load.rs
- Using content-based detection rather than file_type crate (better suited for text configs)
- Following standard Figment patterns for maximum compatibility
- Designing for reusability across different projects