//! What tools read from a run's recording, in the facade's own forms (contract 8).

use std::sync::Arc;

use std::io::Write;

use crate::TraceConfig;

/// What a second run records to name the locals where a trap stopped the first.
pub fn trap_config(source: &str) -> TraceConfig {
    cpl_interp::trace::trap_config(source).into()
}

/// The trace document of a run, beside its brute's when there is one.
pub fn document(
    title: &str,
    description: &str,
    file: &str,
    source: &str,
    fast: (&str, &Recording),
    slow: Option<(&str, &Recording)>,
) -> String {
    let slow = slow.map(|(name, r)| (name, &r.inner));
    cpl_interp::trace::document(
        title,
        description,
        file,
        source,
        (fast.0, &fast.1.inner),
        slow,
    )
}

/// The first step where `after` differs from `before`.
pub fn first_change(before: &Recording, after: &Recording) -> Option<Change> {
    cpl_interp::trace::first_change(&before.inner, &after.inner).map(Into::into)
}

/// [`first_change`] within the steps `old` of `before` and `new` of `after`.
pub fn first_change_in(
    before: &Recording,
    old: std::ops::Range<usize>,
    after: &Recording,
    new: std::ops::Range<usize>,
) -> Option<Change> {
    cpl_interp::trace::first_change_in(&before.inner, old, &after.inner, new).map(Into::into)
}

/// Each variable `step` names, with its value.
pub fn named_values(rec: &Recording, step: &Step) -> std::collections::HashMap<String, String> {
    cpl_interp::trace::named_values(&rec.inner, &step.clone().into())
}

/// The locals where a trap stopped the run, as `cpl run` names them.
pub fn trap_locals(recording: &Recording, source: &str, at: &str) -> Option<String> {
    cpl_interp::trace::trap_locals(&recording.inner, source, at)
}

/// Where a solution and its brute first part ways on one input.
pub fn explain_divergence(source: &str, fast: &Recording, slow: &Recording) -> Option<Divergence> {
    cpl_interp::trace::explain_divergence(source, &fast.inner, &slow.inner).map(Into::into)
}

/// Where a solution that never finishes goes round.
pub fn stuck_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
    cpl_interp::trace::stuck_divergence(source, &fast.inner).map(Into::into)
}

/// The solution's last answer, when it is the line that differs.
pub fn answered_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
    cpl_interp::trace::answered_divergence(source, &fast.inner).map(Into::into)
}

/// The stages a solution and its brute both name, side by side.
pub fn stages(fast: &Recording, slow: &Recording) -> Vec<Stage> {
    cpl_interp::trace::stages(&fast.inner, &slow.inner)
        .into_iter()
        .map(Into::into)
        .collect()
}

use crate::{Outcome, Program, RunInput, Stop};

/// The last `steps` steps of a run of `program` on `input`.
pub fn tail(program: &Program, input: &[u8], config: TraceConfig, steps: usize) -> Outcome {
    cpl_interp::trace::tail(&program.ir, input, config.into(), steps).into()
}

/// The run printing to `output`, with its trace from
/// `config.from_step` on.
pub fn windowed(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
) -> Outcome {
    cpl_interp::trace::windowed(&program.ir, input, output, config.into()).into()
}

/// The last `steps` steps of a run, traced again from a late
/// checkpoint of an untraced run: the steps are numbered from the
/// window's first.
pub fn resumed_tail(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
    steps: usize,
) -> Outcome {
    cpl_interp::trace::resumed_tail(&program.ir, input, output, config.into(), steps).into()
}

/// [`resumed_tail`], both runs ended by `stop`.
pub fn resumed_tail_until(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: TraceConfig,
    steps: usize,
    stop: Stop,
) -> Outcome {
    cpl_interp::trace::resumed_tail_until(&program.ir, input, output, config.into(), steps, stop.0)
        .into()
}

/// The untraced run talking to its judge, and every byte the judge
/// sent it.
pub fn sent(program: &Program, input: RunInput, output: &mut (dyn Write + Send)) -> Vec<u8> {
    cpl_interp::trace::sent(&program.ir, input.0, output)
}

/// The trace of an interactive run: the judge is asked only once.
pub fn interactive(
    program: &Program,
    input: RunInput,
    output: &mut (dyn Write + Send),
    config: TraceConfig,
) -> Outcome {
    cpl_interp::trace::interactive(&program.ir, input.0, output, config.into()).into()
}

/// A run recorded on a thread of its own, which traces any window of
/// it on request.
pub struct Replay(cpl_interp::trace::Replay);

impl Replay {
    /// Records `program`'s run on `input` as `config` counts its steps.
    pub fn start(program: Program, input: Vec<u8>, config: TraceConfig) -> Replay {
        let program = program.ir.clone();
        Replay(cpl_interp::trace::Replay::start(
            program,
            input,
            config.into(),
        ))
    }

    /// The run's trace from `config.from_step` on.
    pub fn window(&self, config: TraceConfig) -> Outcome {
        self.0.window(config.into()).into()
    }
}

