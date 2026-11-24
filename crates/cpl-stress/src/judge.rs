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

use crate::case::{judge_case, judge_limit, Ran, Ruling, Side, Sides, Talked, Transcript};
use crate::program::{Exit, Outcome, Program};
use crate::run::StressError;
use cpl_syntax::ast::StmtKind;
use std::cell::RefCell;
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
    Interactor(Program, Launch),
}

/// How an interactor meets the solution (spec/tools/cli.md, `--interactor`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Launch {
    /// `PROG <case>`, talking to the solution on its standard input and
    /// output.
    #[default]
    Stdio,
    /// `PROG <case> -- <solution argv>`: the interactor runs the solution
    /// itself, as often as the problem needs (a run-twice problem runs it,
    /// transforms what it printed, and runs it again).
    Spawn,
}

impl Launch {
    /// What the interactor at `path` declares: `Spawn` when a line of its
    /// leading `#` comments is `# interactor: spawn`.
    pub fn declared(path: &std::path::Path) -> Launch {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let spawns = text
            .lines()
            .take_while(|line| line.starts_with('#'))
            .any(|line| line.trim_end() == "# interactor: spawn");
        if spawns {
            Launch::Spawn
        } else {
            Launch::Stdio
        }
    }
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

/// The program a failing case stopped in with an error of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trapped {
    Solution,
    Brute,
    Checker,
    Interactor,
}

impl Trapped {
    pub fn name(self) -> &'static str {
        match self {
            Trapped::Solution => "solution",
            Trapped::Brute => "brute",
            Trapped::Checker => "checker",
            Trapped::Interactor => "interactor",
        }
    }
}

impl Verdict {
    pub fn accepted(&self) -> bool {
        *self == Verdict::Accepted
    }

    /// A wrong answer, as the brute, checker or interactor rules it.
    pub fn wrong(&self) -> bool {
        matches!(self, Verdict::Wrong(_))
    }

    pub fn timed_out(&self) -> bool {
        *self == Verdict::TimeLimit
    }

    /// The solution itself never finished cleanly: a crash or the time limit.
    pub fn stopped(&self) -> bool {
        matches!(self, Verdict::RuntimeError(_) | Verdict::TimeLimit)
    }

    /// Which program's own error the case stopped at: whose trap the
    /// debug run that explains it shows.
    pub fn trapped(&self) -> Option<Trapped> {
        match self {
            Verdict::RuntimeError(_) => Some(Trapped::Solution),
            Verdict::BruteError(_) => Some(Trapped::Brute),
            Verdict::JudgeError("checker", _) => Some(Trapped::Checker),
            Verdict::JudgeError(..) => Some(Trapped::Interactor),
            _ => None,
        }
    }

    /// The brute, checker or interactor failed, not the solution: the case
    /// is no test of it.
    pub fn helper_failed(&self) -> bool {
        self.trapped().is_some_and(|t| t != Trapped::Solution)
    }

