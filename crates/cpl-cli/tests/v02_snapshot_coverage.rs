//! Re-porting a v0.2 port in place can't lose its regression coverage: every
//! port not yet in `corpus/sets/v03.txt` has its frozen copy under
//! `tests/v02-snapshot/`, which only its pinned release builds
//! (`pinned_release.rs`): today's checker sees the live ports.

use std::path::Path;

#[test]
fn every_v02_port_has_a_frozen_snapshot() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let missing = cpl_corpus::programs::v02_ports_without_snapshot(&repo);
    assert!(
        missing.is_empty(),
        "v0.2 ports with no tests/v02-snapshot copy: {missing:?}"
    );
}

#[test]
fn a_snapshot_port_runs_on_its_live_problem_tests() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let port = repo.join("tests/v02-snapshot/btoi19p5/port.cp");
    let dir = cpl_corpus::programs::problem_dir(&port);
    assert_eq!(
        dir.canonicalize().unwrap(),
        repo.join("corpus/problems/btoi19p5")
            .canonicalize()
            .unwrap()
    );
    let live = repo.join("corpus/problems/btoi19p5/port.cp");
    assert_eq!(
        cpl_corpus::programs::problem_dir(&live),
        live.parent().unwrap()
    );
}

#[test]
fn no_suite_walks_a_frozen_port_with_todays_checker() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let frozen: Vec<_> = cpl_corpus::programs::corpus_and_conformance(&repo)
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "v02-snapshot"))
        .collect();
    assert!(
        frozen.is_empty(),
        "only pinned_release builds a frozen port (decision 620): {frozen:?}"
    );
}
