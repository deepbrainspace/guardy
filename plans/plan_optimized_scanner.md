# Plan: Optimized Scanner Implementation (scan2)

## 📋 Overview

This plan outlines the implementation of a next-generation scanner (`scan2`) based on Aho-Corasick + keyword prefiltering strategy, inspired by Gitleaks' proven approach. The goal is to achieve **~5x performance improvement** while maintaining comprehensive pattern coverage.

## 🎯 Goals

1. **Performance**: ~5x faster than current scanner through Aho-Corasick prefiltering
2. **Coverage**: Match or exceed current pattern detection capabilities  
3. **Modularity**: Clean, well-structured, maintainable codebase
4. **Compatibility**: Seamless integration with existing config and CLI
5. **Validation**: Comprehensive testing and benchmarking vs current implementation

## 🏗️ Architecture Overview

### High-Level Design

```mermaid
graph TD
    A[File Input] --> B{Path Ignore Check}
    B -->|Ignored| Z[Skip File]
    B -->|Not Ignored| C{File Size Check}
    C -->|"&gt; 50MB (configurable)"| Z[Skip File]
    C -->|"≤ 50MB"| D{Binary Detection}
    D -->|Binary File| Z
    D -->|Text File| G[Load Full Content]
    G --> H[Keyword Prefilter using Aho-Corasick]
    H -->|No Keywords| Z[Skip File]
    H -->|Keywords Found| I[Regex Pattern Matching on Filtered Patterns]
    I --> J[Collect Matches]
    J --> K[Drop matches with 'guardy:allow' comment]
    K --> L[Final Results]
    
    style A fill:#e1f5fe
    style D fill:#f3e5f5
    style G fill:#fff3e0
    style H fill:#e8f5e8
    style I fill:#fff3e0
    style J fill:#ffebee
    style K fill:#e8f5e8
```

### Module Structure
```
src/
├── scanner/                 # Legacy scanner (preserve until migration complete)
│   ├── mod.rs              # Existing scanner interface  
│   ├── core.rs             # Current scanner implementation
│   └── ...                 # All existing scanner modules
├── scan/                   # New optimized scanner architecture
│   ├── mod.rs              # Public API exports
│   ├── core.rs             # Main scanner orchestrator
│   ├── prefilter.rs        # Aho-Corasick keyword filtering
│   ├── patterns/
│   │   ├── mod.rs          # Pattern library management
│   │   ├── classification.rs # Smart pattern categorization
│   │   ├── gitleaks.rs     # Imported Gitleaks patterns (150+ total)
│   │   └── guardy.rs       # Migrated Guardy patterns (40+)
│   ├── engine.rs           # Single-pass pattern matching engine
│   ├── config.rs           # Configuration & CLI integration
│   ├── entropy.rs          # Advanced entropy analysis (migrated)
│   ├── file_handler.rs     # Simple whole-file loading (from core.rs)
│   ├── binary.rs           # Binary file detection (from directory.rs) 
│   ├── directory.rs        # Directory orchestration & walking (from directory.rs)
│   ├── ignore.rs           # Simplified inline ignore system
│   └── types.rs            # All data structures and types
```

### CLI Integration Strategy
- **Phase 1**: Build `guardy scan2` subcommand with clean, modern architecture
- **Phase 2**: After validation, replace `guardy scan` with the new implementation
- **Phase 3**: Remove legacy `scanner/` module entirely

## 🔍 Critical Legacy Functionality Analysis

### **Must-Preserve Components from Existing Scanner**

#### 1. **Advanced Entropy Analysis** (entropy.rs)
- **Multi-metric Statistical Analysis**: Combines distinct values, character class distribution, and bigram frequency analysis
- **Base Detection**: Hex (16), alphanumeric (36), full base64 (64) with probability calculations
- **Performance**: Memoization with `memoize` crate, precompiled regex patterns
- **Tuning**: Configurable thresholds (default: 1.0/1e5), numbers requirement heuristics
- **🚨 CRITICAL**: Extensively tested with real-world data, must preserve exact logic

#### 1.1. **Enhanced File Size Configuration** (UPDATED)
- **Default Maximum File Size**: 50MB
- **Override Capability**: `--max-file-size-mb` flag allows per-scan customization
- **Modern Development**: Accommodates larger bundle files, generated code, and data files

