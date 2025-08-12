use crate::scan::entropy::Entropy;
use crate::scan::types::{ScannerConfig, SecretMatch};
use anyhow::Result;

/// Entropy Filter - Statistical validation wrapper for content-level filtering
///
/// Responsibilities:
/// - Provide clean filter interface for entropy validation in scanning pipeline
/// - Wrap core entropy analysis algorithms with filter-compatible API
/// - Apply configurable entropy thresholds for secret validation
/// - Integration with ScannerConfig for entropy analysis settings
/// - Statistics and performance tracking for entropy validation
///
/// This filter is applied AFTER regex pattern matching and keyword prefiltering
/// to validate that potential secret matches have sufficient entropy to be
/// considered actual secrets rather than common words or constants.
///
/// Algorithm Flow:
/// 1. Receive potential secret matches from regex patterns
/// 2. Extract matched text for entropy analysis
/// 3. Apply core entropy algorithms (bigrams, char classes, distinct values)
/// 4. Compare probability against configurable threshold
/// 5. Filter out low-entropy matches (likely false positives)
/// 6. Pass high-entropy matches to final results
///
/// Performance Characteristics:
/// - Uses shared entropy constants from core entropy module
/// - Fast statistical analysis optimized for secret detection workloads
/// - Configurable thresholds allow tuning false positive vs false negative rates

/// Entropy filtering statistics for debugging and analysis
#[derive(Debug, Clone)]
pub struct EntropyFilterStats {
    pub matches_checked: usize,
    pub matches_passed_entropy: usize,
    pub matches_failed_entropy: usize,
    pub total_entropy_analysis_time_ms: u64,
    pub average_entropy_check_time_ms: f64,
}

impl Default for EntropyFilterStats {
    fn default() -> Self {
        Self {
            matches_checked: 0,
            matches_passed_entropy: 0,
            matches_failed_entropy: 0,
            total_entropy_analysis_time_ms: 0,
            average_entropy_check_time_ms: 0.0,
        }
    }
}

/// Entropy filter for content-level statistical validation
pub struct EntropyFilter {
    /// Enable/disable entropy analysis
    enable_entropy_analysis: bool,
    /// Minimum entropy threshold for accepting matches
    min_entropy_threshold: f64,
    /// Statistics collection for debugging and performance analysis
    stats: std::sync::Mutex<EntropyFilterStats>,
}

impl EntropyFilter {
    /// Create a new entropy filter with configuration
    ///
    /// # Arguments
    /// * `config` - Scanner configuration with entropy analysis settings
    ///
    /// # Returns
    /// A configured entropy filter ready for use
    pub fn new(config: &ScannerConfig) -> Result<Self> {
        tracing::debug!(
            "Entropy filter initialized: enabled={}, threshold={:.2e}",
            config.enable_entropy_analysis,
            config.min_entropy_threshold
        );

        Ok(Self {
            enable_entropy_analysis: config.enable_entropy_analysis,
            min_entropy_threshold: config.min_entropy_threshold,
            stats: std::sync::Mutex::new(EntropyFilterStats::default()),
        })
    }

    /// Check if a secret match should be filtered out due to low entropy
    ///
    /// Uses the core entropy analysis algorithms to determine if the matched text
    /// has sufficient randomness to be considered a real secret.
    ///
    /// # Arguments
    /// * `secret_match` - The secret match to validate
    ///
    /// # Returns
    /// * `Ok(true)` - Match should be filtered out (low entropy)
    /// * `Ok(false)` - Match should be kept (sufficient entropy)
    /// * `Err(_)` - Error during entropy analysis
    ///
    /// # Performance
    /// - Uses shared entropy constants for optimal performance
    /// - Statistical analysis optimized for typical secret lengths
    /// - Fast probability calculations with minimal overhead
    pub fn should_filter_match(&self, secret_match: &SecretMatch) -> Result<bool> {
        // If entropy analysis is disabled, never filter
        if !self.enable_entropy_analysis {
            return Ok(false);
        }

        let start_time = std::time::Instant::now();

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.matches_checked += 1;
        }

        // Perform entropy validation using core entropy module
        let has_sufficient_entropy = Entropy::validate_entropy(
            &secret_match.matched_text,
            self.min_entropy_threshold
        )?;

