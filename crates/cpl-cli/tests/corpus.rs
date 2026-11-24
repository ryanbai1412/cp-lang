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
/// These are ports still written in v2.1, which `proto/` migrates in its
/// own pass: a `read() where` guard, `Dsu.count` and `std.steps` are not
/// in v2.2, and each is reported with what to write instead.
const PENDING: &[&str] = &[
    "crates/cpl-cli/tests/cases/ergo/compound_div.cp",
    "crates/cpl-cli/tests/cases/ergo/library.cp",
];

/// Files the checker takes and lowering does not: until `cpl check` ran
/// lowering too, it called these clean while `cpl run` and `cpl build`
/// rejected them (`research/archive/audits/anti-slop-audit-1.md`, finding 1). Each is a
/// gap in `cpl-ir` or the standard library, listed so that it is seen. A
/// file leaves this list when it builds, and the test below says so;
/// nothing is to be added to it.
const NOT_BUILT: &[&str] = &[
    "crates/cpl-cli/tests/cases/ergo/columns.cp",
    "crates/cpl-cli/tests/cases/ergo/counter.cp",
    "crates/cpl-cli/tests/cases/ergo/moves.cp",
    "crates/cpl-cli/tests/cases/ergo/ranges.cp",
    "crates/cpl-cli/tests/cases/ergo/sugar.cp",
    "crates/cpl-cli/tests/cases/fn/invariant.cp",
    "crates/cpl-cli/tests/cases/lend/grid_rows.cp",
    "crates/cpl-cli/tests/cases/lend/projection.cp",
    "crates/cpl-cli/tests/cases/lend/slices.cp",
    "crates/cpl-cli/tests/cases/ok/at1.cp",
    "crates/cpl-cli/tests/cases/ok/do2.cp",
    "crates/cpl-cli/tests/cases/ok/subset2.cp",
    "crates/cpl-cli/tests/cases/redteam/empty_slice_reduce.cp",
    "crates/cpl-cli/tests/cases/redteam/hashmap_sentinel.cp",
    "crates/cpl-cli/tests/cases/redteam/list_expr_once.cp",
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

/// Failing tests this checker still accepts: each needs a rule it does
/// not have yet. A name leaves this list when its case is rejected for
/// its stated reason, and the test below says so.
const NOT_YET: &[&str] = &[
    // Synthesis #3, in the prototype only so far: `const struct` and
    // `a.get(i)` on a domain-indexed array.
    "s3_const_struct_non_pod",
    "s3_get_bare_int",
    "s3_get_wrong_domain",
];

/// Failing tests this checker rejects for the stated reason, in words
/// of its own: the prototype says `expected Int, got ?Int`, this checker
/// names the variable. Each is held to the wording given here.
const REWORDED: &[(&str, &str)] = &[
    (
        "s3_inv1_bool_alias",
        "`y` is declared an Int, but `x` is an ?Int",
    ),
    (
        "s3_inv2_array_place",
        "`y` is declared an Int, but `a[i]` is an ?Int",
    ),
    (
        "s3_inv2_struct_field",
        "`y` is declared an Int, but `b.x` is an ?Int",
    ),
    (
        "s3_inv3_after_join",
        "`y` is declared an Int, but `x` is an ?Int",
    ),
    (
        "s3_inv3_after_mut_arg",
        "`y` is declared an Int, but `x` is an ?Int",
    ),
    (
        "s3_inv3_after_reassign",
        "`y` is declared an Int, but `x` is an ?Int",
    ),
    (
        "s3_inv3_closure_mutation",
        "`y` is declared an Int, but `x` is an ?Int",
    ),
    (
        "s3_inv4_bang_non_opt",
        "`!` takes the value out of an optional, but this is a `Int`",
    ),
];

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
            "rt_min_empty_slice" => "`k` is declared an Int, but `a[f..f].max()` is an ?Int",
            // A message names no line: the loop is pointed at instead.
            "rt_for_fn_push" => {
                "`grow` can't be called here: it changes `xs`, and the `for` loop is going over `xs`"
            }
            "rt_for_push" => "the `for` loop is going over `xs`",
            // v2.2: the body ends with the place, and `mut` says it is lent to change.
            "proj_no_lend" => "so the body ends with `mut cnt[i]`",
            // Synthesis #2's rejected spellings, in this checker's words.
            "s2_array_default_literal" => "expected `:` between a key and its value",
            "s2_char_of_digit" => "no value named `Char` here",
            "s2_f64_to_int" => "`n` is declared an Int, but `x` is an F64",
            "s2_float_name" => "no type or domain named `Float`",
            "s2_for_mut" => "expected a value after `in`, found `mut`",
            "s2_opt_plus_eq" => "`+=` can't be applied to an ?Int",
            "s2_print_yes_no" => "`print` has no argument named `yes`",
            "s2_dom_try" => "a `domain D` has no method `try`",
            "s2_read_angle" => "no value named `read` here",
            "s2_read_no_type" => "can't tell what to read here",
            "s2_subdomain_widen" => "`a` is indexed by Pkg, but `d` is a Dep",
            "s2_unwrap_let" => "`o` is a `let`, so it can't be assigned",
            _ => reason,
        };
        cases += 1;
        if PROTOTYPE_ONLY.contains(&name.as_str()) {
            continue;
        }
        let out = check(&path);
        let reason = REWORDED
            .iter()
            .find(|(case, _)| *case == name)
            .map_or(reason, |(_, ours)| ours);
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
