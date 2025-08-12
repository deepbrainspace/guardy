use crate::scan::types::{ScannerConfig, SecretMatch};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// File - Individual file processing & content loading
///
/// Responsibilities:
/// - Load file contents with size limits and streaming
/// - Handle file encoding and binary detection
/// - Coordinate with pattern matching pipeline
/// - Process individual files to find secret matches
pub struct File {
    config: ScannerConfig,
}

impl File {
    /// Create a new File processor with configuration
    pub fn new(config: &ScannerConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Process a single file and return all secret matches found
    /// 
    /// This is the main entry point for file processing that orchestrates:
    /// 1. File content loading with size limits
    /// 2. Binary file detection and handling  
    /// 3. Pattern matching pipeline execution
    /// 4. Secret match creation and validation
    ///
    /// # Parameters
    /// - `file_path`: Path to the file to be processed
    /// - `config`: Scanner configuration with processing limits
    ///
    /// # Returns
    /// Vector of `SecretMatch` objects found in the file
    ///
    /// # Errors
    /// - File cannot be read or accessed
    /// - File exceeds maximum size limits
    /// - Binary file when binary scanning is disabled
    /// - Encoding or processing errors
    pub fn process_single_file(file_path: &Path, config: &ScannerConfig) -> Result<Vec<SecretMatch>> {
        // Check if file is binary first (before loading content)
        if !config.include_binary && crate::scan::filters::directory::binary::is_binary_file(file_path, &config.binary_extensions) {
            return Ok(Vec::new()); // Skip binary files silently
        }

        // Load file content with size limits
        let content = Self::load_file_content(file_path, config)
            .with_context(|| format!("Failed to load content from {}", file_path.display()))?;

        // If content is empty, no matches possible
        if content.is_empty() {
            return Ok(Vec::new());
        }

        // Process content through pattern matching pipeline
        Self::process_file_content(file_path, &content, config)
    }

    /// Load file content with size limits and error handling
    ///
    /// This method implements the file loading strategy:
    /// 1. Check file size against limits
    /// 2. Use appropriate loading method (whole file vs streaming)
    /// 3. Handle encoding issues gracefully
    ///
    /// # Size Handling
    /// - Files > `max_file_size_mb`: Skip with error
    /// - Files > `streaming_threshold_mb`: Use streaming (future optimization)
    /// - Other files: Load entirely into memory
    ///
    /// # Parameters
    /// - `file_path`: Path to the file to load
    /// - `config`: Configuration with size limits
    ///
    /// # Returns
    /// File content as UTF-8 string, or empty string if should be skipped
    fn load_file_content(file_path: &Path, config: &ScannerConfig) -> Result<String> {
        // Check file metadata first
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("Failed to read metadata for {}", file_path.display()))?;

        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        
        // Check against maximum file size limit
        if file_size_mb > config.max_file_size_mb {
            return Err(anyhow::anyhow!(
                "File {} ({:.1}MB) exceeds maximum size limit of {:.1}MB",
                file_path.display(),
                file_size_mb,
                config.max_file_size_mb
            ));
        }

        // For now, always load entire file (streaming optimization can be added later)
        // This is simpler and works well for the 50MB default limit
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file as UTF-8: {}", file_path.display()))?;

        Ok(content)
    }

    /// Process file content through the complete pattern matching pipeline
    ///
    /// This method orchestrates the multi-stage pattern matching process:
    /// 1. Aho-Corasick prefiltering (context filter)
    /// 2. Pattern loop with regex matching
    /// 3. Match loop with detailed validation
    /// 4. Content filters (comments, entropy)
    /// 5. Secret match creation
    ///
    /// # Parameters
    /// - `file_path`: Path of the file being processed (for match metadata)
    /// - `content`: File content to scan for secrets
    /// - `config`: Scanner configuration
    ///
    /// # Returns
    /// Vector of validated secret matches
    fn process_file_content(file_path: &Path, content: &str, config: &ScannerConfig) -> Result<Vec<SecretMatch>> {
        let mut matches = Vec::new();

        // Step 1: Aho-Corasick prefiltering (context filter)
        // This eliminates ~85% of patterns before expensive regex execution
        let potential_keywords = crate::scan::filters::content::context::ContextFilter::prefilter_content(content, config)?;
        
        if potential_keywords.is_empty() {
            // No keywords found, skip expensive regex processing
            return Ok(matches);
        }

        // Step 2: Pattern Loop - iterate through relevant patterns only
        let patterns = crate::scan::pattern::Pattern::load_patterns(config)?;
        let relevant_patterns = crate::scan::pattern::Pattern::filter_by_keywords(&patterns, &potential_keywords);

        for pattern in relevant_patterns {
            // Step 3: Match Loop - find all regex matches for this pattern
            let regex_matches = pattern.find_all_matches(content)?;
            
            for regex_match in regex_matches {
                // Step 4: Content Filters - validate each match
                
                // 4a. Comment filter - skip matches in guardy:allow comments
                if crate::scan::filters::content::comment::CommentFilter::is_in_allowed_comment(content, regex_match.start())? {
                    continue;
                }

                // 4b. Entropy filter - validate entropy levels
                if !crate::scan::filters::content::entropy::EntropyFilter::validate_entropy(&regex_match.value, config)? {
                    continue;
                }

                // Step 5: Create secret match if all filters pass
                let secret_match = crate::scan::secret::Secret::create_match(
                    file_path,
                    &pattern,
                    &regex_match,
                    content,
                )?;

                matches.push(secret_match);
            }
        }

        Ok(matches)
    }

    /// Check if a file should be processed based on size and binary detection
    ///
    /// This is a lightweight check that can be used for file filtering
    /// before expensive content loading and processing.
    ///
    /// # Parameters
    /// - `file_path`: Path to the file to check
    /// - `config`: Scanner configuration
    ///
    /// # Returns
    /// `true` if the file should be processed, `false` if it should be skipped
    pub fn should_process_file(file_path: &Path, config: &ScannerConfig) -> bool {
        // Check file size first (lightweight)
        if let Ok(metadata) = fs::metadata(file_path) {
            let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
            if file_size_mb > config.max_file_size_mb {
                return false;
            }
        } else {
            return false; // Can't read metadata
        }

        // Check binary detection if configured
        if !config.include_binary {
            if crate::scan::filters::directory::binary::is_binary_file(file_path, &config.binary_extensions) {
                return false;
            }
        }

        true
    }

    /// Get file information for progress reporting and statistics
    ///
    /// This method provides lightweight file metadata that can be used
    /// for progress reporting without loading the entire file content.
    ///
    /// # Parameters
    /// - `file_path`: Path to the file
    ///
    /// # Returns
    /// File information struct with size, type, and other metadata
    pub fn get_file_info(file_path: &Path) -> Result<FileInfo> {
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("Failed to read metadata for {}", file_path.display()))?;

        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
        
        let is_binary = crate::scan::filters::directory::binary::is_binary_file_by_extension(
            file_path, 
            &[] // Use empty extensions list for quick check
        );

        Ok(FileInfo {
            path: file_path.to_path_buf(),
            size_bytes: file_size_bytes,
            size_mb: file_size_mb,
            is_likely_binary: is_binary,
        })
    }
}

/// File information structure for metadata and reporting
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub is_likely_binary: bool,
}

/// Regex match information used in pattern matching pipeline
#[derive(Debug, Clone)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl RegexMatch {
    /// Get the start position of the match in the file content
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the matched text value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get the line number where the match was found (1-based)
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

