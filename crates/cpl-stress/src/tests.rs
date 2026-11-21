use super::*;
use crate::judge::{declared, first_difference};
use crate::run::{save, stress, Options, Source};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

const SRC: &str = "\
const K = 3
let (n, q): (1..=2e5, 1..=K) = read()
domain V = 1..=n
let a: [V]0..=1e9 = read()
let queries: [0..q](V, V) = read()
print(a.max)
";

fn spec() -> InputSpec {
    InputSpec::from_source(SRC).unwrap()
}

/// A program written in shell: `sh -c <script>`.
fn sh(script: &str) -> Program {
    Program::command("script", &["sh", "-c", script])
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-stress-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The largest number on the second line; wrong (prints 0) when it is
/// above `limit`.
fn max_below(limit: u32) -> Program {
    sh(&format!(
        "awk 'NR==2 {{ m=$1; for (i=2;i<=NF;i++) if ($i>m) m=$i; if (m>{limit}) m=0; print m }}'"
    ))
}

#[test]
fn crate_is_named() {
    assert_eq!(NAME, "cpl-stress");
}

#[test]
fn the_spec_is_read_from_the_types_of_read() {
    let spec = spec();
    assert_eq!(spec.fields.len(), 3);
    assert_eq!(
        spec.fields[0].names,
        [Some("n".to_string()), Some("q".to_string())]
    );
    assert_eq!(spec.fields[1].names, [Some("a".to_string())]);
    assert_eq!((spec.fields[0].line, spec.fields[2].line), (2, 5));
    assert_eq!(spec.consts["K"], 3);
    // `n` and `q` decide how much input there is; `K` is a constant.
    let mut sizes: Vec<&str> = spec.sizes.iter().map(String::as_str).collect();
    sizes.sort();
    assert_eq!(sizes, ["n", "q"]);
}

#[test]
fn types_the_generator_cannot_make_are_reported_where_they_are() {
    let src = "let n: 1..=5 = read()\nlet x: Set<Int> = read()\n";
    let error = InputSpec::from_source(src).unwrap_err();
    assert_eq!(error.span.text(src), "Set<Int>");
    assert_eq!(
        error.message,
        "can't generate a `Set<Int>` yet; pass a generator with `--gen`"
    );
    let untyped = InputSpec::from_source("let n = read()\n").unwrap_err();
    assert!(
        untyped.message.starts_with("this `read()` has no type"),
        "{}",
        untyped.message
    );
    let unknown = InputSpec::from_source("let a: [W]Int = read()\n").unwrap_err();
    assert_eq!(unknown.message, "`W` is not a domain declared above");
}

#[test]
fn cases_are_valid_small_and_the_same_for_the_same_seed() {
    let spec = spec();
    for seed in 0..200 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        assert_eq!(
            Some(&case),
            generate(&spec, &mut Rng::new(seed), Size::Small(8)).as_ref()
        );
        // Fitting a valid case changes nothing.
        assert_eq!(fit(&spec, &case), Some(case.clone()), "seed {seed}");
        let [Value::Tuple(head), Value::Seq(a), Value::Seq(queries)] = case.values.as_slice()
        else {
            panic!("{case:?}");
        };
        let [Value::Int(n), Value::Int(q)] = head.as_slice() else {
            panic!("{head:?}");
        };
        assert!(
            (1..=8).contains(n) && (1..=3).contains(q),
            "n = {n}, q = {q}"
        );
        assert_eq!((a.len() as i128, queries.len() as i128), (*n, *q));
        for value in a {
            assert!(matches!(value, Value::Int(v) if (0..=1_000_000_000).contains(v)));
        }
        for query in queries {
            let Value::Tuple(ends) = query else { panic!() };
            assert!(ends
                .iter()
                .all(|end| matches!(end, Value::Int(v) if (1..=*n).contains(v))));
        }
    }
}

#[test]
fn full_size_cases_use_the_whole_range() {
    let spec = InputSpec::from_source("let n: 1..=2e5 = read()\n").unwrap();
    let largest = (0..200)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Full))
        .map(|case| case.weight().1)
        .max();
    assert_eq!(largest, Some(200_000));
}

#[test]
fn an_empty_range_makes_no_case() {
    let spec =
        InputSpec::from_source("let n: 1..=1 = read()\nlet k: 1..=n - 1 = read()\n").unwrap();
    assert_eq!(generate(&spec, &mut Rng::new(1), Size::Small(8)), None);
}

#[test]
fn a_case_renders_one_row_per_line_and_describes_itself_by_name() {
    let case = Case::of(vec![
        Value::Tuple(vec![Value::Int(3), Value::Int(2)]),
        Value::Seq(vec![Value::Int(1), Value::Int(5), Value::Int(2)]),
        Value::Seq(vec![
            Value::Tuple(vec![Value::Int(1), Value::Int(3)]),
            Value::Tuple(vec![Value::Int(2), Value::Int(2)]),
        ]),
    ]);
    assert_eq!(case.render(), "3 2\n1 5 2\n1 3\n2 2\n");
    assert_eq!(
        case.describe(&spec()),
        "n = 3  q = 2  a = [1, 5, 2]  queries = [(1, 3), (2, 2)]"
    );
    assert_eq!(case.weight(), (9, 21));
}

#[test]
fn grids_are_rows_of_columns() {
    let spec = InputSpec::from_source(
        "let (h, w): (2..=2, 3..=3) = read()\ndomain R = 0..h\ndomain C = 0..w\nlet g: [R, C]0..=0 = read()\n",
    )
    .unwrap();
    let case = generate(&spec, &mut Rng::new(7), Size::Small(8)).unwrap();
    assert_eq!(case.render(), "2 3\n0 0 0\n0 0 0\n");
}

#[test]
fn fitting_cuts_arrays_and_clamps_values_to_the_new_size() {
    let spec = spec();
    let mut case = Case::of(vec![
        Value::Tuple(vec![Value::Int(2), Value::Int(1)]),
        Value::Seq(vec![Value::Int(1), Value::Int(5), Value::Int(2)]),
        Value::Seq(vec![
            Value::Tuple(vec![Value::Int(1), Value::Int(3)]),
            Value::Tuple(vec![Value::Int(2), Value::Int(2)]),
        ]),
    ]);
    // `n` was lowered from 3 to 2 and `q` from 2 to 1.
    assert_eq!(fit(&spec, &case).unwrap().render(), "2 1\n1 5\n1 2\n");
    // An array cannot grow.
    case.values[0] = Value::Tuple(vec![Value::Int(4), Value::Int(1)]);
    assert_eq!(fit(&spec, &case), None);
}

#[test]
fn shrinking_finds_the_smallest_case_that_still_fails() {
    let spec = spec();
    // "Fails" when some element of `a` is at least 100.
    let mut runs = 0;
    let mut fails = |case: &Case| {
        runs += 1;
        matches!(&case.values[1], Value::Seq(a) if a.iter().any(|v| matches!(v, Value::Int(v) if *v >= 100)))
    };
    let start = (0..500)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| case.weight().0 >= 12 && fails(case))
        .expect("a large failing case");
    let smallest = shrink(&spec, &start, 2000, &mut fails);
    // One element, exactly at the threshold, and one query at its lowest.
    assert_eq!(smallest.render(), "1 1\n100\n1 1\n");
    assert!(runs < 400, "{runs} runs");
}

#[test]
fn shrinking_deletes_elements_from_the_middle() {
    let spec = InputSpec::from_source(
        "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]0..=9 = read()\n",
    )
    .unwrap();
    let start = Case::of(vec![
        Value::Int(5),
        Value::Seq([1, 7, 1, 1, 9].map(Value::Int).to_vec()),
    ]);
    // Fails while a 7 comes somewhere before a 9.
    let mut fails = |case: &Case| {
        let Value::Seq(a) = &case.values[1] else {
            return false;
        };
        let seven = a.iter().position(|v| *v == Value::Int(7));
        let nine = a.iter().rposition(|v| *v == Value::Int(9));
        matches!((seven, nine), (Some(s), Some(n)) if s < n)
    };
    assert_eq!(shrink(&spec, &start, 2000, &mut fails).render(), "2\n7 9\n");
}

#[test]
fn shrinking_respects_its_budget_and_never_returns_a_passing_case() {
    let spec = spec();
    let start = generate(&spec, &mut Rng::new(3), Size::Small(8)).unwrap();
    let mut runs = 0;
    let same = shrink(&spec, &start, 5, &mut |_| {
        runs += 1;
        false
    });
    assert_eq!((same, runs), (start, 5));
}

#[test]
fn outputs_are_compared_by_token() {
    assert_eq!(first_difference("1 2\n3\n", "1  2 3\n\n"), None);
    assert_eq!(
        first_difference("1 2\n3\n", "1 2\n4\n").as_deref(),
        Some("first difference at line 2, token 1: expected `3`, got `4`")
    );
    assert_eq!(
        first_difference("1\n", "1\n2\n").as_deref(),
        Some("first difference at line 2, token 1: expected nothing, got `2`")
    );
    assert_eq!(first_difference("", ""), None);
}

