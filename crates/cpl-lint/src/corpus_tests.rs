//! False-positive ratchet: code in `spec/std/` and the corpus's `spec.cp`
//! files is idiomatic by definition, so every warning there is a lint bug or
//! a finding someone has to act on. `corpus_warnings.txt` holds today's count
//! per set and code; a count may not rise. `CPL_LINT_BLESS=1` rewrites it
//! after a fall.

use super::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SETS: &[(&str, &str, &str)] = &[
    ("std", "spec/std", ""),
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
