//! Debug interpreter: runs a lowered program directly.
//!
//! This is `cpl run`. It walks the same [`cpl_ir::Program`] that the C++
//! back end prints, and it checks what the submission build doesn't:
//! indices, input ranges, integer overflow, `!` on `None`, and a variable
//! used while a call is still changing it. A failed check is a [`Trap`]
//! with the source span, never a crash of the interpreter.
//!
//! Strings are byte vectors. Nothing here decodes or validates text.

mod apply;
mod bounded_read;
mod builtin;
pub mod checkpoint;
mod closed_form;
mod float_fn;
mod fn_value;
mod input;
mod io;
mod mutation;
mod read;
mod show_hook;
use io::draw_below;
use io::Feed;
pub use io::{Choices, Io};
mod observe;
pub mod profile;
pub mod replay;
mod stop;
mod threads;
pub mod trace;
mod trap_help;
mod unwind;
mod value;
mod vm;

use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtArg, FmtPiece,
    FnId, Iter, LoopId, Mutation, Place, PlaceKind, Program, SortBy, Spec, Stmt, Ty, UnOp,
};
use input::Input;
pub use input::More;
use observe::Observer;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
pub use stop::{Limits, Stop, CANCELLED, OUT_OF_TIME};
pub use threads::{run_for, run_on_this_thread, run_with_on_this_thread};
use value::{
    show, zero, Array, CaptureVal, ListViewVal, MutViewVal, OptVal, Part, Step, StrViewVal,
    TupleVal, Value,
};

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
    /// Where the failure is written when that is not the user's line.
    pub notes: Vec<String>,
}

impl Trap {
    /// The catalogued `R` code (`spec/diagnostics.md`) for this trap's kind.
    pub fn code(&self) -> cpl_diag::Code {
        use cpl_diag::codes;
        let m = self.message.as_str();
        let m = m.strip_prefix("reading the number of tests: ").unwrap_or(m);
        if m.contains("input line")
            && (m.contains(" is outside `") || m.contains("does not fit in"))
        {
            codes::INPUT_OUT_OF_CONTRACT
        } else if m.contains("integer overflow") || m.contains("does not fit in") {
            codes::OVERFLOW
        } else if m.starts_with("division by zero") {
            codes::DIVISION_BY_ZERO
        } else if m.starts_with("`!` on")
            || m.contains("unwrapped a `None`")
            || m.contains("there is no value here")
        {
            codes::UNWRAP_NONE
        } else if m.starts_with("assertion failed") {
            codes::ASSERTION_FAILED
        } else if m.starts_with("the stack ran out") {
            codes::STACK_EXHAUSTED
        } else if m.contains(" is outside `") {
            codes::RANGE_STORE
        } else if m.starts_with("expected a number, found")
            || m.starts_with("the input ended")
            || m.starts_with("expected a `")
        {
            codes::BAD_INPUT_TOKEN
        } else if m.starts_with("`todo` reached") {
            codes::TODO_REACHED
        } else if m.contains(" is not in `") {
            codes::DOMAIN_CAST
        } else if ((m.starts_with("index ") || m.starts_with("swap(")) && m.contains("outside"))
            || (m.starts_with("key ") && m.contains("is not one of the view's keys"))
            || (m.starts_with("slice ") && m.contains(" is outside "))
        {
            codes::INDEX_OUT_OF_RANGE
        } else {
            codes::RUNTIME
        }
    }

