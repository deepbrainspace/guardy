//! Test multiple config macros in the same file

use fast_config::config;

// First config
config!("sample" => FirstConfig);

// Second config - this would cause error module conflicts if not fixed
config!("yamltest" => SecondConfig);

#[test]
fn test_multiple_configs() {
    // Test that both configs can coexist
    let first = FirstConfig::global();
    let second = SecondConfig::global();

    println!("First config debug: {}", first.debug);
    println!("Second config app.name: {}", second.app.name);

    // Just test compilation - values will be defaults
    let _ = first.server.port;
    let _ = second.server.port;
}
