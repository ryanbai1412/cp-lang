//! A small edit reformats only itself (`handbook/guidelines/lints-checklist.md`,
//! `lints/R22`): in every formatted port, spec and std file, one line at a time
//! gets trailing spaces, a doubled space or a dropped space after a comma, and
//! `cpl fmt` must give back the original file exactly, touching no other line.

use std::path::{Path, PathBuf};

fn sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root.join("corpus/problems"))
        .unwrap()
        .flatten()
    {
        for name in ["port.cp", "spec.cp"] {
            files.push(entry.path().join(name));
        }
    }
    for entry in std::fs::read_dir(root.join("std")).unwrap().flatten() {
        files.push(entry.path());
    }
    files.retain(|p| p.extension().is_some_and(|e| e == "cp") && p.is_file());
    files.sort();
    files
}

fn edits(line: &str) -> Vec<(&'static str, String)> {
    let body = line.trim_start();
    let indent = line.len() - body.len();
    let mut out = vec![("trailing spaces", format!("{line}   "))];
    if let Some(i) = body.find(' ') {
        let at = indent + i;
        out.push(("doubled space", format!("{} {}", &line[..at], &line[at..])));
    }
    if let Some(i) = line.find(", ") {
        out.push((
            "dropped space after comma",
            format!("{}{}", &line[..i + 1], &line[i + 2..]),
        ));
    }
    out
}

#[test]
fn a_one_line_spacing_edit_reformats_only_that_line() {
    let (mut tried, mut failures) = (0, Vec::new());
    for path in sources() {
        let src = std::fs::read_to_string(&path).unwrap();
        if src.contains("\"\"\"") || crate::format(&src).ok().as_deref() != Some(src.as_str()) {
            continue;
        }
        let lines: Vec<&str> = src.split('\n').collect();
        let plain =
            |l: &&str| !l.trim().is_empty() && !l.contains(['"', '\'', '#']) && !l.contains("//");
        let picks: Vec<usize> = (0..lines.len()).filter(|&i| plain(&lines[i])).collect();
        for &i in picks.iter().step_by((picks.len() / 6).max(1)) {
            for (what, edited_line) in edits(lines[i]) {
                let mut edited = lines.clone();
                edited[i] = &edited_line;
                tried += 1;
                if crate::format(&edited.join("\n")).ok().as_deref() != Some(src.as_str()) {
                    failures.push(format!("{}:{}: {what}", path.display(), i + 1));
                }
            }
        }
    }
    assert!(tried > 1000, "only {tried} edits tried");
    assert!(
        failures.is_empty(),
        "{} of {tried} edits reformatted more than the edit:\n{}",
        failures.len(),
        failures[..failures.len().min(15)].join("\n")
    );
}
