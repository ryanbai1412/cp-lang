//! Deciding whether an output is right.
//!
//! Most problems have one answer, and outputs are compared token by
//! token. A problem with many valid answers has a checker, and an
//! interactive one has an interactor; both follow the usual judge
//! conventions, so a testlib checker works unchanged:
//!
//! - `checker <input> <output> <answer>` exits 0 to accept.
//! - `interactor <input>` talks to the solution over stdin and stdout and
//!   exits 0 to accept.

use crate::program::{Exit, Outcome, Program};
use cpl_syntax::ast::StmtKind;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// How outputs are judged.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Judge {
    /// The output must equal the expected one, ignoring whitespace.
    Tokens,
    Checker(Program),
    Interactor(Program),
}

/// The result of one case.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    Accepted,
    /// The output is wrong; the text says how.
    Wrong(String),
    /// The solution exited with an error or crashed.
    RuntimeError(String),
    TimeLimit,
    /// The solution's peak memory went past the problem's limit.
    MemoryLimit,
    /// The brute, not the solution, failed on the case: why.
    BruteError(String),
    /// The `checker` or `interactor`, not the solution, failed on the
    /// case: its trap.
    JudgeError(&'static str, String),
}

impl Verdict {
    pub fn accepted(&self) -> bool {
        *self == Verdict::Accepted
    }

    /// The two letters judges use.
    pub fn code(&self) -> &'static str {
        match self {
            Verdict::Accepted => "ok",
            Verdict::Wrong(_) => "WA",
            Verdict::RuntimeError(_) => "RE",
            Verdict::TimeLimit => "TLE",
            Verdict::MemoryLimit => "MLE",
            Verdict::BruteError(_) => "brute RE",
            Verdict::JudgeError("checker", _) => "checker RE",
            Verdict::JudgeError(..) => "interactor RE",
        }
    }
}

/// Whether a checker's or interactor's trap with `code` is a bug in it, not
/// its verdict on the solution: an index, overflow, range, `None`, `todo`
/// or stack trap. A failed `assert`, a `false` from a `-> Bool` check and
/// output that doesn't read as declared all reject the solution.
pub fn judge_bug(code: &str) -> bool {
    ["R01", "R02", "R03", "R06"]
        .iter()
        .any(|kind| code.starts_with(kind))
        || code == "R0402"
}

/// The trap a judge's stderr shows, unless it is the judge's verdict.
fn broke(stderr: &str) -> Option<String> {
    let at = stderr.find("error[R")?;
    let code = stderr[at + "error[".len()..].split(']').next()?;
    judge_bug(code).then(|| stderr[at..].trim_end().to_string())
}

/// A `@check` or `@interactor` a source file declares.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Declared {
    /// `checker` or `interactor`.
    pub kind: &'static str,
    /// The fn it decorates.
    pub name: String,
    /// 1-based.
    pub line: u32,
}

/// The judges a program declares next to its solution.
pub fn declared(src: &str) -> Vec<Declared> {
    let parsed = cpl_syntax::parse_file(src);
    let lines = cpl_diag::LineIndex::new(src);
    let mut out = Vec::new();
    for (base, item) in &parsed.items {
        for stmt in &item.stmts {
            let StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            for decorator in &stmt.decorators {
                let kind = match decorator.name.text.as_str() {
                    "checker" | "check" => "checker",
                    "interactor" => "interactor",
                    _ => continue,
                };
                out.push(Declared {
                    kind,
                    name: decl.name.text.clone(),
                    line: lines.line_of(decl.name.span.shift(*base).start) + 1,
                });
            }
        }
    }
    out
}

/// Whether two output tokens match. `YES`, `Yes` and `yes` are one answer, as
/// Codeforces (the target judge) accepts yes/no in any case; two numbers,
/// either written with a `.` or an exponent, match within 1e-6 absolute or
/// relative error (spec/tools/cli.md, `cpl test d`).
fn same_token(expected: &str, got: &str) -> bool {
    expected == got
        || ["yes", "no"]
            .iter()
            .any(|word| expected.eq_ignore_ascii_case(word) && got.eq_ignore_ascii_case(word))
        || close_reals(expected, got)
}

fn close_reals(expected: &str, got: &str) -> bool {
    let real = |t: &str| t.contains(['.', 'e', 'E']);
    let number = |t: &str| {
        t.bytes()
            .all(|b| b.is_ascii_digit() || b"+-.eE".contains(&b))
            && t.bytes().any(|b| b.is_ascii_digit())
    };
    if !(number(expected) && number(got) && (real(expected) || real(got))) {
        return false;
    }
    match (expected.parse::<f64>(), got.parse::<f64>()) {
        (Ok(a), Ok(b)) if a.is_finite() && b.is_finite() => {
            let diff = (a - b).abs();
            diff <= 1e-6 || diff <= 1e-6 * a.abs()
        }
        _ => false,
    }
}

