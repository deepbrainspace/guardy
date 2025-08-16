# Plan 018: SuperConfig Incremental Implementation

## Objective
Implement SuperConfig integration with Guardy in small, testable phases. Each phase must be fully working before moving to the next.

## Guiding Principles
1. **Remove all non-functional code** - No placeholders, no TODOs in shipped code
2. **Each phase must pass clippy and tests** before moving forward
3. **Guardy scan must work** at the end of each phase
4. **Small, verifiable steps** - Better to have less that works than more that doesn't

## Phase 1: Minimal Working ConfigBuilder with Defaults Only (1 hour)

### Goal
Make ConfigBuilder work with ONLY defaults. No file loading, no env vars, no CLI. Just defaults.

### Tasks
1. **Clean up superconfig crate** ⏳ IN PROGRESS
   - Remove all references to PartialConfig (not implementing yet)
   - Remove schema.rs (placeholder with todo!())
   - Remove cache.rs if not used
   - Fix ConfigBuilder::build() to return defaults properly
   - Remove all broken tests in superconfig

2. **Fix ConfigBuilder implementation**
   ```rust
   // packages/superconfig/src/builder.rs
   pub fn build(self) -> Result<T> {
       Ok(self.defaults.unwrap_or_else(T::default))
   }
   ```

3. **Clean up superconfig-macros**
   - Remove unused variables (env_prefix, content)
   - Simplify config! macro to just generate empty struct with Default
   - Remove config_builder! macro entirely for now

4. **Integration with Guardy**
   - Create GuardyConfig with proper Default implementation
   - Use ConfigBuilder in guardy/src/config/mod.rs
   - Ensure `guardy scan` works with default config

### Validation
- [ ] cargo clippy --all -- -D warnings (passes)
- [ ] cargo test (all passing tests)
- [ ] guardy scan (works with defaults)

## Phase 2: Add Config File Support (1 hour)

### Goal
ConfigBuilder can load from a config file if it exists, otherwise use defaults.

### Tasks
1. **Enhance ConfigBuilder**
   ```rust
   pub fn with_config_file(mut self, name: &str) -> Self {
       // Try to load, if fails, just keep defaults
       if let Ok(config) = Config::<T>::load(name) {
           self.file_config = Some(config.into_inner());
       }
       self
   }
   
   pub fn build(self) -> Result<T> {
       let mut config = self.defaults.unwrap_or_else(T::default);
       if let Some(file_config) = self.file_config {
           config = file_config; // For now, just replace. Later we'll merge.
       }
       Ok(config)
   }
   ```

2. **Add Config::into_inner() method**
   ```rust
   pub fn into_inner(self) -> T {
       self.config
   }
   ```

3. **Update Guardy integration**
   - Look for guardy.json/yaml in current directory
   - If found, use it; if not, use defaults
   - Test with sample guardy.json file

### Validation
- [ ] cargo clippy passes
- [ ] guardy scan works without config file
- [ ] guardy scan uses config file when present

## Phase 3: Add CLI Override Support (1 hour)

### Goal
Support basic CLI overrides like --debug, --max-threads without complex PartialConfig.

### Tasks
1. **Simple CLI override approach**
   ```rust
   pub struct SimpleOverrides {
       pub debug: Option<bool>,
       pub max_threads: Option<u16>,
       // Add fields as needed
   }
   
   impl<T> ConfigBuilder<T> {
       pub fn with_cli_overrides(mut self, overrides: SimpleOverrides) -> Self {
           self.cli_overrides = Some(overrides);
           self
       }
       
       pub fn build(self) -> Result<T> {
           let mut config = /* ... load from defaults/file ... */;
           
           // Apply simple overrides manually for now
           if let Some(overrides) = self.cli_overrides {
               // This will be type-specific initially
               // We'll generalize later with PartialConfig
           }
           
           Ok(config)
       }
   }
   ```

2. **Update Guardy CLI**
   - Parse CLI args into SimpleOverrides
   - Pass to ConfigBuilder
   - Test with `guardy scan --debug --max-threads 4`

### Validation
- [ ] CLI overrides work for basic flags
- [ ] Config precedence: CLI > File > Defaults

## Phase 4: Add Environment Variable Support (1 hour)

### Goal
Support GUARDY_* environment variables for configuration.

### Tasks
1. **Add env var scanning**
   ```rust
   pub fn with_env_prefix(mut self, prefix: &str) -> Self {
       let mut env_overrides = SimpleOverrides::default();
       
       // Scan for GUARDY_DEBUG, GUARDY_MAX_THREADS, etc.
       if let Ok(val) = std::env::var(format!("{}_DEBUG", prefix)) {
           env_overrides.debug = Some(val.parse().unwrap_or(false));
       }
       // ... more fields
       
       self.env_overrides = Some(env_overrides);
       self
   }
   ```

2. **Update precedence**
   - CLI > Env > File > Defaults
   - Test with GUARDY_DEBUG=true guardy scan

### Validation
- [ ] Env vars are recognized
- [ ] Proper precedence order
- [ ] All tests pass

## Phase 5: Implement PartialConfig (Future)

### Goal
Generic override system using dot notation paths.

### Tasks
- Implement partial.rs with proper serde_json manipulation
- Replace SimpleOverrides with PartialConfig
- Support complex paths like "scanner.hot.max_threads"

### Validation
- [ ] Complex nested overrides work
- [ ] Array extension works
- [ ] Type auto-detection works

## Success Criteria for Each Phase

1. **No broken code** - Everything that exists must work
2. **Clippy clean** - No warnings
3. **Tests pass** - Only include tests for working features
4. **Guardy scan works** - Primary use case must not break
5. **Clear documentation** - Each phase documents what works and what doesn't

## Implementation Order

1. Phase 1: Get basics working (TODAY)
2. Phase 2: Add file support (After Phase 1 validated)
3. Phase 3: Add CLI support (After Phase 2 validated)
4. Phase 4: Add env support (After Phase 3 validated)
5. Phase 5: Full PartialConfig (Future enhancement)

## Notes

- **NO PLACEHOLDERS** - If it doesn't work, remove it
- **NO BROKEN TESTS** - Only test what actually works
- **INCREMENTAL** - Each phase builds on the last
- **VALIDATED** - Don't move forward until current phase is solid

This approach ensures we always have working code and can make steady progress without getting stuck on complex features.

## Progress Tracking

### Phase 1 Status: IN PROGRESS
- [x] Plan created
- [ ] superconfig cleaned up
- [ ] ConfigBuilder fixed
- [ ] superconfig-macros cleaned
- [ ] Tests removed/fixed
- [ ] Guardy integration
- [ ] Clippy passes
- [ ] guardy scan works