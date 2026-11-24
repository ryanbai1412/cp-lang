//! `cpl profile` and `cpl cover`.
//!
//! The interpreter and the profiled C++ keep the same books: on every
//! program under `tests/programs`, run on its `.in`, each line's hit count
//! is the same from both, including on the runs that fail.

use std::path::{Path, PathBuf};
use std::process::Command;

fn cpl(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .output()
        .expect("cpl runs");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

fn scratch(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-profile-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

const SUM: &str = "\
let n: Int = read()
var s = 0
for i in 0..n:
    s += i
    if i % 2 == 0:
        s += 1
print(s)
";

/// `(line, hits)` of each line that ran, from `--format json`.
fn hits(json: &str) -> Vec<(u64, u64)> {
    let lines = &json[json.find("\"lines\":[").expect("lines") + 9..];
    lines
        .split("{\"line\":")
        .skip(1)
        .map(|row| {
            let line = row[..row.find(',').unwrap()].parse().unwrap();
            let hits = &row[row.find("\"hit_count\":").unwrap() + 12..];
            (line, hits[..hits.find(',').unwrap()].parse().unwrap())
        })
        .collect()
}

fn has_gxx() -> bool {
    Command::new("g++").arg("--version").output().is_ok()
}

#[test]
fn profile_counts_each_line() {
    let file = scratch("sum.cp", SUM);
    let input = scratch("sum.in", "10\n");
    let (out, err, code) = cpl(&[
        "profile",
        file.to_str().unwrap(),
        input.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        hits(&out),
        [(1, 1), (2, 1), (3, 1), (4, 10), (5, 10), (6, 5), (7, 1)]
    );
    let (text, _, _) = cpl(&["profile", file.to_str().unwrap(), input.to_str().unwrap()]);
    assert!(text.contains("profile of"), "{text}");
    assert!(
        text.lines()
            .any(|line| line.contains("10") && line.ends_with("    s += i")),
        "{text}"
    );
}

#[test]
fn profile_charges_library_code_to_the_line_that_calls_it() {
    let file = scratch(
        "sorted.cp",
        "let xs = [3, 1, 2]\nlet ys = xs.sorted()\nprint(ys)\n",
    );
    let (out, err, code) = cpl(&["profile", file.to_str().unwrap(), "-", "--format", "json"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(hits(&out), [(1, 1), (2, 1), (3, 1)]);
}

#[test]
fn profile_of_a_failed_run_still_reports_and_fails() {
    let file = scratch(
        "fails.cp",
        "let xs = [1, 2]\nvar i = 0\nwhile true:\n    print(xs[i])\n    i += 1\n",
    );
    let (out, err, code) = cpl(&["profile", file.to_str().unwrap(), "-", "--format", "json"]);
    assert_eq!(code, 3, "{err}");
    assert!(err.contains("the run failed"), "{err}");
    assert_eq!(hits(&out), [(1, 1), (2, 1), (3, 1), (4, 3), (5, 2)]);
}

#[test]
fn cover_lists_lines_and_branches_no_input_reached() {
    let file = scratch("cover.cp", SUM);
    let zero = scratch("zero.in", "0\n");
    let (out, err, code) = cpl(&["cover", file.to_str().unwrap(), zero.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("4 of 7 lines covered"), "{out}");
    for line in [
        "line    4  never ran",
        "line    5  never ran",
        "line    6  never ran",
    ] {
        assert!(out.contains(line), "{out}");
    }
    let one = scratch("one.in", "1\n");
    let (out, _, _) = cpl(&[
        "cover",
        file.to_str().unwrap(),
        one.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        out.contains(
            "\"covered_count\":7,\"uncovered\":[],\"one_way\":[{\"line\":5,\"always\":true}]"
        ),
        "{out}"
    );
    let (out, _, _) = cpl(&[
        "cover",
        file.to_str().unwrap(),
        zero.to_str().unwrap(),
        one.to_str().unwrap(),
    ]);
    assert!(
        out.contains("7 of 7 lines covered (100%) by 2 inputs"),
        "{out}"
    );
}

#[test]
fn cover_reads_the_problems_cases() {
    let file = scratch("cases.cp", SUM);
    let cases = file.with_extension("tests");
    std::fs::create_dir_all(&cases).unwrap();
    std::fs::write(cases.join("1.in"), "3\n").unwrap();
    let (out, err, code) = cpl(&["cover", file.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("7 of 7 lines covered (100%) by 1 input"),
        "{out}"
    );
}

/// The programs are split into `SHARDS` tests by their index in sorted
/// order, as in `differential.rs`, so the test runner spreads them over its
/// threads and nothing else here starts threads of its own.
const SHARDS: usize = 8;

fn programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .collect();
    programs.sort();
    programs
}

/// An empty input file; `/dev/null` doesn't exist on Windows.
fn empty_input() -> String {
    let path = std::env::temp_dir().join(format!("cpl-profile-empty-{}.in", std::process::id()));
    std::fs::write(&path, "").unwrap();
    path.to_string_lossy().into_owned()
}

fn native_agrees_on_shard(shard: usize) {
    if !has_gxx() {
        eprintln!("skipped: no g++");
        return;
    }
    let programs = programs();
    assert!(programs.len() > 150, "{} programs", programs.len());
    let mut failures = Vec::new();
    for program in programs.iter().skip(shard).step_by(SHARDS) {
        let input = program.with_extension("in");
        let input = if input.is_file() {
            input.to_string_lossy().into_owned()
        } else {
            empty_input()
        };
        let program = program.to_string_lossy();
        let (interp, _, _) = cpl(&["profile", &program, &input, "--format", "json"]);
        let (native, err, _) = cpl(&["profile", &program, &input, "--native", "--format", "json"]);
        if !native.contains("\"lines\"") {
            failures.push(format!("{program}: native profile failed: {err}"));
        } else if hits(&interp) != hits(&native) {
            failures.push(format!(
                "{program}:\n  interpreter {:?}\n  native      {:?}",
                hits(&interp),
                hits(&native)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} programs disagree:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

macro_rules! shards {
    ($($test:ident = $shard:expr),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                native_agrees_on_shard($shard);
            }
        )*
    };
}

shards! {
    native_profile_counts_what_the_interpreter_counts_0 = 0,
    native_profile_counts_what_the_interpreter_counts_1 = 1,
    native_profile_counts_what_the_interpreter_counts_2 = 2,
    native_profile_counts_what_the_interpreter_counts_3 = 3,
    native_profile_counts_what_the_interpreter_counts_4 = 4,
    native_profile_counts_what_the_interpreter_counts_5 = 5,
    native_profile_counts_what_the_interpreter_counts_6 = 6,
    native_profile_counts_what_the_interpreter_counts_7 = 7,
}
