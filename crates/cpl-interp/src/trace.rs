//! The trace of a run: what `cpl run --trace` and `cpl stress --trace`
//! show in the trace viewer (`tools/trace/trace-schema.json`).
//!
//! A step is a full snapshot of the file's top-level variables, taken
//! when one of them changes, at each `print` and at each `debug(...)`.
//! A `print` and a `debug(...)` are also checkpoints: two runs of the
//! same input, the solution and its brute, are lined up on them, since
//! their own variables have nothing else in common.

use crate::value::Value;
use cpl_diag::{LineIndex, Span};
use cpl_ir::{Dim, Program, Ty};
use std::collections::HashMap;
use std::sync::Arc;

/// How a run is recorded.
#[derive(Clone, Debug, Default)]
pub struct TraceConfig {
    /// The file's text: steps name the line they ran.
    pub source: String,
    /// Record only while this fn is running.
    pub only_fn: Option<String>,
}

/// The most steps one run keeps; the viewer reads up to 100,000.
const MAX_STEPS: usize = 20_000;
/// The most steps kept of what is above `@!multitest`. The test cases
/// start from the values it ends with, whether or not it fits.
const SETUP_STEPS: usize = 2_000;
/// The most snapshot text one run keeps.
const MAX_BYTES: usize = 48 << 20;
/// A variable with more cells than this is shown by its size.
const MAX_CELLS: usize = 20_000;

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

/// What a run left behind.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    pub vars: Vec<Var>,
    pub steps: Vec<Step>,
    /// Steps past the limit were not kept.
    pub truncated: bool,
    pub lives: Vec<Live>,
    pub shown: Vec<Shown>,
}

pub(crate) struct Tracer {
    lines: LineIndex,
    source: String,
    only_fn: Option<String>,
    /// How many calls of `only_fn` are running.
    pub inside: usize,
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
    /// The tree form of each `[V]?V` snapshot, beside its array form.
    tree_form: HashMap<(usize, usize), Arc<str>>,
    stdout: usize,
    printed: Vec<String>,
    debug_vars: HashMap<String, usize>,
    counts: HashMap<String, usize>,
    prints: usize,
    pub case: Option<u64>,
    bytes: usize,
    pub rec: Recording,
}

/// What the interpreter shows the tracer of its state.
pub(crate) struct View<'a> {
    pub program: &'a Program,
    pub globals: &'a [Value],
    pub domains: &'a [(i64, i64)],
    /// The named locals of the innermost call, with their values.
    pub locals: Vec<(&'a str, &'a Ty, &'a Value)>,
}

