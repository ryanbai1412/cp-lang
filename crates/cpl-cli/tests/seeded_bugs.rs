//! The seeded-bug benchmark: every `bench/stress/<bug>.cp` is a port with
//! a classic bug planted in it and the correct fn as its `@brute`. One
//! `cpl stress <bug>.cp` must find the bug and shrink it to
//! `bench/stress/<bug>.min`, the smallest input that shows it. The time to
//! that minimal counterexample is printed per bug, with the median.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// `GOALS` 4: a seeded bug is found and shrunk in a median of 10 s.
const MEDIAN_LIMIT: Duration = Duration::from_secs(10);

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

/// The JSON string value of `key` in `json`, unescaped.
fn string_field(json: &str, key: &str) -> Option<String> {
    let start = json.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut out = String::new();
    let mut chars = json[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                }
                other => out.push(other),
            },
            c => out.push(c),
        }
    }
    None
}

#[test]
fn stress_finds_every_seeded_bug_and_shrinks_it_to_the_minimal_input() {
    let bugs = bugs();
    assert!(bugs.len() >= 4, "only {} seeded bugs", bugs.len());
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("seeded_bugs");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let mut times = Vec::new();
    let mut wrong = Vec::new();
    for bug in &bugs {
        let name = bug.file_stem().unwrap().to_string_lossy().to_string();
        // A copy, so the saved regression test lands outside the repo.
        let copy = scratch.join(bug.file_name().unwrap());
        std::fs::copy(bug, &copy).unwrap();
        let minimal = std::fs::read_to_string(bug.with_extension("min")).unwrap();
        let start = Instant::now();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["stress".as_ref(), copy.as_os_str(), "--format".as_ref(), "json".as_ref()])
            .output()
            .expect("cpl runs");
        let took = start.elapsed();
        let json = String::from_utf8_lossy(&out.stdout).to_string();
        let found = string_field(&json, "raw_input");
        println!("{name:<32} {:>6.2} s  {}", took.as_secs_f64(), if found.as_deref() == Some(minimal.as_str()) { "minimal" } else { "WRONG" });
        times.push(took);
        if found.as_deref() != Some(minimal.as_str()) {
            wrong.push(format!(
                "{name}: wanted\n{minimal}got {found:?}\n{json}{}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    times.sort();
    let median = times[times.len() / 2];
    println!("median {:.2} s over {} seeded bugs", median.as_secs_f64(), times.len());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(median <= MEDIAN_LIMIT, "median time to a minimal counterexample is {median:?}");
}
