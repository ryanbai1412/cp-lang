use super::*;
use crate::judge::{declared, first_difference};
use crate::run::{save, stress, Options, Source};
use std::path::PathBuf;
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
    let src = "let n: 1..=5 = read()\nlet s: Str = read()\n";
    let error = InputSpec::from_source(src).unwrap_err();
    assert_eq!(error.span.text(src), "Str");
    assert_eq!(
        error.message,
        "can't generate a `Str` yet; pass a generator with `--gen`"
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
    let case = Case {
        values: vec![
            Value::Tuple(vec![Value::Int(3), Value::Int(2)]),
            Value::Seq(vec![Value::Int(1), Value::Int(5), Value::Int(2)]),
            Value::Seq(vec![
                Value::Tuple(vec![Value::Int(1), Value::Int(3)]),
                Value::Tuple(vec![Value::Int(2), Value::Int(2)]),
            ]),
        ],
    };
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
    let mut case = Case {
        values: vec![
            Value::Tuple(vec![Value::Int(2), Value::Int(1)]),
            Value::Seq(vec![Value::Int(1), Value::Int(5), Value::Int(2)]),
            Value::Seq(vec![
                Value::Tuple(vec![Value::Int(1), Value::Int(3)]),
                Value::Tuple(vec![Value::Int(2), Value::Int(2)]),
            ]),
        ],
    };
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
    let start = Case {
        values: vec![
            Value::Int(5),
            Value::Seq([1, 7, 1, 1, 9].map(Value::Int).to_vec()),
        ],
    };
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
    std::fs::write(dir.join("d.cp"), "print(1)\n").unwrap();
    if std::env::var_os("CPL_PROTO").is_none() {
        let error = Program::resolve(&dir.join("d.cp")).unwrap_err().to_string();
        assert!(error.contains("the C++ backend is not written"), "{error}");
        assert!(error.contains("d.cpp"), "{error}");
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
            out.contains("\"verdict\":\"WA\",\"stages\":[],\"saved\":\""),
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
        let (code, _, err) = stress(&[file, "--fast", &good, "--brute", &good]);
        assert_eq!(code, 1);
        assert!(
            err.contains("declares the checker `valid` (line 8)"),
            "{err}"
        );
        assert!(
            err.contains("pass it built with `--checker <program>`"),
            "{err}"
        );
        // With one, it judges, and no brute is needed.
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
        let (code, _, err) = stress(&[path, "--brute", &good]);
        assert_eq!(code, 1);
        assert!(
            err.contains("the C++ backend is not written")
                || std::env::var_os("CPL_PROTO").is_some(),
            "{err}"
        );

        std::fs::write(&file, "let s: Str = read()\n").unwrap();
        let (code, _, err) = stress(&[path, "--fast", &good, "--brute", &good]);
        assert_eq!(code, 1);
        assert!(err.contains("d.cp:1: can't generate a `Str` yet"), "{err}");
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
