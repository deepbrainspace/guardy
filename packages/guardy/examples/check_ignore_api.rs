use ignore::{WalkBuilder, gitignore::GitignoreBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test if we can build gitignore patterns in memory
    println!("Testing in-memory gitignore API...");
    
    let mut builder = GitignoreBuilder::new("/tmp");
    match builder.add_line(None, ".git") {
        Ok(_) => println!("✅ GitignoreBuilder.add_line() works - we can add patterns in memory!"),
        Err(e) => println!("❌ GitignoreBuilder.add_line() failed: {}", e),
    }
    
    match builder.build() {
        Ok(gitignore) => {
            println!("✅ Built in-memory gitignore successfully");
            
            // Test if WalkBuilder can use in-memory gitignore
            let mut walk_builder = WalkBuilder::new("/tmp");
            // Check if there's a method to add the gitignore object directly
            println!("WalkBuilder methods available:");
            println!("- standard_filters()");
            println!("- add_custom_ignore_filename()");
            // walk_builder.add_ignore(gitignore); // This would be ideal
        },
        Err(e) => println!("❌ Failed to build gitignore: {}", e),
    }
    
    Ok(())
}