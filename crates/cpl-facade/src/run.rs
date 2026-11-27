//! Running a program: one entry point for every tool.

use std::io::Write;

pub use cpl_interp::checkpoint::{Checkpoint, Checkpoints};
use std::sync::{Arc, Mutex};

use crate::session::{Checks, Program, Session};

pub use cpl_interp::profile::Profile as RunProfile;
pub use cpl_interp::trace::Recording;

/// How a run is recorded.
#[derive(Clone, Debug, Default)]
pub struct TraceConfig {
    /// The file's text: steps name the line they ran.
    pub source: String,
    /// Record only while matching fns are running: comma-separated globs
    /// (`*`, `?`), with `!pattern` excluding matching fns.
    pub only_fn: Option<String>,
    /// Multitest cases to record (inclusive 1-based ranges). Empty records all.
    pub test_cases: Vec<(u64, u64)>,
    /// Top-level variable names whose changes trigger steps. Empty records all.
    pub watch: Vec<String>,
    /// Keep only the last `N` steps in a ring buffer.
    pub last_steps: Option<usize>,
    /// Maximum call depth to record (`0` = top-level only).
    pub max_depth: Option<usize>,
    /// Record the full call stack (`(fn_name, line)`) on each step for DAP.
    pub record_frames: bool,
    /// Log each value a named variable is given, in a fn too, in
    /// `Recording::assigned`: what `cpl stress --explain` compares.
    pub assignments: bool,
    /// Stop the run when this call returns: the fn's name and which call
    /// of it, 1-based. What a replay runs.
    pub stop_after_call: Option<(String, usize)>,
    /// Keep up to this many steps instead of what fits in a trace file.
    pub max_steps: Option<usize>,
    /// Skip this many steps before keeping any: a window of a long run.
    pub from_step: Option<usize>,
    /// End the run once the steps are full, when what follows them (its
    /// output, a later trap) is not wanted.
    pub stop_when_full: bool,
}

impl From<TraceConfig> for cpl_interp::trace::TraceConfig {
    fn from(c: TraceConfig) -> cpl_interp::trace::TraceConfig {
        cpl_interp::trace::TraceConfig {
            source: c.source,
            only_fn: c.only_fn,
            test_cases: c.test_cases,
            watch: c.watch,
            last_steps: c.last_steps,
            max_depth: c.max_depth,
            record_frames: c.record_frames,
            assignments: c.assignments,
            stop_after_call: c.stop_after_call,
            max_steps: c.max_steps,
            from_step: c.from_step,
            stop_when_full: c.stop_when_full,
        }
    }
}

impl From<cpl_interp::trace::TraceConfig> for TraceConfig {
    fn from(c: cpl_interp::trace::TraceConfig) -> TraceConfig {
        TraceConfig {
            source: c.source,
            only_fn: c.only_fn,
            test_cases: c.test_cases,
            watch: c.watch,
            last_steps: c.last_steps,
            max_depth: c.max_depth,
            record_frames: c.record_frames,
            assignments: c.assignments,
            stop_after_call: c.stop_after_call,
            max_steps: c.max_steps,
            from_step: c.from_step,
            stop_when_full: c.stop_when_full,
        }
    }
}
pub use cpl_interp::{Choices, More};

/// A program that failed while running.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Trap(cpl_interp::Trap);

impl From<cpl_interp::Trap> for Trap {
    fn from(t: cpl_interp::Trap) -> Trap {
        Trap(t)
    }
}

impl Trap {
    /// Its catalogued `R` code (`spec/diagnostics.md`).
    pub fn code(&self) -> cpl_diag::Code {
        self.0.code()
    }

    pub fn message(&self) -> &str {
        self.0.message()
    }

    pub fn into_message(self) -> String {
        self.0.into_message()
    }

    /// The expression or statement that failed.
    pub fn span(&self) -> cpl_diag::Span {
        self.0.span
    }

    /// The 1-based test case it failed in, under `@!multitest`.
    pub fn case(&self) -> Option<u64> {
        self.0.case
    }

    /// The trap as a diagnostic of the file at `path`, with its call stack.
    pub fn to_diagnostic(&self, path: &str, lines: &cpl_diag::LineIndex) -> cpl_diag::Diagnostic {
        self.0.to_diagnostic(path, lines)
    }
}

