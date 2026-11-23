//! A ratchet on the C++ the corpus builds to: a heap value that is only
//! read is read in place, never copied. Each pattern below is a copy the
//! generated code used to make; its count may only go down.

use std::path::{Path, PathBuf};

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

/// The most copies of each kind the corpus may build to, with why the
/// ones left are left.
const BOUNDS: [(&str, fn(&str) -> usize, usize); 2] = [
    // `var cols = m.get(r) ?? []` then pushed to (a copy it needs);
    // `m[k] = (m.get(k) ?? []) + [v]`; and `?? []` passed to a fn.
    ("map_get_copies", map_get_copies, 5),
    ("block_copies", block_copies, 0),
];

fn sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/problems");
    let mut found = Vec::new();
    for dir in std::fs::read_dir(root).expect("corpus/problems exists") {
        for file in std::fs::read_dir(dir.unwrap().path()).unwrap() {
            let path = file.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "cp") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn the_corpus_never_copies_a_heap_value_it_only_reads() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("read_only_copies");
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let chunk = sources.len().div_ceil(8);
    let built = std::sync::atomic::AtomicUsize::new(0);
    let found: Vec<(String, [usize; 2])> = std::thread::scope(|s| {
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
    assert!(built >= 1000, "only {built} corpus programs built");
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
