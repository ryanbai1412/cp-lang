//! The frozen v0.2 ports, `tests/v02-snapshot/<id>/port.cp`, that the corpus
//! suites keep covering while `corpus/problems/<id>/port.cp` is re-ported in
//! place. Shared by every suite that walks the corpus, so none drifts.

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
