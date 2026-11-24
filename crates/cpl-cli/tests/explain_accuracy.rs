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
use cpp_counts::{deep_copies, std_functions};

/// The `.cp` files one directory down from the corpus problems and the
/// conformance cases.
fn sources() -> Vec<PathBuf> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    for root in ["corpus/problems", "spec/conformance"] {
        for dir in std::fs::read_dir(repo.join(root)).expect("source root exists") {
            let dir = dir.unwrap().path();
            if !dir.is_dir() {
                continue;
            }
            for file in std::fs::read_dir(dir).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "cp") {
                    found.push(path);
                }
            }
        }
    }
    found.sort();
    found
}

/// The `"lowering"` and `"detail"` of each row of `cpl explain --json`.
fn rows(json: &str) -> Vec<(String, String)> {
    let field = |line: &str, key: &str| -> Option<String> {
        let start = line.find(&format!("\"{key}\":\""))? + key.len() + 4;
        let rest = &line[start..];
        let mut end = 0;
        let bytes = rest.as_bytes();
        while end < bytes.len() && !(bytes[end] == b'"' && (end == 0 || bytes[end - 1] != b'\\')) {
            end += 1;
        }
        Some(rest[..end].to_string())
    };
    json.lines()
        .filter_map(|l| Some((field(l, "lowering")?, field(l, "detail")?)))
        .collect()
}

/// Whether `cpp` names `ident` as a whole word.
fn names(cpp: &str, ident: &str) -> bool {
    cpp.match_indices(ident).any(|(i, _)| {
        !cpp[i + ident.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
    })
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
    // One source copy may be written in more than one specialization.
    let mut seen = std::collections::BTreeSet::new();
    let distinct: Vec<&str> = cpp.lines().filter(|l| seen.insert(l.trim())).collect();
    let copies = deep_copies(&distinct.join("\n"));
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
