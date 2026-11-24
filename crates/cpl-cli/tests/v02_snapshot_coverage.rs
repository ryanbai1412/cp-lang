//! Re-porting a v0.2 port in place can't lose its regression coverage: every
//! port not yet in `corpus/sets/v03.txt` has its frozen copy under
//! `tests/v02-snapshot/`, and the corpus suites walk the copies that differ.

use std::path::Path;

#[path = "support/v02_snapshot.rs"]
mod v02_snapshot;

#[test]
fn every_v02_port_has_a_frozen_snapshot() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let v03 = std::fs::read_to_string(repo.join("corpus/sets/v03.txt")).unwrap();
    let v03: std::collections::HashSet<&str> = v03.lines().map(str::trim).collect();
    let mut missing = Vec::new();
    for dir in std::fs::read_dir(repo.join("corpus/problems"))
        .unwrap()
        .flatten()
    {
        let id = dir.file_name().to_string_lossy().into_owned();
        if dir.path().join("port.cp").is_file()
            && !v03.contains(id.as_str())
            && !repo
                .join("tests/v02-snapshot")
                .join(&id)
                .join("port.cp")
                .is_file()
        {
            missing.push(id);
        }
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "v0.2 ports with no tests/v02-snapshot copy: {missing:?}"
    );
}

#[test]
fn a_snapshot_port_runs_on_its_live_problem_tests() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let port = repo.join("tests/v02-snapshot/btoi19p5/port.cp");
    let dir = v02_snapshot::problem_dir(&port);
    assert_eq!(
        dir.canonicalize().unwrap(),
        repo.join("corpus/problems/btoi19p5")
            .canonicalize()
            .unwrap()
    );
    let live = repo.join("corpus/problems/btoi19p5/port.cp");
    assert_eq!(v02_snapshot::problem_dir(&live), live.parent().unwrap());
}
