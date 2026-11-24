//! The programs the corpus suites walk, shared so none drifts: the corpus and
//! conformance sources, plus the frozen v0.2 ports
//! (`tests/v02-snapshot/<id>/port.cp`) the suites keep covering while
//! `corpus/problems/<id>/port.cp` is re-ported in place.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The snapshot ports whose live `corpus/problems/<id>/port.cp` differs or is
/// gone; an identical live port already stands for its snapshot.
pub fn v02_snapshot_ports(repo: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(dirs) = std::fs::read_dir(repo.join("tests/v02-snapshot")) else {
        return out;
    };
    for dir in dirs.flatten() {
        let old = dir.path().join("port.cp");
        let live = repo
            .join("corpus/problems")
            .join(dir.file_name())
            .join("port.cp");
        if old.is_file() && std::fs::read(&old).ok() != std::fs::read(&live).ok() {
            out.push(old);
        }
    }
    out.sort();
    out
}

/// The problem directory a port's tests live in: its own for a corpus port,
/// `corpus/problems/<id>` for a snapshot port.
pub fn problem_dir(port: &Path) -> PathBuf {
    let dir = port.parent().unwrap();
    match (dir.parent(), dir.file_name()) {
        (Some(snap), Some(id)) if snap.ends_with("tests/v02-snapshot") => {
            snap.join("../../corpus/problems").join(id)
        }
        _ => dir.to_path_buf(),
    }
}

/// The `.cp` files one directory down from the corpus problems and the
/// conformance cases, plus the snapshot ports that differ from their live port:
/// what the build-and-agree suites run.
pub fn corpus_and_conformance(repo: &Path) -> Vec<PathBuf> {
    let mut found = shipped(repo);
    found.extend(v02_snapshot_ports(repo));
    found.sort();
    found
}

/// The corpus and conformance sources without the snapshot: what the idiom
/// and perf gates judge. The snapshot is a frozen regression input, never
/// rewritten to meet today's taste.
pub fn shipped(repo: &Path) -> Vec<PathBuf> {
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

/// Problems with a live port outside `corpus/sets/v03.txt` (not yet
/// re-ported) and no frozen copy under `tests/v02-snapshot/`.
pub fn v02_ports_without_snapshot(repo: &Path) -> Vec<String> {
    let v03 = std::fs::read_to_string(repo.join("corpus/sets/v03.txt")).unwrap();
    let v03: std::collections::HashSet<&str> = v03.lines().map(str::trim).collect();
    let mut missing: Vec<String> = corpus_and_conformance(repo)
        .iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "port.cp"))
        .filter_map(|p| {
            let dir = p.parent()?;
            let id = dir.file_name()?.to_str()?;
            let live = !dir.parent()?.ends_with("v02-snapshot");
            let frozen = repo.join("tests/v02-snapshot").join(id).join("port.cp");
            (live && !v03.contains(id) && !frozen.is_file()).then(|| id.to_string())
        })
        .collect();
    missing.sort();
    missing
}
