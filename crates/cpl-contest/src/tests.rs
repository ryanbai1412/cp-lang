use super::*;
use crate::bench::{bench, worst_case, WorstShape};
use crate::new::{reads, Scaffold};
use crate::problem::{cases, source_path, Limits};
use crate::test::{render, run_cases};
use crate::watch::{Stamp, Watch};
use cpl_json::{self as json, Value as Json};
use cpl_stress::{InputSpec, Judge, Program, Verdict};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-contest-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg_attr(not(unix), allow(dead_code))]
fn sh(script: &str) -> Program {
    Program::command("script", &["sh", "-c", script])
}

/// Writes an executable shell script and returns its path.
#[cfg_attr(not(unix), allow(dead_code))]
fn script(dir: &Path, name: &str, body: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path.display().to_string()
}

type Command = fn(&[String], &mut dyn std::io::Write, &mut dyn std::io::Write) -> u8;

fn cli(command: Command, args: &[&str]) -> (u8, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = command(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

const COMPANION: &str = r#"{
  "name": "D. Range Max", "group": "Codeforces",
  "url": "https://codeforces.com/problemset/problem/1/D",
  "interactive": false, "memoryLimit": 512, "timeLimit": 1500,
  "tests": [{"input": "3\n1 5 2\n", "output": "5\n"}, {"input": "1\n7\n", "output": "7\n"}]
}"#;

/// Prints the largest number on the second line.
#[cfg_attr(not(unix), allow(dead_code))]
const MAX: &str = "awk 'NR==2 { m=$1; for (i=2;i<=NF;i++) if ($i>m) m=$i; print m }'";

#[test]
fn crate_is_named() {
    assert_eq!(NAME, "cpl-contest");
}

#[test]
fn json_reads_values_escapes_and_nesting() {
    let value = json::parse(
        r#" {"a": [1, 2.5, -3e2], "s": "x\n\"y\"\u0041", "t": true, "n": null, "o": {}} "#,
    )
    .unwrap();
    assert_eq!(value.get("a").unwrap().as_array().unwrap().len(), 3);
    assert_eq!(
        value.get("a").unwrap().as_array().unwrap()[0].as_u64(),
        Some(1)
    );
    assert_eq!(
        value.get("a").unwrap().as_array().unwrap()[1].as_u64(),
        None
    );
    assert_eq!(value.get("s").unwrap().as_str(), Some("x\n\"y\"A"));
    assert_eq!(value.get("t"), Some(&Json::Bool(true)));
    assert_eq!(value.get("n"), Some(&Json::Null));
    assert_eq!(
        value.get("o"),
        Some(&Json::Object(std::collections::BTreeMap::new()))
    );
    assert_eq!(value.get("missing"), None);
    for bad in ["", "{", "[1,", "{\"a\" 1}", "\"open", "1 2", "tru"] {
        assert!(json::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn constraints_become_typed_reads() {
    assert_eq!(
        reads("n, q: 1..=2e5; a: [n]0..=1e9; queries: [q](1..=n, 1..=n); g: [0..n, 0..3]0..=1")
            .unwrap(),
        [
            "let (n, q): (1..=2e5, 1..=2e5) = read()",
            "let a: [0..n]0..=1e9 = read()",
            "let queries: [0..q](1..=n, 1..=n) = read()",
            "let g: [0..n, 0..3]0..=1 = read()",
        ]
    );
    assert_eq!(
        reads("edges: [n - 1](1..=4, 1..=4)").unwrap(),
        ["let edges: [0..n - 1](1..=4, 1..=4) = read()"]
    );
    assert_eq!(
        reads("n 5").unwrap_err(),
        "`n 5` has no type; write `n: 1..=2e5`"
    );
    assert_eq!(
        reads("N: 1..=5").unwrap_err(),
        "`N` is not a name; names look like `n` or `max_w`"
    );
    assert_eq!(
        reads("n: 1..=5; x: Set<Int>").unwrap_err(),
        "in `let x: Set<Int> = read()`: can't generate a `Set<Int>` yet; pass a generator with `--gen`"
    );
}

#[test]
fn a_companion_problem_gives_the_title_limits_and_samples() {
    let scaffold = Scaffold::from_companion(COMPANION).unwrap();
    assert_eq!(scaffold.title.as_deref(), Some("D. Range Max"));
    assert_eq!(scaffold.short_name().as_deref(), Some("d"));
    let limits = scaffold.limits.unwrap();
    assert_eq!(
        (limits.time, limits.memory_kb),
        (Duration::from_millis(1500), 512 * 1024)
    );
    assert_eq!(scaffold.samples.len(), 2);
    assert!(!scaffold.interactive);
    assert_eq!(
        Scaffold::from_companion("[]").unwrap_err(),
        "a Competitive Companion problem is a JSON object"
    );
    assert!(Scaffold::from_companion("{")
        .unwrap_err()
        .starts_with("this is not JSON"));
}

#[test]
fn the_new_source_has_a_header_the_reads_and_the_template() {
    let mut scaffold = Scaffold::from_companion(COMPANION).unwrap();
    scaffold.reads = reads("n: 1..=2e5; a: [n]0..=1e9").unwrap();
    let source = scaffold.source(Some("fn solve():\n    0\n\n"));
    assert_eq!(
        source,
        "// D. Range Max\n// https://codeforces.com/problemset/problem/1/D\n// judge: codeforces\n// limits: 1500 ms, 512 MB\n\nlet n: 1..=2e5 = read()\nlet a: [0..n]0..=1e9 = read()\n\nfn solve():\n    0\n"
    );
    // The limits line is read back by test and bench.
    assert_eq!(
        Limits::of(Path::new("d.cp"), &source),
        scaffold.limits.unwrap()
    );
    // With nothing known, there is still a typed read to start from.
    assert_eq!(
        Scaffold::default().source(None),
        "let n: 1..=2e5 = read()\n"
    );
    assert!(InputSpec::from_source(&source).is_ok());
}

#[test]
fn new_creates_the_file_and_samples_and_never_overwrites() {
    let dir = scratch("new");
    let companion = dir.join("problem.json");
    std::fs::write(&companion, COMPANION).unwrap();
    let name = dir.join("d");
    let (code, out, err) = cli(
        new::run,
        &[
            name.to_str().unwrap(),
            "--companion",
            companion.to_str().unwrap(),
            "--constraints",
            "n: 1..=2e5; a: [n]0..=1e9",
            "--time-limit",
            "3000",
        ],
    );
    assert_eq!((code, err.as_str()), (0, ""));
    assert!(out.contains("d.cp") && out.contains("(2 samples)"), "{out}");
    let source = std::fs::read_to_string(dir.join("d.cp")).unwrap();
    assert!(
        source.contains("// limits: 3000 ms, 512 MB\n\nlet n: 1..=2e5 = read()\n"),
        "{source}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("d.tests/1.in")).unwrap(),
        "3\n1 5 2\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("d.tests/2.out")).unwrap(),
        "7\n"
    );
    let names: Vec<String> = cases(&dir.join("d.cp"))
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(names, ["1", "2"]);

    let (code, _, err) = cli(new::run, &[name.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(
        err.contains("already exists; `cpl new` won't overwrite it"),
        "{err}"
    );
    assert_eq!(std::fs::read_to_string(dir.join("d.cp")).unwrap(), source);

    // A project template is used when there is a cp.toml.
    std::fs::write(dir.join("cp.toml"), "[limits]\ntime_ms = 4000\n").unwrap();
    std::fs::write(dir.join("template.cp"), "fn solve():\n    0\n").unwrap();
    let (code, _, err) = cli(new::run, &[dir.join("e").to_str().unwrap()]);
    assert_eq!((code, err.as_str()), (0, ""));
    assert_eq!(
        std::fs::read_to_string(dir.join("e.cp")).unwrap(),
        "fn solve():\n    0\n"
    );
    assert!(dir.join("e.tests").is_dir());

    for (args, message) in [
        (&[][..], "`new` needs a name"),
        (&["a", "b"][..], "`new` takes one name; `b` is a second"),
        (&["a", "--constraints", "n"][..], "`n` has no type"),
        (
            &["a", "--loud"][..],
            "`new` has no option `--loud`; `cpl new --help` lists them",
        ),
    ] {
        let (code, _, err) = cli(new::run, args);
        assert_eq!(code, 2, "{args:?}");
        assert!(err.contains(message), "{err}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn limits_come_from_the_header_then_cp_toml_then_defaults() {
    let dir = scratch("limits");
    let source = dir.join("d.cp");
    assert_eq!(Limits::of(&source, ""), Limits::default());
    assert_eq!(Limits::default().header(), "// limits: 2000 ms, 256 MB");
    std::fs::write(
        dir.join("cp.toml"),
        "[lints]\ntime_ms = 1\n[limits]\ntime_ms = 4000 # slow judge\nmemory_mb = 64\n",
    )
    .unwrap();
    let project = Limits::of(&source, "");
    assert_eq!(
        (project.time, project.memory_kb),
        (Duration::from_millis(4000), 64 * 1024)
    );
    let own = Limits::of(&source, "// title\n// limits: 3 s, 1024 MB\n");
    assert_eq!(
        (own.time, own.memory_kb),
        (Duration::from_secs(3), 1024 * 1024)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_problem_without_limits_gets_its_judges_usual_ones() {
    let dir = scratch("judge-limits");
    let source = dir.join("d.cp");
    let cses = Limits::of(&source, "// judge: cses\n");
    assert_eq!(
        (cses.time, cses.memory_kb),
        (Duration::from_millis(1000), 512 * 1024)
    );
    let own = Limits::of(&source, "// judge: cses\n// limits: 3000 ms, 64 MB\n");
    assert_eq!(
        (own.time, own.memory_kb),
        (Duration::from_millis(3000), 64 * 1024)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cases_are_samples_in_order_then_the_rest_by_name() {
    let dir = scratch("cases");
    let tests = dir.join("d.tests");
    std::fs::create_dir_all(&tests).unwrap();
    for file in [
        "10.in",
        "2.in",
        "2.out",
        "stress-7.in",
        "stress-7.out",
        "hand.in",
        "notes.txt",
    ] {
        std::fs::write(tests.join(file), "").unwrap();
    }
    let found = cases(&dir.join("d.cp"));
    let seen: Vec<(&str, bool)> = found
        .iter()
        .map(|c| (c.name.as_str(), c.expected.is_some()))
        .collect();
    assert_eq!(
        seen,
        [
            ("2", true),
            ("10", false),
            ("hand", false),
            ("stress-7", true)
        ]
    );
    assert_eq!(cases(&dir.join("none.cp")), []);
    assert_eq!(source_path("d"), PathBuf::from("d.cp"));
    assert_eq!(source_path("d.cp"), PathBuf::from("d.cp"));
    // A binary built beside the problem doesn't shadow it.
    let beside = std::env::temp_dir().join(format!("cpl-source-path-{}", std::process::id()));
    std::fs::create_dir_all(&beside).unwrap();
    let built = beside.join("cf2272A");
    std::fs::write(&built, b"\x7fELF").unwrap();
    assert_eq!(source_path(built.to_str().unwrap()), built);
    std::fs::write(built.with_extension("cp"), "print(1)\n").unwrap();
    assert_eq!(
        source_path(built.to_str().unwrap()),
        built.with_extension("cp")
    );
    std::fs::remove_dir_all(&beside).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

/// A problem with three cases: one right, one wrong, one with no `.out`.
fn problem(dir: &Path) -> PathBuf {
    let source = dir.join("d.cp");
    std::fs::write(
        &source,
        "let n: 1..=2e5 = read()\nlet a: [0..n]0..=1e9 = read()\n",
    )
    .unwrap();
    let tests = dir.join("d.tests");
    std::fs::create_dir_all(&tests).unwrap();
    for (file, text) in [
        ("1.in", "3\n1 5 2\n"),
        ("1.out", "5\n"),
        ("2.in", "2\n4 4\n"),
        ("2.out", "9\n"),
        ("hand.in", "1\n8\n"),
    ] {
        std::fs::write(tests.join(file), text).unwrap();
    }
    source
}

#[test]
fn test_reports_each_case_and_expands_failures() {
    let dir = scratch("test");
    let source = problem(&dir);
    let results = run_cases(
        &sh(MAX),
        &Judge::Tokens,
        &cases(&source),
        Limits {
            time: Duration::from_secs(10),
            ..Limits::default()
        },
    )
    .unwrap();
    let verdicts: Vec<Option<&str>> = results
        .iter()
        .map(|r| r.verdict.as_ref().map(Verdict::code))
        .collect();
    assert_eq!(verdicts, [Some("ok"), Some("WA"), None]);
    assert_eq!(
        results.iter().map(|r| r.passed()).collect::<Vec<_>>(),
        [true, false, true]
    );
    // Times vary; the rest of the report does not.
    let report: String = render(&results)
        .lines()
        .map(|line| match line.find(" ms") {
            Some(at) => format!(
                "{} N ms\n",
                line[..at]
                    .trim_end_matches(|c: char| c.is_ascii_digit())
                    .trim_end()
            ),
            None => format!("{line}\n"),
        })
        .collect();
    assert_eq!(
        report,
        "  1     ok N ms\n  2     WA N ms\n     expected: 9\n     got:      4\n     first difference at line 1, token 1: expected `9`, got `4`\n  hand  ran N ms\n     output: 8\n  2/3 passed (1 had no expected output)\n  if any valid answer is accepted, add a `@check fn` that reads the output with `read_output()`\n"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

// Its programs are shell scripts.
#[cfg(unix)]
#[test]
fn the_test_command_judges_with_a_checker_or_an_interactor() {
    let dir = scratch("test-cli");
    let source = problem(&dir);
    let path = source.to_str().unwrap();
    let solution = script(&dir, "max", MAX);

    let (code, out, _) = cli(test::run, &[path, "--fast", &solution]);
    assert_eq!(code, 1);
    assert!(
        out.ends_with("  2/3 passed (1 had no expected output)\n  if any valid answer is accepted, add a `@check fn` that reads the output with `read_output()`\n"),
        "{out}"
    );
    let (code, out, _) = cli(test::run, &[path, "--fast", &solution, "--format", "json"]);
    assert_eq!(code, 1);
    let json = cpl_json::parse(&out).expect("cpl test --format json is JSON");
    assert_eq!(
        (json.at(&["passed"]).as_i64(), json.at(&["total"]).as_i64()),
        (Some(2), Some(3)),
        "{out}"
    );
    let cases = json.at(&["cases"]).as_array().unwrap();
    let wrong: Vec<_> = cases
        .iter()
        .filter(|c| c.at(&["verdict"]).as_str() == Some("WA"))
        .collect();
    assert_eq!(wrong.len(), 1, "{out}");
    assert!(
        wrong[0].at(&["expected"]).as_str().is_some() && wrong[0].at(&["why"]).as_str().is_some(),
        "{out}"
    );
    let (code, _, err) = cli(test::run, &[path, "--format", "xml"]);
    assert_eq!(code, 2);
    assert!(err.contains("not `xml`"), "{err}");
    // `d` names `d.cp`.
    let (code, _, _) = cli(
        test::run,
        &[dir.join("d").to_str().unwrap(), "--fast", &solution],
    );
    assert_eq!(code, 1);

    // A checker that accepts any single number: every case is judged.
    let checker = script(&dir, "checker", "[ \"$(wc -w < \"$2\")\" -eq 1 ]");
    let (code, out, err) = cli(
        test::run,
        &[path, "--fast", &solution, "--checker", &checker],
    );
    assert_eq!((code, err.as_str()), (0, ""), "{out}");
    assert!(
        out.ends_with("  3/3 passed\n") && !out.contains("ran"),
        "{out}"
    );

    // An interactor that sends the input and expects one line back.
    let interactor = script(
        &dir,
        "interactor",
        "cat \"$1\"; read answer && [ -n \"$answer\" ]",
    );
    // An interactive solution answers as soon as it has read enough.
    let talker = script(&dir, "talker", "read n; read a; echo \"$n\"");
    let (code, out, _) = cli(
        test::run,
        &[path, "--fast", &talker, "--interactor", &interactor],
    );
    assert_eq!(code, 0, "{out}");
    let silent = script(&dir, "silent", "read n; read a");
    let (code, out, _) = cli(
        test::run,
        &[path, "--fast", &silent, "--interactor", &interactor],
    );
    assert_eq!(code, 1);
    assert!(
        out.contains("WA") && out.contains("the interactor rejected the run"),
        "{out}"
    );

    // A declared judge is the judge: `cpl judge <file>` runs it.
    let declared = "let n: 1..=5 = read()\n@check\nfn valid():\n    let out: Int = read_output()\n    assert out > 0\n";
    match cpl_stress::cli::select_judge(declared, path, None, None).unwrap() {
        Judge::Checker(program) => {
            assert_eq!(program.argv[1..], ["judge".to_string(), path.to_string()])
        }
        other => panic!("the declared checker was not used: {other:?}"),
    }

    let (code, _, err) = cli(test::run, &[dir.join("empty.cp").to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(err.contains("can't read"), "{err}");
    let (code, _, err) = cli(test::run, &[]);
    assert_eq!(code, 2);
    assert!(
        err.contains("`test` needs a problem: `cpl test d`"),
        "{err}"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

fn spec(src: &str) -> InputSpec {
    InputSpec::from_source(src).unwrap()
}

#[test]
fn worst_cases_have_every_size_at_its_maximum() {
    let spec = spec("let n: 1..=6 = read()\nlet a: [0..n]2..=9 = read()\n");
    let render = |shape| worst_case(&spec, shape, 1).map(|case| case.render());
    assert_eq!(
        render(WorstShape::AllMin).as_deref(),
        Some("6\n2 2 2 2 2 2\n")
    );
    assert_eq!(
        render(WorstShape::AllMax).as_deref(),
        Some("6\n9 9 9 9 9 9\n")
    );
    assert_eq!(
        render(WorstShape::Ascending).as_deref(),
        Some("6\n2 3 4 5 6 7\n")
    );
    assert_eq!(
        render(WorstShape::Descending).as_deref(),
        Some("6\n9 8 7 6 5 4\n")
    );
    assert_eq!(
        render(WorstShape::Alternating).as_deref(),
        Some("6\n2 9 2 9 2 9\n")
    );
    let random = render(WorstShape::Random).unwrap();
    assert!(
        random.starts_with("6\n") && random.split_whitespace().count() == 7,
        "{random}"
    );
    assert_eq!(render(WorstShape::Random), Some(random));
    // No edge list, so no path or star.
    assert_eq!(
        (render(WorstShape::Path), render(WorstShape::Star)),
        (None, None)
    );
}

#[test]
fn edge_lists_are_shaped_as_a_path_and_a_star() {
    let spec =
        spec("let n: 2..=5 = read()\ndomain V = 1..=n\nlet edges: [0..n - 1](V, V) = read()\n");
    let render = |shape| worst_case(&spec, shape, 1).map(|case| case.render());
    assert_eq!(
        render(WorstShape::Path).as_deref(),
        Some("5\n1 2\n2 3\n3 4\n4 5\n")
    );
    assert_eq!(
        render(WorstShape::Star).as_deref(),
        Some("5\n1 2\n1 3\n1 4\n1 5\n")
    );
    // An array too long to make is skipped, not attempted.
    let huge =
        InputSpec::from_source("let n: 1..=1e12 = read()\nlet a: [0..n]0..=1 = read()\n").unwrap();
    assert_eq!(worst_case(&huge, WorstShape::AllMin, 1), None);
}

#[test]
fn trees_are_shaped_as_a_path_and_a_star() {
    let spec = spec(
        "use std.graph.{Tree}\nlet n: 2..=5 = read()\ndomain V = 1..=n\nlet t: Tree<V> = read()\n",
    );
    let render = |shape| worst_case(&spec, shape, 1).map(|case| case.render());
    assert_eq!(
        render(WorstShape::Path).as_deref(),
        Some("5\n1 2\n2 3\n3 4\n4 5\n")
    );
    assert_eq!(
        render(WorstShape::Star).as_deref(),
        Some("5\n1 2\n1 3\n1 4\n1 5\n")
    );
}

// Its programs are shell scripts.
#[cfg(unix)]
#[test]
fn bench_compares_time_and_memory_with_the_limits() {
    let spec = spec("let n: 1..=50 = read()\nlet a: [0..n]0..=9 = read()\n");
    let limits = Limits {
        time: Duration::from_millis(400),
        memory_kb: 1024 * 1024,
    };
    let fine = bench(&spec, &sh(MAX), limits, 1).unwrap();
    assert_eq!(fine.len(), 6);
    assert!(
        fine.iter()
            .all(|run| run.status == "ok" && run.numbers == 51),
        "{fine:?}"
    );
    let report = bench::render(&fine, limits);
    assert!(
        report.contains("  all smallest         51 numbers  ok "),
        "{report}"
    );
    assert!(
        report.contains("% of the 400 ms limit\n")
            && report.ends_with("  within limits on every shape\n"),
        "{report}"
    );

    let short = Limits {
        time: Duration::from_millis(50),
        ..limits
    };
    let slow = bench(&spec, &sh("sleep 5"), short, 1).unwrap();
    assert!(slow
        .iter()
        .all(|run| run.status == "TLE" && run.elapsed < Duration::from_secs(3)));
    assert!(bench::render(&slow, short).ends_with("  6 shapes are over the limits\n"));
    let crashing = bench(&spec, &sh("cat > /dev/null; exit 3"), limits, 1).unwrap();
    assert!(crashing.iter().all(|run| run.status == "RE"));
    if cfg!(target_os = "linux") {
        let tight = Limits {
            memory_kb: 1,
            ..limits
        };
        let hungry = bench(&spec, &sh("cat > /dev/null; sleep 0.05"), tight, 1).unwrap();
        assert!(hungry.iter().all(|run| run.status == "MLE"), "{hungry:?}");
    }
}

// Its programs are shell scripts.
#[cfg(unix)]
#[test]
fn the_bench_command_prints_a_report_or_json() {
    let dir = scratch("bench-cli");
    let source = dir.join("d.cp");
    std::fs::write(
        &source,
        "// limits: 5000 ms, 512 MB\nlet n: 1..=40 = read()\nlet a: [0..n]0..=9 = read()\n",
    )
    .unwrap();
    let path = source.to_str().unwrap();
    let solution = script(&dir, "max", MAX);
    let (code, out, err) = cli(bench::run, &[path, "--fast", &solution]);
    assert_eq!((code, err.as_str()), (0, ""), "{out}");
    // Peak memory is only measured on Linux so far.
    assert!(
        out.contains("of the 5000 ms limit")
            && (cfg!(not(target_os = "linux")) || out.contains(" MB of 512 MB\n")),
        "{out}"
    );
    let (code, out, _) = cli(
        bench::run,
        &[
            path,
            "--fast",
            &solution,
            "--format",
            "json",
            "--memory-limit",
            "64",
        ],
    );
    assert_eq!(code, 0);
    assert!(out.starts_with("{\"status\":\"ok\",\"time_limit_ms\":5000,\"memory_limit_kb\":65536,\"runs\":[{\"shape\":\"random\",\"numbers\":41,\"status\":\"ok\",\"ms\":"), "{out}");
    let (code, _, _) = cli(
        bench::run,
        &[
            path,
            "--fast",
            &script(&dir, "slow", "sleep 5"),
            "--time-limit",
            "50",
        ],
    );
    assert_eq!(code, 1);

    std::fs::write(&source, "print(1)\n").unwrap();
    let (code, _, err) = cli(bench::run, &[path, "--fast", &solution]);
    assert_eq!(code, 1);
    assert!(
        err.contains("has no typed `read()` to make inputs from"),
        "{err}"
    );
    std::fs::write(&source, "let x: Set<Int> = read()\n").unwrap();
    let (_, _, err) = cli(bench::run, &[path, "--fast", &solution]);
    assert!(
        err.contains("d.cp:1: can't generate a `Set<Int>` yet"),
        "{err}"
    );
    let (code, _, err) = cli(bench::run, &[path, "--format", "xml"]);
    assert_eq!(code, 2);
    assert!(
        err.contains("`--format` is text or json, not `xml`"),
        "{err}"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_stamp_changes_when_a_file_changes_appears_or_disappears() {
    let dir = scratch("stamp");
    let source = problem(&dir);
    let before = Stamp::of(&source, &[]);
    assert_eq!(before, Stamp::of(&source, &[]));
    std::fs::write(dir.join("d.tests/3.in"), "1\n1\n").unwrap();
    let added = Stamp::of(&source, &[]);
    assert_ne!(before, added);
    std::fs::remove_file(dir.join("d.tests/3.in")).unwrap();
    assert_eq!(before, Stamp::of(&source, &[]));
    // A `--fast` program is watched too.
    let fast = dir.join("d.cpp");
    let absent = Stamp::of(&source, std::slice::from_ref(&fast));
    std::fs::write(&fast, "int main() {}\n").unwrap();
    assert_ne!(absent, Stamp::of(&source, std::slice::from_ref(&fast)));
    std::fs::remove_dir_all(dir).unwrap();
}

// Its programs are shell scripts.
#[cfg(unix)]
#[test]
fn a_watch_round_checks_then_tests_and_recovers_after_a_fix() {
    let dir = scratch("watch");
    let source = problem(&dir);
    let solution = script(&dir, "max", MAX);
    let args = ["--fast".to_string(), solution.clone()];
    let mut watch = Watch::new(source.clone());
    let round = |watch: &mut Watch, args: &[String]| {
        let mut out = Vec::new();
        let passed = watch.round(args, &mut out);
        (passed, String::from_utf8(out).unwrap())
    };

    let (passed, out) = round(&mut watch, &args);
    assert!(!passed);
    assert!(
        out.starts_with("  check: no errors\n")
            && out.ends_with("  2/3 passed (1 had no expected output)\n  if any valid answer is accepted, add a `@check fn` that reads the output with `read_output()`\n"),
        "{out}"
    );

    // Fix the wrong expected output: the same watch passes.
    std::fs::write(dir.join("d.tests/2.out"), "4\n").unwrap();
    let (passed, out) = round(&mut watch, &args);
    assert!(
        passed && out.ends_with("  3/3 passed (1 had no expected output)\n"),
        "{out}"
    );

    // An error in the source stops the round before the tests.
    std::fs::write(&source, "let n: 1..=5 = read()\nprint(m)\n").unwrap();
    let (passed, out) = round(&mut watch, &args);
    assert!(!passed);
    assert!(
        out.ends_with("  check: 1 error to fix\n") && !out.contains("passed"),
        "{out}"
    );

    // A solution that cannot be run is reported, not failed.
    let missing = ["--fast".to_string(), dir.join("nope").display().to_string()];
    std::fs::write(&source, "let n: 1..=5 = read()\nprint(n)\n").unwrap();
    let (passed, out) = round(&mut watch, &missing);
    assert!(
        passed && out.contains("  test: skipped: there is no file"),
        "{out}"
    );

    let path = source.to_str().unwrap();
    let (code, out, _) = cli(watch::run, &[path, "--once", "--fast", &solution]);
    assert_eq!(code, 0, "{out}");
    let (code, out, _) = cli(watch::run, &[path, "--once", "--json", "--fast", &solution]);
    assert_eq!(code, 0, "{out}");
    let json = cpl_json::parse(&out).expect("one JSON line");
    assert_eq!(json.at(&["passed"]).as_bool(), Some(true), "{out}");
    assert_eq!(json.at(&["test", "total"]).as_i64(), Some(3), "{out}");
    assert_eq!(json.at(&["error"]), &cpl_json::Value::Null);
    std::fs::write(&source, "let n: 1..=5 = read()\nprint(m)\n").unwrap();
    let (code, out, _) = cli(watch::run, &[path, "--once", "--json", "--fast", &solution]);
    assert_eq!(code, 1, "{out}");
    let json = cpl_json::parse(&out).expect("one JSON line");
    let errors = json.at(&["diagnostics"]).as_array().unwrap();
    assert_eq!(errors.len(), 1, "{out}");
    assert_eq!(errors[0].at(&["severity"]).as_str(), Some("error"));
    assert_eq!(errors[0].at(&["start", "line"]).as_i64(), Some(2));
    assert_eq!(json.at(&["test"]), &cpl_json::Value::Null);
    let (code, _, err) = cli(watch::run, &[dir.join("nope").to_str().unwrap(), "--once"]);
    assert_eq!(code, 1);
    assert!(err.contains("there is no file"), "{err}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_whole_contest_from_the_extension_gets_its_own_folder() {
    use crate::new::contest_source;
    let mut scaffold = Scaffold::from_companion(COMPANION).unwrap();
    scaffold.url = Some("https://codeforces.com/contest/1950/problem/D".into());
    let home = Path::new("/contests");
    assert_eq!(contest_source(&scaffold, 1, home), Some("d.cp".into()));
    assert_eq!(contest_source(&scaffold, 7, home), Some("1950/d.cp".into()));
    // Already in the contest's folder: no folder inside it.
    assert_eq!(
        contest_source(&scaffold, 7, &home.join("1950")),
        Some("d.cp".into())
    );
    scaffold.url = Some("https://example.com/round/3".into());
    assert_eq!(contest_source(&scaffold, 7, home), Some("d.cp".into()));
    scaffold.title = None;
    assert_eq!(contest_source(&scaffold, 7, home), None);
}

#[test]
fn listen_takes_a_problem_posted_by_the_extension() {
    use std::io::{Read, Write};
    // Port 0, held from bind to accept, so no other test can take it between.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let client = std::thread::spawn(move || {
        let mut stream = loop {
            match std::net::TcpStream::connect(("127.0.0.1", port)) {
                Ok(stream) => break stream,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        };
        // A stray request first: it is ignored, and the listener keeps waiting.
        stream.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let head = format!(
            "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            COMPANION.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(COMPANION.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    });
    let mut got = Vec::new();
    let mut notes = Vec::new();
    crate::new::listen_on(listener, &mut notes, &mut |text| {
        got.push(Scaffold::from_companion(text)?);
        Ok(())
    })
    .unwrap();
    assert!(client.join().unwrap().starts_with("HTTP/1.1 200"));
    assert_eq!(got, [Scaffold::from_companion(COMPANION).unwrap()]);
    let notes = String::from_utf8(notes).unwrap();
    assert!(
        notes.contains("waiting for Competitive Companion"),
        "{notes}"
    );
    assert!(notes.contains("ignored a request"), "{notes}");
}

#[test]
fn test_judges_memory_against_the_problems_limit() {
    if !cfg!(target_os = "linux") {
        return;
    }
    let dir = scratch("test-memory");
    let source = problem(&dir);
    let tight = Limits {
        time: Duration::from_secs(10),
        memory_kb: 1,
    };
    let hungry = format!("sleep 0.05; {MAX}");
    let results = run_cases(&sh(&hungry), &Judge::Tokens, &cases(&source), tight).unwrap();
    let verdicts: Vec<Option<&str>> = results
        .iter()
        .map(|r| r.verdict.as_ref().map(Verdict::code))
        .collect();
    assert_eq!(verdicts, [Some("MLE"); 3], "{results:?}");
    assert!(
        render(&results).contains("     peak memory "),
        "{}",
        render(&results)
    );
    let roomy = run_cases(
        &sh(&hungry),
        &Judge::Tokens,
        &cases(&source),
        Limits {
            memory_kb: 1 << 30,
            ..tight
        },
    )
    .unwrap();
    assert_eq!(roomy[0].verdict, Some(Verdict::Accepted));
    let path = source.to_str().unwrap();
    let hungry = script(
        &dir,
        "hungry",
        "exec awk 'BEGIN { s = sprintf(\"%64000000s\", \"\"); system(\"sleep 0.1\") } NR == 1'",
    );
    let (_, out, err) = cli(
        test::run,
        &[path, "--fast", &hungry, "--memory-limit", "16"],
    );
    assert_eq!(err, "");
    assert!(out.starts_with("  1     MLE"), "{out}");
    let (_, out, err) = cli(
        test::run,
        &[path, "--fast", &hungry, "--memory-limit", "1024"],
    );
    assert_eq!(err, "");
    assert!(!out.contains("MLE"), "{out}");
}

#[test]
fn test_discovers_sibling_tests_dir_and_ans_files_and_supports_tests_flag() {
    let dir = scratch("sibling-ans");
    let source = dir.join("sol.cp");
    std::fs::write(
        &source,
        "let a: Int = read()\nlet b: Int = read()\nprint(a + b)\n",
    )
    .unwrap();
    let tests = dir.join("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("01.in"), "2 3\n").unwrap();
    std::fs::write(tests.join("01.ans"), "5\n").unwrap();
    std::fs::write(tests.join("02.in"), "10 20\n").unwrap();
    std::fs::write(tests.join("02.ans"), "30\n").unwrap();

    let found = cases(&source);
    assert_eq!(
        found.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["01", "02"]
    );
    assert!(found.iter().all(|c| c.expected.is_some()));

    let path = source.to_str().unwrap();
    let (code, out, err) = cli(test::run, &[path]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("2/2 passed"), "{out}");

    // `--tests DIR` works from another directory too.
    let other_dir = scratch("tests-flag");
    let other_src = other_dir.join("other.cp");
    std::fs::copy(&source, &other_src).unwrap();
    let (code, out, err) = cli(
        test::run,
        &[
            other_src.to_str().unwrap(),
            "--tests",
            tests.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("2/2 passed"), "{out}");

    std::fs::remove_dir_all(dir).unwrap();
    std::fs::remove_dir_all(other_dir).unwrap();
}

#[test]
fn test_runs_multitest_program_against_in_and_ans() {
    let dir = scratch("multitest-ans");
    let source = dir.join("multi.cp");
    std::fs::write(
        &source,
        "@!multitest\nlet n: 1..=100 = read()\nlet a: [0..n]Int = read()\nprint(a.sum())\n",
    )
    .unwrap();
    let tests = dir.join("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("01.in"), "3\n2\n1 2\n3\n4 5 6\n1\n10\n").unwrap();
    std::fs::write(tests.join("01.ans"), "3\n15\n10\n").unwrap();

    let (code, out, err) = cli(test::run, &[source.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("1/1 passed"), "{out}");

    // A wrong answer in one subcase of a multitest is reported clearly.
    std::fs::write(tests.join("02.in"), "2\n2\n1 2\n2\n3 4\n").unwrap();
    std::fs::write(tests.join("02.ans"), "3\n99\n").unwrap();
    let (code, out, err) = cli(test::run, &[source.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("02  WA"), "{out}");
    assert!(out.contains("line 2"), "{out}");
    assert!(out.contains("1/2 passed"), "{out}");

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_multi_number_problem_gets_the_multitest_divider() {
    let json = COMPANION.replace(
        "\"interactive\"",
        "\"testType\": \"multiNumber\", \"interactive\"",
    );
    let scaffold = Scaffold::from_companion(&json).unwrap();
    assert!(scaffold.multitest);
    let source = scaffold.source(None);
    assert!(
        source.ends_with("\n\n@!multitest\nlet n: 1..=2e5 = read()\n"),
        "{source}"
    );
    // A template that already divides the file keeps its own divider.
    let source = scaffold.source(Some("let sv = 1\n@!multitest\nprint(sv)\n"));
    assert_eq!(source.matches("@!multitest").count(), 1, "{source}");
    assert!(!Scaffold::from_companion(COMPANION).unwrap().multitest);
}

/// Companion calls Codeforces' multitest problems `single`; the samples say otherwise.
#[test]
fn a_codeforces_import_whose_samples_count_tests_gets_the_divider() {
    let import = |url: &str, input: &str, output: &str| {
        let json = format!(
            "{{\"name\": \"A\", \"url\": \"{url}\", \"timeLimit\": 1000, \"memoryLimit\": 256, \
             \"testType\": \"single\", \"interactive\": false, \
             \"tests\": [{{\"input\": \"{input}\", \"output\": \"{output}\"}}]}}"
        );
        Scaffold::from_companion(&json).unwrap().multitest
    };
    let cf = "https://codeforces.com/contest/2289/problem/A";
    assert!(import(
        cf,
        "3\\n2\\n1 2\\n1\\n5\\n3\\n1 2 3\\n",
        "3\\n5\\n6\\n"
    ));
    // One line of input after n is not n tests, nor is a single test.
    assert!(!import(cf, "3\\n1 2 3\\n", "6\\n"));
    assert!(!import(cf, "1\\n5\\n", "5\\n"));
    // n numbers on n lines, but one answer.
    assert!(!import(cf, "2\\n4\\n5\\n", "9\\n"));
    assert!(!import(
        "https://atcoder.jp/contests/abc300/tasks/abc300_a",
        "2\\n4\\n5\\n",
        "4\\n5\\n"
    ));
}

#[test]
fn an_interactive_problem_names_the_interactor_in_the_file() {
    let json = COMPANION.replace("\"interactive\": false", "\"interactive\": true");
    let source = Scaffold::from_companion(&json).unwrap().source(None);
    assert!(
        source.contains("// interactive: an `@interactor fn judge(mut sol: Interactor)` here"),
        "{source}"
    );
}

/// An any-answer problem gets a `@check` to finish, commented out so the
/// file runs as before; uncommented, it is the check `cpl test` uses.
#[test]
fn an_any_answer_problem_gets_a_check_to_finish() {
    let scaffold = Scaffold {
        any_answer: true,
        reads: vec!["let n: Int = read()".into()],
        ..Scaffold::default()
    };
    let source = scaffold.source(None);
    assert!(
        source.starts_with("let n: Int = read()\n\n// Any valid answer is accepted."),
        "{source}"
    );
    let check: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix("// "))
        .skip_while(|line| !line.starts_with("@check"))
        .collect();
    assert_eq!(check[..2], ["@check", "fn ok() -> Bool:"], "{source}");
    assert!(
        check.iter().any(|line| line.contains("read_output()")),
        "{source}"
    );
    assert!(!Scaffold::default().source(None).contains("@check"));
}

#[test]
fn fetch_says_where_the_name_goes_when_given_dash_o() {
    let args: Vec<String> = [
        "https://codeforces.com/contest/2272/problem/A",
        "-o",
        "cf2272A.cp",
    ]
    .map(String::from)
    .to_vec();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(crate::fetch::run(&args, &mut out, &mut err), 2);
    let err = String::from_utf8(err).unwrap();
    assert!(
        err.contains("`cpl fetch https://codeforces.com/contest/2272/problem/A cf2272A`"),
        "{err}"
    );
}

#[test]
fn a_problems_checker_py_judges_its_cases_unless_the_file_declares_one() {
    let python = cpl_stress::program::python();
    let ran = std::process::Command::new(&python[0])
        .args(&python[1..])
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .output();
    if !ran.is_ok_and(|out| out.status.success()) {
        return;
    }
    // The corpus layout: `port.cp`, `tests/` and `checker.py` side by side.
    let dir = std::env::temp_dir().join(format!("cpl-checker-py-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("tests/1.in"), "4\n").unwrap();
    std::fs::write(dir.join("tests/1.out"), "1 3\n").unwrap();
    // Any two numbers summing to n are a right answer.
    std::fs::write(
        dir.join("checker.py"),
        "import sys\nn = int(open(sys.argv[1]).read())\n\
         got = list(map(int, open(sys.argv[2]).read().split()))\n\
         sys.exit(0 if len(got) == 2 and sum(got) == n else 1)\n",
    )
    .unwrap();
    let port = dir.join("port.cp");
    let port = port.to_str().unwrap();
    std::fs::write(port, "let n: 2..=100 = read()\nprint(n - 2, 2)\n").unwrap();
    let (code, out, _) = cli(test::run, &[port]);
    assert_eq!(code, 0, "{out}");
    assert!(out.ends_with("  1/1 passed\n"), "{out}");
    // `@check` in the file wins. `cpl judge` runs it, which in a test would
    // be this test binary, so check which judge is picked instead.
    let declared = "let n: 2..=100 = read()\nprint(n - 2, 2)\n\n\
         @check\nfn check():\n    let a: Int = read_output()\n    assert a == 1\n";
    std::fs::write(port, declared).unwrap();
    let args =
        cpl_stress::args::Args::parse("test", &[port.to_string()], test::VALUED, &[]).unwrap();
    match test::ready(Path::new(port), declared, &args).map(|ready| ready.judge) {
        Ok(Judge::Checker(program)) => {
            assert_eq!(program.argv[1..], ["judge".to_string(), port.to_string()])
        }
        other => panic!("the declared checker was not used: {other:?}"),
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `cpl run` traps on an overflow (R0202); a local build of the same file
/// must trap too, in `cpl test` and in stress's solution, brute and generator.
#[test]
fn test_and_stress_trap_on_overflow_as_run_does() {
    let dir = std::env::temp_dir().join(format!("cpl-overflow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("main.tests")).unwrap();
    let port = dir.join("main.cp");
    std::fs::write(
        &port,
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\nprint(a * b)\n",
    )
    .unwrap();
    let input = "1000000000000 1000000000\n";
    std::fs::write(dir.join("main.tests/1.in"), input).unwrap();
    let ran = Program::resolve(&port)
        .unwrap()
        .run(input, Duration::from_secs(20))
        .unwrap();
    assert!(
        ran.stdout.is_empty() && ran.stderr.contains("overflow"),
        "{ran:?}"
    );
    let (code, out, _) = cli(test::run, &[port.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("RE") && !out.contains("passed\n") || out.contains("0/1"),
        "{out}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
