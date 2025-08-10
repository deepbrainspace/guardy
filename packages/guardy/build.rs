use std::process::Command;

fn main() {
    // Get git commit SHA
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap_or_else(|_| {
            // If git command fails, use a default value
            std::process::Command::new("echo")
                .arg("unknown")
                .output()
                .unwrap()
        });

    let git_sha = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    // Pass the git SHA to the compiler
    println!("cargo:rustc-env=GIT_SHA={git_sha}");

    // Get git branch
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap_or_else(|_| {
            std::process::Command::new("echo")
                .arg("unknown")
                .output()
                .unwrap()
        });

    let git_branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    println!("cargo:rustc-env=GIT_BRANCH={git_branch}");

    // Rerun build if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}