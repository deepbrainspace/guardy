//! Common binary file extensions

use std::collections::HashSet;
use std::sync::LazyLock;

/// Global set of binary file extensions
pub static BINARY_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    
    // Executables
    set.insert("exe");
    set.insert("dll");
    set.insert("so");
    set.insert("dylib");
    set.insert("bin");
    
    // Images
    set.insert("jpg");
    set.insert("jpeg");
    set.insert("png");
    set.insert("gif");
    set.insert("bmp");
    set.insert("ico");
    set.insert("svg");
    set.insert("webp");
    
    // Archives
    set.insert("zip");
    set.insert("tar");
    set.insert("gz");
    set.insert("bz2");
    set.insert("7z");
    set.insert("rar");
    
    // Documents
    set.insert("pdf");
    set.insert("doc");
    set.insert("docx");
    set.insert("xls");
    set.insert("xlsx");
    set.insert("ppt");
    set.insert("pptx");
    
    // Media
    set.insert("mp3");
    set.insert("mp4");
    set.insert("avi");
    set.insert("mov");
    set.insert("wav");
    set.insert("flac");
    
    // Fonts
    set.insert("ttf");
    set.insert("otf");
    set.insert("woff");
    set.insert("woff2");
    
    set
});