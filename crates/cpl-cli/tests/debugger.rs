//! The debugger end to end, through the `cpl` binary: `debug(...)`,
//! `cpl run --trace`, and `cpl stress` with a file's own `@brute`,
//! `@checker` and `@interactor` under `@!multitest`.
//!
//! The programs in `tests/debugger/` are Codeforces Round 1132 solutions
//! from `research/archive/blind-evals/blind-contest-sim-2.md`, two of them with a bug planted;
//! `sum_planted.cp` is a sum with a bug planted for `--explain`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn cpl(args: &[&OsStr], input: &[u8]) -> Output {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl starts");
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().expect("cpl exits")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A copy of `tests/debugger/<name>` in a directory of its own, so what
/// `cpl stress` saves next to it is this test's.
fn program(test: &str, name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/debugger")
        .join(name);
    std::fs::copy(from, &path).unwrap();
    path
}

fn scratch(test: &str, name: &str, source: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    path
}

const MAX: &str = "\
let n: 1..=10 = read()
let a: [0..n]1..=100 = read()
var best = 0
for i in 0..n:
    best max= a[i]
    debug(i, best)
print(best)
";

#[test]
fn debug_shows_on_a_run_and_is_not_in_the_cpp() {
    let path = scratch("debug-strip", "max.cp", MAX);
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"3\n1 5 2\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"5\n");
    assert_eq!(
        text(&out.stderr),
        "[line 6] i = 0, best = 1\n[line 6] i = 1, best = 5\n[line 6] i = 2, best = 5\n"
    );
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    // Below the header that quotes the source, nothing of it is left.
    let cpp = text(&out.stdout);
    let code: String = cpp.lines().filter(|line| !line.starts_with("//")).collect();
    assert!(!code.contains("debug") && !code.contains("cerr"), "{code}");
}

