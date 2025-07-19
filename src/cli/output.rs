use console::{style, Emoji};

// Professional symbols matching Claude Code style  
const SUCCESS: Emoji = Emoji("✔", "✓");
const WARNING: Emoji = Emoji("⚠", "!");
const INFO: Emoji = Emoji("ℹ", "i");

// Output functions matching the goodiebag hooks style
pub fn success(message: &str) {
    println!("{} {}", style(SUCCESS).green().bold(), style(message).green());
}

pub fn warning(message: &str) {
    println!("{} {}", style(WARNING).yellow().bold(), style(message).yellow());
}

pub fn info(message: &str) {
    println!("{} {}", style(INFO).blue().bold(), style(message).blue());
}