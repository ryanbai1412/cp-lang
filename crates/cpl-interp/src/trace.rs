//! The trace of a run: what `cpl run --trace` and `cpl stress --trace`
//! show in the trace viewer (`tools/trace/trace-schema.json`).
//!
//! A step is a full snapshot of the file's top-level variables, taken
//! when one of them changes, at each `print` and at each `debug(...)`.
//! A `print` and a `debug(...)` are also checkpoints: two runs of the
//! same input, the solution and its brute, are lined up on them, since
//! their own variables have nothing else in common.

use crate::value::{OptVal, TupleVal, Value};
use cpl_diag::{LineIndex, Span};
use cpl_ir::{Dim, Program, Ty};
use show::{field_names, field_tys, render, Mode};
pub(crate) use show::{json_of, text_of, ShowTexts};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;

mod change;
mod compare;
mod document;
mod glob;
mod last;
mod lines;
mod log;
mod raw;
mod show;
mod snapshot;
mod trap;
pub(crate) use last::last_values_of;
mod unseen;
mod window;
#[cfg(test)]
use change::members;
pub use change::{first_change, first_change_in};
pub use compare::{
    answered_divergence, explain_divergence, first_divergence, mentions, named_values,
    printed_divergence, stages, stuck_divergence, Divergence, Stage,
};
pub use document::document;
use glob::glob_match;
pub use log::Steps;
pub use snapshot::string;
use snapshot::*;
pub(crate) use snapshot::{args_json, MAX_EXACT};
pub use trap::{shown, trap_config, trap_locals};
pub use window::{
    interactive, resumed_tail, resumed_tail_every, sent, tail, windowed, Replay, Timeline,
};

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
    /// [`Recording::assigned`]: what `cpl stress --explain` compares.
    pub assignments: bool,
    /// Stop the run when this call returns: the fn's name and which call
    /// of it, 1-based. What [`crate::replay`] runs.
    pub stop_after_call: Option<(String, usize)>,
    /// Keep up to this many steps instead of what fits in a trace file.
    pub max_steps: Option<usize>,
    /// Skip this many steps before keeping any: a window of a long run.
    pub from_step: Option<usize>,
    /// End the run once the steps are full, when what follows them (its
    /// output, a later trap) is not wanted.
    pub stop_when_full: bool,
}

/// The message of the trap that ends a run at [`TraceConfig::stop_after_call`].
pub(crate) const STOPPED: &str = "the call to replay returned";

/// A traced run is bounded like any other, with [`crate::Io::until`].
pub use crate::stop::OUT_OF_TIME;

/// The most steps one run keeps; the viewer reads up to 100,000.
const MAX_STEPS: usize = 20_000;
/// The most steps kept of what is above `@!multitest`. The test cases
/// start from the values it ends with, whether or not it fits.
const SETUP_STEPS: usize = 2_000;
/// The most snapshot text one run keeps.
const MAX_BYTES: usize = 48 << 20;
/// A variable with more cells than this is shown by its size.
const MAX_CELLS: usize = 20_000;
/// The most values [`Recording::assigned`] keeps.
const MAX_ASSIGNED: usize = 200_000;

/// How the viewer draws a variable. It follows from the type alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Scalar,
    Array,
    Grid,
    Dp,
    /// `[V]List<V>` or `[V]List<(V, ..)>`: neighbour lists.
    Adjacency,
    /// `[V]?V`: drawn as a tree while every snapshot is one, else as an
    /// array.
    Parents,
    /// `Graph<V, E>` of the standard library.
    Csr,
    /// `Rooted<V, Pre>` of the standard library.
    Rooted,
    Json,
}

/// A variable of the trace.
#[derive(Clone, Debug)]
pub struct Var {
    pub id: String,
    pub name: String,
    /// The type as the file would write it.
    pub ty: String,
    pub kind: &'static str,
    /// More fields of the descriptor, as JSON members with a leading comma.
    pub extra: String,
    /// Compared between the solution and the brute.
    pub compare: bool,
    /// Too large to keep whole at some step, so drawn by a window of its first cells.
    pub window: bool,
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

/// The value of one top-level variable when the run ended, for the
/// editor to show beside its line.
#[derive(Clone, Debug)]
pub struct Live {
    /// 1-based.
    pub line: u32,
    pub name: String,
    pub value: String,
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

/// What a run left behind.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    pub vars: Vec<Var>,
    /// The steps, read one at a time: [`Recording::step`] and [`Recording::steps`].
    log: log::Log,
    /// Steps past the limit were not kept.
    pub truncated: bool,
    pub lives: Vec<Live>,
    pub shown: Vec<Shown>,
    /// With [`TraceConfig::assignments`], in the order they ran.
    pub assigned: Vec<Assigned>,
    /// The calls, in the order they started; none with `--last`.
    pub calls: Vec<Call>,
    /// Steps skipped by [`TraceConfig::from_step`]: the first step kept is step `skipped` of the run.
    pub skipped: usize,
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

/// A step in source terms: its line, and which visit to that line it is.
/// A run recorded again after an edit finds the same place by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub line: u32,
    /// 1-based: the third time the run reached `line` is visit 3.
    pub visit: usize,
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

impl Recording {
    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.len() == 0
    }

