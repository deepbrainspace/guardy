use ignore::WalkBuilder;
use std::path::Path;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new("/home/nsm/code/deepbrain/rusttoolkit/.guardy/cache/repokit");
    
    // Create temporary ignore file
    let ignore_file = source.join(".gitignore_temp");
    fs::write(&ignore_file, ".git\n")?;
    
    let mut builder = WalkBuilder::new(source);
    builder.add_custom_ignore_filename(".gitignore_temp");
    
    println!("Files found:");
    for entry in builder.build() {
        let entry = entry?;
        if entry.path().is_file() || entry.path().is_dir() {
            if let Ok(rel_path) = entry.path().strip_prefix(source) {
                println!("  {}", rel_path.display());
            }
        }
    }
    
    // Cleanup
    let _ = fs::remove_file(&ignore_file);
    
    Ok(())
}