#[test]
fn a_run_captures_output_exit_time_and_memory() {
    let outcome = sh("cat; echo oops >&2; exit 3")
        .run("hello\n", Duration::from_secs(10))
        .unwrap();
    assert_eq!(
        (outcome.stdout.as_str(), outcome.stderr.as_str()),
        ("hello\n", "oops\n")
    );
    assert_eq!(outcome.exit, Exit::Code(3));
    assert!(!outcome.ok());
    let slow = sh("sleep 5").run("", Duration::from_millis(50)).unwrap();
    assert_eq!(slow.exit, Exit::TimedOut);
    assert!(slow.elapsed < Duration::from_secs(4));
    if cfg!(target_os = "linux") {
        let measured = sh("sleep 0.05").run("", Duration::from_secs(10)).unwrap();
        assert!(measured.peak_kb.is_some_and(|kb| kb > 0), "{measured:?}");
    }
}

#[test]
fn programs_resolve_by_extension() {
    let dir = scratch("resolve");
    let missing = Program::resolve(&dir.join("nope.cpp")).unwrap_err();
    assert!(
        missing.to_string().starts_with("there is no file"),
        "{missing}"
    );
    std::fs::write(dir.join("a.py"), "print(1)\n").unwrap();
    assert_eq!(
        Program::resolve(&dir.join("a.py")).unwrap().argv[0],
        "python3"
    );
    // A `.cp` file goes through the compiler; its errors are the reason.
    std::fs::write(dir.join("bad.cp"), "print(m)\n").unwrap();
    let error = Program::resolve(&dir.join("bad.cp"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("bad.cp does not build:") && error.contains("no `m` in scope"),
        "{error}"
    );
    if std::process::Command::new("g++")
        .arg("--version")
        .output()
        .is_ok()
    {
        std::fs::write(dir.join("d.cp"), "let n: 1..=9 = read()\nprint(n + 1)\n").unwrap();
        let built = Program::resolve(&dir.join("d.cp")).unwrap();
        let outcome = built.run("4\n", Duration::from_secs(10)).unwrap();
        assert_eq!(
            (outcome.stdout.as_str(), outcome.exit),
            ("5\n", Exit::Code(0))
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_token_judge_names_each_way_of_failing() {
    let limit = Duration::from_secs(10);
    let verdict = |script: &str, expected: Option<&str>| {
        Judge::Tokens
            .run(&sh(script), "", expected, limit)
            .unwrap()
            .verdict
    };
    assert_eq!(verdict("echo 4", Some("4\n")), Verdict::Accepted);
    assert_eq!(verdict("echo 4", None), Verdict::Accepted);
    assert!(
        matches!(verdict("echo 5", Some("4\n")), Verdict::Wrong(why) if why.contains("expected `4`, got `5`"))
    );
    assert_eq!(
        verdict("echo 'index 9 is out of range' >&2; exit 1", Some("4\n")),
        Verdict::RuntimeError("exit code 1: index 9 is out of range".to_string())
    );
    assert_eq!(
        verdict("kill -SEGV $$", None),
        Verdict::RuntimeError("crashed".to_string())
    );
    let slow = Judge::Tokens
        .run(&sh("sleep 5"), "", None, Duration::from_millis(50))
        .unwrap();
    assert_eq!(
        (slow.verdict.code(), slow.verdict),
        ("TLE", Verdict::TimeLimit)
    );
}

#[test]
fn a_checker_gets_the_input_the_output_and_the_answer() {
    // Accepts any output whose first token equals the input's.
    let checker = Judge::Checker(sh(
        "[ \"$(cat \"$1\")\" = \"$(cat \"$0\")\" ] || { echo 'wrong echo' >&2; exit 1; }; [ \"$(cat \"$2\")\" = ans ]",
    ));
    let limit = Duration::from_secs(10);
    let right = checker
        .run(&sh("cat"), "7\n", Some("ans\n"), limit)
        .unwrap();
    assert_eq!(right.verdict, Verdict::Accepted);
    let wrong = checker
        .run(&sh("echo 8"), "7\n", Some("ans\n"), limit)
        .unwrap();
    assert_eq!(
        wrong.verdict,
        Verdict::Wrong("checker: wrong echo".to_string())
    );
    // A crash is a crash whatever the checker would say.
    let crashed = checker
        .run(&sh("exit 2"), "7\n", Some("ans\n"), limit)
        .unwrap();
    assert_eq!(crashed.verdict.code(), "RE");
}

#[test]
fn an_interactor_talks_to_the_solution_and_rules() {
    // The secret is the input; the solution may ask three questions.
    let interactor = Judge::Interactor(sh(
        "secret=$(cat \"$0\"); for turn in 1 2 3; do read guess || exit 1; if [ \"$guess\" = \"$secret\" ]; then echo yes; exit 0; fi; echo no; done; echo 'too many guesses' >&2; exit 1",
    ));
    let limit = Duration::from_secs(10);
    let counts_up =
        sh("for g in 1 2 3 4 5; do echo $g; read answer; [ \"$answer\" = yes ] && exit 0; done");
    let found = interactor.run(&counts_up, "2\n", None, limit).unwrap();
    assert_eq!(found.verdict, Verdict::Accepted);
    let lost = interactor.run(&counts_up, "5\n", None, limit).unwrap();
    assert_eq!(
        lost.verdict,
        Verdict::Wrong("interactor: too many guesses".to_string())
    );
    let silent = interactor
        .run(&sh("sleep 5"), "2\n", None, Duration::from_millis(100))
        .unwrap();
    assert_eq!(silent.verdict, Verdict::TimeLimit);
}

#[test]
fn declared_judges_are_found_with_their_lines() {
    let src = "@checker(output: Int)\nfn valid(out: Int) -> Bool:\n    out > 0\n\n@interactor(Guess)\nfn judge():\n    0\nfn plain() = 1\n";
    let found = declared(src);
    let seen: Vec<(&str, &str, u32)> = found
        .iter()
        .map(|d| (d.kind, d.name.as_str(), d.line))
        .collect();
    assert_eq!(seen, [("checker", "valid", 2), ("interactor", "judge", 6)]);
    assert_eq!(declared(SRC), []);
}

fn options(cases: usize) -> Options {
    Options {
        cases,
        jobs: 4,
        ..Options::default()
    }
}

#[test]
fn stress_agrees_when_the_solution_is_right() {
    let source = Source::Typed(spec());
    let report = stress(
        &source,
        &max_below(u32::MAX),
        Some(&max_below(u32::MAX)),
        &Judge::Tokens,
        &options(40),
    )
    .unwrap();
    assert_eq!(
        (report.cases, report.skipped, report.failure),
        (40, 0, None)
    );
}

#[test]
fn stress_stops_at_the_first_failure_shrinks_it_and_is_the_same_for_any_jobs() {
    let source = Source::Typed(spec());
    let (fast, brute) = (max_below(1000), max_below(u32::MAX));
    let report = stress(&source, &fast, Some(&brute), &Judge::Tokens, &options(300)).unwrap();
    let failure = report.failure.clone().expect("a value over 1000 turns up");
    assert_eq!(report.cases, failure.case);
    assert_eq!(failure.seed, failure.case as u64);
    // The smallest input with a value over 1000.
    assert_eq!(failure.input, "1 1\n1001\n1 1\n");
    assert_eq!(
        failure.described.as_deref(),
        Some("n = 1  q = 1  a = [1001]  queries = [(1, 1)]")
    );
    assert_eq!(
        (failure.expected.as_str(), failure.got.as_str()),
        ("1001\n", "0\n")
    );
    assert_eq!(failure.verdict.code(), "WA");
    assert!(failure
        .shrunk
        .is_some_and(|(from, to)| from >= to && to == 5));

    let serial = Options {
        jobs: 1,
        ..options(300)
    };
    assert_eq!(
        stress(&source, &fast, Some(&brute), &Judge::Tokens, &serial).unwrap(),
        report
    );
}

#[test]
fn a_generator_program_can_make_the_inputs() {
    // `gen <seed> <size>` prints one number per case: the seed.
    let source = Source::Generator(sh("echo 1; echo $0"));
    let report = stress(
        &source,
        &max_below(4),
        Some(&max_below(u32::MAX)),
        &Judge::Tokens,
        &options(20),
    )
    .unwrap();
    let failure = report.failure.unwrap();
    assert_eq!(
        (failure.case, failure.input.as_str(), failure.shrunk),
        (5, "1\n5\n", None)
    );
}

#[test]
fn a_failing_brute_stops_the_run_with_the_input() {
    let source = Source::Typed(spec());
    let error = stress(
        &source,
        &sh("cat"),
        Some(&sh("exit 1")),
        &Judge::Tokens,
        &options(5),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("the brute script failed on this input"),
        "{error}"
    );
}

#[test]
fn a_failure_is_saved_as_a_regression_test() {
    let dir = scratch("save");
    let source = Source::Typed(spec());
    let report = stress(
        &source,
        &max_below(1000),
        Some(&max_below(u32::MAX)),
        &Judge::Tokens,
        &options(300),
    )
    .unwrap();
    let failure = report.failure.unwrap();
    let saved = save(&dir.join("d.cp"), &failure).unwrap();
    assert_eq!(
        saved,
        dir.join(format!("d.tests/stress-{}.in", failure.seed))
    );
    assert_eq!(std::fs::read_to_string(&saved).unwrap(), "1 1\n1001\n1 1\n");
    assert_eq!(
        std::fs::read_to_string(saved.with_extension("out")).unwrap(),
        "1001\n"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

mod cli {
    use super::{scratch, SRC};
    use crate::cli::run;
    use std::path::Path;

    fn stress(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    /// Writes an executable shell script.
    fn script(dir: &Path, name: &str, body: &str) -> String {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path.display().to_string()
    }

    const MAX: &str = "awk 'NR==2 { m=$1; for (i=2;i<=NF;i++) if ($i>m) m=$i; print m }'";
    const BUGGY: &str =
        "awk 'NR==2 { m=$1; for (i=2;i<=NF;i++) if ($i>m) m=$i; if (m>1000) m=0; print m }'";

    #[test]
    fn reports_agreement_and_a_shrunk_failure() {
        let dir = scratch("cli");
        let file = dir.join("d.cp");
        std::fs::write(&file, SRC).unwrap();
        let file = file.to_str().unwrap();
        let (good, bad) = (script(&dir, "good", MAX), script(&dir, "bad", BUGGY));

        let (code, out, err) = stress(&[file, "--fast", &good, "--brute", &good, "--cases", "30"]);
        assert_eq!((code, err.as_str()), (0, ""), "{out}");
        assert!(out.ends_with("  30 cases agree\n"), "{out}");

        let (code, out, _) = stress(&[file, "--fast", &bad, "--brute", &good, "--cases", "300"]);
        assert_eq!(code, 1);
        assert!(out.contains("FAIL on case"), "{out}");
        assert!(out.contains("    1 1\n    1001\n    1 1\n  that is: n = 1  q = 1  a = [1001]  queries = [(1, 1)]"), "{out}");
        assert!(out.contains("expected `1001`, got `0`"), "{out}");
        assert!(
            out.contains("saved ") && dir.join("d.tests").is_dir(),
            "{out}"
        );

        let (code, out, _) = stress(&[
            file, "--fast", &bad, "--brute", &good, "--cases", "300", "--format", "json",
        ]);
        assert_eq!(code, 1);
        assert!(out.starts_with("{\"status\":\"fail\",\"cases\":"), "{out}");
        assert!(out.contains("\"input\":\"n = 1  q = 1  a = [1001]  queries = [(1, 1)]\",\"raw_input\":\"1 1\\n1001\\n1 1\\n\""), "{out}");
        assert!(
            // No `debug(...)` ran on both sides: these are scripts.
            out.contains("\"verdict\":\"WA\",\"stages\":[],\"saved\":\"")
                && out.contains(",\"trace\":null}"),
            "{out}"
        );
        let (_, out, _) = stress(&[
            file, "--fast", &good, "--brute", &good, "--cases", "5", "--format", "json",
        ]);
        assert_eq!(out, "{\"status\":\"ok\",\"cases\":5}\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_declared_checker_is_never_replaced_by_token_comparison() {
        let dir = scratch("cli-judge");
        let file = dir.join("d.cp");
        std::fs::write(
            &file,
            format!("{SRC}@checker(output: Int)\nfn valid(out: Int) -> Bool:\n    out > 0\n"),
        )
        .unwrap();
        let file = file.to_str().unwrap();
        let good = script(&dir, "good", MAX);
        // The file's own checker judges: `cpl judge <file>` is given the
        // input and the output.
        let src = std::fs::read_to_string(file).unwrap();
        match crate::cli::select_judge(&src, file, None, None).unwrap() {
            crate::Judge::Checker(program) => {
                assert_eq!(program.name, format!("{file} (`valid`)"));
                assert_eq!(program.argv[1..], ["judge".to_string(), file.to_string()]);
            }
            other => panic!("the declared checker was not used: {other:?}"),
        }
        // A program passed with `--checker` judges in its place, and no
        // brute is needed.
        let accept = script(&dir, "accept", "exit 0");
        let (code, out, err) =
            stress(&[file, "--fast", &good, "--checker", &accept, "--cases", "10"]);
        assert_eq!(code, 0, "{err}");
        assert!(
            out.contains("checker: ") && out.contains("10 cases agree"),
            "{out}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn what_is_missing_is_said_plainly() {
        let dir = scratch("cli-errors");
        let file = dir.join("d.cp");
        std::fs::write(&file, SRC).unwrap();
        let path = file.to_str().unwrap();
        let good = script(&dir, "good", MAX);

        let (code, _, err) = stress(&[path, "--fast", &good]);
        assert_eq!(code, 1);
        assert!(err.contains("there is nothing to compare"), "{err}");
        std::fs::write(&file, "let x: Set<Int> = read()\n").unwrap();
        let (code, _, err) = stress(&[path, "--fast", &good, "--brute", &good]);
        assert_eq!(code, 1);
        assert!(
            err.contains("d.cp:1: can't generate a `Set<Int>` yet"),
            "{err}"
        );
        std::fs::write(&file, "print(1)\n").unwrap();
        let (_, _, err) = stress(&[path, "--fast", &good, "--brute", &good]);
        assert!(err.contains("has no typed `read()`"), "{err}");

        for (args, message) in [
            (&[][..], "`stress` needs a file"),
            (
                &["a.cp", "b.cp"][..],
                "`stress` takes one file; `b.cp` is a second",
            ),
            (
                &["a.cp", "--cases", "many"][..],
                "`--cases` takes a whole number, not `many`",
            ),
            (&["a.cp", "--brute"][..], "`--brute` needs a value"),
            (&["a.cp", "--loud"][..], "unknown option `--loud`"),
            (
                &["a.cp", "--checker", "c", "--interactor", "i"][..],
                "a checker or an interactor, not both",
            ),
        ] {
            let (code, _, err) = stress(args);
            assert_eq!(code, 2, "{args:?}");
            assert!(err.contains(message), "{err}");
        }
        let (code, out, _) = stress(&["--help"]);
        assert_eq!(code, 0);
        assert!(out.starts_with("usage: cpl stress"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// Every kind of value a `read()` can ask for.
const KINDS: &str = "\
let n: 2..=8 = read()
domain V = 1..=n
let flag: Bool = read()
let c: Char = read()
let s: Str = read()
let e: [0..2](u: V, v: V) = read()
let p: Perm<V> = read()
let t: Tree<V> = read()
print(n)
";

fn is_perm(value: &Value, n: i128) -> bool {
    let Value::Seq(items) = value else {
        return false;
    };
    let mut seen: Vec<i128> = items.iter().filter_map(Value::number).collect();
    seen.sort();
    seen == (1..=n).collect::<Vec<_>>()
}

fn is_tree(value: &Value, n: i128) -> bool {
    let Value::Seq(edges) = value else {
        return false;
    };
    let mut root: Vec<usize> = (0..=n as usize).collect();
    fn find(root: &mut [usize], v: usize) -> usize {
        if root[v] == v {
            v
        } else {
            let r = find(root, root[v]);
            root[v] = r;
            r
        }
    }
    let mut joined = 0;
    for edge in edges {
        let Value::Tuple(ends) = edge else {
            return false;
        };
        let (Some(a), Some(b)) = (ends[0].number(), ends[1].number()) else {
            return false;
        };
        if !(1..=n).contains(&a) || !(1..=n).contains(&b) {
            return false;
        }
        let (a, b) = (find(&mut root, a as usize), find(&mut root, b as usize));
        if a == b {
            return false;
        }
        root[a] = b;
        joined += 1;
    }
    joined == n - 1
}

#[test]
fn words_letters_flags_records_permutations_and_trees_are_generated_valid() {
    let spec = InputSpec::from_source(KINDS).unwrap();
    for seed in 0..200 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).expect("a case");
        let Some(n) = case.values[0].number() else {
            panic!("n is a number")
        };
        assert!(matches!(case.values[1], Value::Int(0 | 1)));
        assert!(matches!(case.values[2], Value::Char(c) if (97..=122).contains(&c)));
        let Value::Seq(word) = &case.values[3] else {
            panic!("a word")
        };
        assert!(!word.is_empty() && word.iter().all(|c| matches!(c, Value::Char(97..=122))));
        let Value::Seq(pairs) = &case.values[4] else {
            panic!("records")
        };
        assert!(pairs.iter().all(|pair| matches!(pair, Value::Tuple(ends) if ends.iter().all(|end| (1..=n).contains(&end.number().unwrap())))));
        assert!(is_perm(&case.values[5], n), "{case:?}");
        assert!(is_tree(&case.values[6], n), "{case:?}");
        // The lines are what `read()` takes: a word is written together.
        let text = case.render();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4 + 2 + 1 + (n as usize - 1));
        assert!(lines[3].bytes().all(|b| b.is_ascii_lowercase()));
    }
}

#[test]
fn shrinking_keeps_permutations_and_trees_valid_and_makes_them_plain() {
    let spec = InputSpec::from_source(KINDS).unwrap();
    let start = generate(&spec, &mut Rng::new(7), Size::Small(8)).unwrap();
    // Fails whenever the word still has an `x`-or-later letter count of
    // two or more... here: whenever n >= 4. Everything else is noise.
    let mut seen_invalid = false;
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| {
        let n = case.values[0].number().unwrap();
        seen_invalid |= !is_perm(&case.values[5], n) || !is_tree(&case.values[6], n);
        n >= 4
    });
    assert!(!seen_invalid, "a shrink step offered an invalid case");
    assert_eq!(smallest.values[0], Value::Int(4));
    assert_eq!(smallest.values[1], Value::Int(0));
    assert_eq!(smallest.values[2], Value::Char(97));
    assert_eq!(smallest.values[3], Value::Seq(vec![Value::Char(97)]));
    // The permutation in order, and a tree of four vertices.
    assert_eq!(
        smallest.values[5],
        Value::Seq((1..=4).map(Value::Int).collect())
    );
    assert!(is_tree(&smallest.values[6], 4));
    assert_eq!(smallest.describe(&spec).split("  ").next(), Some("n = 4"));
}

#[test]
fn a_multitest_input_has_a_header_and_shrinks_to_one_failing_test() {
    let src = "const LIMIT = 9\n@!multitest\nlet n: 1..=5 = read()\nlet a: [0..n]1..=LIMIT = read()\nprint(a.max)\n";
    let spec = InputSpec::from_source(src).unwrap();
    assert!(spec.fields.is_empty());
    let mut many = None;
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        let mut lines = text.lines();
        // The count of test cases, then two lines for each.
        assert_eq!(
            lines.next().unwrap().parse::<usize>().unwrap(),
            case.cases.len()
        );
        assert_eq!(lines.count(), 2 * case.cases.len());
        if case.cases.len() >= 3 {
            many = Some(case);
        }
    }
    let start = many.expect("some input has three test cases");
    let fails = |case: &Case| {
        case.cases
            .iter()
            .any(|test| matches!(&test.values[1], Value::Seq(a) if a.len() >= 2))
    };
    let mut start = start;
    start.cases[1].values = vec![
        Value::Int(3),
        Value::Seq(vec![Value::Int(4), Value::Int(9), Value::Int(2)]),
    ];
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    assert_eq!(smallest.render(), "1\n2\n1 1\n");
    assert_eq!(smallest.describe(&spec), "n = 2  a = [1, 1]");
}

#[test]
fn a_bug_that_needs_two_tests_shrinks_both_of_them() {
    let src = "@!multitest\nlet n: 1..=5 = read()\nlet a: [0..n]1..=9 = read()\nprint(a.max)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let test = |a: &[i128]| Case {
        values: vec![
            Value::Int(a.len() as i128),
            Value::Seq(a.iter().map(|&x| Value::Int(x)).collect()),
        ],
        cases: Vec::new(),
        until_eof: false,
    };
    let start = Case {
        values: Vec::new(),
        cases: vec![test(&[5, 7]), test(&[3]), test(&[8, 9, 4])],
        until_eof: false,
    };
    // Fails when a later test repeats the first test's largest value, as a
    // count left over from an earlier test would.
    let fails = |case: &Case| {
        let max = |c: &Case| match &c.values[1] {
            Value::Seq(a) => a.iter().filter_map(Value::number).max(),
            _ => None,
        };
        case.cases.len() >= 2
            && case.cases[1..]
                .iter()
                .any(|c| max(c) == max(&case.cases[0]))
    };
    let mut start = start;
    start.cases[2] = test(&[7, 2]);
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    // Both tests shrink to one number each, and the two 7s move together.
    assert_eq!(smallest.render(), "2\n1\n1\n1\n1\n");
}

#[test]
fn an_interactor_says_what_the_hidden_test_is() {
    let src = "\
@!multitest
let n: 1..=100 = read()
let r: Int = ask(\"?\", 1, limit: 7)
print(\"!\", r)

@interactor fn judge(sol: Interactor):
    let n: 1..=100 = read()
    let x: 1..=n = read()
    sol.send(n)
";
    let spec = InputSpec::from_source(src).unwrap();
    let inner = spec.cases.as_ref().expect("a multitest input");
    let names: Vec<_> = inner
        .fields
        .iter()
        .map(|field| field.names.clone())
        .collect();
    assert_eq!(
        names,
        [vec![Some("n".to_string())], vec![Some("x".to_string())]]
    );
}

#[test]
fn an_int_that_sizes_something_is_a_count_and_any_other_int_is_not() {
    let src = "\
let (n, k): (Int, Int) = read()
let q: Int = read()
let shift: Int = read()
var dp: [0..n]Int = 0
fn solve():
    for _ in 0..q: print(List.filled(k, 0))
";
    let spec = InputSpec::from_source(src).unwrap();
    for name in ["n", "k", "q"] {
        assert!(spec.sizes.contains(name), "{name}");
    }
    assert!(!spec.sizes.contains("shift"));
    let mut negative_shift = false;
    for seed in 0..200 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let numbers: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        assert!(
            numbers[..3].iter().all(|&size| (1..=8).contains(&size)),
            "{numbers:?}"
        );
        negative_shift |= numbers[3] < 0;
    }
    assert!(negative_shift);
}

#[test]
fn the_stdlib_perm_and_tree_are_made_as_a_permutation_and_a_tree() {
    let src = "use std.graph.{Tree}\nuse std.seq.{Perm}\nlet n: 1..=10 = read()\ndomain V = 1..=n\nlet p: Perm<V> = read()\nlet t: Tree<V> = read()\nprint(n)\n";
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let analysis = db.analyze("t.cp").expect("analyzed");
    assert!(
        !analysis.has_errors(),
        "the compiler accepts both types: {:?}",
        analysis.diagnostics
    );
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).expect("a case");
        let n = case.values[0].number().unwrap();
        assert!(
            is_perm(&case.values[1], n) && is_tree(&case.values[2], n),
            "{case:?}"
        );
    }
}

#[test]
fn a_letter_domain_is_made_in_its_range_as_the_runtime_reads_it() {
    // `cpl run` and the C++ both read a value of `'A'..'A' + k` as its code
    // point today, so the cases are numbers too.
    let src = "use std.seq.{Perm}\nlet k: 1..=26 = read()\ndomain L = 'A'..'A' + k\nlet c: L = read()\nlet p: Perm<L> = read()\nprint(c)\n";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).expect("a case");
        let k = case.values[0].number().unwrap();
        let letter = |v: &Value| matches!(v, Value::Int(c) if (65..65 + k).contains(c));
        assert!(letter(&case.values[1]), "{case:?}");
        let Value::Seq(p) = &case.values[2] else {
            panic!("{case:?}")
        };
        assert!(p.len() == k as usize && p.iter().all(letter), "{case:?}");
    }
}

#[test]
fn a_weighted_tree_is_made_and_shrunk_as_a_tree_with_its_weights() {
    let src = "let n: 2..=10 = read()\ndomain V = 1..=n\ntype Tree = [0..n - 1](u: V, v: V, w: 1..=9)\nlet t: Tree = read()\nprint(n)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let weights_in_range = |value: &Value| {
        match value {
        Value::Seq(edges) => edges.iter().all(|edge| {
            matches!(edge, Value::Tuple(ends) if ends.len() == 3 && matches!(ends[2], Value::Int(1..=9)))
        }),
        _ => false,
    }
    };
    let mut big = None;
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).expect("a case");
        let n = case.values[0].number().unwrap();
        assert!(
            is_tree(&case.values[1], n) && weights_in_range(&case.values[1]),
            "{case:?}"
        );
        if n >= 5 {
            big = Some(case);
        }
    }
    // Fails while some edge weighs at least 3.
    let heavy = |case: &Case| match &case.values[1] {
        Value::Seq(edges) => edges
            .iter()
            .any(|e| matches!(e, Value::Tuple(ends) if ends[2].number() >= Some(3))),
        _ => false,
    };
    let mut start = big.expect("a tree on at least 5 vertices");
    if let Value::Seq(edges) = &mut start.values[1] {
        if let Value::Tuple(ends) = &mut edges[0] {
            ends[2] = Value::Int(9);
        }
    }
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| heavy(case));
    assert!(is_tree(&smallest.values[1], 2) && weights_in_range(&smallest.values[1]));
    assert_eq!(smallest.render(), "2\n1 2 3\n");
}