#### 2. **Comprehensive Pattern Library** (patterns.rs)
- **40+ Production-Ready Patterns**: Private keys, cloud credentials, API tokens, AI services
- **Modern Coverage**: 2024-2025 AI services (Claude, OpenAI, Hugging Face, Cohere, etc.)
- **Capture Groups**: Support for extracting specific secret content from matches
- **Custom Pattern Support**: Error handling for user-defined regex patterns from config
- **🚨 CRITICAL**: Each pattern represents extensive real-world testing

#### 3. **Clean Ignore System**
- **Path-based**: GlobSet patterns for entire files/directories  
- **Pattern-based**: Line content patterns (DEMO_KEY_, FAKE_, etc.)
- **Inline directives**: Single `guardy:allow` directive like Gitleaks
- **🚨 CRITICAL**: Simple, efficient ignore system optimized for performance

#### 4. **Binary File Detection** (binary.rs)
- **Dual-mode Detection**: Extension-based (279 extensions!) + content inspection
- **Content Inspector Integration**: Uses `content_inspector` crate for accuracy
- **Memory Efficiency**: Avoids loading large binary files unnecessarily
- **🚨 CRITICAL**: Prevents scanning of images, compiled code, compressed files

#### 6. **Configuration System Integration** (core.rs)
- **SuperConfig Integration**: YAML/TOML/JSON support with complex merging
- **CLI Override Support**: Command-line arguments override config file values
- **Environment Variables**: Runtime configuration override support
- **Debug Tracing**: Comprehensive logging for configuration troubleshooting
- **🚨 CRITICAL**: Complex config merging logic handles edge cases

#### 7. **File Processing Engine** (core.rs)
- **Streaming Support**: Large files (>5MB) handled without loading fully into memory
- **Error Recovery**: Graceful handling of permission errors, unreadable files
- **UTF-8 Handling**: Robust text processing with fallback strategies
- **Performance**: OS cache optimization (2.7x speedup on warm caches)
- **🚨 CRITICAL**: Production-tested with 100k+ file repositories

#### 8. **Parallel Integration** (directory.rs)
- **Resource Calculation**: CPU core detection, thread percentage application  
- **Performance-First Approach**: Maximum worker utilization regardless of file count
- **Progress Reporting**: Worker-specific progress with strategy icons (⏳ sequential, ⚡ parallel)
- **🚨 CRITICAL**: Tight integration with existing parallel execution framework

## 📊 Technical Implementation Plan

### Phase 1: Foundation & Data Structure Migration (Day 1)

#### Task 1.1: Data Structure Migration
- **File**: `src/scan/types.rs`
- **Purpose**: Exact copy of existing type system to ensure compatibility
- **Source**: Migrate from `src/scanner/types.rs` preserving all fields
- **Critical Types**:
  ```rust
  // SecretMatch stores metadata about found secrets in files
  // All fields needed for precise location tracking and reporting
  pub struct SecretMatch {
      pub file_path: PathBuf,        // File containing the secret
      pub line_number: usize,        // Line number in file (1-based)
      pub line_content: String,      // Full line content for context
      pub matched_text: String,      // Exact text that matched pattern
      pub start_position: usize,     // Character position within line
      pub end_position: usize,       // End character position within line
      pub secret_type: String,       // Type of secret (API key, token, etc.)
      pub pattern_description: String, // Human-readable pattern description
  }
  
  pub struct ScannerConfig {
      // ALL 20+ existing configuration fields
      pub enable_entropy_analysis: bool,
      pub min_entropy_threshold: f64,
      pub include_binary: bool,
      pub max_file_size_mb: usize,        // UPDATED: 50MB default (was 10MB)
      pub streaming_threshold_mb: usize,   // NEW: 20MB default (was hardcoded 5MB)
      pub ignore_patterns: Vec<String>,
      pub ignore_paths: Vec<String>,
      pub ignore_comments: Vec<String>,
      // ... preserve ALL existing fields
  }
  ```

#### Task 1.2: Entropy Analysis Migration
- **File**: `src/scan/entropy.rs`
- **Purpose**: Exact migration of proven entropy analysis system
- **Source**: Direct copy from `src/scanner/entropy.rs`
- **Key Features**:
  - **EXACT preservation**: All 3 statistical methods, 488 bigram patterns
  - **Memoization**: Keep `memoize` performance optimizations
  - **Probability calculations**: Preserve binomial distribution logic
  - **🚨 WARNING**: Do NOT modify algorithms - they are production-tested

