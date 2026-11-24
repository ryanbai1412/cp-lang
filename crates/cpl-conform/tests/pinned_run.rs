//! A rebuild or checkout during a run can't change its verdicts: the run checks against
//! the std it pinned when it started.

use cpl_check::Session;
use cpl_conform::conformance::{run_one, ConformanceCase};
use cpl_conform::find_repo_root;
use cpl_conform::pin::{self, Pin};
use std::path::{Path, PathBuf};

fn read_std(root: &Path, dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            read_std(root, &path, found);
        } else if path.extension().is_some_and(|e| e == "cp") {
            let name = path.strip_prefix(root).unwrap().with_extension("");
            found.push((
                name.to_string_lossy().replace('\\', "/"),
                std::fs::read_to_string(&path).unwrap(),
            ));
        }
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

fn garble(dir: &Path) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            garble(&path);
        } else {
            std::fs::write(&path, "pub fn (((\n").unwrap();
        }
    }
}

fn spawned_std() -> Option<PathBuf> {
    pin::command("cpl")
        .get_envs()
        .find(|(key, _)| *key == "CPL_STD_DIR")
        .and_then(|(_, dir)| dir.map(PathBuf::from))
}

#[test]
fn std_changed_mid_run_changes_no_verdict() {
    let root = find_repo_root().unwrap();
    let scratch =
        std::env::temp_dir().join(format!("cpl-conform-pinned-run-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let live_std = scratch.join("std");
    copy_tree(&root.join("std"), &live_std);
    let mut loaded = Vec::new();
    read_std(&live_std, &live_std, &mut loaded);
    let modules: Vec<(&str, &str)> = loaded
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();

    let mut db = Session::new();
    let cases = ["core-07/chain-fn", "std-seq/chain-1"].map(|id| ConformanceCase {
        id: id.to_string(),
        path: root.join(format!("spec/conformance/{id}.cp")),
    });
    let before: Vec<String> = cases.iter().map(|c| run_one(&mut db, c).status).collect();

    let pinned = Pin::new(&modules).unwrap();
    garble(&live_std);

    let copy = spawned_std().expect("a spawned cpl reads the pinned std");
    assert_ne!(copy, live_std);
    for (name, text) in &modules {
        assert_eq!(
            &std::fs::read_to_string(copy.join(format!("{name}.cp"))).unwrap(),
            text
        );
    }
    let during: Vec<String> = cases.iter().map(|c| run_one(&mut db, c).status).collect();
    assert_eq!(before, during);
    assert_eq!(during, ["pass", "pass"]);

    drop(pinned);
    assert!(!copy.exists(), "the copy goes when the run ends");
    assert_eq!(spawned_std(), None);
    let _ = std::fs::remove_dir_all(&scratch);
}
