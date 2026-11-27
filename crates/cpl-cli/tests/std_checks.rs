//! std is ordinary CPL: `cpl check std/<file>.cp` holds it to the rules a
//! user library meets. Each file's error count may only drop, toward 0.
//! Enforces compiler-changes/R8 in
//! `handbook/guidelines/compiler-changes-checklist.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[path = "support/parallel.rs"]
mod parallel;

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
    let mut files: Vec<String> = std::fs::read_dir(root.join("crates/cpl-check/frozen/std"))
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
    // std is checked as `std/<file>.cp`, the path its modules load as, so
    // the checks run in process from the repo root.
    std::env::set_current_dir(root.join("crates/cpl-check/frozen")).unwrap();
    let counts = parallel::parallel(&files, |file| {
        let out = cpl_cli::run(&["check", file]);
        let errors = out
            .stderr
            .lines()
            .filter(|l| l.starts_with("error"))
            .count();
        (errors, out.stderr)
    });
    let mut grown = Vec::new();
    let mut shrunk = Vec::new();
    for (file, (errors, stderr)) in files.iter().zip(counts) {
        let limit = allowed.get(file).copied().unwrap_or(0);
        if errors > limit {
            // One row per file, so a red at the base still refuses a land that grows
            // another file (scripts/new_reds.py).
            grown.push(format!(
                "cpl-finding: std_errors_never_grow {file} +{}\n{file}: {errors} errors, at most {limit}\n{stderr}",
                errors - limit
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

#[test]
fn std_own_tests_pass() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files: Vec<String> = std::fs::read_dir(root.join("crates/cpl-check/frozen/std"))
        .expect("std/ exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|x| x == "cp"))
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .any(|l| l.trim_start().starts_with("@test"))
        })
        .map(|path| format!("std/{}", path.file_name().unwrap().to_str().unwrap()))
        .collect();
    files.sort();
    assert!(
        files.len() > 10,
        "only {} std files have @test fns",
        files.len()
    );
    // One `cpl test` per core: a test past the judge's 2 s limit fails, and
    // on a 4-core CI runner every file at once pushed three past it (CI run
    // 45311869328). nextest gives this test the machine (.config/nextest.toml).
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let runs: Vec<(String, std::process::Output)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (root, files, next) = (&root, &files, &next);
                s.spawn(move || {
                    let mut runs = Vec::new();
                    while let Some(file) =
                        files.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
                            .args(["test", file])
                            .current_dir(root.join("crates/cpl-check/frozen"))
                            .output()
                            .expect("cpl runs");
                        runs.push((file.clone(), out));
                    }
                    runs
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let mut problems = Vec::new();
    for (file, out) in &runs {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let fails: Vec<&str> = stdout
            .lines()
            .filter_map(|l| l.trim().strip_prefix("FAIL "))
            .collect();
        let summarized = stdout.lines().any(|l| l.contains(" passed, "));
        if !summarized || (out.status.success() == !fails.is_empty()) {
            let stderr = String::from_utf8_lossy(&out.stderr);
            problems.push(format!(
                "cpl-finding: std_own_tests_pass {file}:not-run +1\n{file}: `cpl test` did not run its tests\n{stdout}{stderr}"
            ));
        }
        for line in fails {
            let name = line.split_whitespace().next().unwrap_or(line);
            problems.push(format!(
                "cpl-finding: std_own_tests_pass {file}:{name} +1\nFAIL {line}"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "std's own @test fns:\n{}",
        problems.join("\n")
    );
}
