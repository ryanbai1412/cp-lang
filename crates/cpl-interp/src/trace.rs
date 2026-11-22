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
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

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
}

/// The message of the trap that ends a run at [`TraceConfig::stop_after_call`].
pub(crate) const STOPPED: &str = "the call to replay returned";

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
    /// Tree-form snapshots for `[V]?V` variables that stayed trees.
    tree_values: Vec<(usize, Arc<str>)>,
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
    pub steps: Vec<Step>,
    /// Steps past the limit were not kept.
    pub truncated: bool,
    pub lives: Vec<Live>,
    pub shown: Vec<Shown>,
    /// With [`TraceConfig::assignments`], in the order they ran.
    pub assigned: Vec<Assigned>,
    /// The calls, in the order they started; none with `--last`.
    pub calls: Vec<Call>,
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
    pub fn place(&self, step: usize) -> Option<Place> {
        let line = self.steps.get(step)?.line;
        let visit = self.steps[..=step].iter().filter(|s| s.line == line).count();
        Some(Place { line, visit })
    }

    /// The step at `place`: the same visit to the same line, else the
    /// line's last visit, else the step nearest `fallback`.
    pub fn find(&self, place: Place, fallback: usize) -> usize {
        let mut last = None;
        let mut seen = 0;
        for (i, step) in self.steps.iter().enumerate() {
            if step.line == place.line {
                seen += 1;
                last = Some(i);
                if seen == place.visit {
                    return i;
                }
            }
        }
        last.unwrap_or_else(|| fallback.min(self.steps.len().saturating_sub(1)))
    }

    /// The innermost call running at `step`, or the call `frame` levels out
    /// from it.
    pub fn call_at(&self, step: usize, frame: usize) -> Option<&Call> {
        let open: Vec<&Call> = self.calls.iter().filter(|c| c.entry <= step && step < c.exit).collect();
        let depth = open.iter().map(|c| c.depth).max()?.checked_sub(frame)?;
        open.into_iter().rev().find(|c| c.depth == depth)
    }

    pub(crate) fn values_at(&self, step: usize) -> BTreeMap<&str, &str> {
        self.steps[step].values.iter().map(|(var, json)| (self.vars[*var].id.as_str(), &**json)).collect()
    }
}

/// Where `after`, a run recorded again after an edit, first parts from
/// `before`. Steps are compared in order: the n-th change of one run against
/// the n-th of the other, `stdout` included.
pub fn first_change(before: &Recording, after: &Recording) -> Option<Change> {
    first_change_in(before, 0..before.steps.len(), after, 0..after.steps.len())
}

