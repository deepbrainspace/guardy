use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new("/home/nsm/code/deepbrain/rusttoolkit/.guardy/cache/repokit");

    // Test 1: Without ignore file
    println!("=== WITHOUT IGNORE ===");
    let builder = WalkBuilder::new(source);
    for entry in builder.build() {
        let entry = entry?;
        if let Ok(rel_path) = entry.path().strip_prefix(source)
            && rel_path.to_string_lossy() != ""
        {
            // Skip root
            println!("  {}", rel_path.display());
        }
    }

    // Test 2: With ignore file
    println!("\n=== WITH .git IGNORE ===");
    let ignore_file = source.join(".gitignore_temp");
    fs::write(&ignore_file, ".git\n")?;

    let mut builder = WalkBuilder::new(source);
    builder.add_custom_ignore_filename(".gitignore_temp");

    for entry in builder.build() {
        let entry = entry?;
        if let Ok(rel_path) = entry.path().strip_prefix(source)
            && rel_path.to_string_lossy() != ""
        {
            // Skip root
            println!("  {}", rel_path.display());
        }
    }

    // Cleanup
    let _ = fs::remove_file(&ignore_file);

    Ok(())
}
