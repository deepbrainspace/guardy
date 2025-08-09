use std::path::{Path, PathBuf};
use std::fs;

fn main() {
    let src = Path::new("/home/nsm/code/deepbrain/rusttoolkit/.guardy/cache/repokit");
    let dst = Path::new("/home/nsm/code/deepbrain/rusttoolkit");
    let file = PathBuf::from(".gitignore");
    
    let src_file = src.join(&file);
    let dst_file = dst.join(&file);
    
    println!("Source file: {}", src_file.display());
    println!("Dest file: {}", dst_file.display());
    println!("Source exists: {}", src_file.exists());
    println!("Dest exists: {}", dst_file.exists());
    
    if let (Ok(src_meta), Ok(dst_meta)) = (fs::metadata(&src_file), fs::metadata(&dst_file)) {
        println!("Source size: {}", src_meta.len());
        println!("Dest size: {}", dst_meta.len());
        println!("Sizes equal: {}", src_meta.len() == dst_meta.len());
    }
}