//! A window of a long run: the run again, its steps before the window
//! counted and dropped. The run is deterministic, so the window holds the
//! steps the whole recording would have held there.

use super::{Recording, TraceConfig, Tracer};
use crate::checkpoint::{
    count_on_this_thread, record_last_on_this_thread, record_live_on_this_thread,
    resume_on_this_thread, Checkpoint,
};
use crate::stop::Limits;
use crate::{observe_on_this_thread, run_observed, Io, Observed};
use cpl_ir::Program;
use std::io::Write;
use std::sync::{mpsc, Mutex};

impl Tracer {
    /// Steps seen so far, skipped or kept.
    pub(crate) fn steps_so_far(&self) -> usize {
        self.rec.skipped + self.rec.log.len()
    }

    /// Whether this step comes before the window and is dropped.
    pub(super) fn skipped(&mut self) -> bool {
        if self.skip == 0 {
            return false;
        }
        self.skip -= 1;
        self.rec.skipped += 1;
        self.skipped_at = None;
        // The first step kept shows every variable, not only what it changed.
        self.touch_all();
        true
    }

    /// A declaration on the line of the one just skipped joins its step, as
    /// [`Tracer::declared`] joins them when it records: count one step.
    pub(super) fn skip_declaration(&mut self, fresh: bool, at: (u32, u32)) {
        if fresh && self.skip > 0 && self.skipped_at == Some(at) {
            self.skip += 1;
            self.rec.skipped -= 1;
        }
    }
}

/// The last `steps` steps of a run of `program` on `input`: one run counts
/// its steps, a second keeps the last ones. Memory follows the window, not
/// the run.
pub fn tail(program: &Program, input: &[u8], config: TraceConfig, steps: usize) -> Observed {
    let counted = run_observed(
        program,
        Io::fixed(input),
        &mut Vec::new(),
        Some(TraceConfig {
            from_step: Some(usize::MAX),
            ..config.clone()
        }),
    );
    let total = counted
        .recording
        .as_ref()
        .map_or(0, |rec: &Recording| rec.skipped);
    let config = TraceConfig {
        from_step: Some(total.saturating_sub(steps)),
        max_steps: Some(steps.max(config.max_steps.unwrap_or(0))),
        ..config
    };
    run_observed(program, Io::fixed(input), &mut Vec::new(), Some(config))
}

/// The run of `program` on `input`, printing to `output`, with its trace
/// from `config.from_step` on: the traced run ends once its window is full,
/// and then the run goes again untraced for its output and how it ended. A
/// long run costs its untraced time plus its window's, not a traced run.
pub fn windowed(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
) -> Observed {
    let config = TraceConfig {
        stop_when_full: true,
        ..config
    };
    let mut printed = Vec::new();
    let window = run_observed(program, Io::fixed(input), &mut printed, Some(config));
    if !window.recording.as_ref().is_some_and(|rec| rec.truncated) {
        let _ = output.write_all(&printed);
        return window;
    }
    Observed {
        recording: window.recording,
        ..run_observed(program, Io::fixed(input), output, None)
    }
}

/// Loop rounds and calls between two checkpoints of a run [`resumed_tail`] keeps.
const CHECKPOINT_EVERY: u64 = 1 << 16;

/// The checkpoints [`resumed_tail`] keeps, the latest ones: each holds a copy
/// of the run's state, so this bounds its memory.
const CHECKPOINTS_KEPT: usize = 8;

/// The last `steps` steps of a run of `program` on `input`, which prints to
/// `output`. The run goes untraced, with a checkpoint every so often, and
/// only its end is traced again, from a late enough checkpoint: memory
/// follows the window and time stays near the untraced run's. The steps
/// before it are not counted, so unlike [`tail`] the window's steps are
/// numbered from its first, as `--last` numbers them.
pub fn resumed_tail(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
    steps: usize,
) -> Observed {
    resumed_tail_every(program, input, output, config, steps, CHECKPOINT_EVERY)
}