#[test]
fn run_format_json_is_what_the_editor_overlays() {
    let path = scratch("run-json", "max.cp", MAX);
    let out = cpl(
        &[
            "run".as_ref(),
            path.as_os_str(),
            "--format".as_ref(),
            "json".as_ref(),
        ],
        b"3\n1 5 2\n",
    );
    assert_eq!(out.status.code(), Some(0));
    let report = text(&out.stdout);
    assert!(
        report.starts_with(
            "{\"values\":[{\"line\":1,\"name\":\"n\",\"value\":\"3\"},{\"line\":2,\"name\":\"a\",\"value\":\"[1, 5, 2]\"},{\"line\":3,\"name\":\"best\",\"value\":\"5\"}],\"stdout\":\"5\\n\",\"time_ms\":"
        ),
        "{report}"
    );
    // A failed run says where, in the same report.
    let out = cpl(
        &[
            "run".as_ref(),
            path.as_os_str(),
            "--format".as_ref(),
            "json".as_ref(),
        ],
        b"3\n1 500 2\n",
    );
    assert_eq!(out.status.code(), Some(3));
    assert!(
        text(&out.stdout).contains("\"error\":{\"line\":2,\"message\":\""),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn run_trace_writes_the_schema_as_json_and_as_one_page() {
    let path = scratch("run-trace", "max.cp", MAX);
    let json = cpl(
        &[
            "run".as_ref(),
            path.as_os_str(),
            "--trace".as_ref(),
            "--format".as_ref(),
            "json".as_ref(),
        ],
        b"3\n1 5 2\n",
    );
    assert_eq!(json.status.code(), Some(0));
    let trace = text(&json.stdout);
    assert!(
        trace.starts_with("{\"schemaVersion\":1,\"title\":\"max.cp\","),
        "{trace}"
    );
    assert!(
        trace.contains("\"a\":{\"name\":\"a\",\"type\":\"[Int]Int\",\"kind\":\"array\"}"),
        "{trace}"
    );
    assert!(
        trace.contains("\"best\":{\"name\":\"best\",\"type\":\"Int\",\"kind\":\"scalar\"}"),
        "{trace}"
    );
    assert!(
        trace
            .contains("\"runs\":{\"fast\":{\"label\":\"cpl run\",\"steps\":[{\"id\":0,\"line\":1,"),
        "{trace}"
    );
    // Safe inside a <script>: no `<` anywhere.
    assert!(!trace.contains('<'));
    let html = cpl(
        &[
            "run".as_ref(),
            path.as_os_str(),
            "--trace".as_ref(),
            "--format".as_ref(),
            "html".as_ref(),
        ],
        b"3\n1 5 2\n",
    );
    let page = text(&html.stdout);
    assert!(
        page.starts_with("<!doctype html>") || page.starts_with("<!DOCTYPE html>"),
        "{}",
        &page[..60]
    );
    let data = format!(
        "<script id=\"trace-data\" type=\"application/json\">{}</script>",
        trace.trim_end()
    );
    assert!(page.contains(&data), "the page holds the same trace");
    let version = format!(
        "<script id=\"trace-version\" type=\"application/json\">\"{}\"</script>",
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        page.contains(&version),
        "the page names the cpl that wrote it"
    );
    // Self-contained: nothing is fetched.
    assert!(!page.contains("src=\"http") && !page.contains("href=\"http"));
}

#[test]
fn stress_finds_the_planted_bug_with_the_files_own_brute_and_traces_both() {
    // CF 2284 A2, sorted the wrong way round.
    let path = program("stress-brute", "cf2284a2_planted.cp");
    let out = cpl(
        &[
            "stress".as_ref(),
            path.as_os_str(),
            "--cases".as_ref(),
            "200".as_ref(),
            "--trace".as_ref(),
            "--format".as_ref(),
            "json".as_ref(),
        ],
        b"",
    );
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}{}", text(&out.stderr));
    assert!(report.starts_with("{\"status\":\"fail\""), "{report}");
    // One test case under the `@!multitest` header, shrunk.
    assert!(
        report.contains("\"raw_input\":\"1\\n3\\n0 0 1\\n\""),
        "{report}"
    );
    assert!(
        report.contains("\"expected\":\"4\\n\",\"got\":\"3\\n\""),
        "{report}"
    );
    // The `debug(\"piles\", ...)` both sides ran, with what each showed.
    assert!(
        report.contains("\"stages\":[{\"name\":\"case 1 · debug piles\",\"line\":17,\"fast\":\"3, 1\",\"brute\":\"4, 4\"}]"),
        "{report}"
    );
    let page =
        std::fs::read_to_string(path.with_extension("tests").join("stress-1.trace.html")).unwrap();
    assert!(
        page.contains("\"runs\":{\"fast\":{\"label\":\"solution\""),
        "the solution's run"
    );
    assert!(
        page.contains("\"slow\":{\"label\":\"slow\""),
        "the brute's run"
    );
    assert!(page.contains("\"checkpoint\":\"case 1 · debug piles\""));
}

#[test]
fn stress_explain_names_the_first_call_of_a_part_its_alt_answers_differently() {
    let path = program("stress-explain-part", "part_planted.cp");
    let out = cpl(
        &["stress".as_ref(), path.as_os_str(), "--explain".as_ref()],
        b"",
    );
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}{}", text(&out.stderr));
    assert!(
        report.contains("first differing value: `tri(k: 1)`, its 1st value"),
        "{report}"
    );
    assert!(
        report.contains("solution tri(k: 1) = 0 at line 4"),
        "{report}"
    );
    assert!(
        report.contains("brute    tri(k: 1) = 1 at line 10"),
        "{report}"
    );
    assert!(
        report.contains("the solution's last steps in it:\n      line 5: k: 1, s: 0"),
        "{report}"
    );
}

#[test]
fn stress_explain_names_the_innermost_wrong_call_of_a_recursive_part() {
    let path = program("stress-explain-recursive", "part_recursive.cp");
    let out = cpl(
        &["stress".as_ref(), path.as_os_str(), "--explain".as_ref()],
        b"",
    );
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}{}", text(&out.stderr));
    assert!(
        report.contains("first differing value: `fib(k: 3)`"),
        "{report}"
    );
    assert!(
        report.contains("solution fib(k: 3) = 1 at line 4"),
        "{report}"
    );
    assert!(
        report.contains("brute    fib(k: 3) = 2 at line 10"),
        "{report}"
    );
}

