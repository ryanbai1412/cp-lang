//! The wrong-answer workflow end to end, through the `cpl` binary, as
//! `handbook/product/workflows/wrong-answer.md` walks it: stress with no
//! brute finds a crash, stress with a brute finds a wrong answer, each
//! report names the next commands, the trace records the saved case, and
//! after each fix `cpl test` and `cpl stress` are green. Use cases
//! `stress/U1`, `stress/U4`, `stress/U13` and `stress/U14` in
//! `handbook/product/cli/stress-usecases.md`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cpl(args: &[&OsStr]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .output()
        .expect("cpl runs");
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text)
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("c.cp")
}

const READS: &str = "let n: 1..=10 = read()\nlet a: [n]0..=9 = read()\n";

const CRASHES: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i + 1])\nprint(best)\n";
const SKIPS_LAST: &str =
    "var best = 0\nfor i in 0..n - 1:\n    best = max(best, a[i])\nprint(best)\n";
const RIGHT: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i])\nprint(best)\n";

const BRUTE: &str = "
@alt(program)
fn brute():
    let n: 1..=10 = read()
    let a: [n]0..=9 = read()
    var m = 0
    for x in a: m = max(m, x)
    print(m)
";

fn saved(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path.with_file_name("c.tests"))
        .map(|dir| {
            dir.map(|e| e.unwrap().file_name().into_string().unwrap())
                .filter(|n| n.ends_with(".in"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_wrong_answer_goes_from_a_crash_and_a_counterexample_to_a_clean_stress() {
    let path = scratch("wrong-answer");
    let file = path.as_os_str();
    let stress = ["stress".as_ref(), file, "--cases".as_ref(), "200".as_ref()];
    let test = ["test".as_ref(), file];

    // No brute yet: a case fails only on a crash or the time limit.
    std::fs::write(&path, format!("{READS}{CRASHES}")).unwrap();
    let (code, report) = cpl(&stress);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "only runtime errors and the time limit fail a case",
        ": RE",
        "outside an array",
        "locals: ",
        "next: `cpl run ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 1, "{report}");

    // The trace records the case stress saved.
    let (_, traced) = cpl(&["run".as_ref(), file, "--trace".as_ref()]);
    assert!(traced.contains("the case saved last"), "{traced}");
    assert!(path.with_file_name("c.trace.html").is_file(), "{traced}");

    // Fixed: the saved case passes and stress runs clean.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(
        clean.contains("200 cases ran with no runtime error"),
        "{clean}"
    );

    // With a brute, the wrong answer that was left is found and explained.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}{BRUTE}")).unwrap();
    let mut explained = stress.to_vec();
    explained.push("--explain".as_ref());
    let (code, report) = cpl(&explained);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "that is: n = 1",
        "expected:",
        "look at line ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 2, "{report}");
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(1), "{rerun}");

    // Fixed again: every saved case passes and the brute agrees.
    std::fs::write(&path, format!("{READS}{RIGHT}{BRUTE}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    assert!(rerun.contains("2/2 passed"), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(clean.contains("200 cases agree"), "{clean}");
}

/// Off the happy path: the brute is the one that is wrong (`stress/U18`).
#[test]
fn a_brute_that_traps_is_blamed_with_its_line_and_locals_and_not_saved() {
    let path = scratch("brute-traps");
    let source = format!(
        "{READS}{RIGHT}
@alt(program)
fn brute():
    let n: 1..=10 = read()
    let a: [n]0..=9 = read()
    var m = 0
    for i in 0..n: m = max(m, a[i + 1])
    print(m)
"
    );
    std::fs::write(&path, source).unwrap();
    let (code, report) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "brute RE",
        "that is: n = 1",
        "failed, so there is nothing to compare with",
        "error[R0101]",
        "c.cp:13:",
        "locals: ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert!(!report.contains("Code("), "{report}");
    assert!(!report.contains("could not be saved"), "{report}");
    assert!(saved(&path).is_empty(), "{report}");
}

/// The editor's lens reads a runtime error as `cpl check --format json` gives it (`stress/U23`).
#[test]
fn a_runtime_error_in_the_json_report_is_a_diagnostic() {
    let path = scratch("json-trap");
    std::fs::write(&path, format!("{READS}print(a[n])\n")).unwrap();
    let (code, report) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        r#""verdict":"RE","error":{"code":"R0101""#,
        r#""start":{"col":7,"line":3"#,
        r#""locals: n = 1"#,
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
}

/// Stress finds nothing at small sizes, and its report points at the
/// lint that names an input at the bounds that traps (`stress/U28`).
#[test]
fn a_clean_stress_points_at_the_bound_lint_and_its_saved_witness() {
    let file = scratch("bound_lint");
    std::fs::write(
        &file,
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\n\
         if a % 1000 == 37:\n    print(a * b)\n",
    )
    .unwrap();
    let (code, text) = cpl(&[OsStr::new("stress"), file.as_os_str()]);
    assert_eq!(code, Some(0), "{text}");
    assert!(
        text.contains(
            "`cpl lint` names inputs at the bounds that fail (L0306 at line 4): `cpl lint --save"
        ),
        "{text}"
    );
    let (_, saved) = cpl(&[OsStr::new("lint"), OsStr::new("--save"), file.as_os_str()]);
    let witness = file.with_extension("tests").join("lint-L0306-4.in");
    let input = std::fs::read(&witness).unwrap_or_else(|e| panic!("{saved}: {e}"));
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&file)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(&input)?;
            child.wait_with_output()
        })
        .unwrap();
    let said = String::from_utf8_lossy(&ran.stderr);
    assert!(said.contains("error[R0202]: integer overflow"), "{said}");
}

/// A swapped-in `@brute(f)` fn that traps is blamed as an `@alt(program)`
/// brute is: its error, line and locals, in text and in JSON (`stress/U18`).
#[test]
fn a_swapped_in_brute_fn_that_traps_is_blamed_with_its_line_and_locals() {
    let path = scratch("part-brute-traps");
    let source = format!(
        "{READS}fn best() -> Int:
    var m = 0
    for i in 0..n: m = max(m, a[i])
    m

@brute(best)
fn best_slow() -> Int:
    var m = 0
    for i in 0..n: m = max(m, a[i + 1])
    m

print(best())
"
    );
    std::fs::write(&path, source).unwrap();
    let (code, report) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_eq!(code, Some(1), "{report}");
    for said in ["brute RE", "error[R0101]", "c.cp:11:", "locals: "] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert!(!report.contains("exit code"), "{report}");
    let (_, json) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert!(json.contains("\"code\":\"R0101\""), "{json}");
}

/// Stress is a checked build: a case that overflows fails as RE with
/// the diagnostic `cpl run` gives, instead of wrapping (`stress/U34`).
#[test]
fn stress_traps_an_overflow_as_cpl_run_does() {
    let file = scratch("overflow");
    std::fs::write(
        &file,
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\nprint(a * b)\n\n\
         @gen\nfn at_bounds():\n    print(9223372037)\n    print(1000000000)\n",
    )
    .unwrap();
    let (code, text) = cpl(&[
        OsStr::new("stress"),
        file.as_os_str(),
        OsStr::new("--cases"),
        OsStr::new("1"),
    ]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("FAIL on case 1 (seed 1): RE"), "{text}");
    assert!(text.contains("error[R0202]: integer overflow"), "{text}");
}

/// A shift past an `Int` traps under stress as under `cpl run`, though
/// C++ would shift it without a word (`stress/U34`).
#[test]
fn stress_traps_a_shift_overflow_as_cpl_run_does() {
    let file = scratch("shift-overflow");
    std::fs::write(
        &file,
        "let k: 0..=63 = read()\nprint(1 << k)\n\n@gen\nfn top():\n    print(63)\n",
    )
    .unwrap();
    let (code, text) = cpl(&[
        OsStr::new("stress"),
        file.as_os_str(),
        OsStr::new("--cases"),
        OsStr::new("1"),
    ]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("FAIL on case 1 (seed 1): RE"), "{text}");
    assert!(
        text.contains("`1 << 63` does not fit in an `Int`"),
        "{text}"
    );
}