fn same_tokens(expected: &[&str], got: &[&str]) -> bool {
    expected.len() == got.len() && expected.iter().zip(got).all(|(a, b)| same_token(a, b))
}

/// Where two outputs first differ, counting tokens.
/// Whether a wrong answer is the toolchain's: the debug interpreter, on the
/// same file and input, prints what the brute expects and the built C++
/// does not (use case stress/U29).
pub fn toolchain_differs(interpreted: &str, expected: &str, got: &str) -> bool {
    first_difference(expected, interpreted).is_none() && first_difference(expected, got).is_some()
}

pub fn first_difference(expected: &str, got: &str) -> Option<String> {
    let mut expected_lines = expected.lines();
    let mut got_lines = got.lines();
    let mut line = 0;
    loop {
        line += 1;
        // Blank lines at the end are not a difference.
        let (a, b) = (expected_lines.next(), got_lines.next());
        let (a_tokens, b_tokens): (Vec<&str>, Vec<&str>) = (
            a.unwrap_or("").split_whitespace().collect(),
            b.unwrap_or("").split_whitespace().collect(),
        );
        if a.is_none() && b.is_none() {
            return None;
        }
        if same_tokens(&a_tokens, &b_tokens) {
            continue;
        }
        // Lines can be split differently and still hold the same tokens.
        let rest_a: Vec<&str> = a_tokens
            .iter()
            .copied()
            .chain(expected_lines.flat_map(str::split_whitespace))
            .collect();
        let rest_b: Vec<&str> = b_tokens
            .iter()
            .copied()
            .chain(got_lines.flat_map(str::split_whitespace))
            .collect();
        if same_tokens(&rest_a, &rest_b) {
            return None;
        }
        let token = rest_a
            .iter()
            .zip(&rest_b)
            .take_while(|(x, y)| same_token(x, y))
            .count();
        let show = |tokens: &[&str]| {
            tokens
                .get(token)
                .map_or("nothing".to_string(), |t| format!("`{t}`"))
        };
        return Some(format!(
            "first difference at line {line}, token {}: expected {}, got {}",
            token + 1,
            show(&rest_a),
            show(&rest_b)
        ));
    }
}

/// A file that is removed when it goes out of scope.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, contents: &str) -> std::io::Result<Scratch> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "cpl-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, contents)?;
        Ok(Scratch(path))
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap_or_default()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// What a failed run of the solution is called.
pub(crate) fn failure(outcome: &Outcome) -> Option<Verdict> {
    match outcome.exit {
        Exit::Code(0) => None,
        Exit::TimedOut => Some(Verdict::TimeLimit),
        Exit::Signal => Some(Verdict::RuntimeError("crashed".to_string())),
        Exit::Code(code) => {
            let last = outcome.stderr.lines().last().unwrap_or("").trim();
            Some(Verdict::RuntimeError(if last.is_empty() {
                format!("exit code {code}")
            } else {
                format!("exit code {code}: {last}")
            }))
        }
    }
}

/// One judged run: the verdict and what the solution printed and cost.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Judged {
    pub verdict: Verdict,
    pub outcome: Outcome,
}