/// How much stack a program's calls may take before the run is stopped. There is no limit on
/// how deep calls nest: only on the stack they fill.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Bytes of the interpreter's own stack those calls may use: the program's recursion is
    /// the interpreter's.
    pub stack_bytes: usize,
}

impl From<Limits> for cpl_interp::Limits {
    fn from(l: Limits) -> cpl_interp::Limits {
        cpl_interp::Limits {
            stack_bytes: l.stack_bytes,
        }
    }
}

/// What ends a run before it finishes: a deadline, a budget of loop rounds and calls, or a
/// cancel another thread asks for. Clones share the cancel, so the caller keeps one and hands
/// the run the other.
#[derive(Clone, Debug, Default)]
pub struct Stop(pub(crate) cpl_interp::Stop);

impl Stop {
    /// Nothing but a cancel ends the run.
    pub fn new() -> Stop {
        Stop(cpl_interp::Stop::new())
    }

    /// The run is out of time after `limit`.
    pub fn after(limit: std::time::Duration) -> Stop {
        Stop(cpl_interp::Stop::after(limit))
    }

    /// The run is out of time after about `rounds` loop rounds and calls, where there is no
    /// clock to read.
    pub fn after_rounds(rounds: u64) -> Stop {
        Stop(cpl_interp::Stop::after_rounds(rounds))
    }

    /// Ends the run that holds a clone of this one, as a [`CANCELLED`] trap.
    pub fn cancel(&self) {
        self.0.cancel()
    }
}

/// The message of the trap that ends a run at its deadline.
pub const OUT_OF_TIME: cpl_diag::msgs::Msg = cpl_interp::OUT_OF_TIME;
/// The message of the trap that ends a run its [`Stop`] cancelled.
pub const CANCELLED: cpl_diag::msgs::Msg = cpl_interp::CANCELLED;

/// What a run is given: everything that changes its output and none of
/// what the compiler answers (contract 0).
pub struct RunInput<'a>(pub(crate) cpl_interp::Io<'a>);

impl<'a> RunInput<'a> {
    /// The whole input, known before the program starts.
    pub fn fixed(input: &'a [u8]) -> RunInput<'a> {
        RunInput(cpl_interp::Io::fixed(input))
    }

    /// An input that arrives while the program runs: the solution's side
    /// of an interactive problem.
    pub fn live(more: More<'a>) -> RunInput<'a> {
        RunInput(cpl_interp::Io::live(more))
    }

    /// The judge's side: `input` is the hidden test, and `solution` is
    /// what the solution prints.
    pub fn judge(input: &'a [u8], solution: More<'a>) -> RunInput<'a> {
        RunInput(cpl_interp::Io::judge(input, solution))
    }

    /// With `rand` and `shuffle` drawing from `seed`: one seed, one run.
    pub fn seeded(self, seed: u64) -> RunInput<'a> {
        RunInput(self.0.seeded(seed))
    }

    /// Ended early by `stop`: a deadline, or the caller's cancel.
    pub fn until(self, stop: Stop) -> RunInput<'a> {
        RunInput(self.0.until(stop.0))
    }

    /// At a judge build's `checks`: where they keep no check, the program
    /// does what that build does instead of trapping.
    pub fn at(self, checks: Checks) -> RunInput<'a> {
        RunInput(self.0.at(checks.into()))
    }

    /// With `rand` and `shuffle` writing their choices down in `choices`,
    /// or replaying the ones it holds.
    pub fn choosing(self, choices: Arc<Mutex<Choices>>) -> RunInput<'a> {
        RunInput(self.0.choosing(choices))
    }

    /// Keeping each top-level variable's last value in the recording,
    /// read against the program's `source`, without tracing the run.
    pub fn with_last_values(self, source: &'a str) -> RunInput<'a> {
        RunInput(self.0.with_last_values(source))
    }
}

/// What a run is asked to keep besides its output.
#[derive(Default)]
pub struct Request {
    /// Record the run.
    pub trace: Option<TraceConfig>,
    /// Count how often each statement ran, and the time spent there.
    pub profile: bool,
    /// Keep each top-level variable's last value in the recording, without
    /// tracing the run.
    pub last_values: bool,
}

