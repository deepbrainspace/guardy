use regex::Regex;

fn main() {
    println!("Testing pattern matching issues...\n");

    // Test data from integration tests
    let test_lines = vec![
        "STRIPE_KEY=***REMOVED***",
        "GITHUB_TOKEN=ghp_1234567890abcdef1234567890abcdef12",
        "\"openai_api_key\": \"sk-proj-ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234567890\"",
        "\"jwt_secret\": \"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature\""
    ];

    // Test patterns that should match
    let patterns = vec![
        ("Stripe API Key", r"[rs]k_live_[\dA-Za-z]{24,247}"),
        ("GitHub Token", r"(?:gh[oprsu]|github_pat)_[\dA-Za-z_]{36}"),
        ("OpenAI API Key", r"sk-proj-[\dA-Za-z]{43,64}"),
        ("JWT Token", r"\beyJ[\dA-Za-z=_-]+(?:\.[\dA-Za-z=_-]{3,}){1,4}"),
        ("Generic Secret", r"(?i:key|token|secret|password|api|auth|credential|pass)\w*[\x22']?]?\s*(?:[:=]|:=|=>|<-|>)\s*[\t \x22'\x60]?([\w+./=~\-\\\x60\^]{15,90})")
    ];

    for (name, pattern_str) in &patterns {
        println!("Testing pattern: {}", name);
        println!("Regex: {}", pattern_str);

        match Regex::new(pattern_str) {
            Ok(regex) => {
                for (i, line) in test_lines.iter().enumerate() {
                    if regex.is_match(line) {
                        println!("  ✓ Line {}: MATCHES - {}", i+1, line);
                        // Show what matched
                        for m in regex.find_iter(line) {
                            println!("    Match: '{}'", m.as_str());
                        }
                    } else {
                        println!("  ✗ Line {}: no match - {}", i+1, line);
                    }
                }
            }
            Err(e) => {
                println!("  ERROR: Invalid regex - {}", e);
            }
        }
        println!();
    }
}
