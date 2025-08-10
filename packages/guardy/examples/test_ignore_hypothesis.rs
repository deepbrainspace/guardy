use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new("/home/nsm/code/deepbrain/rusttoolkit/.guardy/cache/repokit");

    println!("=== TEST 1: Default WalkBuilder (auto-discovers .gitignore) ===");
    let builder = WalkBuilder::new(source);
    for entry in builder.build() {
        let entry = entry?;
        if entry.path().is_file()
            && let Ok(rel_path) = entry.path().strip_prefix(source)
        {
            println!("  {}", rel_path.display());
        }
    }

    println!("\n=== TEST 2: Disabled auto-ignore discovery ===");
    let mut builder = WalkBuilder::new(source);
    builder.standard_filters(false); // This disables all standard ignore files

    for entry in builder.build() {
        let entry = entry?;
        if entry.path().is_file()
            && let Ok(rel_path) = entry.path().strip_prefix(source)
        {
            println!("  {}", rel_path.display());
        }
    }

    println!("\n=== TEST 3: What's in the cached repo's .gitignore? ===");
    let gitignore_content = fs::read_to_string(source.join(".gitignore"))?;
    println!("Cached .gitignore content:");
    println!("{gitignore_content}");

    Ok(())
}
