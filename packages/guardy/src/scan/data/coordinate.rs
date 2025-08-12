//! Coordinate system for match positions
//!
//! Optimized for 50MB file limit with realistic field sizes.
//! Total struct size: 16 bytes (vs 40 bytes with usize)

use std::fmt;

/// Represents a position in a file
/// 
/// Optimized for space efficiency while supporting:
/// - Files up to 50MB (u32 supports up to 4GB)
/// - Up to 16 million lines (u24 via [u8; 3])
/// - Column positions up to 4GB (for minified files)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Coordinate {
    /// Absolute byte offset start in file (supports up to 4GB)
    pub byte_start: u32,
    
    /// Absolute byte offset end in file (supports up to 4GB)
    pub byte_end: u32,
    
    /// Line number (1-indexed, supports up to 16.7 million lines)
    /// Using u32 since u24 doesn't exist and bitpacking isn't worth complexity
    pub line: u32,
    
    /// Column start position (0-indexed, supports full line width)
    pub column_start: u16,
    
    /// Column width (not end position, to save space for most cases)
    /// If width > 65535, we store 65535 and compute from byte positions
    pub column_width: u16,
}
// Total: 16 bytes (60% smaller than usize version)

impl Coordinate {
    /// Create a new coordinate
    pub fn new(
        line: u32,
        column_start: u16,
        column_end: u16,
        byte_start: u32,
        byte_end: u32,
    ) -> Self {
        let column_width = column_end.saturating_sub(column_start);
        
        Self {
            line,
            column_start,
            column_width,
            byte_start,
            byte_end,
        }
    }
    
    /// Create from common usize values (helper for compatibility)
    pub fn from_usize(
        line: usize,
        column_start: usize,
        column_end: usize,
        byte_start: usize,
        byte_end: usize,
    ) -> Option<Self> {
        // Validate bounds
        if byte_start > u32::MAX as usize || byte_end > u32::MAX as usize {
            return None; // File too large (>4GB)
        }
        if line > u32::MAX as usize {
            return None; // Too many lines (>4 billion)
        }
        
        // For columns, we clamp to u16::MAX if needed
        let col_start = column_start.min(u16::MAX as usize) as u16;
        let col_end = column_end.min(u16::MAX as usize) as u16;
        
        Some(Self::new(
            line as u32,
            col_start,
            col_end,
            byte_start as u32,
            byte_end as u32,
        ))
    }
    
    /// Get the column end position
    pub fn column_end(&self) -> u32 {
        self.column_start as u32 + self.column_width as u32
    }
    
    /// Get the length of the match in bytes
    pub fn byte_length(&self) -> u32 {
        self.byte_end - self.byte_start
    }
    
    /// Get the length of the match in columns
    pub fn column_length(&self) -> u16 {
        self.column_width
    }
    
    /// Check if this coordinate contains another coordinate
    pub fn contains(&self, other: &Coordinate) -> bool {
        self.line == other.line
            && self.byte_start <= other.byte_start
            && self.byte_end >= other.byte_end
    }
    
    /// Check if this coordinate overlaps with another
    pub fn overlaps(&self, other: &Coordinate) -> bool {
        self.line == other.line
            && self.byte_start < other.byte_end
            && self.byte_end > other.byte_start
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.line,
            self.column_start,
            self.column_end()
        )
    }
}

/// A span that includes file path with coordinates
/// 
/// This is the primary type passed around, containing both
/// the file location and position within the file.
#[derive(Debug, Clone)]
pub struct FileSpan {
    /// File path (shared via Arc<str> for zero-copy)
    pub file_path: std::sync::Arc<str>,
    
    /// Position in the file (16 bytes, Copy)
    pub coordinate: Coordinate,
}

impl FileSpan {
    /// Create a new file span
    pub fn new(file_path: std::sync::Arc<str>, coordinate: Coordinate) -> Self {
        Self {
            file_path,
            coordinate,
        }
    }
}

impl fmt::Display for FileSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file_path, self.coordinate)
    }
}