#### Task 1.3: File Processing  
- **File**: `src/scan/file_handler.rs`
- **Purpose**: Whole-file content loading with size limits
- **Source**: Extract from `src/scanner/core.rs` (size checks, content loading)
- **Key Features**:
  ```rust
  pub struct FileProcessor {
      max_file_size_mb: usize,      // Default: 50MB (configurable via CLI --max-file-size-mb)
  }
  
  impl FileProcessor {
      pub fn load_file_content(&self, path: &Path) -> Result<String>
      pub fn is_size_allowed(&self, path: &Path) -> bool  // Check against max_file_size_mb
      pub fn get_file_size(&self, path: &Path) -> Result<u64>
  }
  ```

#### Task 1.4: Binary File Detection
- **File**: `src/scan/binary.rs`  
- **Purpose**: Fast and accurate binary file detection
- **Source**: Extract from `src/scanner/directory.rs` (lines 9-48)
- **Key Features**:
  ```rust
  pub struct BinaryDetector {
      binary_extensions: HashSet<String>, // 279 extensions
  }
  
  impl BinaryDetector {
      pub fn is_binary_file(&self, path: &Path) -> bool
      pub fn is_binary_by_extension(&self, path: &Path) -> bool  
      pub fn is_binary_by_content(&self, path: &Path) -> bool
  }
  ```

#### Task 1.5: Directory Orchestration with Performance-First Parallel Processing
- **File**: `src/scan/directory.rs`
- **Purpose**: High-level directory walking and scan orchestration with optimized parallel execution  
- **Source**: Extract from `src/scanner/directory.rs` (DirectoryHandler + orchestration logic)
- **Worker Allocation Strategy**:
  ```rust
  use crate::parallel::{ExecutionStrategy, progress::factories};
  
  pub struct ScanOrchestrator {
      binary_detector: BinaryDetector,
      file_processor: FileProcessor,  
      ignore_system: IgnoreSystem,
  }
  
  impl ScanOrchestrator {
      pub fn scan_directory(&self, path: &Path, strategy: ExecutionStrategy) -> Result<ScanResult>
      pub fn collect_file_paths(&self, path: &Path) -> Result<Vec<PathBuf>>
      
      /// Performance-first worker allocation - use maximum available workers
      pub fn determine_optimal_workers(&self, _file_count: usize, max_workers: usize) -> usize {
          max_workers  // Always use maximum available workers for best performance
      }
      
      pub fn determine_execution_strategy(&self, file_count: usize, config: &ScannerConfig) -> ExecutionStrategy {
          match config.scan_mode {
              ScanMode::Sequential => ExecutionStrategy::Sequential,
              ScanMode::Parallel => {
                  let max_workers = ExecutionStrategy::calculate_optimal_workers(
                      config.max_threads,     // User override (0 = no limit)
                      config.thread_percentage // % of CPU cores (default 75%)
                  );
                  ExecutionStrategy::Parallel { workers: max_workers }
              },
              ScanMode::Auto => {
                  let max_workers = ExecutionStrategy::calculate_optimal_workers(
                      config.max_threads, 
                      config.thread_percentage
                  );
                  ExecutionStrategy::auto(
                      file_count,
                      config.min_files_for_parallel, // Default: 5 files
                      max_workers                     // Use all available workers
                  )
              }
          }
      }
  }
  ```

#### Task 1.6: Ignore System
- **File**: `src/scan/ignore.rs`
- **Purpose**: Two-tier ignore system: file/path ignores and inline comment ignores
- **Source**: Extract ignore logic from existing modules
- **Implementation**:
  ```rust
  pub struct IgnoreSystem {
      path_ignorer: GlobSet,           // File/directory path ignores (*.log, tests/**, etc.)
      pattern_ignorer: Vec<String>,    // Content pattern ignores (DEMO_KEY_, FAKE_, etc.)
  }
  
  impl IgnoreSystem {
      pub fn should_ignore_path(&self, path: &Path) -> bool    // File-level ignores
      pub fn should_ignore_by_pattern(&self, content: &str) -> bool  // Pattern-level ignores
      // Note: Inline `guardy:allow` comments handled in post-processing step during pattern matching
  }
  ```

### Phase 2: Core Pattern System (Day 2)

