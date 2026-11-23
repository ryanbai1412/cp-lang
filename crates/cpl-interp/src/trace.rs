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
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;

mod compare;
mod glob;
mod lines;
mod log;
mod raw;
mod trap;
mod window;
pub use compare::{first_divergence, stages, Divergence, Stage};
use glob::glob_match;
pub use log::Steps;
pub use trap::{shown, trap_config, trap_locals};
pub use window::{resumed_tail, resumed_tail_every, tail, Replay, Timeline};

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

    /// The innermost call running at `step`, or the call `frame` levels out
    /// from it.
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

/// Where `after`, a run recorded again after an edit, first parts from
/// `before`. Steps are compared in order: the n-th change of one run against
/// the n-th of the other, `stdout` included.
pub fn first_change(before: &Recording, after: &Recording) -> Option<Change> {
    first_change_in(before, 0..before.len(), after, 0..after.len())
}

/// [`first_change`] over the steps `old` of `before` and `new` of `after`.
pub fn first_change_in(
    before: &Recording,
    old: std::ops::Range<usize>,
    after: &Recording,
    new: std::ops::Range<usize>,
) -> Option<Change> {
    let label = |run: &Recording, i: usize| run.step(i).map(|s| s.label);
    let last = new
        .end
        .saturating_sub(1)
        .max(new.start)
        .min(after.len().saturating_sub(1));
    let (mut was, mut now) = (before.log.start(old.start), after.log.start(new.start));
    for i in 0..old.len().max(new.len()) {
        let (b, a) = (old.start + i, new.start + i);
        if i >= old.len() || i >= new.len() {
            let step = a.min(last);
            return Some(Change {
                step,
                line: after.line(step).unwrap_or(0),
                variable: None,
                before: label(before, b).filter(|_| i < old.len()),
                after: label(after, a).filter(|_| i < new.len()),
            });
        }
        before.log.advance(&mut was, b);
        after.log.advance(&mut now, a);
        // Until now the two held the same: only what this step changed can differ.
        let ids: Vec<&str> = match i {
            0 => before
                .vars
                .iter()
                .chain(&after.vars)
                .map(|v| v.id.as_str())
                .collect(),
            _ => {
                let (old_vars, old_other) = before.log.touched(b);
                let (new_vars, new_other) = after.log.touched(a);
                let mut ids: Vec<&str> = old_vars
                    .iter()
                    .map(|&v| before.vars[v].id.as_str())
                    .collect();
                ids.extend(new_vars.iter().map(|&v| after.vars[v].id.as_str()));
                if old_other || new_other {
                    ids.push("stdout");
                }
                ids
            }
        };
        let value = |run: &Recording, state: &log::State, id: &str| {
            run.vars
                .iter()
                .position(|v| v.id == id)
                .and_then(|var| run.log.value_in(state, var))
        };
        let differs = ids.into_iter().find_map(|id| {
            let (b, a) = (value(before, &was, id), value(after, &now, id));
            (b != a).then(|| (id.to_string(), b, a))
        });
        let differs = differs.or_else(|| {
            let (was, now) = (before.log.locals_in(&was), after.log.locals_in(&now));
            let (b, a) = (members(&was), members(&now));
            let (b, a): (BTreeMap<_, _>, BTreeMap<_, _>) =
                (b.into_iter().collect(), a.into_iter().collect());
            let id = b
                .keys()
                .chain(a.keys())
                .find(|id| b.get(*id) != a.get(*id))?;
            Some((
                id.to_string(),
                b.get(id).map(|v| v.to_string()),
                a.get(id).map(|v| v.to_string()),
            ))
        });
        if let Some((variable, was, now)) = differs {
            return Some(Change {
                step: a,
                line: after.line(a).unwrap_or(0),
                variable: Some(variable),
                before: was,
                after: now,
            });
        }
    }
    None
}

