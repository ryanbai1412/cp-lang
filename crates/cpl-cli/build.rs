use std::process::Command;

fn main() {
    let commit = std::env::var("CPL_COMMIT").ok().filter(|s| !s.is_empty()).or_else(|| {
        Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    })
    .unwrap_or_else(|| "unknown".to_string());
    let channel = std::env::var("CPL_CHANNEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "dev".to_string());
    println!("cargo:rustc-env=CPL_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=CPL_CHANNEL={channel}");
    println!("cargo:rerun-if-env-changed=CPL_CHANNEL");
    println!("cargo:rerun-if-env-changed=CPL_COMMIT");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
