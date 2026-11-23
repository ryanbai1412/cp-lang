//! Every input a program reads is one `cpl stress` can generate
//! (`debugging/R11` in `handbook/guidelines/debugging-checklist.md`): each
//! corpus problem's spec, each saved-trace program and each conformance
//! program that builds gets an input spec, or is listed with the reason in
//! `generate-gaps.txt`. A fix deletes its line; the list only shrinks. After
//! a change that is meant to change it, run with `CPL_BLESS=1`.

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
fn every_readable_input_can_be_generated_or_is_listed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/generate-gaps.txt");
    let found = gaps(&root);
    let header = "# Programs whose input `cpl stress` can't generate yet, and why.\n\
                  # Checked by readable_shapes.rs; a fix deletes its line.\n";
    if std::env::var_os("CPL_BLESS").is_some() {
        std::fs::write(&list, format!("{header}{}\n", found.join("\n"))).unwrap();
        return;
    }
    let listed: Vec<String> = std::fs::read_to_string(&list)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let new: Vec<&String> = found.iter().filter(|g| !listed.contains(g)).collect();
    let fixed: Vec<&String> = listed.iter().filter(|g| !found.contains(g)).collect();
    assert!(
        new.is_empty(),
        "stress can't generate these inputs; teach the generator the shape:\n{}",
        new.iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        fixed.is_empty(),
        "these now generate; delete their lines from tests/generate-gaps.txt:\n{}",
        fixed
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
