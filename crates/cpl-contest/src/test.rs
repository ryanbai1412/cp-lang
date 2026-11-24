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
    cases_in, find_checker, find_tests_dir, source_path, tests_dir, Limits, TestCase,
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
N.ans; without one, the output is shown instead of judged.

options:
  --tests DIR        read cases from DIR instead of <problem>.tests/
  --case NAME        run only the case NAME (`2` for 2.in)
  --add FILE         add FILE (`-`: standard input) as the next numbered case,
                     then run it; with --expect OUT, OUT is its answer
  --fast PROG        run this instead of building the problem
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts
  --time-limit MS    per case (default: the problem's limit)
  --memory-limit MB  peak memory per case, past which a case is MLE (default:
                     the problem's limit)
  --format json      print `{\"cases\": [...], \"passed\", \"total\"}`, each case
                     with its name, verdict (null when unjudged), ms, peak_kb,
                     expected, output, why and explained
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
            ("ms", Value::Int(r.elapsed.as_millis() as i64)),
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
        ("passed", Value::Int(passed as i64)),
        ("total", Value::Int(results.len() as i64)),
    ])
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
    // The problem's `checker.py`, unless a flag or the file names a judge.
    let found = (path("--checker").is_none()
        && path("--interactor").is_none()
        && cpl_stress::judge::declared(src).is_empty())
    .then(|| find_checker(source))
    .flatten();
    let checker = path("--checker").or(found.as_deref());
    let judge = select_judge(src, &name, checker, path("--interactor"))?;
    let mut limits = Limits::of(source, src);
    if let Some(ms) = args.number("--time-limit")? {
        limits.time = Duration::from_millis(ms);
    }
    if let Some(mb) = args.number("--memory-limit")? {
        limits.memory_kb = mb * 1024;
    }
    Ok(Ready {
        solution: solution(source, path("--fast"))?,
        judge,
        limits,
    })
}

/// `cpl test <folder>`: every problem in a contest folder with cases, one
/// after another; exits with the worst problem's code.
fn run_folder(
    folder: &Path,
    rest: Vec<String>,
    json: bool,
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
        if json {
            rows.push(format!(
                "{{\"path\":{},\"report\":{}}}",
                cpl_json::escape_string(&source.shown()),
                if out.trim().is_empty() {
                    "null"
                } else {
                    out.trim()
                }
            ));
        } else {
            let _ = writeln!(stdout, "{}:", source.shown());
            let _ = write!(stdout, "{out}");
        }
    }
    let total = sources.len();
    if json {
        let _ = writeln!(
            stdout,
            "{{\"problems\":[{}],\"passed\":{passed},\"total\":{total}}}",
            rows.join(",")
        );
    } else {
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
        &[],
    )
    .and_then(|args| {
        let json = match args.value("--format") {
            None | Some("text") => false,
            Some("json") => true,
            Some(other) => return Err(format!("`--format` is text or json, not `{other}`")),
        };
        let file = args.file("test")?.to_string();
        Ok((args, file, json))
    });
    let (args, source, json) = match parsed {
        Ok((args, file, json)) if Path::new(&file).is_dir() => {
            if args.value("--case").is_some() || args.value("--add").is_some() {
                let message =
                    "`--case` and `--add` pick one problem: name its file, not the folder";
                let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
                return 2;
            }
            return run_folder(
                Path::new(&file),
                args_without(raw.to_vec(), &file),
                json,
                stdout,
                stderr,
            );
        }
        Ok((args, file, json)) => (args, source_path(&file), json),
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let result = std::fs::read_to_string(&source)
        .map_err(|e| format!("can't read {}: {e}", source.display()))
        .and_then(|src| {
            let dir = args
                .value("--tests")
                .map(PathBuf::from)
                .unwrap_or_else(|| find_tests_dir(&source));
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
            let mut cases = cases_in(&dir);
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
            let ready = ready(&source, &src, &args)?;
            let mut results = run_cases(&ready.solution, &ready.judge, &cases, ready.limits)
                .map_err(|e| format!("can't run {}: {e}", ready.solution.name))?;
            if args.value("--fast").is_none() {
                explain_runtime_errors(&mut results, &cases, &source);
            }
            Ok(results)
        });
    match result {
        Ok(results) => {
            if json {
                let _ = writeln!(stdout, "{}", render_json(&results));
            } else {
                let _ = write!(stdout, "{}", render(&results));
            }
            results.iter().any(|r| !r.passed()) as u8
        }
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}
