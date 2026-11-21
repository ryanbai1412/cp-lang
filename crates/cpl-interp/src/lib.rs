//! Debug interpreter: runs a lowered program directly.
//!
//! This is `cpl run`. It walks the same [`cpl_ir::Program`] that the C++
//! back end prints, and it checks what the submission build doesn't:
//! indices, input ranges, integer overflow, `!` on `none`, and a variable
//! used while a call is still changing it. A failed check is a [`Trap`]
//! with the source span, never a crash of the interpreter.
//!
//! Strings are byte vectors. Nothing here decodes or validates text.

mod input;
pub mod profile;
pub mod trace;
mod value;

use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtArg, FmtPiece,
    FnId, Iter, LoopId, Mutation, Place, PlaceKind, Program, SortBy, Spec, Stmt, Ty, UnOp,
};
use input::Input;
pub use input::More;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use value::{show, zero, Array, CaptureVal, OptVal, Part, Step, TupleVal, Value};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// One active call on the stack when a program stopped.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frame {
    pub name: String,
    /// The call expression that entered this frame.
    pub call_span: Span,
}

/// A program that failed while running.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Trap {
    pub message: String,
    /// The expression or statement that failed.
    pub span: Span,
    /// Active calls from innermost to outermost (`main` is not listed).
    pub call_stack: Vec<Frame>,
    /// 1-based test case number when running under `@!multitest`.
    pub case: Option<u64>,
}

impl Trap {
    /// Builds the runtime diagnostic with test-case and call-stack notes.
    pub fn to_diagnostic(&self, path: &str, lines: &cpl_diag::LineIndex) -> cpl_diag::Diagnostic {
        let mut diagnostic = cpl_diag::Diagnostic::error(
            cpl_diag::codes::RUNTIME,
            self.span,
            self.message.clone(),
        );
        if let Some(case) = self.case {
            diagnostic = diagnostic.with_note(format!("in test case {case}"));
        }
        if !self.call_stack.is_empty() {
            let mut parts = Vec::with_capacity(self.call_stack.len() + 1);
            let mut current_line = lines.line_of(self.span.start) + 1;
            for frame in &self.call_stack {
                if frame.call_span == Span::default() && frame.name.starts_with("...") {
                    parts.push(frame.name.clone());
                    continue;
                }
                parts.push(format!("{} ({path}:{current_line})", frame.name));
                current_line = lines.line_of(frame.call_span.start) + 1;
            }
            parts.push(format!("top level ({path}:{current_line})"));
            diagnostic = diagnostic.with_note(format!("call stack: {}", parts.join(" <- ")));
        }
        diagnostic
    }
}

/// Stack for the interpreter thread: deep recursion in the program is deep
/// recursion here, about 4 kB a call, so this holds two million.
/// Reserved, not committed; a host that cannot reserve it gets less.
const STACK_BYTES: usize = 8 << 30;

/// Half the machine's memory, where it can be read: a recursion without end
/// fills the stack it may use, and that stack must not take the machine.
fn memory_for_stack() -> usize {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|info| {
            let line = info.lines().find(|line| line.starts_with("MemTotal:"))?;
            let kb: usize = line.split_whitespace().nth(1)?.parse().ok()?;
            Some(kb.saturating_mul(1024) / 2)
        })
        .unwrap_or(usize::MAX)
}

/// How much stack a program's calls may take before the run is stopped.
/// There is no limit on how deep calls nest: only on the stack they fill.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Bytes of the interpreter's own stack those calls may use: the
    /// program's recursion is the interpreter's.
    pub stack_bytes: usize,
}

/// Runs `program` on `input`, writing what it prints to `output`. Output
/// printed before a failure is still written.
///
/// The program runs on a thread of its own with a stack deep enough for
/// any recursion a contest solution needs. Where no thread can be started
/// (WebAssembly), use [`run_on_this_thread`].
pub fn run(program: &Program, input: &[u8], output: &mut (dyn Write + Send)) -> Result<(), Trap> {
    run_with(program, Io::fixed(input), output)
}

/// What a run leaves besides its output.
#[derive(Debug, Default)]
pub struct Observed {
    /// `None` when the program ran to its end.
    pub trap: Option<Trap>,
    /// What its `debug(...)`s showed, a line each: `a.cp:6: i = 0, best = 1`.
    pub debug: Vec<u8>,
    /// The trace, when one was asked for.
    pub recording: Option<trace::Recording>,
    /// The profile, when one was asked for.
    pub profile: Option<profile::Profile>,
}

/// What a program reads.
pub struct Io<'a> {
    /// What `read()` reads.
    input: Feed<'a>,
    /// What an `@interactor` hears from the solution: `sol.recv()`.
    peer: Option<More<'a>>,
}

enum Feed<'a> {
    Fixed(&'a [u8]),
    Live(More<'a>),
}

impl<'a> Io<'a> {
    /// The whole input, known before the program starts.
    pub fn fixed(input: &'a [u8]) -> Io<'a> {
        Io {
            input: Feed::Fixed(input),
            peer: None,
        }
    }

    /// An input that arrives while the program runs: the solution's side
    /// of an interactive problem.
    pub fn live(more: More<'a>) -> Io<'a> {
        Io {
            input: Feed::Live(more),
            peer: None,
        }
    }

    /// The judge's side: `input` is the hidden test, and `solution` is
    /// what the solution prints.
    pub fn judge(input: &'a [u8], solution: More<'a>) -> Io<'a> {
        Io {
            input: Feed::Fixed(input),
            peer: Some(solution),
        }
    }
}

/// [`run`] with an input that can arrive while the program runs.
pub fn run_with(program: &Program, io: Io, output: &mut (dyn Write + Send)) -> Result<(), Trap> {
    match run_observed(program, io, output, None).trap {
        Some(trap) => Err(trap),
        None => Ok(()),
    }
}

/// [`run_with`], keeping what the program's `debug(...)`s showed and,
/// with `trace`, the trace of the run.
pub fn run_observed(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    trace: Option<trace::TraceConfig>,
) -> Observed {
    observe(program, io, output, trace, false)
}

/// [`run_with`], keeping how often each statement ran and the time spent
/// there, as [`profile::Profile`] says.
pub fn run_profiled(program: &Program, io: Io, output: &mut (dyn Write + Send)) -> Observed {
    observe(program, io, output, None, true)
}

fn observe(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    trace: Option<trace::TraceConfig>,
    profile: bool,
) -> Observed {
    let io = std::sync::Mutex::new(Some(io));
    let trace = std::sync::Mutex::new(trace);
    // The writer is shared by reference so that a spawn that fails (too
    // little address space for the stack) can be tried again smaller.
    let output = std::sync::Mutex::new(output);
    let failed = |message: &str| Observed {
        trap: Some(Trap {
            message: message.into(),
            span: Span::default(),
            call_stack: Vec::new(),
            case: None,
        }),
        ..Observed::default()
    };
    let memory = memory_for_stack();
    std::thread::scope(|scope| {
        for bytes in [STACK_BYTES, 3 << 30, 1 << 28, 1 << 23] {
            let (output, io, trace) = (&output, &io, &trace);
            let limits = Limits {
                // The rest is left for the frames under the deepest call.
                stack_bytes: (bytes - (bytes / 64).max(1 << 20)).min(memory),
            };
            let spawned =
                std::thread::Builder::new()
                    .stack_size(bytes)
                    .spawn_scoped(scope, move || {
                        let mut output = output
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let io = io
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .take()
                            .expect("only the thread that starts takes the input");
                        let trace = trace
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .take();
                        run_io(program, io, &mut **output, limits, trace, profile)
                    });
            if let Ok(handle) = spawned {
                return handle
                    .join()
                    .unwrap_or_else(|_| failed("the interpreter itself failed; this is a bug in cpl"));
            }
        }
        failed("could not start the interpreter thread")
    })
}

/// [`run`] on the calling thread, for a host that cannot start one. The
/// caller says how much of its stack the program's calls may take, since
/// only it knows how much there is.
pub fn run_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
) -> Result<(), Trap> {
    match run_io(program, Io::fixed(input), output, limits, None, false).trap {
        Some(trap) => Err(trap),
        None => Ok(()),
    }
}

/// [`run_on_this_thread`], keeping what [`run_observed`] keeps.
pub fn observe_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
) -> Observed {
    run_io(program, Io::fixed(input), output, limits, trace, false)
}

fn run_io(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
    profile: bool,
) -> Observed {
    let top = 0u8;
    let mut interp = Interp {
        program,
        globals: vec![Value::Unit; program.globals.len()],
        domains: vec![(0, 0); program.domains.len()],
        saved_domains: Vec::new(),
        stack: Vec::new(),
        base: 0,
        input: match io.input {
            Feed::Fixed(input) => Input::new(input),
            Feed::Live(more) => Input::live(more),
        },
        peer: io.peer.map(|more| Input::live(more).named("solution's output")),
        queries: 0,
        out: Vec::with_capacity(1 << 16),
        sink: output,
        stack_top: std::ptr::addr_of!(top) as usize,
        stack_budget: limits.stack_bytes,
        frame_locals: Vec::new(),
        call_frames: Vec::new(),
        current_case: None,
        rng_state: 0x9e3779b97f4a7c15u64,
        tracer: trace.map(|config| Box::new(trace::Tracer::new(config, program))),
        profiler: profile.then(|| Box::new(profile::Profiler::new(program))),
        debug_out: Vec::new(),
        feeds: cpl_ir::pushed::Feeds::of(program),
        fed: HashMap::new(),
        fed_loops: Vec::new(),
        in_closure: false,
        fed_left: None,
    };
    let result = interp.main();
    let _ = interp.sink.write_all(&interp.out);
    let _ = interp.sink.flush();
    let trap = result.err().map(|mut trap| {
        if trap.call_stack.is_empty() && !interp.call_frames.is_empty() {
            let frames = &interp.call_frames;
            if frames.len() <= 32 {
                trap.call_stack = frames
                    .iter()
                    .rev()
                    .map(|&(name, call_span)| Frame {
                        name: name.to_string(),
                        call_span,
                    })
                    .collect();
            } else {
                let mut stack = Vec::with_capacity(25);
                for &(name, call_span) in frames[frames.len() - 16..].iter().rev() {
                    stack.push(Frame {
                        name: name.to_string(),
                        call_span,
                    });
                }
                stack.push(Frame {
                    name: format!("... {} frames omitted ...", frames.len() - 24),
                    call_span: Span::default(),
                });
                for &(name, call_span) in frames[..8].iter().rev() {
                    stack.push(Frame {
                        name: name.to_string(),
                        call_span,
                    });
                }
                trap.call_stack = stack;
            }
        }
        if trap.case.is_none() {
            trap.case = interp.current_case;
        }
        *trap
    });
    let recording = interp.tracer.take().map(|tracer| {
        let failed = trap.as_ref().map(|trap| (trap.message.as_str(), trap.span));
        let view = interp.view();
        tracer.finish(&view, failed)
    });
    Observed {
        trap,
        debug: std::mem::take(&mut interp.debug_out),
        recording,
        profile: interp.profiler.take().map(|profiler| profiler.finish()),
    }
}

type Res<T> = Result<T, Box<Trap>>;

fn trap<T>(span: Span, message: impl Into<String>) -> Res<T> {
    Err(Box::new(Trap {
        message: message.into(),
        span,
        call_stack: Vec::new(),
        case: None,
    }))
}

enum Flow {
    Next,
    Break(Option<LoopId>),
    Continue(Option<LoopId>),
    Return(Value),
    Give(Value),
}

struct Interp<'p, 'o> {
    program: &'p Program,
    globals: Vec<Value>,
    /// First label and size of each domain, once it is declared.
    domains: Vec<(i64, i64)>,
    /// Saved `(domain_id, (lo, len))` entries for recursive/block-scoped restore.
    saved_domains: Vec<(cpl_ir::DomainId, (i64, i64))>,
    /// Locals of every active call; `base` is where the current call's start.
    stack: Vec<Value>,
    base: usize,
    input: Input<'p>,
    /// What the solution prints, for an `@interactor`.
    peer: Option<Input<'p>>,
    /// How many times the program has called `ask`.
    queries: i64,
    out: Vec<u8>,
    sink: &'o mut (dyn Write + Send),
    /// Where the interpreter's own stack began, and how much of it calls
    /// may use: the program's recursion is the interpreter's.
    stack_top: usize,
    stack_budget: usize,
    /// The locals of each active fn, innermost last; `main` when empty.
    frame_locals: Vec<&'p [cpl_ir::Local]>,
    /// Active call frames `(fn_name, call_site_span)` from outermost to innermost.
    call_frames: Vec<(&'p str, Span)>,
    /// Current 1-based test case number when running under `@!multitest`.
    current_case: Option<u64>,
    /// Deterministic splitmix64 PRNG state (matches `cpl-codegen`).
    rng_state: u64,
    /// Records the trace of the run, when one was asked for.
    tracer: Option<Box<trace::Tracer>>,
    profiler: Option<Box<profile::Profiler>>,
    /// What `debug(...)` showed.
    debug_out: Vec<u8>,
    /// Which loops over a `yields` fn run their body as the fn yields.
    feeds: cpl_ir::pushed::Feeds,
    /// What [`cpl_ir::pushed::Feeds::feeds`] said of each such loop.
    fed: HashMap<LoopId, Option<(FnId, usize)>>,
    /// The loops being fed by a `yields` fn that is running, innermost last.
    fed_loops: Vec<Fed<'p>>,
    /// True while the body of a closure runs.
    in_closure: bool,
    /// How the body of the fed loop that just ended left it, when that
    /// was not a plain `break`: a `return`, or a jump to an outer loop.
    fed_left: Option<Flow>,
}

/// A loop whose body runs each time the `yields` fn at `callee` pushes
/// to its list at `acc`.
struct Fed<'p> {
    callee: usize,
    acc: usize,
    caller: usize,
    id: LoopId,
    slot: usize,
    body: &'p Block,
    /// How the body left the loop, once it has: the fn stops there.
    left: Option<Flow>,
}

/// The global a place is in, when it is in one.
fn root_global(place: &Place) -> Option<usize> {
    match &place.kind {
        PlaceKind::Global(id) => Some(*id),
        PlaceKind::Local(_) => None,
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => root_global(base),
    }
}

