//! `cpl check`, `cpl run` and `cpl build` report the same errors. Every
//! diagnostics golden that fails to check is given to all three, and each
//! must print the same `error[...]` headers at the same places.
//!
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// The `error[...]` header and `-->` location lines `cpl <args>` prints.
fn errors(args: &[&str], dir: &Path) -> (bool, Vec<String>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdin.take().map(|mut s| s.write_all(b"")));
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stderr).into_owned();
    let mut lines = Vec::new();
    let mut in_error = false;
    for line in text.lines() {
        if line.starts_with("error[") {
            in_error = true;
            lines.push(line.to_string());
        } else if line.starts_with("warning") {
            in_error = false;
        } else if in_error && line.trim_start().starts_with("--> ") {
            lines.push(line.trim().to_string());
            in_error = false;
        }
    }
    (out.status.success(), lines)
}

#[test]
fn check_run_and_build_report_the_same_errors() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/diagnostics");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".cp"))
        .collect();
    names.sort();
    let mut compared = 0;
    let mut differ = Vec::new();
    for name in &names {
        let (ok, check) = errors(&["check", name], &dir);
        if ok || check.is_empty() {
            continue;
        }
        compared += 1;
        for (cmd, args) in [
            ("run", vec!["run", name.as_str()]),
            ("build", vec!["build", name.as_str(), "-o", "-"]),
        ] {
            let (ok, got) = errors(&args, &dir);
            if ok || got != check {
                differ.push(format!(
                    "{name}: `cpl check` reports\n  {}\nbut `cpl {cmd}` reports\n  {}",
                    check.join("\n  "),
                    got.join("\n  ")
                ));
            }
        }
    }
    println!("{compared} failing goldens compared across check, run and build");
    assert!(compared > 50, "only {compared} goldens fail to check");
    assert!(
        differ.is_empty(),
        "{} disagreements:\n{}",
        differ.len(),
        differ.join("\n\n")
    );
}