/// The members of a JSON object the tracer wrote, as written.
fn members(object: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    let (mut key, mut start) = (None, 1);
    for (i, c) in object.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ':' if depth == 1 && key.is_none() => key = Some(object[start..i].trim_matches('"')),
            _ => {}
        }
        if (c == ',' && depth == 1) || (c == '}' && depth == 0) {
            if let Some(name) = key.take() {
                let value_start = object[start..i].find(':').map_or(i, |at| start + at + 1);
                out.push((name, &object[value_start..i]));
            }
            start = i + 1;
        }
    }
    out
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
        (pos, neg)
    }

    pub fn leave_fn(&mut self, (pos, neg): (bool, bool)) {
        self.touch_all();
        if pos {
            self.pos_inside -= 1;
        }
        if neg {
            self.neg_inside -= 1;
        }
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

    fn recording(&self, depth: usize) -> bool {
        if self.finishing {
            return true;
        }
        let fn_ok = (self.include_fns.is_empty() || self.pos_inside > 0) && self.neg_inside == 0;
        let depth_ok = self.max_depth.is_none_or(|max| depth <= max);
        fn_ok && depth_ok && self.case_matches()
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
    fn touch_all(&mut self) {
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
        });
    }

    /// `print(...)` wrote `text`.
    pub fn printed(&mut self, view: &View, span: Span, text: &str) {
        if !self.recording(view.depth) {
            return;
        }
        for line in text.lines() {
            self.rec.log.print(line);
        }
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
        for id in 0..view.globals.len() {
            let global = &view.program.globals[id];
            let value = &view.globals[id];
            if global.span.end == 0
                || !self.visible[id]
                || matches!(value, Value::Unit | Value::Moved)
            {
                continue;
            }
            if matches!(global.ty, Ty::Unit | Ty::Fn(..) | Ty::DomainVal | Ty::Error) {
                continue;
            }
            let mut text = String::new();
            if cells(value) > 200 {
                text = format!("{} values", cells(value));
            } else {
                render(&mut text, value, &global.ty, view, Mode::Text);
            }
            self.rec.lives.push(Live {
                line: self.line(global.span),
                name: global.name.clone(),
                value: text,
            });
        }
        self.rec.lives.sort_by_key(|live| live.line);
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

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Scalar => "scalar",
        Kind::Array | Kind::Parents => "array",
        Kind::Grid => "grid",
        Kind::Dp => "dp",
        Kind::Adjacency | Kind::Csr => "graph",
        Kind::Rooted => "tree",
        Kind::Json => "json",
    }
}

/// One value the viewer shows in a cell.
fn is_cell(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => true,
        Ty::Opt(inner) => is_cell(inner, program),
        Ty::Enum(id) => program.enums[*id]
            .variants
            .iter()
            .all(|(_, fields)| fields.is_empty()),
        _ => false,
    }
}

/// The domain a neighbour list leads into: `V` in `List<V>` and in
/// `List<(V, Int)>`.
fn target(elem: &Ty) -> Option<usize> {
    let Ty::List(item) = elem else {
        return None;
    };
    match &**item {
        Ty::Pos(d) => Some(*d),
        Ty::Tuple(fields) => match fields.first() {
            Some(Ty::Pos(d)) => Some(*d),
            _ => None,
        },
        Ty::Record(fields) => match fields.first() {
            Some((_, Ty::Pos(d))) => Some(*d),
            _ => None,
        },
        _ => None,
    }
}

fn labels(dim: Dim, len: i64, view: &View) -> String {
    let lo = match dim {
        Dim::Pos(d) | Dim::Bound(d) => view.domains[d].0,
        Dim::Sym(_) => 0,
        Dim::Enum(_) => 0,
        Dim::Int => 0,
        Dim::IntFrom(first) => first,
    };
    let mut out = String::from("[");
    for i in 0..len.clamp(0, 1000) {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&(lo + i).to_string());
    }
    out.push(']');
    out
}

fn dim_len(dim: Dim, view: &View) -> Option<i64> {
    match dim {
        Dim::Pos(d) => Some(view.domains[d].1),
        Dim::Bound(d) => Some(view.domains[d].1 + 1),
        Dim::Sym(d) => {
            let n = view.domains[d].1;
            Some(n * (n + 1) / 2)
        }
        Dim::Enum(e) => Some(view.program.enums[e].variants.len() as i64),
        Dim::Int | Dim::IntFrom(_) => None,
    }
}

fn dim_name(dim: Dim, view: &View<'_>) -> Option<String> {
    match dim {
        Dim::Pos(d) => Some(view.program.domains[d].clone()),
        Dim::Bound(d) => Some(format!("{}.cuts", view.program.domains[d])),
        Dim::Sym(d) => Some(format!("Sym<{}>", view.program.domains[d])),
        Dim::Enum(e) => Some(view.program.enums[e].name.clone()),
        Dim::Int | Dim::IntFrom(_) => None,
    }
}

