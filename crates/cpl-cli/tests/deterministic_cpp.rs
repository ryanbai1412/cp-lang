//! `cpl build` is deterministic: building the same program twice gives the
//! same C++, byte for byte, over the corpus and the conformance cases. A
//! submission that changes between builds can't be reproduced or diffed,
//! and a hash-ordered walk in codegen shows up here first. Enforces
//! performance/R18 in handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

#[path = "support/v02_snapshot.rs"]
mod v02_snapshot;

/// The `.cp` files one directory down from the corpus and conformance roots.
fn sources() -> Vec<PathBuf> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    for root in ["corpus/problems", "spec/conformance"] {
        for dir in std::fs::read_dir(repo.join(root)).expect("source root exists") {
            let dir = dir.unwrap().path();
            if !dir.is_dir() {
                continue;
            }
            for file in std::fs::read_dir(dir).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "cp") {
                    found.push(path);
                }
            }
        }
    }
    found.extend(v02_snapshot::v02_snapshot_ports(&repo));
    found.sort();
    found
}

fn build(src: &Path, out: &Path) -> Option<Vec<u8>> {
    let run = cpl_cli::run(&["build", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    let cpp = std::fs::read(out).ok();
    let _ = std::fs::remove_file(out);
    (run.code == 0).then_some(cpp).flatten()
}

#[test]
fn building_a_program_twice_gives_the_same_cpp() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("deterministic_cpp");
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let chunk = sources.len().div_ceil(8);
    let built = std::sync::atomic::AtomicUsize::new(0);
    let differ: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = sources
            .chunks(chunk)
            .enumerate()
            .map(|(n, part)| {
                let (out, built) = (out.join(format!("{n}.cpp")), &built);
                s.spawn(move || {
                    let mut differ = Vec::new();
                    for src in part {
                        let Some(first) = build(src, &out) else {
                            continue;
                        };
                        built.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if build(src, &out).as_ref() != Some(&first) {
                            differ.push(src.display().to_string());
                        }
                    }
                    differ
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let built = built.into_inner();
    assert!(
        built >= 1000,
        "only {built} corpus and conformance programs built"
    );
    assert!(
        differ.is_empty(),
        "{} programs build to different C++ each time:\n{}",
        differ.len(),
        differ.join("\n")
    );
}