/// [`resumed_tail`] with a checkpoint every `every` loop rounds and calls.
#[doc(hidden)]
pub fn resumed_tail_every(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
    steps: usize,
    every: u64,
) -> Observed {
    let config = TraceConfig {
        last_steps: Some(steps),
        ..config
    };
    let output = Mutex::new(output);
    crate::on_interp_thread(&|limits| {
        let mut output = output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (mut whole, taken) = record_last_on_this_thread(
            program,
            input,
            &mut **output,
            limits,
            every,
            CHECKPOINTS_KEPT,
        );
        whole.recording = tail_from(program, input, &taken, &config, steps, limits);
        whole
    })
}

/// The last `steps` steps of the run on `input`, traced again from the
/// latest of `taken` that reaches back far enough.
fn tail_from(
    program: &Program,
    input: &[u8],
    taken: &[Checkpoint],
    config: &TraceConfig,
    steps: usize,
    limits: Limits,
) -> Option<Recording> {
    let mut back = 1;
    loop {
        let trace = Some(config.clone());
        let mut ignored = std::io::sink();
        let Some(from) = taken.len().checked_sub(back).map(|at| &taken[at]) else {
            let io = Io::fixed(input);
            return observe_on_this_thread(program, io, &mut ignored, limits, trace).recording;
        };
        let resumed =
            resume_on_this_thread(program, input, from, &mut ignored, limits, trace).recording;
        if resumed
            .as_ref()
            .is_some_and(|rec: &Recording| rec.len() >= steps)
        {
            return resumed;
        }
        // Twice as far back each time, and the first checkpoint last.
        back = if back == taken.len() {
            back + 1
        } else {
            (back * 2).min(taken.len())
        };
    }
}

