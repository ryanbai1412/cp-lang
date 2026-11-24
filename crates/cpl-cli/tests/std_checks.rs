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

/// `@test` fns in std that fail today because of a gap outside std, each with
/// its reason. Drop a line when its test passes; the test below says so.
const KNOWN_FAILING_STD_TESTS: &[(&str, &str)] = &[];

#[test]
fn std_own_tests_pass() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files: Vec<String> = std::fs::read_dir(root.join("std"))
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
                            .current_dir(root)
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
    let mut failed = Vec::new();
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
                "{file}: `cpl test` did not run its tests\n{stdout}{stderr}"
            ));
        }
        for line in fails {
            let name = line.split(": ").next().unwrap_or(line);
            failed.push(name.to_string());
            if !KNOWN_FAILING_STD_TESTS
                .iter()
                .any(|(known, _)| *known == name)
            {
                problems.push(format!("FAIL {line}"));
            }
        }
    }
    for (known, why) in KNOWN_FAILING_STD_TESTS {
        if !failed.iter().any(|f| f == known) {
            problems.push(format!(
                "{known} now passes ({why}); drop it from KNOWN_FAILING_STD_TESTS"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "std's own @test fns:\n{}",
        problems.join("\n")
    );
}