/// How a variable of type `ty` is drawn, and the rest of its descriptor.
fn classify(ty: &Ty, view: &View) -> (Kind, String) {
    let program = view.program;
    match ty {
        _ if is_cell(ty, program) => (Kind::Scalar, String::new()),
        Ty::Array { dims, elem } if dims.len() == 1 => {
            let dim = dims[0];
            if let (Dim::Pos(d), Some(to)) = (dim, target(elem)) {
                if d == to {
                    return (Kind::Adjacency, ",\"directed\":true".to_string());
                }
            }
            let mut extra = String::new();
            if let Some(len) = dim_len(dim, view) {
                extra = format!(",\"labels\":{}", labels(dim, len, view));
            }
            match &**elem {
                Ty::Opt(inner) if matches!(dim, Dim::Pos(d) if **inner == Ty::Pos(d)) => {
                    (Kind::Parents, extra)
                }
                elem if is_cell(elem, program) => (Kind::Array, extra),
                // A row of named cells per index: the viewer draws a table
                // with the index's labels down the side.
                elem if field_names(elem, program).is_some()
                    && field_tys(elem, program)
                        .iter()
                        .all(|ty| is_cell(ty, program)) =>
                {
                    (Kind::Json, extra)
                }
                _ => (Kind::Json, String::new()),
            }
        }
        Ty::Array { dims, elem } if dims.len() == 2 && is_cell(elem, program) => {
            let mut extra = String::new();
            for (dim, labels_key, domain_key) in [
                (dims[0], "rowLabels", "rowDomain"),
                (dims[1], "colLabels", "colDomain"),
            ] {
                if let Some(len) = dim_len(dim, view) {
                    extra += &format!(",\"{labels_key}\":{}", labels(dim, len, view));
                }
                if let Some(name) = dim_name(dim, view) {
                    extra += &format!(",\"{domain_key}\":");
                    string(&mut extra, &name);
                }
            }
            let kind = match &**elem {
                Ty::Char | Ty::Bool | Ty::Enum(_) => Kind::Grid,
                Ty::Opt(inner) if matches!(**inner, Ty::Char | Ty::Bool | Ty::Enum(_)) => {
                    Kind::Grid
                }
                _ => Kind::Dp,
            };
            (kind, extra)
        }
        Ty::List(elem) if is_cell(elem, program) => (Kind::Array, String::new()),
        Ty::Struct(id) => {
            let def = &program.structs[*id];
            let field = |name: &str| {
                def.fields
                    .iter()
                    .find(|(field, _)| field == name)
                    .map(|(_, ty)| ty)
            };
            // Drawn by shape, not by name, so a program's own struct laid
            // out the same way is drawn the same way: offsets into a flat
            // list of arcs that each start with a vertex is a graph, and
            // a root beside a parent per vertex is a tree.
            let arcs_from = |ty: &Ty| match ty {
                Ty::List(arc) => matches!(field_tys(arc, program).first(), Some(Ty::Pos(_))),
                _ => false,
            };
            let parent_per = |ty: &Ty| match ty {
                Ty::Array { elem, .. } => {
                    matches!(&**elem, Ty::Opt(of) if matches!(**of, Ty::Pos(_)))
                }
                _ => false,
            };
            if field("start") == Some(&Ty::List(Box::new(Ty::Int)))
                && field("arcs").is_some_and(arcs_from)
            {
                (Kind::Csr, ",\"directed\":true".to_string())
            } else if matches!(field("root"), Some(Ty::Pos(_)))
                && field("par").is_some_and(parent_per)
            {
                (Kind::Rooted, String::new())
            } else {
                (Kind::Json, String::new())
            }
        }
        _ => (Kind::Json, String::new()),
    }
}

