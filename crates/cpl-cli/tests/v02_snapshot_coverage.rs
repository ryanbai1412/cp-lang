//! Re-porting a v0.2 port in place can't lose its regression coverage: every
//! port not yet in `corpus/sets/v03.txt` has its frozen copy under
//! `tests/v02-snapshot/`, and the corpus suites walk the copies that differ.

use std::path::Path;

#[path = "support/corpus_programs.rs"]
mod corpus_programs;

#[test]
fn every_v02_port_has_a_frozen_snapshot() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let missing = corpus_programs::v02_ports_without_snapshot(&repo);
    assert!(
        missing.is_empty(),
        "v0.2 ports with no tests/v02-snapshot copy: {missing:?}"
    );
}

#[test]
fn a_snapshot_port_runs_on_its_live_problem_tests() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let port = repo.join("tests/v02-snapshot/btoi19p5/port.cp");
    let dir = corpus_programs::problem_dir(&port);
    assert_eq!(
        dir.canonicalize().unwrap(),
        repo.join("corpus/problems/btoi19p5")
            .canonicalize()
            .unwrap()
    );
    let live = repo.join("corpus/problems/btoi19p5/port.cp");
    assert_eq!(corpus_programs::problem_dir(&live), live.parent().unwrap());
}