    /// What the verdict says of the run, or `""` when it says nothing more.
    pub fn why(&self) -> &str {
        match self {
            Verdict::Wrong(why)
            | Verdict::RuntimeError(why)
            | Verdict::BruteError(why)
            | Verdict::JudgeError(_, why) => why,
            _ => "",
        }
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

/// What an interpreted run that trapped is called.
pub(crate) fn interp_failure(trap: cpl_interp::Trap) -> Verdict {
    match trap.message.as_str() {
        cpl_interp::OUT_OF_TIME => Verdict::TimeLimit,
        _ => Verdict::RuntimeError(trap.message),
    }
}

/// How a run with `@cache` off, `cold`, rules on the cached run's output `got`.
pub(crate) fn recheck(cold: &Outcome, got: &str) -> Option<crate::case::Failed> {
    let why = match cold.ok() {
        false => format!(
            "@cache cold run failed ({:?}) while cached run succeeded",
            cold.exit
        ),
        true => format!(
            "@cache changed the output ({})",
            first_difference(&cold.stdout, got)?
        ),
    };
    let wrong = Verdict::Wrong(why);
    Some(crate::case::Failed::new(
        wrong,
        cold.stdout.clone(),
        got.to_string(),
    ))
}

/// One judged run: the verdict and what the solution printed and cost.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Judged {
    pub verdict: Verdict,
    pub outcome: Outcome,
}

impl Judge {
    /// Runs `solution` on `input` and judges it: by the interactor, by
    /// the checker, or token by token against `expected`.
    pub fn run(
        &self,
        solution: &Program,
        input: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> std::io::Result<Judged> {
        let answered = Answered {
            judge: self,
            solution,
            expected,
            outcome: RefCell::new(None),
            failed: RefCell::new(None),
        };
        let wrong = judge_case(&answered, input, limit).map_err(|e| {
            answered
                .failed
                .take()
                .unwrap_or_else(|| std::io::Error::other(e.0))
        })?;
        let outcome = answered
            .outcome
            .take()
            .ok_or_else(|| std::io::Error::other("no run"))?;
        let verdict = wrong.map_or(Verdict::Accepted, |wrong| wrong.verdict);
        Ok(Judged { verdict, outcome })
    }
}

/// A case whose expected output is known: [`judge_case`] with that output
/// as a brute that has already run.
struct Answered<'a> {
    judge: &'a Judge,
    solution: &'a Program,
    expected: Option<&'a str>,
    outcome: RefCell<Option<Outcome>>,
    failed: RefCell<Option<std::io::Error>>,
}

impl Answered<'_> {
    fn keep<T>(&self, run: std::io::Result<T>) -> Result<T, StressError> {
        run.map_err(|e| {
            let said = StressError(e.to_string());
            self.failed.replace(Some(e));
            said
        })
    }
}

impl Sides for Answered<'_> {
    fn brute(&self, _: &str, _: Duration) -> Result<Option<Ran>, StressError> {
        Ok(self.expected.map(|out| Ran {
            out: out.to_string(),
            stop: None,
        }))
    }

    fn solution(&self, input: &str, limit: Duration) -> Result<Ran, StressError> {
        let (ran, outcome) = self.keep(run(self.solution, input, limit))?;
        self.outcome.replace(Some(outcome));
        Ok(ran)
    }

    fn check(
        &self,
        input: &str,
        got: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> Result<Option<Ruling>, StressError> {
        match self.judge {
            Judge::Checker(checker) => self
                .keep(check(checker, input, got, expected, limit))
                .map(Some),
            _ => Ok(None),
        }
    }

    fn interact(&self, input: &str, limit: Duration) -> Result<Option<Talked>, StressError> {
        let Judge::Interactor(interactor, launch) = self.judge else {
            return Ok(None);
        };
        let (talked, outcome) =
            self.keep(interact(self.solution, interactor, *launch, input, limit))?;
        self.outcome.replace(Some(outcome));
        Ok(Some(talked))
    }
}

/// Runs `program` on `input`: what it printed and why it stopped early.
pub(crate) fn run(
    program: &Program,
    input: &str,
    limit: Duration,
) -> std::io::Result<(Ran, Outcome)> {
    let outcome = program.run(input, limit)?;
    let ran = Ran {
        out: outcome.stdout.clone(),
        stop: failure(&outcome),
    };
    Ok((ran, outcome))
}

/// Runs a testlib-style checker, `checker input output answer`, on `got`.
pub(crate) fn check(
    checker: &Program,
    input: &str,
    got: &str,
    expected: Option<&str>,
    limit: Duration,
) -> std::io::Result<Ruling> {
    let input_file = Scratch::new("in", input)?;
    let output_file = Scratch::new("out", got)?;
    let answer_file = Scratch::new("ans", expected.unwrap_or(""))?;
    let checked = checker
        .with_args(&[input_file.path(), output_file.path(), answer_file.path()])
        .run("", judge_limit(limit))?;
    if checked.ok() {
        return Ok(Ruling::Accepted);
    }
    if let Some(trap) = broke(&checked.stderr) {
        return Ok(Ruling::Broke(trap));
    }
    let said = format!("{}{}", checked.stderr, checked.stdout);
    let said = said.lines().next().unwrap_or("").trim();
    Ok(Ruling::Rejected(
        (!said.is_empty()).then(|| said.to_string()),
    ))
}