/// How many values `value` holds, to keep huge ones out of a trace.
fn cells(value: &Value) -> usize {
    match value {
        Value::Array(array) => match &array.data {
            crate::value::ArrayData::Values(v) => v.iter().map(cells).sum(),
            _ => array.len(),
        },
        Value::List(items) | Value::Enum(_, items) => 1 + items.iter().map(cells).sum::<usize>(),
        Value::Tuple(TupleVal::OneInt(_)) => 2,
        Value::Tuple(TupleVal::Boxed(items)) => 1 + items.iter().map(cells).sum::<usize>(),
        Value::Map(entries) => entries.iter().map(|(k, v)| cells(k) + cells(v)).sum(),
        Value::Set(items) => items.iter().map(cells).sum(),
        Value::Opt(OptVal::Int(_)) => 1,
        Value::Opt(OptVal::Boxed(inner)) => cells(inner),
        Value::Str(bytes) => 1 + bytes.len() / 64,
        _ => 1,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// For the trace: arrays are JSON arrays, `none` is `null`.
    Json,
    /// For a person: `[0, 3, none]`, `(u: 1, v: 2)`.
    Text,
}

/// `text` as a JSON string safe for embedding in HTML `<script>` tags.
pub fn string(out: &mut String, text: &str) {
    let _ = cpl_json::write_script_safe_string(out, text);
}

fn bytes_text(bytes: &[u8]) -> String {
    bytes.iter().map(|b| *b as char).collect()
}

/// The label of a position or cut, or the number itself.
fn label(value: i64, ty: &Ty, view: &View) -> i64 {
    match ty {
        Ty::Pos(d) | Ty::Bound(d) => value + view.domains[*d].0,
        _ => value,
    }
}

fn field_names(ty: &Ty, program: &Program) -> Option<Vec<String>> {
    match ty {
        Ty::Record(fields) => Some(fields.iter().map(|(name, _)| name.clone()).collect()),
        Ty::Struct(id) => Some(
            program.structs[*id]
                .fields
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
        ),
        _ => None,
    }
}

fn field_tys(ty: &Ty, program: &Program) -> Vec<Ty> {
    match ty {
        Ty::Struct(id) => program.structs[*id]
            .fields
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect(),
        other => other.fields().unwrap_or_default(),
    }
}

/// Writes `value` of type `ty`.
fn render(out: &mut String, value: &Value, ty: &Ty, view: &View, mode: Mode) {
    let json = mode == Mode::Json;
    let sep = if json { "," } else { ", " };
    let list = |out: &mut String,
                items: &mut dyn Iterator<Item = (&Value, &Ty)>,
                open: char,
                close: char| {
        out.push(open);
        for (i, (item, ty)) in items.enumerate() {
            if i > 0 {
                out.push_str(sep);
            }
            render(out, item, ty, view, mode);
        }
        out.push(close);
    };
    match (value, ty) {
        // A `U64` the VM keeps as an `Int` shows as one the tree-walker kept.
        (Value::Int(v), Ty::U64) => string(out, &(*v as u64).to_string()),
        (Value::Int(v), _) => {
            let v = label(*v, ty, view);
            // Past 2^53 a JSON number is no longer exact.
            if json && v.unsigned_abs() > 9_007_199_254_740_991 {
                string(out, &v.to_string());
            } else {
                let _ = write!(out, "{v}");
            }
        }
        // Wider than a JSON number holds exactly: as text.
        (Value::I128(v), _) => string(out, &v.to_string()),
        (Value::U64(v), _) => string(out, &v.to_string()),
        (Value::Float(v), _) => {
            if v.is_finite() {
                let text = format!("{v}");
                out.push_str(&text);
                if json && !text.contains(['.', 'e']) {
                    out.push_str(".0");
                }
            } else if json {
                string(out, &format!("{v}"));
            } else {
                out.push_str(&format!("{v}"));
            }
        }
        (Value::Bool(v), _) => out.push_str(if *v { "true" } else { "false" }),
        (Value::Char(c), _) => {
            if json {
                string(out, &(*c as char).to_string());
            } else {
                out.push_str(&format!("'{}'", *c as char));
            }
        }
        (Value::Str(bytes), _) => string(out, &bytes_text(bytes)),
        (Value::Opt(OptVal::None), _) => out.push_str(if json { "null" } else { "none" }),
        (Value::Opt(OptVal::Int(v)), Ty::Opt(inner_ty)) => {
            render(out, &Value::Int(*v), inner_ty, view, mode)
        }
        (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(inner_ty)) => {
            render(out, inner, inner_ty, view, mode)
        }
        (Value::Range(lo, hi), _) => {
            let text = format!("{lo}..{hi}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::Range128(lo, hi), _) => {
            let text = format!("{lo}..{hi}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::Tuple(fields), _) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let tys = field_tys(ty, view.program);
            match field_names(ty, view.program) {
                Some(names) => {
                    if let (Ty::Struct(id), false) = (ty, json) {
                        let name = &view.program.structs[*id].name;
                        out.push_str(name.split(['<', '_']).next().unwrap_or(name));
                    }
                    out.push(if json { '{' } else { '(' });
                    for (i, ((field, ty), name)) in fields.iter().zip(&tys).zip(&names).enumerate()
                    {
                        if i > 0 {
                            out.push_str(sep);
                        }
                        if json {
                            string(out, name);
                            out.push(':');
                        } else {
                            out.push_str(name);
                            out.push_str(": ");
                        }
                        render(out, field, ty, view, mode);
                    }
                    out.push(if json { '}' } else { ')' });
                }
                None if json => list(out, &mut fields.iter().zip(&tys), '[', ']'),
                None => list(out, &mut fields.iter().zip(&tys), '(', ')'),
            }
        }
        (Value::Enum(tag, payload), Ty::Enum(id)) => {
            let (name, tys) = &view.program.enums[*id].variants[*tag as usize];
            if payload.is_empty() {
                if json {
                    string(out, name);
                } else {
                    out.push_str(name);
                }
            } else if json {
                out.push('{');
                string(out, name);
                out.push(':');
                list(out, &mut payload.iter().zip(tys), '[', ']');
                out.push('}');
            } else {
                out.push_str(name);
                list(out, &mut payload.iter().zip(tys), '(', ')');
            }
        }
        (Value::Array(array), Ty::Array { elem, .. }) => {
            // Rows of rows, the last dimension innermost.
            fn rows(
                out: &mut String,
                data: &[Value],
                dims: &[i64],
                elem: &Ty,
                view: &View,
                mode: Mode,
            ) {
                let sep = if mode == Mode::Json { "," } else { ", " };
                out.push('[');
                match dims {
                    [] | [_] => {
                        for (i, cell) in data.iter().enumerate() {
                            if i > 0 {
                                out.push_str(sep);
                            }
                            render(out, cell, elem, view, mode);
                        }
                    }
                    [rows_n, rest @ ..] => {
                        let width = data.len() / (*rows_n).max(1) as usize;
                        for (i, row) in data.chunks(width.max(1)).enumerate() {
                            if i > 0 {
                                out.push_str(sep);
                            }
                            rows(out, row, rest, elem, view, mode);
                        }
                    }
                }
                out.push(']');
            }
            let vals = array.to_values();
            rows(out, &vals, &array.dims, elem, view, mode);
        }
        (Value::List(items), Ty::List(elem)) => {
            list(out, &mut items.iter().map(|item| (item, &**elem)), '[', ']')
        }
        (Value::Set(items), Ty::Set(elem) | Ty::HashSet(elem)) => {
            let (open, close) = if json { ('[', ']') } else { ('{', '}') };
            list(
                out,
                &mut items.iter().map(|item| (item, &**elem)),
                open,
                close,
            )
        }
        (Value::Map(entries), Ty::Map(key, val) | Ty::HashMap(key, val)) => {
            out.push(if json { '[' } else { '{' });
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str(sep);
                }
                if json {
                    out.push('[');
                    render(out, k, key, view, mode);
                    out.push(',');
                    render(out, v, val, view, mode);
                    out.push(']');
                } else {
                    render(out, k, key, view, mode);
                    out.push_str(": ");
                    render(out, v, val, view, mode);
                }
            }
            out.push(if json { ']' } else { '}' });
        }
        _ if json => string(out, "…"),
        _ => out.push('…'),
    }
}

/// `value` as `debug(...)` shows it.
pub(crate) fn text_of(value: &Value, ty: &Ty, view: &View) -> String {
    let mut out = String::new();
    render(&mut out, value, ty, view, Mode::Text);
    out
}

/// `value` as a JSON value.
/// The named `params` among the locals, as a JSON object.
pub(crate) fn args_json(view: &View, params: &[&str]) -> String {
    let mut out = String::from("{");
    for (name, ty, value) in view
        .locals
        .iter()
        .filter(|(name, ..)| params.contains(name))
    {
        if out.len() > 1 {
            out.push(',');
        }
        string(&mut out, name);
        out.push(':');
        out.push_str(&json_of(value, ty, view));
    }
    out.push('}');
    out
}

pub(crate) fn json_of(value: &Value, ty: &Ty, view: &View) -> String {
    let mut out = String::new();
    if cells(value) > MAX_CELLS {
        string(&mut out, &format!("{} values", cells(value)));
    } else {
        render(&mut out, value, ty, view, Mode::Json);
    }
    out
}

/// A graph value of the viewer from its vertices' labels and its edges.
fn graph(nodes: impl Iterator<Item = i64>, edges: &[(i64, i64, Option<String>)]) -> String {
    let mut out = String::from("{\"nodes\":[");
    for (i, node) in nodes.enumerate() {
        if i > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{node},\"label\":\"{node}\"}}");
    }
    out.push_str("],\"edges\":[");
    for (i, (from, to, weight)) in edges.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{i},\"from\":{from},\"to\":{to}");
        if let Some(weight) = weight {
            out += &format!(",\"weight\":{weight}");
        }
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// The edges a parent array holds, parent to child, when it is one
/// rooted tree: a single root and no cycle.
fn tree_edges(parents: &[Option<i64>]) -> Option<Vec<(i64, i64, Option<String>)>> {
    if parents.iter().filter(|p| p.is_none()).count() != 1 {
        return None;
    }
    let n = parents.len();
    // Every vertex must reach the root.
    let mut state = vec![0u8; n];
    for start in 0..n {
        let mut path = Vec::new();
        let mut at = start;
        while state[at] == 0 {
            state[at] = 1;
            path.push(at);
            match parents[at] {
                Some(parent) if (0..n as i64).contains(&parent) => at = parent as usize,
                Some(_) => return None,
                None => break,
            }
        }
        if state[at] == 1 && parents[at].is_some() {
            return None;
        }
        for v in path {
            state[v] = 2;
        }
    }
    Some(
        parents
            .iter()
            .enumerate()
            .filter_map(|(child, parent)| parent.map(|parent| (parent, child as i64, None)))
            .collect(),
    )
}

