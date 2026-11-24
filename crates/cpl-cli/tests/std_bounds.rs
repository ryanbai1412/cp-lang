//! Every public std fn whose first parameter is a bounded generic rejects,
//! with E0606 naming the fn, an argument whose type doesn't meet the bound:
//! a fn value, which has no order, equality, hash or text, and an optional of a type
//! that does, which must not be unwrapped to fit. A fn taking `self` is
//! probed in method form too. Probes run through `cpl check`, which lowers
//! too: method form is checked there, until method lookup by receiver type
//! lets the checker pick the candidate.

use std::path::{Path, PathBuf};

use cpl_syntax::ast::{StmtKind, TypeKind};

/// Calls that type-check today though their argument misses the bound.
/// Only shrinks.
const KNOWN_ACCEPTED: &[&str] = &[];

/// Methods `?T` has itself, so `opt.name(..)` calls the optional's own.
const OPTIONAL_METHODS: &[&str] = &["map", "filter"];

fn probes(bound: &str) -> &'static [(&'static str, &'static str, &'static str)] {
    match bound {
        "Seq" | "Indexed" => &[
            (
                "a fn value",
                "let probe: Int -> Int = x => x",
                "(Int) -> Int",
            ),
            (
                "an optional list",
                "let probe: ?List<Int> = Some([1])",
                "?List<Int>",
            ),
        ],
        "Ord" | "Add" | "Sub" | "Keyed" | "Bidir" | "Undo" | "Read" => &[(
            "a fn value",
            "let probe: Int -> Int = x => x",
            "(Int) -> Int",
        )],
        _ => &[],
    }
}

#[test]
fn std_fns_reject_arguments_that_miss_their_bound() {
    let std_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&std_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    files.sort();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("std_bounds");
    std::fs::create_dir_all(&dir).unwrap();
    let main = dir.join("main.cp");
    let (mut calls, mut accepted) = (0, Vec::new());
    for file in &files {
        let module = file.file_stem().unwrap().to_string_lossy().into_owned();
        if module == "prelude" {
            continue;
        }
        let parsed = cpl_syntax::parse_file(&std::fs::read_to_string(file).unwrap());
        for (_, item) in &parsed.items {
            for stmt in &item.stmts {
                let StmtKind::Fn(f) = &stmt.kind else {
                    continue;
                };
                let Some(first) = f.params.first() else {
                    continue;
                };
                if !stmt.public || f.params.iter().any(|p| p.variadic) {
                    continue;
                }
                let Some(TypeKind::Named { name, args }) = first.ty.as_ref().map(|t| &t.kind)
                else {
                    continue;
                };
                if !args.is_empty() {
                    continue;
                }
                let traits = f.bounds.iter().filter(|b| b.name.text == name.text);
                let traits: Vec<String> = traits
                    .flat_map(|b| &b.traits)
                    .filter_map(|t| match &t.kind {
                        TypeKind::Named { name, .. } => Some(name.text.clone()),
                        _ => None,
                    })
                    .collect();
                let rest: Vec<String> = f.params[1..]
                    .iter()
                    .filter(|p| p.default.is_none())
                    .map(|p| match p.named_only {
                        true => format!("{}: todo", p.label_text()),
                        false => "todo".to_string(),
                    })
                    .collect();
                let fname = &f.name.text;
                for bound in &traits {
                    for (what, probe, ty) in probes(bound) {
                        let free = std::iter::once("probe".to_string())
                            .chain(rest.iter().cloned())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let getter = stmt.decorators.iter().any(|d| d.name.text == "get");
                        let method = match getter {
                            true => format!("probe.{fname}"),
                            false => format!("probe.{fname}({})", rest.join(", ")),
                        };
                        let own = ty.starts_with('?') && OPTIONAL_METHODS.contains(&fname.as_str());
                        let method = (first.name.text == "self" && !own).then_some(method);
                        for call in std::iter::once(format!("{fname}({free})")).chain(method) {
                            let src =
                                format!("use std.{module}.{{{fname}}}\n{probe}\nlet _ = {call}\n");
                            calls += 1;
                            if !misses_bound(&main, &src, fname, ty) {
                                accepted.push(format!(
                                    "std.{module}.{fname} ({bound}) takes {what}:\n{src}"
                                ));
                            }
                        }
                    }
                    if matches!(bound.as_str(), "Seq" | "Indexed") {
                        let call = std::iter::once("[1]".to_string()).chain(rest.iter().cloned());
                        let call = call.collect::<Vec<_>>().join(", ");
                        let src =
                            format!("use std.{module}.{{{fname}}}\nlet _ = {fname}({call})\n");
                        assert!(
                            !misses_bound(&main, &src, fname, "List<Int>"),
                            "a list meets `{bound}`, yet the check reports it missing:\n{src}"
                        );
                    }
                }
            }
        }
    }
    let key = |s: &String| s.split(' ').next().unwrap().to_string();
    let new: Vec<&String> = accepted
        .iter()
        .filter(|a| !KNOWN_ACCEPTED.contains(&key(a).as_str()))
        .collect();
    let fixed: Vec<&&str> = KNOWN_ACCEPTED
        .iter()
        .filter(|k| !accepted.iter().any(|a| key(a) == **k))
        .collect();
    eprintln!("{calls} probe calls, {} accepted", accepted.len());
    assert!(calls > 100, "only {calls} probe calls; the std scan broke");
    assert!(
        new.is_empty() && fixed.is_empty(),
        "{} of {calls} probe calls type-check though the argument misses the bound:\n{}\nnow rejected, delete from KNOWN_ACCEPTED: {fixed:?}",
        new.len(),
        new.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"),
    );
}

/// Whether `cpl check` of `src` reports that `ty` misses a bound of the std fn `fname`.
fn misses_bound(main: &Path, src: &str, fname: &str, ty: &str) -> bool {
    std::fs::write(main, src).unwrap();
    let out = cpl_cli::run(&["check", main.to_str().unwrap()]);
    out.stderr.lines().any(|l| {
        l.starts_with("error[E0606]: `")
            && l.contains(&format!("{ty}` does not implement"))
            && l.ends_with(&format!("required by `{fname}`"))
    })
}
