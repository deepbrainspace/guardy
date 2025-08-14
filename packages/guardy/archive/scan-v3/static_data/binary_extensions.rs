//! Binary file extensions with base + custom support
//!
//! Provides a global set of binary file extensions that is compiled once
//! and shared across all threads for O(1) lookup performance.

use anyhow::Result;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

/// Load custom binary extensions from configuration
fn load_custom_extensions() -> Result<Vec<String>> {
    // TODO: Implement loading from:
    // - ~/.config/guardy/binary_extensions.txt (one per line)
    // - Environment variable GUARDY_BINARY_EXTENSIONS (comma-separated)
    // - ScannerConfig custom extensions
    
    // For now, return empty
    Ok(Vec::new())
}

/// Global shared binary extensions - base + custom merged
pub static BINARY_EXTENSIONS: LazyLock<Arc<HashSet<String>>> = LazyLock::new(|| {
    let start = std::time::Instant::now();
    
    // Base extensions (comprehensive list from v2)
    let base_extensions = vec![
        // Images
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tiff",
        "tif", "avif", "heic", "heif", "dng", "raw", "nef", "cr2",
        "arw", "orf", "rw2", "svg",
        
        // Documents
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt",
        "ods", "odp", "indd",
        
        // Archives
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "dmg",
        "iso", "ace", "cab", "lzh", "arj", "br", "zst", "lz4",
        "lzo", "lzma",
        
        // Executables & Libraries
        "exe", "dll", "so", "dylib", "bin", "app", "deb", "rpm",
        "o", "obj", "lib", "a", "pdb", "exp", "ilk",
        
        // Audio/Video
        "mp3", "wav", "ogg", "flac", "aac", "mp4", "avi", "mkv",
        "mov", "wmv", "webm", "mp2", "m4a", "wma", "amr",
        
        // Fonts
        "ttf", "otf", "woff", "woff2", "eot",
        
        // Security/Crypto (keeping some for secret detection)
        "gpg", "pgp", "p12", "pfx", "der", "crt", "keystore",
        
        // Database & Data Files
        "db", "sqlite", "sqlite3", "mdb", "sst", "ldb", "wal", "snap",
        "dat", "sas7bdat", "sas7bcat",
        
        // CAD & Design
        "dwg", "dxf", "skp", "3ds", "max", "blend", "fbx",
        
        // Compiler & Build Artifacts
        "gcno", "gcda", "gcov", "wasm", "webc",
        
        // Binary Data & VM Images
        "img", "vhd", "vmdk", "qcow2",
        
        // Other binary formats
        "pyc", "pyo", "class", "jar", "war", "ear", "swf", "fla",
        
        // NX cache files
        "nxt",
        
        // DOS/Legacy
        "com", "bat", "cmd",
        
        // Specialized formats
        "bas", "pic", "b", "mcw", "ind", "dsk", "z",
        
        // Data formats that often cause UTF-8 issues
        "gdiff", "srt", "zeno", "cba", "parquet", "avro", "orc",
        
        // Additional problematic formats
        "pak", "rpak", "toast", "data",
    ];
    
    let mut all_extensions = HashSet::new();
    
    // Add base extensions
    for ext in base_extensions {
        all_extensions.insert(ext.to_string());
    }
    
    // Try to load and add custom extensions
    match load_custom_extensions() {
        Ok(custom) => {
            if !custom.is_empty() {
                tracing::info!("Loaded {} custom binary extensions", custom.len());
                for ext in custom {
                    all_extensions.insert(ext);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load custom binary extensions: {}", e);
        }
    }
    
    tracing::info!(
        "Binary extensions initialized with {} extensions in {:?}",
        all_extensions.len(),
        start.elapsed()
    );
    
    Arc::new(all_extensions)
});

/// Check if a file extension is binary
pub fn is_binary_extension(extension: &str) -> bool {
    BINARY_EXTENSIONS.contains(extension)
}