/// The interactor's trap as `cpl check --format json` gives it, from a
/// second run that asks the interactor to write it.
pub fn interactor_trap_json(
    solution: &Program,
    interactor: &Program,
    input: &str,
    limit: Duration,
) -> Option<String> {
    let out = Scratch::new("diagnostic", "").ok()?;
    let mut interactor = interactor.clone();
    interactor
        .env
        .push(("CPL_JUDGE_DIAGNOSTIC".to_string(), out.path().to_string()));
    interact(solution, &interactor, Launch::Stdio, input, limit).ok()?;
    let json = std::fs::read_to_string(out.path()).ok()?;
    Some(json.trim().to_string()).filter(|json| !json.is_empty())
}

/// Runs the solution against the interactor, each reading what the other
/// writes; the interactor reads the hidden test from a file. With
/// `Launch::Spawn` the interactor runs the solution itself.
pub(crate) fn interact(
    solution: &Program,
    interactor: &Program,
    launch: Launch,
    input: &str,
    limit: Duration,
) -> std::io::Result<(Talked, Outcome)> {
    if launch == Launch::Spawn {
        return spawned(solution, interactor, input, limit);
    }
    let input_file = Scratch::new("in", input)?;
    let mut judge = interactor
        .with_args(&[input_file.path()])
        .to_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Each side's output goes through here, so the solution is timed only
    // while it has the turn, and the transcript is what was said.
    let mut contestant = match solution
        .to_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(contestant) => contestant,
        Err(e) => {
            crate::program::kill_group(judge.id());
            let _ = judge.kill();
            let _ = judge.wait();
            return Err(e);
        }
    };
    let start = Instant::now();
    let clock = std::sync::Mutex::new(Clock::new(start));
    let transcript = std::sync::Mutex::new(Transcript::default());
    let pipes = (
        piped(judge.stdout.take())?,
        piped(contestant.stdin.take())?,
        piped(contestant.stdout.take())?,
        piped(judge.stdin.take())?,
        piped(contestant.stderr.take())?,
        piped(judge.stderr.take())?,
    );
    let (heard, to_solution, said, to_judge, solution_err, judge_err) = pipes;
    let (statuses, own, timed_out, solution_stdout, solution_stderr, judge_stderr) =
        std::thread::scope(|scope| {
            let (clock, transcript) = (&clock, &transcript);
            scope.spawn(move || {
                relay(heard, to_solution, Side::Interactor, transcript, || {
                    locked(clock).heard()
                })
            });
            let solution_stdout = scope.spawn(move || {
                relay(said, to_judge, Side::Solution, transcript, || {
                    locked(clock).said()
                })
            });
            let solution_stderr = scope.spawn(move || drain(solution_err));
            let judge_stderr = scope.spawn(move || drain(judge_err));
            // A slow interactor that keeps talking is given up on at `wall`.
            let wall = judge_limit(limit) * 3;
            let mut idled = false;
            let mut own = None;
            let mut cpu = None;
            let mut timed_out = false;
            let mut reject_deadline: Option<Instant> = None;
            // The solution is charged its own CPU time, as judges do; where
            // the system doesn't say, the wall clock while it has the turn.
            let spent = |cpu: Option<Duration>| cpu.unwrap_or_else(|| locked(clock).own());
            let statuses = loop {
                if own.is_none() {
                    cpu = crate::program::cpu_time(contestant.id()).or(cpu);
                }
                let (a, b) = (contestant.try_wait()?, judge.try_wait()?);
                if a.is_some() && own.is_none() {
                    own = Some(spent(cpu));
                }
                if let (Some(a), Some(b)) = (a, b) {
                    break (a, b);
                }
                // Once the interactor has rejected, a solution still waiting for an
                // answer will never get one; give it a brief window to finish exiting
                // before killing it. When the interactor accepted, wait up to `limit`
                // so scheduling delay under load does not turn an accepted run into RE.
                if let Some(status) = b {
                    if a.is_none() && !status.success() {
                        reject_deadline
                            .get_or_insert_with(|| Instant::now() + Duration::from_millis(200));
                    }
                }
                // A solution blocked on its turn for twice its limit is idle.
                let busy = own.unwrap_or_else(|| spent(cpu)) > limit;
                idled = locked(clock).own() > limit * 2;
                let expired = busy || idled || start.elapsed() >= wall;
                let grace_over = reject_deadline.is_some_and(|d| Instant::now() >= d);
                if expired || grace_over {
                    timed_out = expired;
                    crate::program::kill_group(contestant.id());
                    crate::program::kill_group(judge.id());
                    let _ = contestant.kill();
                    let _ = judge.kill();
                    break (contestant.wait()?, judge.wait()?);
                }
                std::thread::sleep(Duration::from_micros(300));
            };
            let own = match idled {
                true => locked(clock).own(),
                false => own.unwrap_or_else(|| spent(cpu)),
            };
            std::io::Result::Ok((
                statuses,
                own,
                timed_out,
                solution_stdout.join().unwrap_or_default(),
                solution_stderr.join().unwrap_or_default(),
                judge_stderr.join().unwrap_or_default(),
            ))
        })?;
    let (solution_status, judge_status) = statuses;
    let exit = if timed_out {
        Exit::TimedOut
    } else {
        solution_status.code().map_or(Exit::Signal, Exit::Code)
    };
    let outcome = Outcome {
        stdout: solution_stdout,
        stderr: solution_stderr,
        exit,
        elapsed: own,
        peak_kb: None,
    };
    let (mut talked, outcome) = talked(rule(judge_status.success(), &judge_stderr), outcome);
    talked.transcript = transcript
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok((talked, outcome))
}

