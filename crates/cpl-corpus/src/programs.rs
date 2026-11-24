//! The programs the corpus suites walk, shared so none drifts: the corpus and
//! conformance sources. A frozen v0.2 port (`tests/v02-snapshot/<id>/port.cp`)
//! records what an old release accepted, so only its pinned release builds it
//! (`pinned_release.rs`); today's checker sees the live ports instead.

use std::path::{Path, PathBuf};

/// The problem directory a port's tests live in: its own for a corpus port,
/// `corpus/problems/<id>` for a snapshot port.
pub fn problem_dir(port: &Path) -> PathBuf {
    let dir = port.parent().unwrap_or(port);
    match (dir.parent(), dir.file_name()) {
        (Some(snap), Some(id)) if snap.ends_with("tests/v02-snapshot") => {
            crate::package(&snap.join("../.."), &id.to_string_lossy())
        }
        _ => dir.to_path_buf(),
    }
}

/// The `.cp` files one directory down from the corpus problems and the
/// conformance cases: what the build-and-agree suites run.
pub fn corpus_and_conformance(repo: &Path) -> Vec<PathBuf> {
    shipped(repo)
}

/// The corpus and conformance sources without the snapshot: what the idiom
/// and perf gates judge. The snapshot is a frozen regression input, never
/// rewritten to meet today's taste.
pub fn shipped(repo: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let conformance = std::fs::read_dir(repo.join("spec/conformance"))
        .into_iter()
        .flatten();
    let dirs = conformance
        .flatten()
        .map(|d| d.path())
        .filter(|d| d.is_dir());
    for dir in crate::packages(repo).into_iter().chain(dirs) {
        let files = std::fs::read_dir(dir).into_iter().flatten().flatten();
        found.extend(
            files
                .map(|f| f.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "cp")),
        );
    }
    found.sort();
    found
}

/// Problems with a live port outside `corpus/sets/v03.txt` (not yet
/// re-ported) and no frozen copy under `tests/v02-snapshot/`.
pub fn v02_ports_without_snapshot(repo: &Path) -> Vec<String> {
    let v03 = std::fs::read_to_string(repo.join("corpus/sets/v03.txt")).unwrap_or_default();
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
