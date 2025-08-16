use superconfig::FastConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TestConfig {
    name: String,
    port: u16,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to see our timing logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_level(true)
        .with_target(false)
        .init();

    // Create a test config file
    let test_config = TestConfig {
        name: "timing-test".to_string(),
        port: 8080,
        debug: true,
    };
    
    let config_json = serde_json::to_string_pretty(&test_config)?;
    std::fs::write("timing_test.json", &config_json)?;
    
    println!("🚀 Starting timing test with detailed tracing...\n");
    
    // Clear any existing cache first (only if cache feature enabled)
    cfg_if::cfg_if! {
        if #[cfg(feature = "cache")] {
            if let Ok(cache_manager) = superconfig::CacheManager::new("timing_test") {
                let _ = cache_manager.clear_cache();
            }
        }
    }
    
    // Test cold load
    println!("=== COLD LOAD TEST ===");
    let start = std::time::Instant::now();
    let config = FastConfig::<TestConfig>::load("timing_test")?;
    let total_time = start.elapsed();
    
    println!("\n✅ Cold load completed in {:?}", total_time);
    println!("Config loaded: name={}, port={}", config.get().name, config.get().port);
    
    // Test subsequent loads (direct parsing by default, cache if feature enabled)
    println!("\n=== SECOND LOAD TEST ===");
    std::thread::sleep(std::time::Duration::from_millis(100)); // Give background cache time if enabled
    
    let start = std::time::Instant::now();
    let _config2 = FastConfig::<TestConfig>::load("timing_test")?;
    let second_time = start.elapsed();
    
    cfg_if::cfg_if! {
        if #[cfg(feature = "cache")] {
            println!("\n✅ Second load completed in {:?} (cached)", second_time);
        } else {
            println!("\n✅ Second load completed in {:?} (direct parsing)", second_time);
        }
    }
    
    // Cleanup
    std::fs::remove_file("timing_test.json").ok();
    
    Ok(())
}