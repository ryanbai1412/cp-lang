//! False-positive ratchet: code in `std/` and the corpus's `spec.cp`
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

/// `cpl lint` and the editor refine the syntax findings with the same
/// checker facts, so they report the same lints on every std file.
#[test]
fn the_cli_and_the_editor_read_the_same_facts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = Config::default();
    let key = |d: &Diagnostic| (d.code, d.span.start, d.span.end);
    for path in files(&root, crate::STD_DIR, "") {
        let src = std::fs::read_to_string(&path).unwrap();
        let mut db = cpl_check::Database::new();
        db.set_file("<lint>.cp", src.as_str());
        let Some(analysis) = db.analyze("<lint>.cp") else {
            continue;
        };
        if analysis.has_syntax_errors {
            continue;
        }
        let items: Vec<_> = analysis
            .items
            .iter()
            .map(|i| (i.offset, &i.syntax.parsed))
            .collect();
        let mut editor = lint_parsed(&analysis.text, &items, &config);
        refine_with_analysis(&mut editor, &analysis);
        let cli: Vec<_> = lint(&src, &config).iter().map(key).collect();
        let editor: Vec<_> = editor.iter().map(key).collect();
        assert_eq!(editor, cli, "{}", path.display());
    }
}

/// Proving one item gives what proving the whole file gives inside it, on
/// every corpus port and spec and every `std` module.
#[test]
fn one_item_proves_what_the_whole_file_proves_there() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = files(&root, crate::STD_DIR, "");
    paths.extend(files(&root, "corpus/problems", "spec.cp"));
    paths.extend(files(&root, "corpus/problems", "port.cp"));
    let mut compared = 0;
    for path in paths {
        let src = std::fs::read_to_string(&path).unwrap();
        let parsed = cpl_syntax::parse_file(&src);
        if !parsed.diagnostics.is_empty() {
            continue;
        }
        let items: Vec<(u32, &ParsedItem)> = parsed.items.iter().map(|(o, p)| (*o, p)).collect();
        let (loops, indexes) = proven_loops_and_indexes_parsed(&src, &items);
        for (i, (start, _)) in items.iter().enumerate() {
            let end = items.get(i + 1).map_or(u32::MAX, |(b, _)| *b);
            let inside = |at: u32| *start <= at && at < end;
            let (one_loops, one_indexes) = proven_loops_and_indexes_in_item(&src, &items, i);
            let want_loops: Vec<_> = loops.iter().filter(|l| inside(l.head.0)).cloned().collect();
            let want_indexes: Vec<_> = indexes
                .iter()
                .filter(|x| inside(x.access.0))
                .cloned()
                .collect();
            assert_eq!(one_loops, want_loops, "{} item {i}", path.display());
            assert_eq!(one_indexes, want_indexes, "{} item {i}", path.display());
            compared += 1;
        }
    }
    assert!(compared > 1000, "compared {compared} items");
}

/// Two lints that fire on one span either share a root cause (the inner one
/// is in `SUPERSEDED_BY`, so only the wider is shown) or teach two things
/// (listed here). A new pair has to be one or the other.
/// The rewrite of the outer one keeps the inner code, so both still teach.
const DISTINCT: &[(&str, &str)] = &[
    ("L0415", "L0506"),
    ("L0501", "L0549"),
    ("L0504", "L0203"),
    ("L0504", "L0415"),
    ("L0504", "L0506"),
    ("L0504", "L0540"),
    ("L0507", "L0510"),
    ("L0507", "L0540"),
    ("L0531", "L0506"),
    ("L0536", "L0506"),
    ("L0549", "L0203"),
    ("L0551", "L0506"),
    ("L0580", "L0523"),
    ("L0585", "L0504"),
    ("L0585", "L0523"),
    ("L0585", "L0540"),
    ("L0586", "L0101"),
    ("L0504", "L0588"),
];

#[test]
fn nested_findings_are_one_cause_or_listed_distinct() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = std::collections::BTreeSet::new();
    let sets = [
        ("corpus/newcomer", ""),
        (crate::STD_DIR, ""),
        ("corpus/problems", "port.cp"),
    ];
    for (dir, leaf) in sets {
        for file in files(&root, dir, leaf) {
            let src = std::fs::read_to_string(&file).unwrap();
            let (config, _) = Config::discover(&file);
            let diags = lint(&src, &config);
            for outer in &diags {
                for inner in &diags {
                    if outer.code != inner.code
                        && outer.span != inner.span
                        && outer.span.contains(inner.span)
                    {
                        found.insert((outer.code.0, inner.code.0));
                    }
                }
            }
        }
    }
    let new: Vec<_> = found
        .into_iter()
        .filter(|(o, i)| !DISTINCT.contains(&(*o, *i)))
        .collect();
    assert!(
        new.is_empty(),
        "lints nested in another's span; supersede the inner one or list the pair in DISTINCT: {new:?}"
    );
}
