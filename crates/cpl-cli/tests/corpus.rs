//! The cp source in this repository, through `cpl check`.
//!
//! Everything written for the language (the standard library, the sample
//! local library, the ported solutions and the prototype's passing tests)
//! must check with no diagnostics, and each of the prototype's failing
//! tests must be rejected for the reason its first line gives.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.cp` file under `dir`, sorted.
fn sources(dir: &str) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "cp") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root().join(dir), &mut out);
    out.sort();
    assert!(!out.is_empty(), "no .cp files under {dir}");
    out
}

fn check(path: &Path) -> cpl_cli::Outcome {
    cpl_cli::run(&["check", path.to_str().unwrap()])
}

#[test]
fn the_libraries_ports_and_passing_tests_check_clean() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for dir in [
        "stdlib",
        "examples/local-lib",
        "proto/ports",
        "proto/tests/lend",
        "proto/tests/ok",
        "proto/tests/fn",
    ] {
        for path in sources(dir) {
            checked += 1;
            let out = check(&path);
            if out.code != 0 {
                failures.push(out.stderr);
            }
        }
    }
    assert!(checked >= 80, "only {checked} files were found");
    assert!(
        failures.is_empty(),
        "{} of {checked} files have diagnostics:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The reason a failing test states on its first line: `// error: <text>`.
fn stated_reason(text: &str) -> Option<&str> {
    text.lines().next()?.strip_prefix("// error: ")
}

#[test]
fn each_failing_test_is_rejected_for_its_stated_reason() {
    let mut wrong = Vec::new();
    let mut cases = 0;
    for path in sources("proto/tests/bad") {
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let Some(reason) = stated_reason(&text) else {
            // `ok_*.cp` sit next to the failing cases they are the fixed
            // versions of.
            if name.starts_with("ok_") {
                let out = check(&path);
                if out.code != 0 {
                    wrong.push(format!("{name} should check clean:\n{}", out.stderr));
                }
            }
            continue;
        };
        // The prototype says "index of type Other into array over Cell";
        // this checker says which array, and what it is indexed by.
        let reason = match name.as_str() {
            "index_wrong_struct" => "`d` is indexed by Cell, but `Other(Row.first)` is an Other",
            _ => reason,
        };
        cases += 1;
        let out = check(&path);
        if out.code == 0 {
            wrong.push(format!("{name} was accepted; expected: {reason}"));
        } else if !out.stderr.contains(reason) {
            wrong.push(format!("{name}: expected `{reason}` in:\n{}", out.stderr));
        }
    }
    assert!(cases >= 29, "only {cases} failing tests state a reason");
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