/// Whether `value` is `m` edges on `1..=n` with no loop and no pair twice,
/// connected if asked, and with no directed cycle if `dag`.
fn is_graph(value: &Value, n: i128, m: usize, connected: bool, directed: bool, dag: bool) -> bool {
    let Value::Seq(edges) = value else {
        return false;
    };
    let mut pairs = std::collections::HashSet::new();
    let mut reach = vec![vec![false; n as usize + 1]; n as usize + 1];
    for edge in edges {
        let Value::Tuple(ends) = edge else {
            return false;
        };
        let (Some(a), Some(b)) = (ends[0].number(), ends[1].number()) else {
            return false;
        };
        let key = if directed {
            (a, b)
        } else {
            (a.min(b), a.max(b))
        };
        if !(1..=n).contains(&a) || !(1..=n).contains(&b) || a == b || !pairs.insert(key) {
            return false;
        }
        reach[a as usize][b as usize] = true;
        if !directed {
            reach[b as usize][a as usize] = true;
        }
    }
    let n = n as usize;
    for k in 1..=n {
        for i in 1..=n {
            for j in 1..=n {
                reach[i][j] |= reach[i][k] && reach[k][j];
            }
        }
    }
    let cycle = (1..=n).any(|v| reach[v][v]);
    let joined = (2..=n).all(|v| reach[1][v]);
    edges.len() == m && (!connected || joined) && (!dag || !cycle)
}

