//! `cpl test`: every case in `<name>.tests/`.
//!
//! ```text
//! $ cpl test d
//!   1  ok     3 ms
//!   2  WA     2 ms
//!      expected: 7
//!      got:      6
//!      first difference at line 1, token 1: expected `7`, got `6`
//!   stress-48213  ok     2 ms
//!   2/3 passed
//! ```

use crate::problem::{
    cases_in, find_checker, find_grader, find_interactor, find_tests_dir, hidden_cases_in,
    source_path, tests_dir, Limits, TestCase,
};
use cpl_diag::Shown;
use cpl_stress::args::Args;
use cpl_stress::cli::{select_judge, solution};
use cpl_stress::{Judge, Program, Verdict};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl test <problem> [options]

Runs the solution on every case in <problem>.tests/ (or a sibling tests/
folder). A contest folder (`cpl fetch <contest page>`) runs each problem in it
with cases, then says how many pass. A case is N.in with an optional N.out or
N.ans; without one, the output is shown instead of judged. A checker.py or
interactor.py in the cases folder (or beside a sibling tests/) judges each case,
and a grader.cpp there is linked in as --grader links it. A corpus package's
cases are made first from its tests.toml, into target/corpus-tests/<id>/.

options:
  --tests DIR        read cases from DIR instead of <problem>.tests/
  --make             make the cases the package <problem>'s tests.toml lists,
                     into --tests DIR (else target/corpus-tests/<id>/), and
                     run nothing
  --list             print the cases, one name a line, in the order they
                     run, and run nothing; with --format json,
                     `{\"cases\": [{\"name\", \"input\"}]}`
  --case NAME        run only the case NAME (`2` for 2.in)
  --add FILE         add FILE (`-`: standard input) as the next numbered case,
                     then run it; with --expect OUT, OUT is its answer
  --fast PROG        run this instead of building the problem
  --grader FILE      link the judge's grader FILE (a .cpp with the `main`,
                     beside the headers it includes) into the build: how a
                     program of `@grader` fns runs
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts; with a
                     `# interactor: spawn` comment, `PROG <input> -- <solution>`
                     runs the solution itself
  --time-limit MS    per case (default: the problem's limit)
  --memory-limit MB  peak memory per case, past which a case is MLE (default:
                     the problem's limit)
  --format json      print `{\"cases\": [...], \"passed_count\",
                     \"case_count\"}`, each case with its name, verdict (null
                     when unjudged), time_ms, peak_kb, expected, output, why
                     and explained
  --format teamcity  print one TeamCity service message per case event, as
                     IntelliJ's test tree and CI servers read them: each case
                     located at its `.in` file, a wrong answer as an
                     expected/actual comparison
";

/// What happened on one case.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CaseResult {
    pub name: String,
    /// `None` when there was nothing to judge against: the case has no
    /// `.out` and the problem no judge.
    pub verdict: Option<Verdict>,
    pub elapsed: Duration,
    pub peak_kb: Option<u64>,
    pub expected: Option<String>,
    pub output: String,
    /// For a runtime error: what the debug interpreter reports on the same
    /// input, with the source line.
    pub explained: Option<String>,
}

impl CaseResult {
    pub fn passed(&self) -> bool {
        self.verdict.as_ref().is_none_or(Verdict::accepted)
    }
}

/// Runs `solution` on each case.
pub fn run_cases(
    solution: &Program,
    judge: &Judge,
    cases: &[TestCase],
    limits: Limits,
) -> std::io::Result<Vec<CaseResult>> {
    cases
        .iter()
        .map(|case| {
            let input = std::fs::read_to_string(&case.input)?;
            let expected = case
                .expected
                .as_ref()
                .map(std::fs::read_to_string)
                .transpose()?;
            let mut judged = judge.run(solution, &input, expected.as_deref(), limits.time)?;
            // A judge stops a program at its memory limit, whatever it
            // would have printed.
            if judged.verdict != Verdict::TimeLimit
                && judged
                    .outcome
                    .peak_kb
                    .is_some_and(|kb| kb > limits.memory_kb)
            {
                judged.verdict = Verdict::MemoryLimit;
            }
            // Without an expected output a token judge has nothing to say
            // beyond "it ran".
            let unjudged =
                expected.is_none() && *judge == Judge::Tokens && judged.verdict.accepted();
            Ok(CaseResult {
                explained: None,
                name: case.name.clone(),
                verdict: (!unjudged).then_some(judged.verdict),
                elapsed: judged.outcome.elapsed,
                peak_kb: judged.outcome.peak_kb,
                expected,
                output: judged.outcome.stdout,
            })
        })
        .collect()
}

/// Fills in `explained` for each runtime error: the `.cp` solution `source`
/// run on that case's input in the debug interpreter.
pub fn explain_runtime_errors(results: &mut [CaseResult], cases: &[TestCase], source: &Path) {
    for (result, case) in results.iter_mut().zip(cases) {
        if matches!(result.verdict, Some(Verdict::RuntimeError(_))) {
            if let Ok(input) = std::fs::read_to_string(&case.input) {
                result.explained = cpl_stress::explain::runtime_error(source, &input);
            }
        }
    }
}

/// At most `max` lines of `text`, each after `prefix`.
fn block(prefix: &str, text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let pad = " ".repeat(prefix.len());
    let mut out = String::new();
    for (index, line) in lines.iter().take(max).enumerate() {
        out.push_str(if index == 0 { prefix } else { &pad });
        out.push_str(line);
        out.push('\n');
    }
    if lines.is_empty() {
        out.push_str(&format!("{prefix}(nothing)\n"));
    } else if lines.len() > max {
        out.push_str(&format!("{pad}... ({} more lines)\n", lines.len() - max));
    }
    out
}

/// One line per passing case; failures expand.
pub fn render(results: &[CaseResult]) -> String {
    let width = results.iter().map(|r| r.name.len()).max().unwrap_or(0);
    let mut out = String::new();
    for result in results {
        let code = result.verdict.as_ref().map_or("ran", Verdict::code);
        out.push_str(&format!(
            "  {:width$}  {code:<4} {:>4} ms\n",
            result.name,
            result.elapsed.as_millis()
        ));
        match &result.verdict {
            None => out.push_str(&block("     output: ", &result.output, 10)),
            Some(Verdict::Wrong(why)) => {
                if let Some(expected) = &result.expected {
                    out.push_str(&block("     expected: ", expected, 10));
                }
                out.push_str(&block("     got:      ", &result.output, 10));
                out.push_str(&format!("     {why}\n"));
            }
            Some(Verdict::RuntimeError(why)) => match &result.explained {
                Some(explained) => out.push_str(&block("     ", explained, 12)),
                None => out.push_str(&format!("     {why}\n")),
            },
            Some(Verdict::MemoryLimit) => {
                if let Some(kb) = result.peak_kb {
                    out.push_str(&format!("     peak memory {} MB\n", kb.div_ceil(1024)));
                }
            }
            Some(
                Verdict::Accepted
                | Verdict::TimeLimit
                | Verdict::BruteError(_)
                | Verdict::JudgeError(..),
            ) => {}
        }
    }
    let passed = results.iter().filter(|r| r.passed()).count();
    out.push_str(&format!("  {passed}/{} passed", results.len()));
    // A case with nothing to judge against passes without being checked.
    match results.iter().filter(|r| r.verdict.is_none()).count() {
        0 => {}
        1 if results.len() == 1 => out.push_str(" (it had no expected output)"),
        unchecked => out.push_str(&format!(" ({unchecked} had no expected output)")),
    }
    out.push('\n');
    let compared = |r: &CaseResult| matches!(&r.verdict, Some(Verdict::Wrong(why)) if why.starts_with("first difference"));
    if results.iter().any(compared) {
        out.push_str("  if any valid answer is accepted, add a `@check fn` that reads the output with `read_output()`\n");
    }
    out
}

/// The results as one JSON object, for editors and scripts.
pub fn render_json(results: &[CaseResult]) -> String {
    cpl_json::to_string(&results_json(results))
}

/// [`render_json`]'s object: `cases`, `passed` and `total`.
pub fn results_json(results: &[CaseResult]) -> cpl_json::Value {
    use cpl_json::{obj, Value};
    let text = |s: &str| Value::String(s.to_string());
    let maybe = |s: Option<&str>| s.map_or(Value::Null, text);
    let cases = results.iter().map(|r| {
        let why = match &r.verdict {
            Some(Verdict::Wrong(why) | Verdict::RuntimeError(why)) => Some(why.as_str()),
            _ => None,
        };
        obj([
            ("name", text(&r.name)),
            ("verdict", maybe(r.verdict.as_ref().map(Verdict::code))),
            ("time_ms", Value::Int(r.elapsed.as_millis() as i64)),
            (
                "peak_kb",
                r.peak_kb.map_or(Value::Null, |kb| Value::Int(kb as i64)),
            ),
            ("expected", maybe(r.expected.as_deref())),
            ("output", text(&r.output)),
            ("why", maybe(why)),
            ("explained", maybe(r.explained.as_deref())),
        ])
    });
    let passed = results.iter().filter(|r| r.passed()).count();
    obj([
        ("cases", Value::Array(cases.collect())),
        ("passed_count", Value::Int(passed as i64)),
        ("case_count", Value::Int(results.len() as i64)),
    ])
}

/// One TeamCity service message (`##teamcity[KIND name='value' ...]`) and its newline.
fn service_message(kind: &str, attrs: &[(&str, &str)]) -> String {
    let mut out = format!("##teamcity[{kind}");
    for (key, value) in attrs {
        let mut escaped = String::new();
        for c in value.chars() {
            match c {
                '|' | '\'' | '[' | ']' => escaped.extend(['|', c]),
                '\n' => escaped.push_str("|n"),
                '\r' => escaped.push_str("|r"),
                '\u{85}' => escaped.push_str("|x"),
                '\u{2028}' => escaped.push_str("|l"),
                '\u{2029}' => escaped.push_str("|p"),
                _ => escaped.push(c),
            }
        }
        out.push_str(&format!(" {key}='{escaped}'"));
    }
    out.push_str("]\n");
    out
}

/// The results as TeamCity service messages, for IntelliJ's test tree and CI
/// servers: one test per case located at `DIR/NAME.in`, its output, and a
/// failure that is an expected/actual comparison when the answer is known.
pub fn render_teamcity(results: &[CaseResult], dir: &Path) -> String {
    let dir = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
    let mut out = String::new();
    for r in results {
        let name = r.name.as_str();
        let input = dir.join(format!("{name}.in")).display().to_string();
        let hint = format!("file://{}", input.replace('\\', "/"));
        out += &service_message("testStarted", &[("name", name), ("locationHint", &hint)]);
        if !r.output.is_empty() {
            out += &service_message("testStdOut", &[("name", name), ("out", &r.output)]);
        }
        if let Some(verdict) = r.verdict.as_ref().filter(|_| !r.passed()) {
            let why = match verdict {
                Verdict::Wrong(why) | Verdict::RuntimeError(why) => Some(why.as_str()),
                _ => None,
            };
            let message = [Some(verdict.code()), why, r.explained.as_deref()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(": ");
            let mut attrs = vec![("name", name), ("message", message.as_str())];
            if let (Verdict::Wrong(_), Some(expected)) = (verdict, r.expected.as_deref()) {
                attrs.extend([
                    ("type", "comparisonFailure"),
                    ("expected", expected),
                    ("actual", &r.output),
                ]);
            }
            out += &service_message("testFailed", &attrs);
        }
        let ms = r.elapsed.as_millis().to_string();
        out += &service_message("testFinished", &[("name", name), ("duration", &ms)]);
    }
    out
}

/// What `cpl test --format` prints.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Report {
    Text,
    Json,
    TeamCity,
}

/// A problem ready to run: its solution, judge, and limits.
pub(crate) struct Ready {
    pub solution: Program,
    pub judge: Judge,
    pub limits: Limits,
}

/// Resolves what the test, bench and watch commands all need.
/// The options of `cpl test` (and `cpl watch`) that take a value; the
/// first three name programs.
pub(crate) const VALUED: &[&str] = &[
    "--tests",
    "--fast",
    "--checker",
    "--interactor",
    "--time-limit",
    "--memory-limit",
    "--grader",
];

/// Writes the next numbered case into `dir`: `from` (a file, or `-` for
/// standard input) as `N.in` and, with `expect`, that file as `N.out`.
fn add_case(dir: &Path, from: &str, expect: Option<&str>) -> Result<String, String> {
    let read = |path: &str| {
        let mut bytes = Vec::new();
        match path {
            "-" => std::io::Read::read_to_end(&mut std::io::stdin(), &mut bytes).map(|_| bytes),
            path => std::fs::read(path),
        }
        .map_err(|e| format!("can't read `{path}`: {e}"))
    };
    let input = read(from)?;
    let expected = expect.map(read).transpose()?;
    let numbers = cases_in(dir)
        .into_iter()
        .filter_map(|c| c.name.parse::<u64>().ok());
    let name = (numbers.max().unwrap_or(0) + 1).to_string();
    let write = |ext: &str, bytes: &[u8]| {
        std::fs::create_dir_all(dir)
            .and_then(|()| std::fs::write(dir.join(format!("{name}.{ext}")), bytes))
            .map_err(|e| format!("can't write to {}: {e}", dir.display()))
    };
    write("in", &input)?;
    if let Some(expected) = expected {
        write("out", &expected)?;
    }
    Ok(name)
}

pub(crate) fn ready(source: &Path, src: &str, args: &Args) -> Result<Ready, String> {
    let name = source.display().to_string();
    let path = |flag: &str| args.value(flag).map(Path::new);
    if path("--checker").is_some() && path("--interactor").is_some() {
        return Err("a problem has a checker or an interactor, not both".to_string());
    }
    // The problem's `interactor.py` or `checker.py`, unless a flag or the
    // file names a judge.
    let package = (path("--checker").is_none()
        && path("--interactor").is_none()
        && cpl_stress::judge::declared(src).is_empty())
    .then(|| (find_interactor(source), find_checker(source)));
    let (found_interactor, found_checker) = package.unwrap_or_default();
    if let (Some(interactor), Some(checker)) = (&found_interactor, &found_checker) {
        return Err(format!(
            "a problem has a checker or an interactor, not both: {} and {}",
            checker.display(),
            interactor.display()
        ));
    }
    let checker = path("--checker").or(found_checker.as_deref());
    let interactor = path("--interactor").or(found_interactor.as_deref());
    let judge = select_judge(src, &name, checker, interactor)?;
    let mut limits = Limits::of(source, src);
    if let Some(ms) = args.number("--time-limit")? {
        limits.time = Duration::from_millis(ms);
    }
    if let Some(mb) = args.number("--memory-limit")? {
        limits.memory_kb = mb * 1024;
    }
    if path("--fast").is_some() && path("--grader").is_some() {
        return Err("`--fast` runs a program as it is, so it takes no `--grader`".to_string());
    }
    let grader = match path("--grader") {
        Some(grader) => Some(grader.to_path_buf()),
        None if path("--fast").is_none() => find_grader(source, src),
        None => None,
    };
    let solution = match grader {
        Some(grader) => Program::resolve_with_grader(source, &grader).map_err(|e| e.to_string())?,
        None => solution(source, path("--fast"))?,
    };
    Ok(Ready {
        solution,
        judge,
        limits,
    })
}

/// `cpl test <folder>`: every problem in a contest folder with cases, one
/// after another; exits with the worst problem's code.
fn run_folder(
    folder: &Path,
    rest: Vec<String>,
    report: Report,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut sources: Vec<PathBuf> = std::fs::read_dir(folder)
        .map(|entries| entries.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    sources.retain(|p| p.extension().is_some_and(|e| e == "cp") && tests_dir(p).is_dir());
    sources.sort();
    if sources.is_empty() {
        let _ = writeln!(
            stderr,
            "cpl: {} has no problem with cases (a `.cp` file with its `.tests/` folder)",
            folder.shown()
        );
        return 1;
    }
    let (mut worst, mut passed, mut rows) = (0, 0, Vec::new());
    for source in &sources {
        let mut problem_args = vec![source.to_string_lossy().into_owned()];
        problem_args.extend(rest.iter().cloned());
        let mut out = Vec::new();
        let code = run(&problem_args, &mut out, stderr);
        worst = worst.max(code);
        passed += usize::from(code == 0);
        let out = String::from_utf8_lossy(&out);
        if report == Report::TeamCity {
            let suite = source.shown();
            let _ = write!(
                stdout,
                "{}",
                service_message("testSuiteStarted", &[("name", &suite)])
            );
            let _ = write!(stdout, "{out}");
            let _ = write!(
                stdout,
                "{}",
                service_message("testSuiteFinished", &[("name", &suite)])
            );
        } else if report == Report::Json {
            let report = Some(out.trim()).filter(|report| !report.is_empty());
            rows.push(cpl_json::write_with(|w| {
                w.object().key("path").str(&source.shown());
                w.key("report").opt_json(report).end();
            }));
        } else {
            let _ = writeln!(stdout, "{}:", source.shown());
            let _ = write!(stdout, "{out}");
        }
    }
    let total = sources.len();
    if report == Report::Json {
        let json = cpl_json::write_with(|w| {
            w.object().key("problems").array();
            for row in &rows {
                w.json(row);
            }
            w.end().key("passed_count").int(passed);
            w.key("problem_count").int(total).end();
        });
        let _ = writeln!(stdout, "{json}");
    } else if report == Report::Text {
        let _ = writeln!(stdout, "{passed} of {total} problems pass every case");
    }
    worst
}

/// `args` again, as strings, without the positional `file`.
fn args_without(args: Vec<String>, file: &str) -> Vec<String> {
    let mut rest = args;
    if let Some(at) = rest.iter().position(|a| a == file) {
        rest.remove(at);
    }
    rest
}

/// `cpl test --make`: makes the cases of `package` (or of the package a
/// file is in) into `out`, else its `target/corpus-tests/<id>/`.
fn make_cases(
    package: &Path,
    out: Option<&str>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let package = match package.is_dir() {
        true => package,
        false => package.parent().unwrap_or(Path::new(".")),
    };
    let out = out.map_or_else(|| crate::package::cases_dir(package), PathBuf::from);
    match crate::package::make(package, &out) {
        Ok(()) => {
            let _ = writeln!(stdout, "{}", out.display());
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}

/// `cpl test --list`: the problem's cases in the order `cpl test` runs
/// them, so an editor's test tree lists them as the CLI does.
fn list_cases(file: &str, tests: Option<&str>, json: bool, stdout: &mut dyn Write) -> u8 {
    let dir = tests.map_or_else(|| find_tests_dir(&source_path(file)), PathBuf::from);
    let cases = cases_in(&dir);
    if json {
        let cases = cases.iter().map(|case| {
            cpl_json::obj([
                ("name", case.name.as_str().into()),
                ("input", case.input.to_string_lossy().as_ref().into()),
            ])
        });
        let list = cpl_json::obj([("cases", cpl_json::Value::Array(cases.collect()))]);
        let _ = writeln!(stdout, "{}", cpl_json::to_string(&list));
    } else {
        for case in &cases {
            let _ = writeln!(stdout, "{}", case.name);
        }
    }
    0
}

/// Runs `cpl test`. Returns the exit code: 0 when every case passed.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let raw = args;
    let parsed = Args::parse(
        "test",
        args,
        &[VALUED, &["--format", "--case", "--add", "--expect"]].concat(),
        &["--make", "--list"],
    )
    .and_then(|args| {
        let report = match args.value("--format") {
            None | Some("text") => Report::Text,
            Some("json") => Report::Json,
            Some("teamcity") => Report::TeamCity,
            Some(other) => {
                return Err(format!(
                    "`--format` is text, json or teamcity, not `{other}`"
                ))
            }
        };
        let file = args.file("test")?.to_string();
        Ok((args, file, report))
    });
    if let Ok((args, file, report)) = &parsed {
        if args.has("--make") {
            return make_cases(Path::new(file), args.value("--tests"), stdout, stderr);
        }
        if args.has("--list") {
            let json = matches!(report, Report::Json);
            return list_cases(file, args.value("--tests"), json, stdout);
        }
    }
    let (args, source, report) = match parsed {
        Ok((args, file, report)) if Path::new(&file).is_dir() => {
            if args.value("--case").is_some() || args.value("--add").is_some() {
                let message =
                    "`--case` and `--add` pick one problem: name its file, not the folder";
                let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
                return 2;
            }
            return run_folder(
                Path::new(&file),
                args_without(raw.to_vec(), &file),
                report,
                stdout,
                stderr,
            );
        }
        Ok((args, file, report)) => (args, source_path(&file), report),
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    // A package's cases are made before its cases folder is looked for.
    let made = match source.parent().filter(|p| p.join("tests.toml").is_file()) {
        Some(package) if args.value("--tests").is_none() => {
            crate::package::make(package, &crate::package::cases_dir(package))
        }
        _ => Ok(()),
    };
    let dir = args
        .value("--tests")
        .map(PathBuf::from)
        .unwrap_or_else(|| find_tests_dir(&source));
    let result = made
        .and_then(|()| {
            std::fs::read_to_string(&source)
                .map_err(|e| format!("can't read {}: {e}", source.display()))
        })
        .and_then(|src| {
            let mut only = args.value("--case").map(str::to_string);
            if let Some(from) = args.value("--add") {
                let name = add_case(&dir, from, args.value("--expect"))?;
                let _ = writeln!(
                    stderr,
                    "cpl: added {}",
                    dir.join(format!("{name}.in")).display()
                );
                only = Some(name);
            }
            let ready = ready(&source, &src, &args)?;
            let mut cases = match ready.judge {
                Judge::Interactor(..) => hidden_cases_in(&dir),
                _ => cases_in(&dir),
            };
            if let Some(name) = only.as_deref() {
                let names: Vec<&str> = cases.iter().map(|c| c.name.as_str()).collect();
                if !names.contains(&name) {
                    return Err(format!(
                        "{} has no case `{name}`; it has {}",
                        dir.display(),
                        names.join(", ")
                    ));
                }
                cases.retain(|c| c.name == name);
            }
            if cases.is_empty() {
                return Err(format!(
                    "{} has no cases; put `1.in` and `1.out` (or `1.ans`) there",
                    dir.display()
                ));
            }
            let mut results = run_cases(&ready.solution, &ready.judge, &cases, ready.limits)
                .map_err(|e| format!("can't run {}: {e}", ready.solution.name))?;
            if args.value("--fast").is_none() {
                explain_runtime_errors(&mut results, &cases, &source);
            }
            Ok(results)
        });
    match result {
        Ok(results) => {
            match report {
                Report::Text => _ = write!(stdout, "{}", render(&results)),
                Report::Json => _ = writeln!(stdout, "{}", render_json(&results)),
                Report::TeamCity => _ = write!(stdout, "{}", render_teamcity(&results, &dir)),
            }
            results.iter().any(|r| !r.passed()) as u8
        }
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}