/// What a run left behind: opaque, so a later engine can answer it from its event stream.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    pub(crate) inner: cpl_interp::trace::Recording,
    calls: Vec<Call>,
    vars: Vec<Var>,
    lives: Vec<Live>,
    shown: Vec<Shown>,
    assigned: Vec<Assigned>,
}

impl From<cpl_interp::trace::Recording> for Recording {
    fn from(r: cpl_interp::trace::Recording) -> Recording {
        fn each<A: Clone + Into<B>, B>(xs: &[A]) -> Vec<B> {
            xs.iter().cloned().map(Into::into).collect()
        }
        Recording {
            calls: each(&r.calls),
            vars: each(&r.vars),
            lives: each(&r.lives),
            shown: each(&r.shown),
            assigned: each(&r.assigned),
            inner: r,
        }
    }
}

impl Recording {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn step(&self, step: usize) -> Option<Step> {
        self.inner.step(step).map(Into::into)
    }

    /// The 1-based line `step` ran.
    pub fn line(&self, step: usize) -> Option<u32> {
        self.inner.line(step)
    }

    /// The calls running around `step`.
    pub fn depth(&self, step: usize) -> usize {
        self.inner.depth(step)
    }

    pub fn checkpoints(&self) -> impl Iterator<Item = (usize, &str)> {
        self.inner.checkpoints()
    }

    pub fn bytes(&self) -> usize {
        self.inner.bytes()
    }

    /// `step` in source terms, which a recording after an edit finds again.
    pub fn place(&self, step: usize) -> Option<Place> {
        self.inner.place(step).map(Into::into)
    }

    /// The step at `place`, or `fallback`.
    pub fn find(&self, place: Place, fallback: usize) -> usize {
        self.inner.find(place.into(), fallback)
    }

    /// What a trace of this run holds, as its document's description says it.
    pub fn note(&self) -> &'static str {
        self.inner.note()
    }

    /// Steps past the limit were not kept.
    pub fn truncated(&self) -> bool {
        self.inner.truncated
    }

    /// The first step kept is step `skipped` of the run.
    pub fn skipped(&self) -> usize {
        self.inner.skipped
    }

    /// The calls, in the order they started.
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    pub fn vars(&self) -> &[Var] {
        &self.vars
    }

    pub fn lives(&self) -> &[Live] {
        &self.lives
    }

    pub fn shown(&self) -> &[Shown] {
        &self.shown
    }

    /// Each value a named variable was given, in the order they ran.
    pub fn assigned(&self) -> &[Assigned] {
        &self.assigned
    }
}

