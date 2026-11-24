//! End-to-end tests for the commands no other test ran through the binary:
//! each one's `--help`, a success, and a usage error with its own usage.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use cpl_cli::{EXIT_ERRORS, EXIT_FAILED, EXIT_USAGE};

fn cpl(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("cpl runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// What `cpl` printed, with Windows' CRLF line ends as LF, so one expected
/// text holds on every OS.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

/// `--help` prints the command's own usage on stdout and exits 0.
/// Whether `text` holds `command`'s usage line, `usage: cpl <command>` alone
/// or with arguments.
fn shows_usage(text: &str, command: &str) -> bool {
    text.lines().any(|line| {
        line.split_whitespace()
            .take(3)
            .eq(["usage:", "cpl", command])
    })
}

fn helps(command: &str) {
    let out = cpl(Path::new("."), &[command, "--help"]);
    assert_eq!(out.status.code(), Some(0), "{command} --help");
    let usage = text(&out.stdout);
    assert!(
        shows_usage(usage.lines().next().unwrap_or(""), command),
        "{usage}"
    );
    assert!(out.stderr.is_empty());
}

/// A usage error names the problem, then shows the command's usage, not
/// the list of every command.
fn misused(command: &str, args: &[&str], message: &str) {
    let out = cpl(Path::new("."), args);
    assert_eq!(out.status.code(), Some(EXIT_USAGE.into()), "{args:?}");
    let stderr = text(&out.stderr);
    assert!(stderr.contains(message), "{stderr}");
    assert!(shows_usage(&stderr, command), "{stderr}");
    assert!(!stderr.contains("commands:"), "{stderr}");
}

mod explain;
mod help;
mod project;
mod run;
mod setup;
mod style;