/// A parent array as ordinals: `None` at a root.
fn parents_of(value: &Value) -> Option<Vec<Option<i64>>> {
    let Value::Array(array) = value else {
        return None;
    };
    let vals = array.to_values();
    Some(
        vals.iter()
            .map(|cell| match cell {
                Value::Opt(OptVal::Int(parent)) => Some(*parent),
                Value::Opt(OptVal::Boxed(parent)) => Some(parent.int()),
                _ => None,
            })
            .collect(),
    )
}

/// Past this a JSON number is no longer exact.
pub(crate) const MAX_EXACT: u64 = 9_007_199_254_740_991;

/// Item `index` of a one-dimensional array of `Int`s, when its JSON is
/// the number itself.
fn int_item(value: &Value, ty: &Ty, index: usize) -> Option<i64> {
    let (Value::Array(array), Ty::Array { dims, elem }) = (value, ty) else {
        return None;
    };
    if dims.len() != 1 || !matches!(**elem, Ty::Int) || array.len() > MAX_CELLS {
        return None;
    }
    array
        .get_int(index)
        .filter(|v| v.unsigned_abs() <= MAX_EXACT)
}

/// Item `index` of a one-dimensional array or list, as JSON, as it is in
/// the JSON of the whole.
fn item_json(value: &Value, ty: &Ty, index: usize, view: &View) -> Option<String> {
    let (item, elem) = match (value, ty) {
        (Value::Array(array), Ty::Array { dims, elem }) if dims.len() == 1 => {
            (array.get(index)?, &**elem)
        }
        (Value::List(items), Ty::List(elem)) => (items.get(index)?.clone(), &**elem),
        _ => return None,
    };
    let scalar = matches!(
        elem,
        Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Bool | Ty::Char | Ty::Pos(_) | Ty::Bound(_)
    );
    let len = match value {
        Value::Array(array) => array.len(),
        Value::List(items) => items.len() + 1,
        _ => unreachable!("matched above"),
    };
    if (if scalar { len } else { cells(value) }) > MAX_CELLS {
        return None;
    }
    let mut json = String::new();
    render(&mut json, &item, elem, view, Mode::Json);
    Some(json)
}

