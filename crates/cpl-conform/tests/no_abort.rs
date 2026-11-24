//! Lowering every conformance case finishes: a recursion in the checker or
//! lowerer that never ends overflows a worker's stack and aborts the whole run,
//! so no case gets a verdict. This test lowers each case on the runner's own
//! workers; an abort fails it.

use cpl_conform::find_repo_root;
use cpl_conform::runner::{compile_program, parallel_with_db};
use std::path::{Path, PathBuf};

fn cases(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            cases(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

#[test]
fn lowering_every_conformance_case_finishes() {
    let root = find_repo_root().unwrap();
    let mut files = Vec::new();
    cases(&root.join("spec/conformance"), &mut files);
    assert!(files.len() > 1000, "found only {} cases", files.len());
    let done = parallel_with_db(&files, |db, path| {
        let name = path.to_string_lossy().into_owned();
        let text = std::fs::read_to_string(path).unwrap();
        let _ = compile_program(db, &name, &text);
        (name, ())
    });
    assert_eq!(done.len(), files.len());
}