/// The untraced run talking to its judge through `io` and printing to
/// `output`, and every byte the judge sent it.
pub fn sent(program: &Program, io: Io, output: &mut (dyn Write + Send)) -> Vec<u8> {
    let io = Mutex::new(Some(io));
    let output = Mutex::new(output);
    let sent = Mutex::new(Vec::new());
    crate::on_interp_thread(&|limits| {
        let mut output = output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let io = io
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect("only the thread that starts takes the input");
        let (whole, _, read) =
            record_live_on_this_thread(program, io, &mut **output, limits, u64::MAX, 1);
        *sent.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = read;
        whole
    });
    sent.into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The trace of an interactive run, which talks to its judge through `io`
/// and prints to `output`. The run goes untraced, keeping what it was sent
/// and, for `--last`, checkpoints; the steps kept are then traced again
/// with that transcript as fixed input, so the judge is asked only once.
pub fn interactive(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    config: TraceConfig,
) -> Observed {
    let io = Mutex::new(Some(io));
    let output = Mutex::new(output);
    crate::on_interp_thread(&|limits| {
        let mut output = output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let io = io
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect("only the thread that starts takes the input");
        let tail = config.last_steps.filter(|_| config.from_step.is_none());
        let (every, keep) = match tail {
            Some(_) => (CHECKPOINT_EVERY, CHECKPOINTS_KEPT),
            None => (u64::MAX, 1),
        };
        let (mut whole, taken, sent) =
            record_live_on_this_thread(program, io, &mut **output, limits, every, keep);
        whole.recording = match tail {
            Some(steps) => tail_from(program, &sent, &taken, &config, steps, limits),
            None => {
                let config = TraceConfig {
                    stop_when_full: true,
                    ..config.clone()
                };
                let io = Io::fixed(&sent);
                let mut ignored = std::io::sink();
                observe_on_this_thread(program, io, &mut ignored, limits, Some(config)).recording
            }
        };
        whole
    })
}

/// Checkpoints a [`Timeline`] keeps across the run: each holds a copy of the
/// run's state, so this bounds its memory.
const TIMELINE_KEPT: usize = 64;

/// A run's checkpoints, each with the trace steps before it: a window
/// anywhere in the run is traced again from the last checkpoint before it,
/// not from the start. Its values are the interpreter's, so it stays on the
/// thread that recorded it; [`Replay`] serves one to other threads.
#[derive(Default)]
pub struct Timeline {
    taken: Vec<Checkpoint>,
    /// The run's steps, as its config counts them.
    pub steps: usize,
}

impl Timeline {
    /// One run of `program` on `input`, counting the steps `config` would
    /// record.
    pub fn record_on_this_thread(
        program: &Program,
        input: &[u8],
        config: &TraceConfig,
        limits: crate::Limits,
        every: u64,
    ) -> Timeline {
        let config = config.clone();
        let (counted, taken) =
            count_on_this_thread(program, input, limits, config, every, TIMELINE_KEPT);
        Timeline {
            taken,
            steps: counted.recording.map_or(0, |rec| rec.skipped),
        }
    }

    /// The steps before each checkpoint kept, in order.
    #[doc(hidden)]
    pub fn starts(&self) -> Vec<usize> {
        self.taken.iter().map(|c| c.steps).collect()
    }

    /// The run's trace from step `config.from_step` on, as the whole run
    /// with that `from_step` records it.
    pub fn window_on_this_thread(
        &self,
        program: &Program,
        input: &[u8],
        config: TraceConfig,
        limits: crate::Limits,
    ) -> Observed {
        let from = config.from_step.unwrap_or(0);
        let config = TraceConfig {
            stop_when_full: true,
            ..config
        };
        let mut ignored = std::io::sink();
        let Some(start) = self.taken.iter().rev().find(|c| c.steps <= from) else {
            let io = Io::fixed(input);
            return observe_on_this_thread(program, io, &mut ignored, limits, Some(config));
        };
        let config = TraceConfig {
            from_step: Some(from - start.steps),
            ..config
        };
        let mut window =
            resume_on_this_thread(program, input, start, &mut ignored, limits, Some(config));
        if let Some(rec) = &mut window.recording {
            rec.skipped += start.steps;
        }
        window
    }
}

type Ask = (TraceConfig, mpsc::Sender<Observed>);

/// A [`Timeline`] on its own interpreter thread, which traces any window of
/// the run on request.
pub struct Replay {
    asks: Mutex<mpsc::Sender<Ask>>,
}

impl Replay {
    /// Records `program`'s run on `input` as `config` counts its steps.
    pub fn start(program: Program, input: Vec<u8>, config: TraceConfig) -> Replay {
        Replay::start_every(program, input, config, CHECKPOINT_EVERY)
    }

    #[doc(hidden)]
    pub fn start_every(
        program: Program,
        input: Vec<u8>,
        config: TraceConfig,
        every: u64,
    ) -> Replay {
        let (asks, asked) = mpsc::channel::<Ask>();
        let asked = Mutex::new(Some(asked));
        crate::spawn_interp_thread(std::sync::Arc::new(move |limits| {
            let Some(asked) = asked.lock().ok().and_then(|mut a| a.take()) else {
                return;
            };
            let timeline =
                Timeline::record_on_this_thread(&program, &input, &config, limits, every);
            for (config, reply) in asked {
                let _ =
                    reply.send(timeline.window_on_this_thread(&program, &input, config, limits));
            }
        }));
        Replay {
            asks: Mutex::new(asks),
        }
    }

    /// The run's trace from `config.from_step` on.
    pub fn window(&self, config: TraceConfig) -> Observed {
        let (reply, answer) = mpsc::channel();
        let sent = (self.asks.lock())
            .map_err(|_| ())
            .and_then(|asks| asks.send((config, reply)).map_err(|_| ()));
        match sent.ok().and_then(|()| answer.recv().ok()) {
            Some(observed) => observed,
            None => Observed {
                trap: Some(crate::Trap {
                    message: "the interpreter itself failed; this is a bug in cpl".into(),
                    span: cpl_diag::Span::default(),
                    call_stack: Vec::new(),
                    case: None,
                }),
                ..Observed::default()
            },
        }
    }
}
