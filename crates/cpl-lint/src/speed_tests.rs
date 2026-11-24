//! Lint and fmt add little to the editor's response
//! (`handbook/guidelines/lints-checklist.md`, `lints/R24`): on the largest
//! corpus ports, the files a contestant edits, linting a checked file and
//! formatting it each take at most a tenth of the 500 ms front-end budget.
//! The largest `std` modules, ten times longer, are timed and printed.
//!
//! The budget describes the optimized build: `cargo test --release` asserts
//! it, an unoptimized build only prints the numbers:
//!
//! ```text
//! cargo test --release -p cpl-lint speed_tests -- --nocapture
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_millis(50);
const SAMPLES: usize = 7;

fn largest(dir: &Path, leaf: &str, n: usize) -> Vec<PathBuf> {
    let mut files: Vec<(u64, PathBuf)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| {
            if leaf.is_empty() {
                e.path()
            } else {
                e.path().join(leaf)
            }
        })
        .filter(|p| p.extension().is_some_and(|e| e == "cp") && p.is_file())
        .map(|p| (p.metadata().unwrap().len(), p))
        .collect();
    files.sort();
    files.into_iter().rev().take(n).map(|(_, p)| p).collect()
}

fn median(mut run: impl FnMut()) -> Duration {
    let mut samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            run();
            start.elapsed()
        })
        .collect();
    samples.sort();
    samples[SAMPLES / 2]
}

#[test]
fn lint_and_fmt_of_the_largest_files_fit_a_tenth_of_the_front_end_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = crate::Config::default();
    let ports = largest(&root.join("corpus/problems"), "port.cp", 3);
    let modules = largest(&root.join(crate::STD_DIR), "", 3);
    let mut slow = Vec::new();
    for path in ports.iter().chain(&modules) {
        let src = std::fs::read_to_string(path).unwrap();
        let mut db = cpl_check::Session::new();
        db.set_file("<lint>.cp", src.as_str());
        let Some(analysis) = db.analyze("<lint>.cp") else {
            continue;
        };
        let lint = median(|| drop(crate::typed::lint_analysis(&analysis, &config)));
        let fmt = median(|| drop(cpl_fmt::format(&src)));
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        println!("{name}: lint {lint:?}, fmt {fmt:?}");
        if lint.max(fmt) > BUDGET && ports.contains(path) {
            slow.push(format!("{name}: lint {lint:?}, fmt {fmt:?}"));
        }
    }
    if !cfg!(debug_assertions) {
        assert!(slow.is_empty(), "over {BUDGET:?}:\n{}", slow.join("\n"));
    }
}
