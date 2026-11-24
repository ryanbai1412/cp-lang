//! `cpl check`, `cpl run` and `cpl build` report the same errors. Every
//! diagnostics golden that fails to check is given to all three, and each
//! must print the same `error[...]` headers at the same places.
//!
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.

use std::path::Path;

#[path = "support/parallel.rs"]
mod parallel;

/// The `error[...]` header and `-->` location lines `cpl <args>` prints,
/// run in process on empty input.
fn errors(args: &[&str]) -> (bool, Vec<String>) {
    let out = cpl_cli::run_with_input(args, b"");
    let text = out.stderr;
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
    (out.code == 0, lines)
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
    let compared_each = parallel::parallel(&names, |name| {
        let file = dir.join(name);
        let file = file.to_str().unwrap();
        let (ok, check) = errors(&["check", file]);
        if ok || check.is_empty() {
            return None;
        }
        let mut differ = Vec::new();
        for (cmd, args) in [
            ("run", vec!["run", file]),
            ("build", vec!["build", file, "-o", "-"]),
        ] {
            let (ok, got) = errors(&args);
            if ok || got != check {
                differ.push(format!(
                    "{name}: `cpl check` reports\n  {}\nbut `cpl {cmd}` reports\n  {}",
                    check.join("\n  "),
                    got.join("\n  ")
                ));
            }
        }
        Some(differ)
    });
    let compared = compared_each.iter().flatten().count();
    let differ: Vec<String> = compared_each.into_iter().flatten().flatten().collect();
    println!("{compared} failing goldens compared across check, run and build");
    assert!(compared > 50, "only {compared} goldens fail to check");
    assert!(
        differ.is_empty(),
        "{} disagreements:\n{}",
        differ.len(),
        differ.join("\n\n")
    );
}
