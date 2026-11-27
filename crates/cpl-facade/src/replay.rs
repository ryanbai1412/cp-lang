//! Running one recorded call again with its fn edited.

use crate::{Change, Recording as Recorded, Trap};

/// What a call did: its result and each top-level variable it changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    /// JSON.
    pub returned: Option<String>,
    /// Variable ids and their JSON values when the call returned; `stdout`
    /// is what it printed.
    pub writes: Vec<(String, String)>,
}

impl From<cpl_interp::replay::Effect> for Effect {
    fn from(e: cpl_interp::replay::Effect) -> Effect {
        Effect {
            returned: e.returned,
            writes: e.writes,
        }
    }
}

#[derive(Debug)]
pub struct Replay {
    /// The call as recorded, and as the edited fn ran it.
    pub recorded: Call,
    pub replayed: Option<Call>,
    pub before: Effect,
    pub after: Effect,
    /// The first step of the replayed call that differs from the recording,
    /// as a step of [`Replay::recording`]; `variable` is `returned` when
    /// only the result differs.
    pub change: Option<Change>,
    /// A step before the call that already differs: the edit changes an
    /// earlier call of the fn too.
    pub earlier: Option<Change>,
    /// Why the whole file ran instead of the one call.
    pub fallback: Option<String>,
    pub recording: Recorded,
    pub trap: Option<Trap>,
    json: String,
}

impl Replay {
    /// The report as JSON, for the trace viewer: values are the trace's
    /// JSON, `null` when there is none.
    pub fn json(&self) -> String {
        self.json.clone()
    }
}

impl From<cpl_interp::replay::Replay> for Replay {
    fn from(r: cpl_interp::replay::Replay) -> Replay {
        let json = r.json();
        Replay {
            json,
            recorded: r.recorded.into(),
            replayed: r.replayed.map(Into::into),
            before: r.before.into(),
            after: r.after.into(),
            change: r.change.map(Into::into),
            earlier: r.earlier.map(Into::into),
            fallback: r.fallback,
            recording: r.recording.into(),
            trap: r.trap.map(Into::into),
        }
    }
}

/// The call running at `step`, `frame` calls out from the innermost.
pub fn call_at(recorded: &Recording, step: usize, frame: usize) -> Result<Call, String> {
    cpl_interp::replay::call_at(&recorded.inner, step, frame).map(Into::into)
}

use crate::Call;
use crate::{Limits, Program, Recording, Stop, TraceConfig};

/// The call to run again, and the programs before and after the edit.
pub struct Rerun<'a> {
    pub recorded: &'a Recording,
    pub original: &'a Program,
    pub edited: &'a Program,
    pub input: &'a [u8],
    pub call: &'a Call,
}

impl<'a> Rerun<'a> {
    fn today<'b>(&'b self, call: &'b cpl_interp::trace::Call) -> cpl_interp::replay::Rerun<'b> {
        cpl_interp::replay::Rerun {
            recorded: &self.recorded.inner,
            original: &self.original.ir,
            edited: &self.edited.ir,
            input: self.input,
            call,
        }
    }
}

/// Runs `rerun`'s call again with the fns of its edited program.
pub fn replay(rerun: Rerun, config: TraceConfig) -> Replay {
    let call = rerun.call.clone().into();
    cpl_interp::replay::replay(rerun.today(&call), config.into()).into()
}

/// [`replay`] where no thread can be started, its calls held to
/// `limits` and each run ended by `stop`.
pub fn replay_on_this_thread(
    rerun: Rerun,
    config: TraceConfig,
    limits: Limits,
    stop: Stop,
) -> Replay {
    let call = rerun.call.clone().into();
    cpl_interp::replay::replay_on_this_thread(
        rerun.today(&call),
        config.into(),
        limits.into(),
        stop.0,
    )
    .into()
}