    /// Step `step`, rebuilt from the snapshot before it and the changes since.
    pub fn step(&self, step: usize) -> Option<Step> {
        (step < self.len()).then(|| {
            self.log
                .step_of(step, &self.log.state_at(step), self.vars.len())
        })
    }

    /// The steps `range`, rebuilt one from the last.
    pub fn steps(&self, range: std::ops::Range<usize>) -> Steps<'_> {
        Steps::new(
            &self.log,
            range.start..range.end.min(self.len()),
            self.vars.len(),
        )
    }

    pub fn line(&self, step: usize) -> Option<u32> {
        self.log.line(step)
    }

    /// How many frames are on the stack at `step`, the top level included.
    pub fn depth(&self, step: usize) -> usize {
        self.log.depth(step)
    }

    /// The checkpoints, with their steps, in the order they were reached.
    pub fn checkpoints(&self) -> impl Iterator<Item = (usize, &str)> {
        self.log.checkpoints()
    }

    /// Each variable's value at `step`, as JSON, by id.
    pub(crate) fn values_at(&self, step: usize) -> BTreeMap<String, String> {
        let Some(step) = self.step(step) else {
            return BTreeMap::new();
        };
        step.values
            .into_iter()
            .map(|(var, json)| (self.vars[var].id.clone(), json.to_string()))
            .collect()
    }

    /// How many bytes the recording holds.
    pub fn bytes(&self) -> usize {
        self.log.bytes()
    }

    pub fn place(&self, step: usize) -> Option<Place> {
        let line = self.line(step)?;
        let visit = (0..=step).filter(|&i| self.line(i) == Some(line)).count();
        Some(Place { line, visit })
    }

    /// The step at `place`: the same visit to the same line, else the
    /// line's last visit, else the step nearest `fallback`.
    pub fn find(&self, place: Place, fallback: usize) -> usize {
        let mut last = None;
        let mut seen = 0;
        for i in 0..self.len() {
            if self.line(i) == Some(place.line) {
                seen += 1;
                last = Some(i);
                if seen == place.visit {
                    return i;
                }
            }
        }
        last.unwrap_or_else(|| fallback.min(self.len().saturating_sub(1)))
    }

    /// The innermost call running at `step`, or the call `frame` levels out from it.
    pub fn call_at(&self, step: usize, frame: usize) -> Option<&Call> {
        let open: Vec<&Call> = self
            .calls
            .iter()
            .filter(|c| c.entry <= step && step < c.exit)
            .collect();
        let depth = open.iter().map(|c| c.depth).max()?.checked_sub(frame)?;
        open.into_iter().rev().find(|c| c.depth == depth)
    }
}

pub(crate) struct Tracer {
    lines: LineIndex,
    /// The last span start looked up and its line: a step's span is
    /// looked up more than once.
    last_line: std::cell::Cell<(u32, u32)>,
    /// The names of the imported modules' fns (see [`Tracer::step_line`]).
    library: HashSet<String>,
    /// A fn's source name by its lowered one, where they differ.
    shown: HashMap<String, String>,
    /// No option drops or filters steps: every write is a step.
    unfiltered: bool,
    source: String,
    include_fns: Vec<String>,
    exclude_fns: Vec<String>,
    /// Active calls matching `include_fns` / `exclude_fns`.
    pos_inside: usize,
    neg_inside: usize,
    /// By active call, whether it is an imported module's fn the trace
    /// steps over: a library's own steps are not the user's, unless the
    /// trace's fn names ask for it.
    over: Vec<bool>,
    test_cases: Vec<(u64, u64)>,
    watch: Vec<String>,
    last_steps: Option<usize>,
    max_depth: Option<usize>,
    record_frames: bool,
    /// Set during `finish` so the final/error step is always captured.
    finishing: bool,
    kinds: Vec<Kind>,
    /// The variable of each global, once it has been seen.
    of_global: Vec<Option<usize>>,
    /// Globals in scope: declared, and in this test case.
    pub visible: Vec<bool>,
    /// Those declared above `@!multitest`.
    setup: Vec<bool>,
    /// Whether every snapshot of a `[V]?V` was one rooted tree.
    tree: Vec<bool>,
    stdout: usize,
    debug_vars: HashMap<String, usize>,
    counts: HashMap<String, usize>,
    prints: usize,
    pub case: Option<u64>,
    max_steps: usize,
    max_bytes: usize,
    /// The stage of the test case it is in, interned in the log.
    stage_of_case: Option<(Option<u64>, u32)>,
    /// What the locals render to, kept between steps.
    locals: String,
    local_names: String,
    /// Where the names in `local_names` are, so they are escaped once.
    local_name_at: Vec<(usize, usize)>,
    local_ints: Vec<i64>,
    /// The globals written since the last step, each with the one item
    /// written when only one was, unless `all` may have changed.
    touched: Vec<(usize, Option<usize>)>,
    all_touched: bool,
    pub assignments: bool,
    pub rec: Recording,
    /// How many times each fn has been called so far.
    ordinals: HashMap<String, usize>,
    /// The index in `rec.calls` of each running call, if it is kept.
    open: Vec<Option<usize>>,
    /// Calls entered so far.
    entries: u64,
    /// [`Self::entries`] and the call depth at the last step recorded.
    last_step_at: (u64, usize),
    stop_after_call: Option<(String, usize)>,
    /// The depth of the call to stop after, while it runs.
    stopping_at: Option<usize>,
    /// The call to stop after has returned.
    pub stopped: bool,
    stop_when_full: bool,
    /// Steps still to skip before the window ([`TraceConfig::from_step`]).
    skip: usize,
    /// The line and stage of the declaration just skipped, if the last step skipped was one.
    skipped_at: Option<(u32, u32)>,
}