const GRAPHS: &str = "use std.graph.{SimpleGraph, ConnectedGraph, Dag, Digraph}\n\
let (n, m): (2..=8, 1..=12) = read()\n\
domain V = 1..=n\n\
domain E = 0..m\n\
let a: SimpleGraph<V, E> = read()\n\
let b: ConnectedGraph<V, E> = read()\n\
let c: Dag<V, E> = read()\n\
type SimpleGraph = [E](u: V, v: V, w: 1..=9)\n\
let d: SimpleGraph = read()\n\
let e: Digraph<V, E> = read()\n\
print(n)\n";

fn graphs_valid(case: &Case) -> bool {
    let Value::Tuple(nm) = &case.values[0] else {
        return false;
    };
    let (n, m) = (nm[0].number().unwrap(), nm[1].number().unwrap() as usize);
    is_graph(&case.values[1], n, m, false, false, false)
        && is_graph(&case.values[2], n, m, true, false, false)
        && is_graph(&case.values[3], n, m, false, true, true)
        && is_graph(&case.values[4], n, m, false, false, false)
        && is_graph(&case.values[5], n, m, false, true, false)
}

#[test]
fn simple_connected_and_acyclic_graphs_are_generated_valid() {
    let spec = InputSpec::from_source(GRAPHS).unwrap();
    let mut made = 0;
    for seed in 0..300 {
        // No case when `m` has no room on `n` vertices, or is too few to connect them.
        let Some(case) = generate(&spec, &mut Rng::new(seed), Size::Small(8)) else {
            continue;
        };
        made += 1;
        assert!(graphs_valid(&case), "{case:?}");
        let Value::Seq(weighted) = &case.values[4] else {
            panic!("{case:?}")
        };
        assert!(weighted
            .iter()
            .all(|e| matches!(e, Value::Tuple(ends) if matches!(ends[2], Value::Int(1..=9)))));
    }
    assert!(made >= 100, "only {made} of 300 seeds made a case");
    // A `Digraph` may have a cycle, and an arc both ways; a `Dag` neither.
    let cyclic = (0..300)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .filter(|case| !is_graph(&case.values[5], 8, 0, false, true, true))
        .count();
    assert!(cyclic >= 20, "{cyclic} digraphs with a cycle");
    let full = generate(&spec, &mut Rng::new(1), Size::Full).map(|case| graphs_valid(&case));
    assert_ne!(full, Some(false));
}

