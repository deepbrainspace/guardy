//! Binary file filtering using two-stage detection
//!
//! This implements the v2-style binary detection with optimal I/O efficiency:
//! Stage 1: Extension check (O(1) HashSet lookup, no I/O)
//! Stage 2: Content inspection (reads first 512 bytes for unknown extensions)

use crate::scan::filters::{Filter, FilterDecision};
use crate::scan::static_data::binary_extensions::is_binary_extension;
use anyhow::Result;
use smallvec::SmallVec;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Statistics for binary filter performance analysis
#[derive(Debug, Clone, Default)]
pub struct BinaryFilterStats {
    pub files_checked: usize,
    pub files_binary_by_extension: usize,
    pub files_binary_by_content: usize,
    pub files_text_confirmed: usize,
    pub files_content_inspection_skipped: usize,
    pub content_inspections_performed: usize,
    pub extension_cache_hits: usize,
}

/// Binary filter with two-stage detection for optimal I/O efficiency
/// 
/// Performance characteristics:
/// - Stage 1: Extension check (O(1) lookup, no I/O) - handles ~95% of binary files
/// - Stage 2: Content inspection (512 bytes read) - only for unknown extensions
/// - Much more efficient than reading full files in FilePipeline
#[derive(Clone)]
pub struct BinaryFilter {
    /// Whether to skip binary files (from config)
    skip_binary: bool,
    /// Statistics collection for performance analysis
    stats: Arc<Mutex<BinaryFilterStats>>,
}

impl BinaryFilter {
    /// Create a new binary filter
    pub fn new(skip_binary: bool) -> Self {
        tracing::debug!("Binary filter initialized: skip_binary={}", skip_binary);
        
        Self {
            skip_binary,
            stats: Arc::new(Mutex::new(BinaryFilterStats::default())),
        }
    }
    
    /// Determine if a file is binary using optimized two-stage detection
    /// 
    /// Stage 1: Fast extension check (O(1) HashSet lookup, no I/O)
    /// - Check if file extension is in the known binary extensions set
    /// - Handles ~90-95% of binary files instantly with no I/O
    /// - Uses shared static binary extensions for zero-copy access
    /// 
    /// Stage 2: Content inspection with size threshold (only for unknown extensions)
    /// - First checks file size (cached metadata syscall ~0.1μs)
    /// - Skip content inspection for small files (< 4KB) - assume text
    /// - For files ≥ 4KB, read first 512 bytes to detect binary content
    /// - 4KB threshold works well for all storage types (HDD/SSD/NVMe)
    pub fn is_binary_file(&self, path: &Path) -> Result<bool> {
        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_checked += 1;
        }
        
        // Stage 1: Fast extension check (O(1) lookup, no I/O)
        if let Some(extension) = path.extension() 
            && let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            
            if is_binary_extension(&ext_lower) {
                // Update statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.extension_cache_hits += 1;
                    stats.files_binary_by_extension += 1;
                }
                
                tracing::trace!("File detected as binary by extension '{}': {}", ext_lower, path.display());
                return Ok(true);
            }
        }
        
        // Stage 2: Content inspection with size threshold for unknown extensions
        // First check file size to determine if content inspection is worth it
        const CONTENT_INSPECTION_THRESHOLD: u64 = 4096; // 4KB - works well for all storage types
        
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        
        if file_size < CONTENT_INSPECTION_THRESHOLD {
            // Small files are likely text - skip content inspection to avoid syscall overhead
            tracing::trace!(
                "Small file ({} bytes < {} threshold), assuming text: {}", 
                file_size, CONTENT_INSPECTION_THRESHOLD, path.display()
            );
            
            // Update statistics
            if let Ok(mut stats) = self.stats.lock() {
                stats.files_content_inspection_skipped += 1;
            }
            
            return Ok(false);
        }
        
        // File is large enough to warrant content inspection
        tracing::trace!(
            "Performing content inspection for large unknown extension file ({} bytes): {}", 
            file_size, path.display()
        );
        
        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.content_inspections_performed += 1;
        }
        
        // Read first 512 bytes for content inspection
        let mut buffer = [0u8; 512];
        let bytes_read = match std::fs::File::open(path)
            .and_then(|mut file| {
                use std::io::Read;
                file.read(&mut buffer)
            }) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("Failed to read file for content inspection {}: {}", path.display(), e);
                // If we can't read the file, assume it's binary to be safe
                return Ok(true);
            }
        };
        
        let is_binary = match content_inspector::inspect(&buffer[..bytes_read]) {
            content_inspector::ContentType::BINARY => {
                tracing::trace!("File detected as binary by content inspection: {}", path.display());
                
                // Update statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.files_binary_by_content += 1;
                }
                
                true
            }
            content_inspector::ContentType::UTF_8 
            | content_inspector::ContentType::UTF_8_BOM 
            | content_inspector::ContentType::UTF_16LE
            | content_inspector::ContentType::UTF_16BE
            | content_inspector::ContentType::UTF_32LE
            | content_inspector::ContentType::UTF_32BE => {
                tracing::trace!("File confirmed as text by content inspection: {}", path.display());
                
                // Update statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.files_text_confirmed += 1;
                }
                
                false
            }
        };
        
        Ok(is_binary)
    }
}

impl Filter for BinaryFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // If we're not skipping binary files, always process
        if !self.skip_binary {
            return Ok(FilterDecision::Process);
        }
        
        // Check if file is binary using two-stage detection with size threshold
        let is_binary = self.is_binary_file(path)?;
        
        if is_binary {
            tracing::trace!("Skipping binary file: {}", path.display());
            Ok(FilterDecision::Skip("binary file detected"))
        } else {
            Ok(FilterDecision::Process)
        }
    }
    
    fn name(&self) -> &'static str {
        "BinaryFilter"
    }
    
    fn get_stats(&self) -> SmallVec<[(String, String); 8]> {
        let stats = self.stats.lock()
            .map(|s| s.clone())
            .unwrap_or_default();
            
        smallvec::smallvec![
            ("Files checked".to_string(), stats.files_checked.to_string()),
            ("Binary by extension".to_string(), stats.files_binary_by_extension.to_string()),
            ("Binary by content".to_string(), stats.files_binary_by_content.to_string()),
            ("Text confirmed".to_string(), stats.files_text_confirmed.to_string()),
            ("Extension cache hits".to_string(), stats.extension_cache_hits.to_string()),
            ("Content inspections".to_string(), stats.content_inspections_performed.to_string()),
        ]
    }
}

