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

/// Files written in syntax that reached `main` after this checker did,
/// which it does not read yet. A file leaves this list when it checks
/// clean, and the test below says so.
const PENDING: &[&str] = &[];

/// Passing tests the language has since moved away from, which the
/// prototype rejects too: `offset_check` indexes with `i + 1`, and a step
/// is a `?V` now.
const STALE: &[&str] = &["proto/tests/ok/offset_check.cp"];

#[test]
fn the_libraries_ports_and_passing_tests_check_clean() {
    let mut failures = Vec::new();
    let mut checked = 0;
    // Not here: `proto/ports/scout_batch_*`, which are sketches of code
    // the language does not accept yet (their `// WISH` lines), and
    // `hillclimb_dp_graph/before`, the versions its ports replaced.
    for (dir, nested) in [
        ("stdlib", false),
        ("examples/local-lib", true),
        ("proto/ports", false),
        ("proto/ports/hillclimb_dp_graph", false),
        ("proto/tests/lend", false),
        ("proto/tests/ergo", false),
        ("proto/tests/ok", false),
        ("proto/tests/fn", false),
        ("proto/tests/redteam", false),
    ] {
        for path in sources(dir, nested) {
            if STALE.iter().any(|file| path.ends_with(file)) {
                continue;
            }
            checked += 1;
            let out = check(&path);
            let pending = PENDING.iter().any(|file| path.ends_with(file));
            if pending && out.code == 0 {
                failures.push(format!(
                    "{} checks clean now: take it off `PENDING`",
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

/// Failing tests this checker still accepts: each needs a rule it does
/// not have yet. A name leaves this list when its case is rejected for
/// its stated reason, and the test below says so.
const NOT_YET: &[&str] = &[];

/// Failing tests that state a limit of the prototype's code generator,
/// not a rule of the language. `a[nx()] max= 4` evaluates `a[nx()]` once
/// here (`tests/programs/semantics.cp`), and simultaneous tuple assignment
/// `(a, b) = (b, a + b)` is supported across `stdlib/` and `examples/local-lib/`.
const PROTOTYPE_ONLY: &[&str] = &["rt_minassign_twice", "tuple_assign"];

/// The reason a failing test states on its first line: `// error: <text>`.
fn stated_reason(text: &str) -> Option<&str> {
    text.lines().next()?.strip_prefix("// error: ")
}

#[test]
fn each_failing_test_is_rejected_for_its_stated_reason() {
    let mut wrong = Vec::new();
    let mut cases = 0;
    for path in sources("proto/tests/bad", false) {
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
        // this checker says which array, and what it is indexed by, and
        // why a name can't be assigned. And a `lend` need not be last:
        // what follows it is the write-back.
        let reason = match name.as_str() {
            "index_wrong_struct" => "`d` is indexed by Cell, but `Other(Row.first)` is an Other",
            "lend_outside_projection" => "`lend` is a statement of a projection's body",
            "counter_not_var" => "`seen` is a `let`, so it can't be assigned",
            "use_unknown_name" => "`std.dsu` has no `Union`",
            "span_two_domains" => "span of an I and a J",
            "rt_min_empty_slice" => "`k` is declared an Int, but `max(a[f..f])` is an ?Int",
            // A message names no line: the loop is pointed at instead.
            "rt_for_fn_push" => {
                "`grow` can't be called here: it changes `xs`, and the `for` loop is going over `xs`"
            }
            "rt_for_push" => "the `for` loop is going over `xs`",
            _ => reason,
        };
        cases += 1;
        if PROTOTYPE_ONLY.contains(&name.as_str()) {
            continue;
        }
        let out = check(&path);
        if NOT_YET.contains(&name.as_str()) {
            if out.code != 0 && out.stderr.contains(reason) {
                wrong.push(format!("{name} is rejected now: take it off `NOT_YET`"));
            }
        } else if out.code == 0 {
            wrong.push(format!("{name} was accepted; expected: {reason}"));
        } else if !out.stderr.contains(reason) {
            wrong.push(format!("{name}: expected `{reason}` in:\n{}", out.stderr));
        }
    }
    assert!(cases >= 29, "only {cases} failing tests state a reason");
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
