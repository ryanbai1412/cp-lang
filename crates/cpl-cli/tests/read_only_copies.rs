//! A ratchet on the C++ the corpus and the conformance cases build to: a
//! heap value that is only read is read in place, never copied. Each
//! pattern below is a copy the generated code used to make, or (for
//! `deep_copies`) any deep copy of a container; its count may only go down.
//! Enforces performance/R12, R13 and R14 in
//! handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

#[path = "support/cpp_counts.rs"]
mod cpp_counts;
use cpp_counts::{deep_copies, std_functions};

/// `std::optional<std::vector<..>> t = cpl::map_get(m, k)`: the entry
/// copied out of the map to be read.
fn map_get_copies(cpp: &str) -> usize {
    cpp.lines()
        .filter(|l| l.contains("std::optional<std::vector<") && l.contains("= cpl::map_get("))
        .count()
}

/// `[&]() -> std::vector<..> { D_len = ..; return xs; }()`: a block that
/// sizes a domain from a list, then gives the list by value.
fn block_copies(cpp: &str) -> usize {
    let lines: Vec<&str> = cpp.lines().collect();
    (0..lines.len())
        .filter(|&i| lines[i].contains("[&]() -> std::vector<") && lines[i].ends_with('{'))
        .filter(|&i| {
            let mut j = i + 1;
            while lines.get(j).is_some_and(|l| {
                let l = l.trim();
                l.split(" = ")
                    .next()
                    .is_some_and(|lhs| lhs.ends_with("_len"))
            }) {
                j += 1;
            }
            j > i + 1
                && lines.get(j).is_some_and(|l| {
                    let l = l.trim();
                    l.strip_prefix("return ")
                        .and_then(|r| r.strip_suffix(';'))
                        .is_some_and(|v| v.chars().all(|c| c.is_alphanumeric() || c == '_'))
                })
        })
        .count()
}

/// A copy kind: its name, its counter over a C++ file, and its bound.
type Bound = (&'static str, fn(&str) -> usize, usize);

/// `std::vector<ll>(cpl::map_or(m, k, d))`: a copy of an entry, made to
/// be changed.
fn entry_copies(cpp: &str) -> usize {
    cpp.lines().filter(|l| l.contains(">(cpl::map_or(")).count()
}

/// The most copies of each kind the corpus may build to, with why the
/// ones left are left.
const BOUNDS: [Bound; 5] = [
    // `var cols = m.get(r) ?? []` then pushed to: a copy it needs.
    ("map_get_copies", map_get_copies, 0),
    ("block_copies", block_copies, 0),
    ("entry_copies", entry_copies, 0),
    // Measured when the ratchet began: snapshots taken before a change
    // (`old_s = self.state`) and temporaries copied out of places.
    ("deep_copies", deep_copies, 30),
    ("std_functions", std_functions, 154),
];

/// The `.cp` files one directory down from each root: the corpus
/// problems and the conformance cases.
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

#[test]
fn the_corpus_and_conformance_never_copy_a_heap_value_they_only_read() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("read_only_copies");
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let chunk = sources.len().div_ceil(8);
    let built = std::sync::atomic::AtomicUsize::new(0);
    let found: Vec<(String, [usize; BOUNDS.len()])> = std::thread::scope(|s| {
        let jobs: Vec<_> = sources
            .chunks(chunk)
            .enumerate()
            .map(|(n, part)| {
                let (out, built) = (out.join(format!("{n}.cpp")), &built);
                s.spawn(move || {
                    let mut found = Vec::new();
                    for src in part {
                        let built_now = cpl_cli::run(&[
                            "build",
                            src.to_str().unwrap(),
                            "-o",
                            out.to_str().unwrap(),
                        ]);
                        let Ok(cpp) = std::fs::read_to_string(&out) else {
                            continue;
                        };
                        let _ = std::fs::remove_file(&out);
                        if built_now.code != 0 {
                            continue;
                        }
                        built.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let counts = BOUNDS.map(|(_, count, _)| count(&cpp));
                        if counts.iter().any(|&c| c > 0) {
                            found.push((src.display().to_string(), counts));
                        }
                    }
                    found
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let built = built.into_inner();
    assert!(
        built >= 1000,
        "only {built} corpus and conformance programs built"
    );
    for (k, (name, _, bound)) in BOUNDS.iter().enumerate() {
        let total: usize = found.iter().map(|(_, c)| c[k]).sum();
        let offenders: Vec<String> = found
            .iter()
            .filter(|(_, c)| c[k] > 0)
            .map(|(src, c)| format!("{src}: {}", c[k]))
            .collect();
        assert!(
            total <= *bound,
            "{name}: {total} copies, more than the {bound} allowed:\n{}",
            offenders.join("\n")
        );
        if total < *bound {
            eprintln!("{name}: {total} copies, under the bound of {bound}: lower it");
        }
    }
}