#### Task 2.1: Aho-Corasick Keyword Prefilter
- **File**: `src/scan/prefilter.rs`
- **Purpose**: Ultra-fast keyword filtering to eliminate 85% of content
- **Strategy**: Extract keywords from all patterns to build Aho-Corasick automaton
- **Implementation**:
  ```rust
  use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
  
  pub struct KeywordPrefilter {
      automaton: AhoCorasick,
      keyword_to_patterns: HashMap<String, Vec<usize>>, // keyword -> pattern indices
  }
  
  impl KeywordPrefilter {
      pub fn new(patterns: &[ClassifiedPattern]) -> Result<Self> {
          let keywords: Vec<String> = patterns
              .iter()
              .enumerate()
              .flat_map(|(idx, pattern)| {
                  pattern.keywords.iter().map(move |kw| (kw.clone(), idx))
              })
              .collect();
              
          let automaton = AhoCorasickBuilder::new()
              .match_kind(MatchKind::LeftmostFirst)
              .build(&keywords.iter().map(|(kw, _)| kw).collect::<Vec<_>>())?;
              
          Ok(Self { automaton, keyword_to_patterns })
      }
      
      pub fn prefilter_content(&self, content: &str) -> HashSet<usize> {
          // Returns set of pattern indices that have keywords present
          let mut active_patterns = HashSet::new();
          for mat in self.automaton.find_iter(content) {
              if let Some(pattern_indices) = self.keyword_to_patterns.get(&content[mat.range()]) {
                  active_patterns.extend(pattern_indices);
              }
          }
          active_patterns
      }
  }
  ```

#### Task 2.2: Pattern Classification System
- **File**: `src/scan/patterns/classification.rs`
- **Purpose**: Intelligent pattern categorization for optimal performance
- **Strategy**: Classify patterns based on performance characteristics
- **Categories**:
  ```rust
  #[derive(Debug, Clone)]
  pub enum PatternClass {
      Specific,    // High-specificity patterns with reliable keywords (e.g., "sk_live_")
      Contextual,  // Patterns needing context analysis (e.g., generic API keys)
      AlwaysRun,   // Patterns without reliable keywords (e.g., entropy-only)
  }
  
  pub struct ClassifiedPattern {
      pub pattern: SecretPattern,
      pub class: PatternClass,
      pub keywords: Vec<String>,      // Extracted keywords for prefiltering
      pub priority: u8,               // 1-10, higher = run first
  }
  
  impl ClassifiedPattern {
      pub fn classify(pattern: &SecretPattern) -> Self {
          let keywords = Self::extract_keywords(&pattern.regex);
          let class = if keywords.len() >= 2 && keywords.iter().any(|k| k.len() >= 4) {
              PatternClass::Specific
          } else if keywords.len() == 1 {
              PatternClass::Contextual  
          } else {
              PatternClass::AlwaysRun
          };
          
          Self {
              pattern: pattern.clone(),
              class,
              keywords,
              priority: Self::calculate_priority(&class, &keywords),
          }
      }
  }
  ```

#### Task 2.3: Enhanced Pattern Library Integration
- **File**: `src/scan/patterns/mod.rs`
- **Purpose**: Combine existing Guardy patterns with select Gitleaks patterns
- **Strategy**: 
  - **Phase 2.3a**: Direct migration of all 40+ Guardy patterns
  - **Phase 2.3b**: Selective import of 20-30 high-value Gitleaks patterns
  - **Phase 2.3c**: Pattern deduplication and conflict resolution
- **Implementation**:
  ```rust
  pub struct PatternLibrary {
      guardy_patterns: Vec<ClassifiedPattern>,
      gitleaks_patterns: Vec<ClassifiedPattern>,
      combined_patterns: Vec<ClassifiedPattern>,
  }
  
  impl PatternLibrary {
      pub fn load_all() -> Result<Self> {
          let guardy_patterns = guardy::load_patterns()?;
          let gitleaks_patterns = gitleaks::load_selected_patterns()?;
          let combined = Self::merge_and_deduplicate(guardy_patterns, gitleaks_patterns)?;
          
          Ok(Self { guardy_patterns, gitleaks_patterns, combined })
      }
  }
  ```

### Phase 3: Core Scanner Architecture (Day 3)