        let duration = start_time.elapsed();

        // Update statistics with timing
        if let Ok(mut stats) = self.stats.lock() {
            let duration_ms = duration.as_millis() as u64;
            stats.total_entropy_analysis_time_ms += duration_ms;

            if has_sufficient_entropy {
                stats.matches_passed_entropy += 1;
            } else {
                stats.matches_failed_entropy += 1;
            }

            // Update average timing
            if stats.matches_checked > 0 {
                stats.average_entropy_check_time_ms =
                    stats.total_entropy_analysis_time_ms as f64 / stats.matches_checked as f64;
            }
        }

        let should_filter = !has_sufficient_entropy;

        if should_filter {
            tracing::debug!(
                "Secret match filtered due to low entropy in {}:{} - matched_text: '{}'",
                secret_match.file_path,
                secret_match.line_number,
                secret_match.matched_text
            );
        } else {
            tracing::trace!(
                "Secret match passed entropy validation in {}:{} - matched_text: '{}'",
                secret_match.file_path,
                secret_match.line_number,
                secret_match.matched_text
            );
        }

        Ok(should_filter)
    }

    /// Filter a list of secret matches, removing those with insufficient entropy
    ///
    /// # Arguments
    /// * `matches` - List of secret matches to filter
    ///
    /// # Returns
    /// Vector of matches that passed entropy validation
    pub fn filter_matches(&self, matches: &[SecretMatch]) -> Vec<SecretMatch> {
        if !self.enable_entropy_analysis {
            // Entropy analysis disabled - return all matches unchanged
            return matches.to_vec();
        }

        matches
            .iter()
            .filter(|secret_match| {
                match self.should_filter_match(secret_match) {
                    Ok(should_filter) => !should_filter,
                    Err(e) => {
                        tracing::warn!(
                            "Error during entropy validation for match in {}:{}: {}",
                            secret_match.file_path,
                            secret_match.line_number,
                            e
                        );
                        true // Include matches we can't validate (conservative approach)
                    }
                }
            })
            .cloned()
            .collect()
    }

    /// Validate entropy for a raw string value
    ///
    /// This is a convenience method for validating entropy of arbitrary strings,
    /// useful for testing and validation scenarios.
    ///
    /// # Arguments
    /// * `value` - String value to validate
    ///
    /// # Returns
    /// * `Ok(true)` - Value has sufficient entropy
    /// * `Ok(false)` - Value has insufficient entropy
    /// * `Err(_)` - Error during entropy analysis
    pub fn validate_string_entropy(&self, value: &str) -> Result<bool> {
        if !self.enable_entropy_analysis {
            return Ok(true); // Always pass when disabled
        }

        Entropy::validate_entropy(value, self.min_entropy_threshold)
    }

    /// Get current filter statistics
    ///
    /// # Returns
    /// Statistics about matches processed and entropy analysis performance
    pub fn get_stats(&self) -> EntropyFilterStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = EntropyFilterStats::default();
        }
    }

    /// Get configuration information for debugging
    ///
    /// # Returns
    /// Tuple of (enabled, threshold) for current entropy settings
    pub fn get_config_info(&self) -> (bool, f64) {
        (self.enable_entropy_analysis, self.min_entropy_threshold)
    }

    /// Check if entropy filtering is enabled
    ///
    /// # Returns
    /// True if entropy analysis is enabled and will filter matches
    pub fn is_enabled(&self) -> bool {
        self.enable_entropy_analysis
    }

    /// Get the current entropy threshold
    ///
    /// # Returns
    /// Minimum entropy threshold used for validation
    pub fn get_threshold(&self) -> f64 {
        self.min_entropy_threshold
    }

    /// Calculate entropy statistics for a match without filtering
    ///
    /// This method provides detailed entropy analysis for debugging and
    /// development purposes without affecting the filtering decision.
    ///
    /// # Arguments
    /// * `text` - Text to analyze
    ///
    /// # Returns
    /// Entropy probability calculated by core algorithms
    pub fn calculate_entropy_probability(&self, text: &str) -> f64 {
        Entropy::calculate_randomness_probability(text.as_bytes())
    }
}

// All tests moved to tests/integration/scan/filters/content/
// to allow proper integration testing with git-crypted test data