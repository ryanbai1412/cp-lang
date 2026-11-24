//! The in-process engine (`cpl-wasm`'s stress, on the calling thread).

use cpl_stress::run::Options;
use cpl_stress::stress_source;

const WORKS_OUT_THEN_READS: &str = "var sq: [0..=100000]Int = 0
for i in 0..=100000:
    sq[i] = i * i
let n: 0..=100000 = read()
print(if n == 7: 0 else: sq[n])

@alt(program)
fn slow():
    let n: 0..=100000 = read()
    print(n * n)
";

/// What a program works out before it reads runs once; every later case
/// starts there, and the cases fail as before.
#[test]
fn a_case_started_past_the_precompute_fails_as_a_whole_run_does() {
    let options = Options {
        cases: 300,
        size: 20,
        jobs: 1,
        ..Options::default()
    };
    let run = |stack| stress_source("p.cp", WORKS_OUT_THEN_READS, &options, stack, false).unwrap();
    let (resumed, whole) = (run(Some(1 << 26)), run(None));
    let failure = resumed.report.failure.as_ref().expect("n = 7 fails");
    assert_eq!(failure.input.trim(), "7");
    assert_eq!(resumed.report.failure, whole.report.failure);
}

/// A runtime error in the report is the diagnostic `cpl check --format json`
/// gives, as in `cpl stress --format json` (`stress/U23`).
#[test]
fn a_runtime_error_in_the_in_process_report_is_a_diagnostic() {
    let source =
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\nprint(a * b)\n\n\
                  @gen\nfn at_bounds():\n    print(9223372037)\n    print(1000000000)\n";
    let options = Options {
        cases: 1,
        jobs: 1,
        ..Options::default()
    };
    for stack in [Some(1 << 26), None] {
        let json = stress_source("o.cp", source, &options, stack, false)
            .unwrap()
            .to_json();
        let report = cpl_json::parse(&json).unwrap();
        let error = report
            .get("counterexample")
            .and_then(|c| c.get("error"))
            .unwrap_or_else(|| panic!("no error in {json}"));
        let code = error.get("code").and_then(|c| c.as_str());
        assert_eq!(code, Some("R0202"), "{json}");
        let line = error.get("start").and_then(|s| s.get("line"));
        assert_eq!(line.and_then(|l| l.as_i64()), Some(3), "{json}");
    }
}

/// A checker that traps is blamed, not the solution (`stress/U30`).
#[test]
fn a_trapping_checker_in_process_is_a_checker_re() {
    let source = "let n: 1..=5 = read()\nprint(n)\n\n@check\nfn valid():\n    let n: 1..=5 = read()\n    let out: Int = read_output()\n    let a: [n]Int = 0\n    assert a[out] == 0\n";
    let options = Options {
        cases: 5,
        jobs: 1,
        ..Options::default()
    };
    let result = stress_source("c.cp", source, &options, Some(1 << 26), false).unwrap();
    let failure = result.report.failure.expect("the checker fails");
    assert_eq!(failure.verdict.code(), "checker RE", "{failure:?}");
    let error = result.error.expect("the checker's trap");
    assert!(error.contains("\"code\":\"R0101\""), "{error}");
    assert!(error.contains("\"line\":9"), "{error}");
}