#### Task 3.1: Single-Pass Pattern Matching Engine
- **File**: `src/scan/engine.rs`
- **Purpose**: Execute pattern matching on prefiltered content with unified single/multi-line approach
- **Strategy**: Smart execution based on pattern classification
- **Implementation**:
  ```rust
  pub struct MatchingEngine {
      entropy_analyzer: EntropyAnalyzer,
      binary_detector: BinaryDetector,
      file_processor: FileProcessor,
  }
  
  impl MatchingEngine {
      pub fn scan_with_patterns(&self, 
          content: &str, 
          path: &Path,
          active_patterns: &[ClassifiedPattern], 
          config: &ScannerConfig
      ) -> Result<Vec<SecretMatch>> {
          let test_ranges = self.ignore_system.build_test_ignore_ranges(content, path);
          let mut all_matches = Vec::new();
          
          for pattern in active_patterns {
              match pattern.class {
                  PatternClass::Specific => {
                      // Fast path for high-confidence patterns
                      all_matches.extend(self.scan_specific_pattern(content, pattern, &test_ranges)?);
                  },
                  PatternClass::Contextual => {
                      // Include entropy analysis for context
                      all_matches.extend(self.scan_contextual_pattern(content, pattern, &test_ranges)?);
                  },
                  PatternClass::AlwaysRun => {
                      // Entropy-heavy patterns, run regardless of keywords
                      all_matches.extend(self.scan_entropy_pattern(content, pattern, &test_ranges)?);
                  }
              }
          }
          
          Ok(self.deduplicate_and_rank(all_matches))
      }
  }
  ```

#### Task 3.2: Main Scanner Core with Parallel Integration
- **File**: `src/scan/core.rs`
- **Purpose**: Primary scanner orchestrator with parallel execution
- **Architecture**: Single-pass scanning with intelligent filtering and parallel processing
- **Implementation**:
  ```rust
  use crate::parallel::ExecutionStrategy;
  
  pub struct OptimizedScanner {
      prefilter: KeywordPrefilter,
      engine: MatchingEngine,
      pattern_library: PatternLibrary,
      orchestrator: ScanOrchestrator,
      config: ScannerConfig,
  }
  
  impl OptimizedScanner {
      pub fn new(config: ScannerConfig) -> Result<Self> {
          let pattern_library = PatternLibrary::load_all()?;
          let prefilter = KeywordPrefilter::new(&pattern_library.combined_patterns)?;
          let engine = MatchingEngine::new(&config)?;
          
          Ok(Self {
              prefilter,
              engine,
              pattern_library,
              config,
          })
      }
      
      pub fn scan_file(&self, path: &Path) -> Result<Vec<SecretMatch>> {
          // Step 1: Binary detection
          if self.engine.binary_detector.is_binary_file(path) {
              return Ok(vec![]);
          }
          
          // Step 2: Size check
          if !self.engine.file_processor.is_size_allowed(path) {
              return Ok(vec![]); // Skip large files
          }
          
          // Step 3: Load entire file content
          let content = self.engine.file_processor.load_file_content(path)?;
          self.scan_content(&content, path)
      }
      
      pub fn scan_paths(&self, paths: &[String]) -> Result<Vec<SecretMatch>> {
          // Step 1: Collect all file paths from input paths (directories and files)
          let file_paths = self.orchestrator.collect_file_paths_from_inputs(paths)?;
          
          // Step 2: Determine execution strategy based on file count
          let strategy = self.orchestrator.determine_execution_strategy(file_paths.len(), &self.config);
          
          // Step 3: Execute scanning with chosen strategy
          self.orchestrator.scan_files_with_strategy(&file_paths, strategy, |file_path| {
              self.scan_file(file_path)
          })
      }
      
      pub fn scan_content(&self, content: &str, path: &Path) -> Result<Vec<SecretMatch>> {
          // Step 1: Keyword prefiltering (eliminates ~85% of patterns)
          let active_pattern_indices = self.prefilter.prefilter_content(content);
          let active_patterns: Vec<_> = active_pattern_indices
              .iter()
              .map(|&idx| &self.pattern_library.combined_patterns[idx])
              .collect();
              
          // Step 2: Single-pass pattern matching (includes multi-line)
          let mut matches = self.engine.scan_with_patterns(content, path, &active_patterns, &self.config)?;
          
          // Step 3: Post-filter inline ignore directives
          matches.retain(|m| !m.line_content.contains("guardy:allow"));
          
          Ok(matches)
      }
  }
  ```

#### Task 3.3: Configuration Integration & CLI Support
- **File**: `src/scan/config.rs`  
- **Purpose**: Seamless integration with existing configuration system
- **Features**:
  - **Essential Features**: Preserve all critical functionality from legacy scanner
  - **Modern CLI**: Clean, intuitive command-line interface with improved defaults
  - **Performance Options**: New scan2-specific optimization controls
