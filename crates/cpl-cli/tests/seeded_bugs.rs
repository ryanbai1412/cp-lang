//! The seeded-bug benchmark: every `bench/stress/<bug>.cp` is a port with
//! a classic bug planted in it and the correct fn as its `@brute`. One
//! `cpl stress <bug>.cp` must find the bug and shrink it to
//! `bench/stress/<bug>.min`, the smallest input that shows it. The time to
//! that minimal counterexample is printed per bug, with the median.
//!
//! What passes is decided by the input alone: a fixed seed and case
//! count, and a run time limit no loaded machine reaches. The times are
//! printed, never asserted, so a busy gate can't fail it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const STRESS: [&str; 8] = [
    "--seed",
    "1",
    "--cases",
    "1000",
    "--time-limit",
    "60000",
    "--format",
    "json",
];

fn bugs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/stress");
    let mut bugs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    bugs.sort();
    bugs
}

/// The JSON string value of `key` in `json`, unescaped (top-level or nested under `counterexample`).
fn string_field(json: &str, key: &str) -> Option<String> {
    let root = cpl_json::parse(json).ok()?;
    root.get(key)
        .or_else(|| root.get("counterexample")?.get(key))
        .and_then(cpl_json::Value::as_str)
        .map(str::to_string)
}

/// The bug classes a seeded bug may name on its `// class:` line: the P0
/// classes of `AGENTS.md` goal 2, then `logic` for a wrong algorithm.
const CLASSES: [&str; 12] = [
    "off-by-one",
    "index-space",
    "overflow",
    "boundaries",
    "multitest-leak",
    "io-flush",
    "sentinel",
    "complexity",
    "mutation",
    "modulus",
    "floating-point",
    "logic",
];

/// What `bug`'s `// class:` line names.
fn class_of(bug: &Path) -> Option<String> {
    let source = std::fs::read_to_string(bug).ok()?;
    let line = source
        .lines()
        .find_map(|line| line.strip_prefix("// class: "))?;
    Some(line.trim().to_string())
}

#[test]
fn every_seeded_bug_names_its_bug_class() {
    let mut count = [0usize; CLASSES.len()];
    for bug in bugs() {
        let class = class_of(&bug);
        let at = class
            .as_deref()
            .and_then(|c| CLASSES.iter().position(|&k| k == c));
        let Some(at) = at else {
            panic!(
                "{}: no `// class:` line naming one of {CLASSES:?}",
                bug.display()
            );
        };
        count[at] += 1;
    }
    for (class, n) in CLASSES.iter().zip(count) {
        println!("{class:<16} {n}");
    }
    let p0 = CLASSES.len() - 1;
    let covered = count[..p0].iter().filter(|&&n| n > 0).count();
    println!("{covered} of {p0} P0 classes seeded");
}

/// `run` on every bug, a few at a time.
fn each<T: Send>(bugs: &[PathBuf], run: impl Fn(&PathBuf) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(4);
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(bug) = bugs.get(at) else { break done };
                        done.push(run(bug));
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    })
}

/// `cpl stress <bug>` on a copy of `bug` in `scratch`, so the saved
/// regression test lands outside the repo, with `args` after the file.
fn stress(bug: &Path, scratch: &Path, args: &[&str]) -> std::process::Output {
    let copy = scratch.join(bug.file_name().unwrap());
    std::fs::copy(bug, &copy).unwrap();
    // A bug that never ends names a short limit, so each case it hangs on stops fast.
    let source = std::fs::read_to_string(bug).unwrap();
    let limit = (source.lines())
        .find_map(|line| line.strip_prefix("// time-limit-ms: "))
        .map(|ms| ["--time-limit", ms.trim()]);
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("stress")
        .arg(&copy)
        .args(args)
        .args(limit.iter().flatten())
        .output()
        .expect("cpl runs")
}

/// A fresh scratch folder for the copies of the bugs one test stresses.
fn scratch(test: &str) -> PathBuf {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    scratch
}

#[test]
fn stress_finds_every_seeded_bug_and_shrinks_it_to_the_minimal_input() {
    let bugs = bugs();
    assert!(bugs.len() >= 21, "only {} seeded bugs", bugs.len());
    let scratch = scratch("seeded_bugs");
    let run = |bug: &PathBuf| -> (String, Duration, Option<String>) {
        let name = bug.file_stem().unwrap().to_string_lossy().to_string();
        let minimal = std::fs::read_to_string(bug.with_extension("min")).unwrap();
        let start = Instant::now();
        let out = stress(bug, &scratch, &STRESS);
        let took = start.elapsed();
        let json = String::from_utf8_lossy(&out.stdout).to_string();
        let found = string_field(&json, "raw_input");
        let wrong = (found.as_deref() != Some(minimal.as_str())).then(|| {
            format!(
                "{name}: wanted\n{minimal}got {found:?}\n{json}{}",
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (name, took, wrong)
    };
    let mut results = each(&bugs, run);
    results.sort_by(|a, b| a.0.cmp(&b.0));
    let mut times = Vec::new();
    let mut wrong = Vec::new();
    for (name, took, failure) in results {
        let verdict = if failure.is_some() {
            "WRONG"
        } else {
            "minimal"
        };
        let class = class_of(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../bench/stress")
                .join(format!("{name}.cp")),
        )
        .unwrap_or_default();
        println!(
            "{name:<32} {class:<16} {:>6.2} s  {verdict}",
            took.as_secs_f64()
        );
        times.push(took);
        wrong.extend(failure);
    }
    times.sort();
    let median = times[times.len() / 2];
    println!(
        "median {:.2} s over {} seeded bugs",
        median.as_secs_f64(),
        times.len()
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// How many seeded bugs `cpl stress --explain` points at the very line
/// the bug was planted on, the line ending in `// planted`. Raise it when
/// the explanation gets better; it may never drop.
const NAMED: usize = 19;

/// The lines a `cpl stress --explain` report blames: where the debug run
/// stopped, and where the solution first gave a value the brute did not.
fn blamed(report: &str) -> Vec<u32> {
    let number = |text: &str| -> Option<u32> {
        let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    };
    (report.lines())
        .filter_map(|line| {
            let line = line.trim_start();
            if line.starts_with("--> ") {
                number(line.rsplit_once(".cp:")?.1)
            } else if let Some(rest) = line.strip_prefix("look at line ") {
                number(rest)
            } else if line.starts_with("solution ") {
                number(line.rsplit_once(" at line ")?.1)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn explain_names_the_line_each_bug_was_planted_on() {
    let bugs = bugs();
    let scratch = scratch("seeded_bugs_explain");
    let args = &STRESS[..STRESS.len() - 2];
    let mut results = each(&bugs, |bug| {
        let name = bug.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(bug).unwrap();
        let planted = (source.lines().position(|line| line.ends_with("// planted")))
            .unwrap_or_else(|| panic!("{name}: no line ends in `// planted`"))
            as u32
            + 1;
        let out = stress(bug, &scratch, &[args, &["--explain"]].concat());
        let report = String::from_utf8_lossy(&out.stdout).to_string();
        (name, planted, blamed(&report))
    });
    results.sort();
    let mut named = 0;
    for (name, planted, blamed) in &results {
        let verdict = if blamed.contains(planted) {
            named += 1;
            "named"
        } else {
            "missed"
        };
        println!("{name:<32} planted at {planted:<3} blamed {blamed:?} {verdict}");
    }
    println!(
        "{named} of {} seeded bugs named at their line",
        results.len()
    );
    assert!(named >= NAMED, "only {named} named, down from {NAMED}");
}
