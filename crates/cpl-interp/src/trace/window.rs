//! A window of a long run: the run again, its steps before the window
//! counted and dropped. The run is deterministic, so the window holds the
//! steps the whole recording would have held there.

use super::{Recording, TraceConfig, Tracer};
use crate::{run_observed, Io, Observed};
use cpl_ir::Program;

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
