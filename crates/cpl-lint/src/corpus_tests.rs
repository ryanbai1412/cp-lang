//! False-positive ratchet: code in `spec/std/` and the corpus's `spec.cp`
//! files is idiomatic by definition, so every warning there is a lint bug or
//! a finding someone has to act on. `corpus_warnings.txt` holds today's count
//! per set and code; a count may not rise. `CPL_LINT_BLESS=1` rewrites it
//! after a fall.

use super::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SETS: &[(&str, &str, &str)] = &[
    ("std", crate::STD_DIR, ""),
    ("spec", "corpus/problems", "spec.cp"),
];

fn files(root: &Path, dir: &str, leaf: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root.join(dir)).unwrap() {
        let path = entry.unwrap().path();
        if leaf.is_empty() {
            if path.extension().is_some_and(|e| e == "cp") {
                out.push(path);
            }
        } else if path.join(leaf).is_file() {
            out.push(path.join(leaf));
        }
    }
    out.sort();
    out
}

fn counts(root: &Path) -> BTreeMap<(String, String), usize> {
    let mut out = BTreeMap::new();
    for (set, dir, leaf) in SETS {
        for file in files(root, dir, leaf) {
            let src = std::fs::read_to_string(&file).unwrap();
            let (config, _) = Config::discover(&file);
            for diag in lint(&src, &config) {
                *out.entry((set.to_string(), diag.code.0.to_string()))
                    .or_default() += 1;
            }
        }
    }
    out
}

#[test]
fn corpus_warnings_only_go_down() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.join("../..");
    let baseline_path = crate_dir.join("corpus_warnings.txt");
    let now = counts(&root);
    if std::env::var_os("CPL_LINT_BLESS").is_some() {
        let mut text = String::from("# set code count; see src/corpus_tests.rs\n");
        for ((set, code), n) in &now {
            text.push_str(&format!("{set} {code} {n}\n"));
        }
        std::fs::write(&baseline_path, text).unwrap();
        return;
    }
    let mut baseline = BTreeMap::new();
    for line in std::fs::read_to_string(&baseline_path).unwrap().lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let n: usize = parts[2].parse().unwrap();
        baseline.insert((parts[0].to_string(), parts[1].to_string()), n);
    }
    let mut keys: Vec<_> = baseline.keys().chain(now.keys()).cloned().collect();
    keys.sort();
    keys.dedup();
    let mut rose = Vec::new();
    for key in keys {
        let was = baseline.get(&key).copied().unwrap_or(0);
        let is = now.get(&key).copied().unwrap_or(0);
        if is > was {
            rose.push(format!("{} {}: {was} -> {is}", key.0, key.1));
        } else if is < was {
            eprintln!(
                "{} {}: {was} -> {is}; lower it with CPL_LINT_BLESS=1",
                key.0, key.1
            );
        }
    }
    assert!(
        rose.is_empty(),
        "corpus lint warnings rose (a new false positive, or corpus code that is \
         not idiomatic):\n{}",
        rose.join("\n")
    );
}

fn checker_errors(path: &Path, src: &str) -> usize {
    let key = path.to_string_lossy().into_owned();
    let mut db = cpl_check::Database::new();
    db.set_file(&key, src);
    db.analyze(&key).map_or(0, |analysis| {
        analysis
            .diagnostics
            .iter()
            .filter(|d| d.severity == cpl_diag::Severity::Error)
            .count()
    })
}

/// Dream ports a fix still makes worse, each waiting on another domain. The
/// list only shrinks: an entry that stops getting worse fails the test.
const KNOWN_WORSE: &[(&str, &str)] = &[(
    "corpus/problems/cf2262d/dream.cp",
    "the checker types `find` as an index search, not `find(p)` (compiler worklog)",
)];

/// A verified autofix never makes a file worse: `cpl fix` leaves the checker
/// with no more errors than before, on idiomatic code and on dream ports.
#[test]
fn fixes_never_add_checker_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dream = ("dream", "corpus/problems", "dream.cp");
    let all: Vec<PathBuf> = SETS
        .iter()
        .chain([&dream])
        .flat_map(|(_, dir, leaf)| files(&root, dir, leaf))
        .collect();
    let made_worse = |file: &PathBuf| {
        let src = std::fs::read_to_string(file).unwrap();
        let (config, _) = Config::discover(file);
        let (fixed, applied) = fix_at(Some(file), &src, &config);
        if applied == 0 {
            return None;
        }
        let (before, after) = (checker_errors(file, &src), checker_errors(file, &fixed));
        let rel = file.strip_prefix(&root).unwrap().to_string_lossy();
        (after > before).then(|| (rel.into_owned(), before, after))
    };
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = all.len().div_ceil(threads).max(1);
    let found: Vec<(String, usize, usize)> = std::thread::scope(|scope| {
        let jobs: Vec<_> = all
            .chunks(chunk)
            .map(|files| {
                scope.spawn(move || files.iter().filter_map(made_worse).collect::<Vec<_>>())
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let (known, worse): (Vec<_>, Vec<_>) = found
        .into_iter()
        .partition(|(rel, _, _)| KNOWN_WORSE.iter().any(|(k, _)| k == rel));
    let worse: Vec<String> = worse
        .iter()
        .map(|(rel, before, after)| format!("{rel}: {before} -> {after}"))
        .collect();
    assert!(
        worse.is_empty(),
        "fixes add checker errors:\n{}",
        worse.join("\n")
    );
    let fixed: Vec<_> = KNOWN_WORSE
        .iter()
        .filter(|(k, _)| !known.iter().any(|(w, _, _)| w == k))
        .collect();
    assert!(
        fixed.is_empty(),
        "no longer made worse, drop from KNOWN_WORSE: {fixed:?}"
    );
}