/// The solution's own time in an interactive run: the wall clock while it
/// has the turn. It has it from its start, and from each time the
/// interactor writes to it until it writes back. The interactor's launch
/// and its replies are not the solution's time; nor is the wait before the
/// interactor's first line when the interactor speaks first.
struct Clock {
    own: Duration,
    turn: Option<Instant>,
    spoke: bool,
    heard: bool,
}

impl Clock {
    fn new(start: Instant) -> Self {
        Clock {
            own: Duration::ZERO,
            turn: Some(start),
            spoke: false,
            heard: false,
        }
    }

    fn own(&self) -> Duration {
        self.own + self.turn.map_or(Duration::ZERO, |turn| turn.elapsed())
    }

    fn heard(&mut self) {
        match self.spoke || self.heard {
            true => {
                self.turn.get_or_insert_with(Instant::now);
            }
            false => self.turn = Some(Instant::now()),
        }
        self.heard = true;
    }

    fn said(&mut self) {
        self.spoke = true;
        if let Some(turn) = self.turn.take() {
            self.own += turn.elapsed();
        }
    }
}

/// Copies `from` to `to` as it arrives, noting each chunk on the clock
/// first and each line in the transcript, and returns all of it. Once
/// `to` is gone, `from` is still read to its end, so the side writing it
/// never blocks.
fn relay(
    mut from: impl std::io::Read,
    to: impl std::io::Write,
    side: Side,
    transcript: &std::sync::Mutex<Transcript>,
    mut note: impl FnMut(),
) -> String {
    let mut to = Some(to);
    let mut all = Vec::new();
    let mut chunk = vec![0; 1 << 16];
    let mut line = Vec::new();
    while let Ok(read @ 1..) = from.read(&mut chunk) {
        note();
        if let Some(pipe) = to.as_mut() {
            if pipe
                .write_all(&chunk[..read])
                .and_then(|()| pipe.flush())
                .is_err()
            {
                to = None;
            }
        }
        all.extend_from_slice(&chunk[..read]);
        line.extend_from_slice(&chunk[..read]);
        if let Some(end) = line.iter().rposition(|&b| b == b'\n') {
            let text = String::from_utf8_lossy(&line[..=end]).replace("\r\n", "\n");
            locked(transcript).say(side, &text);
            line.drain(..=end);
        }
    }
    drop(to);
    if !line.is_empty() {
        locked(transcript).say(side, &String::from_utf8_lossy(&line));
    }
    String::from_utf8_lossy(&all).replace("\r\n", "\n")
}

