//! std is ordinary CPL: `cpl check std/<file>.cp` holds it to the rules a
//! user library meets. Each file's error count may only drop, toward 0.
//! Enforces compiler-changes/R8 in
//! `handbook/guidelines/compiler-changes-checklist.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

fn baseline(root: &Path) -> BTreeMap<String, usize> {
    let text = std::fs::read_to_string(root.join("crates/cpl-cli/tests/std-check-errors.txt"))
        .expect("the baseline exists");
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let (file, count) = line.rsplit_once(' ').expect("`<file> <count>`");
            (file.to_string(), count.parse().expect("a count"))
        })
        .collect()
}

#[test]
fn std_errors_never_grow() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let allowed = baseline(&root);
    let mut files: Vec<String> = std::fs::read_dir(root.join("std"))
        .expect("std/ exists")
        .map(|entry| {
            entry
                .expect("readable entry")
                .file_name()
                .into_string()
                .unwrap()
        })
        .filter(|name| name.ends_with(".cp"))
        .map(|name| format!("std/{name}"))
        .collect();
    files.sort();
    let mut grown = Vec::new();
    let mut shrunk = Vec::new();
    for file in &files {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["check", file])
            .current_dir(&root)
            .output()
            .expect("cpl runs");
        let stderr = String::from_utf8_lossy(&out.stderr);
        let errors = stderr.lines().filter(|l| l.starts_with("error")).count();
        let limit = allowed.get(file).copied().unwrap_or(0);
        if errors > limit {
            grown.push(format!(
                "{file}: {errors} errors, at most {limit}\n{stderr}"
            ));
        } else if errors < limit {
            shrunk.push(format!("{file} {errors}"));
        }
    }
    if !shrunk.is_empty() {
        eprintln!(
            "std got cleaner; lower tests/std-check-errors.txt to:\n{}",
            shrunk.join("\n")
        );
    }
    assert!(
        grown.is_empty(),
        "`cpl check` errors in std grew:\n{}",
        grown.join("\n")
    );
}
