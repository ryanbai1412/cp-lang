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
use cpp_counts::{deep_copies, is_place, optional_copies, std_functions};

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

/// The containers [`constructed_copies`] sees built from another value.
const CONSTRUCTED: [&str; 8] = [
    "std::vector<",
    "std::array<",
    "std::map<",
    "std::set<",
    "std::unordered_map<",
    "std::deque<",
    "cpl::Grid<",
    "std::string",
];

/// `xs`, `w[i]`, `self.rows`: a name, then indexes, then fields.
fn is_indexed_place(text: &str) -> bool {
    let word = |t: &str| {
        t.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && t.chars().all(|c| c.is_alphanumeric() || c == '_')
    };
    let (head, fields) = text.split_once('.').unwrap_or((text, ""));
    let (base, index) = head.split_once('[').unwrap_or((head, ""));
    let index_ok = index.is_empty()
        || index.ends_with(']')
            && index[..index.len() - 1]
                .split("][")
                .all(|i| !i.is_empty() && !i.contains(['[', ']']));
    word(base) && index_ok && (fields.is_empty() || fields.split('.').all(word))
}

/// The argument of the call whose `(` opens `rest`, when it is a place.
fn place_arg(rest: &str) -> bool {
    let Some(arg) = rest.strip_prefix('(') else {
        return false;
    };
    if let Some(at) = arg.strip_prefix("cpl::at(") {
        return at
            .split_once(')')
            .is_some_and(|(inner, after)| !inner.contains('(') && after.starts_with(')'));
    }
    arg.split_once(')')
        .is_some_and(|(a, _)| is_indexed_place(a))
}

/// A second heap value spelled as a call: a container constructed from a
/// place (`std::array<ll, 4>(a)`, `std::set<ll>(a)`), `cpl::sorted(xs)`
/// (which takes its list by value) and `cpl::to_list(..)`. Mirrors `COPY`
/// in scripts/specgate/explain_copies.py. Counted here on top of
/// [`deep_copies`], not under the `// copies:` headers.
fn constructed_copies(cpp: &str) -> usize {
    cpp.lines()
        .map(|line| {
            let sorted = line
                .match_indices("cpl::sorted(")
                .filter(|(at, _)| {
                    line[at + "cpl::sorted(".len()..]
                        .split_once(')')
                        .is_some_and(|(a, _)| is_place(a) && !a.starts_with("cpl::"))
                })
                .count();
            let built = CONSTRUCTED
                .iter()
                .flat_map(|ty| line.match_indices(ty).map(move |(at, _)| at + ty.len()))
                .filter(|&start| {
                    let rest = &line[start..];
                    if !line[..start].ends_with('<') {
                        return place_arg(rest);
                    }
                    let mut depth = 1;
                    rest.char_indices()
                        .find_map(|(i, c)| {
                            depth += match c {
                                '<' => 1,
                                '>' => -1,
                                ';' | '(' | ')' => return Some(None),
                                _ => 0,
                            };
                            (depth == 0).then_some(Some(i))
                        })
                        .flatten()
                        .is_some_and(|end| place_arg(&rest[end + 1..]))
                })
                .count();
            line.matches("cpl::to_list(").count() + sorted + built
        })
        .sum()
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
    (listed, cpp_copies(&cpp) + constructed_copies(&cpp))
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
    "core-05/exercise-34-loop-writes-separate-clone.cp",
    "std-models/dp-small.cp",
    "std-models/link-cut.cp",
    "std-models/historic-beats.cp",
    "std-models/euler-tour-tree.cp",
    "std-seq/set-union-intersect-subtract.cp",
];

#[test]
fn a_container_built_from_a_place_or_sorted_is_a_copy() {
    let copies = [
        "a2 = std::array<ll, 4>(a);",
        "u = std::set<ll>(a);",
        "std::vector<ll> want_2 = std::vector<ll>(sums);",
        "g = std::vector<std::vector<ll>>(rows[i]);",
        "auto s = std::string(self.name);",
        "m = std::map<ll, std::vector<ll>>(cpl::at(ms, i));",
        "a = cpl::sorted(a);",
        "show_1(cpl::sorted(self.xs), 10);",
        "auto v = cpl::to_list(it);",
    ];
    for line in copies {
        assert_eq!(constructed_copies(line), 1, "{line}");
    }
    let not_copies = [
        "std::vector<ll> xs = std::vector<ll>();",
        "auto v = std::vector<ll>(s.begin(), s.end());",
        "auto v = std::vector<ll>(5);",
        "auto t = cpl::sorted(f(a));",
        "std::vector<std::string> names;",
        "auto v = std::vector<ll>{k};",
    ];
    for line in not_copies {
        assert_eq!(constructed_copies(line), 0, "{line}");
    }
}
