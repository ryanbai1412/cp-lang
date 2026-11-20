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

use crate::args::Args;
use crate::problem::{cases, source_path, Limits, TestCase};
use cpl_stress::cli::{select_judge, solution};
use cpl_stress::{Judge, Program, Verdict};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl test <problem> [options]

Runs the solution on every case in <problem>.tests/. A case is N.in with
an optional N.out; without one, the output is shown instead of judged.

options:
  --fast PROG        run this instead of building the problem
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts
  --time-limit MS    per case (default: the problem's limit)
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
    limit: Duration,
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
            let judged = judge.run(solution, &input, expected.as_deref(), limit)?;
            // Without an expected output a token judge has nothing to say
            // beyond "it ran".
            let unjudged =
                expected.is_none() && *judge == Judge::Tokens && judged.verdict.accepted();
            Ok(CaseResult {
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
            Some(Verdict::RuntimeError(why)) => out.push_str(&format!("     {why}\n")),
            Some(Verdict::Accepted | Verdict::TimeLimit) => {}
        }
    }
    let passed = results.iter().filter(|r| r.passed()).count();
    out.push_str(&format!("  {passed}/{} passed\n", results.len()));
    out
}

/// A problem ready to run: its solution, judge, and limits.
pub(crate) struct Ready {
    pub solution: Program,
    pub judge: Judge,
    pub limits: Limits,
}

/// Resolves what the test, bench and watch commands all need.
pub(crate) fn ready(source: &Path, src: &str, args: &Args) -> Result<Ready, String> {
    let name = source.display().to_string();
    let path = |flag: &str| args.value(flag).map(Path::new);
    if path("--checker").is_some() && path("--interactor").is_some() {
        return Err("a problem has a checker or an interactor, not both".to_string());
    }
    let judge = select_judge(src, &name, path("--checker"), path("--interactor"))?;
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

/// Runs `cpl test`. Returns the exit code: 0 when every case passed.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let parsed = Args::parse(
        args,
        &["--fast", "--checker", "--interactor", "--time-limit"],
        &[],
    )
    .and_then(|args| {
        args.file("test")
            .map(source_path)
            .map(|source| (args, source))
    });
    let (args, source) = match parsed {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = write!(stderr, "error: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let result = std::fs::read_to_string(&source)
        .map_err(|e| format!("can't read {}: {e}", source.display()))
        .and_then(|src| {
            let cases = cases(&source);
            if cases.is_empty() {
                return Err(format!(
                    "{} has no cases; put `1.in` and `1.out` there",
                    crate::problem::tests_dir(&source).display()
                ));
            }
            let ready = ready(&source, &src, &args)?;
            run_cases(&ready.solution, &ready.judge, &cases, ready.limits.time)
                .map_err(|e| format!("can't run {}: {e}", ready.solution.name))
        });
    match result {
        Ok(results) => {
            let _ = write!(stdout, "{}", render(&results));
            results.iter().any(|r| !r.passed()) as u8
        }
        Err(message) => {
            let _ = writeln!(stderr, "error: {message}");
            1
        }
    }
}
