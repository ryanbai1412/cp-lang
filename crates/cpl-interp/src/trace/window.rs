//! A window of a long run: the run again, its steps before the window
//! counted and dropped. The run is deterministic, so the window holds the
//! steps the whole recording would have held there.

use super::{Recording, TraceConfig, Tracer};
use crate::checkpoint::{record_last_on_this_thread, resume_on_this_thread};
use crate::{observe_on_this_thread, run_observed, Io, Observed};
use cpl_ir::Program;
use std::io::Write;
use std::sync::Mutex;

impl Tracer {
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
        if program.multitest {
            let config = Some(config.clone());
            return observe_on_this_thread(
                program,
                Io::fixed(input),
                &mut **output,
                limits,
                config,
            );
        }
        let (mut whole, taken) = record_last_on_this_thread(
            program,
            input,
            &mut **output,
            limits,
            every,
            CHECKPOINTS_KEPT,
        );
        let mut back = 1;
        let recording = loop {
            let trace = Some(config.clone());
            let mut ignored = std::io::sink();
            let Some(from) = taken.len().checked_sub(back).map(|at| &taken[at]) else {
                let io = Io::fixed(input);
                break observe_on_this_thread(program, io, &mut ignored, limits, trace).recording;
            };
            let resumed =
                resume_on_this_thread(program, input, from, &mut ignored, limits, trace).recording;
            if resumed
                .as_ref()
                .is_some_and(|rec: &Recording| rec.len() >= steps)
            {
                break resumed;
            }
            // Twice as far back each time, and the first checkpoint last.
            back = if back == taken.len() {
                back + 1
            } else {
                (back * 2).min(taken.len())
            };
        };
        whole.recording = recording;
        whole
    })
}
