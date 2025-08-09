use guardy::sync::manager::SyncManager;
use guardy::sync::{SyncRepo, SyncConfig};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the same sync config as in guardy.yaml
    let sync_repo = SyncRepo {
        name: "repokit".to_string(),
        repo: "git@github.com:deepbrainspace/repokit.git".to_string(),
        version: "main".to_string(),
        source_path: ".".to_string(),
        dest_path: ".".to_string(),
        include: vec!["*".to_string()],
        exclude: vec![".git".to_string()],
        protected: true,
    };
    
    let sync_config = SyncConfig {
        repos: vec![sync_repo],
        protection: Default::default(),
    };
    
    let manager = SyncManager::with_config(sync_config)?;
    
    println!("=== Calling check_sync_status() ===");
    let status = manager.check_sync_status()?;
    println!("Status: {:?}", status);
    
    Ok(())
}