#[test]
fn shrinking_keeps_graphs_simple_connected_and_acyclic() {
    let src = "use std.graph.{ConnectedGraph}\nlet (n, m): (2..=8, 1..=12) = read()\ndomain V = 1..=n\ndomain E = 0..m\nlet g: ConnectedGraph<V, E> = read()\nprint(n)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let valid = |case: &Case| {
        let Value::Tuple(nm) = &case.values[0] else {
            return false;
        };
        is_graph(
            &case.values[1],
            nm[0].number().unwrap(),
            nm[1].number().unwrap() as usize,
            true,
            false,
            false,
        )
    };
    let start = (0..300)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| matches!(&case.values[0], Value::Tuple(nm) if nm[0].number() >= Some(6) && nm[1].number() >= Some(8)))
        .expect("a case with n >= 6 and m >= 8");
    let mut seen_invalid = false;
    // Fails while the connected graph has a vertex of degree three.
    let smallest = shrink(&spec, &start, 5000, &mut |case: &Case| {
        seen_invalid |= !valid(case);
        let Value::Seq(edges) = &case.values[1] else {
            return false;
        };
        let mut degree = std::collections::HashMap::new();
        for edge in edges {
            if let Value::Tuple(ends) = edge {
                for end in &ends[..2] {
                    *degree.entry(end.number()).or_insert(0) += 1;
                }
            }
        }
        degree.values().any(|d| *d >= 3)
    });
    assert!(!seen_invalid, "a shrink step offered an invalid case");
    // A star on four vertices: the fewest vertices and edges that fail.
    assert_eq!(
        smallest.values[0],
        Value::Tuple(vec![Value::Int(4), Value::Int(3)])
    );
    assert!(valid(&smallest), "{smallest:?}");
    assert_eq!(smallest.render(), "4 3\n1 2\n1 3\n1 4\n");
}

#[test]
fn deleting_a_graph_vertex_renames_the_vertices_read_beside_it() {
    let src = "use std.graph.{SimpleGraph}\nlet (n, m): (2..=8, 1..=12) = read()\ndomain V = 1..=n\ndomain E = 0..m\nlet g: SimpleGraph<V, E> = read()\nlet (s, t): (V, V) = read()\nprint(n)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let joined = |case: &Case| {
        let (Value::Seq(edges), Value::Tuple(st)) = (&case.values[1], &case.values[2]) else {
            return false;
        };
        let (s, t) = (st[0].number(), st[1].number());
        s != t
            && edges.iter().any(|edge| {
                matches!(edge, Value::Tuple(ends)
                    if (ends[0].number(), ends[1].number()) == (s, t)
                        || (ends[0].number(), ends[1].number()) == (t, s))
            })
    };
    let start = (0..500)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| {
            joined(case) && matches!(&case.values[0], Value::Tuple(nm) if nm[0].number() >= Some(6))
        })
        .expect("a case with an edge from s to t and n >= 6");
    // Fails while the edge s-t is in the graph, so a deleted or renamed
    // vertex must carry `s` and `t` along with the edges.
    let smallest = shrink(&spec, &start, 5000, &mut |case: &Case| joined(case));
    assert_eq!(smallest.render(), "2 1\n1 2\n1 2\n");
}

#[test]
fn values_that_fail_only_while_equal_are_lowered_together() {
    let src = "let n: 1..=5 = read()\nlet a: [0..n]1..=9 = read()\nprint(a.max)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let start = Case {
        values: vec![
            Value::Int(3),
            Value::Seq(vec![Value::Int(7), Value::Int(4), Value::Int(7)]),
        ],
        cases: Vec::new(),
        until_eof: false,
    };
    // Fails while two values repeat: lowering either 7 alone passes.
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| {
        let Value::Seq(a) = &case.values[1] else {
            return false;
        };
        (0..a.len()).any(|i| (0..i).any(|j| a[i] == a[j]))
    });
    assert_eq!(smallest.render(), "2\n1 1\n");
}

/// The degree of each vertex of an edge list on `1..=n`.
fn degrees(value: &Value, n: i128) -> Vec<usize> {
    let mut degree = vec![0; n as usize + 1];
    let Value::Seq(edges) = value else {
        return degree;
    };
    for edge in edges {
        if let Value::Tuple(ends) = edge {
            for end in &ends[..2] {
                degree[end.number().unwrap() as usize] += 1;
            }
        }
    }
    degree
}

#[test]
fn random_trees_include_paths_and_stars() {
    let src = "use std.graph.{Tree}\nlet n: 7..=7 = read()\ndomain V = 1..=n\nlet t: Tree<V> = read()\nprint(n)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let (mut paths, mut stars) = (0, 0);
    for seed in 0..400 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).expect("a case");
        assert!(is_tree(&case.values[1], 7), "{case:?}");
        let degree = degrees(&case.values[1], 7);
        paths += usize::from(degree.iter().all(|&d| d <= 2));
        stars += usize::from(degree.contains(&6));
    }
    eprintln!("{paths} paths and {stars} stars in 400 trees on 7 vertices");
    assert!(paths >= 20 && stars >= 20, "{paths} paths, {stars} stars");
}

#[test]
fn shrinking_a_digraph_keeps_each_arc_the_way_it_points() {
    let src = "use std.graph.{Digraph}\nlet (n, m): (2..=8, 1..=12) = read()\ndomain V = 1..=n\ndomain E = 0..m\nlet g: Digraph<V, E> = read()\nprint(n)\n";
    let spec = InputSpec::from_source(src).unwrap();
    let pointing_down = |case: &Case| {
        let Value::Seq(arcs) = &case.values[1] else {
            return false;
        };
        arcs.iter()
            .any(|arc| matches!(arc, Value::Tuple(ends) if ends[0].number() > ends[1].number()))
    };
    let start = (0..300)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| pointing_down(case) && case.weight().0 > 8)
        .expect("an arc from a higher vertex to a lower one");
    let smallest = shrink(&spec, &start, 5000, &mut |case: &Case| pointing_down(case));
    assert_eq!(smallest.render(), "2 1\n2 1\n");
}

