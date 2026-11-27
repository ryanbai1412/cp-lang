//! Every byte of every CP Lang file in the repository survives the tree:
//! the conformance suite, the std library, the cpl-next test programs,
//! the corpus and the lint specs.

use cpl_next_syntax::{lex, parse, NodeKind};
use std::path::{Path, PathBuf};

fn cp_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

fn repo_files(dirs: &[&str]) -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = Vec::new();
    for dir in dirs {
        let before = paths.len();
        cp_files(&root.join(dir), &mut paths);
        assert!(paths.len() > before, "no .cp files under {dir}");
    }
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| Some((p.clone(), std::fs::read_to_string(&p).ok()?)))
        .collect()
}

fn round_trips(dirs: &[&str]) {
    let mut broken = Vec::new();
    for (path, src) in repo_files(dirs) {
        let lexed = lex(&src);
        let tokens: String = lexed
            .lossless(&src)
            .iter()
            .map(|t| t.span.text(&src))
            .collect();
        let file = parse(&src);
        let tree = file.root.text();
        let items_are_whole = file.items().all(|i| i.kind() == NodeKind::Item);
        if tokens != src || tree != src || !items_are_whole {
            broken.push(path.display().to_string());
        }
    }
    assert!(
        broken.is_empty(),
        "{} files lose bytes: {broken:#?}",
        broken.len()
    );
}

#[test]
fn every_conformance_file_round_trips() {
    round_trips(&["spec/conformance"]);
}

#[test]
fn every_std_file_round_trips() {
    round_trips(&["std"]);
}

#[test]
fn every_test_program_and_corpus_file_round_trips() {
    round_trips(&[
        "crates/cpl-next/tests",
        "corpus",
        "spec/lints",
        "spec/checker-lints",
    ]);
}