#[test]
fn stress_explain_names_the_first_value_the_solution_and_brute_give_differently() {
    let path = program("stress-explain", "sum_planted.cp");
    let args = |json: bool| {
        let mut args: Vec<&OsStr> = vec!["stress".as_ref(), path.as_os_str(), "--explain".as_ref()];
        if json {
            args.extend([OsStr::new("--format"), OsStr::new("json")]);
        }
        args
    };
    let out = cpl(&args(false), b"");
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}{}", text(&out.stderr));
    assert!(report.contains("that is: n = 1  a = [2]"), "{report}");
    assert!(
        report.contains("first differing value: `s`, its 2nd value"),
        "{report}"
    );
    assert!(report.contains("solution s = 4 at line 6"), "{report}");
    assert!(report.contains("brute    s = 2 at line 12"), "{report}");
    let out = cpl(&args(true), b"");
    let report = text(&out.stdout);
    assert!(
        report.contains(
            "\"explain\":{\"divergence\":{\"name\":\"s\",\"nth\":2,\"fast\":{\"line\":6,\"value\":\"4\"},\"brute\":{\"line\":12,\"value\":\"2\"}}}"
        ),
        "{report}"
    );
    // Without the flag, nothing is traced and the report has no `explain`.
    let out = cpl(
        &[
            "stress".as_ref(),
            path.as_os_str(),
            "--format".as_ref(),
            "json".as_ref(),
        ],
        b"",
    );
    assert!(
        !text(&out.stdout).contains("\"explain\""),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn stress_judges_a_constructive_multitest_with_the_files_own_checker() {
    // CF 2284 B: any valid arrangement is accepted.
    let good = program("stress-checker-ok", "cf2284b.cp");
    let out = cpl(
        &[
            "stress".as_ref(),
            good.as_os_str(),
            "--cases".as_ref(),
            "150".as_ref(),
        ],
        b"",
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(text(&out.stdout).contains("150 cases agree"));
    // One donation dropped from the answer: `read_output()` runs out.
    let bad = program("stress-checker-bad", "cf2284b_planted.cp");
    let out = cpl(
        &[
            "stress".as_ref(),
            bad.as_os_str(),
            "--cases".as_ref(),
            "150".as_ref(),
        ],
        b"",
    );
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}");
    assert!(
        report.contains("that is: n = 2  k = 2  a = [1, 2]"),
        "{report}"
    );
    assert!(report.contains("checker: "), "{report}");
}

#[test]
fn an_interactive_multitest_counts_queries_per_test_case() {
    let path = program("interactive", "guess_multitest.cp");
    // Four test cases of up to 7 queries each: 25 lines in all, well
    // over the limit of one.
    let out = cpl(
        &["run".as_ref(), path.as_os_str()],
        b"4\n100 37\n100 100\n64 1\n5 3\n",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains("accepted the run (25 lines from the solution)"),
        "{}",
        text(&out.stderr)
    );
    // The C++ for the judge starts each test case at zero too.
    let out = cpl(
        &[
            "build".as_ref(),
            path.as_os_str(),
            "-o".as_ref(),
            "-".as_ref(),
        ],
        b"",
    );
    assert!(text(&out.stdout).contains("{ cpl::asked = 0; prog::solve(); }"));
    // `cpl stress` makes the hidden tests from what the interactor reads.
    let out = cpl(
        &[
            "stress".as_ref(),
            path.as_os_str(),
            "--cases".as_ref(),
            "40".as_ref(),
        ],
        b"",
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    // A binary search that skips the answer is caught and shrunk.
    // A Windows checkout may have CRLF lines.
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n")
        .replace("hi = mid\n", "hi = mid - 1\n");
    let bad = scratch("interactive-bad", "guess.cp", &source);
    let out = cpl(
        &[
            "stress".as_ref(),
            bad.as_os_str(),
            "--cases".as_ref(),
            "60".as_ref(),
        ],
        b"",
    );
    let report = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{report}");
    assert!(
        report.contains("interactor: assertion failed: v == x: wrong answer"),
        "{report}"
    );
}

#[test]
fn check_says_what_run_and_build_would() {
    // `1 - z` on a `Mod<P>` works on both sides of the operator.
    let path = scratch(
        "mod-left",
        "m.cp",
        "const P = 998244353\ntype Z = Mod<P>\nlet z: Z = Z(3)\nprint(1 - z, 2 * z, z - 1, 10 + z)\n",
    );
    let out = cpl(&["run".as_ref(), path.as_os_str()], b"");
    assert_eq!(
        text(&out.stdout),
        "998244351 6 2 13\n",
        "{}",
        text(&out.stderr)
    );
    // What only lowering knew is now an error in `cpl check` as well.
    for (source, message) in [
        ("let n: 1..=9 = read()\ndomain V = 1..=n\nprint(V.from(2))\n", "has no method `from`"),
        ("let n: 1..=9 = read()\ndomain V = 1..=n\nprint(V(2)!)\n", "`!` takes the value out of an optional"),
        (
            "let n: 1..=9 = read()\nprint(n)\n\n@interactor fn judge(sol: Interactor):\n    sol.send(m)\n",
            "no `m` in scope",
        ),
    ] {
        let path = scratch("check-gaps", "g.cp", source);
        let out = cpl(&["check".as_ref(), path.as_os_str()], b"");
        assert_eq!(out.status.code(), Some(1), "{source}");
        assert!(text(&out.stderr).contains(message) || text(&out.stdout).contains(message), "{source}\n{}", text(&out.stderr));
    }
}

/// Milestone 1's loop on one program: WA, `cpl stress` against the file's
/// own brute, checker or interactor, the shrunk case saved, the fix, then
/// `cpl test` and `cpl stress` both pass.
fn stress_loop(test: &str, planted: &str, fixed: &str, saved: &[&str]) -> String {
    assert_ne!(fixed, planted);
    let path = scratch(test, "p.cp", planted);
    let run = |args: &[&OsStr]| {
        let out = cpl(args, b"");
        (out.status.code(), text(&out.stdout) + &text(&out.stderr))
    };
    let stress = [
        "stress".as_ref(),
        path.as_os_str(),
        "--cases".as_ref(),
        "200".as_ref(),
    ];
    let rerun = ["test".as_ref(), path.as_os_str()];

    let (code, failed) = run(&stress);
    assert_eq!(code, Some(1), "{failed}");
    let tests = path.with_file_name("p.tests");
    let mut cases: Vec<_> = std::fs::read_dir(&tests)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    cases.sort();
    assert_eq!(cases, saved, "{failed}");
    let (code, report) = run(&rerun);
    assert_eq!(code, Some(1), "{report}");
    assert!(report.contains("0/1 passed"), "{report}");

    std::fs::write(&path, fixed).unwrap();
    let (code, report) = run(&rerun);
    assert_eq!(code, Some(0), "{report}");
    assert!(report.contains("1/1 passed"), "{report}");
    let (code, report) = run(&stress);
    assert_eq!(code, Some(0), "{report}");
    assert!(report.contains("200 cases agree"), "{report}");
    failed
}

fn source(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn a_wrong_answer_goes_from_stress_to_a_saved_case_to_a_green_rerun() {
    let planted = source("../../examples/debugging/knapsack.cp");
    let fixed = planted.replace("for c in 0..=cap:", "for c in (0..=cap).rev:");
    stress_loop(
        "loop-alt",
        &planted,
        &fixed,
        &["stress-3.in", "stress-3.out"],
    );
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("loop-alt/p.tests/stress-3.out");
    assert_eq!(std::fs::read_to_string(path).unwrap().trim(), "1");
}

#[test]
fn the_loop_reruns_a_checked_multitest_through_the_files_checker() {
    let planted = source("tests/debugger/cf2284b_planted.cp");
    let fixed = source("tests/debugger/cf2284b.cp");
    let failed = stress_loop("loop-check", &planted, &fixed, &["stress-1.in"]);
    assert!(
        failed.contains("checker: ") && failed.contains("p.cp (`check`)"),
        "{failed}"
    );
}

#[test]
fn the_loop_reruns_an_interactive_multitest_through_the_files_interactor() {
    let fixed = source("tests/debugger/guess_multitest.cp");
    let planted = fixed.replace("else: lo = mid + 1", "else: lo = mid");
    let failed = stress_loop("loop-interactor", &planted, &fixed, &["stress-1.in"]);
    assert!(
        failed.contains("interactor: ") && failed.contains("p.cp (`judge`)"),
        "{failed}"
    );
}

#[test]
fn with_no_cpp_compiler_the_loop_runs_in_the_interpreter() {
    // A machine without `g++` on its PATH, as a fresh Windows one often is.
    let planted = source("../../examples/debugging/knapsack.cp");
    let path = scratch("loop-no-cxx", "p.cp", &planted);
    let empty = path.with_file_name("bin");
    std::fs::create_dir_all(&empty).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "stress".as_ref(),
            path.as_os_str(),
            "--cases".as_ref(),
            "200".as_ref(),
        ])
        .env("PATH", &empty)
        .output()
        .expect("cpl runs");
    let report = text(&out.stdout) + &text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{report}");
    assert!(report.contains("expected `1`, got `2`"), "{report}");
    assert!(
        path.with_file_name("p.tests/stress-3.in").is_file(),
        "{report}"
    );
}