impl Judge {
    /// Runs `solution` on `input` and judges it. `expected` is the
    /// brute's or the saved output; a checker gets it as the answer file,
    /// and with no expected output a token judge accepts any clean run.
    pub fn run(
        &self,
        solution: &Program,
        input: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> std::io::Result<Judged> {
        if let Judge::Interactor(interactor) = self {
            return interact(solution, interactor, input, limit);
        }
        let outcome = solution.run(input, limit)?;
        let verdict = match (failure(&outcome), self) {
            (Some(verdict), _) => verdict,
            (None, Judge::Checker(checker)) => {
                let input_file = Scratch::new("in", input)?;
                let output_file = Scratch::new("out", &outcome.stdout)?;
                let answer_file = Scratch::new("ans", expected.unwrap_or(""))?;
                let checked = checker
                    .with_args(&[input_file.path(), output_file.path(), answer_file.path()])
                    .run("", limit.max(Duration::from_secs(10)))?;
                if checked.ok() {
                    Verdict::Accepted
                } else if let Some(trap) = broke(&checked.stderr) {
                    Verdict::JudgeError("checker", trap)
                } else {
                    let said = format!("{}{}", checked.stderr, checked.stdout);
                    let said = said.lines().next().unwrap_or("").trim();
                    Verdict::Wrong(if said.is_empty() {
                        "the checker rejected the output".to_string()
                    } else {
                        format!("checker: {said}")
                    })
                }
            }
            (None, _) => {
                match expected.and_then(|expected| first_difference(expected, &outcome.stdout)) {
                    Some(difference) => Verdict::Wrong(difference),
                    None => Verdict::Accepted,
                }
            }
        };
        Ok(Judged { verdict, outcome })
    }
}

/// Runs a solution against an interactor: each one's stdout is the
/// other's stdin. The interactor's exit code is the verdict.
fn interact(
    solution: &Program,
    interactor: &Program,
    input: &str,
    limit: Duration,
) -> std::io::Result<Judged> {
    let input_file = Scratch::new("in", input)?;
    let start = Instant::now();
    let mut judge = interactor
        .with_args(&[input_file.path()])
        .to_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // The solution reads what the interactor writes, and the other way.
    let mut contestant = solution
        .to_command()
        .stdin(judge.stdout.take().unwrap())
        .stdout(judge.stdin.take().unwrap())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut timed_out = false;
    let mut reject_deadline: Option<Instant> = None;
    let (solution_status, judge_status) = loop {
        let (a, b) = (contestant.try_wait()?, judge.try_wait()?);
        if let (Some(a), Some(b)) = (a, b) {
            break (a, b);
        }
        // Once the interactor has rejected, a solution still waiting for an
        // answer will never get one; give it a brief window to finish exiting
        // before killing it. When the interactor accepted, wait up to `limit`
        // so scheduling delay under load does not turn an accepted run into RE.
        if let Some(status) = b {
            if a.is_none() && !status.success() {
                reject_deadline.get_or_insert_with(|| Instant::now() + Duration::from_millis(200));
            }
        }
        let expired = start.elapsed() >= limit;
        let grace_over = reject_deadline.is_some_and(|d| Instant::now() >= d);
        if expired || grace_over {
            if expired {
                timed_out = true;
            }
            crate::program::kill_group(contestant.id());
            crate::program::kill_group(judge.id());
            let _ = contestant.kill();
            let _ = judge.kill();
            break (contestant.wait()?, judge.wait()?);
        }
        std::thread::sleep(Duration::from_micros(300));
    };
    let read = |pipe: Option<&mut dyn std::io::Read>| {
        let mut text = String::new();
        if let Some(pipe) = pipe {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    };
    let solution_stderr = read(
        contestant
            .stderr
            .as_mut()
            .map(|p| p as &mut dyn std::io::Read),
    );
    let judge_stderr = read(judge.stderr.as_mut().map(|p| p as &mut dyn std::io::Read));
    let exit = if timed_out {
        Exit::TimedOut
    } else {
        solution_status.code().map_or(Exit::Signal, Exit::Code)
    };
    let outcome = Outcome {
        stdout: String::new(),
        stderr: solution_stderr,
        exit,
        elapsed: start.elapsed(),
        peak_kb: None,
    };
    let verdict = if timed_out {
        Verdict::TimeLimit
    } else if judge_status.success() {
        // The interactor accepted; a crash after the last answer still counts.
        failure(&outcome).unwrap_or(Verdict::Accepted)
    } else {
        let said = judge_stderr.lines().next().unwrap_or("").trim();
        // A judge left with nothing to read after the solution failed (say,
        // over its query limit) only follows from that failure.
        if said.starts_with("the solution's output ended") {
            if let Some(verdict) = failure(&outcome) {
                return Ok(Judged { verdict, outcome });
            }
        }
        if let Some(trap) = broke(&judge_stderr) {
            return Ok(Judged {
                verdict: Verdict::JudgeError("interactor", trap),
                outcome,
            });
        }
        Verdict::Wrong(if said.is_empty() {
            "the interactor rejected the run".to_string()
        } else {
            format!("interactor: {said}")
        })
    };
    Ok(Judged { verdict, outcome })
}

#[cfg(test)]
mod toolchain_tests {
    use super::toolchain_differs;

    #[test]
    fn only_an_interpreter_that_agrees_with_the_brute_blames_the_toolchain() {
        assert!(toolchain_differs("3\n", "3", "4\n"));
        assert!(
            !toolchain_differs("4\n", "3", "4\n"),
            "both builds are wrong: a WA"
        );
        assert!(!toolchain_differs("3", "3", "3\n"), "no difference at all");
    }
}
