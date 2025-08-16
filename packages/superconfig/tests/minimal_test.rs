//! Minimal test to see what the macro actually generates

use superconfig::config;

// This should generate the SampleConfig struct and impl from sample.json
config!("sample" => SampleConfig);

#[test]
fn check_compilation() {
    // Test that the struct exists and can be accessed
    let config = SampleConfig::global();

    // Debug: Print what we got
    println!("Config server.port: {}", config.server.port);
    println!("Config server.host: {}", config.server.host);
    println!("Config debug: {}", config.debug);

    // For now, just test that the struct compiles and has the right fields
    // The values might be default (0, empty string, false) if loading fails
    let _ = config.server.port;
    let _ = config.server.host;
    let _ = config.debug;

    // If the config loaded correctly, these should be the actual values
    // If not, they'll be default values (which is OK for this compilation test)
}