- **Implementation**:
  ```rust
  pub struct ScanConfig {
      // Essential functionality (no legacy baggage)
      pub enable_entropy_analysis: bool,       // Default: true
      pub min_entropy_threshold: f64,          // Default: 1.0/1e5
      pub max_file_size_mb: usize,            // Default: 50MB (modern default)
      pub streaming_threshold_mb: usize,      // Default: 20MB (modern default)
      pub include_binary: bool,               // Default: false
      pub ignore_patterns: Vec<String>,       // Pattern-based ignores
      pub ignore_paths: Vec<String>,          // Path-based ignores 
      pub ignore_comments: Vec<String>,       // Comment-based ignores
      pub ignore_test_code: bool,             // Default: true
      
      // Performance optimizations
      pub enable_keyword_prefilter: bool,    // Default: true
      pub pattern_classification: bool,      // Default: true  
      pub prefilter_threshold: f32,          // Default: 0.1
      pub max_multiline_size: usize,         // Default: 1MB
      
      // Parallel processing (integrated with existing parallel module)
      pub max_threads: usize,                // Default: 0 (auto-detect, no hard limit)
      pub thread_percentage: u8,             // Default: 75 (75% of available CPU cores)
      pub min_files_for_parallel: usize,     // Default: 5 (optimal threshold for I/O-bound file scanning)
      pub scan_mode: ScanMode,               // Default: Auto (Sequential/Parallel/Auto)
      
      // Worker Allocation Strategy:
      // 1. System Detection: detect available CPU cores using num_cpus::get()
      // 2. Percentage Application: apply thread_percentage (e.g., 75% of 16 cores = 12 workers)
      // 3. User Override: respect max_threads if set (0 = no override)
      // 4. Threshold Decision: use parallel if file_count >= min_files_for_parallel
      // 5. Performance-First: always use maximum calculated workers (no file-count scaling)
  }
  
  impl Default for ScanConfig {
      fn default() -> Self {
          Self {
              enable_entropy_analysis: true,
              min_entropy_threshold: 1.0 / 1e5,
              max_file_size_mb: 50,           // Modern default
              streaming_threshold_mb: 20,     // Modern default
              include_binary: false,
              ignore_patterns: vec![
                  "# TEST_SECRET:".to_string(),
                  "DEMO_KEY_".to_string(),
                  "FAKE_".to_string(),
              ],
              ignore_paths: vec![
                  "tests/*".to_string(),
                  "*_test.rs".to_string(), 
                  ".git/**".to_string(),
              ],
              ignore_comments: vec![
                  "guardy:ignore".to_string(),
                  "guardy:ignore-line".to_string(),
                  "guardy:ignore-next".to_string(),
              ],
              ignore_test_code: true,
              enable_keyword_prefilter: true,
              pattern_classification: true,
              prefilter_threshold: 0.1,
              max_multiline_size: 1024 * 1024,
              max_threads: 0,                 // No hard limit - use percentage calculation
              thread_percentage: 75,              // Use 75% of available CPU cores  
              min_files_for_parallel: 5,          // Lower threshold for I/O-bound scanning
          }
      }
  }
  ```

### Phase 4: CLI Integration & Testing (Day 4)

#### Task 4.1: Clean CLI Implementation  
- **File**: `src/cli/commands/scan2.rs`
- **Purpose**: Modern, clean CLI interface for optimized scanner
- **Strategy**: Fresh implementation focused on essential features with better defaults
- **Implementation**:
  ```rust
  use crate::scan::{OptimizedScanner, ScanConfig};
  
  #[derive(clap::Args)]
  pub struct Scan2Args {
      /// Paths to scan (files or directories)
      #[arg(required = true)]
      pub paths: Vec<String>,
      
      /// Output format
      #[arg(long, default_value = "text")]
      pub format: String,
      
      /// Skip entropy analysis (faster but less accurate)
      #[arg(long)]
      pub no_entropy: bool,
      
      /// Include binary files in scan
      #[arg(long)]
      pub include_binary: bool,
      
      /// Maximum file size to scan in MB
      #[arg(long, default_value = "50")]
      pub max_file_size_mb: usize,
      
      /// File size threshold for streaming in MB
      #[arg(long, default_value = "20")]  
      pub streaming_threshold_mb: usize,
      
      /// Disable keyword prefiltering (debug option)
      #[arg(long)]
      pub no_prefilter: bool,
      
      /// Pattern classification threshold (0.0-1.0)
      #[arg(long, default_value = "0.1")]
      pub prefilter_threshold: f32,
      
      /// Maximum threads (0 = auto-detect)
      #[arg(long, default_value = "0")]
      pub max_threads: usize,
      
      /// CPU percentage to use
      #[arg(long, default_value = "75")]
      pub thread_percentage: u8,
  }
  
  impl Scan2Args {
      pub fn execute(&self) -> Result<()> {
          let mut config = ScanConfig::default();
          
          // Apply CLI overrides
          config.enable_entropy_analysis = !self.no_entropy;
          config.include_binary = self.include_binary;
          config.max_file_size_mb = self.max_file_size_mb;
          config.streaming_threshold_mb = self.streaming_threshold_mb;
          config.enable_keyword_prefilter = !self.no_prefilter;
          config.prefilter_threshold = self.prefilter_threshold;
          config.max_threads = self.max_threads;
          config.thread_percentage = self.thread_percentage;
          
          let scanner = OptimizedScanner::new(config)?;
          let results = scanner.scan_paths(&self.paths)?;
          
          self.output_results(results)?;
          Ok(())
      }
  }
  ```

