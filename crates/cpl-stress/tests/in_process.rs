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
            .render();
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
    let error = result.trap.expect("the checker's trap");
    assert_eq!(error.at(&["code"]).as_str(), Some("R0101"), "{error:?}");
    assert_eq!(error.at(&["start", "line"]).as_i64(), Some(9), "{error:?}");
}

const GUESS: &str = "let n: Int = read()
var lo = 1
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    let answer: Str = ask(\"?\", mid, limit: 10)
    if answer == \"<\": hi = mid
    else: lo = mid + 1
ask(\"!\", lo)

@interactor
fn judge(mut sol: Interactor):
    let n: 1..=1000 = read()
    let secret: 1..=n = read()
    sol.send(n)
    while true:
        let kind: Str = sol.recv()
        let x: Int = sol.recv()
        if kind == \"!\":
            assert x == secret
            return
        sol.send(if secret <= x: \"<\" else: \">=\")
";

fn interactive(cases: usize) -> Options {
    Options {
        cases,
        size: 50,
        jobs: 1,
        ..Options::default()
    }
}

/// A file with only an `@interactor`, no brute, is judged by it
/// (`stress/U22`), on this thread as in the browser and on its own.
#[test]
fn an_interactive_solution_is_judged_by_its_interactor_in_process() {
    for stack in [Some(1 << 26), None] {
        let result = stress_source("g.cp", GUESS, &interactive(100), stack, false).unwrap();
        assert_eq!(result.report.failure, None, "{}", result.render());
        assert_eq!(result.report.cases, 100);
    }
}

/// A wrong guess is a WA the interactor names, with what each side said.
#[test]
fn an_interactive_failure_has_its_transcript_and_the_interactors_words() {
    let wrong = GUESS.replace("hi = mid\n", "hi = mid - 1\n");
    for stack in [Some(1 << 26), None] {
        let result = stress_source("g.cp", &wrong, &interactive(100), stack, false).unwrap();
        let json = cpl_json::parse(&result.render()).unwrap();
        let case = json.at(&["counterexample"]);
        assert_eq!(case.at(&["verdict"]).as_str(), Some("WA"), "{json:?}");
        let interaction = case.at(&["interaction"]);
        assert_eq!(
            interaction.at(&["interactor", "verdict"]).as_str(),
            Some("rejected")
        );
        let lines = interaction.at(&["transcript"]).as_array().unwrap();
        let first = &lines[0];
        assert_eq!(first.at(&["from"]).as_str(), Some("interactor"));
        let n = case
            .at(&["raw_input"])
            .as_str()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        assert_eq!(first.at(&["line"]).as_str(), Some(n));
        let last = lines.last().unwrap();
        assert_eq!(last.at(&["from"]).as_str(), Some("solution"));
        assert!(
            last.at(&["line"]).as_str().unwrap().starts_with("! "),
            "{json:?}"
        );
        assert_eq!(interaction.at(&["truncated"]).as_i64(), Some(0));
    }
}

/// An interactor that fails itself is blamed, with its own trap.
#[test]
fn an_interactor_that_traps_is_an_interactor_re_with_its_trap() {
    let broken = GUESS.replace(
        "    sol.send(n)\n",
        "    let a: [n]Int = 0\n    sol.send(a[n])\n",
    );
    let result = stress_source("g.cp", &broken, &interactive(5), Some(1 << 26), false).unwrap();
    let failure = result
        .report
        .failure
        .as_ref()
        .expect("the interactor fails");
    assert_eq!(failure.verdict.code(), "interactor RE", "{failure:?}");
    let error = result.trap.as_ref().expect("the interactor's trap");
    assert_eq!(error.at(&["code"]).as_str(), Some("R0101"), "{error:?}");
    assert_eq!(error.at(&["start", "line"]).as_i64(), Some(16), "{error:?}");
}

/// Under `--explain`, an answer the interactor rejects is explained as
/// `cpl stress` explains it: the `print` that gave it and the lines behind
/// it (`cpl_stress::judge::explaining` makes the choice for both).
#[test]
fn an_interactive_failure_is_explained_by_the_answer_it_printed() {
    let source = include_str!("../../../bench/stress/guess-unread-verdict.cp");
    let options = Options {
        cases: 30,
        size: 8,
        seed: 1,
        jobs: 1,
        limit: std::time::Duration::from_millis(1000),
        ..Options::default()
    };
    let result = stress_source("g.cp", source, &options, Some(1 << 26), true).unwrap();
    let json = cpl_json::parse(&result.render()).unwrap();
    let divergence = json.at(&["counterexample", "explain", "divergence"]);
    assert_eq!(
        divergence.at(&["name"]).as_str(),
        Some("answer"),
        "{json:?}"
    );
    assert_eq!(
        divergence.at(&["fast", "line"]).as_i64(),
        Some(14),
        "{json:?}"
    );
    let blame = divergence.at(&["blame"]).as_array().unwrap();
    assert!(
        blame.iter().any(|b| b.at(&["line"]).as_i64() == Some(7)),
        "{json:?}"
    );
}

/// Against `@alt(f)` parts, `--explain` names the innermost wrong call of
/// a part, as `cpl stress` does (`cpl_stress::calls::first_call_divergence`).
#[test]
fn a_parts_wrong_call_is_explained_as_the_cli_explains_it() {
    let cases = [
        (
            include_str!("../../cpl-cli/tests/debugger/part_planted.cp"),
            "tri(k: 1)",
            1,
        ),
        (
            include_str!("../../cpl-cli/tests/debugger/part_recursive.cp"),
            "fib(k: 3)",
            2,
        ),
    ];
    for (source, name, nth) in cases {
        let options = Options {
            jobs: 1,
            ..Options::default()
        };
        let result = stress_source("p.cp", source, &options, Some(1 << 26), true).unwrap();
        let json = cpl_json::parse(&result.render()).unwrap();
        let divergence = json.at(&["counterexample", "explain", "divergence"]);
        assert_eq!(divergence.at(&["name"]).as_str(), Some(name), "{json:?}");
        assert_eq!(divergence.at(&["call"]).as_bool(), Some(true), "{json:?}");
        assert_eq!(divergence.at(&["nth"]).as_i64(), Some(nth), "{json:?}");
    }
}

#[test]
fn a_part_brute_that_traps_is_reported_at_its_trap() {
    let source = "let n: 1..=9 = read()\n\nfn tri(k: Int) -> Int = k * (k + 1) / 2\n\n@alt(tri)\nfn tri_slow(k: Int) -> Int = [0][k]\n\nprint(tri(n))\n";
    let options = Options {
        jobs: 1,
        ..Options::default()
    };
    let result = stress_source("p.cp", source, &options, Some(1 << 26), false).unwrap();
    let json = cpl_json::parse(&result.render()).unwrap();
    let counterexample = json.at(&["counterexample"]);
    assert_eq!(
        counterexample.at(&["verdict"]).as_str(),
        Some("brute RE"),
        "{json:?}"
    );
    let error = counterexample.at(&["error"]);
    assert!(error.at(&["code"]).as_str().is_some(), "{json:?}");
}