/// What the interpreter shows the tracer of its state.
pub(crate) struct View<'a> {
    pub program: &'a Program,
    pub globals: &'a [Value],
    pub domains: &'a [(i64, i64)],
    /// The named locals of the innermost call, with their values.
    pub locals: Vec<(&'a str, &'a Ty, &'a Value)>,
    /// The same locals read straight from the VM's registers, in place of
    /// `locals`, when each is an exact `Int`.
    pub raw: Option<&'a [(&'a str, i64)]>,
    /// Current call depth (`0` at top level).
    pub depth: usize,
    /// Active call frames `(fn_name, call_site_span)` from outermost to innermost.
    pub call_frames: &'a [(&'a str, Span)],
    /// What each value of a type with an `impl Show` shows as.
    pub shown: &'a ShowTexts,
    /// Bytes of input read so far.
    pub read: usize,
}

impl Tracer {
    pub fn new(config: TraceConfig, program: &Program) -> Tracer {
        let mut include_fns = Vec::new();
        let mut exclude_fns = Vec::new();
        if let Some(spec) = config.only_fn {
            for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                if let Some(rest) = part.strip_prefix('!') {
                    if !rest.is_empty() {
                        exclude_fns.push(rest.to_string());
                    }
                } else {
                    include_fns.push(part.to_string());
                }
            }
        }
        let mut tracer = Tracer {
            lines: LineIndex::new(&config.source),
            last_line: std::cell::Cell::new((u32::MAX, 0)),
            library: (program.fns.iter().zip(&program.library_fns))
                .filter(|(_, &library)| library)
                .map(|(f, _)| f.name.clone())
                .collect(),
            shown: lines::shown_names(program),
            unfiltered: false,
            source: config.source.clone(),
            include_fns,
            exclude_fns,
            pos_inside: 0,
            over: Vec::new(),
            neg_inside: 0,
            test_cases: config.test_cases,
            watch: config.watch,
            last_steps: config.last_steps,
            max_depth: config.max_depth,
            record_frames: config.record_frames,
            finishing: false,
            kinds: Vec::new(),
            of_global: vec![None; program.globals.len()],
            visible: vec![false; program.globals.len()],
            setup: vec![false; program.globals.len()],
            tree: Vec::new(),
            stdout: 0,
            debug_vars: HashMap::new(),
            counts: HashMap::new(),
            prints: 0,
            case: None,
            max_steps: config.max_steps.unwrap_or(MAX_STEPS),
            max_bytes: config.max_steps.map_or(MAX_BYTES, |steps| {
                steps
                    .saturating_mul(64)
                    .clamp(MAX_BYTES, u32::MAX as usize / 2)
            }),
            stage_of_case: None,
            locals: String::new(),
            local_names: String::new(),
            local_name_at: Vec::new(),
            local_ints: Vec::new(),
            touched: Vec::new(),
            all_touched: true,
            assignments: config.assignments,
            rec: Recording {
                log: log::Log::new(&config.source, config.record_frames),
                ..Recording::default()
            },
            ordinals: HashMap::new(),
            open: Vec::new(),
            entries: 0,
            last_step_at: (0, 0),
            stop_after_call: config.stop_after_call,
            stopping_at: None,
            stopped: false,
            stop_when_full: config.stop_when_full,
            skip: config.from_step.unwrap_or(0),
            skipped_at: None,
        };
        tracer.stdout = tracer.add_var(
            Var {
                id: "stdout".into(),
                name: "stdout".into(),
                ty: "what the program printed, a line at a time".into(),
                kind: "json",
                extra: String::new(),
                compare: true,
                window: false,
            },
            Kind::Json,
        );
        tracer.unfiltered = tracer.include_fns.is_empty()
            && tracer.exclude_fns.is_empty()
            && tracer.test_cases.is_empty()
            && tracer.watch.is_empty()
            && tracer.last_steps.is_none()
            && tracer.max_depth.is_none();
        tracer
    }

    fn add_var(&mut self, var: Var, kind: Kind) -> usize {
        self.rec.vars.push(var);
        self.kinds.push(kind);
        self.tree.push(kind == Kind::Parents);
        self.rec.vars.len() - 1
    }

    /// A call of `name` starts, with `args` (JSON, by name) when asked for.
    pub fn enter_call(&mut self, name: &str, args: impl FnOnce() -> String) {
        self.entries += 1;
        let ordinal = self.ordinals.entry(name.to_string()).or_insert(0);
        *ordinal += 1;
        let ordinal = *ordinal;
        if self
            .stop_after_call
            .as_ref()
            .is_some_and(|(n, k)| n == name && *k == ordinal)
        {
            self.stopping_at = Some(self.open.len());
        }
        let kept = self.last_steps.is_none() && self.skip == 0 && self.rec.calls.len() < MAX_STEPS;
        self.open.push(kept.then_some(self.rec.calls.len()));
        if kept {
            self.rec.calls.push(Call {
                name: name.to_string(),
                ordinal,
                entry: self.rec.log.len(),
                exit: self.rec.log.len(),
                depth: self.open.len(),
                args: args(),
                returned: None,
            });
        }
    }

    /// Whether the call that runs now is kept, so its result is wanted.
    pub fn keeps_call(&self) -> bool {
        self.open.last().is_some_and(Option::is_some)
    }

    /// The running call ends, with what it returned (JSON) when kept.
    pub fn leave_call(&mut self, returned: Option<String>) {
        if let Some(Some(index)) = self.open.pop() {
            let call = &mut self.rec.calls[index];
            call.exit = self.rec.log.len();
            call.returned = returned;
        }
        if self.stopping_at == Some(self.open.len()) {
            self.stopped = true;
        }
    }

    pub fn enter_fn(&mut self, name: &str) -> (bool, bool) {
        self.touch_all();
        let pos =
            !self.include_fns.is_empty() && self.include_fns.iter().any(|p| glob_match(p, name));
        let neg =
            !self.exclude_fns.is_empty() && self.exclude_fns.iter().any(|p| glob_match(p, name));
        if pos {
            self.pos_inside += 1;
        }
        if neg {
            self.neg_inside += 1;
        }
        self.over.push(!pos && self.library.contains(name));
        (pos, neg)
    }

    /// Whether the trace steps over a call of `name`, an imported module's
    /// fn the trace's fn names don't ask for.
    pub(crate) fn steps_over(&self, name: &str) -> bool {
        self.library.contains(name) && !self.include_fns.iter().any(|p| glob_match(p, name))
    }

    /// Inside a call the trace steps over.
    pub(crate) fn over_library(&self) -> bool {
        self.over.last().copied().unwrap_or(false)
    }

    /// A call the trace stepped over took a `mut` argument: the step at
    /// its call shows what it changed.
    pub(crate) fn stepped_over(&mut self, view: &View, span: Span) {
        self.touch_all();
        self.step(view, span, None, None, None);
    }

    pub fn leave_fn(&mut self, (pos, neg): (bool, bool)) {
        self.touch_all();
        if pos {
            self.pos_inside -= 1;
        }
        if neg {
            self.neg_inside -= 1;
        }
        self.over.pop();
    }

    fn case_matches(&self) -> bool {
        if self.test_cases.is_empty() {
            return true;
        }
        match self.case {
            Some(c) if c > 0 => self.test_cases.iter().any(|&(lo, hi)| c >= lo && c <= hi),
            _ => false,
        }
    }

    pub(crate) fn recording(&self, depth: usize) -> bool {
        if self.finishing {
            return true;
        }
        let fn_ok = (self.include_fns.is_empty() || self.pos_inside > 0) && self.neg_inside == 0;
        let depth_ok = self.max_depth.is_none_or(|max| depth <= max);
        fn_ok && depth_ok && !self.over_library() && self.case_matches()
    }

    fn watched(&self, name: &str) -> bool {
        self.watch.is_empty() || self.watch.iter().any(|w| w == name)
    }

    fn stage_id(&mut self) -> u32 {
        match self.stage_of_case {
            Some((case, id)) if case == self.case => id,
            _ => {
                let id = self.rec.log.stage(&self.stage());
                self.stage_of_case = Some((self.case, id));
                id
            }
        }
    }

    fn stage(&self) -> String {
        match self.case {
            Some(0) => "setup".to_string(),
            Some(case) => format!("test case {case}"),
            None => "main".to_string(),
        }
    }

    /// `@!multitest` is reached: what is declared so far stays in scope
    /// in every test case.
    pub fn divider(&mut self) {
        self.touch_all();
        self.setup = self.visible.clone();
    }

    /// A test case starts: its variables and its output start anew.
    pub fn start_case(&mut self, case: u64) {
        self.touch_all();
        self.case = Some(case);
        self.visible = self.setup.clone();
        self.prints = 0;
        self.rec.log.new_case();
    }

    fn checkpoint(&mut self, name: &str) -> String {
        let name = match self.case {
            Some(case) if case > 0 => format!("case {case} · {name}"),
            _ => name.to_string(),
        };
        let count = self.counts.entry(name.clone()).or_insert(0);
        *count += 1;
        match *count {
            1 => name,
            n => format!("{name} #{n}"),
        }
    }

    /// The variable of global `id`, made the first time it is in scope.
    fn var_of(&mut self, id: usize, view: &View) -> Option<usize> {
        if let Some(var) = self.of_global[id] {
            return Some(var);
        }
        let global = &view.program.globals[id];
        // The compiler's own variables have no place in the file.
        if global.span.end == 0
            || matches!(global.ty, Ty::Unit | Ty::Fn(..) | Ty::DomainVal | Ty::Error)
        {
            return None;
        }
        let (kind, extra) = classify(&global.ty, view);
        let taken = self
            .rec
            .vars
            .iter()
            .filter(|v| v.name == global.name)
            .count();
        let id_text = match taken {
            0 => global.name.clone(),
            n => format!("{} ({})", global.name, n + 1),
        };
        let var = self.add_var(
            Var {
                id: id_text,
                name: global.name.clone(),
                ty: view.program.show_ty(&global.ty),
                kind: kind_name(kind),
                extra,
                compare: false,
                window: false,
            },
            kind,
        );
        self.of_global[id] = Some(var);
        Some(var)
    }

    /// Records a step.
    pub fn step(
        &mut self,
        view: &View,
        span: Span,
        label: Option<String>,
        checkpoint: Option<&str>,
        focus: Option<(usize, &[usize])>,
    ) {
        if !self.recording(view.depth) || self.skipped() {
            return;
        }
        // What is above `@!multitest` is the same on every input: it gets
        // a share of the trace, and the test cases get the rest.
        let len = self.rec.log.len();
        if self.last_steps.is_none() {
            let bytes = self.rec.log.bytes();
            if self.case == Some(0)
                && (len >= SETUP_STEPS.max(self.max_steps / 10) || bytes >= self.max_bytes / 16)
            {
                return;
            }
            if len >= self.max_steps || bytes >= self.max_bytes {
                self.rec.truncated = true;
                self.stopped |= self.stop_when_full;
                return;
            }
        }
        self.last_step_at = (self.entries, view.depth);
        self.rec.log.snapshot_if_due();
        let mut touched = std::mem::take(&mut self.touched);
        if std::mem::replace(&mut self.all_touched, false) {
            (0..view.globals.len()).for_each(|id| self.record_global(id, view, None));
        } else {
            touched
                .iter()
                .for_each(|&(id, item)| self.record_global(id, view, item));
            #[cfg(debug_assertions)]
            self.check_untouched(view);
        }
        touched.clear();
        self.touched = touched;
        let shown = |value: &Value| {
            !matches!(
                value,
                Value::Unit | Value::Moved | Value::Closure(..) | Value::Alias(..) | Value::Part(_)
            )
        };
        if let Some(raw) = view.raw {
            self.raw_locals(raw);
            return self.step_rest(view, span, label, checkpoint, focus);
        }
        let (mut names, mut ints) = (
            std::mem::take(&mut self.local_names),
            std::mem::take(&mut self.local_ints),
        );
        ints.clear();
        let (mut raw, mut same) = (true, true);
        for (name, ty, value) in view.locals.iter().filter(|(_, _, value)| shown(value)) {
            match (ty, value) {
                (Ty::Int, Value::Int(v)) if v.unsigned_abs() <= MAX_EXACT => ints.push(*v),
                _ => {
                    raw = false;
                    break;
                }
            }
            same &= self.local_name_at.get(ints.len() - 1)
                == Some(&(name.as_ptr() as usize, name.len()));
        }
        if raw {
            if !same || ints.len() != self.local_name_at.len() {
                self.local_name_at.clear();
                names.clear();
                for (name, _, _) in view.locals.iter().filter(|(_, _, value)| shown(value)) {
                    self.local_name_at
                        .push((name.as_ptr() as usize, name.len()));
                    names.push('\n');
                    string(&mut names, name);
                }
            }
            self.rec.log.locals_ints(&names, &ints);
            (self.local_names, self.local_ints) = (names, ints);
        } else {
            (self.local_names, self.local_ints) = (names, ints);
            self.json_locals(view);
        }
        self.step_rest(view, span, label, checkpoint, focus);
    }

    fn json_locals(&mut self, view: &View) {
        let mut locals = std::mem::take(&mut self.locals);
        locals.clear();
        locals.push('{');
        for (name, ty, value) in &view.locals {
            if matches!(
                value,
                Value::Unit | Value::Moved | Value::Closure(..) | Value::Alias(..) | Value::Part(_)
            ) {
                continue;
            }
            if locals.len() > 1 {
                locals.push(',');
            }
            string(&mut locals, name);
            locals.push(':');
            if cells(value) > 200 {
                string(&mut locals, &format!("{} values", cells(value)));
            } else {
                render(&mut locals, value, ty, view, Mode::Json);
            }
        }
        locals.push('}');
        self.rec.log.locals_changed(&locals);
        self.locals = locals;
    }

    fn step_rest(
        &mut self,
        view: &View,
        span: Span,
        label: Option<String>,
        checkpoint: Option<&str>,
        focus: Option<(usize, &[usize])>,
    ) {
        let line = self.step_line(view, span);
        let focus = focus.and_then(|(global, path)| {
            let var = self.of_global[global]?;
            let fits = match self.kinds[var] {
                Kind::Array | Kind::Parents => path.len() == 1,
                Kind::Grid | Kind::Dp => path.len() == 2,
                _ => false,
            };
            fits.then_some((var, path))
        });
        let checkpoint = checkpoint.map(|name| self.checkpoint(name));
        let frame = self.record_frames.then(|| self.frame_of(view));
        let stage = self.stage_id();
        self.rec
            .log
            .step(line, frame, stage, label, checkpoint, focus);
        if let Some(limit) = self.last_steps.filter(|&limit| limit > 0) {
            if self.rec.log.len() >= 2 * limit {
                self.rec.log.keep_last(limit);
                self.rec.truncated = true;
            }
        }
    }

    /// Global `id` was written, item `item` of it alone when that is known.
    fn touch(&mut self, id: usize, item: Option<usize>) {
        if self.touched.len() >= 16 {
            self.touch_all();
        } else if !self.all_touched {
            self.touched.push((id, item));
        }
    }

    /// Any global may have changed, e.g. through a call's `mut` parameter.
    pub(crate) fn touch_all(&mut self) {
        self.all_touched = true;
        self.touched.clear();
    }

    /// Records the value of global `id`, or only its item `item`.
    fn record_global(&mut self, id: usize, view: &View, item: Option<usize>) {
        if !self.visible[id] {
            if let Some(var) = self.of_global[id] {
                self.rec.log.unset(var);
            }
            return;
        }
        let Some(var) = self.var_of(id, view) else {
            return;
        };
        let value = &view.globals[id];
        // Lent to a call that is still running: it reads as it was.
        if matches!(value, Value::Moved | Value::Unit) {
            return;
        }
        let ty = &view.program.globals[id].ty;
        if let (Kind::Scalar, Value::Int(v), Ty::Int) = (self.kinds[var], value, ty) {
            if v.unsigned_abs() <= MAX_EXACT {
                return self.rec.log.set_int(var, *v);
            }
        }
        if let Some((i, v)) = item
            .filter(|_| self.kinds[var] == Kind::Array)
            .and_then(|i| Some((i, int_item(value, ty, i)?)))
        {
            if self.rec.log.set_int_item(var, i, v) {
                return;
            }
        }
        if let Some(json) = item
            .filter(|_| self.kinds[var] == Kind::Array)
            .and_then(|i| item_json(value, ty, i, view))
        {
            if self.rec.log.set_item(var, item.unwrap_or_default(), &json) {
                return;
            }
        }
        let (json, tree) = snapshot(value, ty, self.kinds[var], view);
        self.rec.vars[var].window |= drawn_by_window(value, ty, self.kinds[var]);
        if self.kinds[var] == Kind::Parents {
            match tree {
                Some(tree) => self.rec.log.set_tree(var, &tree),
                None => self.tree[var] = false,
            }
        }
        self.rec.log.set(var, &json);
    }

    /// Debug builds check that the globals no write was seen to touch hold
    /// what they held: a write the tracer is not told of would show a stale value.
    #[cfg(debug_assertions)]
    fn check_untouched(&self, view: &View) {
        for id in 0..view.globals.len() {
            let (Some(var), value) = (self.of_global[id], &view.globals[id]) else {
                continue;
            };
            if !self.visible[id] || matches!(value, Value::Moved | Value::Unit) {
                continue;
            }
            let (json, _) = snapshot(value, &view.program.globals[id].ty, self.kinds[var], view);
            let name = &view.program.globals[id].name;
            assert_eq!(
                self.rec.log.value_now(var).as_deref(),
                Some(json.as_str()),
                "`{name}` changed unseen"
            );
        }
    }

    /// `mut` work on global `id` (`push`, `sort`, ...) is done.
    pub fn mutated(&mut self, id: usize, view: &View, span: Span) {
        self.visible[id] = true;
        self.touch(id, None);
        self.step(view, span, None, None, None);
    }

    /// A global was declared or set as a whole. Lines that declare
    /// several (`let (n, q) = read()`) are one step.
    pub fn declared(&mut self, id: usize, view: &View, span: Span) {
        self.touch(id, None);
        let fresh = !self.visible[id];
        self.visible[id] = true;
        if view.program.globals[id].span.end == 0 {
            return;
        }
        if !self.watched(&view.program.globals[id].name) || !self.recording(view.depth) {
            return;
        }
        let line = self.step_line(view, span);
        let stage = self.stage_id();
        let same = self.rec.log.last().is_some_and(|(last, note, at)| {
            last == line && note.is_none_or(|note| note.checkpoint.is_none()) && at == stage
        });
        if fresh && same {
            self.rec.log.reopen();
        }
        self.skip_declaration(fresh, (line, stage));
        self.step(view, span, None, None, None);
        self.skipped_at = (self.rec.log.len() == 0).then_some((line, stage));
    }

    /// A part of global `id` changed (e.g. an array cell or struct field).
    pub fn changed(&mut self, id: usize, view: &View, span: Span, path: &[usize]) {
        self.touch(id, (path.len() == 1).then(|| path[0]));
        if !self.watched(&view.program.globals[id].name) {
            return;
        }
        self.step(view, span, None, None, Some((id, path)));
    }

    /// Local `name` of a running call changed. Top-level code has only
    /// globals worth a step; its loop counters would double the trace.
    pub fn local_changed(&mut self, view: &View, span: Span, name: &str) {
        if view.depth > 0 {
            self.touch_all();
        }
        if view.depth == 0 || name.is_empty() || name.starts_with('_') || !self.watched(name) {
            return;
        }
        self.step(view, span, None, None, None);
    }

    /// Local `name` of the innermost call was given a value.
    pub fn assigned_local(&mut self, view: &View, span: Span, name: &str) {
        if let Some((_, ty, value)) = view.locals.iter().rev().find(|(local, ..)| *local == name) {
            self.assigned(view, span, name, ty, value);
        }
    }

    /// Global `id` was given a value, or a part of it was.
    pub fn assigned_global(&mut self, view: &View, span: Span, id: usize) {
        let global = &view.program.globals[id];
        self.assigned(view, span, &global.name, &global.ty, &view.globals[id]);
    }

    fn assigned(&mut self, view: &View, span: Span, name: &str, ty: &Ty, value: &Value) {
        if !self.assignments
            || self.rec.assigned.len() >= MAX_ASSIGNED
            || name.is_empty()
            || name.starts_with('_')
            || matches!(value, Value::Unit | Value::Moved)
            || matches!(ty, Ty::Unit | Ty::Fn(..) | Ty::DomainVal | Ty::Error)
            || !self.recording(view.depth)
        {
            return;
        }
        let mut text = String::new();
        if cells(value) > 200 {
            text = format!("{} values", cells(value));
        } else {
            render(&mut text, value, ty, view, Mode::Text);
        }
        let line = self.step_line(view, span);
        self.rec.assigned.push(Assigned {
            line,
            name: name.to_string(),
            value: text,
            read: view.read,
        });
    }

    /// `print(...)` wrote `text`.
    pub fn printed(&mut self, view: &View, span: Span, text: &str) {
        if self.over_library() && self.case_matches() {
            // What a stepped-over call prints shows at the next step.
            self.rec.log.print(text);
            return;
        }
        if !self.recording(view.depth) {
            return;
        }
        self.rec.log.print(text);
        self.prints += 1;
        let shown: String = text.trim_end().chars().take(60).collect();
        let name = format!("print {}", self.prints);
        self.step(
            view,
            span,
            Some(format!("print → {shown}")),
            Some(&name),
            None,
        );
    }

    /// `debug(...)` ran: `name` is its label or its arguments as written,
    /// `text` what it shows, and `json` the values as one JSON value.
    pub fn debugged(&mut self, view: &View, span: Span, name: &str, text: String, json: String) {
        let line = self.step_line(view, span);
        if !self.recording(view.depth) {
            return;
        }
        let var = match self.debug_vars.get(name) {
            Some(var) => *var,
            None => {
                let var = self.add_var(
                    Var {
                        id: format!("debug: {name}"),
                        name: format!("debug({name})"),
                        ty: format!("line {line}"),
                        kind: "json",
                        extra: String::new(),
                        compare: true,
                        window: false,
                    },
                    Kind::Json,
                );
                self.debug_vars.insert(name.to_string(), var);
                var
            }
        };
        self.rec.log.set(var, &json);
        let checkpoint = format!("debug {name}");
        self.step(
            view,
            span,
            Some(format!("debug: {text}")),
            Some(&checkpoint),
            None,
        );
        let name = match self
            .rec
            .log
            .last()
            .and_then(|(_, note, _)| note?.checkpoint.clone())
        {
            Some(checkpoint) => checkpoint,
            None => checkpoint,
        };
        self.rec.shown.push(Shown { line, name, text });
    }

    /// The run is over: the last step, and each variable's last value.
    pub fn finish(mut self, view: &View, failed: Option<(&str, Span)>) -> Recording {
        self.finishing = true;
        match failed {
            Some((message, span)) => {
                let label = Some(format!("failed: {message}"));
                // The core has left the trapping call's frame by now: when
                // the last step was in that same call, its locals stand.
                if view.locals.is_empty()
                    && view.depth > 0
                    && self.last_step_at == (self.entries, view.depth)
                {
                    self.step_rest(view, span, label, None, None)
                } else {
                    self.step(view, span, label, None, None)
                }
            }
            None if self.stopped => {}
            None => {
                let last = view
                    .program
                    .main
                    .stmts
                    .iter()
                    .rev()
                    .find_map(cpl_ir::Stmt::span);
                let at = last.map_or(self.source.trim_end().len() as u32, |span| span.end);
                let end = Span::new(at, at);
                self.step(
                    view,
                    end,
                    Some("the program ended".to_string()),
                    Some("end"),
                    None,
                )
            }
        }
        if self.rec.log.len() == 0 && self.rec.skipped == 0 {
            self.step(
                view,
                Span::default(),
                Some("nothing was recorded".to_string()),
                None,
                None,
            );
        }
        while !self.open.is_empty() {
            self.leave_call(None);
        }
        if let Some(limit) = self
            .last_steps
            .filter(|&limit| limit > 0 && self.rec.log.len() > limit)
        {
            self.rec.log.keep_last(limit);
            self.rec.truncated = true;
        }
        self.rec.log.shrink();
        let lives = last::last_values(view, |id| self.visible[id], |span| self.line(span));
        self.rec.lives = lives;
        // A `[V]?V` that was a rooted tree at every step is drawn as one.
        for var in 0..self.rec.vars.len() {
            if self.kinds[var] != Kind::Parents {
                continue;
            }
            if self.tree[var] {
                self.rec.vars[var].kind = "tree";
                self.rec.vars[var].extra = String::new();
                self.rec.log.draw_as_tree(var);
            }
        }
        self.rec
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_collection_of_numbers_has_a_cell_per_item() {
        let ints = |n: i64| Value::List(std::rc::Rc::new((0..n).map(Value::Int).collect()));
        assert_eq!(cells(&ints(5)), 6);
        assert_eq!(
            cells(&Value::List(std::rc::Rc::new(vec![ints(2), ints(3)]))),
            1 + 3 + 4
        );
        assert_eq!(cells(&Value::List(std::rc::Rc::new(Vec::new()))), 1);
    }

    use super::*;

    fn run(assigned: &[(u32, &str, &str)]) -> Recording {
        Recording {
            assigned: assigned
                .iter()
                .map(|&(line, name, value)| Assigned {
                    line,
                    name: name.to_string(),
                    value: value.to_string(),
                    read: 0,
                })
                .collect(),
            ..Recording::default()
        }
    }

    #[test]
    fn the_first_divergence_is_the_earliest_value_the_solution_gives_differently() {
        let fast = run(&[
            (1, "i", "0"),
            (2, "s", "0"),
            (3, "s", "0"),
            (3, "t", "5"),
            (4, "s", "7"),
        ]);
        let slow = run(&[(1, "i", "0"), (9, "s", "0"), (9, "t", "6"), (10, "s", "8")]);
        // `s` repeats 0 and stays in step; `t` is the first value that differs.
        assert_eq!(
            first_divergence(&fast, &slow),
            Some(Divergence {
                name: "t".to_string(),
                nth: 1,
                fast: Some((3, "5".to_string())),
                brute: Some((9, "6".to_string())),
                inside: Vec::new(),
                blame: Vec::new(),
                call: false,
                stuck: false,
            })
        );
    }

    #[test]
    fn a_value_one_side_never_gives_is_a_divergence_and_loop_order_is_not() {
        // The same values in another order, ending the same, are a loop in another order.
        let fast = run(&[(1, "x", "1"), (1, "x", "2"), (1, "x", "3"), (2, "s", "1")]);
        let slow = run(&[
            (5, "x", "2"),
            (5, "x", "1"),
            (5, "x", "3"),
            (6, "s", "1"),
            (6, "s", "3"),
        ]);
        let d = first_divergence(&fast, &slow).unwrap();
        assert_eq!((d.name.as_str(), d.nth, d.fast), ("s", 2, None));
        assert_eq!(d.brute, Some((6, "3".to_string())));
        // Names only one side has say nothing.
        assert_eq!(
            first_divergence(&run(&[(1, "a", "1")]), &run(&[(1, "b", "2")])),
            None
        );
    }

    #[test]
    fn members_splits_a_written_object_at_its_top_level() {
        let object = r#"{"a":1,"b":[1,{"c":2}],"s":"x,\"}y","e":{}}"#;
        assert_eq!(
            members(object),
            [
                ("a", "1"),
                ("b", "[1,{\"c\":2}]"),
                ("s", r#""x,\"}y""#),
                ("e", "{}")
            ]
        );
        assert!(members("{}").is_empty());
    }
}
