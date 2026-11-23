//! The editor shows the errors `cpl check` prints: every golden in
//! `crates/cpl-cli/tests/diagnostics/` gets the same codes on the same lines.

use super::*;

/// `(code, 1-based line)` of each error, and of each warning, in a golden.
fn golden_diagnostics(err: &str) -> (Vec<(String, u32)>, Vec<(String, u32)>) {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let mut lines = err.lines().peekable();
    while let Some(line) = lines.next() {
        let (list, rest) = if let Some(rest) = line.strip_prefix("error[") {
            (&mut errors, rest)
        } else if let Some(rest) = line.strip_prefix("warning[") {
            (&mut warnings, rest)
        } else {
            continue;
        };
        let Some((code, _)) = rest.split_once(']') else {
            continue;
        };
        let at = lines
            .peek()
            .and_then(|l| l.trim_start().strip_prefix("--> "));
        let Some(number) = at.and_then(|at| at.rsplit(':').nth(1)?.parse().ok()) else {
            continue;
        };
        list.push((code.to_string(), number));
    }
    (errors, warnings)
}

#[test]
fn the_editor_shows_the_errors_the_cli_prints() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../cpl-cli/tests/diagnostics");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    paths.sort();
    let mut wrong = Vec::new();
    let mut compared = 0;
    for path in &paths {
        let Ok(err) = std::fs::read_to_string(path.with_extension("err")) else {
            continue;
        };
        let source = std::fs::read_to_string(path).unwrap();
        let (cli_errors, cli_warnings) = golden_diagnostics(&err);
        let (_, published) = open(&source);
        let shown: Vec<(String, u32, u32)> = published
            .iter()
            .map(|d| {
                let code = d.at(&["code"]).as_str().unwrap_or("").to_string();
                let line = d.at(&["range", "start", "line"]).as_u32().unwrap_or(0) + 1;
                (code, line, d.at(&["severity"]).as_u32().unwrap_or(0))
            })
            .collect();
        let mut editor_errors: Vec<(String, u32)> = shown
            .iter()
            .filter(|d| d.2 == 1)
            .map(|(code, line, _)| (code.clone(), *line))
            .collect();
        let mut cli_errors = cli_errors;
        editor_errors.sort();
        cli_errors.sort();
        let missing_warnings: Vec<_> = cli_warnings
            .iter()
            .filter(|(code, line)| !shown.iter().any(|d| &d.0 == code && d.1 == *line))
            .collect();
        if editor_errors != cli_errors || !missing_warnings.is_empty() {
            wrong.push(format!(
                "{}: cli {cli_errors:?} {cli_warnings:?}, editor {shown:?}",
                path.file_name().unwrap().to_string_lossy()
            ));
        }
        compared += 1;
    }
    assert!(compared >= 100, "only {compared} goldens compared");
    assert!(
        wrong.is_empty(),
        "{} of {compared} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
