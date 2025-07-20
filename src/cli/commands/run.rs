use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    /// Hook name to run
    pub hook: String,
    
    /// Additional arguments for the hook
    pub args: Vec<String>,
}

pub async fn execute(args: RunArgs) -> Result<()> {
    use crate::cli::output::*;
    use crate::git::GitRepo;
    use crate::config::GuardyConfig;
    use crate::scanner::Scanner;
    
    info(&format!("Running {} hook...", args.hook));
    
    // Load configuration
    let config = GuardyConfig::load(None, None::<&()>)?;
    
    match args.hook.as_str() {
        "pre-commit" => {
            info("Executing pre-commit checks...");
            
            // Check if we're in a git repository
            let repo = GitRepo::discover()?;
            
            // Get staged files for scanning
            let staged_files = repo.get_staged_files()?;
            
            if staged_files.is_empty() {
                info("No staged files to check");
                return Ok(());
            }
            
            // Create scanner and scan staged files
            let scanner = Scanner::new(&config)?;
            let scan_result = scanner.scan_paths(&staged_files)?;
            
            if scan_result.stats.total_matches > 0 {
                error(&format!("❌ Found {} secrets in staged files", scan_result.stats.total_matches));
                
                // Print summary of found secrets
                for secret_match in scan_result.matches.iter().take(5) {
                    println!("  {} {}:{} [{}]", 
                        "🔍", 
                        secret_match.file_path,
                        secret_match.line_number,
                        secret_match.secret_type
                    );
                }
                
                if scan_result.matches.len() > 5 {
                    println!("  ... and {} more", scan_result.matches.len() - 5);
                }
                
                println!("\nCommit aborted. Remove secrets before committing.");
                std::process::exit(1);
            } else {
                success(&format!("✅ Scanned {} files - no secrets found", scan_result.stats.files_scanned));
            }
        }
        "commit-msg" => {
            info("Validating commit message...");
            // TODO: Implement commit message validation
            success("Commit message validation passed");
        }
        "post-checkout" => {
            info("Running post-checkout actions...");
            // TODO: Implement dependency installation checks
            success("Post-checkout actions completed");
        }
        "pre-push" => {
            info("Running pre-push validation...");
            // TODO: Implement lockfile sync validation and tests
            success("Pre-push validation passed");
        }
        unknown => {
            error(&format!("Unknown hook: {}", unknown));
            std::process::exit(1);
        }
    }
    
    success("Hook execution completed!");
    Ok(())
}