/// How a run ended and what it left besides its output.
#[derive(Default)]
pub struct Outcome {
    /// `None` when the program ran to its end.
    pub trap: Option<Trap>,
    /// What its `debug(...)`s showed, a line each.
    pub debug: Vec<u8>,
    /// The recording, when the request asked for one.
    pub recording: Option<Recording>,
    /// The profile, when the request asked for one.
    pub profile: Option<RunProfile>,
}

/// A run that took checkpoints, or started from one.
pub struct Checkpointed {
    /// How the run ended.
    pub outcome: Outcome,
    /// The checkpoints it took, in order.
    pub taken: Vec<Checkpoint>,
    /// Every byte of input the run was given: a run from one of its
    /// checkpoints is given this as its input.
    pub sent: Vec<u8>,
}

impl From<cpl_interp::Observed> for Outcome {
    fn from(observed: cpl_interp::Observed) -> Outcome {
        Outcome {
            trap: observed.trap.map(Trap),
            debug: observed.debug,
            recording: observed.recording,
            profile: observed.profile,
        }
    }
}

impl Outcome {
    /// The trap, as the error of a run that had one.
    pub fn result(self) -> Result<(), Trap> {
        self.trap.map_or(Ok(()), Err)
    }

    /// Whether the run was ended by its deadline.
    pub fn out_of_time(&self) -> bool {
        self.trap
            .as_ref()
            .is_some_and(|trap| trap.message() == OUT_OF_TIME.text())
    }
}

impl Session {
    /// Runs `program` on `input`, writing what it prints to `output`.
    pub fn run(
        &self,
        program: &Program,
        input: RunInput,
        output: &mut (dyn Write + Send),
        request: Request,
    ) -> Outcome {
        run(program, input, output, request)
    }
}

/// [`Session::run`]: a built program holds all a run needs. The run has a
/// thread of its own.
pub fn run(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    request: Request,
) -> Outcome {
    let io = match request.last_values {
        true => input.0.with_last_values(program.text()),
        false => input.0,
    };
    if request.profile {
        cpl_interp::run_profiled(&program.ir, io, output).into()
    } else {
        cpl_interp::run_observed(&program.ir, io, output, request.trace.map(Into::into)).into()
    }
}

/// [`run`] asking for nothing but the output: `Err` is the trap.
pub fn run_to_end(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
) -> Result<(), Trap> {
    run(program, input, output, Request::default()).result()
}

/// [`run_to_end`] on the calling thread, within `limits` of its stack: for
/// a host that cannot start a thread (the browser), or a caller that runs
/// many short programs.
pub fn run_within(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    limits: Limits,
) -> Result<(), Trap> {
    cpl_interp::run_with_on_this_thread(&program.ir, input.0, output, limits.into()).map_err(Trap)
}

/// [`run`] recording what `trace` asks for, if anything.
pub fn run_traced(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    trace: Option<TraceConfig>,
) -> Outcome {
    let request = Request {
        trace,
        ..Request::default()
    };
    run(program, input, output, request)
}

/// [`run_traced`] on the calling thread, within `limits` of its stack.
pub fn run_traced_within(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<TraceConfig>,
) -> Outcome {
    cpl_interp::observe_on_this_thread(
        &program.ir,
        input.0,
        output,
        limits.into(),
        trace.map(Into::into),
    )
    .into()
}

/// [`run_within`] taking the checkpoints `want` asks for, or starting from
/// the one it names.
pub fn run_checkpointed(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    limits: Limits,
    want: Checkpoints,
) -> Checkpointed {
    let run = cpl_interp::checkpoint::observe_on_this_thread(
        &program.ir,
        input.0,
        output,
        limits.into(),
        None,
        want,
    );
    Checkpointed {
        outcome: run.observed.into(),
        taken: run.taken,
        sent: run.sent,
    }
}

/// The output of `program` on `input` and how it ended, or `None` when it
/// was still running after `limit`.
pub fn run_for(
    program: &Program,
    input: &[u8],
    limit: std::time::Duration,
) -> Option<(Vec<u8>, Result<(), Trap>)> {
    cpl_interp::run_for(&program.ir, input, limit).map(|(out, end)| (out, end.map_err(Trap)))
}