/// A checkpoint both runs reached, with what each showed there.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
}
/// A variable of the trace.
#[derive(Clone, Debug)]
pub struct Var {
    pub id: String,
    pub name: String,
    /// The type as the file would write it.
    pub ty: String,
    pub kind: &'static str,
    /// More fields of the descriptor, in the order the file writes them.
    pub extra: Vec<(&'static str, cpl_json::Value)>,
    /// Compared between the solution and the brute.
    pub compare: bool,
    /// Too large to keep whole at some step, so drawn by a window of its first cells.
    pub window: bool,
}

impl From<cpl_interp::trace::Var> for Var {
    fn from(d: cpl_interp::trace::Var) -> Var {
        Var {
            id: d.id,
            name: d.name,
            ty: d.ty,
            kind: d.kind,
            extra: d.extra,
            compare: d.compare,
            window: d.window,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Step {
    /// 1-based.
    pub line: u32,
    pub label: String,
    pub stage: String,
    /// Each variable in scope, by index into [`Recording::vars`], as JSON.
    pub values: Vec<(usize, Arc<str>)>,
    /// A JSON object.
    pub locals: String,
    pub checkpoint: Option<String>,
    /// A variable and the storage path of the cell that changed.
    pub focus: Option<(usize, Vec<usize>)>,
    /// Call stack at this step (innermost first), populated when `record_frames` is true.
    pub frames: Vec<(String, u32)>,
}

impl From<cpl_interp::trace::Step> for Step {
    fn from(d: cpl_interp::trace::Step) -> Step {
        Step {
            line: d.line,
            label: d.label,
            stage: d.stage,
            values: d.values,
            locals: d.locals,
            checkpoint: d.checkpoint,
            focus: d.focus,
            frames: d.frames,
        }
    }
}

impl From<Step> for cpl_interp::trace::Step {
    fn from(d: Step) -> cpl_interp::trace::Step {
        cpl_interp::trace::Step {
            line: d.line,
            label: d.label,
            stage: d.stage,
            values: d.values,
            locals: d.locals,
            checkpoint: d.checkpoint,
            focus: d.focus,
            frames: d.frames,
        }
    }
}

/// The value of one top-level variable when the run ended, for the
/// editor to show beside its line.
#[derive(Clone, Debug)]
pub struct Live {
    /// 1-based.
    pub line: u32,
    pub name: String,
    pub value: String,
}

impl From<cpl_interp::trace::Live> for Live {
    fn from(d: cpl_interp::trace::Live) -> Live {
        Live {
            line: d.line,
            name: d.name,
            value: d.value,
        }
    }
}

/// One `debug(...)` that ran.
#[derive(Clone, Debug)]
pub struct Shown {
    /// 1-based.
    pub line: u32,
    /// The checkpoint it is: the label, or the arguments as written, and
    /// which time it ran.
    pub name: String,
    /// `i = 3, best = 7`.
    pub text: String,
}

impl From<cpl_interp::trace::Shown> for Shown {
    fn from(d: cpl_interp::trace::Shown) -> Shown {
        Shown {
            line: d.line,
            name: d.name,
            text: d.text,
        }
    }
}

/// A value a named variable was given.
#[derive(Clone, Debug)]
pub struct Assigned {
    /// 1-based.
    pub line: u32,
    pub name: String,
    pub value: String,
    /// Bytes of input the run had read when it was given.
    pub read: usize,
}

impl From<cpl_interp::trace::Assigned> for Assigned {
    fn from(d: cpl_interp::trace::Assigned) -> Assigned {
        Assigned {
            line: d.line,
            name: d.name,
            value: d.value,
            read: d.read,
        }
    }
}

/// One call of a fn: the steps it took and what it returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub name: String,
    /// 1-based: the third call of `name` in the run is call 3.
    pub ordinal: usize,
    /// The steps taken while it ran are `entry..exit`.
    pub entry: usize,
    pub exit: usize,
    /// Fns running around it; a fn called from the top level is at 1.
    pub depth: usize,
    /// Its arguments as JSON, by parameter name.
    pub args: String,
    /// Its result as JSON, unless it returns nothing or never returned.
    pub returned: Option<String>,
}

impl From<cpl_interp::trace::Call> for Call {
    fn from(d: cpl_interp::trace::Call) -> Call {
        Call {
            name: d.name,
            ordinal: d.ordinal,
            entry: d.entry,
            exit: d.exit,
            depth: d.depth,
            args: d.args,
            returned: d.returned,
        }
    }
}

impl From<Call> for cpl_interp::trace::Call {
    fn from(d: Call) -> cpl_interp::trace::Call {
        cpl_interp::trace::Call {
            name: d.name,
            ordinal: d.ordinal,
            entry: d.entry,
            exit: d.exit,
            depth: d.depth,
            args: d.args,
            returned: d.returned,
        }
    }
}

/// A step in source terms: its line, and which visit to that line it is.
/// A run recorded again after an edit finds the same place by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub line: u32,
    /// 1-based: the third time the run reached `line` is visit 3.
    pub visit: usize,
}

impl From<cpl_interp::trace::Place> for Place {
    fn from(d: cpl_interp::trace::Place) -> Place {
        Place {
            line: d.line,
            visit: d.visit,
        }
    }
}

impl From<Place> for cpl_interp::trace::Place {
    fn from(d: Place) -> cpl_interp::trace::Place {
        cpl_interp::trace::Place {
            line: d.line,
            visit: d.visit,
        }
    }
}

/// The first step at which two runs of a file hold different values or
/// have printed different output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The step of the later run, or its last step when it ended first.
    pub step: usize,
    pub line: u32,
    /// The variable that differs; `None` when one run ended first.
    pub variable: Option<String>,
    /// The variable's JSON in each run, or the step's label when one run ended first.
    /// `None`: the run had no such variable, or had ended.
    pub before: Option<String>,
    pub after: Option<String>,
}

impl From<cpl_interp::trace::Change> for Change {
    fn from(d: cpl_interp::trace::Change) -> Change {
        Change {
            step: d.step,
            line: d.line,
            variable: d.variable,
            before: d.before,
            after: d.after,
        }
    }
}

impl From<cpl_interp::trace::Stage> for Stage {
    fn from(d: cpl_interp::trace::Stage) -> Stage {
        Stage {
            name: d.name,
            line: d.line,
            fast: d.fast,
            brute: d.brute,
        }
    }
}

/// Where a solution and its brute first give a variable they both name
/// different values: its `nth` value (1-based, a repeat of the value
/// before not counted), with the line and the value on each side, `None`
/// on a side that never gives it an `nth` value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    pub name: String,
    pub nth: usize,
    pub fast: Option<(u32, String)>,
    pub brute: Option<(u32, String)>,
    /// The solution's own last steps in a wrong call, not those of the
    /// calls it makes: each line and its locals, as `k: 1, s: 0`.
    pub inside: Vec<(u32, String)>,
    /// The lines to look at, each with why: where a wrong call made its
    /// value, where what that line reads last changed, or the declaration
    /// of a variable an earlier test left a value in.
    pub blame: Vec<(u32, String)>,
    /// `name` is a call, `f(k: 1)`, and `nth` which call of `f` it is.
    pub call: bool,
    /// The solution never finishes: `name` is the loop it goes round in,
    /// `fast` that loop's line and the state it keeps coming back to, and
    /// `nth` how many steps one time round takes.
    pub stuck: bool,
}

impl From<cpl_interp::trace::Divergence> for Divergence {
    fn from(d: cpl_interp::trace::Divergence) -> Divergence {
        Divergence {
            name: d.name,
            nth: d.nth,
            fast: d.fast,
            brute: d.brute,
            inside: d.inside,
            blame: d.blame,
            call: d.call,
            stuck: d.stuck,
        }
    }
}
