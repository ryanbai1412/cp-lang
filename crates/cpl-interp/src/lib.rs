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
mod arith;
mod assign;
mod bounded_read;
mod builtin;
use arith::{binary, combine};
mod call;
mod cases;
pub mod checkpoint;
mod closed_form;
mod compare;
mod float_fn;
mod fn_value;
mod for_loop;
mod input;
mod io;
mod mutation;
mod place;
mod read;
mod release;
mod show_hook;
mod sort;
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

use assign::{cell_mut, field_mut, inside_mut};
use compare::compare;
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, DimSpec, Expr, ExprKind, FmtArg, FmtPiece,
    FnId, Iter, LoopId, Mutation, Place, PlaceKind, Program, SortBy, Spec, Stmt, Ty, UnOp,
};
use input::Input;
pub use input::More;
use observe::Observer;
pub(crate) use place::root_global;
use place::{leading_indices, root_local};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
pub use stop::{Limits, Stop, CANCELLED, OUT_OF_TIME};
pub use threads::{run_for, run_on_this_thread, run_with_on_this_thread};
use value::{
    show, zero, Array, CaptureVal, ListViewVal, MutViewVal, OptVal, Part, Step, StrViewVal,
    TupleVal, Value, Wide,
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
            || (m.starts_with("key ") && m.contains(" is not one of "))
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
    let _checks = release::Set::new(io.checks);
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
        trace_show_tys: show_hook::ShowTys::default(),
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
        profile: interp
            .profiler
            .take()
            .map(|profiler| profile::Profiler::finish(*profiler)),
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
    /// Which types, globals and frames hold a value with an `impl Show`.
    trace_show_tys: show_hook::ShowTys,
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
                    if matches!(e.kind, ExprKind::MutSlice(..) | ExprKind::MutLine(..))
                        || matches!(e.ty, Ty::MutView(..))
                    {
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
            if !has_mut_view {
                let (traced, profiled) = (self.observed(), self.profiler.is_some());
                self.vm = Some(Rc::new(vm::CompiledProgram::compile(
                    program,
                    &self.feeds,
                    traced,
                    profiled,
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
        self.run_after_setup()
    }

    /// What the tracer sees of the program's state right now.
    fn view(&self) -> trace::View<'_> {
        self.view_in(Vec::new())
    }

    fn view_in<'s>(&'s self, mut locals: Vec<(&'s str, &'s Ty, &'s Value)>) -> trace::View<'s> {
        // The map shortens the locals' `'p` to `'s`.
        #[allow(clippy::map_identity)]
        locals.extend(self.named_locals().map(|(n, t, v)| (n, t, v)));
        trace::View {
            program: self.program,
            globals: &self.globals,
            domains: &self.domains,
            locals,
            raw: None,
            depth: self.call_frames.len(),
            call_frames: &self.call_frames,
            shown: &self.trace_shown,
            read: self.input.cursor(),
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
            observe::Observer::step(&mut **profiler, stmt);
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
            // Skipped (R09.3.7), but a condition that stores still runs, as in the C++.
            Stmt::Assert {
                cond, kept: false, ..
            } if !release::keeps(cpl_ir::checks::Check::Asserts) => {
                self.eval(cond)?;
            }
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

    // ----- places -----

    /// The first or last element of any sequence value, `None` when it is empty.
    /// Anything else is an internal error, never a silent `None`.
    pub(crate) fn seq_end(&self, seq: &Value, last: bool, span: Span) -> Res<Option<Value>> {
        let pick = |len: usize| len.checked_sub(1).map(|l| if last { l } else { 0 });
        Ok(match seq {
            Value::List(items) => pick(items.len()).map(|i| items.get_unchecked_val(i)),
            Value::ListView(view) => pick(view.len).and_then(|i| view.get(i)),
            Value::MutView(mv) => match pick(mv.len) {
                Some(i) => Some(self.mut_view_get(mv, i, span)?),
                None => None,
            },
            Value::Str(bytes) => pick(bytes.len()).map(|i| Value::Char(bytes[i])),
            Value::StrView(view) => pick(view.len).and_then(|i| view.get(i)).map(Value::Char),
            Value::Array(array) => pick(array.len()).and_then(|i| array.get(i)),
            Value::Range(lo, hi) => (lo < hi).then(|| Value::Int(if last { hi - 1 } else { *lo })),
            Value::Range128(r) => {
                (r.0 < r.1).then(|| Value::I128((if last { r.1 - 1 } else { r.0 }).into()))
            }
            other => return trap(span, format!("internal: `first`/`last` of {other:?}")),
        })
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
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
                let vals = self.eval_operands(e)?;
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
                if matches!(&e.ty, Ty::Range(inner) if **inner == Ty::Float) {
                    Value::RangeF(self.eval(lo)?.float(), self.eval(hi)?.float())
                } else if matches!(&e.ty, Ty::Range(inner) if **inner == Ty::I128) {
                    Value::range128(self.eval(lo)?.i128(), self.eval(hi)?.i128())
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
                let (target, base_lo, base_len, step, key0) = match self.part(&part) {
                    Some(Value::List(items)) => (part, 0usize, items.len(), 1, 0),
                    Some(Value::Array(arr)) => (part, 0usize, arr.len(), 1, 0),
                    Some(Value::MutView(mv)) => {
                        (mv.target.clone(), mv.lo, mv.len, mv.step, mv.key0)
                    }
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
                    lo: base_lo + lo as usize * step,
                    len: (hi - lo) as usize,
                    step,
                    key0: key0 + lo,
                }))
            }
            ExprKind::MutLine(place, at, axis) => {
                let at = self.eval(at)?.int();
                let mut indices = Vec::new();
                self.indices(place, &mut indices)?;
                let part = self.part_of(place, &mut indices.into_iter());
                let (rows, cols) = match self.part(&part) {
                    Some(Value::Array(arr)) if arr.dims.len() == 2 => {
                        (arr.dims[0] as usize, arr.dims[1] as usize)
                    }
                    Some(Value::Moved) => return trap(span, LENT),
                    _ => return trap(span, "internal: a grid line needs a 2D array"),
                };
                let (fixed, len, step) = if *axis == 1 {
                    (rows, cols, 1)
                } else {
                    (cols, rows, cols)
                };
                let Some(at) = usize::try_from(at).ok().filter(|at| *at < fixed) else {
                    return trap(
                        span,
                        format!("index {at} is outside an axis of length {fixed}"),
                    );
                };
                let lo = if *axis == 1 { at * cols } else { at };
                Value::MutView(Rc::new(MutViewVal {
                    target: part,
                    lo,
                    len,
                    step,
                    key0: 0,
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
                        let v = value::Cells::unwrap_values(items);
                        (v, SortKind::List)
                    }
                    Value::Array(arr) => {
                        let a = Rc::try_unwrap(arr).unwrap_or_else(|r| (*r).clone());
                        let dims = a.dims.clone();
                        (a.data.into_values(), SortKind::Array(dims))
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
                                    dst.set(mv.lo + k * mv.step, item);
                                }
                            }
                            Value::Array(arr) => {
                                let dst = Rc::make_mut(arr);
                                for (k, item) in items_vec.into_iter().enumerate() {
                                    dst.set(mv.lo + k * mv.step, item);
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
                        *self.resolve(place, &mut indices.iter())? = Value::list(items_vec);
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
            Cast::IntToI128 => Value::I128(i128::from(value.int()).into()),
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
            Cast::U64ToI128 => Value::I128(i128::from(value.u64()).into()),
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

#[cfg(test)]
mod tests;

/// What an empty `seq` is called in a trap.
fn seq_noun(seq: &Value) -> &'static str {
    match seq {
        Value::Str(_) | Value::StrView(_) => "string",
        Value::Array(_) => "array",
        Value::Range(..) | Value::Range128(..) | Value::RangeF(..) => "range",
        _ => "list",
    }
}
