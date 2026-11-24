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

/// The `.cp` files in `dir`, sorted. With `nested`, those in its folders
/// too.
fn sources(dir: &str, nested: bool) -> Vec<PathBuf> {
    fn walk(dir: &Path, nested: bool, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if nested {
                    walk(&path, nested, out);
                }
            } else if path.extension().is_some_and(|ext| ext == "cp") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root().join(dir), nested, &mut out);
    out.sort();
    assert!(!out.is_empty(), "no .cp files under {dir}");
    out
}

fn check(path: &Path) -> cpl_cli::Outcome {
    cpl_cli::run(&["check", path.to_str().unwrap()])
}

/// Files this checker does not take as they stand. A file leaves this
/// list when it checks clean, and the test below says so.
///
/// `compound_div` reads a word into a `[domain S]Letter`, which `read()`
/// can't size yet (sent to Core).
const PENDING: &[&str] = &["crates/cpl-cli/tests/cases/ergo/compound_div.cp"];

/// Files the checker takes and lowering does not: until `cpl check` ran
/// lowering too, it called these clean while `cpl run` and `cpl build`
/// rejected them (`research/archive/audits/anti-slop-audit-1.md`, finding 1). Each is a
/// gap in `cpl-ir` or the standard library, listed so that it is seen. A
/// file leaves this list when it builds, and the test below says so;
/// nothing is to be added to it.
const NOT_BUILT: &[&str] = &[
    "crates/cpl-cli/tests/cases/ergo/counter.cp",
    "crates/cpl-cli/tests/cases/ergo/ranges.cp",
    "crates/cpl-cli/tests/cases/lend/grid_rows.cp",
    "crates/cpl-cli/tests/cases/lend/projection.cp",
];

#[test]
fn the_libraries_ports_and_passing_tests_check_clean() {
    let mut failures = Vec::new();
    let mut checked = 0;
    /// The `spec.cp` corpus problems that currently pass `cpl check` (ratchet: only grow).
    const SPEC_PORTS: &[&str] = &[
        "aac4p6",
        "aac5p4",
        "ccc03s4",
        "ccc18s4",
        "ccc19s2",
        "cco18p4",
        "cco19p5",
        "cco21p4",
        "cco24p1",
        "ceoi19pp1",
        "dmopc20c4p4",
        "dmopc21c2p1",
        "dmopc21c4p1",
        "dmopc21c4p2",
        "dmopc21c5p2",
        "dmopc21c9p1",
        "egoi22p3",
        "etsr23r1p4",
        "ioi01p1",
        "ioi15p5",
        "ioi15p6",
        "ioi16p6",
        "ioi16p6io",
        "ioi19p5",
        "ioi19p6",
        "ioi22p5",
        "mccc4j1",
        "nccc10s4",
        "nccc8s3",
        "noip14p6",
        "sac22cc3jp2",
        "sac22cc3jp4",
        "sac22cc4jp2",
        "shoi16p3",
        "ucrpc21c",
        "yac6p3",
    ];
    for id in SPEC_PORTS {
        let path = root().join(format!("corpus/problems/{id}/spec.cp"));
        checked += 1;
        let out = check(&path);
        if out.code != 0 {
            failures.push(out.stderr);
        }
    }
    /// The `std/*.cp` modules that currently pass standalone `cpl check` (ratchet: only grow).
    const SPEC_STD: &[&str] = &[
        "deque.cp",
        "dsu.cp",
        "interval.cp",
        "io.cp",
        "mo.cp",
        "opt.cp",
        "pair.cp",
        "rand.cp",
        "slope.cp",
        "timetree.cp",
        "tree.cp",
        "window.cp",
    ];
    for file in SPEC_STD {
        let path = root().join(format!("std/{file}"));
        checked += 1;
        let out = check(&path);
        if out.code != 0 {
            failures.push(out.stderr);
        }
    }
    for (dir, nested) in [
        ("crates/cpl-cli/tests/cases/lend", false),
        ("crates/cpl-cli/tests/cases/ergo", false),
        ("crates/cpl-cli/tests/cases/ok", false),
        ("crates/cpl-cli/tests/cases/fn", false),
        ("crates/cpl-cli/tests/cases/redteam", false),
    ] {
        for path in sources(dir, nested) {
            checked += 1;
            let out = check(&path);
            let pending = PENDING
                .iter()
                .chain(NOT_BUILT)
                .any(|file| path.ends_with(file));
            if pending && out.code == 0 {
                failures.push(format!(
                    "{} checks clean now: take it off `PENDING` or `NOT_BUILT`",
                    path.display()
                ));
            } else if !pending && out.code != 0 {
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

/// Failing tests that state a limit of the prototype's code generator,
/// not a rule of the language. `a[nx()] max= 4` evaluates `a[nx()]` once
/// here (`tests/programs/semantics.cp`), and simultaneous tuple assignment
/// `(a, b) = (b, a + b)` is supported across `std/`.
const PROTOTYPE_ONLY: &[&str] = &["rt_minassign_twice", "tuple_assign"];

/// The reason a failing test states on its first line: `// error: <text>`.
fn stated_reason(text: &str) -> Option<&str> {
    text.lines().next()?.strip_prefix("// error: ")
}

#[test]
fn each_failing_test_is_rejected_for_its_stated_reason() {
    let mut wrong = Vec::new();
    let mut cases = 0;
    for path in sources("crates/cpl-cli/tests/cases/bad", false) {
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
        cases += 1;
        if PROTOTYPE_ONLY.contains(&name.as_str()) {
            continue;
        }
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