#[test]
fn reads_in_a_loop_are_generated_as_one_row_per_pass() {
    let src = "let (n, q): (1..=5, 0..=3) = read()
domain I = 1..=n
for i in I:
    let (x, y): (1..=9, 1..=9) = read()
    let w: -3..=3 = read()
    print(x + y + w + i)
for _ in 0..q:
    let k: 1..=4 = read()
    for _ in 0..n:
        let c: Char = read()
        print(c)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 3);
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap_or(-100))
            .collect();
        let (n, q) = (tokens[0], tokens[1]);
        let mut at = 2;
        for _ in 0..n {
            assert!((1..=9).contains(&tokens[at]) && (1..=9).contains(&tokens[at + 1]));
            assert!((-3..=3).contains(&tokens[at + 2]));
            at += 3;
        }
        // Letters of one pass are written together, as one word.
        let words: String = case.render().split_whitespace().skip(at).collect();
        let mut rest = words.chars();
        for _ in 0..q {
            let k = rest.next().and_then(|k| k.to_digit(10)).unwrap();
            assert!((1..=4).contains(&k));
            for _ in 0..n {
                assert!(rest.next().is_some_and(|c| c.is_ascii_lowercase()));
            }
        }
        assert_eq!(rest.next(), None, "{}", case.render());
    }
}

#[test]
fn loops_whose_reads_a_field_cannot_describe_are_refused() {
    let refused = [
        // The type uses a value worked out in the pass.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let k: 1..=3 = read()\n    let m = k + 1\n    let a: [m]Int = read()\n",
        // The loop can stop early.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: break\n",
        // A pass can skip a read.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: continue\n    let b: Int = read()\n",
        // A read that is not a typed `let`.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    print(read())\n",
        // A loop over a list, whose length is not a read.
        "for x in [1, 2]:\n    let a: Int = read()\n",
    ];
    for src in refused {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(
            err.message.contains("this loop reads"),
            "{src}: {}",
            err.message
        );
    }
    // `continue` after the last read leaves every read in place.
    let src = "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: continue\n    print(a)\n";
    assert!(InputSpec::from_source(src).is_ok());
}

#[test]
fn a_jump_that_ends_only_a_nested_loop_or_fn_keeps_the_pass_whole() {
    let accepted = [
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    var x = a\n    while true:\n        if x == 1: break\n        x -= 1\n    let b: 1..=9 = read()\n    print(x + b)\n",
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: continue\n        print(i)\n    let b: 1..=9 = read()\n    print(b)\n",
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    fn f(x: Int) -> Int:\n        if x > 3: return 0\n        x\n    let b: 1..=9 = read()\n    print(f(a) + b)\n",
    ];
    for src in accepted {
        let spec = InputSpec::from_source(src).unwrap_or_else(|e| panic!("{src}: {}", e.message));
        let case = generate(&spec, &mut Rng::new(3), Size::Small(5)).unwrap();
        let tokens = case.render().split_whitespace().count() as i128;
        let n: i128 = case
            .render()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(tokens, 1 + 2 * n, "{src}");
    }
    let refused = [
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: return\n    let b: 1..=9 = read()\n",
        "let n: 1..=5 = read()\n'outer: for _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: break 'outer\n    let b: 1..=9 = read()\n",
    ];
    for src in refused {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(
            err.message.contains("this loop reads"),
            "{src}: {}",
            err.message
        );
    }
}

#[test]
fn a_read_with_its_type_as_an_argument_is_generated_from_it() {
    let src = "let n: 1..=5 = read()
let a = read<[n]1..=9>()
let b = [0..n] _ => read::<-3..=-1>()
let c: [0..n]100..=200 = [0..n] _ => read()
for _ in 0..n:
    let (x, y) = read<(20..=30, 40..=50)>()
    print(x + y)
print(a, b, c)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 5);
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0] as usize;
        assert_eq!(tokens.len(), 1 + 5 * n);
        let part = |at: usize| &tokens[1 + at * n..1 + (at + 1) * n];
        assert!(part(0).iter().all(|v| (1..=9).contains(v)));
        assert!(part(1).iter().all(|v| (-3..=-1).contains(v)));
        assert!(part(2).iter().all(|v| (100..=200).contains(v)));
        for row in part(3).iter().chain(part(4)).collect::<Vec<_>>().chunks(2) {
            assert!(
                (20..=30).contains(row[0]) && (40..=50).contains(row[1]),
                "{tokens:?}"
            );
        }
    }
    // A read that may run another number of times is refused, not left out.
    for src in [
        "let n: 1..=5 = read()\nprint(read() + n)\n",
        "let n: 1..=5 = read()\nlet a = [0..n] i => if i == 0: 0 else: read::<Int>()\n",
        "let n: 1..=5 = read()\nlet a = [read::<Int>() for x in 0..n if x > 1]\n",
        "let n: 1..=5 = read()\nfor a in read<[n]1..=3>():\n    print(read::<Int>())\n",
    ] {
        assert!(InputSpec::from_source(src).is_err(), "{src}");
    }
}

#[test]
fn typed_reads_inside_an_expression_are_fields_in_the_order_they_run() {
    let src = "let n: 1..=5 = read()
print(read::<1..=3>() + n * read::<7..=8>())
let b = [0..n] _ => (read::<10..=19>(), read::<20..=29>())
for a in read<[n]30..=39>():
    print(a)
var at: [0..n]Int = 0
for i in 0..n:
    at[read<0..n>()] = i
for _ in 0..n:
    match read::<40..=41>():
        40 => print(1)
        _ => print(2)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0] as usize;
        assert_eq!(tokens.len(), 3 + 5 * n, "{tokens:?}");
        assert!((1..=3).contains(&tokens[1]) && (7..=8).contains(&tokens[2]));
        for (at, pair) in tokens[3..3 + 2 * n].chunks(2).enumerate() {
            assert!(
                (10..=19).contains(&pair[0]) && (20..=29).contains(&pair[1]),
                "{at}"
            );
        }
        assert!(tokens[3 + 2 * n..3 + 3 * n]
            .iter()
            .all(|v| (30..=39).contains(v)));
        assert!(tokens[3 + 3 * n..3 + 4 * n]
            .iter()
            .all(|v| (0..n as i128).contains(v)));
        assert!(tokens[3 + 4 * n..].iter().all(|v| (40..=41).contains(v)));
    }
}

#[test]
fn a_pass_reads_a_count_then_that_many_values() {
    let src = "let n: 1..=5 = read()
for _ in 0..n:
    let k: 0..=4 = read()
    let a: [k]1..=9 = read()
    for _ in 0..k:
        let (lo, c): (0..=k, 0..=k) = read()
        print(lo + c)
    print(a)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 2);
    let tokens = |case: &Case| -> Vec<i128> {
        case.render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect()
    };
    let rows = |tokens: &[i128]| {
        let mut at = 1;
        let mut rows = Vec::new();
        for _ in 0..tokens[0] {
            let k = tokens[at];
            let a = tokens[at + 1..][..k as usize].to_vec();
            let pairs = tokens[at + 1 + k as usize..][..2 * k as usize].to_vec();
            at += 1 + 3 * k as usize;
            rows.push((k, a, pairs));
        }
        assert_eq!(at, tokens.len(), "{tokens:?}");
        rows
    };
    let mut counts = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        for (k, a, pairs) in rows(&tokens(&case)) {
            assert!((0..=4).contains(&k));
            assert!(a.iter().all(|v| (1..=9).contains(v)));
            assert!(pairs.iter().all(|v| (0..=k).contains(v)));
            counts.insert(k);
        }
    }
    assert_eq!(counts.len(), 5, "every count is made: {counts:?}");
    // A 7 anywhere fails: the pass with it shrinks to one value, its
    // count with it, and the other passes go.
    let fails = |case: &Case| rows(&tokens(case)).iter().any(|(_, a, _)| a.contains(&7));
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case) && tokens(case).len() > 8)
        .unwrap();
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    assert_eq!(
        tokens(&smallest),
        [1, 1, 7, 0, 0],
        "from {:?}",
        tokens(&start)
    );
}

#[test]
fn a_value_and_an_element_go_together_when_neither_goes_alone() {
    // A FIFO memory of `m` words that holds only `m - 1`: failing needs a
    // word read again `m - 1` new words later, so neither `m` nor the
    // words shrink alone.
    let src = "let (m, n): (1..=100, 1..=1000) = read()
for _ in 0..n:
    let w: 0..=1000 = read()
    print(w)
";
    let spec = InputSpec::from_source(src).unwrap();
    let misses = |m: usize, words: &[i128], room: usize| {
        let mut memory: Vec<i128> = Vec::new();
        let mut count = 0;
        for &w in words {
            if memory.contains(&w) {
                continue;
            }
            count += 1;
            memory.push(w);
            if memory.len() > room.min(m) {
                memory.remove(0);
            }
        }
        count
    };
    let fails = |case: &Case| {
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let m = tokens[0] as usize;
        let words = &tokens[2..];
        misses(m, words, m) != misses(m, words, m - 1)
    };
    let start = Case {
        values: vec![
            Value::Tuple(vec![Value::Int(4), Value::Int(5)]),
            Value::Seq([4, 0, 1, 2, 4].map(Value::Int).to_vec()),
        ],
        cases: Vec::new(),
        until_eof: false,
    };
    assert!(fails(&start));
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    assert_eq!(smallest.render(), "1 2\n0 0\n");
}

