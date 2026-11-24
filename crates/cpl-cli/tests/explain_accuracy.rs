//! `cpl explain` checked against the C++ it describes, over the corpus
//! and the conformance cases. Explain says each closure it calls
//! `static` is called without a `std::function`, names `cpl_fnN` for the
//! direct ones, and lists every copy ("every other heap read is a
//! reference"). Each claim is checked against the C++ itself, using the
//! counters the copy ratchet uses, so explain can't call a guess proven.
//! Enforces performance/R17 in handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

#[path = "support/cpp_counts.rs"]
mod cpp_counts;
use cpp_counts::{deep_copies, optional_copies, std_functions};

/// The `.cp` files one directory down from the corpus problems and the
/// conformance cases.
fn sources() -> Vec<PathBuf> {
    cpl_corpus::programs::corpus_and_conformance(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
    )
}

/// The `"lowering"` and `"detail"` of each row of `cpl explain --json`.
fn rows(json: &str) -> Vec<(String, String)> {
    let json = cpl_json::parse(json).expect("cpl explain --json is JSON");
    let field = |row: &cpl_json::Value, key: &str| row.get(key)?.as_str().map(str::to_string);
    json.as_array()
        .unwrap_or_default()
        .iter()
        .filter_map(|row| Some((field(row, "lowering")?, field(row, "detail")?)))
        .collect()
}

/// Whether `cpp` names `ident` as a whole word.
fn names(cpp: &str, ident: &str) -> bool {
    cpp.match_indices(ident).any(|(i, _)| {
        !cpp[i + ident.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
    })
}

/// The deep copies `cpp` spells out, each source copy once though more
/// than one specialization may write it.
fn cpp_copies(cpp: &str) -> usize {
    let mut seen = std::collections::BTreeSet::new();
    let distinct: Vec<&str> = cpp.lines().filter(|l| seen.insert(l.trim())).collect();
    let distinct = distinct.join("\n");
    deep_copies(&distinct) + optional_copies(&distinct)
}

/// Each way explain's report on `src` disagrees with the C++ it builds.
fn disagreements(src: &Path, out: &Path) -> Option<Vec<String>> {
    let built = cpl_cli::run(&["build", src.to_str()?, "-o", out.to_str()?]);
    let cpp = std::fs::read_to_string(out).ok();
    let _ = std::fs::remove_file(out);
    let cpp = cpp.filter(|_| built.code == 0)?;
    let explained = cpl_cli::run(&["explain", src.to_str()?, "--json"]);
    if explained.code != 0 {
        return Some(vec![format!(
            "builds, but `cpl explain` fails: {}",
            explained.stderr
        )]);
    }
    let rows = rows(&String::from_utf8_lossy(&explained.stdout));
    let mut wrong = Vec::new();
    let dynamic = rows
        .iter()
        .filter(|(k, _)| k == "dynamic_std_function")
        .count();
    let functions = std_functions(&cpp);
    if dynamic == 0 && functions > 0 {
        wrong.push(format!(
            "explain lists no std::function, the C++ declares {functions}"
        ));
    }
    if dynamic > 0 && functions == 0 {
        wrong.push(format!(
            "explain lists {dynamic} std::functions, the C++ declares none"
        ));
    }
    for (_, closure) in rows.iter().filter(|(k, _)| k == "static_direct") {
        if !names(&cpp, closure) {
            wrong.push(format!(
                "explain says a call goes to {closure}, the C++ has none"
            ));
        }
    }
    let listed = rows.iter().filter(|(k, _)| k == "copy").count();
    let copies = cpp_copies(&cpp);
    if copies > listed {
        wrong.push(format!(
            "explain lists {listed} copies, the C++ spells out {copies}"
        ));
    }
    Some(wrong)
}

/// The programs split into this many tests, so nextest spreads them over
/// CI's test partitions instead of one job running them all.
const SHARDS: usize = 8;

fn explain_claims_on_shard(shard: usize) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("explain_accuracy");
    std::fs::create_dir_all(&out).unwrap();
    let sources: Vec<PathBuf> = sources().into_iter().skip(shard).step_by(SHARDS).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let checked = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let wrong: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = (0..threads)
            .map(|n| {
                let out = out.join(format!("{shard}-{n}.cpp"));
                let (sources, next, checked) = (&sources, &next, &checked);
                s.spawn(move || {
                    let mut wrong = Vec::new();
                    while let Some(src) =
                        sources.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let Some(found) = disagreements(src, &out) else {
                            continue;
                        };
                        checked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        wrong.extend(found.into_iter().map(|w| format!("{}: {w}", src.display())));
                    }
                    wrong
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let checked = checked.into_inner();
    assert!(checked >= 1000 / SHARDS, "only {checked} programs built");
    assert!(
        wrong.is_empty(),
        "{} disagreements:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn explain_claims_only_what_the_cpp_does_0() {
    explain_claims_on_shard(0);
}

#[test]
fn explain_claims_only_what_the_cpp_does_1() {
    explain_claims_on_shard(1);
}

#[test]
fn explain_claims_only_what_the_cpp_does_2() {
    explain_claims_on_shard(2);
}

#[test]
fn explain_claims_only_what_the_cpp_does_3() {
    explain_claims_on_shard(3);
}

#[test]
fn explain_claims_only_what_the_cpp_does_4() {
    explain_claims_on_shard(4);
}

#[test]
fn explain_claims_only_what_the_cpp_does_5() {
    explain_claims_on_shard(5);
}

#[test]
fn explain_claims_only_what_the_cpp_does_6() {
    explain_claims_on_shard(6);
}

#[test]
fn explain_claims_only_what_the_cpp_does_7() {
    explain_claims_on_shard(7);
}

/// The copies `cpl explain --json` lists for `src`, and the ones the C++
/// `cpl build --full` writes for it makes.
fn listed_and_made(src: &Path, out: &Path) -> (usize, usize) {
    let built = cpl_cli::run(&[
        "build",
        src.to_str().unwrap(),
        "--full",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(built.code, 0, "{}: {}", src.display(), built.stderr);
    let cpp = std::fs::read_to_string(out).unwrap();
    let explained = cpl_cli::run(&["explain", src.to_str().unwrap(), "--json"]);
    assert_eq!(explained.code, 0, "{}: {}", src.display(), explained.stderr);
    let rows = rows(&String::from_utf8_lossy(&explained.stdout));
    let listed = rows.iter().filter(|(k, _)| k == "copy").count();
    (listed, cpp_copies(&cpp))
}

#[test]
fn explain_lists_the_copies_the_judge_cpp_makes() {
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("explain_counts");
    std::fs::create_dir_all(&out).unwrap();
    for case in CASES {
        let src = conformance.join(case);
        let (listed, made) = listed_and_made(&src, &out.join("a.cpp"));
        assert_eq!(
            listed, made,
            "{case}: explain lists {listed} copies, the C++ makes {made}"
        );
    }
}

/// Cases whose copies explain once miscounted, against the C++ for the
/// judge.
const CASES: &[&str] = &[
    "std-lazy/rope-matches-brute.cp",
    "std-lazy/rope-lazy-treap-to-array-matches-brute.cp",
    "std-hashmap/reserve-keeps-entries.cp",
    "std-poly/advanced-free-fns-match-brute.cp",
    "core-01/while-let-tuple-pattern.cp",
];
