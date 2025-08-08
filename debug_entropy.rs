use guardy::scanner::entropy::{is_likely_secret, calculate_randomness_probability};

fn main() {
    let test_cases = [
        ("sk_test_4eC39HqLyjWDarjtT1zdp7dc", "Stripe test key"),
        ("ghp_1234567890abcdef1234567890abcdef12", "GitHub token"),
        ("sk-proj-4eC39HqLyjWDarjtT1zdp7dcSomeRandomText", "OpenAI test key"),
        ("API_KEY_CONSTANT", "Non-secret constant"),
        ("hello_world_test", "Simple words"),
        ("123456789", "Simple numbers"),
    ];
    
    let threshold = 1.0 / 1e5;
    
    for (test_str, desc) in test_cases {
        let bytes = test_str.as_bytes();
        let prob = calculate_randomness_probability(bytes);
        let is_secret = is_likely_secret(bytes, threshold);
        
        println\!("{}: {} (prob: {:.2e}, threshold: {:.2e}, is_secret: {})", 
                 desc, test_str, prob, threshold, is_secret);
    }
}
EOF < /dev/null