impl Tracer {
    pub fn new(config: TraceConfig, program: &Program) -> Tracer {
        let mut tracer = Tracer {
            lines: LineIndex::new(&config.source),
            source: config.source,
            only_fn: config.only_fn,
            inside: 0,
            kinds: Vec::new(),
            of_global: vec![None; program.globals.len()],
            visible: vec![false; program.globals.len()],
            setup: vec![false; program.globals.len()],
            last: Vec::new(),
            tree: Vec::new(),
            tree_form: HashMap::new(),
            stdout: 0,
            printed: Vec::new(),
            debug_vars: HashMap::new(),
            counts: HashMap::new(),
            prints: 0,
            case: None,
            bytes: 0,
            rec: Recording::default(),
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

    pub fn wants_fn(&self, name: &str) -> bool {
        self.only_fn.as_deref() == Some(name)
    }

    fn recording(&self) -> bool {
        self.only_fn.is_none() || self.inside > 0
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
        if !self.recording() {
            return;
        }
        // What is above `@!multitest` is the same on every input: it gets
        // a share of the trace, and the test cases get the rest.
        if self.case == Some(0) && (self.rec.steps.len() >= SETUP_STEPS || self.bytes >= MAX_BYTES / 16) {
            return;
        }
        if self.rec.steps.len() >= MAX_STEPS || self.bytes >= MAX_BYTES {
            self.rec.truncated = true;
            return;
        }
        let line = self.line(span).max(1);
        let index = self.rec.steps.len();
        let mut values = Vec::new();
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
                            self.tree_form.insert((index, var), tree.into());
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
            if matches!(value, Value::Unit | Value::Moved | Value::Closure(..) | Value::Alias(..)) {
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
        self.rec.steps.push(Step {
            line,
            label: label.unwrap_or_else(|| self.line_text(line)),
            stage: self.stage(),
            values,
            locals,
            checkpoint,
            focus,
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
        let line = self.line(span).max(1);
        if fresh
            && self.rec.steps.last().is_some_and(|last| {
                last.line == line && last.checkpoint.is_none() && last.stage == self.stage()
            })
        {
            let index = self.rec.steps.len() - 1;
            self.rec.steps.pop();
            self.tree_form.retain(|(step, _), _| *step != index);
        }
        self.step(view, span, None, None, None);
    }

    /// `print(...)` wrote `text`.
    pub fn printed(&mut self, view: &View, span: Span, text: &str) {
        if !self.recording() {
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
        if !self.recording() {
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
        let before = self.rec.steps.len();
        self.step(view, span, Some(format!("debug: {text}")), Some(&checkpoint), None);
        let name = match self.rec.steps.get(before).and_then(|step| step.checkpoint.clone()) {
            Some(checkpoint) => checkpoint,
            None => checkpoint,
        };
        self.rec.shown.push(Shown { line, name, text });
    }

    /// The run is over: the last step, and each variable's last value.
    pub fn finish(mut self, view: &View, failed: Option<(&str, Span)>) -> Recording {
        self.inside = usize::from(self.only_fn.is_some());
        match failed {
            Some((message, span)) => {
                self.step(view, span, Some(format!("failed: {message}")), None, None)
            }
            None => {
                let end = Span::new(self.source.trim_end().len() as u32, self.source.trim_end().len() as u32);
                self.step(view, end, Some("the program ended".to_string()), Some("end"), None)
            }
        }
        if self.rec.steps.is_empty() {
            // The viewer needs one step; a run with `only_fn` that never
            // called it has this one.
            self.only_fn = None;
            self.step(view, Span::default(), Some("nothing was recorded".to_string()), None, None);
        }
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
                for (index, step) in self.rec.steps.iter_mut().enumerate() {
                    for (at, json) in &mut step.values {
                        if *at == var {
                            if let Some(tree) = self.tree_form.get(&(index, var)) {
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
        Dim::Pos(d) => view.domains[d].0,
        Dim::Bound(_) | Dim::Int => 0,
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
        Dim::Int | Dim::IntFrom(_) => None,
    }
}

fn dim_name<'a>(dim: Dim, view: &View<'a>) -> Option<&'a str> {
    match dim {
        Dim::Pos(d) | Dim::Bound(d) => Some(view.program.domains[d].as_str()),
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
                    string(&mut extra, name);
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
            let field = |name: &str| def.fields.iter().any(|(field, _)| field == name);
            if def.name.starts_with("Graph") && field("start") && field("arcs") {
                (Kind::Csr, ",\"directed\":true".to_string())
            } else if def.name.starts_with("Rooted") && field("root") && field("par") {
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
        Value::Array(array) => array.data.iter().map(cells).sum(),
        Value::List(items) | Value::Tuple(items) | Value::Enum(_, items) => {
            1 + items.iter().map(cells).sum::<usize>()
        }
        Value::Map(entries) => entries.iter().map(|(k, v)| cells(k) + cells(v)).sum(),
        Value::Set(items) => items.iter().map(cells).sum(),
        Value::Opt(Some(inner)) => cells(inner),
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

/// The label of a position, or the number itself.
fn label(value: i64, ty: &Ty, view: &View) -> i64 {
    match ty {
        Ty::Pos(d) => value + view.domains[*d].0,
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
        (Value::Opt(None), _) => out.push_str(if json { "null" } else { "none" }),
        (Value::Opt(Some(inner)), Ty::Opt(inner_ty)) => render(out, inner, inner_ty, view, mode),
        (Value::Range(lo, hi), _) => {
            let text = format!("{lo}..{hi}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::Tuple(fields), _) => {
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
            rows(out, &array.data, &array.dims, elem, view, mode);
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
    Some(
        array
            .data
            .iter()
            .map(|cell| match cell {
                Value::Opt(Some(parent)) => Some(parent.int()),
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
            for (from, list) in array.data.iter().enumerate() {
                let Value::List(items) = list else { continue };
                for item in items.iter() {
                    let (to, weight) = match item {
                        Value::Tuple(fields) => (
                            fields.first().map_or(0, Value::int),
                            match (&weighted, fields.get(1)) {
                                (Some(ty @ (Ty::Int | Ty::Float)), Some(w)) => Some(json_of(w, ty, view)),
                                _ => None,
                            },
                        ),
                        other => (other.int(), None),
                    };
                    edges.push((from as i64 + lo, to + lo, weight));
                }
            }
            (graph((0..array.data.len() as i64).map(|i| i + lo), &edges), None)
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
                        edges.push((from as i64 + lo, arc.first().map_or(0, Value::int) + lo, None));
                    }
                }
            }
            (graph((0..n as i64).map(|i| i + lo), &edges), None)
        }
        (Kind::Rooted, Value::Tuple(fields), Ty::Struct(id)) => {
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
        (Kind::Grid | Kind::Dp, Value::Array(array), _) if array.dims.len() != 2 || array.data.is_empty() => {
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