#[test]
fn reads_no_field_describes_are_refused_not_left_out() {
    let cases = [
        ("let n: 1..=5 = read()\nfn row() -> Int:\n    let x: Int = read()\n    if x > 0: return x\n    x\nprint(row())\n", "this fn reads input"),
        ("let n: 1..=5 = read()\n@memo\nfn row(i: Int) -> Int:\n    let x: Int = read()\n    x\nprint(row(n))\n", "this fn reads input"),
        (
            "@!multitest\nlet n: 2..=9 = read()\nfn pick() -> Int: n\nlet x: 1..=n = read(gen: pick)\n",
            "read(gen: ..)",
        ),
        ("let n: 1..=5 = read()\nfor _ in 0..n:\n    let x: ?Int = read(none: \"-\")\n", "this loop reads"),
    ];
    for (src, wanted) in cases {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(err.message.contains(wanted), "{src}: {}", err.message);
    }
    // `read2(n)` is a call of the program's own fn, not of `read`.
    let src = "let n: 1..=5 = read()\nfn read2(x: Int) -> Int: x\nprint(read2(n))\n";
    assert!(InputSpec::from_source(src).is_ok());
}

#[test]
fn a_domain_read_as_a_size_is_generated_with_its_arrays() {
    let src = "domain V = read(1..=6)
let p: [V]V = read()
let t: 1..=3 = read()
for _ in 0..t:
    domain S = read(0..=4)
    let s: [S]V = read()
    print(s)
print(p)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut sizes = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0];
        assert!((1..=6).contains(&n));
        let p = &tokens[1..][..n as usize];
        assert!(p.iter().all(|v| (0..n).contains(v)), "{tokens:?}");
        let mut at = 1 + n as usize;
        for _ in 0..tokens[at] {
            at += 1;
            let k = tokens[at];
            assert!((0..=4).contains(&k));
            let s = &tokens[at + 1..][..k as usize];
            assert!(s.iter().all(|v| (0..n).contains(v)), "{tokens:?}");
            at += k as usize;
        }
        assert_eq!(at + 1, tokens.len(), "{tokens:?}");
        sizes.insert(n);
    }
    assert!(sizes.len() > 3);
}

#[test]
fn a_fn_that_reads_is_read_where_each_call_is() {
    let src = "fn read_pair() -> (Int, Int):
    let a: 1..=3 = read()
    let b: 7..=8 = read()
    (a, b)

fn solve():
    let p = read_pair()
    let k: 20..=29 = read()
    print(p.0 + k)

let first = read_pair()
solve()
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len(), 5, "{tokens:?}");
        assert!((1..=3).contains(&tokens[0]) && (7..=8).contains(&tokens[1]));
        assert!((1..=3).contains(&tokens[2]) && (7..=8).contains(&tokens[3]));
        assert!((20..=29).contains(&tokens[4]), "{tokens:?}");
    }
    // A call that runs only some of the time, a reader with a parameter
    // or an early `return` reads what no field can describe.
    for refused in [
        "fn one() -> Int:\n    let a: 1..=3 = read()\n    a\nlet c: 0..=1 = read()\nif c == 1: print(one())\n",
        "fn one(m: Int) -> Int:\n    let a: 1..=3 = read()\n    a + m\nlet b = one(2)\n",
        "fn one() -> Int:\n    let c: 0..=1 = read()\n    if c == 0: return 0\n    let a: 1..=3 = read()\n    a\nlet b = one()\n",
    ] {
        let error = InputSpec::from_source(refused).unwrap_err();
        assert!(
            ["this fn reads input", "is not a top-level"].iter().any(|m| error.message.contains(m)),
            "{refused}: {}",
            error.message
        );
    }
}

#[test]
fn a_fn_that_reads_fixed_values_reads_them_in_any_expression() {
    let src = "fn ri() -> Int:
    let x: 1..=9 = read()
    return x

let n: 1..=4 = read()
var a = List<Int>()
for _ in 0..n: a.push(ri() + ri())
print(a.sum())
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("value-reader-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + 2 * tokens[0], "{text}");
        assert!(tokens[1..].iter().all(|t| (1..=9).contains(t)), "{text}");
    }
    std::fs::remove_file(&path).ok();
    // Called only some of the time, its reads are not a field.
    let branch = "fn ri() -> Int:\n    let x: Int = read()\n    x\nlet c: 0..=1 = read()\nif c == 1: print(ri())\n";
    assert!(InputSpec::from_source(branch).is_err());
    // A type that uses a value the body read is not the same each call.
    let dependent = "fn row() -> Int:\n    let k: 1..=3 = read()\n    let a: [k]Int = read()\n    a.sum()\nfor _ in 0..2: print(row())\n";
    assert!(InputSpec::from_source(dependent).is_err());
}