/// [`first_change`] over the steps `old` of `before` and `new` of `after`.
pub fn first_change_in(
    before: &Recording,
    old: std::ops::Range<usize>,
    after: &Recording,
    new: std::ops::Range<usize>,
) -> Option<Change> {
    let label = |run: &Recording, i: usize| run.steps.get(i).map(|s| s.label.clone());
    let last = new.end.saturating_sub(1).max(new.start).min(after.steps.len().saturating_sub(1));
    for i in 0..old.len().max(new.len()) {
        let (b, a) = (old.start + i, new.start + i);
        if i >= old.len() || i >= new.len() {
            let step = a.min(last);
            return Some(Change {
                step,
                line: after.steps.get(step).map_or(0, |s| s.line),
                variable: None,
                before: label(before, b).filter(|_| i < old.len()),
                after: label(after, a).filter(|_| i < new.len()),
            });
        }
        let (mut was, mut now) = (before.values_at(b), after.values_at(a));
        was.extend(members(&before.steps[b].locals));
        now.extend(members(&after.steps[a].locals));
        let differs = was.keys().chain(now.keys()).find(|id| was.get(*id) != now.get(*id));
        if let Some(id) = differs {
            return Some(Change {
                step: a,
                line: after.steps[a].line,
                variable: Some(id.to_string()),
                before: was.get(id).map(|v| v.to_string()),
                after: now.get(id).map(|v| v.to_string()),
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
    /// The last snapshot of each variable.
    last: Vec<Option<Arc<str>>>,
    /// Whether every snapshot of a `[V]?V` was one rooted tree.
    tree: Vec<bool>,
    steps: VecDeque<Step>,
    stdout: usize,
    printed: Vec<String>,
    debug_vars: HashMap<String, usize>,
    counts: HashMap<String, usize>,
    prints: usize,
    pub case: Option<u64>,
    bytes: usize,
    pub assignments: bool,
    pub rec: Recording,
    /// How many times each fn has been called so far.
    ordinals: HashMap<String, usize>,
    /// The index in `rec.calls` of each running call, if it is kept.
    open: Vec<Option<usize>>,
    stop_after_call: Option<(String, usize)>,
    /// The depth of the call to stop after, while it runs.
    stopping_at: Option<usize>,
    /// The call to stop after has returned.
    pub stopped: bool,
}

/// What the interpreter shows the tracer of its state.
pub(crate) struct View<'a> {
    pub program: &'a Program,
    pub globals: &'a [Value],
    pub domains: &'a [(i64, i64)],
    /// The named locals of the innermost call, with their values.
    pub locals: Vec<(&'a str, &'a Ty, &'a Value)>,
    /// Current call depth (`0` at top level).
    pub depth: usize,
    /// Active call frames `(fn_name, call_site_span)` from outermost to innermost.
    pub call_frames: &'a [(&'a str, Span)],
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut match_t) = (None, 0usize);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            match_t = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            match_t += 1;
            ti = match_t;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
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
            source: config.source,
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
            last: Vec::new(),
            tree: Vec::new(),
            steps: VecDeque::new(),
            stdout: 0,
            printed: Vec::new(),
            debug_vars: HashMap::new(),
            counts: HashMap::new(),
            prints: 0,
            case: None,
            bytes: 0,
            assignments: config.assignments,
            rec: Recording::default(),
            ordinals: HashMap::new(),
            open: Vec::new(),
            stop_after_call: config.stop_after_call,
            stopping_at: None,
            stopped: false,
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
        tracer
    }

    fn add_var(&mut self, var: Var, kind: Kind) -> usize {
        self.rec.vars.push(var);
        self.kinds.push(kind);
        self.last.push(None);
        self.tree.push(kind == Kind::Parents);
        self.rec.vars.len() - 1
    }

    /// A call of `name` starts, with `args` (JSON, by name) when asked for.
    pub fn enter_call(&mut self, name: &str, args: impl FnOnce() -> String) {
        let ordinal = self.ordinals.entry(name.to_string()).or_insert(0);
        *ordinal += 1;
        let ordinal = *ordinal;
        if self.stop_after_call.as_ref().is_some_and(|(n, k)| n == name && *k == ordinal) {
            self.stopping_at = Some(self.open.len());
        }
        let kept = self.last_steps.is_none() && self.rec.calls.len() < MAX_STEPS;
        self.open.push(kept.then_some(self.rec.calls.len()));
        if kept {
            self.rec.calls.push(Call {
                name: name.to_string(),
                ordinal,
                entry: self.steps.len(),
                exit: self.steps.len(),
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
            call.exit = self.steps.len();
            call.returned = returned;
        }
        if self.stopping_at == Some(self.open.len()) {
            self.stopped = true;
        }
    }

    pub fn enter_fn(&mut self, name: &str) -> (bool, bool) {
        let pos = !self.include_fns.is_empty()
            && self.include_fns.iter().any(|p| glob_match(p, name));
        let neg = !self.exclude_fns.is_empty()
            && self.exclude_fns.iter().any(|p| glob_match(p, name));
        if pos {
            self.pos_inside += 1;
        }
        if neg {
            self.neg_inside += 1;
        }
        (pos, neg)
    }

    pub fn leave_fn(&mut self, (pos, neg): (bool, bool)) {
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

    pub fn line(&self, span: Span) -> u32 {
        self.lines.line_of(span.start) + 1
    }

    fn line_text(&self, line: u32) -> String {
        let text = self.source.lines().nth(line as usize - 1).unwrap_or("").trim();
        let mut short: String = text.chars().take(80).collect();
        if short.len() < text.len() {
            short.push('…');
        }
        short
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
        self.setup = self.visible.clone();
    }

    /// A test case starts: its variables and its output start anew.
    pub fn start_case(&mut self, case: u64) {
        self.case = Some(case);
        self.visible = self.setup.clone();
        self.printed.clear();
        self.prints = 0;
        self.last[self.stdout] = None;
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
        if global.span.end == 0 || matches!(global.ty, Ty::Unit | Ty::Fn(..) | Ty::DomainVal | Ty::Error) {
            return None;
        }
        let (kind, extra) = classify(&global.ty, view);
        let taken = self.rec.vars.iter().filter(|v| v.name == global.name).count();
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
        focus: Option<(usize, Vec<usize>)>,
    ) {
        if !self.recording(view.depth) {
            return;
        }
        // What is above `@!multitest` is the same on every input: it gets
        // a share of the trace, and the test cases get the rest.
        if self.last_steps.is_none() {
            if self.case == Some(0)
                && (self.steps.len() >= SETUP_STEPS || self.bytes >= MAX_BYTES / 16)
            {
                return;
            }
            if self.steps.len() >= MAX_STEPS || self.bytes >= MAX_BYTES {
                self.rec.truncated = true;
                return;
            }
        }
        let line = self.line(span).max(1);
        let mut values = Vec::new();
        let mut tree_values = Vec::new();
        for id in 0..view.globals.len() {
            if !self.visible[id] {
                continue;
            }
            let Some(var) = self.var_of(id, view) else {
                continue;
            };
            let value = &view.globals[id];
            // Lent to a call that is still running: it reads as it was.
            if !matches!(value, Value::Moved | Value::Unit) {
                let ty = &view.program.globals[id].ty;
                let (json, tree) = snapshot(value, ty, self.kinds[var], view);
                if self.kinds[var] == Kind::Parents {
                    match tree {
                        Some(tree) => {
                            tree_values.push((var, tree.into()));
                        }
                        None => self.tree[var] = false,
                    }
                }
                if self.last[var].as_deref() != Some(json.as_str()) {
                    self.bytes += json.len();
                    self.last[var] = Some(json.into());
                }
            }
            if let Some(json) = &self.last[var] {
                values.push((var, Arc::clone(json)));
            }
        }
        if self.last[self.stdout].is_none() {
            let mut json = String::from("[");
            for (i, line) in self.printed.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                string(&mut json, line);
            }
            json.push(']');
            self.bytes += json.len();
            self.last[self.stdout] = Some(json.into());
        }
        values.push((self.stdout, Arc::clone(self.last[self.stdout].as_ref().unwrap())));
        for var in self.debug_vars.values() {
            if let Some(json) = &self.last[*var] {
                values.push((*var, Arc::clone(json)));
            }
        }
        values.sort_by_key(|(var, _)| *var);
        let mut locals = String::from("{");
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
        self.bytes += locals.len();
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
        if let Some(limit) = self.last_steps {
            if limit > 0 && self.steps.len() >= limit {
                self.steps.pop_front();
                self.rec.truncated = true;
            }
        }
        let frames = if self.record_frames {
            let mut out = Vec::with_capacity(view.call_frames.len() + 1);
            let current_fn = view
                .call_frames
                .last()
                .map(|(name, _)| (*name).to_string())
                .unwrap_or_else(|| "main".to_string());
            out.push((current_fn, line));
            for i in (0..view.call_frames.len()).rev() {
                let caller = if i == 0 {
                    "main".to_string()
                } else {
                    view.call_frames[i - 1].0.to_string()
                };
                let call_line = self.line(view.call_frames[i].1).max(1);
                out.push((caller, call_line));
            }
            out
        } else {
            Vec::new()
        };
        self.steps.push_back(Step {
            line,
            label: label.unwrap_or_else(|| self.line_text(line)),
            stage: self.stage(),
            values,
            locals,
            checkpoint,
            focus,
            frames,
            tree_values,
        });
    }

    /// A global was declared or set as a whole. Lines that declare
    /// several (`let (n, q) = read()`) are one step.
    pub fn declared(&mut self, id: usize, view: &View, span: Span) {
        let fresh = !self.visible[id];
        self.visible[id] = true;
        if view.program.globals[id].span.end == 0 {
            return;
        }
        if !self.watched(&view.program.globals[id].name) {
            return;
        }
        let line = self.line(span).max(1);
        if fresh
            && self.steps.back().is_some_and(|last| {
                last.line == line && last.checkpoint.is_none() && last.stage == self.stage()
            })
        {
            self.steps.pop_back();
        }
        self.step(view, span, None, None, None);
    }

    /// A part of global `id` changed (e.g. an array cell or struct field).
    pub fn changed(&mut self, id: usize, view: &View, span: Span, path: Vec<usize>) {
        if !self.watched(&view.program.globals[id].name) {
            return;
        }
        self.step(view, span, None, None, Some((id, path)));
    }

    /// Local `name` of a running call changed. Top-level code has only
    /// globals worth a step; its loop counters would double the trace.
    pub fn local_changed(&mut self, view: &View, span: Span, name: &str) {
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
        let line = self.line(span).max(1);
        self.rec.assigned.push(Assigned { line, name: name.to_string(), value: text });
    }

    /// `print(...)` wrote `text`.
    pub fn printed(&mut self, view: &View, span: Span, text: &str) {
        if !self.recording(view.depth) {
            return;
        }
        self.printed.extend(text.lines().map(str::to_string));
        self.last[self.stdout] = None;
        self.prints += 1;
        let shown: String = text.trim_end().chars().take(60).collect();
        let name = format!("print {}", self.prints);
        self.step(view, span, Some(format!("print → {shown}")), Some(&name), None);
    }

    /// `debug(...)` ran: `name` is its label or its arguments as written,
    /// `text` what it shows, and `json` the values as one JSON value.
    pub fn debugged(&mut self, view: &View, span: Span, name: &str, text: String, json: String) {
        let line = self.line(span).max(1);
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
        self.bytes += json.len();
        self.last[var] = Some(json.into());
        let checkpoint = format!("debug {name}");
        self.step(view, span, Some(format!("debug: {text}")), Some(&checkpoint), None);
        let name = match self.steps.back().and_then(|step| step.checkpoint.clone()) {
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
                self.step(view, span, Some(format!("failed: {message}")), None, None)
            }
            None if self.stopped => {}
            None => {
                let end = Span::new(self.source.trim_end().len() as u32, self.source.trim_end().len() as u32);
                self.step(view, end, Some("the program ended".to_string()), Some("end"), None)
            }
        }
        if self.steps.is_empty() {
            self.step(view, Span::default(), Some("nothing was recorded".to_string()), None, None);
        }
        while !self.open.is_empty() {
            self.leave_call(None);
        }
        self.rec.steps = std::mem::take(&mut self.steps).into_iter().collect();
        for id in 0..view.globals.len() {
            let global = &view.program.globals[id];
            let value = &view.globals[id];
            if global.span.end == 0 || !self.visible[id] || matches!(value, Value::Unit | Value::Moved) {
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
                for step in &mut self.rec.steps {
                    if let Some((_, tree)) = step.tree_values.iter().find(|(at, _)| *at == var) {
                        for (at, json) in &mut step.values {
                            if *at == var {
                                *json = Arc::clone(tree);
                            }
                        }
                    }
                    if step.focus.as_ref().is_some_and(|(at, _)| *at == var) {
                        step.focus = None;
                    }
                }
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
        Ty::Enum(id) => program.enums[*id].variants.iter().all(|(_, fields)| fields.is_empty()),
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
                Ty::Opt(inner) if matches!(dim, Dim::Pos(d) if **inner == Ty::Pos(d)) => (Kind::Parents, extra),
                elem if is_cell(elem, program) => (Kind::Array, extra),
                // A row of named cells per index: the viewer draws a table
                // with the index's labels down the side.
                elem if field_names(elem, program).is_some()
                    && field_tys(elem, program).iter().all(|ty| is_cell(ty, program)) =>
                {
                    (Kind::Json, extra)
                }
                _ => (Kind::Json, String::new()),
            }
        }
        Ty::Array { dims, elem } if dims.len() == 2 && is_cell(elem, program) => {
            let mut extra = String::new();
            for (dim, labels_key, domain_key) in
                [(dims[0], "rowLabels", "rowDomain"), (dims[1], "colLabels", "colDomain")]
            {
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
                Ty::Opt(inner) if matches!(**inner, Ty::Char | Ty::Bool | Ty::Enum(_)) => Kind::Grid,
                _ => Kind::Dp,
            };
            (kind, extra)
        }
        Ty::List(elem) if is_cell(elem, program) => (Kind::Array, String::new()),
        Ty::Struct(id) => {
            let def = &program.structs[*id];
            let field = |name: &str| def.fields.iter().find(|(field, _)| field == name).map(|(_, ty)| ty);
            // Drawn by shape, not by name, so a program's own struct laid
            // out the same way is drawn the same way: offsets into a flat
            // list of arcs that each start with a vertex is a graph, and
            // a root beside a parent per vertex is a tree.
            let arcs_from = |ty: &Ty| match ty {
                Ty::List(arc) => matches!(field_tys(arc, program).first(), Some(Ty::Pos(_))),
                _ => false,
            };
            let parent_per = |ty: &Ty| match ty {
                Ty::Array { elem, .. } => matches!(&**elem, Ty::Opt(of) if matches!(**of, Ty::Pos(_))),
                _ => false,
            };
            if field("start") == Some(&Ty::List(Box::new(Ty::Int))) && field("arcs").is_some_and(arcs_from) {
                (Kind::Csr, ",\"directed\":true".to_string())
            } else if matches!(field("root"), Some(Ty::Pos(_))) && field("par").is_some_and(parent_per) {
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
        Value::List(items) | Value::Enum(_, items) => {
            1 + items.iter().map(cells).sum::<usize>()
        }
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

/// `text` as a JSON string.
pub fn string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // `<` would let `</script>` end the page a trace is embedded in.
            '<' => out.push_str("\\u003c"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
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
        Ty::Struct(id) => Some(program.structs[*id].fields.iter().map(|(name, _)| name.clone()).collect()),
        _ => None,
    }
}

fn field_tys(ty: &Ty, program: &Program) -> Vec<Ty> {
    match ty {
        Ty::Struct(id) => program.structs[*id].fields.iter().map(|(_, ty)| ty.clone()).collect(),
        other => other.fields().unwrap_or_default(),
    }
}

/// Writes `value` of type `ty`.
fn render(out: &mut String, value: &Value, ty: &Ty, view: &View, mode: Mode) {
    let json = mode == Mode::Json;
    let sep = if json { "," } else { ", " };
    let list = |out: &mut String, items: &mut dyn Iterator<Item = (&Value, &Ty)>, open: char, close: char| {
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
        (Value::Int(v), _) => {
            let v = label(*v, ty, view);
            // Past 2^53 a JSON number is no longer exact.
            if json && v.unsigned_abs() > 9_007_199_254_740_991 {
                string(out, &v.to_string());
            } else {
                out.push_str(&v.to_string());
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
                    for (i, ((field, ty), name)) in fields.iter().zip(&tys).zip(&names).enumerate() {
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
            fn rows(out: &mut String, data: &[Value], dims: &[i64], elem: &Ty, view: &View, mode: Mode) {
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
        (Value::List(items), Ty::List(elem)) => list(out, &mut items.iter().map(|item| (item, &**elem)), '[', ']'),
        (Value::Set(items), Ty::Set(elem) | Ty::HashSet(elem)) => {
            let (open, close) = if json { ('[', ']') } else { ('{', '}') };
            list(out, &mut items.iter().map(|item| (item, &**elem)), open, close)
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
    for (name, ty, value) in view.locals.iter().filter(|(name, ..)| params.contains(name)) {
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
        vals
            .iter()
            .map(|cell| match cell {
                Value::Opt(OptVal::Int(parent)) => Some(*parent),
                Value::Opt(OptVal::Boxed(parent)) => Some(parent.int()),
                _ => None,
            })
            .collect(),
    )
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
            Kind::Adjacency | Kind::Csr | Kind::Rooted => out.push_str("{\"nodes\":[],\"edges\":[]}"),
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
                let nodes = std::iter::once(root).chain((0..parents.len() as i64).filter(move |v| *v != root));
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
                .filter_map(|(child, parent)| parent.map(|parent| (parent + lo, child as i64 + lo, None)))
                .collect();
            let nodes = std::iter::once(root).chain(
                (0..parents.len() as i64).filter(|v| *v != root && parents[*v as usize].is_some()),
            );
            (graph(nodes.map(|v| v + lo), &edges), None)
        }
        (Kind::Grid | Kind::Dp, Value::Array(array), _) if array.dims.len() != 2 || array.is_empty() => {
            ("[]".to_string(), None)
        }
        _ => (plain(), None),
    }
}

/// One run of a trace document: `{"label": .., "steps": [..]}`.
fn run_json(label: &str, rec: &Recording, ids: &[String], dropped: &dyn Fn(&str) -> bool) -> String {
    let mut out = String::from("{\"label\":");
    string(&mut out, label);
    out.push_str(",\"steps\":[");
    for (index, step) in rec.steps.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{index},\"line\":{},\"label\":", step.line);
        string(&mut out, &step.label);
        out.push_str(",\"stage\":");
        string(&mut out, &step.stage);
        out.push_str(",\"values\":{");
        for (i, (var, json)) in step.values.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            string(&mut out, &ids[*var]);
            out.push(':');
            out.push_str(json);
        }
        out.push_str("},\"locals\":");
        out.push_str(&step.locals);
        if let Some(checkpoint) = step.checkpoint.as_deref().filter(|name| !dropped(name)) {
            out.push_str(",\"checkpoint\":");
            string(&mut out, checkpoint);
        }
        if !step.frames.is_empty() {
            out.push_str(",\"frames\":[");
            for (i, (name, line)) in step.frames.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str("{\"fn\":");
                string(&mut out, name);
                out += &format!(",\"line\":{line}}}");
            }
            out.push(']');
        }
        if let Some((var, path)) = &step.focus {
            out.push_str(",\"focus\":{\"variable\":");
            string(&mut out, &ids[*var]);
            let path: Vec<String> = path.iter().map(usize::to_string).collect();
            out += &format!(",\"path\":[{}]}}", path.join(","));
        }
        out.push('}');
    }
    out.push_str("]}");
    out
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
            let same = vars
                .iter()
                .find(|known| known.id == var.id && known.kind == var.kind && known.extra == var.extra);
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
        let position: HashMap<&str, usize> = slow
            .steps
            .iter()
            .enumerate()
            .filter_map(|(i, step)| Some((step.checkpoint.as_deref()?, i)))
            .collect();
        let mut reached = None;
        for name in fast.steps.iter().filter_map(|step| step.checkpoint.as_deref()) {
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
    out += &run_json(fast_label, fast, &fast_ids, &is_dropped);
    if let Some((slow_label, slow)) = slow {
        out.push_str(",\"slow\":");
        out += &run_json(slow_label, slow, &slow_ids, &is_dropped);
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

/// A checkpoint both runs reached, with what each showed there.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
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
}

/// The first value the solution gives one of the variables both runs
/// name that the brute does not give it, from [`Recording::assigned`]. A
/// variable that takes the same values in another order and ends alike
/// (a loop walked the other way) is not a divergence.
pub fn first_divergence(fast: &Recording, slow: &Recording) -> Option<Divergence> {
    fn histories(rec: &Recording) -> HashMap<&str, Vec<(usize, &Assigned)>> {
        let mut out: HashMap<&str, Vec<(usize, &Assigned)>> = HashMap::new();
        for (at, assigned) in rec.assigned.iter().enumerate() {
            let history = out.entry(assigned.name.as_str()).or_default();
            if history.last().is_none_or(|(_, last)| last.value != assigned.value) {
                history.push((at, assigned));
            }
        }
        out
    }
    let (ours, theirs) = (histories(fast), histories(slow));
    let mut best: Option<(usize, &str, usize)> = None;
    for (name, a) in &ours {
        let Some(b) = theirs.get(name) else {
            continue;
        };
        let values = |history: &[(usize, &Assigned)]| -> Vec<String> {
            history.iter().map(|(_, assigned)| assigned.value.clone()).collect()
        };
        let (va, vb) = (values(a), values(b));
        if va == vb {
            continue;
        }
        let (mut sa, mut sb) = (va.clone(), vb.clone());
        sa.sort();
        sb.sort();
        if sa == sb && va.last() == vb.last() {
            continue;
        }
        let nth = va.iter().zip(&vb).take_while(|(x, y)| x == y).count();
        let at = a.get(nth).map_or(fast.assigned.len(), |(at, _)| *at);
        if best.is_none_or(|(best_at, best_name, _)| (at, *name) < (best_at, best_name)) {
            best = Some((at, name, nth));
        }
    }
    let (_, name, nth) = best?;
    let side = |history: &Vec<(usize, &Assigned)>| {
        history.get(nth).map(|(_, assigned)| (assigned.line, assigned.value.clone()))
    };
    Some(Divergence {
        name: name.to_string(),
        nth: nth + 1,
        fast: side(&ours[name]),
        brute: side(&theirs[name]),
    })
}

/// The `debug(...)` checkpoints both runs reached, in the solution's
/// order, and the last line each printed: where the two first differ is
/// where to look.
pub fn stages(fast: &Recording, slow: &Recording) -> Vec<Stage> {
    let theirs: HashMap<&str, &Shown> = slow.shown.iter().map(|shown| (shown.name.as_str(), shown)).collect();
    fast.shown
        .iter()
        .filter_map(|shown| {
            let other = theirs.get(shown.name.as_str())?;
            Some(Stage {
                name: shown.name.clone(),
                line: shown.line,
                fast: shown.text.clone(),
                brute: other.text.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(assigned: &[(u32, &str, &str)]) -> Recording {
        Recording {
            assigned: assigned
                .iter()
                .map(|&(line, name, value)| Assigned { line, name: name.to_string(), value: value.to_string() })
                .collect(),
            ..Recording::default()
        }
    }

    #[test]
    fn the_first_divergence_is_the_earliest_value_the_solution_gives_differently() {
        let fast = run(&[(1, "i", "0"), (2, "s", "0"), (3, "s", "0"), (3, "t", "5"), (4, "s", "7")]);
        let slow = run(&[(1, "i", "0"), (9, "s", "0"), (9, "t", "6"), (10, "s", "8")]);
        // `s` repeats 0 and stays in step; `t` is the first value that differs.
        assert_eq!(
            first_divergence(&fast, &slow),
            Some(Divergence {
                name: "t".to_string(),
                nth: 1,
                fast: Some((3, "5".to_string())),
                brute: Some((9, "6".to_string())),
            })
        );
    }

    #[test]
    fn a_value_one_side_never_gives_is_a_divergence_and_loop_order_is_not() {
        // The same values in another order, ending the same, are a loop in another order.
        let fast = run(&[(1, "x", "1"), (1, "x", "2"), (1, "x", "3"), (2, "s", "1")]);
        let slow = run(&[(5, "x", "2"), (5, "x", "1"), (5, "x", "3"), (6, "s", "1"), (6, "s", "3")]);
        let d = first_divergence(&fast, &slow).unwrap();
        assert_eq!((d.name.as_str(), d.nth, d.fast), ("s", 2, None));
        assert_eq!(d.brute, Some((6, "3".to_string())));
        // Names only one side has say nothing.
        assert_eq!(first_divergence(&run(&[(1, "a", "1")]), &run(&[(1, "b", "2")])), None);
    }

    #[test]
    fn members_splits_a_written_object_at_its_top_level() {
        let object = r#"{"a":1,"b":[1,{"c":2}],"s":"x,\"}y","e":{}}"#;
        assert_eq!(members(object), [("a", "1"), ("b", "[1,{\"c\":2}]"), ("s", r#""x,\"}y""#), ("e", "{}")]);
        assert!(members("{}").is_empty());
    }
}
