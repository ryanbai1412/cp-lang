//! Cold start: every `cpl` command answers within 100 ms (the M2 speed row in
//! handbook/product/milestones.md). Each sample is a fresh process, so the
//! number is what a contestant waits for from the shell or an editor's task.
//!
//! The median of several samples is asserted, and the 90th percentile must
//! stay within twice the budget, so a descheduled sample doesn't fail it.
//! The commands come from `cpl --help`, so a new one is covered without an
//! edit here. Each answers `--help`, the front-end commands also run on the
//! largest corpus ports, and `cpl test` on the largest ports with cases.
//! A program that `use`s every std module has the front-end budget instead,
//! 500 ms (`handbook/principles.md`, stdlib's "fast front end with std
//! loaded" row): today `check` takes about 110 ms and `explain` 210 ms on it.
//!
//! The budget describes the optimized build people run: `cargo test
//! --release` asserts it everywhere. An unoptimized build asserts it for
//! `--help` and prints the corpus numbers without asserting them (a check of
//! a 230-line port takes about 100 ms unoptimized, against 35 ms optimized):
//!
//! ```text
//! cargo test --release -p cpl-cli --test cold_start -- --nocapture
//! ```
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r7-cold-start`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_millis(100);
const SAMPLES: usize = 9;
const LARGEST_PORTS: usize = 3;

fn cpl() -> &'static str {
    env!("CARGO_BIN_EXE_cpl")
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Median and 90th percentile of `SAMPLES` fresh runs of `cpl args`.
fn time(args: &[&str]) -> (Duration, Duration) {
    let mut samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let out = Command::new(cpl())
                .args(args)
                .current_dir(root())
                .output()
                .unwrap();
            let took = start.elapsed();
            assert!(
                out.status.code().is_some(),
                "cpl {} was killed",
                args.join(" ")
            );
            took
        })
        .collect();
    samples.sort();
    (samples[SAMPLES / 2], samples[SAMPLES * 9 / 10])
}

fn commands() -> Vec<String> {
    let out = Command::new(cpl()).arg("--help").output().unwrap();
    let help = String::from_utf8(out.stdout).unwrap();
    let names: Vec<String> = help
        .lines()
        .skip_while(|line| *line != "commands:")
        .take_while(|line| *line != "options:")
        // A command starts its line; a wrapped description is indented more.
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|word| word.chars().all(|c| c.is_ascii_lowercase()))
        .map(str::to_string)
        .collect();
    assert!(names.len() >= 20, "`cpl --help` lists {names:?}");
    names
}

fn ports(dir: &Path, out: &mut Vec<(usize, PathBuf)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            ports(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            let lines = std::fs::read_to_string(&path).map_or(0, |s| s.lines().count());
            out.push((lines, path));
        }
    }
}

/// The largest corpus ports; with `tested`, only those with a `tests/` folder.
fn largest_ports(tested: bool) -> Vec<String> {
    let mut all = Vec::new();
    ports(&root().join("corpus"), &mut all);
    all.retain(|(_, path)| !tested || path.with_file_name("tests").is_dir());
    all.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    all.into_iter()
        .take(LARGEST_PORTS)
        .map(|(_, path)| {
            path.strip_prefix(root())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// A program that imports every module in `std/` and calls the prelude.
fn every_std_module_program() -> String {
    let mut modules: Vec<String> = std::fs::read_dir(root().join("std"))
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let stem = name.strip_suffix(".cp")?;
            (stem != "prelude").then(|| stem.to_string())
        })
        .collect();
    modules.sort();
    assert!(modules.len() >= 30, "std/ has {modules:?}");
    let mut src: String = modules.iter().map(|m| format!("use std.{m}.*\n")).collect();
    src.push_str("let a = [5, 1, 4]\nprint(a.sorted())\n");
    src
}

#[test]
fn every_command_starts_cold_in_under_100_ms() {
    let optimized = !cfg!(debug_assertions);
    let out = std::env::temp_dir().join(format!("cpl-cold-start-{}.cpp", std::process::id()));
    let out = out.to_str().unwrap();
    let mut runs: Vec<(Vec<String>, bool)> = commands()
        .into_iter()
        .map(|c| (vec![c, "--help".to_string()], true))
        .collect();
    for port in largest_ports(false) {
        for cmd in ["check", "lint", "explain"] {
            runs.push((vec![cmd.to_string(), port.clone()], optimized));
        }
        runs.push((
            vec!["build".into(), port.clone(), "-o".into(), out.into()],
            optimized,
        ));
    }
    // `cpl test` compiles the C++ once per source; the start that counts is
    // the next run, against the cache, which is every run while you iterate.
    for port in largest_ports(true) {
        Command::new(cpl())
            .args(["test", &port])
            .current_dir(root())
            .output()
            .unwrap();
        runs.push((vec!["test".into(), port], optimized));
    }
    let mut over = Vec::new();
    for (args, asserted) in &runs {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let (median, p90) = time(&args);
        println!(
            "{median:>10.1?} median {p90:>10.1?} p90  cpl {}",
            args.join(" ")
        );
        if *asserted && (median > BUDGET || p90 > 2 * BUDGET) {
            over.push(format!(
                "cpl {}: median {median:.1?}, p90 {p90:.1?}",
                args.join(" ")
            ));
        }
    }
    let _ = std::fs::remove_file(out);
    assert!(
        over.is_empty(),
        "over the {BUDGET:?} cold-start budget:\n{}",
        over.join("\n")
    );
}

#[test]
fn every_std_module_loads_within_the_front_end_budget() {
    const FRONT_END: Duration = Duration::from_millis(500);
    let program =
        std::env::temp_dir().join(format!("cpl-cold-start-std-{}.cp", std::process::id()));
    std::fs::write(&program, every_std_module_program()).unwrap();
    let path = program.to_str().unwrap();
    let mut over = Vec::new();
    for cmd in ["check", "explain"] {
        let (median, p90) = time(&[cmd, path]);
        println!("{median:>10.1?} median {p90:>10.1?} p90  cpl {cmd} <every std module>");
        if !cfg!(debug_assertions) && (median > FRONT_END || p90 > 2 * FRONT_END) {
            over.push(format!("cpl {cmd}: median {median:.1?}, p90 {p90:.1?}"));
        }
    }
    let _ = std::fs::remove_file(&program);
    assert!(
        over.is_empty(),
        "over the {FRONT_END:?} front-end budget with all of std:\n{}",
        over.join("\n")
    );
}
