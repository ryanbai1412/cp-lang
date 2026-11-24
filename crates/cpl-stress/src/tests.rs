use super::*;
use crate::judge::{declared, first_difference};
use crate::run::{save, stress, Options, Source};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod spec;

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
fn shrinking_moves_a_negative_value_toward_0_or_its_ranges_end_nearest_0() {
    let spec = InputSpec::from_source(
        "let n: 1..=9 = read()\nlet a: [n]-1000000000..=1000000000 = read()\nlet b: -9..=-3 = read()\n",
    )
    .unwrap();
    let start = Case::of(vec![
        Value::Int(3),
        Value::Seq([5, -999999992, 7].map(Value::Int).to_vec()),
        Value::Int(-8),
    ]);
    // Fails while some element is at most -10.
    let mut fails = |case: &Case| matches!(&case.values[1], Value::Seq(a) if a.iter().any(|v| matches!(v, Value::Int(v) if *v <= -10)));
    assert_eq!(
        shrink(&spec, &start, 2000, &mut fails).render(),
        "1\n-10\n-3\n"
    );
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
fn yes_and_no_match_in_any_case() {
    assert_eq!(first_difference("Yes\nNO\n", "YES\nno\n"), None);
    // 1059D's sample: a real matches within 1e-6, absolute or relative.
    assert_eq!(first_difference("0.625000000\n", "0.625\n"), None);
    assert_eq!(first_difference("1e9\n", "1000000000.5\n"), None);
    assert_eq!(first_difference("0.5\n", "0.5000009\n"), None);
    assert_eq!(first_difference("3\n", "3.0000001\n"), None);
    assert!(first_difference("0.5\n", "0.50001\n").is_some());
    // Two integers, and words, still match exactly.
    assert!(first_difference("10\n", "11\n").is_some());
    assert!(first_difference("1e5\n", "inf\n").is_some());
    assert!(first_difference("e\n", "E\n").is_some());
    assert_eq!(
        first_difference("Yes\n", "No\n").as_deref(),
        Some("first difference at line 1, token 1: expected `Yes`, got `No`")
    );
    assert_eq!(
        first_difference("Yes 1\n", "YES 2\n").as_deref(),
        Some("first difference at line 1, token 2: expected `1`, got `2`")
    );
    assert!(first_difference("abc\n", "ABC\n").is_some());
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
        crate::program::python()[0]
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
fn a_rebuild_reuses_the_emitted_cpp_until_a_file_it_read_changes() {
    if std::process::Command::new("g++")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = scratch("emitted");
    std::fs::create_dir_all(dir.join("mylib")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nmylib = \"mylib\"\n").unwrap();
    let helper = dir.join("mylib/helper.cp");
    std::fs::write(&helper, "pub fn f() -> Int = 1\n").unwrap();
    let main = dir.join("main.cp");
    std::fs::write(&main, "use mylib.helper.{f}\nprint(f())\n").unwrap();
    let run = || {
        let built = Program::resolve(&main).unwrap();
        let out = built.run("", Duration::from_secs(10)).unwrap().stdout;
        (built.argv, out)
    };
    let (first, out) = run();
    assert_eq!(out, "1\n");
    assert_eq!(run(), (first.clone(), out));
    // The library is read again, so its edit is built, not the old C++.
    std::fs::write(&helper, "pub fn f() -> Int = 2\n").unwrap();
    let (second, out) = run();
    assert_eq!(out, "2\n");
    assert_ne!(second, first);
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
    // Windows has no signals: MSYS reports a killed shell as an exit code.
    #[cfg(unix)]
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
fn an_interactive_failure_is_not_explained_by_a_run_without_its_interactor() {
    let src = "let n: Int = read()\nlet r: Str = ask(\"?\", n, limit: 1)\nlet s: Str = ask(\"?\", n, limit: 1)\nprint(r, s)\n";
    let path = std::env::temp_dir().join(format!("ask-explain-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    assert_eq!(crate::explain::runtime_error(&path, "3\n"), None);
}

#[test]
fn a_solution_over_its_query_limit_is_blamed_not_the_starved_interactor() {
    let interactor = Judge::Interactor(
        sh("read guess || { echo \"the solution's output ended where a word was expected\" >&2; exit 1; }"),
        Launch::Stdio,
    );
    let over = sh("echo 'query limit exceeded: the limit is 30' >&2; exit 3");
    let judged = interactor
        .run(&over, "2\n", None, Duration::from_secs(10))
        .unwrap();
    assert_eq!(
        judged.verdict,
        Verdict::RuntimeError("exit code 3: query limit exceeded: the limit is 30".to_string())
    );
}

#[test]
fn an_interactor_talks_to_the_solution_and_rules() {
    // The secret is the input; the solution may ask three questions.
    let interactor = Judge::Interactor(
        sh("secret=$(cat \"$0\"); for turn in 1 2 3; do read guess || exit 1; if [ \"$guess\" = \"$secret\" ]; then echo yes; exit 0; fi; echo no; done; echo 'too many guesses' >&2; exit 1"),
        Launch::Stdio,
    );
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
    assert_eq!(
        lost.outcome.stdout, "1\n2\n3\n4\n5\n",
        "all the solution said"
    );
    let silent = interactor
        .run(&sh("sleep 5"), "2\n", None, Duration::from_millis(100))
        .unwrap();
    assert_eq!(silent.verdict, Verdict::TimeLimit);
}

#[test]
fn a_spawning_interactor_runs_the_solution_itself_as_often_as_it_needs() {
    // A run-twice judge: the first run's output, transformed, is the
    // second run's input (cf2237g's shape).
    let dir = scratch("spawn");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("interactor.sh");
    std::fs::write(
        &path,
        "#!/bin/sh\n# interactor: spawn\ncase=$1; shift 2\n\
         first=$(\"$@\" < \"$case\") || { echo 'first run failed' >&2; exit 1; }\n\
         second=$(echo \"again $first\" | \"$@\")\n\
         [ \"$second\" = \"done $(cat \"$case\")\" ] || { echo \"got $second\" >&2; exit 1; }\n",
    )
    .unwrap();
    assert_eq!(crate::Launch::declared(&path), crate::Launch::Spawn);
    let interactor = Judge::Interactor(Program::resolve(&path).unwrap(), crate::Launch::Spawn);
    let limit = Duration::from_secs(10);
    let twice = sh("read a b; if [ \"$a\" = again ]; then echo done $b; else echo $a; fi");
    let judged = interactor.run(&twice, "5\n", None, limit).unwrap();
    assert_eq!(judged.verdict, Verdict::Accepted);
    let once = sh("read a b; echo $a");
    let judged = interactor.run(&once, "5\n", None, limit).unwrap();
    assert_eq!(
        judged.verdict,
        Verdict::Wrong("interactor: got again".to_string())
    );
    let stdio = dir.join("stdio.sh");
    std::fs::write(&stdio, "#!/bin/sh\n# a judge on stdio\nread x\n").unwrap();
    assert_eq!(crate::Launch::declared(&stdio), crate::Launch::Stdio);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_accepted_interactive_solution_finishing_slowly_is_not_killed_as_re() {
    let interactor = Judge::Interactor(sh("read ans; [ \"$ans\" = 42 ]"), Launch::Stdio);
    let slow_exit = sh("echo 42; sleep 0.08; exit 0");
    let judged = interactor
        .run(&slow_exit, "", None, Duration::from_secs(5))
        .unwrap();
    assert_eq!(judged.verdict, Verdict::Accepted);
}

/// Judges charge a solution its CPU time: a turn it spends blocked (here
/// asleep) is not time it used, so a 1 s limit passes a 1.5 s nap. Only
/// twice the limit blocked on its turn is idle.
#[cfg(target_os = "linux")]
#[test]
fn an_interactive_solution_is_charged_its_cpu_time_not_its_turns() {
    let interactor = Judge::Interactor(sh("echo 7; read ans; [ \"$ans\" = 8 ]"), Launch::Stdio);
    let limit = Duration::from_millis(1000);
    let run = |solution: &str| interactor.run(&sh(solution), "", None, limit).unwrap();
    let napping = run("read q; sleep 1.5; echo $((q + 1))");
    assert_eq!(napping.verdict, Verdict::Accepted, "{napping:?}");
    assert!(napping.outcome.elapsed < limit, "{napping:?}");
    let stuck = run("read q; sleep 5");
    assert_eq!(stuck.verdict, Verdict::TimeLimit, "{stuck:?}");
    assert!(stuck.outcome.elapsed > limit * 2, "{stuck:?}");
    let spinning = run("read q; while :; do :; done");
    assert_eq!(spinning.verdict, Verdict::TimeLimit, "{spinning:?}");
    assert!(spinning.outcome.elapsed > limit, "{spinning:?}");
}

#[test]
fn declared_judges_are_found_with_their_lines() {
    let src = "@check\nfn valid():\n    let out: Int = read_output()\n    assert out > 0\n\n@interactor(Guess)\nfn judge():\n    0\nfn plain() = 1\n";
    let found = declared(src);
    let seen: Vec<(&str, &str, u32)> = found
        .iter()
        .map(|d| (d.kind, d.name.as_str(), d.line))
        .collect();
    assert_eq!(seen, [("checker", "valid", 2), ("interactor", "judge", 7)]);
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
fn a_failing_brute_is_a_failing_case_blamed_on_the_brute() {
    let source = Source::Typed(spec());
    let report = stress(
        &source,
        &sh("cat"),
        Some(&sh("exit 1")),
        &Judge::Tokens,
        &options(5),
    )
    .unwrap();
    let failure = report.failure.expect("the brute fails");
    assert_eq!(
        failure.verdict,
        Verdict::BruteError("exit code 1".to_string())
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
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
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
        assert!(
            out.contains("  30 cases agree\n  tried: n from 1 to 8, a up to 8 long"),
            "{out}"
        );

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
        let report = cpl_json::parse(&out).unwrap();
        assert_eq!(report.at(&["status"]).as_str(), Some("fail"), "{out}");
        let case = report.at(&["counterexample"]);
        // The case the text run saved runs first and still fails.
        assert_eq!(
            case.at(&["raw_input"]).as_str(),
            Some("1 1\n1001\n1 1\n"),
            "{out}"
        );
        assert_eq!(case.at(&["rerun"]).as_bool(), Some(true), "{out}");
        assert_eq!(case.at(&["verdict"]).as_str(), Some("WA"), "{out}");
        // No `debug(...)` ran on both sides: these are scripts.
        assert_eq!(
            case.at(&["stages"]).as_array().map(<[_]>::len),
            Some(0),
            "{out}"
        );
        assert!(case.at(&["saved"]).as_str().is_some(), "{out}");
        assert!(
            case.at(&["trace"]).is_null() && case.get("trace").is_some(),
            "{out}"
        );
        let (_, out, _) = stress(&[
            file, "--fast", &good, "--brute", &good, "--cases", "5", "--format", "json",
        ]);
        let report = cpl_json::parse(&out).unwrap();
        assert_eq!(report.at(&["status"]).as_str(), Some("ok"), "{out}");
        assert_eq!(report.at(&["case_count"]).as_i64(), Some(5), "{out}");
        assert_eq!(report.at(&["saved_count"]).as_i64(), Some(1), "{out}");
        assert!(report.at(&["tried"]).as_str().is_some(), "{out}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_declared_checker_is_never_replaced_by_token_comparison() {
        let dir = scratch("cli-judge");
        let file = dir.join("d.cp");
        std::fs::write(
            &file,
            format!(
                "{SRC}@check\nfn valid():\n    let out: Int = read_output()\n    assert out > 0\n"
            ),
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

        let (code, out, _) = stress(&[path, "--fast", &good, "--cases", "20"]);
        assert_eq!(code, 0, "{out}");
        assert!(
            out.contains("only runtime errors and the time limit fail a case")
                && out.contains("20 cases ran with no runtime error or time limit"),
            "{out}"
        );
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
            (
                &["a.cp", "--loud"][..],
                "`stress` has no option `--loud`; `cpl stress --help` lists them",
            ),
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
    let mut db = cpl_check::Session::new();
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

/// Each case runs the interactor once: the solution is what replays.
#[test]
fn an_in_process_interactor_runs_once_per_case() {
    let src = "let n: Int = read()\nask(\"!\", n)\n\n@interactor\nfn judge(mut sol: Interactor):\n    let n: 1..=9 = read()\n    sol.send(n)\n    sol.send(n)\n    let kind: Str = sol.recv()\n    let x: Int = sol.recv()\n    assert kind == \"!\" and x == n\n";
    let before = crate::duplex::INTERACTOR_RUNS.with(|runs| runs.get());
    let result = crate::stress_source("o.cp", src, &options(40), Some(1 << 26), false).unwrap();
    assert_eq!(result.report.failure, None, "{}", result.render());
    let runs = crate::duplex::INTERACTOR_RUNS.with(|runs| runs.get()) - before;
    assert_eq!(runs, 40);
}

/// A solution, with no brute, that runs out of any limit under the full
/// one, and out of the full limit on inputs longer than `stuck_over` bytes.
/// `full_runs` counts the runs that spent the full limit.
struct Spins {
    stuck_over: usize,
    full_runs: std::sync::atomic::AtomicU32,
}

impl crate::engine::Engine for Spins {
    fn has_brute(&self) -> bool {
        false
    }

    fn stops(&self, _: &str) -> bool {
        true
    }

    fn limit(&self) -> Duration {
        Duration::from_millis(300)
    }

    fn run_within(
        &self,
        input: &str,
        limit: Duration,
    ) -> Result<crate::case::Wrong, crate::run::StressError> {
        let full = limit >= self.limit();
        let spins = !full || input.len() > self.stuck_over;
        if full && spins {
            self.full_runs
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(spins
            .then(|| crate::case::Failed::new(Verdict::TimeLimit, String::new(), String::new())))
    }
}

/// Runs out of the full time limit cost the whole limit each, so narrowing
/// and shrinking a time-limit case spend a bounded number of them.
#[test]
fn a_time_limit_case_spends_a_bounded_number_of_full_limit_runs() {
    let spec = InputSpec::from_source(
        "let k: Int = read()\nlet n: 1..=8 = read()\nlet a: [0..n]0..=10 = read()\n",
    )
    .unwrap();
    // Stuck on every input, the smallest included; then stuck only on
    // long ones, so the short case the quick shrink reaches passes.
    for stuck_over in [0, 12] {
        let engine = Spins {
            stuck_over,
            full_runs: Default::default(),
        };
        let options = Options {
            cases: 30,
            jobs: 1,
            size: 8,
            ..Options::default()
        };
        let report =
            crate::run::stress_with(&crate::run::InputSource::Typed(&spec), &engine, &options)
                .unwrap();
        let failure = report.failure.expect("a time-limit case");
        assert_eq!(failure.verdict, Verdict::TimeLimit);
        let runs = engine.full_runs.into_inner();
        // The case itself, the bounded runs, and the check that the case
        // a quick shrink reached still runs out of the full limit.
        assert!(
            runs <= 2 + crate::run::TLE_RUNS,
            "{runs} full-limit runs (stuck over {stuck_over} bytes)"
        );
    }
}

#[test]
fn each_verdict_names_the_program_whose_trap_explains_it() {
    use crate::judge::Trapped;
    let named = [
        (Verdict::Accepted, None),
        (Verdict::Wrong("4 vs 5".into()), None),
        (Verdict::TimeLimit, None),
        (Verdict::MemoryLimit, None),
        (
            Verdict::RuntimeError("crashed".into()),
            Some(Trapped::Solution),
        ),
        (Verdict::BruteError("crashed".into()), Some(Trapped::Brute)),
        (
            Verdict::JudgeError("checker", "R0101".into()),
            Some(Trapped::Checker),
        ),
        (
            Verdict::JudgeError("interactor", "R0101".into()),
            Some(Trapped::Interactor),
        ),
    ];
    for (verdict, trapped) in named {
        assert_eq!(verdict.trapped(), trapped, "{verdict:?}");
        let helper = matches!(
            trapped,
            Some(Trapped::Brute | Trapped::Checker | Trapped::Interactor)
        );
        assert_eq!(verdict.helper_failed(), helper, "{verdict:?}");
        assert_eq!(
            verdict.stopped(),
            matches!(verdict, Verdict::RuntimeError(_) | Verdict::TimeLimit)
        );
    }
}

#[test]
fn every_stress_decorator_gives_the_file_a_judge_and_a_mention_does_not() {
    for name in crate::STRESS_DECORATORS {
        let src = format!("let n: 1..=9 = read()\n@{name}\nfn other():\n    print(n)\nprint(n)\n");
        assert!(crate::stress_targets(&src), "@{name}");
        let mut db = cpl_check::Session::new();
        db.set_file("s.cp", src.as_str());
        let analysis = db.analyze("s.cp").unwrap();
        let judged = cpl_ir::brute(&analysis).is_some()
            || cpl_ir::checker(&analysis).is_some()
            || cpl_ir::interactor(&analysis).is_some();
        assert!(judged, "@{name} names no judge `cpl stress` runs");
    }
    let mentioned = "// @alt, @brute and @interactor\nlet s = \"@check\"\nprint(s)\n";
    assert!(!crate::stress_targets(mentioned));
}
