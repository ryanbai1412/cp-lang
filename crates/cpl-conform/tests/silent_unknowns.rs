//! Silent unknowns: expressions in std, the corpus and the conformance
//! suite that the checker typed `Unknown` without reporting anything there.
//! Each is a fact every tool downstream guesses at, so the list only shrinks:
//! `silent_unknowns.txt` holds each file's count and each cause's
//! (`cause:index`, the kind of expression whose type was lost); a count that
//! grows fails, and one that falls must be tightened (`BLESS=1` rewrites the
//! list). A cause with no row has a ceiling of 0.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const LIST: &str = "crates/cpl-conform/silent_unknowns.txt";

fn sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(root.join("std"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    for package in cpl_corpus::packages(root) {
        for entry in std::fs::read_dir(&package).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "cp") {
                out.push(path);
            }
        }
    }
    let mut dirs = vec![root.join("spec/conformance")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "cp") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn found(root: &Path, files: &[PathBuf]) -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    let mut db = cpl_check::Session::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        db.set_file(&rel, text.as_str());
        let Some(analysis) = db.analysis(&rel) else {
            continue;
        };
        for (span, cause) in analysis.silent_unknown_causes() {
            let at = analysis.lines.line_col(span.start);
            let code = text[span.start as usize..span.end as usize]
                .lines()
                .next()
                .unwrap_or("");
            let line = format!("{rel}:{}:{}: {code}", at.line + 1, at.col + 1);
            out.push((line, cause));
        }
    }
    out
}

#[test]
fn silent_unknowns_only_go_down() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let files = sources(&root);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = files.len().div_ceil(threads);
    let found: Vec<(String, &str)> = std::thread::scope(|s| {
        let jobs: Vec<_> = (files.chunks(chunk))
            .map(|part| s.spawn(|| found(&root, part)))
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let mut now: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let causes: Vec<String> = found.iter().map(|(_, c)| format!("cause:{c}")).collect();
    for ((line, _), cause) in found.iter().zip(&causes) {
        now.entry(line.split(':').next().unwrap())
            .or_default()
            .push(line);
        now.entry(cause.as_str()).or_default().push(line);
    }
    if std::env::var_os("BLESS").is_some() {
        let body: String = (now.iter())
            .map(|(file, at)| format!("{} {file}\n", at.len()))
            .collect();
        std::fs::write(root.join(LIST), body).unwrap();
        return;
    }
    let text = std::fs::read_to_string(root.join(LIST)).unwrap_or_default();
    let listed: BTreeMap<&str, usize> = (text.lines())
        .filter_map(|l| l.split_once(' '))
        .map(|(n, file)| (file, n.parse().unwrap()))
        .collect();
    let mut grew = Vec::new();
    let mut fell = Vec::new();
    for file in listed.keys().chain(now.keys()).collect::<BTreeSet<_>>() {
        let (was, is) = (
            listed.get(file).copied().unwrap_or(0),
            now.get(file).map_or(0, Vec::len),
        );
        if is > was {
            grew.push(format!(
                "{file}: {was} -> {is}\n  {}",
                now[file][..now[file].len().min(40)].join("\n  ")
            ));
        } else if is < was {
            fell.push(format!("{file}: {was} -> {is}"));
        }
    }
    assert!(
        grew.is_empty() && fell.is_empty(),
        "silent unknowns, the checker's lost types (fix the checker; `BLESS=1` rewrites {LIST} \
         once only falls are left):\ngrew:\n{}\nfell, tighten:\n{}",
        grew.join("\n"),
        fell.join("\n"),
    );
}