/// How many indices of `place` lead straight from its variable: `2` in
/// `dp[i][j]` and in `dp[i, j]`, `1` in `a[i].x`.
fn leading_indices(place: &Place) -> usize {
    match &place.kind {
        PlaceKind::Global(_) | PlaceKind::Local(_) => 0,
        PlaceKind::Index(base, indices) => match &base.kind {
            PlaceKind::Global(_) | PlaceKind::Index(..) if leading_indices(base) == index_count(base) => {
                leading_indices(base) + indices.len()
            }
            _ => leading_indices(base),
        },
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => leading_indices(base),
    }
}

fn index_count(place: &Place) -> usize {
    match &place.kind {
        PlaceKind::Global(_) | PlaceKind::Local(_) => 0,
        PlaceKind::Index(base, indices) => index_count(base) + indices.len(),
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => index_count(base),
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl<'p, 'o> Interp<'p, 'o> {
    fn main(&mut self) -> Res<()> {
        let program = self.program;
        self.stack.clear();
        self.stack.resize(program.main_locals.len(), Value::Unit);
        self.base = 0;
        if !program.multitest {
            self.block(&program.main)?;
            return Ok(());
        }
        // Above `@!multitest`: once. A top-level `return` there ends the
        // program.
        if let Some(tracer) = &mut self.tracer {
            tracer.case = Some(0);
        }
        if let Flow::Return(_) = self.block(&program.setup)? {
            return Ok(());
        }
        if let Some(tracer) = &mut self.tracer {
            tracer.divider();
        }
        let mut left = if program.until_eof {
            None
        } else {
            match self.input.int() {
                // The judge's side of an interactive problem tells the
                // solution how many test cases there are.
                Ok((count, _)) if program.announce_tests => {
                    self.out.extend_from_slice(format!("{count}\n").as_bytes());
                    let _ = self.sink.write_all(&self.out);
                    let _ = self.sink.flush();
                    self.out.clear();
                    Some(count)
                }
                Ok((count, _)) => Some(count),
                Err(error) => {
                    return trap(
                        Span::default(),
                        format!("reading the number of tests: {}", error.message),
                    )
                }
            }
        };
        let mut case = 0;
        loop {
            match &mut left {
                Some(left) if *left <= 0 => break,
                Some(left) => *left -= 1,
                None if self.input.at_eof() => break,
                None => {}
            }
            case += 1;
            self.current_case = Some(case);
            // `ask(.., limit: K)` allows `K` queries in each test case.
            self.queries = 0;
            if let Some(tracer) = &mut self.tracer {
                tracer.start_case(case);
            }
            self.stack.clear();
            self.stack.resize(program.main_locals.len(), Value::Unit);
            self.block(&program.main)?;
        }
        Ok(())
    }

    /// What the tracer sees of the program's state right now.
    fn view(&self) -> trace::View<'_> {
        let locals = self
            .locals()
            .iter()
            .enumerate()
            .filter(|(_, local)| !local.name.is_empty() && !local.name.starts_with('_'))
            .filter_map(|(slot, local)| {
                let value = self.stack.get(self.base + slot)?;
                let value = match value {
                    Value::Alias(true, id) => self.globals.get(*id)?,
                    Value::Alias(false, at) => self.stack.get(*at)?,
                    Value::Part(part) => self.part(part)?,
                    value => value,
                };
                Some((local.name.as_str(), &local.ty, value))
            })
            .collect();
        trace::View {
            program: self.program,
            globals: &self.globals,
            domains: &self.domains,
            locals,
        }
    }

    /// Runs `record` on the tracer, when the run is traced.
    fn traced(&mut self, record: impl FnOnce(&mut trace::Tracer, &trace::View)) {
        if let Some(mut tracer) = self.tracer.take() {
            record(&mut tracer, &self.view());
            self.tracer = Some(tracer);
        }
    }

    /// A step for a change to `place`, when it is in a top-level variable.
    fn trace_change(&mut self, place: &Place, indices: &[Value]) {
        if self.tracer.is_none() {
            return;
        }
        let Some(global) = root_global(place) else {
            return;
        };
        if let PlaceKind::Global(id) = &place.kind {
            let id = *id;
            return self.traced(|tracer, view| tracer.declared(id, view, place.span));
        }
        let path: Vec<usize> = indices
            .iter()
            .take(leading_indices(place))
            .map(|index| index.int().max(0) as usize)
            .collect();
        self.traced(|tracer, view| tracer.step(view, place.span, None, None, Some((global, path))));
    }

    fn flush_if_large(&mut self) {
        if self.out.len() >= 1 << 16 {
            let _ = self.sink.write_all(&self.out);
            self.out.clear();
        }
    }

    // ----- statements -----

    fn block(&mut self, block: &'p Block) -> Res<Flow> {
        for stmt in &block.stmts {
            match self.stmt(stmt)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, stmt: &'p Stmt) -> Res<Flow> {
        if let Some(profiler) = &mut self.profiler {
            profiler.enter(stmt);
        }
        match stmt {
            Stmt::Let(slot, init) => {
                let value = match init {
                    Some(init) => self.eval(init)?,
                    None => zero(&self.locals()[*slot].ty, self.program),
                };
                let base = self.base;
                self.stack[base + slot] = value;
            }
            Stmt::Assign(place, op, value) => {
                if self.tracer.is_none() {
                    self.assign_untraced(place, *op, value)?;
                } else {
                    let indices = self.assign(place, *op, value)?;
                    self.trace_change(place, &indices);
                }
            }
            Stmt::Expr(e) => {
                self.eval(e)?;
            }
            Stmt::If(cond, then, otherwise) => {
                let branch = if self.eval(cond)?.bool() {
                    then
                } else {
                    otherwise
                };
                return self.block(branch);
            }
            Stmt::While(id, cond, body) => {
                while self.eval(cond)?.bool() {
                    match self.block(body)? {
                        Flow::Next | Flow::Continue(None) => {}
                        Flow::Break(None) => break,
                        Flow::Break(Some(target)) if target == *id => break,
                        Flow::Continue(Some(target)) if target == *id => {}
                        other => return Ok(other),
                    }
                }
            }
            Stmt::For(id, slot, iter, body) => return self.for_loop(*id, *slot, iter, body),
            Stmt::Return(value) => {
                let value = match value {
                    Some(value) => self.eval(value)?,
                    None => Value::Unit,
                };
                return Ok(Flow::Return(value));
            }
            Stmt::Break(target) => return Ok(Flow::Break(*target)),
            Stmt::Continue(target) => return Ok(Flow::Continue(*target)),
            Stmt::Give(value) => return Ok(Flow::Give(self.eval(value)?)),
            Stmt::Assert {
                cond,
                message,
                text,
                span,
                ..
            } => {
                if !self.eval(cond)?.bool() {
                    let mut report = format!("assertion failed: {text}");
                    if let Some(message) = message {
                        if let Value::Str(bytes) = self.eval(message)? {
                            report.push_str(": ");
                            report.extend(bytes.iter().map(|b| *b as char));
                        }
                    }
                    return trap(*span, report);
                }
            }
            Stmt::DefDomain { id, lo, len } => {
                let len_span = len.span;
                let lo = self.eval(lo)?.int();
                let len = self.eval(len)?.int().max(0);
                self.domains[*id] = (lo, len);
                self.size_subsets(*id, len_span)?;
            }
            Stmt::DefDomainFromVal { id, val } => {
                if let Value::DomainVal(lo, len) = self.eval(val)? {
                    self.domains[*id] = (lo, len.max(0));
                    self.size_subsets(*id, val.span)?;
                }
            }
            Stmt::Trap(message, span) => return trap(*span, message.clone()),
        }
        Ok(Flow::Next)
    }

    fn locals(&self) -> &'p [cpl_ir::Local] {
        self.frame_locals
            .last()
            .copied()
            .unwrap_or(&self.program.main_locals)
    }

    fn for_loop(&mut self, id: LoopId, slot: usize, iter: &'p Iter, body: &'p Block) -> Res<Flow> {
        let base = self.base;
        macro_rules! round {
            ($value:expr) => {{
                self.stack[base + slot] = $value;
                match self.block(body)? {
                    Flow::Next | Flow::Continue(None) => {}
                    Flow::Break(None) => break,
                    Flow::Break(Some(target)) if target == id => break,
                    Flow::Continue(Some(target)) if target == id => {}
                    other => return Ok(other),
                }
            }};
        }
        match iter {
            Iter::Range {
                lo,
                hi,
                step,
                rev,
                closed,
            } => {
                let lo_i64 = self.eval(lo)?.int();
                let hi_i64 = self.eval(hi)?.int();
                if step.is_none() {
                    if *closed {
                        if lo_i64 <= hi_i64 {
                            if *rev {
                                let mut v = hi_i64;
                                loop {
                                    round!(Value::Int(v));
                                    if v == lo_i64 {
                                        break;
                                    }
                                    v -= 1;
                                }
                            } else {
                                let mut v = lo_i64;
                                loop {
                                    round!(Value::Int(v));
                                    if v == hi_i64 {
                                        break;
                                    }
                                    v += 1;
                                }
                            }
                        }
                    } else if lo_i64 < hi_i64 {
                        if *rev {
                            let mut v = hi_i64 - 1;
                            loop {
                                round!(Value::Int(v));
                                if v == lo_i64 {
                                    break;
                                }
                                v -= 1;
                            }
                        } else {
                            let mut v = lo_i64;
                            while v < hi_i64 {
                                round!(Value::Int(v));
                                v += 1;
                            }
                        }
                    }
                    return Ok(Flow::Next);
                }
                let lo = lo_i64 as i128;
                // One past the last value, which an `Int` may not hold.
                let hi = hi_i64 as i128 + i128::from(*closed);
                let step = match step {
                    Some(step) => {
                        let value = self.eval(step)?.int();
                        if value <= 0 {
                            return trap(
                                step.span,
                                format!("a range steps forward: `by {value}` is not positive"),
                            );
                        }
                        value
                    }
                    None => 1,
                };
                if lo >= hi {
                    return Ok(Flow::Next);
                }
                // The values are lo, lo + step, ..., below hi.
                let step = step as i128;
                let count = (hi - lo + step - 1) / step;
                if *rev {
                    let mut k = count - 1;
                    loop {
                        if k < 0 {
                            break;
                        }
                        round!(Value::Int((lo + k * step) as i64));
                        k -= 1;
                    }
                } else {
                    let mut k = 0;
                    loop {
                        if k >= count {
                            break;
                        }
                        round!(Value::Int((lo + k * step) as i64));
                        k += 1;
                    }
                }
            }
            Iter::RangeVal(expr, rev) => {
                if let Value::Range(lo, hi) = self.eval(expr)? {
                    if lo < hi {
                        if *rev {
                            let mut v = hi - 1;
                            while v >= lo {
                                round!(Value::Int(v));
                                if v == lo {
                                    break;
                                }
                                v -= 1;
                            }
                        } else {
                            let mut v = lo;
                            while v < hi {
                                round!(Value::Int(v));
                                v += 1;
                            }
                        }
                    }
                }
            }
            Iter::Subsets(mask_expr) => {
                let mask = self.eval(mask_expr)?.int();
                let mut g = mask;
                loop {
                    round!(Value::Int(g));
                    if g == 0 {
                        break;
                    }
                    g = (g - 1) & mask;
                }
            }
            Iter::Members(mask_expr) => {
                let mut left = self.eval(mask_expr)?.int();
                while left != 0 {
                    round!(Value::Int(i64::from(left.trailing_zeros())));
                    left &= left - 1;
                }
            }
            Iter::Seq(seq, false) if self.is_fed(id, seq, body).is_some() => {
                let (ExprKind::Call(callee, args), Some((_, acc))) = (&seq.kind, self.is_fed(id, seq, body)) else {
                    unreachable!("a fed loop is over a call");
                };
                let fed = Fed {
                    callee: 0,
                    acc,
                    caller: base,
                    id,
                    slot,
                    body,
                    left: None,
                };
                self.call_feeding(*callee, args, seq.span, Some(fed))?;
                if let Some(flow) = self.fed_left.take() {
                    return Ok(flow);
                }
            }
            Iter::Seq(seq, rev) => {
                // The loop walks the value as it was when the loop began.
                let held = self.eval(seq)?;
                match &held {
                    Value::List(items) => {
                        let n = items.len();
                        for k in 0..n {
                            let k = if *rev { n - 1 - k } else { k };
                            round!(items[k].clone());
                        }
                    }
                    Value::Array(array) => {
                        let n = array.data.len();
                        for k in 0..n {
                            let k = if *rev { n - 1 - k } else { k };
                            round!(array.data[k].clone());
                        }
                    }
                    Value::Str(bytes) => {
                        let n = bytes.len();
                        for k in 0..n {
                            let k = if *rev { n - 1 - k } else { k };
                            round!(Value::Char(bytes[k]));
                        }
                    }
                    Value::Set(items) => {
                        if *rev {
                            for item in items.iter().rev() {
                                round!(item.clone());
                            }
                        } else {
                            for item in items.iter() {
                                round!(item.clone());
                            }
                        }
                    }
                    Value::Map(entries) => {
                        if *rev {
                            for (key, value) in entries.iter().rev() {
                                round!(Value::tuple(vec![key.clone(), value.clone()]));
                            }
                        } else {
                            for (key, value) in entries.iter() {
                                round!(Value::tuple(vec![key.clone(), value.clone()]));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(Flow::Next)
    }

    /// Sizes each `Subset<D>` of the domain `base`, which was just set.
    fn size_subsets(&mut self, base: cpl_ir::DomainId, span: Span) -> Res<()> {
        for &(subset, of) in &self.program.subsets {
            if of == base {
                let len = self.domains[base].1;
                if len > cpl_ir::MAX_SUBSET_BITS {
                    return trap(
                        span,
                        format!(
                            "a `Subset` needs a domain of at most {} values, and this one has {len}",
                            cpl_ir::MAX_SUBSET_BITS
                        ),
                    );
                }
                self.domains[subset] = (0, 1 << len);
            }
        }
        Ok(())
    }

    // ----- places -----

    /// Evaluates every index along `place`, outermost container first.
    fn indices(&mut self, place: &'p Place, out: &mut Vec<Value>) -> Res<()> {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.indices(base, out)?,
            PlaceKind::Index(base, indices) => {
                self.indices(base, out)?;
                for index in indices {
                    out.push(self.eval(index)?);
                }
            }
        }
        Ok(())
    }

    /// The storage `place` names, with its indices already evaluated.
    fn resolve<'s>(
        &'s mut self,
        place: &Place,
        indices: &mut std::slice::Iter<'_, Value>,
    ) -> Res<&'s mut Value> {
        let span = place.span;
        match &place.kind {
            PlaceKind::Local(slot) => {
                let at = self.base + slot;
                let part = match &self.stack[at] {
                    Value::Alias(global, at) => Part {
                        global: *global,
                        at: *at,
                        steps: Vec::new(),
                    },
                    Value::Part(part) => return self.part_mut(&Rc::clone(part), span),
                    _ => Part {
                        global: false,
                        at,
                        steps: Vec::new(),
                    },
                };
                self.part_mut(&part, span)
            }
            PlaceKind::Global(id) => Ok(&mut self.globals[*id]),
            PlaceKind::Field(base, index) => field_mut(self.resolve(base, indices)?, *index, span),
            PlaceKind::Some(base) => inside_mut(self.resolve(base, indices)?, span),
            PlaceKind::Index(base, exprs) => {
                let holder = self.resolve(base, indices)?;
                cell_mut(holder, indices, exprs.len(), span)
            }
        }
    }

    /// The part of a variable `place` names, with its indices evaluated.
    fn part_of(&self, place: &Place, indices: &mut std::vec::IntoIter<Value>) -> Part {
        let (base, step) = match &place.kind {
            PlaceKind::Global(id) => {
                return Part {
                    global: true,
                    at: *id,
                    steps: Vec::new(),
                }
            }
            PlaceKind::Local(slot) => {
                let at = self.base + slot;
                return match &self.stack[at] {
                    Value::Alias(global, at) => Part {
                        global: *global,
                        at: *at,
                        steps: Vec::new(),
                    },
                    Value::Part(part) => Part::clone(part),
                    _ => Part {
                        global: false,
                        at,
                        steps: Vec::new(),
                    },
                };
            }
            PlaceKind::Field(base, index) => (base, Step::Field(*index)),
            PlaceKind::Some(base) => (base, Step::Some),
            PlaceKind::Index(base, exprs) => {
                let mut part = self.part_of(base, indices);
                part.steps
                    .push(Step::Index(indices.by_ref().take(exprs.len()).collect()));
                return part;
            }
        };
        let mut part = self.part_of(base, indices);
        part.steps.push(step);
        part
    }

    /// The storage of `part`.
    fn part_mut(&mut self, part: &Part, span: Span) -> Res<&mut Value> {
        let mut value = if part.global {
            &mut self.globals[part.at]
        } else {
            &mut self.stack[part.at]
        };
        for step in &part.steps {
            value = match step {
                Step::Field(index) => field_mut(value, *index, span)?,
                Step::Some => inside_mut(value, span)?,
                Step::Index(indices) => cell_mut(value, &mut indices.iter(), indices.len(), span)?,
            };
        }
        Ok(value)
    }

    /// The value of `part`, if it is still there.
    fn part(&self, part: &Part) -> Option<&Value> {
        let mut value = if part.global {
            self.globals.get(part.at)?
        } else {
            self.stack.get(part.at)?
        };
        for step in &part.steps {
            value = match (step, value) {
                (Step::Field(index), Value::Tuple(TupleVal::Boxed(fields))) => fields.get(*index)?,
                (Step::Some, Value::Opt(OptVal::Boxed(inner))) => inner,
                (Step::Index(indices), Value::Array(array)) => {
                    let at = array_offset(&array.dims, indices.iter(), Span::new(0, 0)).ok()?;
                    array.data.get(at)?
                }
                (Step::Index(indices), Value::List(items)) => {
                    items.get(usize::try_from(indices.first()?.int()).ok()?)?
                }
                (Step::Index(indices), Value::Map(entries)) => entries.get(indices.first()?)?,
                _ => return None,
            };
        }
        Some(value)
    }
}

fn field_mut(holder: &mut Value, index: usize, span: Span) -> Res<&mut Value> {
    match holder {
        Value::Tuple(fields) => Ok(&mut fields.as_mut_vec()[index]),
        Value::Moved => trap(span, LENT),
        _ => trap(span, "internal: field of a value without fields"),
    }
}

fn inside_mut(holder: &mut Value, span: Span) -> Res<&mut Value> {
    match holder {
        Value::Opt(opt) => {
            if let OptVal::Int(v) = *opt {
                *opt = OptVal::Boxed(Rc::new(Value::Int(v)));
            }
            match opt {
                OptVal::Boxed(inner) => Ok(Rc::make_mut(inner)),
                OptVal::None => trap(span, "there is no value here: it is `none`"),
                OptVal::Int(_) => unreachable!(),
            }
        }
        Value::Moved => trap(span, LENT),
        _ => trap(span, "there is no value here: it is `none`"),
    }
}

/// The cell of `holder` at the next `count` of `indices`.
fn cell_mut<'s>(
    holder: &'s mut Value,
    indices: &mut std::slice::Iter<'_, Value>,
    count: usize,
    span: Span,
) -> Res<&'s mut Value> {
    match holder {
        Value::Array(array) => {
            let at = array_offset(&array.dims, indices.by_ref().take(count), span)?;
            Ok(&mut Rc::make_mut(array).data[at])
        }
        Value::List(items) => {
            let index = indices.next().map_or(0, Value::int);
            let len = items.len();
            match usize::try_from(index).ok().filter(|i| *i < len) {
                Some(i) => Ok(&mut Rc::make_mut(items)[i]),
                None => trap(
                    span,
                    format!("index {index} is outside a list of length {len}"),
                ),
            }
        }
        Value::Map(entries) => {
            let key = indices.next().cloned().unwrap_or_default();
            match Rc::make_mut(entries).get_mut(&key) {
                Some(value) => Ok(value),
                None => trap(
                    span,
                    "this key is not in the map; store with `m[k] = v` first",
                ),
            }
        }
        Value::Moved => trap(span, LENT),
        _ => trap(span, "internal: index of a value that has no elements"),
    }
}

impl<'p, 'o> Interp<'p, 'o> {
    /// Fast-path assignment when tracing is disabled: avoids allocating an
    /// index `Vec` for direct locals/globals and 1D/2D array or list writes.
    fn assign_untraced(&mut self, place: &'p Place, op: AssignOp, value: &'p Expr) -> Res<()> {
        match &place.kind {
            PlaceKind::Local(slot) => {
                let value = self.eval(value)?;
                let at = self.base + slot;
                if !matches!(self.stack[at], Value::Alias(..) | Value::Part(..)) {
                    let cell = &mut self.stack[at];
                    if op == AssignOp::Set {
                        *cell = value;
                    } else {
                        let old = std::mem::take(cell);
                        if matches!(old, Value::Moved) {
                            *cell = old;
                            return trap(place.span, LENT);
                        }
                        *cell = combine(op, old, value, place.span)?;
                    }
                    return Ok(());
                }
                let cell = self.resolve(place, &mut [].iter())?;
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(place.span, LENT);
                    }
                    *cell = combine(op, old, value, place.span)?;
                }
                return Ok(());
            }
            PlaceKind::Global(id) => {
                let value = self.eval(value)?;
                let cell = &mut self.globals[*id];
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(place.span, LENT);
                    }
                    *cell = combine(op, old, value, place.span)?;
                }
                return Ok(());
            }
            PlaceKind::Index(base, exprs)
                if exprs.len() <= 2
                    && !matches!(base.ty, Ty::Str | Ty::Map(..) | Ty::HashMap(..))
                    && matches!(base.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) =>
            {
                let value = self.eval(value)?;
                let n = exprs.len();
                let i0 = if n >= 1 { self.eval(&exprs[0])?.int() } else { 0 };
                let i1 = if n == 2 { self.eval(&exprs[1])?.int() } else { 0 };
                let holder: &mut Value = match &base.kind {
                    PlaceKind::Global(id) => &mut self.globals[*id],
                    PlaceKind::Local(slot) => {
                        let at = self.base + slot;
                        match &self.stack[at] {
                            Value::Alias(true, id) => {
                                let id = *id;
                                &mut self.globals[id]
                            }
                            Value::Alias(false, target) => {
                                let target = *target;
                                &mut self.stack[target]
                            }
                            Value::Part(part) => {
                                let part = Rc::clone(part);
                                self.part_mut(&part, base.span)?
                            }
                            _ => &mut self.stack[at],
                        }
                    }
                    _ => unreachable!(),
                };
                let span = place.span;
                let cell: &mut Value = match holder {
                    Value::Array(array) if array.dims.len() == 1 && n == 1 => {
                        let len = array.dims[0];
                        if i0 < 0 || i0 >= len {
                            return trap(
                                span,
                                format!("index {i0} is outside an array dimension of length {len}"),
                            );
                        }
                        &mut Rc::make_mut(array).data[i0 as usize]
                    }
                    Value::Array(array) if array.dims.len() == 2 && n == 2 => {
                        let l0 = array.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                span,
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = array.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                span,
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        &mut Rc::make_mut(array).data[(i0 * l1 + i1) as usize]
                    }
                    Value::List(items) if n == 1 => {
                        let len = items.len();
                        match usize::try_from(i0).ok().filter(|i| *i < len) {
                            Some(i) => &mut Rc::make_mut(items)[i],
                            None => {
                                return trap(
                                    span,
                                    format!("index {i0} is outside a list of length {len}"),
                                )
                            }
                        }
                    }
                    _ => {
                        let buf = [Value::Int(i0), Value::Int(i1)];
                        self.resolve(place, &mut buf[..n].iter())?
                    }
                };
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(span, LENT);
                    }
                    *cell = combine(op, old, value, span)?;
                }
                return Ok(());
            }
            _ => {}
        }
        let _ = self.assign(place, op, value)?;
        Ok(())
    }

    /// Stores into `place`, and gives the indices it was reached by.
    fn assign(&mut self, place: &'p Place, op: AssignOp, value: &'p Expr) -> Res<Vec<Value>> {
        // As in C++17: the right side first, then the place.
        let value = self.eval(value)?;
        let mut indices = Vec::new();
        self.indices(place, &mut indices)?;
        // Storing into a string or under a new map key has no cell to
        // point at, so those two go through their container.
        if let PlaceKind::Index(base, _) = &place.kind {
            if matches!(base.ty, Ty::Str | Ty::Map(..) | Ty::HashMap(..)) {
                let (key, rest) = indices.split_last().expect("an index");
                let program = self.program;
                let holder = self.resolve(base, &mut rest.iter())?;
                match holder {
                    Value::Str(bytes) => {
                        let index = key.int();
                        let len = bytes.len();
                        let Some(i) = usize::try_from(index).ok().filter(|i| *i < len) else {
                            return trap(
                                place.span,
                                format!("index {index} is outside a string of length {len}"),
                            );
                        };
                        let old = Value::Char(bytes[i]);
                        let new = combine(op, old, value, place.span)?;
                        if let Value::Char(c) = new {
                            Rc::make_mut(bytes)[i] = c;
                        }
                        return Ok(indices);
                    }
                    Value::Map(entries) if op == AssignOp::Set => {
                        Rc::make_mut(entries).insert(key.clone(), value);
                        return Ok(indices);
                    }
                    Value::Map(entries) => {
                        let map = Rc::make_mut(entries);
                        let next = if let Some(old) = map.get(key).cloned() {
                            combine(op, old, value, place.span)?
                        } else {
                            match op {
                                AssignOp::Max
                                | AssignOp::Min
                                | AssignOp::Binary(
                                    BinOp::Add
                                    | BinOp::Mul
                                    | BinOp::BitOr
                                    | BinOp::BitXor
                                    | BinOp::BitAnd,
                                ) => value,
                                AssignOp::Binary(BinOp::Sub) => {
                                    combine(op, zero(&place.ty, program), value, place.span)?
                                }
                                _ => {
                                    return trap(
                                        place.span,
                                        "this key is not in the map; store with `m[k] = v` first",
                                    )
                                }
                            }
                        };
                        map.insert(key.clone(), next);
                        return Ok(indices);
                    }
                    Value::Moved => return trap(place.span, LENT),
                    _ => {}
                }
            }
        }
        let cell = self.resolve(place, &mut indices.iter())?;
        if op == AssignOp::Set {
            *cell = value;
        } else {
            let old = std::mem::take(cell);
            if matches!(old, Value::Moved) {
                *cell = old;
                return trap(place.span, LENT);
            }
            *cell = combine(op, old, value, place.span)?;
        }
        Ok(indices)
    }

    // ----- expressions -----

    fn eval(&mut self, e: &'p Expr) -> Res<Value> {
        let span = e.span;
        Ok(match &e.kind {
            ExprKind::Unit | ExprKind::Error => Value::Unit,
            ExprKind::Int(v) => Value::Int(*v),
            ExprKind::Float(v) => Value::Float(*v),
            ExprKind::Bool(v) => Value::Bool(*v),
            ExprKind::Char(v) => Value::Char(*v),
            ExprKind::Str(bytes) => Value::Str(Rc::new(bytes.clone())),
            ExprKind::None => Value::Opt(OptVal::None),
            ExprKind::Some(inner) => Value::some(self.eval(inner)?),
            ExprKind::Local(slot) => match self.local(*slot) {
                Value::Moved => return trap(span, LENT),
                value => value.clone(),
            },
            ExprKind::Global(id) => match &self.globals[*id] {
                Value::Moved => return trap(span, LENT),
                value => value.clone(),
            },
            ExprKind::Unary(op, operand) => {
                let value = self.eval(operand)?;
                match (op, value) {
                    (UnOp::Neg, Value::Int(v)) => match v.checked_neg() {
                        Some(v) => Value::Int(v),
                        None => return trap(span, format!("integer overflow: `-{v}` does not fit in an `Int` (64 bits)")),
                    },
                    (UnOp::Neg, Value::I128(v)) => match v.checked_neg() {
                        Some(v) => Value::I128(v),
                        None => return trap(span, format!("integer overflow: `-{v}` does not fit in an `I128` (128 bits)")),
                    },
                    (UnOp::Neg, Value::U64(v)) => Value::U64(v.wrapping_neg()),
                    (UnOp::Neg, Value::Float(v)) => Value::Float(-v),
                    (UnOp::Not, Value::Bool(v)) => Value::Bool(!v),
                    (UnOp::BitNot, Value::Int(v)) => Value::Int(!v),
                    (UnOp::BitNot, Value::I128(v)) => Value::I128(!v),
                    (UnOp::BitNot, Value::U64(v)) => Value::U64(!v),
                    (_, other) => other,
                }
            }
            ExprKind::Binary(op, lhs, rhs) => match op {
                BinOp::And => Value::Bool(self.eval(lhs)?.bool() && self.eval(rhs)?.bool()),
                BinOp::Or => Value::Bool(self.eval(lhs)?.bool() || self.eval(rhs)?.bool()),
                BinOp::Coalesce => match self.eval(lhs)? {
                    Value::Opt(OptVal::Int(v)) => {
                        if lhs.ty == e.ty {
                            Value::Opt(OptVal::Int(v))
                        } else {
                            Value::Int(v)
                        }
                    }
                    Value::Opt(OptVal::Boxed(inner)) => {
                        // An optional default keeps the optional; a plain
                        // one gives what is inside, optional or not.
                        if lhs.ty == e.ty {
                            Value::Opt(OptVal::Boxed(inner))
                        } else {
                            Rc::try_unwrap(inner).unwrap_or_else(|rc| (*rc).clone())
                        }
                    }
                    _ => self.eval(rhs)?,
                },
                _ => {
                    let l = self.eval(lhs)?;
                    let r = self.eval(rhs)?;
                    binary(*op, l, r, span)?
                }
            },
            ExprKind::Cmp(op, lhs, rhs) => {
                let l = self.eval(lhs)?;
                let r = self.eval(rhs)?;
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    (Value::Char(a), Value::Char(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    (Value::Bool(a), Value::Bool(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    _ => {}
                }
                if matches!(op, CmpOp::Eq | CmpOp::Ne) {
                    let eq = self.eq_ty_val(&lhs.ty, &l, &r, span)?;
                    return Ok(Value::Bool(if *op == CmpOp::Eq { eq } else { !eq }));
                }
                if matches!(op, CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge) {
                    let ord = self.cmp_ty_val(&lhs.ty, &l, &r, span)?;
                    let res = match op {
                        CmpOp::Lt => ord.is_lt(),
                        CmpOp::Le => ord.is_le(),
                        CmpOp::Gt => ord.is_gt(),
                        CmpOp::Ge => ord.is_ge(),
                        _ => unreachable!(),
                    };
                    return Ok(Value::Bool(res));
                }
                Value::Bool(compare(*op, &l, &r))
            }
            ExprKind::Cast(cast, operand) => {
                let value = self.eval(operand)?;
                self.cast(*cast, value, span)?
            }
            ExprKind::Index(base, indices) => self.index(base, indices, span)?,
            ExprKind::Field(base, index) => {
                // A field of a variable is read in place, without first
                // copying the whole value.
                let direct = match &base.kind {
                    ExprKind::Local(slot) => Some(self.local(*slot)),
                    ExprKind::Global(id) => Some(&self.globals[*id]),
                    _ => None,
                };
                match direct {
                    Some(Value::Tuple(fields)) => fields
                        .get(*index)
                        .expect("field index within tuple"),
                    Some(Value::Moved) => return trap(span, LENT),
                    _ => match self.eval(base)? {
                        Value::Tuple(fields) => fields
                            .get(*index)
                            .expect("field index within tuple"),
                        _ => return trap(span, "internal: field of a value without fields"),
                    },
                }
            }
            ExprKind::Tuple(items) => {
                if items.len() == 1
                    && !matches!(
                        &e.ty,
                        Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..)
                    )
                {
                    let v = self.eval(&items[0])?;
                    if let Value::Int(x) = v {
                        return Ok(Value::Tuple(TupleVal::OneInt(x)));
                    }
                    return Ok(Value::Tuple(TupleVal::Boxed(Rc::new(vec![v]))));
                }
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item)?);
                }
                match &e.ty {
                    Ty::List(_) => Value::List(Rc::new(values)),
                    Ty::Set(_) | Ty::HashSet(_) => Value::Set(Rc::new(values.into_iter().collect())),
                    Ty::Map(..) | Ty::HashMap(..) => Value::Map(Rc::default()),
                    _ => Value::tuple(values),
                }
            }
            ExprKind::Variant(tag, items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item)?);
                }
                Value::Enum(*tag as u32, Rc::new(values))
            }
            ExprKind::IsVariant(value, tag) => match self.eval(value)? {
                Value::Enum(actual, _) => Value::Bool(actual as usize == *tag),
                _ => Value::Bool(false),
            },
            ExprKind::Payload(value, tag, index) => match self.eval(value)? {
                Value::Enum(actual, payload) if actual as usize == *tag => payload[*index].clone(),
                _ => return trap(span, "internal: payload of another variant"),
            },
            ExprKind::IsSome(value) => {
                Value::Bool(matches!(self.eval(value)?, Value::Opt(opt) if opt.is_some()))
            }
            ExprKind::Unwrap(value, _) => match self.eval(value)? {
                Value::Opt(opt) => match opt.unwrap_owned() {
                    Some(v) => v,
                    None => return trap(span, "`!` on `none`: there is no value here"),
                },
                _ => return trap(span, "`!` on `none`: there is no value here"),
            },
            ExprKind::If(cond, then, otherwise) => {
                if self.eval(cond)?.bool() {
                    self.eval(then)?
                } else {
                    self.eval(otherwise)?
                }
            }
            ExprKind::Do(block) => match self.block(block)? {
                Flow::Give(value) => value,
                _ => return trap(span, "internal: a block ended without its value"),
            },
            ExprKind::Call(id, args) => self.call(*id, args, span)?,
            ExprKind::CallExpr(callee, args) => {
                let fv = self.eval(callee)?;
                let mut vals = vec![Value::Unit; args.len()];
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Value(e) = arg {
                        vals[i] = self.eval(e)?;
                    }
                }
                // `mut` arguments are lent for the call, as to a
                // top-level fn.
                let mut lent: Vec<(usize, &'p Place, Vec<Value>)> = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Mut(place) = arg {
                        let mut indices = Vec::new();
                        self.indices(place, &mut indices)?;
                        let cell = self.resolve(place, &mut indices.iter())?;
                        let value = std::mem::replace(cell, Value::Moved);
                        if matches!(value, Value::Moved) {
                            return trap(place.span, LENT);
                        }
                        vals[i] = value;
                        lent.push((i, place, indices));
                    }
                }
                let at: Vec<usize> = lent.iter().map(|(i, ..)| *i).collect();
                let (value, back) = self.call_lending(fv, vals, &at, span)?;
                for ((_, place, indices), value) in lent.into_iter().zip(back) {
                    *self.resolve(place, &mut indices.iter())? = value;
                }
                value
            }
            ExprKind::Closure(cid) => {
                let def = &self.program.closures[*cid];
                let mut caps = Vec::with_capacity(def.captures.len());
                for cap in &def.captures {
                    if cap.is_global {
                        if cap.by_ref {
                            caps.push(CaptureVal::GlobalRef(cap.outer));
                        } else {
                            caps.push(CaptureVal::Val(self.globals[cap.outer].clone()));
                        }
                    } else if cap.by_ref {
                        // A variable this fn itself captured stays where it lives.
                        caps.push(match self.stack[self.base + cap.outer] {
                            Value::Alias(true, id) => CaptureVal::GlobalRef(id),
                            Value::Alias(false, at) => CaptureVal::StackRef(at),
                            Value::Part(ref part) => CaptureVal::PartRef(Rc::clone(part)),
                            _ => CaptureVal::StackRef(self.base + cap.outer),
                        });
                    } else {
                        caps.push(CaptureVal::Val(self.local(cap.outer).clone()));
                    }
                }
                Value::Closure(*cid, Rc::new(caps))
            }
            ExprKind::Range { lo, hi } => {
                let lo = self.eval(lo)?.int();
                let hi = self.eval(hi)?.int();
                Value::Range(lo, hi)
            }
            ExprKind::DomainVal(id) => {
                let (lo, len) = self.domains[*id];
                Value::DomainVal(lo, len)
            }
            ExprKind::Builtin(op, args) => self.builtin(*op, args, e)?,
            ExprKind::Mutate(op, place, args) => {
                let value = self.mutate(*op, place, args, span)?;
                if self.tracer.is_some() && root_global(place).is_some() {
                    let global = root_global(place).unwrap_or_default();
                    self.traced(|tracer, view| {
                        tracer.visible[global] = true;
                        tracer.step(view, span, None, None, None)
                    });
                }
                value
            }
            ExprKind::Sort(place, sort_by, desc) => {
                let desc = match desc {
                    Some(desc) => self.eval(desc)?.bool(),
                    None => false,
                };
                let mut indices = Vec::new();
                self.indices(place, &mut indices)?;
                let cell = self.resolve(place, &mut indices.iter())?;
                let taken = std::mem::replace(cell, Value::Moved);
                let (mut items_vec, is_array, dims) = match taken {
                    Value::List(items) => {
                        let v = Rc::try_unwrap(items).unwrap_or_else(|r| (*r).clone());
                        (v, false, Vec::new())
                    }
                    Value::Array(arr) => {
                        let a = Rc::try_unwrap(arr).unwrap_or_else(|r| (*r).clone());
                        (a.data, true, a.dims)
                    }
                    _ => return trap(span, LENT),
                };
                let elem_ty = match &place.ty {
                    Ty::List(e) | Ty::Array { elem: e, .. } => (**e).clone(),
                    _ => Ty::Int,
                };
                match sort_by {
                    SortBy::Natural => {
                        items_vec = self.merge_sort_ty(items_vec, &elem_ty, desc, span)?;
                    }
                    SortBy::Key(slot, key) => {
                        let base = self.base;
                        let mut keyed = Vec::with_capacity(items_vec.len());
                        for item in items_vec {
                            self.stack[base + slot] = item.clone();
                            keyed.push((self.eval(key)?, item));
                        }
                        keyed = self.merge_sort_keyed(keyed, &key.ty, desc, span)?;
                        items_vec = keyed.into_iter().map(|(_, item)| item).collect();
                    }
                    SortBy::Fn { func, is_order } => {
                        let fv = self.eval(func)?;
                        if *is_order {
                            // Merge-sort using fallible comparator `fv(a, b) -> Bool`.
                            items_vec = self.merge_sort_by(items_vec, &fv, desc, span)?;
                        } else {
                            let key_ty = match &func.ty {
                                Ty::Fn(_, r) => (**r).clone(),
                                _ => Ty::Int,
                            };
                            let mut keyed = Vec::with_capacity(items_vec.len());
                            for item in items_vec {
                                let k = self.call_value(fv.clone(), vec![item.clone()], span)?;
                                keyed.push((k, item));
                            }
                            keyed = self.merge_sort_keyed(keyed, &key_ty, desc, span)?;
                            items_vec = keyed.into_iter().map(|(_, item)| item).collect();
                        }
                    }
                }
                let restored = if is_array {
                    Value::Array(Rc::new(Array {
                        dims,
                        data: items_vec,
                    }))
                } else {
                    Value::List(Rc::new(items_vec))
                };
                *self.resolve(place, &mut indices.iter())? = restored;
                Value::Unit
            }
            ExprKind::Read(spec) => self.read(spec, span)?,
            ExprKind::Debug(line, args) => {
                let mut values = Vec::new();
                for arg in args {
                    values.push(self.eval(&arg.value)?);
                }
                let view = self.view();
                // A string literal names the checkpoint; without one the
                // arguments as written do, and each value is shown by name.
                let label: Vec<&str> = args
                    .iter()
                    .zip(&values)
                    .filter(|(arg, _)| arg.label)
                    .filter_map(|(_, value)| match value {
                        Value::Str(bytes) => std::str::from_utf8(bytes).ok(),
                        _ => None,
                    })
                    .collect();
                let shown: Vec<(&cpl_ir::DebugArg, &Value)> =
                    args.iter().zip(&values).filter(|(arg, _)| !arg.label).collect();
                let name = if label.is_empty() {
                    shown.iter().map(|(arg, _)| arg.text.as_str()).collect::<Vec<_>>().join(", ")
                } else {
                    label.join(" ")
                };
                let text = shown
                    .iter()
                    .map(|(arg, value)| {
                        let value = trace::text_of(value, &arg.value.ty, &view);
                        if label.is_empty() {
                            format!("{} = {value}", arg.text)
                        } else {
                            value
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let json = match shown.as_slice() {
                    [(arg, value)] => trace::json_of(value, &arg.value.ty, &view),
                    many => {
                        let parts: Vec<String> = many
                            .iter()
                            .map(|(arg, value)| trace::json_of(value, &arg.value.ty, &view))
                            .collect();
                        format!("[{}]", parts.join(","))
                    }
                };
                let prefix = if label.is_empty() { String::new() } else { format!("{name}: ") };
                self.debug_out
                    .extend_from_slice(format!("[line {line}] {prefix}{text}\n").as_bytes());
                let name = name.clone();
                self.traced(|tracer, view| tracer.debugged(view, span, &name, text, json));
                Value::Unit
            }
            ExprKind::Print(args, end) => {
                let start = self.out.len();
                let sparse = args.iter().any(|arg| arg.value.ty.may_show_nothing());
                for (i, arg) in args.iter().enumerate() {
                    let value = self.eval(&arg.value)?;
                    let mut buf = Vec::new();
                    self.show_val(&mut buf, &value, &arg.value.ty, arg.digits, span)?;
                    // An empty collection shows nothing and takes no space.
                    if buf.is_empty() && arg.value.ty.may_show_nothing() {
                        continue;
                    }
                    if i > 0 && (self.out.len() > start || !sparse) {
                        self.out.push(b' ');
                    }
                    self.out.extend_from_slice(&buf);
                }
                match end {
                    Some(end) => {
                        let value = self.eval(end)?;
                        let mut buf = Vec::new();
                        self.show_val(&mut buf, &value, &Ty::Str, 6, span)?;
                        self.out.extend_from_slice(&buf);
                    }
                    None => self.out.push(b'\n'),
                }
                if self.tracer.is_some() {
                    let text = String::from_utf8_lossy(&self.out[start.min(self.out.len())..]).into_owned();
                    self.traced(|tracer, view| tracer.printed(view, span, &text));
                }
                if self.program.interactive {
                    let _ = self.sink.write_all(&self.out);
                    let _ = self.sink.flush();
                    self.out.clear();
                } else {
                    self.flush_if_large();
                }
                Value::Unit
            }
            ExprKind::Format(pieces) => {
                let mut text = Vec::new();
                for piece in pieces {
                    match piece {
                        FmtPiece::Text(bytes) => text.extend_from_slice(bytes),
                        FmtPiece::Value(FmtArg { value, digits }) => {
                            let v = self.eval(value)?;
                            self.show_val(&mut text, &v, &value.ty, *digits, span)?;
                        }
                        FmtPiece::Number(value, fmt) => match self.eval(value)? {
                            Value::Int(v) => text.extend(fmt.int(v).bytes()),
                            Value::Float(v) => text.extend(fmt.float(v).bytes()),
                            _ => unreachable!("a number format takes an Int or an F64"),
                        },
                    }
                }
                Value::Str(Rc::new(text))
            }
            ExprKind::Fill(dims, value) => {
                let dims = self.dims(dims, span)?;
                let cell = match value {
                    Some(value) => self.eval(value)?,
                    None => match &e.ty {
                        Ty::Array { elem, .. } => zero(elem, self.program),
                        _ => Value::Unit,
                    },
                };
                let cells = dims.iter().product::<i64>() as usize;
                Value::Array(Rc::new(Array {
                    dims,
                    data: vec![cell; cells],
                }))
            }
            ExprKind::DomainLen(id) => Value::Int(self.domains[*id].1),
            ExprKind::DomainLo(id) => Value::Int(self.domains[*id].0),
        })
    }

    /// The length of each dimension of an array being made.
    fn dims(&mut self, dims: &'p [DimSpec], span: Span) -> Res<Vec<i64>> {
        let mut out = Vec::with_capacity(dims.len());
        let mut cells: i64 = 1;
        for dim in dims {
            let len = match dim {
                DimSpec::Pos(d) => self.domains[*d].1,
                DimSpec::Bound(d) => self.domains[*d].1 + 1,
                DimSpec::Int(len) | DimSpec::IntFrom(_, len) => {
                    let len = self.eval(len)?.int();
                    if len < 0 {
                        return trap(span, format!("an array can't have length {len}"));
                    }
                    len
                }
            };
            cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                Some(cells) => cells,
                None => return trap(span, "this array is too large to allocate"),
            };
            out.push(len);
        }
        Ok(out)
    }

    fn index(&mut self, base: &'p Expr, indices: &'p [Expr], span: Span) -> Res<Value> {
        // A variable is indexed in place: only the element is copied.
        let direct = matches!(base.kind, ExprKind::Local(_) | ExprKind::Global(_));
        let held;
        let mut stack_vals: [Value; 2] = [Value::Unit, Value::Unit];
        let mut heap_vals: Vec<Value> = Vec::new();
        let n = indices.len();
        let holder: &Value = if direct {
            if n <= 2 {
                for (i, index) in indices.iter().enumerate() {
                    stack_vals[i] = self.eval(index)?;
                }
            } else {
                heap_vals.reserve(n);
                for index in indices {
                    heap_vals.push(self.eval(index)?);
                }
            }
            match &base.kind {
                ExprKind::Local(slot) => self.local(*slot),
                ExprKind::Global(id) => &self.globals[*id],
                _ => unreachable!("checked above"),
            }
        } else {
            held = self.eval(base)?;
            if n <= 2 {
                for (i, index) in indices.iter().enumerate() {
                    stack_vals[i] = self.eval(index)?;
                }
            } else {
                heap_vals.reserve(n);
                for index in indices {
                    heap_vals.push(self.eval(index)?);
                }
            }
            &held
        };
        let values: &[Value] = if n <= 2 { &stack_vals[..n] } else { &heap_vals };
        match holder {
            Value::Array(array) => {
                let at = if array.dims.len() == 1 && values.len() == 1 {
                    let idx = values[0].int();
                    let len = array.dims[0];
                    if idx < 0 || idx >= len {
                        return trap(
                            span,
                            format!("index {idx} is outside an array dimension of length {len}"),
                        );
                    }
                    idx as usize
                } else if array.dims.len() == 2 && values.len() == 2 {
                    let i0 = values[0].int();
                    let l0 = array.dims[0];
                    if i0 < 0 || i0 >= l0 {
                        return trap(
                            span,
                            format!("index {i0} is outside an array dimension of length {l0}"),
                        );
                    }
                    let i1 = values[1].int();
                    let l1 = array.dims[1];
                    if i1 < 0 || i1 >= l1 {
                        return trap(
                            span,
                            format!("index {i1} is outside an array dimension of length {l1}"),
                        );
                    }
                    (i0 * l1 + i1) as usize
                } else {
                    array_offset(&array.dims, values.iter(), span)?
                };
                Ok(array.data[at].clone())
            }
            Value::List(items) => {
                let index = values[0].int();
                match usize::try_from(index).ok().and_then(|i| items.get(i)) {
                    Some(item) => Ok(item.clone()),
                    None => trap(
                        span,
                        format!("index {index} is outside a list of length {}", items.len()),
                    ),
                }
            }
            Value::Str(bytes) => {
                let index = values[0].int();
                match usize::try_from(index).ok().and_then(|i| bytes.get(i)) {
                    Some(byte) => Ok(Value::Char(*byte)),
                    None => trap(
                        span,
                        format!(
                            "index {index} is outside a string of length {}",
                            bytes.len()
                        ),
                    ),
                }
            }
            Value::Map(entries) => match entries.get(&values[0]) {
                Some(value) => Ok(value.clone()),
                None => {
                    let mut key = Vec::new();
                    if let Ty::Map(key_ty, _) = &base.ty {
                        show(&mut key, &values[0], key_ty, 6, self.program, &self.domains);
                    }
                    let key: String = key.iter().map(|b| *b as char).collect();
                    trap(
                        span,
                        format!("the key `{key}` is not in the map; `m.get(k)` gives an optional"),
                    )
                }
            },
            Value::Moved => trap(span, LENT),
            _ => trap(span, "internal: index of a value that has no elements"),
        }
    }

    fn cast(&mut self, cast: Cast, value: Value, span: Span) -> Res<Value> {
        Ok(match cast {
            Cast::IntToFloat => Value::Float(value.int() as f64),
            Cast::IntToI128 => Value::I128(i128::from(value.int())),
            Cast::IntToU64 => Value::U64(value.int() as u64),
            Cast::I128ToInt => match (&value, i64::try_from(value.i128())) {
                (_, Ok(v)) => Value::Int(v),
                _ => return trap(span, format!("{} does not fit in an `Int`", value.i128())),
            },
            Cast::U64ToInt => match i64::try_from(value.u64()) {
                Ok(v) => Value::Int(v),
                Err(_) => return trap(span, format!("{} does not fit in an `Int`", value.u64())),
            },
            Cast::I128ToU64 => match u64::try_from(value.i128()) {
                Ok(v) => Value::U64(v),
                Err(_) => return trap(span, format!("{} does not fit in a `U64`", value.i128())),
            },
            Cast::U64ToI128 => Value::I128(i128::from(value.u64())),
            Cast::I128ToFloat => Value::Float(value.i128() as f64),
            Cast::U64ToFloat => Value::Float(value.u64() as f64),
            Cast::FloatToInt => {
                let v = value.float().trunc();
                if !(-9.2e18..=9.2e18).contains(&v) {
                    return trap(span, format!("{v} does not fit in an `Int`"));
                }
                Value::Int(v as i64)
            }
            Cast::CharToInt => match value {
                Value::Char(c) => Value::Int(i64::from(c)),
                other => other,
            },
            Cast::StepToChar => match u8::try_from(value.int()) {
                Ok(c) if c.is_ascii() => Value::Char(c),
                _ => return trap(span, "`Char + Int` left ASCII (0..=127)"),
            },
            Cast::IntToChar => match u8::try_from(value.int()) {
                Ok(c) => Value::Char(c),
                Err(_) => {
                    return trap(
                        span,
                        format!("{} is not a byte, so it is not a `Char`", value.int()),
                    )
                }
            },
            Cast::Ord | Cast::Trust | Cast::Raw => value,
            Cast::Label(d) => Value::Int(value.int() + self.domains[d].0),
            Cast::FromLabel(d) => {
                let (lo, len) = self.domains[d];
                let label = value.int();
                if label < lo || label - lo >= len {
                    let name = &self.program.domains[d];
                    return trap(
                        span,
                        format!("{label} is not in `{name}`, which is {}", describe(lo, len)),
                    );
                }
                Value::Int(label - lo)
            }
            Cast::FromOrd(d) => {
                let (lo, len) = self.domains[d];
                let ord = value.int();
                if ord < 0 || ord >= len {
                    let name = &self.program.domains[d];
                    return trap(
                        span,
                        format!(
                            "`{name}` has no position {ord}: it is {}",
                            describe(lo, len)
                        ),
                    );
                }
                value
            }
        })
    }

    /// The fn and its list when the loop `id` runs its body as the fn
    /// yields, as the C++ does.
    fn is_fed(&mut self, id: LoopId, seq: &Expr, body: &Block) -> Option<(FnId, usize)> {
        if self.in_closure || self.fed_loops.last().is_some_and(|fed| fed.callee == self.base) {
            return None;
        }
        if let Some(known) = self.fed.get(&id) {
            return *known;
        }
        let found = self.feeds.feeds(id, seq, body);
        self.fed.insert(id, found);
        found
    }

    /// One value of the `yields` fn that is running goes to the body of
    /// the loop it feeds. An `Err` with `left` set means the body left
    /// the loop, and the fn stops where it is.
    fn feed(&mut self, value: Value, span: Span) -> Res<()> {
        let fed = self.fed_loops.last().expect("asked by the caller");
        let (callee, caller, slot, id, body) = (fed.callee, fed.caller, fed.slot, fed.id, fed.body);
        let locals = self.frame_locals.pop();
        self.base = caller;
        self.stack[caller + slot] = value;
        let flow = self.block(body);
        self.base = callee;
        self.frame_locals.extend(locals);
        let left = match flow? {
            Flow::Next | Flow::Continue(None) => return Ok(()),
            Flow::Continue(Some(target)) if target == id => return Ok(()),
            Flow::Break(None) => Flow::Next,
            Flow::Break(Some(target)) if target == id => Flow::Next,
            other => other,
        };
        self.fed_loops.last_mut().expect("still the innermost").left = Some(left);
        trap(span, String::new())
    }

    fn call(&mut self, id: FnId, args: &'p [Arg], span: Span) -> Res<Value> {
        self.call_feeding(id, args, span, None)
    }

    /// Calls a fn; with `fed`, each value it yields runs that loop's body.
    fn call_feeding(&mut self, id: FnId, args: &'p [Arg], span: Span, fed: Option<Fed<'p>>) -> Res<Value> {
        let f = &self.program.fns[id];
        if !self.program.judge_fns.is_empty() && self.program.judge_fns.contains(&id) {
            return trap(span, NO_GRADER);
        }
        self.stack_left(span, Some(&f.name))?;
        let base = match args {
            [] => {
                let base = self.stack.len();
                self.stack.resize(base + f.locals.len(), Value::Unit);
                base
            }
            [Arg::Value(e0)] => {
                let v0 = self.eval(e0)?;
                let base = self.stack.len();
                self.stack.push(v0);
                if f.locals.len() > 1 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            [Arg::Value(e0), Arg::Value(e1)] => {
                let v0 = self.eval(e0)?;
                let v1 = self.eval(e1)?;
                let base = self.stack.len();
                self.stack.push(v0);
                self.stack.push(v1);
                if f.locals.len() > 2 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            [Arg::Value(e0), Arg::Value(e1), Arg::Value(e2)] => {
                let v0 = self.eval(e0)?;
                let v1 = self.eval(e1)?;
                let v2 = self.eval(e2)?;
                let base = self.stack.len();
                self.stack.push(v0);
                self.stack.push(v1);
                self.stack.push(v2);
                if f.locals.len() > 3 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            _ => {
                let mut values = vec![Value::Unit; args.len()];
                // Evaluate by-value arguments first (two-phase borrowing), so patterns
                // like `a.union(a.shifted(w))` can read `a` while preparing the call.
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Value(e) = arg {
                        values[i] = self.eval(e)?;
                    }
                }
                // A `mut` argument names the caller's variable or the part of it:
                // the callee, and a fn value it runs (`() => ids.len`), read and
                // write the one value, as in the C++, where it is a reference.
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Mut(place) = arg {
                        let mut indices = Vec::new();
                        self.indices(place, &mut indices)?;
                        let part = self.part_of(place, &mut indices.into_iter());
                        if matches!(self.part_mut(&part, place.span)?, Value::Moved) {
                            return trap(place.span, LENT);
                        }
                        values[i] = match part.steps.is_empty() {
                            true => Value::Alias(part.global, part.at),
                            false => Value::Part(Rc::new(part)),
                        };
                    }
                }
                let base = self.stack.len();
                self.stack.extend(values);
                self.stack.resize(base + f.locals.len(), Value::Unit);
                base
            }
        };
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let wanted = self.tracer.as_ref().is_some_and(|tracer| tracer.wants_fn(&f.name));
        if let (true, Some(tracer)) = (wanted, &mut self.tracer) {
            tracer.inside += 1;
        }
        let feeding = fed.is_some();
        if let Some(fed) = fed {
            self.fed_loops.push(Fed { callee: base, ..fed });
        }
        let in_closure = std::mem::replace(&mut self.in_closure, false);
        let dom_mark = self.saved_domains.len();
        for d in self.program.fn_guarded_domains(&f.ret, &f.body) {
            self.saved_domains.push((d, self.domains[d]));
        }
        let mut flow = self.block(&f.body);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        if let (true, Some(tracer)) = (wanted, &mut self.tracer) {
            tracer.inside -= 1;
        }
        self.frame_locals.pop();
        self.base = saved;
        if feeding {
            // The body left the loop: the fn stopped at that `yield`.
            if let Some(left) = self.fed_loops.pop().and_then(|fed| fed.left) {
                self.fed_left = Some(left).filter(|left| !matches!(left, Flow::Next));
                flow = Ok(Flow::Next);
            }
        }
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }

    /// The value of a local, through the alias a by-reference capture holds.
    fn local(&self, slot: usize) -> &Value {
        match &self.stack[self.base + slot] {
            Value::Alias(true, id) => &self.globals[*id],
            Value::Alias(false, at) => &self.stack[*at],
            // A part that is gone (its list shrank) reads as lent out.
            Value::Part(part) => self.part(part).unwrap_or(&Value::Moved),
            value => value,
        }
    }

    fn call_value(&mut self, fv: Value, args: Vec<Value>, span: Span) -> Res<Value> {
        Ok(self.call_lending(fv, args, &[], span)?.0)
    }

    /// Calls a fn value. The arguments at `lent` are `mut`: what the fn
    /// left in them comes back, in that order, next to its result.
    fn call_lending(&mut self, fv: Value, args: Vec<Value>, lent: &[usize], span: Span) -> Res<(Value, Vec<Value>)> {
        let Value::Closure(cid, caps) = fv else {
            return trap(span, "internal: tried to call a non-function value");
        };
        let def = &self.program.closures[cid];
        self.stack_left(span, None)?;
        let base = self.stack.len();
        self.stack.resize(base + def.locals.len(), Value::Unit);
        for (slot, val) in def.params.iter().zip(args) {
            self.stack[base + slot] = val;
        }
        for (cap, cval) in def.captures.iter().zip(caps.iter()) {
            let v = match cval {
                CaptureVal::Val(v) => v.clone(),
                // Captured by reference: the slot names the variable, so
                // every call, however deep, reads and writes the one value.
                CaptureVal::StackRef(at) => Value::Alias(false, *at),
                CaptureVal::GlobalRef(id) => Value::Alias(true, *id),
                CaptureVal::PartRef(part) => Value::Part(Rc::clone(part)),
            };
            self.stack[base + cap.inner] = v;
        }
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&def.locals);
        self.call_frames.push(("<closure>", span));
        let in_closure = std::mem::replace(&mut self.in_closure, true);
        let dom_mark = self.saved_domains.len();
        for d in self.program.fn_guarded_domains(&def.ret, &def.body) {
            self.saved_domains.push((d, self.domains[d]));
        }
        let flow = self.block(&def.body);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        self.frame_locals.pop();
        self.base = saved;
        let flow = flow?;
        self.call_frames.pop();
        let back = lent.iter().map(|i| std::mem::take(&mut self.stack[base + def.params[*i]])).collect();
        self.stack.truncate(base);
        let value = match flow {
            Flow::Return(v) | Flow::Give(v) => v,
            _ => Value::Unit,
        };
        Ok((value, back))
    }

    fn eq_ty_val(&mut self, ty: &Ty, l: &Value, r: &Value, span: Span) -> Res<bool> {
        match ty {
            Ty::Struct(id) => {
                if let Some(eq_fn) = self.program.structs[*id].eq_fn {
                    return Ok(self.call_direct(eq_fn, vec![l.clone(), r.clone()], span)?.bool());
                }
                let fields = self.program.structs[*id].fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, ft), (lv, rv)) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        if !self.eq_ty_val(ft, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Tuple(ts) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (t, (lv, rv)) in ts
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        if !self.eq_ty_val(t, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Record(fs) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, t), (lv, rv)) in fs
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        if !self.eq_ty_val(t, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::List(elem) => {
                if let (Value::List(ls), Value::List(rs)) = (l, r) {
                    if ls.len() != rs.len() {
                        return Ok(false);
                    }
                    for (lv, rv) in ls.iter().zip(rs.iter()) {
                        if !self.eq_ty_val(elem, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Opt(inner) => match (l, r) {
                (Value::Opt(OptVal::None), Value::Opt(OptVal::None)) => Ok(true),
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Int(b))) => Ok(a == b),
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.eq_ty_val(inner, a, b, span)
                }
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.eq_ty_val(inner, &Value::Int(*a), b, span)
                }
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Int(b))) => {
                    self.eq_ty_val(inner, a, &Value::Int(*b), span)
                }
                _ => Ok(false),
            },
            _ => Ok(l == r),
        }
    }

    fn cmp_ty_val(
        &mut self,
        ty: &Ty,
        l: &Value,
        r: &Value,
        span: Span,
    ) -> Res<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match ty {
            Ty::Struct(id) => {
                if let Some(lt_fn) = self.program.structs[*id].lt_fn {
                    if self.call_direct(lt_fn, vec![l.clone(), r.clone()], span)?.bool() {
                        return Ok(Ordering::Less);
                    }
                    if self.call_direct(lt_fn, vec![r.clone(), l.clone()], span)?.bool() {
                        return Ok(Ordering::Greater);
                    }
                    return Ok(Ordering::Equal);
                }
                let fields = self.program.structs[*id].fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, ft), (lv, rv)) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        let o = self.cmp_ty_val(ft, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::Tuple(ts) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (t, (lv, rv)) in ts
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        let o = self.cmp_ty_val(t, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::Record(fs) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, t), (lv, rv)) in fs
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        let o = self.cmp_ty_val(t, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::List(elem) => {
                if let (Value::List(ls), Value::List(rs)) = (l, r) {
                    for (lv, rv) in ls.iter().zip(rs.iter()) {
                        let o = self.cmp_ty_val(elem, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(ls.len().cmp(&rs.len()));
                }
                Ok(l.cmp(r))
            }
            Ty::Opt(inner) => match (l, r) {
                (Value::Opt(OptVal::None), Value::Opt(OptVal::None)) => Ok(Ordering::Equal),
                (Value::Opt(OptVal::None), _) => Ok(Ordering::Less),
                (_, Value::Opt(OptVal::None)) => Ok(Ordering::Greater),
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Int(b))) => Ok(a.cmp(b)),
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.cmp_ty_val(inner, a, b, span)
                }
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.cmp_ty_val(inner, &Value::Int(*a), b, span)
                }
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Int(b))) => {
                    self.cmp_ty_val(inner, a, &Value::Int(*b), span)
                }
                _ => Ok(l.cmp(r)),
            },
            _ => Ok(l.cmp(r)),
        }
    }

    fn merge_sort_ty(&mut self, mut xs: Vec<Value>, ty: &Ty, desc: bool, span: Span) -> Res<Vec<Value>> {
        if xs.len() <= 1 {
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = self.merge_sort_ty(xs.split_off(mid), ty, desc, span)?;
        let left = self.merge_sort_ty(xs, ty, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            // `desc`: the right one goes first only when it is the larger.
            let order = self.cmp_ty_val(ty, &right[j], &left[i], span)?;
            if if desc { order.is_gt() } else { order.is_lt() } {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    fn merge_sort_keyed(
        &mut self,
        mut xs: Vec<(Value, Value)>,
        key_ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<(Value, Value)>> {
        if xs.len() <= 1 {
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = self.merge_sort_keyed(xs.split_off(mid), key_ty, desc, span)?;
        let left = self.merge_sort_keyed(xs, key_ty, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            let order = self.cmp_ty_val(key_ty, &right[j].0, &left[i].0, span)?;
            if if desc { order.is_gt() } else { order.is_lt() } {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    fn merge_sort_by(&mut self, mut xs: Vec<Value>, cmp: &Value, desc: bool, span: Span) -> Res<Vec<Value>> {
        if xs.len() <= 1 {
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = xs.split_off(mid);
        let left = self.merge_sort_by(xs, cmp, desc, span)?;
        let right = self.merge_sort_by(right, cmp, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            let pair = if desc {
                vec![left[i].clone(), right[j].clone()]
            } else {
                vec![right[j].clone(), left[i].clone()]
            };
            let lt = self.call_value(cmp.clone(), pair, span)?.bool();
            if lt {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    fn show_val(
        &mut self,
        out: &mut Vec<u8>,
        value: &Value,
        ty: &Ty,
        digits: u32,
        span: Span,
    ) -> Res<()> {
        if let Ty::Struct(id) = ty {
            if let Some(show_fn) = self.program.structs[*id].show_fn {
                let saved = self.call_direct(show_fn, vec![value.clone()], span)?;
                if let Value::Str(bytes) = saved {
                    out.extend_from_slice(&bytes);
                    return Ok(());
                }
            }
        }
        show(out, value, ty, digits, self.program, &self.domains);
        Ok(())
    }

    /// Stops the run before a call would overflow the interpreter's own
    /// stack: the program's recursion is the interpreter's.
    fn stack_left(&self, span: Span, callee: Option<&str>) -> Res<()> {
        let here = 0u8;
        let used = self.stack_top.abs_diff(std::ptr::addr_of!(here) as usize);
        if used > self.stack_budget {
            let callee = callee.map_or("a closure".into(), |name| format!("`{name}`"));
            return trap(span, format!("the stack ran out: {callee} never stops calling itself?"));
        }
        Ok(())
    }

    fn call_direct(&mut self, id: FnId, values: Vec<Value>, span: Span) -> Res<Value> {
        let f = &self.program.fns[id];
        self.stack_left(span, Some(&f.name))?;
        let base = self.stack.len();
        self.stack.extend(values);
        self.stack.resize(base + f.locals.len(), Value::Unit);
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let dom_mark = self.saved_domains.len();
        for d in self.program.fn_guarded_domains(&f.ret, &f.body) {
            self.saved_domains.push((d, self.domains[d]));
        }
        let flow = self.block(&f.body);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.frame_locals.pop();
        self.base = saved;
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }

    fn builtin(&mut self, op: Builtin, args: &'p [Expr], e: &'p Expr) -> Res<Value> {
        let span = e.span;
        // `k in m` and `m.get(k)` on a variable look in it where it is:
        // copying a map to ask it one question makes a loop quadratic.
        if let (Builtin::Contains | Builtin::MapGet, [holder, key]) = (op, args) {
            if matches!(holder.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                let key = self.eval(key)?;
                let held = match &holder.kind {
                    ExprKind::Local(slot) => self.local(*slot),
                    ExprKind::Global(id) => &self.globals[*id],
                    _ => unreachable!("checked above"),
                };
                match (op, held) {
                    (Builtin::Contains, Value::Map(entries)) => {
                        return Ok(Value::Bool(entries.contains_key(&key)));
                    }
                    (Builtin::Contains, Value::Set(items)) => {
                        return Ok(Value::Bool(items.contains(&key)));
                    }
                    (Builtin::MapGet, Value::Map(entries)) => {
                        return Ok(Value::Opt(OptVal::from_option(entries.get(&key).cloned())));
                    }
                    _ => {}
                }
            }
        }
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval(arg)?);
        }
        let mut values = values.into_iter();
        let first = values.next().unwrap_or_default();
        let optional = |value: Option<Value>| Value::Opt(OptVal::from_option(value));
        Ok(match op {
            Builtin::Min => values.fold(first, |best, v| if v < best { v } else { best }),
            Builtin::Max => values.fold(first, |best, v| if v > best { v } else { best }),
            Builtin::Abs => match first {
                Value::Int(v) => match v.checked_abs() {
                    Some(v) => Value::Int(v),
                    None => return trap(span, format!("integer overflow: `abs({v})` does not fit in an `Int` (64 bits)")),
                },
                Value::I128(v) => match v.checked_abs() {
                    Some(v) => Value::I128(v),
                    None => return trap(span, format!("integer overflow: `abs({v})` does not fit in an `I128` (128 bits)")),
                },
                Value::Float(v) => Value::Float(v.abs()),
                other => other,
            },
            Builtin::Sqrt => Value::Float(first.float().sqrt()),
            Builtin::Isqrt => {
                let x = first.int();
                if x < 0 {
                    return trap(span, format!("isqrt({x}) of a negative number"));
                }
                let mut r = (x as f64).sqrt() as i64;
                while r.saturating_add(1).saturating_mul(r.saturating_add(1)) <= x {
                    r += 1;
                }
                while r.saturating_mul(r) > x {
                    r -= 1;
                }
                Value::Int(r)
            }
            Builtin::Flush => {
                let _ = self.sink.write_all(&self.out);
                let _ = self.sink.flush();
                self.out.clear();
                Value::Unit
            }
            Builtin::Atan2 => {
                let x = values.next().unwrap_or_default().float();
                Value::Float(first.float().atan2(x))
            }
            Builtin::Floor | Builtin::Ceil | Builtin::Round => {
                let v = match op {
                    Builtin::Floor => first.float().floor(),
                    Builtin::Ceil => first.float().ceil(),
                    _ => first.float().round(),
                };
                if !(-9.2e18..=9.2e18).contains(&v) {
                    return trap(span, format!("{v} does not fit in an `Int`"));
                }
                Value::Int(v as i64)
            }
            Builtin::CountQuery => {
                self.queries += 1;
                let limit = first.int();
                if self.queries > limit {
                    return trap(
                        span,
                        format!("query limit exceeded: this is query {}, and the limit is {limit}", self.queries),
                    );
                }
                Value::Unit
            }
            Builtin::SwapInput => {
                match &mut self.peer {
                    Some(peer) => std::mem::swap(&mut self.input, peer),
                    None => return trap(span, "there is no solution to talk to: an `@interactor` runs under `cpl run` or `cpl stress`"),
                }
                Value::Unit
            }
            Builtin::Popcount => Value::Int(i64::from(bits64(&first).count_ones())),
            Builtin::Ctz => Value::Int(i64::from(bits64(&first).trailing_zeros())),
            Builtin::Clz => Value::Int(i64::from(bits64(&first).leading_zeros())),
            Builtin::AxisLen => match (&first, values.next()) {
                (Value::Array(array), Some(Value::Int(axis))) => Value::Int(array.dims[axis as usize]),
                _ => Value::Int(0),
            },
            Builtin::BitWidth => {
                let u = bits64(&first);
                Value::Int(i64::from(64 - u.leading_zeros()))
            }
            Builtin::BitCeil => {
                let x = first.int();
                if x <= 1 {
                    Value::Int(1)
                } else {
                    let shift = 64 - ((x - 1) as u64).leading_zeros();
                    Value::Int(1i64.wrapping_shl(shift))
                }
            }
            Builtin::Rand => {
                let lo = first.int();
                let hi = values.next().unwrap_or_default().int();
                if lo > hi {
                    return trap(span, format!("rand({lo}, {hi}): lo > hi"));
                }
                let span_u = (hi as i128 - lo as i128 + 1) as u128;
                let r = splitmix64(&mut self.rng_state) as u128;
                let off = (r % span_u) as i128;
                Value::Int((lo as i128 + off) as i64)
            }
            Builtin::Copy => first,
            Builtin::ListFilled => {
                let n = first.int();
                let val = values.next().unwrap_or_default();
                if n < 0 {
                    return trap(span, format!("List.filled with negative length {n}"));
                }
                Value::List(Rc::new(vec![val; n as usize]))
            }
            Builtin::Join => {
                let text = |value: Value| match value {
                    Value::Str(bytes) => bytes,
                    _ => Rc::new(Vec::new()),
                };
                let sep = text(first);
                let seq = values.next().unwrap_or_default();
                let none_text = values.next().map(text);
                let elem = match &args[1].ty {
                    Ty::List(elem) | Ty::Array { elem, .. } => (**elem).clone(),
                    _ => Ty::Unit,
                };
                let items: &[Value] = match &seq {
                    Value::List(items) => items,
                    Value::Array(array) => &array.data,
                    _ => &[],
                };
                let mut buf = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        buf.extend_from_slice(&sep);
                    }
                    match (item, &none_text) {
                        (Value::Opt(OptVal::None), Some(text)) => buf.extend_from_slice(text),
                        _ => self.show_val(&mut buf, item, &elem, cpl_ir::FLOAT_DIGITS, span)?,
                    }
                }
                Value::Str(Rc::new(buf))
            }
            Builtin::SeqGet => {
                let at = values.next().unwrap_or_default().int();
                let item = match &first {
                    Value::List(items) => usize::try_from(at).ok().and_then(|i| items.get(i)).cloned(),
                    _ => None,
                };
                optional(item)
            }
            Builtin::ShowStr => {
                let mut buf = Vec::new();
                let arg_ty = args.first().map(|a| &a.ty).unwrap_or(&Ty::Unit);
                self.show_val(&mut buf, &first, arg_ty, cpl_ir::FLOAT_DIGITS, span)?;
                Value::Str(Rc::new(buf))
            }
            Builtin::ParseInt => match first {
                Value::Str(bytes) => {
                    let s = std::str::from_utf8(&bytes).unwrap_or("").trim();
                    match s.parse::<i64>() {
                        Ok(v) => Value::Int(v),
                        Err(_) => return trap(span, format!("`{s}` is not an Int")),
                    }
                }
                other => other,
            },
            Builtin::StrToList => match first {
                Value::Str(bytes) => {
                    Value::List(Rc::new(bytes.iter().copied().map(Value::Char).collect()))
                }
                other => other,
            },
            Builtin::ListToStr => match first {
                Value::List(items) => {
                    let bytes: Vec<u8> = items
                        .iter()
                        .map(|v| match v {
                            Value::Char(c) => *c,
                            _ => 0,
                        })
                        .collect();
                    Value::Str(Rc::new(bytes))
                }
                other => other,
            },
            Builtin::ArrayToList => match first {
                Value::Array(arr) => Value::List(Rc::new(arr.data.clone())),
                other => other,
            },
            Builtin::ListToArray => match first {
                Value::List(items) => Value::Array(Rc::new(Array {
                    dims: vec![items.len() as i64],
                    data: items.to_vec(),
                })),
                other => other,
            },
            Builtin::Len => Value::Int(match &first {
                Value::Array(array) => array.data.len() as i64,
                Value::List(items) => items.len() as i64,
                Value::Str(bytes) => bytes.len() as i64,
                Value::Map(entries) => entries.len() as i64,
                Value::Set(items) => items.len() as i64,
                Value::Range(lo, hi) => (hi - lo).max(0),
                Value::DomainVal(_, len) => *len,
                _ => 0,
            }),
            Builtin::First => match &first {
                Value::List(items) => optional(items.first().cloned()),
                Value::Array(array) => optional(array.data.first().cloned()),
                Value::Range(lo, hi) => optional((lo < hi).then_some(Value::Int(*lo))),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::Last => match &first {
                Value::List(items) => optional(items.last().cloned()),
                Value::Array(array) => optional(array.data.last().cloned()),
                Value::Range(lo, hi) => optional((lo < hi).then_some(Value::Int(*hi - 1))),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::FirstVal => match &first {
                Value::List(items) => match items.first() {
                    Some(v) => v.clone(),
                    None => return trap(span, "the list is empty"),
                },
                _ => return trap(span, "the list is empty"),
            },
            Builtin::LastVal => match &first {
                Value::List(items) => match items.last() {
                    Some(v) => v.clone(),
                    None => return trap(span, "the list is empty"),
                },
                _ => return trap(span, "the list is empty"),
            },
            Builtin::Contains => {
                let needle = values.next().unwrap_or_default();
                Value::Bool(match &first {
                    Value::List(items) => items.contains(&needle),
                    Value::Array(array) => array.data.contains(&needle),
                    Value::Set(items) => items.contains(&needle),
                    Value::Map(entries) => entries.contains_key(&needle),
                    Value::Str(bytes) => match &needle {
                        Value::Char(c) => bytes.contains(c),
                        Value::Str(part) => {
                            part.is_empty() || bytes.windows(part.len()).any(|at| at == &part[..])
                        }
                        _ => false,
                    },
                    Value::Range(lo, hi) => {
                        let x = needle.int();
                        x >= *lo && x < *hi
                    }
                    _ => false,
                })
            }
            Builtin::MapGet => {
                let key = values.next().unwrap_or_default();
                match &first {
                    Value::Map(entries) => optional(entries.get(&key).cloned()),
                    _ => Value::Opt(OptVal::None),
                }
            }
            Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt => {
                use std::ops::Bound::{Excluded, Included, Unbounded};
                let key = values.next().unwrap_or_default();
                let range = match op {
                    Builtin::FirstGe => (Included(key), Unbounded),
                    Builtin::FirstGt => (Excluded(key), Unbounded),
                    Builtin::LastLe => (Unbounded, Included(key)),
                    _ => (Unbounded, Excluded(key)),
                };
                let first_in = matches!(op, Builtin::FirstGe | Builtin::FirstGt);
                match &first {
                    Value::Set(items) => {
                        let mut found = items.range(range);
                        optional(if first_in { found.next() } else { found.next_back() }.cloned())
                    }
                    Value::Map(entries) => {
                        let mut found = entries.range(range);
                        let entry = if first_in { found.next() } else { found.next_back() };
                        optional(entry.map(|(key, val)| Value::tuple(vec![key.clone(), val.clone()])))
                    }
                    _ => Value::Opt(OptVal::None),
                }
            }
            Builtin::SetMin => match &first {
                Value::Set(items) => optional(items.first().cloned()),
                Value::Map(entries) => optional(entries.iter().next().map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::SetMax => match &first {
                Value::Set(items) => optional(items.last().cloned()),
                Value::Map(entries) => optional(entries.iter().next_back().map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::Sorted => match first {
                Value::List(mut items) => {
                    Rc::make_mut(&mut items).sort();
                    Value::List(items)
                }
                Value::Array(arr) => {
                    let mut items = arr.data.clone();
                    items.sort();
                    Value::List(Rc::new(items))
                }
                Value::Set(items) => Value::List(Rc::new(items.iter().cloned().collect())),
                other => other,
            },
            Builtin::Reversed => match first {
                Value::List(mut items) => {
                    Rc::make_mut(&mut items).reverse();
                    Value::List(items)
                }
                Value::Str(mut bytes) => {
                    Rc::make_mut(&mut bytes).reverse();
                    Value::Str(bytes)
                }
                Value::Array(arr) => {
                    let mut items = arr.data.clone();
                    items.reverse();
                    Value::List(Rc::new(items))
                }
                other => other,
            },
            Builtin::PosNext => {
                let len = values.next().unwrap_or_default().int();
                let next = first.int() + 1;
                optional((next < len).then_some(Value::Int(next)))
            }
            Builtin::PosPrev => {
                let ord = first.int();
                optional((ord > 0).then_some(Value::Int(ord - 1)))
            }
            Builtin::PosTryOrd(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                optional((ord >= 0 && ord < len).then_some(Value::Int(ord)))
            }
            Builtin::PosOffset(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let delta = values.next().unwrap_or_default().int();
                let target = ord.saturating_add(delta);
                optional((target >= 0 && target < len).then_some(Value::Int(target)))
            }
            Builtin::PosCyc(d) => {
                let (_, len) = self.domains[d];
                if len <= 0 {
                    return trap(span, "cyc on empty domain");
                }
                let ord = first.int();
                let delta = values.next().unwrap_or_default().int();
                Value::Int((ord + delta).rem_euclid(len))
            }
            Builtin::BoundStepL(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let k = values.next().unwrap_or_default().int();
                let b = ord + 1 - k;
                if b < 0 || b > len {
                    return trap(span, format!("window of {k} leaves domain (0..={len})"));
                }
                Value::Int(b)
            }
            Builtin::BoundStepR(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let k = values.next().unwrap_or_default().int();
                let b = ord + k;
                if b < 0 || b > len {
                    return trap(span, format!("window of {k} leaves domain (0..={len})"));
                }
                Value::Int(b)
            }
            Builtin::BoundLeft(_d) => {
                let ord = first.int();
                optional((ord > 0).then_some(Value::Int(ord - 1)))
            }
            Builtin::BoundRight(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                optional((ord < len).then_some(Value::Int(ord)))
            }
            Builtin::RangeLo => match first {
                Value::Range(lo, _) => Value::Int(lo),
                _ => Value::Int(0),
            },
            Builtin::RangeHi => match first {
                Value::Range(_, hi) => Value::Int(hi),
                _ => Value::Int(0),
            },
            Builtin::RangeRev => match first {
                Value::Range(lo, hi) => {
                    let mut out = Vec::with_capacity((hi - lo).max(0) as usize);
                    let mut v = hi - 1;
                    while v >= lo {
                        out.push(Value::Int(v));
                        if v == lo {
                            break;
                        }
                        v -= 1;
                    }
                    Value::List(Rc::new(out))
                }
                _ => Value::List(Rc::new(Vec::new())),
            },
        })
    }

    fn mutate(
        &mut self,
        op: Mutation,
        place: &'p Place,
        args: &'p [Expr],
        span: Span,
    ) -> Res<Value> {
        // As in C++17: what is called on comes before the arguments.
        let mut indices = Vec::new();
        self.indices(place, &mut indices)?;
        let mut arg_vals = Vec::with_capacity(args.len());
        for a in args {
            arg_vals.push(self.eval(a)?);
        }
        // A `yield` of a fn that feeds a loop: the body runs on the value.
        if let (PlaceKind::Local(slot), Some(fed)) = (&place.kind, self.fed_loops.last()) {
            if fed.callee == self.base && fed.acc == *slot {
                match (op, arg_vals.pop()) {
                    (Mutation::Push, Some(value)) => self.feed(value, span)?,
                    (Mutation::Extend, Some(Value::List(items))) => {
                        for item in items.iter() {
                            self.feed(item.clone(), span)?;
                        }
                    }
                    (Mutation::Extend, Some(Value::Array(array))) => {
                        for item in array.data.iter() {
                            self.feed(item.clone(), span)?;
                        }
                    }
                    _ => return trap(span, "internal: a `yields` fn did something else to its list"),
                }
                return Ok(Value::Unit);
            }
        }
        let arg = arg_vals.first().cloned().unwrap_or(Value::Unit);
        let arg2 = arg_vals.get(1).cloned().unwrap_or(Value::Unit);
        if op == Mutation::Shuffle {
            let mut rng_state = self.rng_state;
            let cell = self.resolve(place, &mut indices.iter())?;
            let res = match cell {
                Value::Moved => return trap(span, LENT),
                Value::List(items) => {
                    let v = Rc::make_mut(items);
                    for i in (1..v.len()).rev() {
                        let j = (splitmix64(&mut rng_state) as usize) % (i + 1);
                        v.swap(i, j);
                    }
                    Value::Unit
                }
                _ => return trap(span, "internal: this value can't be shuffled"),
            };
            self.rng_state = rng_state;
            return Ok(res);
        }
        let cell = self.resolve(place, &mut indices.iter())?;
        Ok(match (op, cell) {
            (_, Value::Moved) => return trap(span, LENT),
            (Mutation::Push, Value::List(items)) => {
                Rc::make_mut(items).push(arg);
                Value::Unit
            }
            (Mutation::Take, cell) => {
                let empty = match &*cell {
                    Value::List(_) => Value::List(Rc::default()),
                    Value::Str(_) => Value::Str(Rc::default()),
                    Value::Map(_) => Value::Map(Rc::default()),
                    Value::Set(_) => Value::Set(Rc::default()),
                    // What is left behind in `swap` is written over.
                    other => other.clone(),
                };
                std::mem::replace(cell, empty)
            }
            (Mutation::Push, Value::Str(bytes)) => {
                if let Value::Char(c) = arg {
                    Rc::make_mut(bytes).push(c);
                }
                Value::Unit
            }
            (Mutation::Pop, Value::List(items)) => {
                Value::Opt(OptVal::from_option(Rc::make_mut(items).pop()))
            }
            (Mutation::PopFirst, Value::Set(items)) => {
                Value::Opt(OptVal::from_option(Rc::make_mut(items).pop_first()))
            }
            (Mutation::PopLast, Value::Set(items)) => {
                Value::Opt(OptVal::from_option(Rc::make_mut(items).pop_last()))
            }
            (Mutation::PopFirst | Mutation::PopLast, Value::Map(entries)) => {
                let entries = Rc::make_mut(entries);
                let found = if op == Mutation::PopFirst {
                    entries.pop_first()
                } else {
                    entries.pop_last()
                };
                Value::Opt(OptVal::from_option(
                    found.map(|(k, v)| Value::tuple(vec![k, v])),
                ))
            }
            (Mutation::PopVal, Value::List(items)) => match Rc::make_mut(items).pop() {
                Some(v) => v,
                None => return trap(span, "pop from an empty list"),
            },
            (Mutation::Insert, Value::Set(items)) => Value::Bool(Rc::make_mut(items).insert(arg)),
            (Mutation::Remove, Value::Set(items)) => Value::Bool(Rc::make_mut(items).remove(&arg)),
            (Mutation::Remove, Value::Map(entries)) => {
                Value::Bool(Rc::make_mut(entries).remove(&arg).is_some())
            }
            (Mutation::Clear, Value::List(items)) => {
                Rc::make_mut(items).clear();
                Value::Unit
            }
            (Mutation::Clear, Value::Set(items)) => {
                Rc::make_mut(items).clear();
                Value::Unit
            }
            (Mutation::Clear, Value::Map(entries)) => {
                Rc::make_mut(entries).clear();
                Value::Unit
            }
            (Mutation::Clear, Value::Str(bytes)) => {
                Rc::make_mut(bytes).clear();
                Value::Unit
            }
            (Mutation::Reverse, Value::List(items)) => {
                Rc::make_mut(items).reverse();
                Value::Unit
            }
            (Mutation::Reverse, Value::Array(arr)) => {
                Rc::make_mut(arr).data.reverse();
                Value::Unit
            }
            (Mutation::Swap, Value::List(items)) => {
                let (i, j) = (arg.int(), arg2.int());
                let v = Rc::make_mut(items);
                let len = v.len();
                match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                    (Some(i), Some(j)) if i < len && j < len => v.swap(i, j),
                    _ => {
                        return trap(
                            span,
                            format!("swap({i}, {j}) is outside a list of length {len}"),
                        )
                    }
                }
                Value::Unit
            }
            (Mutation::Swap, Value::Array(arr)) => {
                let (i, j) = (arg.int(), arg2.int());
                let a = Rc::make_mut(arr);
                let len = a.data.len();
                match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                    (Some(i), Some(j)) if i < len && j < len => a.data.swap(i, j),
                    _ => {
                        return trap(
                            span,
                            format!("swap({i}, {j}) is outside an array of length {len}"),
                        )
                    }
                }
                Value::Unit
            }
            (Mutation::Resize, Value::List(items)) => {
                let n = arg.int();
                if n < 0 {
                    return trap(span, format!("resize to negative length {n}"));
                }
                Rc::make_mut(items).resize(n as usize, arg2);
                Value::Unit
            }
            (Mutation::Truncate, Value::List(items)) => {
                let n = arg.int();
                if n < 0 {
                    return trap(span, format!("truncate to negative length {n}"));
                }
                Rc::make_mut(items).truncate(n as usize);
                Value::Unit
            }
            (Mutation::Extend, Value::List(items)) => {
                let v = Rc::make_mut(items);
                match arg {
                    Value::List(other) => v.extend(other.iter().cloned()),
                    Value::Array(arr) => v.extend(arr.data.iter().cloned()),
                    Value::Str(bytes) => v.extend(bytes.iter().copied().map(Value::Char)),
                    Value::Set(set) => v.extend(set.iter().cloned()),
                    _ => {}
                }
                Value::Unit
            }
            (Mutation::Fill, Value::Array(arr)) => {
                let a = Rc::make_mut(arr);
                a.data.fill(arg);
                Value::Unit
            }
            (Mutation::Fill, Value::List(items)) => {
                Rc::make_mut(items).fill(arg);
                Value::Unit
            }
            _ => return trap(span, "internal: this value can't be changed that way"),
        })
    }

    // ----- input -----

    fn read(&mut self, spec: &'p Spec, span: Span) -> Res<Value> {
        // A read that waits shows what was printed first, as the C++
        // does: a judge answers only what it has seen.
        if !self.out.is_empty() && self.input.starved() {
            let _ = self.sink.write_all(&self.out);
            let _ = self.sink.flush();
            self.out.clear();
        }
        let fail = |error: input::InputError| -> Box<Trap> {
            Box::new(Trap {
                message: error.message,
                span,
                call_stack: Vec::new(),
                case: None,
            })
        };
        Ok(match spec {
            Spec::Plain(ty) => match ty {
                Ty::Int => Value::Int(self.input.int().map_err(fail)?.0),
                Ty::I128 => Value::I128(self.input.wide("an `I128`").map_err(fail)?),
                Ty::U64 => Value::U64(self.input.wide("a `U64`").map_err(fail)?),
                Ty::Float => Value::Float(self.input.float().map_err(fail)?),
                Ty::Char => Value::Char(self.input.char().map_err(fail)?),
                // A `Bool` is written `0` or `1`.
                Ty::Bool => match self.input.int().map_err(fail)? {
                    (bit @ (0 | 1), _) => Value::Bool(bit == 1),
                    (other, start) => {
                        let message = format!("{other} is not a `Bool`, which is read as 0 or 1");
                        return Err(fail(self.input.fail(start, message)));
                    }
                },
                Ty::Str => Value::Str(Rc::new(
                    self.input.token("a word").map_err(fail)?.0.to_vec(),
                )),
                Ty::Pos(d) => {
                    let (label, start) = self.input.int().map_err(fail)?;
                    let (lo, len) = self.domains[*d];
                    if label < lo || label - lo >= len {
                        let name = &self.program.domains[*d];
                        let message =
                            format!("{label} is not in `{name}`, which is {}", describe(lo, len));
                        return Err(fail(self.input.fail(start, message)));
                    }
                    Value::Int(label - lo)
                }
                _ => Value::Unit,
            },
            Spec::Int { lo, hi, text } => {
                let (value, start) = self.input.int().map_err(fail)?;
                let lo = match lo {
                    Some(lo) => self.eval(lo)?.int(),
                    None => i64::MIN,
                };
                let hi = match hi {
                    Some(hi) => self.eval(hi)?.int(),
                    None => i64::MAX,
                };
                if value < lo || value > hi {
                    let message = format!("{value} is outside `{text}`");
                    return Err(fail(self.input.fail(start, message)));
                }
                Value::Int(value)
            }
            Spec::Fields(_, fields) => {
                let mut values = Vec::with_capacity(fields.len());
                for field in fields {
                    values.push(self.read(field, span)?);
                }
                Value::tuple(values)
            }
            Spec::Array { dims, elem } => {
                let dims = self.dims(dims, span)?;
                let cells = dims.iter().product::<i64>() as usize;
                let mut data = Vec::with_capacity(cells);
                for _ in 0..cells {
                    data.push(self.read(elem, span)?);
                }
                Value::Array(Rc::new(Array { dims, data }))
            }
            Spec::Opt(inner, Some(absent)) => {
                let absent = self.eval(absent)?.int();
                let (value, start) = self.input.int().map_err(fail)?;
                if value == absent {
                    return Ok(Value::Opt(OptVal::None));
                }
                // The number is already read; the inner type checks it.
                self.input.rewind(start);
                Value::some(self.read(inner, span)?)
            }
            Spec::Opt(..) => Value::Opt(OptVal::None),
        })
    }
}

const NO_GRADER: &str =
    "this fn has no body: the judge's grader supplies it, so there is nothing to run here";
/// The 64 bits of an `Int` or a `U64`.
fn bits64(value: &Value) -> u64 {
    match value {
        Value::U64(bits) => *bits,
        other => other.int() as u64,
    }
}

const LENT: &str = "this variable is being changed by a call that has not returned yet";

fn binop_sym(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Pow => "**",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::Coalesce => "??",
    }
}

/// A domain for a message: `1..=5`, or that it is empty.
fn describe(lo: i64, len: i64) -> String {
    if len == 0 {
        "empty".into()
    } else {
        format!("{lo}..={}", lo + len - 1)
    }
}

/// The cell of an array that `indices` name, checked against each dimension.
fn array_offset<'v>(
    dims: &[i64],
    indices: impl Iterator<Item = &'v Value>,
    span: Span,
) -> Res<usize> {
    let mut at: i64 = 0;
    for (len, index) in dims.iter().zip(indices) {
        let index = index.int();
        if index < 0 || index >= *len {
            return trap(
                span,
                format!("index {index} is outside an array dimension of length {len}"),
            );
        }
        at = at * len + index;
    }
    Ok(at as usize)
}

fn compare(op: CmpOp, l: &Value, r: &Value) -> bool {
    // Floats compare as IEEE says, so a NaN is neither less nor equal.
    if let (Value::Float(a), Value::Float(b)) = (l, r) {
        return match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
            _ => false,
        };
    }
    let order = l.cmp(r);
    match op {
        CmpOp::Eq => order == Ordering::Equal,
        CmpOp::Ne => order != Ordering::Equal,
        CmpOp::Lt => order == Ordering::Less,
        CmpOp::Le => order != Ordering::Greater,
        CmpOp::Gt => order == Ordering::Greater,
        CmpOp::Ge => order != Ordering::Less,
        _ => false,
    }
}

fn binary(op: BinOp, l: Value, r: Value, span: Span) -> Res<Value> {
    Ok(match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            let result = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                // Euclidean: the remainder is never negative, whatever the
                // signs, and the quotient is the one that goes with it.
                BinOp::Div | BinOp::Rem if b == 0 => {
                    return trap(span, format!("division by zero: `{a} {} 0`", binop_sym(op)))
                }
                BinOp::Div => a.checked_div_euclid(b),
                BinOp::Rem => a.checked_rem_euclid(b),
                BinOp::Pow => match u32::try_from(b) {
                    Ok(exp) => a.checked_pow(exp),
                    Err(_) if b < 0 => {
                        return trap(
                            span,
                            format!("`**` needs a power that is not negative, and this is {b}"),
                        )
                    }
                    Err(_) => match a {
                        0 | 1 => Some(a),
                        -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                        _ => None,
                    },
                },
                BinOp::BitAnd => Some(a & b),
                BinOp::BitOr => Some(a | b),
                BinOp::BitXor => Some(a ^ b),
                BinOp::Shl | BinOp::Shr if !(0..64).contains(&b) => {
                    return trap(span, format!("a shift by {b} is outside 0..64"))
                }
                BinOp::Shl => {
                    if a < 0 {
                        return trap(span, format!("`<<` of the negative number {a}"));
                    }
                    Some(a << b).filter(|shifted| *shifted >= 0 && shifted >> b == a)
                }
                BinOp::Shr => Some(a >> b),
                BinOp::And | BinOp::Or | BinOp::Coalesce => None,
            };
            match result {
                Some(value) => Value::Int(value),
                None => {
                    return trap(
                        span,
                        format!(
                            "integer overflow: `{a} {} {b}` does not fit in an `Int` (64 bits)",
                            binop_sym(op)
                        ),
                    )
                }
            }
        }
        (Value::I128(a), Value::I128(b)) => {
            let result = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                BinOp::Div | BinOp::Rem if b == 0 => {
                    return trap(span, format!("division by zero: `{a} {} 0`", binop_sym(op)))
                }
                BinOp::Div => a.checked_div_euclid(b),
                BinOp::Rem => a.checked_rem_euclid(b),
                BinOp::Pow => match u32::try_from(b) {
                    Ok(exp) => a.checked_pow(exp),
                    Err(_) if b < 0 => {
                        return trap(span, format!("`**` needs a power that is not negative, and this is {b}"))
                    }
                    Err(_) => match a {
                        0 | 1 => Some(a),
                        -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                        _ => None,
                    },
                },
                BinOp::BitAnd => Some(a & b),
                BinOp::BitOr => Some(a | b),
                BinOp::BitXor => Some(a ^ b),
                _ => None,
            };
            match result {
                Some(value) => Value::I128(value),
                None => {
                    return trap(
                        span,
                        format!(
                            "integer overflow: `{a} {} {b}` does not fit in an `I128` (128 bits)",
                            binop_sym(op)
                        ),
                    )
                }
            }
        }
        (Value::I128(a), Value::Int(b)) => {
            if !(0..128).contains(&b) {
                return trap(span, format!("a shift by {b} is outside 0..128"));
            }
            match op {
                BinOp::Shl if a < 0 => return trap(span, format!("`<<` of the negative number {a}")),
                BinOp::Shl => match Some(a << b).filter(|shifted| *shifted >= 0 && shifted >> b == a) {
                    Some(value) => Value::I128(value),
                    None => {
                        return trap(
                            span,
                            format!("integer overflow: `{a} << {b}` does not fit in an `I128` (128 bits)"),
                        )
                    }
                },
                _ => Value::I128(a >> b),
            }
        }
        // A `U64` is the 64 bits: what does not fit is dropped.
        (Value::U64(a), Value::U64(b)) => Value::U64(match op {
            BinOp::Add => a.wrapping_add(b),
            BinOp::Sub => a.wrapping_sub(b),
            BinOp::Mul => a.wrapping_mul(b),
            BinOp::Div | BinOp::Rem if b == 0 => {
                return trap(span, format!("division by zero: `{a} {} 0`", binop_sym(op)))
            }
            BinOp::Div => a / b,
            BinOp::Rem => a % b,
            BinOp::Pow => {
                let (mut base, mut exp, mut result) = (a, b, 1u64);
                while exp > 0 {
                    if exp & 1 == 1 {
                        result = result.wrapping_mul(base);
                    }
                    base = base.wrapping_mul(base);
                    exp >>= 1;
                }
                result
            }
            BinOp::BitAnd => a & b,
            BinOp::BitOr => a | b,
            BinOp::BitXor => a ^ b,
            _ => a,
        }),
        (Value::U64(a), Value::Int(b)) => {
            if !(0..64).contains(&b) {
                return trap(span, format!("a shift by {b} is outside 0..64"));
            }
            Value::U64(if op == BinOp::Shl { a << b } else { a >> b })
        }
        (Value::Float(a), Value::Float(b)) => Value::Float(match op {
            BinOp::Add => a + b,
            BinOp::Sub => a - b,
            BinOp::Mul => a * b,
            BinOp::Div => a / b,
            BinOp::Pow => a.powf(b),
            _ => return trap(span, "internal: this operator has no float form"),
        }),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(match op {
            BinOp::BitAnd => a & b,
            BinOp::BitOr => a | b,
            _ => a ^ b,
        }),
        (Value::Str(mut a), Value::Str(b)) => {
            Rc::make_mut(&mut a).extend_from_slice(&b);
            Value::Str(a)
        }
        _ => return trap(span, "internal: operands of different kinds"),
    })
}

/// `old op= new`.
fn combine(op: AssignOp, old: Value, new: Value, span: Span) -> Res<Value> {
    Ok(match op {
        AssignOp::Set => new,
        AssignOp::Max => {
            if new > old {
                new
            } else {
                old
            }
        }
        AssignOp::Min => {
            if new < old {
                new
            } else {
                old
            }
        }
        AssignOp::Binary(op) => binary(op, old, new, span)?,
    })
}

#[cfg(test)]
mod tests;