    /// Builds the runtime diagnostic with test-case and call-stack notes.
    pub fn to_diagnostic(&self, path: &str, lines: &cpl_diag::LineIndex) -> cpl_diag::Diagnostic {
        let mut diagnostic =
            cpl_diag::Diagnostic::error(self.code(), self.span, self.message.clone());
        if let Some(help) = trap_help::help(&self.message) {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic = self
            .notes
            .iter()
            .cloned()
            .fold(diagnostic, cpl_diag::Diagnostic::with_note);
        if let Some(case) = self.case {
            diagnostic = diagnostic.with_note(format!("in test case {case}"));
        }
        if !self.call_stack.is_empty() {
            let mut parts = Vec::with_capacity(self.call_stack.len() + 1);
            let mut current_line = lines.line_of(self.span.start) + 1;
            for frame in &self.call_stack {
                if frame.call_span == Span::default() && frame.name.starts_with("...") {
                    let n = frame.name.split(' ').nth(1).and_then(|n| n.parse().ok());
                    parts.push((None, n.unwrap_or(0)));
                    continue;
                }
                parts.push((Some(format!("{} ({path}:{current_line})", frame.name)), 1));
                current_line = lines.line_of(frame.call_span.start) + 1;
            }
            parts.push((Some(format!("top level ({path}:{current_line})")), 1));
            let parts = unwind::runs(&parts);
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
    let output = std::sync::Mutex::new(output);
    on_interp_thread(&|limits| {
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
    })
}

/// `run` on a thread with the interpreter's deep stack. What it needs is
/// shared by reference, so that a spawn that fails (too little address
/// space for the stack) can be tried again smaller.
pub(crate) fn on_interp_thread(run: &(dyn Fn(Limits) -> Observed + Sync)) -> Observed {
    let failed = |message: &str| Observed {
        trap: Some(Trap {
            message: message.into(),
            span: Span::default(),
            call_stack: Vec::new(),
            case: None,
            notes: Vec::new(),
        }),
        ..Observed::default()
    };
    let memory = memory_for_stack();
    std::thread::scope(|scope| {
        for bytes in [STACK_BYTES, 3 << 30, 1 << 28, 1 << 23] {
            let limits = Limits {
                // The rest is left for the frames under the deepest call.
                stack_bytes: (bytes - (bytes / 64).max(1 << 20)).min(memory),
            };
            let spawned = std::thread::Builder::new()
                .stack_size(bytes)
                .spawn_scoped(scope, move || run(limits));
            if let Ok(handle) = spawned {
                return handle.join().unwrap_or_else(|_| {
                    failed("the interpreter itself failed; this is a bug in cpl")
                });
            }
        }
        failed("could not start the interpreter thread")
    })
}

/// Starts `job` on a long-lived thread with the interpreter's deep stack,
/// trying smaller stacks as [`on_interp_thread`] does; whether one started.
pub(crate) fn spawn_interp_thread(job: std::sync::Arc<dyn Fn(Limits) + Send + Sync>) -> bool {
    let memory = memory_for_stack();
    [STACK_BYTES, 3 << 30, 1 << 28, 1 << 23]
        .into_iter()
        .any(|bytes| {
            let limits = Limits {
                stack_bytes: (bytes - (bytes / 64).max(1 << 20)).min(memory),
            };
            let job = job.clone();
            std::thread::Builder::new()
                .stack_size(bytes)
                .spawn(move || job(limits))
                .is_ok()
        })
}

/// [`run_on_this_thread`], keeping what [`run_observed`] keeps.
pub fn observe_on_this_thread(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
) -> Observed {
    run_io(program, io, output, limits, trace, false)
}

fn run_io(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
    profile: bool,
) -> Observed {
    match trace {
        Some(config) => {
            let tracer = Box::new(trace::Tracer::new(config, program));
            run_as(program, io, output, limits, Some(tracer), profile)
        }
        None => run_as::<observe::Untraced>(program, io, output, limits, None, profile),
    }
}

/// [`run_io`] with `tracer` watching: a separate build of the interpreter
/// for each kind of observer, so an unobserved run carries no hooks.
fn run_as<O: Observer>(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    tracer: Option<Box<O>>,
    profile: bool,
) -> Observed {
    let off = checkpoint::Clock::default();
    run_from(program, io, output, limits, tracer, profile, off).0
}

/// [`run_as`], taking checkpoints or starting from one as `ckpt` says.
fn run_from<O: Observer>(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    tracer: Option<Box<O>>,
    profile: bool,
    ckpt: checkpoint::Clock,
) -> (Observed, checkpoint::Clock) {
    let top = 0u8;
    let seed = io.seed;
    let last_values = io.last_values;
    let mut interp = Interp {
        program,
        globals: vec![Value::Unit; program.globals.len()],
        domains: vec![(0, 0); program.domains.len()],
        saved_domains: Vec::new(),
        fn_guarded: program
            .fns
            .iter()
            .map(|f| program.fn_guarded_domains(&f.ret, &f.body))
            .collect(),
        closure_guarded: program
            .closures
            .iter()
            .map(|c| program.fn_guarded_domains(&c.ret, &c.body))
            .collect(),
        fast_closures: program.closures.iter().map(classify_fast_closure).collect(),
        stack: Vec::new(),
        base: 0,
        input: match io.input {
            Feed::Fixed(input) => Input::new(input),
            Feed::Live(more) => Input::live(more),
        },
        peer: io
            .peer
            .map(|more| Input::live(more).named("solution's output")),
        queries: 0,
        out: Vec::with_capacity(1 << 16),
        sink: output,
        stack_top: std::ptr::addr_of!(top) as usize,
        stack_budget: limits.stack_bytes,
        frame_locals: Vec::new(),
        call_frames: Vec::new(),
        current_case: None,
        rng_state: seed.map_or(0x9e3779b97f4a7c15, |seed| seed ^ 0x9e3779b97f4a7c15),
        choices: io.choices,
        tracer,
        stop: io.stop,
        countdown: stop::EVERY,
        ckpt,
        profiler: profile.then(|| Box::new(profile::Profiler::new(program))),
        debug_out: Vec::new(),
        feeds: cpl_ir::pushed::Feeds::of(program),
        fed: HashMap::new(),
        fed_loops: Vec::new(),
        in_closure: false,
        fed_left: None,
        vm: None,
        vm_regs_i: Vec::new(),
        fed_bodies: 0,
        fed_hosts: Vec::new(),
        lent: Vec::new(),
        frameless_base: None,
        vm_pool_v: Vec::new(),
        view_locals: Vec::new(),
        trace_shown: trace::ShowTexts::new(),
        trace_show_tys: HashMap::new(),
        trap_frame: None,
        trace_indices: Vec::new(),
        trace_path: Vec::new(),
        trace_raw: Vec::new(),
        vm_stats: vm::VmStats::from_env(),
    };
    let result = match interp.main() {
        Err(trap) if trap.message == trace::STOPPED => Ok(()),
        result => result,
    };
    if let Some(stats) = &interp.vm_stats {
        stats.emit_stderr();
    }
    let _ = interp.sink.write_all(&interp.out);
    let _ = interp.sink.flush();
    let trap = result.err().map(|mut trap| {
        if trap.call_stack.is_empty() && !interp.call_frames.is_empty() {
            trap.call_stack = unwind::call_stack(interp.program, &interp.call_frames);
            trap.span = unwind::user_span(interp.program, &interp.call_frames, trap.span);
        }
        if trap.case.is_none() {
            trap.case = interp.current_case;
        }
        *trap
    });
    let recording = interp.tracer.take().and_then(|tracer| {
        let failed = trap.as_ref().map(|trap| (trap.message.as_str(), trap.span));
        let depth = interp.call_frames.len();
        let kept = (interp.trap_frame.take()).filter(|kept| failed.is_some() && kept.0 == depth);
        interp.fill_shown();
        let mut view = interp.view();
        if let Some((_, locals)) = &kept {
            view.locals = locals
                .iter()
                .map(|(name, ty, value)| (*name, *ty, value))
                .collect();
        }
        tracer.finish(&view, failed)
    });
    let recording = recording.or_else(|| Some(trace::last_values_of(&interp.view(), last_values?)));
    let observed = Observed {
        trap,
        debug: std::mem::take(&mut interp.debug_out),
        recording,
        profile: interp.profiler.take().map(|profiler| profiler.finish()),
    };
    if interp.ckpt.on() {
        interp.ckpt.read = interp.input.bytes().to_vec();
    }
    (observed, interp.ckpt)
}

type Res<T> = Result<T, Box<Trap>>;

/// A whole float as an `Int`: strictly between `-2^63` and `2^63`.
fn float_int(v: f64, span: Span) -> Res<Value> {
    if !(v > -9223372036854775808.0 && v < 9223372036854775808.0) {
        return trap(span, format!("{v} does not fit in an `Int`"));
    }
    Ok(Value::Int(v as i64))
}

/// What a failed `assert`, its `message` evaluated, or a `Trap` reports.
fn stmt_failure(stmt: &Stmt, message: Option<Value>) -> Box<Trap> {
    match stmt {
        Stmt::Assert {
            text, note, span, ..
        } => {
            let mut report = cpl_ir::assert_head(text);
            if let Some(Value::Str(bytes)) = &message {
                report.push_str(if text.is_empty() { "" } else { ": " });
                report.extend(bytes.iter().map(|b| *b as char));
            }
            let mut failed = trap::<()>(*span, report).unwrap_err();
            failed.notes.extend(note.clone());
            failed
        }
        Stmt::Trap(message, span) => trap::<()>(*span, message.clone()).unwrap_err(),
        _ => unreachable!("only an assert or a trap fails"),
    }
}

fn trap<T>(span: Span, message: impl Into<String>) -> Res<T> {
    Err(Box::new(Trap {
        message: message.into(),
        span,
        call_stack: Vec::new(),
        case: None,
        notes: Vec::new(),
    }))
}

enum Flow {
    Next,
    Break(Option<LoopId>),
    Continue(Option<LoopId>),
    Return(Value),
    Give(Value),
}

#[derive(Clone)]
enum FastClosure {
    MinMax {
        is_max: bool,
        a: usize,
        b: usize,
        ty: Ty,
    },
    Bin {
        op: BinOp,
        a: usize,
        b: usize,
    },
    IntAddMul {
        x: usize,
        f: usize,
        w: usize,
    },
    IntCmpConst {
        op: CmpOp,
        a: usize,
        c: i64,
    },
    Identity(usize),
}

fn classify_fast_closure(def: &cpl_ir::ClosureDef) -> Option<FastClosure> {
    if !def.captures.is_empty() || !def.muts.is_empty() || def.body.stmts.len() != 1 {
        return None;
    }
    let expr = match &def.body.stmts[0] {
        Stmt::Return(Some(e)) | Stmt::Give(e) => e,
        _ => return None,
    };
    let param_idx = |e: &Expr| -> Option<usize> {
        let ExprKind::Local(slot) = e.kind else {
            return None;
        };
        def.params.iter().position(|&p| p == slot)
    };
    match &expr.kind {
        ExprKind::Builtin(op @ (Builtin::Min | Builtin::Max), args) if args.len() == 2 => {
            let a = param_idx(&args[0])?;
            let b = param_idx(&args[1])?;
            Some(FastClosure::MinMax {
                is_max: matches!(op, Builtin::Max),
                a,
                b,
                ty: expr.ty.clone(),
            })
        }
        ExprKind::Binary(
            op @ (BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXor),
            l,
            r,
        ) => {
            if let (Some(a), Some(b)) = (param_idx(l), param_idx(r)) {
                return Some(FastClosure::Bin { op: *op, a, b });
            }
            if matches!(op, BinOp::Add) && expr.ty == Ty::Int {
                if let (Some(x), ExprKind::Binary(BinOp::Mul, fl, fr)) = (param_idx(l), &r.kind) {
                    if let (Some(f), Some(w)) = (param_idx(fl), param_idx(fr)) {
                        return Some(FastClosure::IntAddMul { x, f, w });
                    }
                }
                if let (ExprKind::Binary(BinOp::Mul, fl, fr), Some(x)) = (&l.kind, param_idx(r)) {
                    if let (Some(f), Some(w)) = (param_idx(fl), param_idx(fr)) {
                        return Some(FastClosure::IntAddMul { x, f, w });
                    }
                }
            }
            None
        }
        ExprKind::Cmp(op, l, r) => {
            if let (Some(a), ExprKind::Int(c)) = (param_idx(l), &r.kind) {
                if l.ty == Ty::Int {
                    return Some(FastClosure::IntCmpConst { op: *op, a, c: *c });
                }
            }
            None
        }
        ExprKind::Local(_) => param_idx(expr).map(FastClosure::Identity),
        _ => None,
    }
}

struct Interp<'p, 'o, O: Observer> {
    program: &'p Program,
    globals: Vec<Value>,
    /// First label and size of each domain, once it is declared.
    domains: Vec<(i64, i64)>,
    /// Saved `(domain_id, (lo, len))` entries for recursive/block-scoped restore.
    saved_domains: Vec<(cpl_ir::DomainId, (i64, i64))>,
    /// Precomputed guarded domains per fn / closure (avoids walking IR + HashSet allocs per call).
    fn_guarded: Vec<Vec<cpl_ir::DomainId>>,
    closure_guarded: Vec<Vec<cpl_ir::DomainId>>,
    fast_closures: Vec<Option<FastClosure>>,
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
    /// The choices `rand` and `shuffle` write down or replay.
    choices: Option<std::sync::Arc<std::sync::Mutex<Choices>>>,
    /// Records the trace of the run, when one was asked for.
    tracer: Option<Box<O>>,
    /// What ends the run early, looked at when `countdown` reaches 0.
    stop: Option<Stop>,
    countdown: u32,
    ckpt: checkpoint::Clock,
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
    /// How the fed loop that just ended was left, if not by a plain `break`.
    fed_left: Option<Flow>,
    /// Compiled register bytecode program for plain (untraced, unprofiled) runs.
    vm: Option<Rc<vm::CompiledProgram<'p>>>,
    /// Shared unboxed `i64` register stack for the VM.
    vm_regs_i: Vec<i64>,
    /// Fed loop bodies running, which take no checkpoint.
    fed_bodies: u32,
    /// Traced VM frames, by base, running a fallback that may feed a loop their chunk holds.
    fed_hosts: Vec<(usize, *const vm::Chunk<'p>)>,
    /// Lent params of running VM calls: the stack slot holding each value, and its place.
    lent: Vec<(usize, Rc<Part>)>,
    /// The base of the frame whose VM callee runs without a frame of its own.
    frameless_base: Option<usize>,
    /// Reusable pool of `Value` register frames for VM calls.
    vm_pool_v: Vec<Vec<Value>>,
    /// The allocation [`Interp::view`] lists locals in, kept between steps.
    view_locals: Vec<(&'static str, &'static Ty, &'static Value)>,
    /// The text of each value a trace shows through its `impl Show`.
    trace_shown: trace::ShowTexts,
    /// Whether a type holds a value with an `impl Show`, per type seen.
    trace_show_tys: HashMap<Ty, bool>,
    /// The innermost call's locals when a traced run trapped, and its depth.
    trap_frame: Option<unwind::TrapFrame<'p>>,
    /// The indices of the place a trace point writes, kept between steps.
    trace_indices: Vec<Value>,
    trace_path: Vec<usize>,
    /// The named `Int` locals a raw trace step shows, kept between steps.
    trace_raw: Vec<(&'p str, i64)>,
    /// Optional fallback counter enabled via `CPL_VM_STATS`.
    vm_stats: Option<vm::VmStats>,
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
pub(crate) fn root_global(place: &Place) -> Option<usize> {
    match &place.kind {
        PlaceKind::Global(id) => Some(*id),
        PlaceKind::Local(_) => None,
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            root_global(base)
        }
    }
}

/// The local a place is in, when it is in one.
fn root_local(place: &Place) -> Option<usize> {
    match &place.kind {
        PlaceKind::Local(slot) => Some(*slot),
        PlaceKind::Global(_) => None,
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            root_local(base)
        }
    }
}

/// How many indices of `place` lead straight from its variable: `2` in
/// `dp[i][j]` and in `dp[i, j]`, `1` in `a[i].x`.
/// Empties `locals` and gives back its allocation for borrows of another
/// lifetime; collecting in place keeps the buffer.
fn relifetime<'a, 'b>(
    mut locals: Vec<(&'a str, &'a Ty, &'a Value)>,
) -> Vec<(&'b str, &'b Ty, &'b Value)> {
    locals.clear();
    locals.into_iter().map(|_| unreachable!()).collect()
}

fn leading_indices(place: &Place) -> usize {
    match &place.kind {
        PlaceKind::Global(_) | PlaceKind::Local(_) => 0,
        PlaceKind::Index(base, indices) => match &base.kind {
            PlaceKind::Global(_) | PlaceKind::Index(..)
                if leading_indices(base) == index_count(base) =>
            {
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

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    fn main(&mut self) -> Res<()> {
        let program = self.program;
        self.stack.clear();
        self.stack.resize(program.main_locals.len(), Value::Unit);
        self.base = 0;
        if let Some(from) = self.ckpt.resume.take() {
            return self.resume(from);
        }
        {
            let mut has_mut_view = program
                .main_locals
                .iter()
                .any(|l| matches!(l.ty, Ty::MutView(..)))
                || program
                    .fns
                    .iter()
                    .any(|f| f.locals.iter().any(|l| matches!(l.ty, Ty::MutView(..))))
                || program
                    .closures
                    .iter()
                    .any(|c| c.locals.iter().any(|l| matches!(l.ty, Ty::MutView(..))));
            if !has_mut_view {
                let mut check = |e: &cpl_ir::Expr| {
                    if matches!(e.kind, ExprKind::MutSlice(..)) || matches!(e.ty, Ty::MutView(..)) {
                        has_mut_view = true;
                    }
                };
                cpl_ir::walk::deep_exprs(&program.main, &mut check);
                for f in &program.fns {
                    cpl_ir::walk::deep_exprs(&f.body, &mut check);
                }
                for c in &program.closures {
                    cpl_ir::walk::deep_exprs(&c.body, &mut check);
                }
            }
            // `CPL_TRACE_ENGINE=tree` traces on the tree-walker;
            // trace_goldens holds both engines to the saved traces.
            let tree = self.observed()
                && std::env::var_os("CPL_TRACE_ENGINE").is_some_and(|e| e == "tree");
            if self.profiler.is_none() && !has_mut_view && !tree {
                let traced = self.observed();
                self.vm = Some(Rc::new(vm::CompiledProgram::compile(
                    program,
                    &self.feeds,
                    traced,
                )));
            }
        }
        if !program.multitest {
            if let Some(compiled) = self.vm.clone() {
                self.run_main_vm(&compiled)?;
                return Ok(());
            }
            self.block(&program.main)?;
            return Ok(());
        }
        // Above `@!multitest`: once. A top-level `return` there ends the
        // program.
        if let Some(tracer) = &mut self.tracer {
            tracer.begin_cases();
        }
        let returned = match self.vm.clone() {
            Some(compiled) => match &compiled.setup {
                Some(setup) => self.run_setup_vm(&compiled, setup)?,
                None => unreachable!("a multitest program compiles its setup"),
            },
            None => matches!(self.block(&program.setup)?, Flow::Return(_)),
        };
        if returned {
            return Ok(());
        }
        if let Some(tracer) = &mut self.tracer {
            tracer.divider();
        }
        let left = if program.until_eof {
            None
        } else {
            match self.input.int() {
                // The judge's side of an interactive problem tells the
                // solution how many test cases there are.
                Ok((count, _)) if program.announce_tests => {
                    self.out.extend_from_slice(format!("{count}\n").as_bytes());
                    self.write_out(true);
                    Some(count)
                }
                Ok((count, _)) => Some(count),
                Err(error) => {
                    return trap(
                        program.multitest_span,
                        format!("reading the number of tests: {}", error.message),
                    )
                }
            }
        };
        self.run_cases(left, 0)
    }

    /// The test cases after the first `case`, while `left` (or the input) has more.
    pub(crate) fn run_cases(&mut self, mut left: Option<i64>, mut case: u64) -> Res<()> {
        let program = self.program;
        loop {
            match &mut left {
                Some(left) if *left <= 0 => break,
                Some(left) => *left -= 1,
                None if self.input.at_eof() => break,
                None => {}
            }
            case += 1;
            self.ckpt.cases_left = left;
            self.current_case = Some(case);
            // `ask(.., limit: K)` allows `K` queries in each test case.
            self.queries = 0;
            if let Some(tracer) = &mut self.tracer {
                tracer.start_case(case);
            }
            self.stack.clear();
            self.stack.resize(program.main_locals.len(), Value::Unit);
            if let Some(compiled) = self.vm.clone() {
                self.run_main_vm(&compiled)?;
                continue;
            }
            self.block(&program.main)?;
        }
        Ok(())
    }

    /// What the tracer sees of the program's state right now.
    fn view(&self) -> trace::View<'_> {
        self.view_in(Vec::new())
    }

    fn view_in<'s>(&'s self, mut locals: Vec<(&'s str, &'s Ty, &'s Value)>) -> trace::View<'s> {
        // The map shortens the locals' `'p` to `'s`.
        #[allow(clippy::map_identity)]
        locals.extend(
            self.named_locals()
                .map(|(name, ty, value)| (name, ty, value)),
        );
        trace::View {
            program: self.program,
            globals: &self.globals,
            domains: &self.domains,
            locals,
            raw: None,
            depth: self.call_frames.len(),
            call_frames: &self.call_frames,
            shown: &self.trace_shown,
        }
    }

    /// The current call's named locals and their values.
    fn named_locals(&self) -> impl Iterator<Item = (&'p str, &'p Ty, &Value)> + use<'_, 'p, 'o, O> {
        self.locals()
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
    }

    fn flush_if_large(&mut self) {
        if self.out.len() >= 1 << 16 {
            self.write_out(false);
        }
    }

    /// Sends what was printed on, and with `flush` makes it go out now; a
    /// checkpoint counts it as printed.
    fn write_out(&mut self, flush: bool) {
        self.ckpt.printed += self.out.len() as u64;
        let _ = self.sink.write_all(&self.out);
        if flush {
            let _ = self.sink.flush();
        }
        self.out.clear();
    }

    // ----- statements -----

    fn block(&mut self, block: &'p Block) -> Res<Flow> {
        self.tick(Span::default())?;
        for stmt in &block.stmts {
            match self.stmt(stmt)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, stmt: &'p Stmt) -> Res<Flow> {
        if let Some(stats) = &mut self.vm_stats {
            stats.walked += 1;
            *stats.walked_counts.entry(vm::stmt_tag(stmt)).or_default() += 1;
        }
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
                if let Some(init) = init {
                    self.trace_local(*slot, init.span);
                    self.trace_assigned_local(*slot, init.span);
                }
            }
            Stmt::Assign(place, op, value) => {
                if !self.observed() {
                    self.assign_untraced(place, *op, value)?;
                } else {
                    let indices = self.assign(place, *op, value)?;
                    self.trace_change(place, &indices);
                    self.trace_assigned(place);
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
            Stmt::Assert { cond, message, .. } => {
                if !self.eval(cond)?.bool() {
                    let message = match message {
                        Some(message) => Some(self.eval(message)?),
                        None => None,
                    };
                    return Err(stmt_failure(stmt, message));
                }
            }
            Stmt::DefDomain { id, lo, len } => {
                let lo = self.eval(lo)?.int();
                let len = self.eval(len)?.int().max(0);
                self.domains[*id] = (lo, len);
            }
            Stmt::DefDomainFromVal { id, val } => {
                if let Value::DomainVal(lo, len) = self.eval(val)? {
                    self.domains[*id] = (lo, len.max(0));
                }
            }
            Stmt::Trap(..) => return Err(stmt_failure(stmt, None)),
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
        let span = match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range { lo: e, .. }
            | Iter::RangeVal(e, _)
            | Iter::Seq(e, _)
            | Iter::TreeRange { container: e, .. } => e.span,
        };
        macro_rules! round {
            ($value:expr) => {{
                self.stack[base + slot] = $value;
                self.trace_local(slot, span);
                self.trace_assigned_local(slot, span);
                match self.block(body)? {
                    Flow::Next | Flow::Continue(None) => {}
                    Flow::Break(None) => break,
                    Flow::Break(Some(target)) if target == id => break,
                    Flow::Continue(Some(target)) if target == id => {}
                    other => return Ok(other),
                }
            }};
        }
        let is_enum_slot =
            matches!(self.locals().get(slot), Some(l) if matches!(l.ty, Ty::Enum(_)));
        let wrap = |v: i64| {
            if is_enum_slot {
                Value::Enum(v as u32, Rc::new(Vec::new()))
            } else {
                Value::Int(v)
            }
        };
        match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range {
                lo,
                hi,
                step,
                rev,
                closed,
            } => {
                let (lo_i64, hi_i64) = match self.ckpt.range_from.take() {
                    Some(from) => from,
                    None => (self.eval(lo)?.int(), self.eval(hi)?.int()),
                };
                if step.is_none() {
                    if *closed {
                        if lo_i64 <= hi_i64 {
                            if *rev {
                                let mut v = hi_i64;
                                loop {
                                    round!(wrap(v));
                                    if v == lo_i64 {
                                        break;
                                    }
                                    v -= 1;
                                }
                            } else {
                                let mut v = lo_i64;
                                loop {
                                    round!(wrap(v));
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
                                round!(wrap(v));
                                if v == lo_i64 {
                                    break;
                                }
                                v -= 1;
                            }
                        } else {
                            let mut v = lo_i64;
                            while v < hi_i64 {
                                round!(wrap(v));
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
            Iter::RangeVal(expr, rev) => match self.eval(expr)? {
                Value::Range(lo, hi) if lo < hi => {
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
                Value::Range128(lo, hi) if lo < hi => {
                    if *rev {
                        let mut v = hi - 1;
                        while v >= lo {
                            round!(Value::I128(v));
                            if v == lo {
                                break;
                            }
                            v -= 1;
                        }
                    } else {
                        let mut v = lo;
                        while v < hi {
                            round!(Value::I128(v));
                            v += 1;
                        }
                    }
                }
                _ => {}
            },
            Iter::TreeRange {
                container,
                lo,
                hi,
                rev,
            } => {
                use std::ops::Bound;
                let held = self.eval(container)?;
                let lo_b = match lo {
                    Some((e, true)) => Bound::Included(self.eval(e)?),
                    Some((e, false)) => Bound::Excluded(self.eval(e)?),
                    None => Bound::Unbounded,
                };
                let hi_b = match hi {
                    Some((e, true)) => Bound::Included(self.eval(e)?),
                    Some((e, false)) => Bound::Excluded(self.eval(e)?),
                    None => Bound::Unbounded,
                };
                let empty = match (&lo_b, &hi_b) {
                    (Bound::Included(a), Bound::Included(b)) => a > b,
                    (Bound::Included(a), Bound::Excluded(b))
                    | (Bound::Excluded(a), Bound::Included(b))
                    | (Bound::Excluded(a), Bound::Excluded(b)) => a >= b,
                    _ => false,
                };
                if !empty {
                    match &held {
                        Value::Set(items) => {
                            let range = items.range((lo_b, hi_b));
                            if *rev {
                                for item in range.rev() {
                                    round!(item.clone());
                                }
                            } else {
                                for item in range {
                                    round!(item.clone());
                                }
                            }
                        }
                        Value::Map(entries) => {
                            let range = entries.range((lo_b, hi_b));
                            if *rev {
                                for (k, v) in range.rev() {
                                    round!(Value::tuple(vec![k.clone(), v.clone()]));
                                }
                            } else {
                                for (k, v) in range {
                                    round!(Value::tuple(vec![k.clone(), v.clone()]));
                                }
                            }
                        }
                        _ => unreachable!("TreeRange only lowers on Set/Map"),
                    }
                }
            }
            Iter::Seq(seq, false) if self.is_fed(id, seq, body).is_some() => {
                let (ExprKind::Call(callee, args), Some((_, acc))) =
                    (&seq.kind, self.is_fed(id, seq, body))
                else {
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
                if !(self.ckpt.waits() && self.resume_fed(seq, &fed)?) {
                    self.call_feeding(*callee, args, seq.span, Some(fed))?;
                }
                if let Some(flow) = self.fed_left.take() {
                    return Ok(flow);
                }
            }
            Iter::Seq(seq, rev) => {
                // The loop walks the value as it was when the loop began.
                let (held, from) = self.seq_held(seq)?;
                match &held {
                    Value::List(items) => {
                        let n = items.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(items[k].clone());
                        }
                    }
                    Value::ListView(view) => {
                        let n = view.len;
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(view.get(k).unwrap().clone());
                        }
                    }
                    Value::MutView(mv) => {
                        let items = self.mut_view_to_vec(mv, span)?;
                        let n = items.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(items[k].clone());
                        }
                    }
                    Value::Array(array) => {
                        let n = array.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(array.get_unchecked_val(k));
                        }
                    }
                    Value::Str(bytes) => {
                        let n = bytes.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(Value::Char(bytes[k]));
                        }
                    }
                    Value::StrView(view) => {
                        let n = view.len;
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(Value::Char(view.get(k).unwrap()));
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
                if matches!(base.ty, Ty::MutView(..)) {
                    let mv = match self.resolve(base, indices)? {
                        Value::MutView(mv) => Rc::clone(mv),
                        Value::Moved => return trap(span, LENT),
                        _ => return trap(span, "internal: expected mutable view"),
                    };
                    let index = indices.next().map_or(0, Value::int);
                    let Some(i) = usize::try_from(index).ok().filter(|i| *i < mv.len) else {
                        return trap(
                            span,
                            format!("index {index} is outside a list of length {}", mv.len),
                        );
                    };
                    let target = self.part_mut(&mv.target, span)?;
                    return match target {
                        Value::List(items) => Ok(&mut Rc::make_mut(items)[mv.lo + i]),
                        Value::Array(arr) => Ok(&mut Rc::make_mut(arr).as_mut_values()[mv.lo + i]),
                        Value::Moved => trap(span, LENT),
                        _ => trap(span, "internal: mutable view target is not a list or array"),
                    };
                }
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
    /// The caller's place a param is lent from.
    pub(crate) fn lender(&mut self, part: &Part) -> Res<&mut Value> {
        self.part_mut(part, Span::default())
    }

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
                (Step::Field(index), Value::Tuple(TupleVal::Boxed(fields))) => {
                    fields.get(*index)?
                }
                (Step::Some, Value::Opt(OptVal::Boxed(inner))) => inner,
                (Step::Index(indices), Value::Array(array)) => {
                    let at = array_offset(&array.dims, indices.iter(), Span::new(0, 0)).ok()?;
                    match &array.data {
                        value::ArrayData::Values(v) => v.get(at)?,
                        _ => return None,
                    }
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

    /// The first or last element of any sequence value, `None` when it is empty.
    /// Anything else is an internal error, never a silent `None`.
    pub(crate) fn seq_end(&self, seq: &Value, last: bool, span: Span) -> Res<Option<Value>> {
        let pick = |len: usize| len.checked_sub(1).map(|l| if last { l } else { 0 });
        Ok(match seq {
            Value::List(items) => pick(items.len()).map(|i| items[i].clone()),
            Value::ListView(view) => pick(view.len).and_then(|i| view.get(i)).cloned(),
            Value::MutView(mv) => match pick(mv.len) {
                Some(i) => Some(self.mut_view_get(mv, i, span)?),
                None => None,
            },
            Value::Str(bytes) => pick(bytes.len()).map(|i| Value::Char(bytes[i])),
            Value::StrView(view) => pick(view.len).and_then(|i| view.get(i)).map(Value::Char),
            Value::Array(array) => {
                if last {
                    array.last()
                } else {
                    array.first()
                }
            }
            Value::Range(lo, hi) => (lo < hi).then(|| Value::Int(if last { hi - 1 } else { *lo })),
            Value::Range128(lo, hi) => {
                (lo < hi).then(|| Value::I128(if last { hi - 1 } else { *lo }))
            }
            other => return trap(span, format!("internal: `first`/`last` of {other:?}")),
        })
    }

    fn mut_view_get(&self, mv: &MutViewVal, idx: usize, span: Span) -> Res<Value> {
        if idx >= mv.len {
            return trap(
                span,
                format!("index {idx} is outside a list of length {}", mv.len),
            );
        }
        match self.part(&mv.target) {
            Some(Value::List(items)) => Ok(items[mv.lo + idx].clone()),
            Some(Value::Array(arr)) => Ok(arr.get_unchecked_val(mv.lo + idx)),
            Some(Value::Moved) => trap(span, LENT),
            _ => trap(span, "internal: mutable view target is not a list or array"),
        }
    }

    fn mut_view_to_vec(&self, mv: &MutViewVal, span: Span) -> Res<Vec<Value>> {
        match self.part(&mv.target) {
            Some(Value::List(items)) => Ok(items[mv.lo..mv.lo + mv.len].to_vec()),
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(mv.len);
                for i in 0..mv.len {
                    out.push(arr.get_unchecked_val(mv.lo + i));
                }
                Ok(out)
            }
            Some(Value::Moved) => trap(span, LENT),
            _ => trap(span, "internal: mutable view target is not a list or array"),
        }
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
                OptVal::None => trap(span, "there is no value here: it is `None`"),
                OptVal::Int(_) => unreachable!(),
            }
        }
        Value::Moved => trap(span, LENT),
        _ => trap(span, "there is no value here: it is `None`"),
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
            Ok(&mut Rc::make_mut(array).as_mut_values()[at])
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

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
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
                let i0 = if n >= 1 {
                    self.eval(&exprs[0])?.int()
                } else {
                    0
                };
                let i1 = if n == 2 {
                    self.eval(&exprs[1])?.int()
                } else {
                    0
                };
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
                        let at = i0 as usize;
                        let arr = Rc::make_mut(array);
                        if op == AssignOp::Set {
                            arr.set(at, value);
                            return Ok(());
                        }
                        let old = arr.get_unchecked_val(at);
                        let next = combine(op, old, value, span)?;
                        arr.set(at, next);
                        return Ok(());
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
                        let at = (i0 * l1 + i1) as usize;
                        let arr = Rc::make_mut(array);
                        if op == AssignOp::Set {
                            arr.set(at, value);
                            return Ok(());
                        }
                        let old = arr.get_unchecked_val(at);
                        let next = combine(op, old, value, span)?;
                        arr.set(at, next);
                        return Ok(());
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
                        let k = match &base.ty {
                            Ty::Map(kt, _) | Ty::HashMap(kt, _) => {
                                value::strip_cached(key, kt, program)
                            }
                            _ => key.clone(),
                        };
                        Rc::make_mut(entries).insert(k, value);
                        return Ok(indices);
                    }
                    Value::Map(entries) => {
                        let k = match &base.ty {
                            Ty::Map(kt, _) | Ty::HashMap(kt, _) => {
                                value::strip_cached(key, kt, program)
                            }
                            _ => key.clone(),
                        };
                        let map = Rc::make_mut(entries);
                        let next =
                            if let Some(old) = map.get(&k).cloned() {
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
                                    _ => return trap(
                                        place.span,
                                        "this key is not in the map; store with `m[k] = v` first",
                                    ),
                                }
                            };
                        map.insert(k, next);
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
            ExprKind::Cmp(..)
            | ExprKind::Unary(..)
            | ExprKind::Cast(..)
            | ExprKind::Tuple(_)
            | ExprKind::Variant(..)
            | ExprKind::IsVariant(..)
            | ExprKind::Payload(..) => {
                let mut vals = Vec::new();
                for operand in apply::operands(e) {
                    vals.push(self.eval(operand)?);
                }
                self.apply(e, vals)?
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
                    Some(Value::Tuple(fields)) => {
                        fields.get(*index).expect("field index within tuple")
                    }
                    Some(Value::Moved) => return trap(span, LENT),
                    _ => match self.eval(base)? {
                        Value::Tuple(fields) => {
                            fields.get(*index).expect("field index within tuple")
                        }
                        _ => return trap(span, "internal: field of a value without fields"),
                    },
                }
            }
            ExprKind::IsSome(value) => {
                Value::Bool(matches!(self.eval(value)?, Value::Opt(opt) if opt.is_some()))
            }
            ExprKind::Unwrap(value, _) => match self.eval(value)? {
                Value::Opt(opt) => match opt.unwrap_owned() {
                    Some(v) => v,
                    None => return trap(span, "`!` on `None`: there is no value here"),
                },
                _ => return trap(span, "`!` on `None`: there is no value here"),
            },
            ExprKind::RangeCheck {
                value,
                lo,
                hi,
                text,
            } => {
                let v = self.eval(value)?.int();
                let lo = match lo {
                    Some(lo) => self.eval(lo)?.int(),
                    None => cpl_ir::INT_MIN,
                };
                let hi = match hi {
                    Some(hi) => self.eval(hi)?.int(),
                    None => i64::MAX,
                };
                if v < lo || v > hi {
                    return trap(span, format!("{v} is outside `{text}`"));
                }
                Value::Int(v)
            }
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
            ExprKind::Call(id, args) => self.call_or_resume(e, *id, args, span)?,
            ExprKind::CallExpr(callee, args) => self.call_expr(e, callee, args, span)?,
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
                if matches!(&e.ty, Ty::Range(inner) if **inner == Ty::I128) {
                    let lo = self.eval(lo)?.i128();
                    let hi = self.eval(hi)?.i128();
                    Value::Range128(lo, hi)
                } else {
                    let lo = self.eval(lo)?.int();
                    let hi = self.eval(hi)?.int();
                    Value::Range(lo, hi)
                }
            }
            ExprKind::DomainVal(id) => {
                let (lo, len) = self.domains[*id];
                Value::DomainVal(lo, len)
            }
            ExprKind::Builtin(op, args) => self.builtin(*op, args, e)?,
            ExprKind::MutSlice(place, lo, hi) => {
                let lo = self.eval(lo)?.int();
                let hi = self.eval(hi)?.int();
                let mut indices = Vec::new();
                self.indices(place, &mut indices)?;
                let part = self.part_of(place, &mut indices.into_iter());
                let (target, base_lo, base_len) = match self.part(&part) {
                    Some(Value::List(items)) => (part, 0usize, items.len()),
                    Some(Value::Array(arr)) => (part, 0usize, arr.len()),
                    Some(Value::MutView(mv)) => (mv.target.clone(), mv.lo, mv.len),
                    Some(Value::Moved) => return trap(span, LENT),
                    _ => return trap(span, "slice expects a list or array"),
                };
                if lo < 0 || hi < lo || hi as usize > base_len {
                    return trap(
                        span,
                        format!("slice {lo}..{hi} is outside a list of length {base_len}"),
                    );
                }
                Value::MutView(Rc::new(MutViewVal {
                    target,
                    lo: base_lo + lo as usize,
                    len: (hi - lo) as usize,
                }))
            }
            ExprKind::Mutate(op, place, args) => {
                let value = self.mutate(*op, place, args, span)?;
                self.trace_mutated(place, span);
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
                enum SortKind {
                    List,
                    Array(Vec<i64>),
                    Str,
                    MutView(Rc<MutViewVal>),
                }
                let (mut items_vec, kind) = match taken {
                    Value::List(items) => {
                        let v = Rc::try_unwrap(items).unwrap_or_else(|r| (*r).clone());
                        (v, SortKind::List)
                    }
                    Value::Array(arr) => {
                        let a = Rc::try_unwrap(arr).unwrap_or_else(|r| (*r).clone());
                        let dims = a.dims.clone();
                        (a.into_values(), SortKind::Array(dims))
                    }
                    Value::Str(bytes) => {
                        let v = bytes.iter().copied().map(Value::Char).collect();
                        (v, SortKind::Str)
                    }
                    Value::MutView(mv) => {
                        *self.resolve(place, &mut indices.iter())? = Value::MutView(Rc::clone(&mv));
                        let v = self.mut_view_to_vec(&mv, span)?;
                        (v, SortKind::MutView(mv))
                    }
                    _ => return trap(span, LENT),
                };
                let elem_ty = match &place.ty {
                    Ty::List(e) | Ty::MutView(e, _) | Ty::Array { elem: e, .. } => (**e).clone(),
                    Ty::Str => Ty::Char,
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
                            // Merge-sort using fallible comparator `fv(a, b) -> Bool`,
                            // trapping on strict-weak-order violations (matching `cpl::sort_cmp` under LOCAL).
                            items_vec = self.checked_merge_sort_by(items_vec, &fv, desc, span)?;
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
                match kind {
                    SortKind::MutView(mv) => {
                        let target = self.part_mut(&mv.target, span)?;
                        match target {
                            Value::List(items) => {
                                let dst = Rc::make_mut(items);
                                for (k, item) in items_vec.into_iter().enumerate() {
                                    dst[mv.lo + k] = item;
                                }
                            }
                            Value::Array(arr) => {
                                let dst = Rc::make_mut(arr);
                                for (k, item) in items_vec.into_iter().enumerate() {
                                    dst.set(mv.lo + k, item);
                                }
                            }
                            _ => return trap(span, LENT),
                        }
                    }
                    SortKind::Array(dims) => {
                        *self.resolve(place, &mut indices.iter())? =
                            Value::Array(Rc::new(Array::from_values(dims, items_vec)));
                    }
                    SortKind::List => {
                        *self.resolve(place, &mut indices.iter())? =
                            Value::List(Rc::new(items_vec));
                    }
                    SortKind::Str => {
                        let bytes: Vec<u8> = items_vec
                            .into_iter()
                            .map(|v| match v {
                                Value::Char(c) => c,
                                _ => 0,
                            })
                            .collect();
                        *self.resolve(place, &mut indices.iter())? = Value::Str(Rc::new(bytes));
                    }
                }
                // Not a step of its own, as before: the next step shows it.
                if let (Some(_), Some(tracer)) = (root_global(place), self.tracer.as_mut()) {
                    tracer.globals_may_change();
                }
                Value::Unit
            }
            ExprKind::Read(spec) => self.read(spec, span, &[])?,
            ExprKind::Parse(src, spec) => {
                let s = self.eval(src)?;
                let bytes = match s {
                    Value::Str(b) => (*b).clone(),
                    Value::StrView(v) => v.to_vec(),
                    _ => Vec::new(),
                };
                let saved = std::mem::replace(&mut self.input, Input::owned(bytes));
                let res = self.read(spec, span, &[]);
                self.input = saved;
                Value::Opt(OptVal::from_option(res.ok()))
            }
            ExprKind::Debug(line, args) => {
                let mut values = Vec::new();
                for arg in args {
                    values.push(self.eval(&arg.value)?);
                }
                self.fill_shown();
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
                let shown: Vec<(&cpl_ir::DebugArg, &Value)> = args
                    .iter()
                    .zip(&values)
                    .filter(|(arg, _)| !arg.label)
                    .collect();
                let name = if label.is_empty() {
                    shown
                        .iter()
                        .map(|(arg, _)| arg.text.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
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
                let prefix = if label.is_empty() {
                    String::new()
                } else {
                    format!("{name}: ")
                };
                self.debug_out
                    .extend_from_slice(format!("[line {line}] {prefix}{text}\n").as_bytes());
                let name = name.clone();
                self.traced(|tracer, view| tracer.debugged(view, span, &name, text, json));
                Value::Unit
            }
            ExprKind::Print(args, end) => {
                let mut vals = Vec::with_capacity(args.len());
                for arg in args {
                    vals.push(self.eval(&arg.value)?);
                }
                let end = match end {
                    Some(end) => Some(self.eval(end)?),
                    None => None,
                };
                self.print_values(args, &vals, end.as_ref(), span)?;
                Value::Unit
            }
            ExprKind::Format(pieces) => {
                let mut vals = Vec::new();
                for piece in pieces {
                    match piece {
                        FmtPiece::Text(_) => {}
                        FmtPiece::Value(FmtArg { value, .. }) | FmtPiece::Number(value, _) => {
                            vals.push(self.eval(value)?)
                        }
                    }
                }
                self.format_values(pieces, &vals, span)?
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
                Value::Array(Rc::new(Array::filled(dims, cell)))
            }
            ExprKind::DomainLen(id) => Value::Int(self.domains[*id].1),
            ExprKind::DomainLo(id) => Value::Int(self.domains[*id].0),
        })
    }

    /// The length of each dimension of an array being made.
    fn dims(&mut self, dims: &'p [DimSpec], span: Span) -> Res<Vec<i64>> {
        self.read_dims(dims, span, &[])
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
        self.index_value(holder, values, &base.ty, span)
    }

    /// The element of `holder`, a value of `base_ty`, at `values`.
    pub(crate) fn index_value(
        &self,
        holder: &Value,
        values: &[Value],
        base_ty: &Ty,
        span: Span,
    ) -> Res<Value> {
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
                Ok(array.get_unchecked_val(at))
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
            Value::ListView(view) => {
                let index = values[0].int();
                match usize::try_from(index).ok().and_then(|i| view.get(i)) {
                    Some(item) => Ok(item.clone()),
                    None => trap(
                        span,
                        format!("index {index} is outside a list of length {}", view.len),
                    ),
                }
            }
            Value::MutView(mv) => {
                let index = values[0].int();
                match usize::try_from(index).ok().filter(|i| *i < mv.len) {
                    Some(i) => self.mut_view_get(mv, i, span),
                    None => trap(
                        span,
                        format!("index {index} is outside a list of length {}", mv.len),
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
            Value::Range(lo, hi) => {
                let index = values[0].int();
                let len = hi.saturating_sub(*lo).max(0);
                if index < 0 || index >= len {
                    trap(
                        span,
                        format!("index {index} is outside a range of length {len}"),
                    )
                } else {
                    Ok(Value::Int(lo + index))
                }
            }
            Value::StrView(view) => {
                let index = values[0].int();
                match usize::try_from(index).ok().and_then(|i| view.get(i)) {
                    Some(byte) => Ok(Value::Char(byte)),
                    None => trap(
                        span,
                        format!("index {index} is outside a string of length {}", view.len),
                    ),
                }
            }
            Value::Map(entries) => {
                let lookup_key = match base_ty {
                    Ty::Map(kt, _) | Ty::HashMap(kt, _) => {
                        value::strip_cached(&values[0], kt, self.program)
                    }
                    _ => values[0].clone(),
                };
                match entries.get(&lookup_key) {
                    Some(value) => Ok(value.clone()),
                    None => {
                        let mut key = Vec::new();
                        if let Ty::Map(key_ty, _) | Ty::HashMap(key_ty, _) = base_ty {
                            show(&mut key, &values[0], key_ty, 6, self.program, &self.domains);
                        }
                        let key: String = key.iter().map(|b| *b as char).collect();
                        trap(
                            span,
                            format!(
                                "the key `{key}` is not in the map; `m.get(k)` gives an optional"
                            ),
                        )
                    }
                }
            }
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
                (_, Ok(v)) if v != i64::MIN => Value::Int(v),
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
                float_int(v, span)?
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
                Ok(c) if c.is_ascii() => Value::Char(c),
                _ => return trap(span, format!("{} is outside `0..=127`", value.int())),
            },
            Cast::EnumOrd(_) => match value {
                Value::Enum(tag, _) => Value::Int(tag as i64),
                other => other,
            },
            Cast::EnumVal(eid) => match value {
                Value::Enum(tag, _) => {
                    let val = self.program.enums[eid]
                        .values
                        .get(tag as usize)
                        .copied()
                        .flatten()
                        .unwrap_or(tag as i64);
                    Value::Int(val)
                }
                other => other,
            },
            Cast::OrdToEnum(_) => Value::Enum(value.int() as u32, Rc::new(Vec::new())),
            Cast::Ord | Cast::Trust => value,
            Cast::Raw => match value {
                Value::U64(u) => Value::Int(u as i64),
                other => other,
            },
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
        if self.in_closure
            || self
                .fed_loops
                .last()
                .is_some_and(|fed| fed.callee == self.base)
        {
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
    fn feed(&mut self, value: Value, span: Span) -> Res<Value> {
        let fed = self.fed_loops.last().expect("asked by the caller");
        let (callee, caller, slot, id, body) = (fed.callee, fed.caller, fed.slot, fed.id, fed.body);
        let locals = self.frame_locals.pop();
        self.base = caller;
        self.stack[caller + slot] = value;
        self.fed_bodies += 1;
        self.feeding();
        let flow = match self.fed_body_vm(id, caller) {
            Some(flow) => flow,
            None => self.block(body),
        };
        self.fed_bodies -= 1;
        let updated = self.stack[caller + slot].clone();
        self.base = callee;
        self.frame_locals.extend(locals);
        let left = match flow? {
            Flow::Next | Flow::Continue(None) => return Ok(updated),
            Flow::Continue(Some(target)) if target == id => return Ok(updated),
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
    fn call_feeding(
        &mut self,
        id: FnId,
        args: &'p [Arg],
        span: Span,
        fed: Option<Fed<'p>>,
    ) -> Res<Value> {
        let f = &self.program.fns[id];
        if !self.program.judge_fns.is_empty() && self.program.judge_fns.contains(&id) {
            let cpp = self.program.cpp_bodies.iter().any(|(c, _)| *c == id);
            return trap(span, if cpp { CPP_ONLY } else { NO_GRADER });
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
                        values[i] = self.lend(place)?;
                    }
                }
                let base = self.stack.len();
                self.stack.extend(values);
                self.stack.resize(base + f.locals.len(), Value::Unit);
                base
            }
        };
        self.call_at(id, base, span, fed)
    }

    /// Runs fn `id` in the frame at `base`, which holds its arguments.
    pub(crate) fn call_at(
        &mut self,
        id: FnId,
        base: usize,
        span: Span,
        fed: Option<Fed<'p>>,
    ) -> Res<Value> {
        let f = &self.program.fns[id];
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let entered = self.tracer.as_mut().map(|tracer| tracer.enter_fn(&f.name));
        if self.observed() {
            let params: Vec<&str> = f.locals[..f.params.len()]
                .iter()
                .map(|l| l.name.as_str())
                .collect();
            self.traced(|tracer, view| {
                tracer.enter_call(&f.name, || trace::args_json(view, &params))
            });
        }
        let feeding = fed.is_some();
        if let Some(fed) = fed {
            self.fed_loops.push(Fed {
                callee: base,
                ..fed
            });
        }
        let in_closure = std::mem::replace(&mut self.in_closure, false);
        let dom_mark = self.saved_domains.len();
        for &d in &self.fn_guarded[id] {
            self.saved_domains.push((d, self.domains[d]));
        }
        // A traced chunk leaves a `yield` to the tree-walker, which feeds the loop.
        let fed_ok = !feeding || self.vm.as_ref().is_some_and(|c| c.main.traced);
        let mut flow = if fed_ok {
            if let Some(compiled) = self.vm.clone() {
                if compiled.fns.get(id).is_some_and(Option::is_some) {
                    self.run_fn_vm_waited(&compiled, id, span, saved, dom_mark)
                } else {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record(format!("fn:{}(uncompiled)", f.name));
                    }
                    self.block(&f.body)
                }
            } else {
                self.block(&f.body)
            }
        } else {
            if let Some(stats) = &mut self.vm_stats {
                let why = if feeding { "feeding" } else { "part" };
                stats.record(format!("fn:{}({why})", f.name));
            }
            self.block(&f.body)
        };
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        if let (Some(entered), Some(tracer)) = (entered, &mut self.tracer) {
            tracer.leave_fn(entered);
        }
        if self.observed() {
            self.traced(|tracer, view| {
                let returned = match &flow {
                    Ok(Flow::Return(value)) if tracer.keeps_call() && f.ret != Ty::Unit => {
                        Some(trace::json_of(value, &f.ret, view))
                    }
                    _ => None,
                };
                tracer.leave_call(returned);
            });
            flow = flow.and_then(|flow| self.unstopped(span).map(|()| flow));
        }
        if flow.is_err() {
            self.keep_trap_frame();
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
        self.after_call(id, span);
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

    fn eq_ty_val(&mut self, ty: &Ty, l: &Value, r: &Value, span: Span) -> Res<bool> {
        match ty {
            Ty::Struct(id) => {
                if let Some(eq_fn) = self.program.structs[*id].eq_fn {
                    return Ok(self
                        .call_direct(eq_fn, vec![l.clone(), r.clone()], span)?
                        .bool());
                }
                let fields = self.program.structs[*id].fields.clone();
                let cached = self.program.structs[*id].cached_fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (i, ((_, ft), (lv, rv))) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                        .enumerate()
                    {
                        if cached.contains(&i) {
                            continue;
                        }
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

    fn cmp_ty_val(&mut self, ty: &Ty, l: &Value, r: &Value, span: Span) -> Res<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match ty {
            Ty::Struct(id) => {
                if let Some(hook) = self.program.structs[*id].ord {
                    return self.user_order(hook, l, r, span);
                }
                let fields = self.program.structs[*id].fields.clone();
                let cached = self.program.structs[*id].cached_fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (i, ((_, ft), (lv, rv))) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                        .enumerate()
                    {
                        if cached.contains(&i) {
                            continue;
                        }
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

    fn merge_sort_ty(
        &mut self,
        mut xs: Vec<Value>,
        ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
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

    fn checked_merge_sort_by(
        &mut self,
        xs: Vec<Value>,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        let saved_profiler = self.profiler.take();
        for x in &xs {
            let res = self.call_value(cmp.clone(), vec![x.clone(), x.clone()], span);
            match res {
                Ok(v) if v.bool() => {
                    self.profiler = saved_profiler;
                    return trap(
                        span,
                        "sort `cmp:` comparator is not a strict weak order: cmp(x, x) is true",
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    self.profiler = saved_profiler;
                    return Err(e);
                }
            }
        }
        self.profiler = saved_profiler;
        let out = self.merge_sort_by(xs, cmp, desc, span)?;
        let saved_profiler = self.profiler.take();
        for i in 1..out.len() {
            let checks = [
                (&out[i], &out[i - 1]),
                (&out[i], &out[0]),
                (out.last().unwrap(), &out[i - 1]),
            ];
            for (lhs, rhs) in checks {
                let pair = if desc {
                    vec![rhs.clone(), lhs.clone()]
                } else {
                    vec![lhs.clone(), rhs.clone()]
                };
                match self.call_value(cmp.clone(), pair, span) {
                    Ok(v) if v.bool() => {
                        self.profiler = saved_profiler;
                        return trap(
                            span,
                            "sort `cmp:` comparator is not a strict weak order: result is not ordered",
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        self.profiler = saved_profiler;
                        return Err(e);
                    }
                }
            }
        }
        self.profiler = saved_profiler;
        Ok(out)
    }

    fn merge_sort_by(
        &mut self,
        mut xs: Vec<Value>,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
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
            let (first, second) = if desc {
                (&left[i], &right[j])
            } else {
                (&right[j], &left[i])
            };
            let lt = self
                .call_value(cmp.clone(), vec![first.clone(), second.clone()], span)?
                .bool();
            if lt {
                let saved_profiler = self.profiler.take();
                let rev = self.call_value(cmp.clone(), vec![second.clone(), first.clone()], span);
                self.profiler = saved_profiler;
                if rev?.bool() {
                    return trap(
                        span,
                        "sort `cmp:` comparator is not a strict weak order: both cmp(a, b) and cmp(b, a) are true",
                    );
                }
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

    /// Renders a format string; `vals` holds one value per non-text piece, in order.
    fn format_values(&mut self, pieces: &'p [FmtPiece], vals: &[Value], span: Span) -> Res<Value> {
        let mut text = Vec::new();
        let mut vals = vals.iter();
        for piece in pieces {
            match piece {
                FmtPiece::Text(bytes) => text.extend_from_slice(bytes),
                FmtPiece::Value(FmtArg { value, digits }) => {
                    let v = vals.next().expect("one value per format piece");
                    self.show_val(&mut text, v, &value.ty, *digits, span)?;
                }
                FmtPiece::Number(_, fmt) => {
                    match vals.next().expect("one value per format piece") {
                        Value::Int(v) => text.extend(fmt.int(*v).bytes()),
                        Value::Float(v) => text.extend(fmt.float(*v).bytes()),
                        _ => unreachable!("a number format takes an Int or an F64"),
                    }
                }
            }
        }
        Ok(Value::Str(Rc::new(text)))
    }

    fn print_values(
        &mut self,
        args: &'p [FmtArg],
        vals: &[Value],
        end: Option<&Value>,
        span: Span,
    ) -> Res<()> {
        let start = self.out.len();
        let sparse = args.iter().any(|arg| arg.value.ty.may_show_nothing());
        for (i, (arg, value)) in args.iter().zip(vals).enumerate() {
            let mut buf = Vec::new();
            self.show_val(&mut buf, value, &arg.value.ty, arg.digits, span)?;
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
                let mut buf = Vec::new();
                self.show_val(&mut buf, end, &Ty::Str, 6, span)?;
                self.out.extend_from_slice(&buf);
            }
            None => self.out.push(b'\n'),
        }
        if self.observed() {
            let text = String::from_utf8_lossy(&self.out[start.min(self.out.len())..]).into_owned();
            self.traced(|tracer, view| tracer.printed(view, span, &text));
        }
        if self.program.interactive {
            self.write_out(true);
        } else {
            self.flush_if_large();
        }
        Ok(())
    }

    fn show_val(
        &mut self,
        out: &mut Vec<u8>,
        value: &Value,
        ty: &Ty,
        digits: u32,
        span: Span,
    ) -> Res<()> {
        // Printed once: an empty list keeps its element type, so asking
        // again would find the same hook and never stop.
        if let Some((parts, ty)) = self.shown_parts(value, ty, span)? {
            show(out, &parts, &ty, digits, self.program, &self.domains);
            return Ok(());
        }
        if let Value::MutView(mv) = value {
            let items = Value::List(Rc::new(self.mut_view_to_vec(mv, span)?));
            show(out, &items, ty, digits, self.program, &self.domains);
            return Ok(());
        }
        show(out, value, ty, digits, self.program, &self.domains);
        Ok(())
    }

    fn builtin(&mut self, op: Builtin, args: &'p [Expr], e: &'p Expr) -> Res<Value> {
        // `k in m` and `m.get(k)` on a variable look in it where it is:
        // copying a map to ask it one question makes a loop quadratic.
        if let (Builtin::Contains | Builtin::MapGet, [holder, key]) = (op, args) {
            if matches!(holder.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                let key = value::strip_cached(&self.eval(key)?, &key.ty, self.program);
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
        self.builtin_values(op, args, e, values)
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
                    (Mutation::Push, Some(value)) => {
                        let updated = self.feed(value, span)?;
                        if let Some(Expr {
                            kind: ExprKind::Local(item_slot),
                            ..
                        }) = args.first()
                        {
                            let at = self.base + *item_slot;
                            let lent = match &self.stack[at] {
                                Value::Alias(global, to) => Some(Part {
                                    global: *global,
                                    at: *to,
                                    steps: Vec::new(),
                                }),
                                Value::Part(part) => Some(Part::clone(part)),
                                _ => None,
                            };
                            match lent {
                                Some(part) => *self.part_mut(&part, span)? = updated,
                                None => self.stack[at] = updated,
                            }
                        }
                    }
                    (Mutation::Extend, Some(Value::List(items))) => {
                        for item in items.iter() {
                            self.feed(item.clone(), span)?;
                        }
                    }
                    (Mutation::Extend, Some(Value::Array(array))) => {
                        for i in 0..array.len() {
                            self.feed(array.get_unchecked_val(i), span)?;
                        }
                    }
                    _ => {
                        return trap(
                            span,
                            "internal: a `yields` fn did something else to its list",
                        )
                    }
                }
                return Ok(Value::Unit);
            }
        }
        let arg = arg_vals.first().cloned().unwrap_or(Value::Unit);
        let arg2 = arg_vals.get(1).cloned().unwrap_or(Value::Unit);
        if op == Mutation::Shuffle {
            let mut rng_state = self.rng_state;
            let choices = self.choices.clone();
            let mut below = |n: usize| {
                draw_below(choices.as_deref(), &mut rng_state, n as u128, span.start) as usize
            };
            let mv_opt = match self.resolve(place, &mut indices.iter())? {
                Value::MutView(mv) => Some(Rc::clone(mv)),
                _ => None,
            };
            let res = if let Some(mv) = mv_opt {
                match self.part_mut(&mv.target, span)? {
                    Value::List(items) => {
                        let v = Rc::make_mut(items);
                        for i in (1..mv.len).rev() {
                            let j = below(i + 1);
                            v.swap(mv.lo + i, mv.lo + j);
                        }
                        Value::Unit
                    }
                    Value::Array(arr) => {
                        let a = Rc::make_mut(arr);
                        for i in (1..mv.len).rev() {
                            let j = below(i + 1);
                            a.swap(mv.lo + i, mv.lo + j);
                        }
                        Value::Unit
                    }
                    Value::Moved => return trap(span, LENT),
                    _ => return trap(span, "internal: this value can't be shuffled"),
                }
            } else {
                let cell = self.resolve(place, &mut indices.iter())?;
                match cell {
                    Value::Moved => return trap(span, LENT),
                    Value::List(items) => {
                        let v = Rc::make_mut(items);
                        for i in (1..v.len()).rev() {
                            let j = below(i + 1);
                            v.swap(i, j);
                        }
                        Value::Unit
                    }
                    Value::Array(arr) => {
                        let a = Rc::make_mut(arr);
                        let n = a.len();
                        for i in (1..n).rev() {
                            let j = below(i + 1);
                            a.swap(i, j);
                        }
                        Value::Unit
                    }
                    _ => return trap(span, "internal: this value can't be shuffled"),
                }
            };
            self.rng_state = rng_state;
            return Ok(res);
        }
        let mv_opt = match self.resolve(place, &mut indices.iter())? {
            Value::MutView(mv) => Some(Rc::clone(mv)),
            _ => None,
        };
        if let Some(mv) = mv_opt {
            let target = self.part_mut(&mv.target, span)?;
            return Ok(match (op, target) {
                (_, Value::Moved) => return trap(span, LENT),
                (Mutation::Reverse, Value::List(items)) => {
                    Rc::make_mut(items)[mv.lo..mv.lo + mv.len].reverse();
                    Value::Unit
                }
                (Mutation::Reverse, Value::Array(arr)) => {
                    let a = Rc::make_mut(arr);
                    for i in 0..mv.len / 2 {
                        a.swap(mv.lo + i, mv.lo + mv.len - 1 - i);
                    }
                    Value::Unit
                }
                (Mutation::Swap, Value::List(items)) => {
                    let (i, j) = (arg.int(), arg2.int());
                    let v = Rc::make_mut(items);
                    match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                        (Some(i), Some(j)) if i < mv.len && j < mv.len => {
                            v.swap(mv.lo + i, mv.lo + j);
                        }
                        _ => {
                            return trap(
                                span,
                                format!("swap({i}, {j}) is outside a list of length {}", mv.len),
                            )
                        }
                    }
                    Value::Unit
                }
                (Mutation::Swap, Value::Array(arr)) => {
                    let (i, j) = (arg.int(), arg2.int());
                    let a = Rc::make_mut(arr);
                    match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                        (Some(i), Some(j)) if i < mv.len && j < mv.len => {
                            a.swap(mv.lo + i, mv.lo + j);
                        }
                        _ => {
                            return trap(
                                span,
                                format!("swap({i}, {j}) is outside an array of length {}", mv.len),
                            )
                        }
                    }
                    Value::Unit
                }
                (Mutation::Fill, Value::List(items)) => {
                    Rc::make_mut(items)[mv.lo..mv.lo + mv.len].fill(arg);
                    Value::Unit
                }
                (Mutation::Fill, Value::Array(arr)) => {
                    let a = Rc::make_mut(arr);
                    for i in 0..mv.len {
                        a.set(mv.lo + i, arg.clone());
                    }
                    Value::Unit
                }
                _ => return trap(span, "internal: this value can't be changed that way"),
            });
        }
        let program = self.program;
        let cell = self.resolve(place, &mut indices.iter())?;
        mutation::mutate_cell(op, cell, &place.ty, arg, arg2, program, span)
    }

    // ----- input -----
}

const CPP_ONLY: &str =
    "this fn's body is C++ (`@cpp`), so only the C++ build runs it: `cpl run` does that by itself";
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
pub(crate) fn describe(lo: i64, len: i64) -> String {
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
            match result.filter(|v| *v != i64::MIN) {
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
                BinOp::Shl if a < 0 => {
                    return trap(span, format!("`<<` of the negative number {a}"))
                }
                BinOp::Shl => {
                    match Some(a << b).filter(|shifted| *shifted >= 0 && shifted >> b == a) {
                        Some(value) => Value::I128(value),
                        None => {
                            return trap(
                                span,
                                format!(
                            "integer overflow: `{a} << {b}` does not fit in an `I128` (128 bits)"
                        ),
                            )
                        }
                    }
                }
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

/// What an empty `seq` is called in a trap.
fn seq_noun(seq: &Value) -> &'static str {
    match seq {
        Value::Str(_) | Value::StrView(_) => "string",
        Value::Array(_) => "array",
        Value::Range(..) | Value::Range128(..) => "range",
        _ => "list",
    }
}