#### Task 4.2: Module Integration in lib.rs
- **File**: `src/lib.rs`
- **Purpose**: Add new `scan` module alongside existing `scanner`
- **Implementation**:
  ```rust
  // Add to existing modules
  pub mod scan;        // New optimized scanner
  pub mod scanner;     // Legacy scanner (preserve)
  ```

#### Task 4.3: CLI Root Integration
- **File**: `src/cli/mod.rs` and `src/cli/commands/mod.rs`
- **Purpose**: Add scan2 subcommand to CLI structure
- **Implementation**:
  ```rust
  pub mod scan2;  // New subcommand
  
  #[derive(clap::Subcommand)]
  pub enum Commands {
      /// Scan files and directories for secrets (current engine)
      Scan(scan::ScanArgs),
      
      /// Scan files and directories for secrets (optimized engine - experimental)
      Scan2(scan2::Scan2Args),
      
      // ... other existing commands
  }
  ```

#### Task 4.4: Performance Benchmarking Integration
- **File**: `benches/scan_comparison.rs`
- **Purpose**: Automated benchmarking between legacy and optimized scanners
- **Key Metrics**:
  - **Performance**: 5x speed improvement target
  - **Memory**: Single allocation per file vs multiple passes
  - **Accuracy**: Pattern detection parity validation
  - **File Size Impact**: Measure benefits of 50MB limit vs streaming
- **Test Scenarios**:
  - Small files (< 1MB): Majority of source code
  - Medium files (1-10MB): Package locks, configs
  - Large files (10-50MB): Generated code, large configs
  - **Files >50MB**: Skipped entirely (no streaming complexity)

## 🧪 Testing Strategy

### Performance Benchmarks
1. **Micro-benchmarks**: Individual component performance
2. **Real-world datasets**: Test on actual codebases
3. **Comparative analysis**: scan vs scan2 performance
4. **Memory usage**: Ensure reasonable memory consumption

### Correctness Testing
1. **Pattern coverage**: Ensure no detection regressions
2. **Edge cases**: Binary files, large files, Unicode content
3. **Configuration compatibility**: All existing configs work

### Validation Process
1. Run both scanners on identical datasets
2. Compare results for accuracy
3. Measure performance improvements
4. Document any behavioral differences

## 📈 Success Metrics

### Performance Targets
- **Speed**: 5x faster on typical codebases  
- **Memory**: ≤2x memory usage increase
- **Accuracy**: ≥99% pattern detection retention

### Quality Gates
- All existing tests pass with scan2
- No clippy warnings or formatting issues
- Comprehensive documentation
- Clean, maintainable code architecture

## 🚀 Migration & Rollout Strategy

### Phase 1: Experimental Release (Day 5)
- **New Command**: `guardy scan2` subcommand available
- **Coexistence**: Legacy `guardy scan` remains unchanged and default
- **Testing**: Validate on real codebases
- **Monitoring**: Performance benchmarking and accuracy validation

### Phase 2: Validation & Refinement (Day 6-7)  
- **Issue Resolution**: Address any bugs or performance issues found
- **Feature Parity**: Ensure 100% compatibility with existing functionality
- **Benchmarking**: Quantify actual performance improvements on diverse codebases

### Phase 3: Promotion to Default (Day 8)
- **CLI Update**: Make new `scan/` module the default for `guardy scan` command
- **Legacy Preservation**: Keep `guardy scan --legacy` flag for backwards compatibility
- **Migration Notice**: Notify users of the engine change in release notes

### Phase 4: Legacy Deprecation (Future Version)
- **Deprecation Warning**: Add deprecation notices for legacy scanner
- **Community Notice**: Announce timeline for legacy scanner removal
- **Final Migration**: Remove legacy `scanner/` module in future major version

## 🔧 Implementation Notes

