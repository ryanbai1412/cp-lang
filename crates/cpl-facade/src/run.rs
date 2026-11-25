//! Running a program: one entry point for every tool.

use std::io::Write;

pub use cpl_interp::checkpoint::{Checkpoint, Checkpoints};
use std::sync::{Arc, Mutex};

use crate::session::{Checks, Program, Session};

pub use cpl_interp::profile::Profile as RunProfile;
pub use cpl_interp::trace::{Recording, TraceConfig};
pub use cpl_interp::{Choices, Limits, More, Stop, Trap};

/// The message of the trap that ends a run at its deadline.
pub const OUT_OF_TIME: cpl_diag::msgs::Msg = cpl_interp::OUT_OF_TIME;
/// The message of the trap that ends a run its [`Stop`] cancelled.
pub const CANCELLED: cpl_diag::msgs::Msg = cpl_interp::CANCELLED;

enum Feed<'a> {
    Fixed(&'a [u8]),
    Live(More<'a>),
}

/// What a run is given: everything that changes its output and none of
/// what the compiler answers (contract 0).
pub struct RunInput<'a> {
    feed: Feed<'a>,
    peer: Option<More<'a>>,
    seed: Option<u64>,
    stop: Option<Stop>,
    choices: Option<Arc<Mutex<Choices>>>,
    checks: Checks,
    last_values: Option<&'a str>,
}

impl<'a> RunInput<'a> {
    fn of(feed: Feed<'a>, peer: Option<More<'a>>) -> RunInput<'a> {
        RunInput {
            feed,
            peer,
            seed: None,
            stop: None,
            choices: None,
            checks: Checks::LOCAL,
            last_values: None,
        }
    }

    /// The whole input, known before the program starts.
    pub fn fixed(input: &'a [u8]) -> RunInput<'a> {
        RunInput::of(Feed::Fixed(input), None)
    }

    /// An input that arrives while the program runs: the solution's side
    /// of an interactive problem.
    pub fn live(more: More<'a>) -> RunInput<'a> {
        RunInput::of(Feed::Live(more), None)
    }

    /// The judge's side: `input` is the hidden test, and `solution` is
    /// what the solution prints.
    pub fn judge(input: &'a [u8], solution: More<'a>) -> RunInput<'a> {
        RunInput::of(Feed::Fixed(input), Some(solution))
    }

    /// With `rand` and `shuffle` drawing from `seed`: one seed, one run.
    pub fn seeded(self, seed: u64) -> RunInput<'a> {
        RunInput {
            seed: Some(seed),
            ..self
        }
    }

    /// Ended early by `stop`: a deadline, or the caller's cancel.
    pub fn until(self, stop: Stop) -> RunInput<'a> {
        RunInput {
            stop: Some(stop),
            ..self
        }
    }

    /// At a judge build's `checks`: where they keep no check, the program
    /// does what that build does instead of trapping.
    pub fn at(self, checks: Checks) -> RunInput<'a> {
        RunInput { checks, ..self }
    }

    /// With `rand` and `shuffle` writing their choices down in `choices`,
    /// or replaying the ones it holds.
    pub fn choosing(self, choices: Arc<Mutex<Choices>>) -> RunInput<'a> {
        RunInput {
            choices: Some(choices),
            ..self
        }
    }

    /// Keeping each top-level variable's last value in the recording,
    /// read against the program's `source`, without tracing the run.
    pub fn with_last_values(self, source: &'a str) -> RunInput<'a> {
        RunInput {
            last_values: Some(source),
            ..self
        }
    }

    pub(crate) fn io(self, last_values: Option<&'a str>) -> cpl_interp::Io<'a> {
        let last_values = last_values.or(self.last_values);
        let mut io = match (self.feed, self.peer) {
            (Feed::Fixed(input), Some(peer)) => cpl_interp::Io::judge(input, peer),
            (Feed::Fixed(input), None) => cpl_interp::Io::fixed(input),
            (Feed::Live(more), _) => cpl_interp::Io::live(more),
        };
        io = io.at(self.checks);
        if let Some(seed) = self.seed {
            io = io.seeded(seed);
        }
        if let Some(stop) = self.stop {
            io = io.until(stop);
        }
        if let Some(choices) = self.choices {
            io = io.choosing(choices);
        }
        if let Some(source) = last_values {
            io = io.with_last_values(source);
        }
        io
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
    /// Run on the calling thread, with this much of its stack: a host that
    /// cannot start a thread (the browser).
    pub on_this_thread: Option<Limits>,
    /// Take checkpoints, or start from one. Only a run on the calling
    /// thread takes them.
    pub checkpoints: Option<Checkpoints>,
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
    /// The checkpoints it took, in order.
    pub taken: Vec<Checkpoint>,
    /// With checkpoints taken, every byte of input the run was given: a
    /// run from one of them is given this as its input.
    pub sent: Vec<u8>,
}

impl From<cpl_interp::Observed> for Outcome {
    fn from(observed: cpl_interp::Observed) -> Outcome {
        Outcome {
            trap: observed.trap,
            debug: observed.debug,
            recording: observed.recording,
            profile: observed.profile,
            taken: Vec::new(),
            sent: Vec::new(),
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

/// [`Session::run`]: a built program holds all a run needs.
pub fn run(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    request: Request,
) -> Outcome {
    let io = input.io(request.last_values.then_some(program.text()));
    let (mut taken, mut sent) = (Vec::new(), Vec::new());
    let observed = match (request.on_this_thread, request.checkpoints, request.profile) {
        (Some(limits), Some(want), _) => {
            let run = cpl_interp::checkpoint::observe_on_this_thread(
                &program.ir,
                io,
                output,
                limits,
                request.trace,
                want,
            );
            (taken, sent) = (run.taken, run.sent);
            run.observed
        }
        (Some(limits), None, _) => {
            cpl_interp::observe_on_this_thread(&program.ir, io, output, limits, request.trace)
        }
        (None, _, true) => cpl_interp::run_profiled(&program.ir, io, output),
        (None, _, false) => cpl_interp::run_observed(&program.ir, io, output, request.trace),
    };
    Outcome {
        trap: observed.trap,
        debug: observed.debug,
        recording: observed.recording,
        profile: observed.profile,
        taken,
        sent,
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

/// [`run_to_end`] on the calling thread, within `limits` of its stack.
pub fn run_within(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    limits: Limits,
) -> Result<(), Trap> {
    let request = Request {
        on_this_thread: Some(limits),
        ..Request::default()
    };
    run(program, input, output, request).result()
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
    let request = Request {
        trace,
        on_this_thread: Some(limits),
        ..Request::default()
    };
    run(program, input, output, request)
}

/// The output of `program` on `input` and how it ended, or `None` when it
/// was still running after `limit`.
pub fn run_for(
    program: &Program,
    input: &[u8],
    limit: std::time::Duration,
) -> Option<(Vec<u8>, Result<(), Trap>)> {
    cpl_interp::run_for(&program.ir, input, limit)
}
