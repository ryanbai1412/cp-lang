use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
    let commit = std::env::var("CPL_COMMIT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| git(&["rev-parse", "--short", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    let channel = std::env::var("CPL_CHANNEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "dev".to_string());
    println!("cargo:rustc-env=CPL_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=CPL_CHANNEL={channel}");
    println!("cargo:rerun-if-env-changed=CPL_CHANNEL");
    println!("cargo:rerun-if-env-changed=CPL_COMMIT");
    // A missing path reruns this script (and rebuilds the crate) on every
    // build, and in a git worktree `.git` is a file, so ask git where HEAD is.
    match git(&["rev-parse", "--absolute-git-dir"]) {
        Some(dir) => println!("cargo:rerun-if-changed={dir}/HEAD"),
        None => println!("cargo:rerun-if-changed=build.rs"),
    }
}