#[test]
fn a_write_of_read_into_a_typed_array_reads_its_element_type() {
    let src = "let n: 1..=5 = read()
var a: [0..=n]0..=9 = 0
for i in 1..=n: a[i] = read()
var grid: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in 0..=i: grid[i, j] = read()
print(a.sum() + grid[n - 1, 0])
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("elem-read-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let words: Vec<i128> = text.split_whitespace().map(|w| w.parse().unwrap()).collect();
        let n = words[0];
        assert_eq!(words.len() as i128, 1 + n + n * (n + 1) / 2, "{text}");
        assert!(words[1..=n as usize].iter().all(|v| (0..=9).contains(v)), "{text}");
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_loop_read_may_use_the_loop_name() {
    let src = "let n: 1..=5 = read()
var best = 0
for r in 1..=n:
    let row: [0..r]0..=9 = read()
    best = max(best, row.sum())
print(best)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("loop-index-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let n: i128 = text.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(
            text.split_whitespace().count() as i128,
            1 + n * (n + 1) / 2,
            "{text}"
        );
        // Shrinking keeps row `r` at `r` values.
        let shrunk = crate::shrink::fit(&spec, &case).unwrap().render();
        assert_eq!(
            shrunk.split_whitespace().count(),
            text.split_whitespace().count()
        );
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_word_tag_chooses_by_the_words_the_program_compares_it_with() {
    let src = "let q: 1..=6 = read()
var total = 0
for _ in 0..q:
    let op: Str = read()
    if op == \"ADD\":
        let x: 1..=9 = read()
        total += x
    elif op == \"SHOW\": print(total)
    else:
        let (a, b): (1..=9, 1..=9) = read()
        total -= a * b
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("word-tag-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut seen = HashSet::new();
    for seed in 0..60 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(6)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let mut tokens = text.split_whitespace();
        for _ in 0..tokens.next().unwrap().parse::<usize>().unwrap() {
            let op = tokens.next().unwrap();
            seen.insert(op.to_string());
            let more = match op {
                "ADD" => 1,
                "SHOW" => 0,
                _ => 2,
            };
            for _ in 0..more {
                assert!(
                    (1..=9).contains(&tokens.next().unwrap().parse::<i128>().unwrap()),
                    "{text}"
                );
            }
        }
        assert!(tokens.next().is_none(), "{text}");
        let fitted = crate::shrink::fit(&spec, &case).unwrap();
        assert_eq!(fitted.render(), text);
    }
    assert_eq!(seen.len(), 3, "{seen:?}");
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_reader_may_return_once_its_reads_are_done() {
    let src = "fn main_solve():
    let n: 1..=4 = read()
    let xs: [0..n]0..=9 = read()
    fn deep(i: Int) -> Int:
        if i == n: return 0
        xs[i] + deep(i + 1)
    if deep(0) == 0:
        print(\"none\")
        return
    print(deep(0))

main_solve()
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(4)).unwrap();
        let text = case.render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, tokens[0] + 1, "{text}");
    }
}

#[test]
fn floats_and_wide_ints_are_read_as_whole_numbers() {
    let src = "let (k, n): (F64, 1..=5) = read()
let xs: [0..n]F64 = read()
let w: I128 = read()
print(k + xs.sum(), w)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("floats-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, tokens[1] + 3, "{text}");
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_tag_read_first_in_a_tuple_or_above_other_reads_still_chooses() {
    let src = "domain Cell = 1..=9
let q: 1..=6 = read()
for _ in 0..q:
    let (kind, l): (0..=2, Cell) = read()
    let r: Cell = read()
    match kind:
        1 =>
            let (a, b): (Cell, Cell) = read()
            print(a + b + l + r)
        _ => print(l + r)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut kinds = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        let mut tokens = text.split_whitespace().map(|t| t.parse::<i128>().unwrap());
        for _ in 0..tokens.next().unwrap() {
            let kind = tokens.next().unwrap();
            assert!((0..=2).contains(&kind), "{text}");
            kinds.insert(kind);
            for _ in 0..if kind == 1 { 4 } else { 2 } {
                assert!((1..=9).contains(&tokens.next().unwrap()), "{text}");
            }
        }
        assert!(tokens.next().is_none(), "{text}");
    }
    assert_eq!(kinds.len(), 3);
    let shadowed = "let (kind, l): (0..=1, 0..=1) = read()
let kind: 5..=6 = read()
if kind == 5:
    let x: Int = read()
    print(x)
";
    assert!(InputSpec::from_source(shadowed).is_err());
}

#[test]
fn a_query_reads_what_the_branch_its_tag_takes_reads() {
    let src = "let q: 1..=6 = read()
for _ in 0..q:
    let op: Int = read()
    if op == 1:
        let (l, r): (1..=9, 1..=9) = read()
        print(l + r)
    elif op == 2 or op == 3:
        let i: 1..=9 = read()
        print(i)
    else:
        print(0)
    let kind: Char = read()
    match kind:
        'B' =>
            let k: 0..=3 = read()
            let a: [k]5..=7 = read()
            print(a)
        _ => print(kind)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut seen = HashSet::new();
    for seed in 0..200 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        let mut tokens = text.split_whitespace();
        let mut next = || tokens.next().expect("a token");
        for _ in 0..next().parse::<usize>().unwrap() {
            let op: i128 = next().parse().unwrap();
            let reads = match op {
                1 => 2,
                2 | 3 => 1,
                _ => 0,
            };
            for _ in 0..reads {
                assert!((1..=9).contains(&next().parse::<i128>().unwrap()), "{text}");
            }
            let kind = next();
            if kind == "B" {
                let k: usize = next().parse().unwrap();
                for _ in 0..k {
                    assert!((5..=7).contains(&next().parse::<i128>().unwrap()), "{text}");
                }
            }
            seen.insert((op.clamp(0, 4), kind == "B"));
        }
        assert!(tokens.next().is_none(), "{text}");
    }
    assert!(seen.len() >= 6, "{seen:?}");
    // Fails when some query asks for `B` with a 7 in it: the smallest
    // such input is one query that reads nothing, then a `B` with one 7.
    let mut fails = |case: &Case| {
        let Value::Seq(rows) = &case.values[1] else {
            return false;
        };
        rows.iter().any(|row| {
            matches!(row, Value::Tuple(parts) if matches!(&parts[1],
                Value::Tuple(b) if b.first() == Some(&Value::Char(i128::from(b'B')))
                    && matches!(b.get(2), Some(Value::Seq(a)) if a.contains(&Value::Int(7)))))
        })
    };
    let start = (0..500)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| case.weight().0 >= 12 && fails(case))
        .expect("a large failing case");
    let smallest = shrink(&spec, &start, 4000, &mut fails);
    assert_eq!(
        smallest.render(),
        "1\n4\nB\n1\n7\n",
        "from {}",
        start.render()
    );
}

#[test]
fn stdlib_structs_residues_and_unsigned_values_are_generated() {
    let src = "const P = 7
let p: Pt = read()
let q: Pt3 = read()
let r: Mod<P> = read()
let s: Mod<5> = read()
let u: U64 = read()
print(p.x + p.y + q.z, r, s, u)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("std-structs-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len(), 8, "{text}");
        assert!(
            (0..7).contains(&tokens[5]) && (0..5).contains(&tokens[6]),
            "{text}"
        );
        assert!(tokens[7] >= 0, "{text}");
    }
    let min = shrink(
        &spec,
        &generate(&spec, &mut Rng::new(1), Size::Small(8)).unwrap(),
        2000,
        &mut |_| true,
    );
    let text = min.render();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[2..], ["0", "0", "0"]);
    let shadowed = "struct Pt(x: 1..=2)\nlet p: Pt = read()\nprint(p.x)\n";
    let case = generate(
        &InputSpec::from_source(shadowed).unwrap(),
        &mut Rng::new(3),
        Size::Small(8),
    )
    .unwrap();
    assert!(
        ["1\n", "2\n"].contains(&case.render().as_str()),
        "{}",
        case.render()
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn structs_and_tagged_enums_are_read_field_by_field() {
    let src = "domain Row = 1..=3
struct Pt(x: 0..=9, y: Row)
@tags(\"R\", \"C\")
enum Stroke:
    Across(r: Row)
    Down(c: 1..=4, d: 0..=1)
let n: 1..=5 = read()
let ps: [n]Pt = read()
let k: 1..=5 = read()
let s: [k]Stroke = read()
for p in ps:
    print(p.x + p.y)
for x in s:
    match x:
        Across(r) => print(r)
        Down(c, d) => print(c * 10 + d)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("records-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut tags = HashSet::new();
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        tags.extend(
            text.split_whitespace()
                .filter(|t| *t == "R" || *t == "C")
                .map(String::from),
        );
    }
    assert_eq!(tags.len(), 2);
    let fails = |case: &Case| case.render().split_whitespace().any(|t| t == "C");
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    assert_eq!(min.render(), "1\n0 1\n1\nC 1 0\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn an_optional_read_with_none_written_as_a_number_is_generated_both_ways() {
    let src = "let n: 1..=6 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let b: ?(5..=9) = read(none: \"-1\")
var placed = 0
for i in P:
    if let v = a[i]:
        placed += v
if let x = b:
    placed += x
print(placed)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("maybe-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let (mut absent, mut present) = (0, 0);
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0];
        for v in &tokens[1..=n as usize] {
            assert!((0..=n).contains(v), "{text}");
            if *v == 0 {
                absent += 1
            } else {
                present += 1
            }
        }
        assert!(tokens[1 + n as usize] == -1 || (5..=9).contains(&tokens[1 + n as usize]));
    }
    assert!(absent > 10 && present > 10, "{absent} {present}");
    // A value that is there but no longer needed shrinks to `none`.
    let fails = |case: &Case| {
        case.render()
            .lines()
            .nth(1)
            .is_some_and(|a| a.split(' ').any(|t| t == "3"))
    };
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    assert_eq!(min.render(), "3\n0 0 3\n-1\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_read_with_gen_draws_what_its_fn_makes_from_the_input_above() {
    let src = "let n: 3..=8 = read()
domain I = 0..n
fn word() -> [I]Char: [I] _ => 'a' + rand(0, 2)
let s: [I]Char = read(gen: word)
let k: 1..=n = read()
var c = 0
for i in I:
    if s[i] == 'c':
        c += 1
print(c + k)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("drawn-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut words = HashSet::new();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let lines: Vec<&str> = text.lines().collect();
        let n: usize = lines[0].parse().unwrap();
        assert_eq!(lines[1].len(), n, "{text}");
        assert!(
            lines[1].bytes().all(|b| (b'a'..=b'c').contains(&b)),
            "{text}"
        );
        let k: usize = lines[2].parse().unwrap();
        assert!((1..=n).contains(&k), "{text}");
        words.insert(lines[1].to_string());
    }
    assert!(words.len() > 10, "{words:?}");
    let fails = |case: &Case| {
        case.render()
            .lines()
            .nth(1)
            .is_some_and(|s| s.contains("cc"))
    };
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case) && case.render().lines().next() != Some("3"))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    let text = min.render();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        (lines[0], lines[1].len(), lines[2]),
        ("3", 3, "1"),
        "{}",
        min.render()
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_swapped_in_brute_builds_once_and_leaves_no_file_beside_the_source() {
    if std::process::Command::new("g++")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = scratch("part-brute");
    let file = dir.join("p.cp");
    let src = "fn f(n: Int) -> Int = n * 2\n\n@brute(f)\nfn f_slow(n: Int) -> Int = n + n\n\nlet n: 1..=9 = read()\nprint(f(n))\n";
    std::fs::write(&file, src).unwrap();
    let swapped = crate::cli::swap_in_part_brutes(src).unwrap();
    let first = crate::cli::build_part_brute(&file, "p.cp", &swapped).unwrap();
    let again = crate::cli::build_part_brute(&file, "p.cp", &swapped).unwrap();
    assert_eq!(first.argv, again.argv);
    // Named for the source alone: a name that changed per run (say, with
    // the process id) would change the C++ and miss the build cache.
    let binary = first.argv.iter().find(|a| a.contains(".p.brute")).unwrap();
    let name = Path::new(binary).file_name().unwrap().to_str().unwrap();
    let hashes = name.strip_prefix(".p.brute-").unwrap().split('-');
    assert!(
        hashes
            .into_iter()
            .all(|h| h.len() == 16 && h.chars().all(|c| c.is_ascii_hexdigit())),
        "{name}"
    );
    let outcome = first.run("4\n", Duration::from_secs(10)).unwrap();
    assert_eq!(outcome.stdout, "8\n");
    let left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["p.cp"]);
}