### Dependencies
- `aho-corasick`: For keyword prefiltering (already in Cargo.toml)
- `regex`: For pattern matching (already in Cargo.toml)
- No new external dependencies required for MVP

### Future Performance Optimization
- **Vectorscan Integration**: Optional high-performance backend for scan3
  - **10-50x performance gains** on complex patterns vs current regex
  - **Full ARM/Mac support** (unlike Intel Hyperscan)
  - **Open source BSD license** (vs proprietary Intel versions)
  - **Implementation**: Phase after scan2 MVP if performance gains justify complexity

### Design Considerations
- Clean, modern architecture without legacy constraints
- Improved defaults for modern development practices
- Streamlined CLI interface focused on essential features
- Optimized data structures and algorithms

### Error Handling
- Comprehensive error reporting and debugging information
- Robust file processing with graceful error recovery
- Clear validation messages for configuration issues

## 📝 Documentation Plan

### Code Documentation
- Comprehensive rustdoc comments
- Architecture decision records
- Performance characteristics documentation

### User Documentation  
- Clean installation and usage guide
- Performance optimization recommendations  
- Advanced configuration options

## ✅ Approval Required

**User Review Points:**
1. **Architecture**: Approve the module structure and design approach
2. **Pattern Strategy**: Confirm Gitleaks pattern integration approach  
3. **Performance Targets**: Validate 5x improvement goal is realistic
4. **Timeline**: Confirm 3-4 day implementation timeline
5. **Testing Strategy**: Approve the validation and benchmarking plan

**Next Steps After Approval:**
1. Begin Phase 1 implementation starting with data structure migration
2. Set up automated benchmarking infrastructure  
3. Create initial `src/scan/` module structure
4. Start with exact legacy functionality preservation
5. Implement Aho-Corasick prefilter as first optimization

## 🎯 Key Success Criteria

### Technical Requirements
- ✅ **Zero Regression**: All existing secrets must be detected by scan2
- ✅ **Performance Target**: Achieve 5x speed improvement on typical codebases
- ✅ **Memory Efficiency**: Keep memory usage within 2x of current scanner
- ✅ **API Compatibility**: All existing CLI flags and config options work identically

### Quality Gates  
- ✅ **Test Coverage**: Comprehensive test suite covering all migrated functionality
- ✅ **Code Quality**: Pass all clippy lints and formatting checks
- ✅ **Documentation**: Complete rustdoc and user documentation
- ✅ **Benchmarks**: Automated performance comparison infrastructure

### Implementation Safety
- ✅ **Clean Architecture**: No legacy code constraints or technical debt
- ✅ **Essential Features**: All critical functionality preserved and optimized
- ✅ **Modern Defaults**: Better defaults for contemporary development practices
- ✅ **Performance Focus**: Built for speed from the ground up

## 📋 Implementation Checklist

**Phase 1: Foundation (Day 1)**
- [ ] Create `src/scan/` module structure
- [ ] Migrate all data structures with exact compatibility
- [ ] Copy entropy analysis system (preserve algorithms exactly)
- [ ] Implement whole-file processing (50MB size limit)
- [ ] Migrate binary file detection system
- [ ] Implement two-tier ignore system (file/path ignores + inline 'guardy:allow' comments)
- [ ] Verify core functionality works with new architecture

**Phase 2: Pattern System & Optimization (Day 2)**  
- [ ] Implement Aho-Corasick keyword prefilter
- [ ] Create pattern classification system
- [ ] Migrate all Guardy patterns to new system
- [ ] Import select Gitleaks patterns for enhanced coverage
- [ ] Implement single-pass whole-file scanning engine
- [ ] Create pattern matching with inline ignore filtering

**Phase 3: Integration (Day 3)**
- [ ] Implement main scanner orchestrator
- [ ] Create configuration integration layer
- [ ] Add CLI subcommand `guardy scan2`
- [ ] Set up automated benchmarking
- [ ] Implement comprehensive test coverage
- [ ] Performance validation on real codebases

**Phase 4: Rollout (Days 4-8)**
- [ ] Release experimental `scan2` subcommand
- [ ] Validate performance and accuracy improvements
- [ ] Address any issues found during testing
- [ ] Promote to default scanner engine
- [ ] Plan legacy scanner deprecation timeline

---

**Created**: 2025-08-11  
**Status**: Awaiting User Approval  
**Estimated Timeline**: 3-4 days implementation + 4-5 days rollout  
**Risk Level**: Low-Medium (simplified architecture, proven Gitleaks approach)  
**Dependencies**: User approval, comprehensive legacy functionality analysis complete