fn locked<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A pipe asked for with `Stdio::piped()`.
fn piped<T>(pipe: Option<T>) -> std::io::Result<T> {
    pipe.ok_or_else(|| std::io::Error::other("a pipe asked for was not made"))
}

fn drain(mut pipe: impl std::io::Read) -> String {
    let mut text = Vec::new();
    let _ = pipe.read_to_end(&mut text);
    String::from_utf8_lossy(&text).replace("\r\n", "\n")
}

/// `interactor <case> -- <solution argv>`: the interactor owns the
/// solution's processes, so the case's deadline covers every run it makes,
/// and only its exit and what it says decide.
fn spawned(
    solution: &Program,
    interactor: &Program,
    input: &str,
    limit: Duration,
) -> std::io::Result<(Talked, Outcome)> {
    use std::io::Read;
    let input_file = Scratch::new("in", input)?;
    let mut args = vec![input_file.path(), "--"];
    args.extend(solution.argv.iter().map(String::as_str));
    let mut command = interactor.with_args(&args).to_command();
    command.envs(solution.env.iter().map(|(key, value)| (key, value)));
    let start = Instant::now();
    let mut judge = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let pid = judge.id();
    let mut pipe = piped(judge.stderr.take())?;
    let said = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = pipe.read_to_string(&mut text);
        text
    });
    let (done, exited) = std::sync::mpsc::channel();
    let waiter = std::thread::spawn(move || {
        let status = judge.wait();
        let _ = done.send(());
        status
    });
    let timed_out = exited.recv_timeout(judge_limit(limit)).is_err();
    // Whatever it started and left running ends with it.
    crate::program::kill_group(pid);
    let status = waiter
        .join()
        .map_err(|_| std::io::Error::other("the interactor's waiter panicked"))??;
    let judge_stderr = said.join().unwrap_or_default();
    let outcome = Outcome {
        stdout: String::new(),
        stderr: String::new(),
        exit: if timed_out {
            Exit::TimedOut
        } else {
            Exit::Code(0)
        },
        elapsed: start.elapsed(),
        peak_kb: None,
    };
    Ok(talked(rule(status.success(), &judge_stderr), outcome))
}

/// The interactor's ruling from its exit and the first line it said.
fn rule(success: bool, said: &str) -> Ruling {
    if success {
        Ruling::Accepted
    } else if let Some(trap) = broke(said) {
        Ruling::Broke(trap)
    } else {
        let said = said.lines().next().unwrap_or("").trim();
        Ruling::Rejected((!said.is_empty()).then(|| said.to_string()))
    }
}

fn talked(ruling: Ruling, outcome: Outcome) -> (Talked, Outcome) {
    let talked = Talked {
        solution: Ran {
            out: outcome.stdout.clone(),
            stop: failure(&outcome),
        },
        ruling,
        transcript: Transcript::default(),
        trap: None,
        solution_trap: None,
    };
    (talked, outcome)
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

/// What `--explain` shows of a failing case: one choice, made here for
/// both `cpl stress` and the browser's in-process stress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Explaining {
    /// The `print` whose answer the file's `@interactor` rejected.
    Answer,
    /// The step a run that never finished goes round.
    Stuck,
    /// The first value the solution and its own brute give differently.
    Versus,
}

/// What `--explain` shows of a case failing with `verdict`, for an
/// `interactive` solution or one with a `brute` in its file, or why it
/// shows nothing. A case the brute, checker or interactor failed on is no
/// test of the solution: nothing explains it.
pub fn explaining(
    verdict: &Verdict,
    interactive: bool,
    brute: bool,
) -> Option<Result<Explaining, String>> {
    Some(match verdict {
        _ if verdict.helper_failed() => return None,
        _ if interactive => Ok(Explaining::Answer),
        Verdict::TimeLimit => Ok(Explaining::Stuck),
        _ if brute => Ok(Explaining::Versus),
        _ => Err("`--explain` runs the file against its own `@alt` fns".to_string()),
    })
}
