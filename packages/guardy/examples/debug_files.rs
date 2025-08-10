use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new("/home/nsm/code/deepbrain/rusttoolkit/.guardy/cache/repokit");

    // Create temporary ignore file with same exclude pattern as config
    let ignore_file = source.join(".gitignore_temp");
    fs::write(&ignore_file, ".git\n")?;

    let mut builder = WalkBuilder::new(source);
    builder.add_custom_ignore_filename(".gitignore_temp");

    println!("Files detected by get_files() logic:");
    for entry in builder.build() {
        let entry = entry?;
        if entry.path().is_file() {
            if let Ok(rel_path) = entry.path().strip_prefix(source) {
                println!("  {}", rel_path.display());
            }
        }
    }

    // Cleanup
    let _ = fs::remove_file(&ignore_file);

    Ok(())
}
