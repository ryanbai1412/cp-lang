//! Every input a program reads is one `cpl stress` can generate
//! (`debugging/R11` in `handbook/guidelines/debugging-checklist.md`): each
//! corpus problem's spec, each saved-trace program and each conformance
//! program that builds gets an input spec. There is no list of exceptions: a
//! program stress can't generate fails here until the generator learns it.

use std::path::{Path, PathBuf};

use cpl_stress::spec::InputSpec;

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|x| x == "cp") {
            out.push(path);
        }
    }
}

/// `path: why` for each program whose input stress can't generate.
fn gaps(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for dir in [
        "corpus/problems",
        "spec/conformance",
        "crates/cpl-cli/tests/programs",
    ] {
        walk(&root.join(dir), &mut files);
    }
    files.retain(|f| !f.starts_with(root.join("corpus")) || f.ends_with("spec.cp"));
    files.sort();
    let mut gaps = Vec::new();
    for file in files {
        let src = std::fs::read_to_string(&file).unwrap();
        let builds = src
            .lines()
            .find_map(|l| l.strip_prefix("// expect: "))
            .is_none_or(|expect| expect.starts_with("ok") || expect.starts_with("trap"));
        if !builds || !src.contains("read") {
            continue;
        }
        if let Err(error) = InputSpec::from_source(&src) {
            let rel = file.strip_prefix(root).unwrap().display().to_string();
            gaps.push(format!("{rel}: {}", error.message));
        }
    }
    gaps
}

#[test]
fn every_readable_input_can_be_generated() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let found = gaps(&root);
    assert!(
        found.is_empty(),
        "stress can't generate these inputs; teach the generator the shape:\n{}",
        found.join("\n")
    );
}