/// The snapshot of a variable, and for a `[V]?V` its tree form when it
/// is a tree.
fn snapshot(value: &Value, ty: &Ty, kind: Kind, view: &View) -> (String, Option<String>) {
    if cells(value) > MAX_CELLS {
        let mut out = String::new();
        // Too large to draw: an empty value of the right shape, so the
        // viewer still takes the trace.
        match kind {
            Kind::Array | Kind::Parents | Kind::Grid | Kind::Dp => out.push_str("[]"),
            Kind::Adjacency | Kind::Csr | Kind::Rooted => {
                out.push_str("{\"nodes\":[],\"edges\":[]}")
            }
            Kind::Scalar | Kind::Json => string(&mut out, &format!("{} values", cells(value))),
        }
        return (out, None);
    }
    let plain = || {
        let mut out = String::new();
        render(&mut out, value, ty, view, Mode::Json);
        out
    };
    let lo_of = |dim: Option<&Dim>| match dim {
        Some(Dim::Pos(d)) => view.domains[*d].0,
        _ => 0,
    };
    match (kind, value, ty) {
        (Kind::Adjacency, Value::Array(array), Ty::Array { dims, elem }) => {
            let lo = lo_of(dims.first());
            let weighted = match &**elem {
                Ty::List(item) => field_tys(item, view.program).get(1).cloned(),
                _ => None,
            };
            let mut edges = Vec::new();
            let vals = array.to_values();
            for (from, list) in vals.iter().enumerate() {
                let Value::List(items) = list else { continue };
                for item in items.iter() {
                    let (to, weight) = match item {
                        Value::Tuple(fields) => {
                            let mut scratch = [Value::Unit];
                            let fields = fields.as_slice(&mut scratch);
                            (
                                fields.first().map_or(0, Value::int),
                                match (&weighted, fields.get(1)) {
                                    (Some(ty @ (Ty::Int | Ty::Float)), Some(w)) => {
                                        Some(json_of(w, ty, view))
                                    }
                                    _ => None,
                                },
                            )
                        }
                        other => (other.int(), None),
                    };
                    edges.push((from as i64 + lo, to + lo, weight));
                }
            }
            (graph((0..array.len() as i64).map(|i| i + lo), &edges), None)
        }
        (Kind::Parents, _, Ty::Array { dims, .. }) => {
            let lo = lo_of(dims.first());
            let tree = parents_of(value).and_then(|parents| {
                let edges: Vec<_> = tree_edges(&parents)?
                    .into_iter()
                    .map(|(from, to, weight)| (from + lo, to + lo, weight))
                    .collect();
                // The root first: the viewer roots a tree at its first node.
                let root = parents.iter().position(Option::is_none)? as i64;
                let nodes = std::iter::once(root)
                    .chain((0..parents.len() as i64).filter(move |v| *v != root));
                Some(graph(nodes.map(|v| v + lo), &edges))
            });
            (plain(), tree)
        }
        (Kind::Csr, Value::Tuple(fields), Ty::Struct(id)) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let def = &view.program.structs[*id];
            let at = |name: &str| def.fields.iter().position(|(field, _)| field == name);
            let (Some(start), Some(arcs)) = (at("start"), at("arcs")) else {
                return (graph(std::iter::empty(), &[]), None);
            };
            let lo = match &def.fields[arcs].1 {
                Ty::List(item) => match field_tys(item, view.program).first() {
                    Some(Ty::Pos(d)) => view.domains[*d].0,
                    _ => 0,
                },
                _ => 0,
            };
            let (Value::List(start), Value::List(arcs)) = (&fields[start], &fields[arcs]) else {
                return (graph(std::iter::empty(), &[]), None);
            };
            let n = start.len().saturating_sub(1);
            let mut edges = Vec::new();
            for from in 0..n {
                for arc in start[from].int()..start[from + 1].int() {
                    if let Some(Value::Tuple(arc)) = arcs.get(arc as usize) {
                        edges.push((
                            from as i64 + lo,
                            arc.get(0).map_or(0, |v| v.int()) + lo,
                            None,
                        ));
                    }
                }
            }
            (graph((0..n as i64).map(|i| i + lo), &edges), None)
        }
        (Kind::Rooted, Value::Tuple(fields), Ty::Struct(id)) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let def = &view.program.structs[*id];
            let at = |name: &str| def.fields.iter().position(|(field, _)| field == name);
            let empty = (graph(std::iter::empty(), &[]), None);
            let (Some(root), Some(par)) = (at("root"), at("par")) else {
                return empty;
            };
            let lo = match &def.fields[root].1 {
                Ty::Pos(d) => view.domains[*d].0,
                _ => 0,
            };
            let Some(parents) = parents_of(&fields[par]) else {
                return empty;
            };
            let root = fields[root].int();
            // Vertices the root does not reach are left out: a tree has
            // one root.
            let edges: Vec<_> = parents
                .iter()
                .enumerate()
                .filter_map(|(child, parent)| {
                    parent.map(|parent| (parent + lo, child as i64 + lo, None))
                })
                .collect();
            let nodes = std::iter::once(root).chain(
                (0..parents.len() as i64).filter(|v| *v != root && parents[*v as usize].is_some()),
            );
            (graph(nodes.map(|v| v + lo), &edges), None)
        }
        (Kind::Grid | Kind::Dp, Value::Array(array), _)
            if array.dims.len() != 2 || array.is_empty() =>
        {
            ("[]".to_string(), None)
        }
        _ => (plain(), None),
    }
}

/// The trace document of one run, or of a solution (`fast`) and its
/// brute (`slow`) on the same input. The two are lined up on the
/// checkpoints they share, and compared on what both printed and on the
/// `debug(...)`s both ran.
pub fn document(
    title: &str,
    description: &str,
    file: &str,
    source: &str,
    fast: (&str, &Recording),
    slow: Option<(&str, &Recording)>,
) -> String {
    let (fast_label, fast) = fast;
    // One table of variables for both runs: the same name is the same
    // variable when both describe it alike.
    let mut vars: Vec<Var> = fast.vars.clone();
    let fast_ids: Vec<String> = fast.vars.iter().map(|var| var.id.clone()).collect();
    let mut slow_ids = Vec::new();
    let mut dropped = std::collections::HashSet::new();
    if let Some((_, slow)) = slow {
        for var in &slow.vars {
            let same = vars.iter().find(|known| {
                known.id == var.id && known.kind == var.kind && known.extra == var.extra
            });
            match same {
                Some(_) => slow_ids.push(var.id.clone()),
                None if vars.iter().any(|known| known.id == var.id) => {
                    let mut var = var.clone();
                    var.id = format!("{} (brute)", var.id);
                    slow_ids.push(var.id.clone());
                    vars.push(var);
                }
                None => {
                    slow_ids.push(var.id.clone());
                    vars.push(var.clone());
                }
            }
        }
        // Shared checkpoints must come in one order in both runs: one
        // that does not is left as a plain step.
        let position: HashMap<&str, usize> =
            slow.checkpoints().map(|(i, name)| (name, i)).collect();
        let mut reached = None;
        for (_, name) in fast.checkpoints() {
            match position.get(name) {
                Some(at) if reached.is_none_or(|last| *at > last) => reached = Some(*at),
                Some(_) => {
                    dropped.insert(name.to_string());
                }
                None => {}
            }
        }
    }
    let mut out = String::from("{\"schemaVersion\":1,\"title\":");
    string(&mut out, title);
    out.push_str(",\"description\":");
    string(&mut out, description);
    out.push_str(",\"source\":{\"file\":");
    string(&mut out, file);
    out.push_str(",\"code\":");
    string(&mut out, source);
    out.push_str("},\"variables\":{");
    for (i, var) in vars.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(&mut out, &var.id);
        out.push_str(":{\"name\":");
        string(&mut out, &var.name);
        out.push_str(",\"type\":");
        string(&mut out, &var.ty);
        out += &format!(",\"kind\":\"{}\"{}}}", var.kind, var.extra);
    }
    let is_dropped = |name: &str| dropped.contains(name);
    out.push_str("},\"runs\":{\"fast\":");
    out += &compare::run_json(fast_label, fast, &fast_ids, &is_dropped);
    if let Some((slow_label, slow)) = slow {
        out.push_str(",\"slow\":");
        out += &compare::run_json(slow_label, slow, &slow_ids, &is_dropped);
        // Compared: what both runs have and mean alike.
        let compare: Vec<&Var> = vars
            .iter()
            .filter(|var| var.compare && fast_ids.contains(&var.id) && slow_ids.contains(&var.id))
            .collect();
        out.push_str("},\"compare\":[");
        for (i, var) in compare.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            string(&mut out, &var.id);
        }
        out.push(']');
    } else {
        out.push('}');
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(assigned: &[(u32, &str, &str)]) -> Recording {
        Recording {
            assigned: assigned
                .iter()
                .map(|&(line, name, value)| Assigned {
                    line,
                    name: name.to_string(),
                    value: value.to_string(),
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
                call: false,
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
