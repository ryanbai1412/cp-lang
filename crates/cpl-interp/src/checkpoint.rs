//! Checkpoints: the state of a run at a round of a loop, taken on the
//! untraced run, and a run that starts again from one.
//!
//! A checkpoint holds each call under way as a frame, outermost (main)
//! first: its locals, the loops it is in, and the statement whose call it
//! waits on. Every value is shared and copied on write, so a checkpoint
//! copies locals and globals, not the heap. A run from a checkpoint runs on
//! the tree-walker, traced or not, and reads and prints exactly what the
//! whole run did from there on.
//!
//! The VM keeps the calls under way on the Rust stack; while checkpoints are
//! on, each VM call also pushes a [`Suspended`] record of its caller, so a
//! checkpoint can read every frame. A checkpoint is taken only at a jump
//! back to the top of a `while` or a `for v in lo..hi` whose enclosing loops
//! are the same kinds, and only when every call under way is a VM call that
//! is the whole value of a `let`, an assignment to a local, a `return` or an
//! expression statement, to a fn without `mut` parameters.

// Recording runs on the VM, which wasm builds leave out.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use crate::observe::{Observer, Unobserved};
use crate::value::Value;
#[cfg(not(target_arch = "wasm32"))]
use crate::vm::{Chunk, CompiledProgram, LoopReg, SeqHeld, SlotLoc};
use crate::{trace, trap, Flow, Interp, Io, Limits, Observed, Res};
use cpl_diag::Span;
use cpl_ir::{Block, FnId, Iter, LoopId, Program, Stmt};
use std::io::Write;
use std::rc::Rc;

/// Where one of a frame's loops was, outermost first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoopAt {
    While(LoopId),
    /// In the round with `v`; for the innermost loop of the innermost frame,
    /// the round about to start.
    Range {
        id: LoopId,
        v: i64,
        hi: i64,
    },
    /// In the round at position `k` (as for `Range`), walking the frame's
    /// `held` sequence `held`.
    Seq {
        id: LoopId,
        k: i64,
        held: usize,
    },
}

impl LoopAt {
    fn id(self) -> LoopId {
        match self {
            LoopAt::While(id) | LoopAt::Range { id, .. } | LoopAt::Seq { id, .. } => id,
        }
    }
}

/// One call under way at a checkpoint.
#[derive(Clone)]
struct FrameAt {
    body: Body,
    locals: Vec<Value>,
    loops: Vec<LoopAt>,
    /// The statement, then the call in it, the frame waits on, by address;
    /// `None` for the innermost frame.
    site: Option<(usize, usize)>,
    /// Where the frame was called from.
    span: Span,
    /// How many saved domains there were before the call.
    doms: usize,
    /// What the calls before `site` in its statement returned, by address.
    before: Vec<(usize, Value)>,
    /// The sequences its `for` loops walk.
    held: Vec<Value>,
}

/// Whose body a frame runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Body {
    Main,
    Fn(FnId),
    Closure(cpl_ir::ClosureId),
}

/// A run's state at the start of a round of a loop.
#[derive(Clone)]
pub struct Checkpoint {
    /// Loop rounds and calls before it: the run's step clock.
    pub ticks: u64,
    /// Bytes the run had printed.
    pub printed: u64,
    /// Trace steps before it, on a counted run ([`count_on_this_thread`]).
    pub steps: usize,
    /// How many calls were under way.
    depth: usize,
    /// The innermost frame, and up from it the frames that called it.
    frames: Rc<Chain>,
    globals: Vec<Value>,
    domains: Vec<(i64, i64)>,
    saved_domains: Vec<(cpl_ir::DomainId, (i64, i64))>,
    input_at: usize,
    rng_state: u64,
    queries: i64,
    /// Under `@!multitest`: the case, and how many are left after it.
    case: Option<(u64, Option<i64>)>,
}

impl Checkpoint {
    /// How many calls were under way.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Under `@!multitest`, the test case, from 1.
    pub fn case(&self) -> Option<u64> {
        self.case.map(|(case, _)| case)
    }
}

/// A frame and, up from it, the frames that called it.
struct Chain {
    frame: FrameAt,
    up: Option<Rc<Chain>>,
}

/// Whether a checkpoint can stand inside a caller's call, and its frame
/// with the ones around it when it can.
enum Snap {
    Unknown,
    No,
    Yes(Rc<Chain>),
}

/// A VM caller waiting on its call: where its registers are and where the
/// call is.
pub(crate) struct Suspended {
    caller: *const (),
    pc: u32,
    base_i: usize,
    regs_v: *const Value,
    n_v: usize,
    base: usize,
    callee: Body,
    depth: usize,
    doms: usize,
    span: Span,
    snap: Snap,
    /// Whether the caller's frame can change during the call, so each
    /// checkpoint reads it again.
    volatile: bool,
    /// When the call is the tree-walker's, in a statement the caller falls
    /// back to: the caller's integer registers, which are then not at
    /// `base_i` of the registers the callee runs on. Its locals are on the
    /// stack.
    fallback: Option<(*const i64, usize)>,
}

/// A VM caller running a statement on the tree-walker, which a call in it
/// waits in.
#[derive(Clone, Copy)]
pub(crate) struct Fallback {
    caller: *const (),
    pc: u32,
    regs_i: *const i64,
    n_i: usize,
    regs_v: *const Value,
    n_v: usize,
    base: usize,
    depth: usize,
    volatile: bool,
}

/// A call's caller chunk and address, and what it returned last.
type Kept = (*const (), usize, Value);

/// The step clock, the checkpoints taken, and the one a run starts from.
pub(crate) struct Clock {
    done: u64,
    loaded: u32,
    every: u64,
    next_at: u64,
    keep: usize,
    spread: usize,
    taken: Vec<Checkpoint>,
    /// The VM calls under way, outermost first, while checkpoints are on.
    frames: Vec<Suspended>,
    /// How many of `frames`, outermost first, have their [`Snap`] worked out.
    known: usize,
    /// The test cases still to run after the current one.
    pub(crate) cases_left: Option<i64>,
    /// In a run from a checkpoint, the call (by address) the caller waits
    /// on, and the frames from its callee in.
    pending: Option<(usize, Vec<FrameAt>)>,
    /// In a run from a checkpoint, what the calls before the one waited on
    /// returned, by address.
    returned: Vec<(usize, Value)>,
    /// While checkpoints are on, per call depth, what the last call at each
    /// kept call (by caller chunk and call address) returned.
    kept: Vec<Vec<Kept>>,
    /// While checkpoints are on, the VM statement running on the tree-walker.
    fallback: Option<Fallback>,
    pub(crate) printed: u64,
    pub(crate) resume: Option<Checkpoint>,
    pub(crate) range_from: Option<(i64, i64)>,
    /// In a run from a checkpoint, the sequence the next `for` over one
    /// walks, and the round it starts from.
    seq_from: Option<(Value, i64)>,
    /// How many more looks at the clock may come one tick apart while a
    /// checkpoint is due.
    soon: u32,
}

/// How many looks one tick apart a due checkpoint gets, after each one
/// taken.
const SOON: u32 = 4096;

impl Default for Clock {
    fn default() -> Clock {
        Clock {
            done: 0,
            loaded: crate::stop::EVERY,
            every: 0,
            next_at: 0,
            keep: usize::MAX,
            spread: usize::MAX,
            taken: Vec::new(),
            frames: Vec::new(),
            known: 0,
            cases_left: None,
            pending: None,
            returned: Vec::new(),
            kept: Vec::new(),
            fallback: None,
            printed: 0,
            resume: None,
            range_from: None,
            seq_from: None,
            soon: SOON,
        }
    }
}

impl Clock {
    /// Keeps what the call `e` of `caller` at call depth `depth` returned.
    fn keep(&mut self, depth: usize, caller: *const (), e: usize, value: Value) {
        if self.kept.len() <= depth {
            self.kept.resize_with(depth + 1, Vec::new);
        }
        let kept = &mut self.kept[depth];
        match kept.iter_mut().find(|k| k.0 == caller && k.1 == e) {
            Some(k) => k.2 = value,
            None => kept.push((caller, e, value)),
        }
    }

    /// Whether a run from a checkpoint still waits on a call. Never in the
    /// wasm build, which does not run from checkpoints, so none of the
    /// replay is in it.
    #[inline]
    pub(crate) fn waits(&self) -> bool {
        cfg!(not(target_arch = "wasm32")) && self.pending.is_some()
    }

    pub(crate) fn on(&self) -> bool {
        self.every > 0
    }

    /// Counts the ticks just used up and loads `next` more.
    pub(crate) fn reload(&mut self, next: u32) -> u32 {
        self.done += u64::from(self.loaded);
        self.loaded = next;
        next
    }

    /// The countdown to load after a look at the clock that loaded `next`.
    /// While a checkpoint is due, the next tick looks again: the tick that
    /// came due may be one no checkpoint is taken at, a call or a statement
    /// on the tree-walker, and a loop's rounds can bring every look to the
    /// same such tick.
    pub(crate) fn after_look(&mut self, next: u32) -> u32 {
        if self.due() && self.soon > 0 {
            self.soon -= 1;
            self.loaded = 1;
            return 1;
        }
        next
    }

    /// Loads `next` in place of the load the look just made, none of which
    /// is used yet.
    pub(crate) fn load_instead(&mut self, next: u32) -> u32 {
        self.loaded = next;
        next
    }

    pub(crate) fn due(&self) -> bool {
        self.every > 0 && self.done >= self.next_at
    }

    fn ticks(&self, countdown: u32) -> u64 {
        self.done + u64::from(self.loaded - countdown)
    }
}

/// The untraced run, with a checkpoint about every `every` loop rounds and
/// calls, at a round of a loop of main's or of a fn's.
pub fn record_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
    every: u64,
) -> (Observed, Vec<Checkpoint>) {
    record_last_on_this_thread(program, input, output, limits, every, usize::MAX)
}

/// The same run keeping only the last `keep` checkpoints, so a run's memory
/// stays bounded by `keep` copies of its state however long it runs.
pub fn record_last_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
    every: u64,
    keep: usize,
) -> (Observed, Vec<Checkpoint>) {
    let every = every.max(1);
    let ckpt = Clock {
        every,
        next_at: every,
        keep: keep.max(1),
        ..Clock::default()
    };
    let io = Io::fixed(input);
    let (observed, clock) =
        crate::run_from::<Unobserved>(program, io, output, limits, None, false, ckpt);
    (observed, clock.taken)
}

/// The traced run counting its steps and recording none, with a checkpoint
/// about every `every` loop rounds and calls, each stamped with the steps
/// before it. Past `spread` checkpoints, every other one goes and the rest
/// stay twice as far apart, so they cover the whole run in bounded memory.
pub fn count_on_this_thread(
    program: &Program,
    input: &[u8],
    limits: Limits,
    config: trace::TraceConfig,
    every: u64,
    spread: usize,
) -> (Observed, Vec<Checkpoint>) {
    let every = every.max(1);
    let ckpt = Clock {
        every,
        next_at: every,
        spread: spread.max(1),
        ..Clock::default()
    };
    let config = trace::TraceConfig {
        from_step: Some(usize::MAX),
        last_steps: None,
        ..config
    };
    let tracer = Some(Box::new(trace::Tracer::new(config, program)));
    let io = Io::fixed(input);
    let (observed, clock) = crate::run_from(
        program,
        io,
        &mut std::io::sink(),
        limits,
        tracer,
        false,
        ckpt,
    );
    (observed, clock.taken)
}

/// The run from `from` to its end, on the same program and input: it prints
/// what the whole run printed after `from.printed` bytes, and with `trace`,
/// records what the whole run's trace holds from there on.
pub fn resume_on_this_thread(
    program: &Program,
    input: &[u8],
    from: &Checkpoint,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
) -> Observed {
    let ckpt = Clock {
        resume: Some(from.clone()),
        ..Clock::default()
    };
    let io = Io::fixed(input);
    match trace {
        Some(config) => {
            let tracer = Some(Box::new(trace::Tracer::new(config, program)));
            crate::run_from(program, io, output, limits, tracer, false, ckpt).0
        }
        None => crate::run_from::<Unobserved>(program, io, output, limits, None, false, ckpt).0,
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    #[cfg(not(target_arch = "wasm32"))]
    /// Before a VM call at `pc` of `chunk` to `fn_id`.
    #[inline(always)]
    pub(crate) fn suspend(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        base_i: usize,
        regs_v: &[Value],
        callee: Body,
        base: usize,
    ) {
        if self.ckpt.on() {
            self.suspend_recording(chunk, pc, base_i, regs_v, callee, base);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cold]
    #[inline(never)]
    fn suspend_recording(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        base_i: usize,
        regs_v: &[Value],
        callee: Body,
        base: usize,
    ) {
        self.ckpt.frames.push(Suspended {
            caller: chunk as *const Chunk<'p> as *const (),
            pc: pc as u32,
            base_i,
            regs_v: regs_v.as_ptr(),
            n_v: regs_v.len(),
            base,
            callee,
            depth: self.call_frames.len() - 1,
            doms: self.saved_domains.len(),
            span: chunk.spans[pc],
            snap: Snap::Unknown,
            volatile: chunk.has_by_ref_closure,
            fallback: None,
        });
    }

    /// Before the fallback op at `pc` of `chunk` runs its statement on the
    /// tree-walker; what it returns goes back to `leave_fallback`.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    pub(crate) fn enter_fallback(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        regs_i: &[i64],
        regs_v: &[Value],
    ) -> Option<Fallback> {
        if !self.ckpt.on() {
            return None;
        }
        let here = Fallback {
            caller: chunk as *const Chunk<'p> as *const (),
            pc: pc as u32,
            regs_i: regs_i.as_ptr(),
            n_i: regs_i.len(),
            regs_v: regs_v.as_ptr(),
            n_v: regs_v.len(),
            base: self.base,
            depth: self.call_frames.len(),
            volatile: chunk.has_by_ref_closure,
        };
        self.ckpt.fallback.replace(here)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    pub(crate) fn leave_fallback(&mut self, outer: Option<Fallback>) {
        if self.ckpt.on() {
            self.ckpt.fallback = outer;
        }
    }

    /// Before the tree-walker runs fn `fn_id` on the VM, from the call at
    /// `span`, its caller's frame at `base`, with `doms` saved domains: when
    /// that call is in a statement a VM caller falls back to, that caller
    /// waits on it. Says whether it does, for `unsuspend`.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    pub(crate) fn suspend_fallback(
        &mut self,
        fn_id: FnId,
        span: Span,
        base: usize,
        doms: usize,
    ) -> bool {
        let Some(at) = self.ckpt.fallback else {
            return false;
        };
        if at.depth + 1 != self.call_frames.len() || at.base != base {
            return false;
        }
        self.ckpt.frames.push(Suspended {
            caller: at.caller,
            pc: at.pc,
            base_i: 0,
            regs_v: at.regs_v,
            n_v: at.n_v,
            base,
            callee: Body::Fn(fn_id),
            depth: at.depth,
            doms,
            span,
            snap: Snap::Unknown,
            volatile: at.volatile,
            fallback: Some((at.regs_i, at.n_i)),
        });
        true
    }

    /// The tree-walker's call to fn `fn_id` on the VM, which a VM caller
    /// falling back to the statement making it waits on.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn run_fn_vm_waited(
        &mut self,
        compiled: &CompiledProgram<'p>,
        fn_id: FnId,
        span: Span,
        base: usize,
        doms: usize,
    ) -> Res<Flow> {
        let waits = self.suspend_fallback(fn_id, span, base, doms);
        let flow = self.run_fn_vm_from_stack(compiled, fn_id).map(Flow::Return);
        if waits {
            self.unsuspend();
        }
        flow
    }

    /// After the tree-walker's call `e` returned `value`.
    #[inline(always)]
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub(crate) fn keep_called(&mut self, e: &cpl_ir::Expr, value: &Value) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(at) = self.ckpt.fallback {
            if at.depth == self.call_frames.len() {
                let e = e as *const cpl_ir::Expr as usize;
                self.ckpt.keep(at.depth, at.caller, e, value.clone());
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// After that call.
    #[inline(always)]
    pub(crate) fn unsuspend(&mut self) {
        if self.ckpt.on() {
            self.unsuspend_recording();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cold]
    #[inline(never)]
    fn unsuspend_recording(&mut self) {
        self.ckpt.frames.pop();
        self.ckpt.known = self.ckpt.known.min(self.ckpt.frames.len());
    }

    /// After the call op at `pc` of `chunk` put what it returned in `regs_i`
    /// or `regs_v`.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    pub(crate) fn keep_result(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        regs_i: &[i64],
        regs_v: &[Value],
    ) {
        if self.ckpt.on() && !chunk.keeps.is_empty() {
            self.keep_result_recording(chunk, pc, regs_i, regs_v);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cold]
    #[inline(never)]
    fn keep_result_recording(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        regs_i: &[i64],
        regs_v: &[Value],
    ) {
        let Ok(i) = chunk.keeps.binary_search_by_key(&(pc as u32), |k| k.0) else {
            return;
        };
        let (_, loc, e) = &chunk.keeps[i];
        let value =
            (loc.as_ref()).map_or(Value::Unit, |loc| crate::vm::read_slot(loc, regs_i, regs_v));
        let caller = chunk as *const Chunk<'p> as *const ();
        self.ckpt.keep(self.call_frames.len(), caller, *e, value);
    }

    /// At the jump back at `pc` of `chunk`, once a checkpoint is due.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn take_checkpoint(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        pc: usize,
        base_i: usize,
        vm_regs_i: &[i64],
        regs_v: &[Value],
    ) {
        if let Some(taken) = self.checkpoint_here(compiled, chunk, pc, base_i, vm_regs_i, regs_v) {
            self.ckpt.next_at = taken.ticks + self.ckpt.every;
            self.ckpt.soon = SOON;
            self.countdown = self.ckpt.load_instead(crate::stop::EVERY);
            self.ckpt.taken.push(taken);
            if self.ckpt.taken.len() > self.ckpt.keep {
                self.ckpt.taken.remove(0);
            }
            if self.ckpt.taken.len() > self.ckpt.spread {
                let mut left = self.ckpt.taken.len();
                self.ckpt.taken.retain(|_| {
                    left -= 1;
                    left.is_multiple_of(2)
                });
                self.ckpt.every *= 2;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn checkpoint_here(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        pc: usize,
        base_i: usize,
        vm_regs_i: &[i64],
        regs_v: &[Value],
    ) -> Option<Checkpoint> {
        if !self.fed_loops.is_empty() || self.peer.is_some() || self.program.interactive {
            return None;
        }
        let edge = chunk.back_edges.iter().find(|e| e.pc as usize == pc)?;
        let depth = self.ckpt.frames.len();
        if self.call_frames.len() != depth {
            return None;
        }
        for k in self.ckpt.known..depth {
            self.ckpt.frames[k].snap = self.snap_caller(compiled, vm_regs_i, k);
        }
        let frames = &self.ckpt.frames;
        self.ckpt.known = (self.ckpt.known..depth)
            .find(|&k| frames[k].volatile)
            .unwrap_or(depth);
        let (up, caller, caller_fn, span, doms) = match self.ckpt.frames.last() {
            None => (None, &compiled.main, Body::Main, Span::default(), 0),
            Some(s) => {
                let Snap::Yes(up) = &s.snap else {
                    return None;
                };
                (
                    Some(up.clone()),
                    chunk_of(compiled, s.callee)?,
                    s.callee,
                    s.span,
                    s.doms,
                )
            }
        };
        if !std::ptr::eq(chunk, caller) {
            return None;
        }
        let regs_i = &vm_regs_i[base_i..base_i + chunk.num_i];
        let at = (self.base, span, doms);
        let frame = self.frame_at(
            chunk,
            caller_fn,
            at,
            regs_i,
            regs_v,
            &edge.loops,
            None,
            false,
        )?;
        Some(Checkpoint {
            ticks: self.ckpt.ticks(self.countdown),
            printed: self.ckpt.printed + self.out.len() as u64,
            steps: self.tracer.as_ref().map_or(0, |t| t.steps()),
            depth,
            frames: Rc::new(Chain { frame, up }),
            globals: self.globals.clone(),
            domains: self.domains.clone(),
            saved_domains: self.saved_domains.clone(),
            input_at: self.input.cursor(),
            rng_state: self.rng_state,
            queries: self.queries,
            case: self.current_case.map(|case| (case, self.ckpt.cases_left)),
        })
    }

    /// The frame of the `k`th VM caller waiting on its call, with the frames
    /// around it: what every checkpoint until that call returns shares, since
    /// a caller's locals stay as they are while it waits.
    #[cfg(not(target_arch = "wasm32"))]
    fn snap_caller(&self, compiled: &CompiledProgram<'p>, vm_regs_i: &[i64], k: usize) -> Snap {
        let s = &self.ckpt.frames[k];
        let (caller, caller_fn, span, doms, up) = match k.checked_sub(1) {
            None => (&compiled.main, Body::Main, Span::default(), 0, None),
            Some(outer) => {
                let o = &self.ckpt.frames[outer];
                let Snap::Yes(up) = &o.snap else {
                    return Snap::No;
                };
                let Some(caller) = chunk_of(compiled, o.callee) else {
                    return Snap::No;
                };
                (caller, o.callee, o.span, o.doms, Some(up.clone()))
            }
        };
        if s.depth != k || !std::ptr::eq(s.caller, caller as *const Chunk<'p> as *const ()) {
            return Snap::No;
        }
        let lends = match s.callee {
            Body::Fn(id) => self.program.fns[id].params.iter().any(|p| p.mutable),
            Body::Closure(cid) => !self.program.closures[cid].muts.is_empty(),
            Body::Main => true,
        };
        if lends {
            return Snap::No;
        }
        let Some(site) = (caller.call_sites.iter()).find(|c| c.pc == s.pc && c.call.span == s.span)
        else {
            return Snap::No;
        };
        // SAFETY: a caller waiting on its call neither moves nor touches
        // its registers until the call returns.
        let regs_i = match s.fallback {
            Some((at, n)) => unsafe { std::slice::from_raw_parts(at, n) },
            None => &vm_regs_i[s.base_i..s.base_i + caller.num_i],
        };
        let regs_v = unsafe { std::slice::from_raw_parts(s.regs_v, s.n_v) };
        let at = (s.base, span, doms);
        let site_at = Some((site.stmt, site.call));
        // A caller with closures keeps its locals on the stack during calls
        // to fns, where a closure the callee runs may change them. A closure
        // the VM runs takes its captures by value, so during a call to one
        // they stay in the caller's registers.
        let to_fn = !matches!(s.callee, Body::Closure(_));
        let from_stack = (caller.has_by_ref_closure && to_fn) || s.fallback.is_some();
        let Some(mut frame) = self.frame_at(
            caller,
            caller_fn,
            at,
            regs_i,
            regs_v,
            &site.loops,
            site_at,
            from_stack,
        ) else {
            return Snap::No;
        };
        let kept = self.ckpt.kept.get(k).map_or(&[][..], Vec::as_slice);
        frame.before = (site.before.iter())
            .filter_map(|&e| {
                let e = e as *const cpl_ir::Expr as usize;
                let (.., value) = kept.iter().find(|x| x.0 == s.caller && x.1 == e)?;
                Some((e, value.clone()))
            })
            .collect();
        Snap::Yes(Rc::new(Chain { frame, up }))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// One frame: `chunk`'s, run for `callee` on the stack
    /// at `base`, in `loops`, waiting on `site` unless it is the innermost.
    #[allow(clippy::too_many_arguments)]
    fn frame_at(
        &self,
        chunk: &Chunk<'p>,
        callee: Body,
        (base, span, doms): (usize, Span, usize),
        regs_i: &[i64],
        regs_v: &[Value],
        loops: &[LoopReg],
        site: Option<(&'p Stmt, &'p cpl_ir::Expr)>,
        from_stack: bool,
    ) -> Option<FrameAt> {
        let (n, body) = self.body_of(callee);
        let stacked = callee == Body::Main || chunk.needs_stack;
        let mut locals = match stacked {
            true => self.stack.get(base..base + n)?.to_vec(),
            false => vec![Value::Unit; n],
        };
        if from_stack && !stacked {
            return None;
        }
        if !from_stack {
            // A register reads 0 before its local is declared, where the
            // tree-walker has none: a local with no value yet is taken as
            // undeclared unless it surely is declared by now.
            let mut surely = vec![false; n];
            match callee {
                Body::Main => {}
                Body::Fn(id) => (surely.iter_mut())
                    .take(self.program.fns[id].params.len())
                    .for_each(|d| *d = true),
                Body::Closure(cid) => {
                    let def = &self.program.closures[cid];
                    let given = def.captures.iter().map(|cap| cap.inner);
                    def.params
                        .iter()
                        .copied()
                        .chain(given)
                        .for_each(|k| surely[k] = true);
                }
            }
            let site_stmt = site.map(|s| s.0 as *const Stmt as usize);
            let ids: Vec<LoopId> = loops.iter().map(LoopReg::id).collect();
            declared(body, &ids, site_stmt, &mut surely);
            for (k, (held, loc)) in locals.iter_mut().zip(&chunk.slot_map).enumerate() {
                if stacked && crate::vm::undeclared(chunk.traced, loc, held) {
                    continue;
                }
                *held = match unset(loc, regs_i, regs_v) && !surely[k] {
                    true => Value::Unit,
                    false => crate::vm::read_slot(loc, regs_i, regs_v),
                };
            }
        }
        // A place a frame names, or a closure taking one by reference, is
        // where the run left it on the stack, not where a run from here does.
        let names_place = |v: &Value| match v {
            Value::Alias(..) | Value::Part(_) => true,
            Value::Closure(_, caps) => (caps.iter()).any(|c| {
                use crate::value::CaptureVal;
                matches!(c, CaptureVal::StackRef(_) | CaptureVal::PartRef(_))
            }),
            _ => false,
        };
        if locals.iter().any(names_place) {
            return None;
        }
        let mut held = Vec::new();
        let mut at = Vec::with_capacity(loops.len());
        for l in loops {
            at.push(match *l {
                LoopReg::While(id) => LoopAt::While(id),
                LoopReg::Range { id, r_var, r_hi } => LoopAt::Range {
                    id,
                    v: regs_i[r_var as usize],
                    hi: regs_i[r_hi as usize],
                },
                LoopReg::Seq { id, r_idx, held: h } => {
                    let seq = match h {
                        SeqHeld::Local(v) => regs_v.get(v as usize)?,
                        SeqHeld::Global(g) => self.globals.get(g as usize)?,
                    };
                    let walkable = matches!(
                        seq,
                        Value::List(_)
                            | Value::ListView(_)
                            | Value::Array(_)
                            | Value::Str(_)
                            | Value::StrView(_)
                    );
                    if !walkable {
                        return None;
                    }
                    held.push(seq.clone());
                    LoopAt::Seq {
                        id,
                        k: regs_i[r_idx as usize],
                        held: held.len() - 1,
                    }
                }
            });
        }
        let loops = at;
        let site =
            site.map(|(s, c)| (s as *const Stmt as usize, c as *const cpl_ir::Expr as usize));
        if !locates(body, &loops, site.map(|s| s.0)) {
            return None;
        }
        Some(FrameAt {
            body: callee,
            locals,
            loops,
            site,
            span,
            doms,
            before: Vec::new(),
            held,
        })
    }

    /// The run from `from` on.
    pub(crate) fn resume(&mut self, from: Checkpoint) -> Res<()> {
        self.globals = from.globals;
        self.domains = from.domains;
        self.saved_domains = from.saved_domains;
        self.input.rewind(from.input_at);
        self.rng_state = from.rng_state;
        self.queries = from.queries;
        self.ckpt.printed = from.printed;
        let mut frames = Vec::with_capacity(from.depth + 1);
        let mut at = Some(&from.frames);
        while let Some(chain) = at {
            frames.push(chain.frame.clone());
            at = chain.up.as_ref();
        }
        frames.reverse();
        self.stack = frames[0].locals.clone();
        self.base = 0;
        if let Some((case, left)) = from.case {
            self.current_case = Some(case);
            if let Some(tracer) = &mut self.tracer {
                tracer.resume_case(case);
            }
            self.resume_frame(&frames, 0)?;
            return self.run_cases(left, case);
        }
        self.resume_frame(&frames, 0).map(|_| ())
    }

    /// Frame `k` of `frames`, from where it was to its end.
    fn resume_frame(&mut self, frames: &[FrameAt], k: usize) -> Res<Flow> {
        let body = self.body_of(frames[k].body).1;
        self.resume_block(body, &frames[k].loops, frames, k)
    }

    /// How many locals `body` has, and its block.
    fn body_of(&self, body: Body) -> (usize, &'p Block) {
        let program = self.program;
        match body {
            Body::Main => (program.main_locals.len(), &program.main),
            Body::Fn(id) => (program.fns[id].locals.len(), &program.fns[id].body),
            Body::Closure(cid) => {
                let def = &program.closures[cid];
                (def.locals.len(), &def.body)
            }
        }
    }

    /// The closure call frame `k` of `frames` makes, from where it was, as
    /// `call_lending` makes it; what it returns.
    fn resume_closure(
        &mut self,
        frames: &[FrameAt],
        k: usize,
        cid: cpl_ir::ClosureId,
    ) -> Res<Value> {
        let frame = &frames[k];
        let program = self.program;
        let base = self.stack.len();
        self.stack.extend(frame.locals.iter().cloned());
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&program.closures[cid].locals);
        self.call_frames.push(("<closure>", frame.span));
        let in_closure = std::mem::replace(&mut self.in_closure, true);
        let flow = self.resume_frame(frames, k);
        while self.saved_domains.len() > frame.doms {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        self.frame_locals.pop();
        self.base = saved;
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        Ok(match flow {
            Flow::Return(value) | Flow::Give(value) => value,
            _ => Value::Unit,
        })
    }

    /// The call frame `k` of `frames` makes, from where it was; what it returns.
    fn resume_callee(&mut self, frames: &[FrameAt], k: usize) -> Res<Value> {
        let frame = &frames[k];
        let id = match frame.body {
            Body::Fn(id) => id,
            Body::Closure(cid) => return self.resume_closure(frames, k, cid),
            Body::Main => unreachable!("a callee frame is a fn's or a closure's"),
        };
        let f = &self.program.fns[id];
        let span = frame.span;
        let base = self.stack.len();
        self.stack.extend(frame.locals.iter().cloned());
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let entered = self.tracer.as_mut().map(|tracer| tracer.enter_fn(&f.name));
        let in_closure = std::mem::replace(&mut self.in_closure, false);
        let mut flow = self.resume_frame(frames, k);
        while self.saved_domains.len() > frame.doms {
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
                    Ok(Flow::Return(value)) if tracer.keeps_call() && f.ret != cpl_ir::Ty::Unit => {
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
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }

    fn resume_block(
        &mut self,
        block: &'p Block,
        at: &[LoopAt],
        frames: &[FrameAt],
        k: usize,
    ) -> Res<Flow> {
        let Some(mark) = mark(at, frames[k].site.map(|s| s.0)) else {
            return trap(Span::default(), "the checkpoint is at no loop and no call");
        };
        let Some(i) = block.stmts.iter().position(|s| holds(s, mark)) else {
            return trap(Span::default(), "the checkpoint does not match the program");
        };
        let stmt = &block.stmts[i];
        let mut flow = match mark {
            Mark::Site(site) if stmt as *const Stmt as usize == site => {
                self.resume_call(stmt, frames, k)?
            }
            _ => self.resume_stmt(stmt, at, frames, k)?,
        };
        for stmt in &block.stmts[i + 1..] {
            if !matches!(flow, Flow::Next) {
                break;
            }
            flow = self.stmt(stmt)?;
        }
        Ok(flow)
    }

    fn resume_stmt(
        &mut self,
        stmt: &'p Stmt,
        at: &[LoopAt],
        frames: &[FrameAt],
        k: usize,
    ) -> Res<Flow> {
        let inside = at.len() > 1 || frames[k].site.is_some();
        let rest = at.get(1..).unwrap_or(&[]);
        match (stmt, at.first()) {
            (Stmt::If(_, then, otherwise), _) => {
                let mark = mark(at, frames[k].site.map(|s| s.0)).expect("resume_block found one");
                let branch = if block_holds(then, mark) {
                    then
                } else {
                    otherwise
                };
                self.resume_block(branch, at, frames, k)
            }
            (Stmt::While(id, _, body), Some(LoopAt::While(_))) => {
                if inside {
                    if let Some(left) = left_loop(self.resume_block(body, rest, frames, k)?, *id) {
                        return Ok(left);
                    }
                }
                self.stmt(stmt)
            }
            (Stmt::For(id, _, iter, body), Some(&LoopAt::Range { v, hi, .. })) => {
                let mut from = v;
                if inside {
                    if let Some(left) = left_loop(self.resume_block(body, rest, frames, k)?, *id) {
                        return Ok(left);
                    }
                    if matches!(iter, Iter::Range { closed: true, .. }) && v == hi {
                        return Ok(Flow::Next);
                    }
                    from = v + 1;
                }
                self.ckpt.range_from = Some((from, hi));
                self.stmt(stmt)
            }
            (
                Stmt::For(id, _, Iter::Seq(_, rev), body),
                Some(&LoopAt::Seq { k: at_k, held, .. }),
            ) => {
                let mut from = at_k;
                if inside {
                    if let Some(left) = left_loop(self.resume_block(body, rest, frames, k)?, *id) {
                        return Ok(left);
                    }
                    from = if *rev { at_k - 1 } else { at_k + 1 };
                }
                self.ckpt.seq_from = Some((frames[k].held[held].clone(), from));
                self.stmt(stmt)
            }
            _ => trap(Span::default(), "the checkpoint does not match the program"),
        }
    }

    /// `stmt`, whose call frame `k + 1` is: `stmt` again, untraced up to
    /// that call, which goes on from where it was.
    fn resume_call(&mut self, stmt: &'p Stmt, frames: &[FrameAt], k: usize) -> Res<Flow> {
        let (_, call) = frames[k].site.expect("a caller frame waits on a call");
        self.ckpt.pending = Some((call, frames[k + 1..].to_vec()));
        self.ckpt.returned = frames[k].before.clone();
        let flow = self.stmt(stmt);
        self.ckpt.returned.clear();
        if self.ckpt.pending.take().is_some() {
            return trap(Span::default(), "the checkpoint does not match the program");
        }
        flow
    }

    /// What a `for` over `seq` walks, and the round it starts from when a
    /// run from a checkpoint goes on in it: the sequence as it was when the
    /// loop began.
    pub(crate) fn seq_held(&mut self, seq: &'p cpl_ir::Expr) -> Res<(Value, Option<i64>)> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some((held, from)) = self.ckpt.seq_from.take() {
            return Ok((held, Some(from)));
        }
        Ok((self.eval(seq)?, None))
    }

    /// The call `e` to `id`, or the rest of it when a run from a checkpoint
    /// waits on it.
    #[inline]
    pub(crate) fn call_or_resume(
        &mut self,
        e: &cpl_ir::Expr,
        id: FnId,
        args: &'p [cpl_ir::Arg],
        span: Span,
    ) -> Res<Value> {
        if self.ckpt.waits() {
            if let Some(value) = self.resume_pending(e)? {
                return Ok(value);
            }
        }
        let value = self.call(id, args, span)?;
        self.keep_called(e, &value);
        Ok(value)
    }

    /// At the call `e`: when it is the call a run from a checkpoint waits
    /// on, that call from where it was, and what it returns.
    #[cold]
    pub(crate) fn resume_pending(&mut self, e: &cpl_ir::Expr) -> Res<Option<Value>> {
        let at = e as *const cpl_ir::Expr as usize;
        if let Some(i) = self.ckpt.returned.iter().position(|d| d.0 == at) {
            return Ok(Some(self.ckpt.returned.swap_remove(i).1));
        }
        if !matches!(&self.ckpt.pending, Some((call, _)) if *call == at) {
            return Ok(None);
        }
        let (_, frames) = self.ckpt.pending.take().unwrap();
        self.ckpt.returned.clear();
        self.resume_callee(&frames, 0).map(Some)
    }
}

/// The chunk the VM runs `body` on, if it does.
#[cfg(not(target_arch = "wasm32"))]
fn chunk_of<'c, 'p>(compiled: &'c CompiledProgram<'p>, body: Body) -> Option<&'c Chunk<'p>> {
    match body {
        Body::Main => Some(&compiled.main),
        Body::Fn(id) => compiled.fns[id].as_ref(),
        Body::Closure(cid) => compiled.closures[cid].as_ref(),
    }
}

/// What a frame's position is found by: its outermost loop, or with none,
/// the statement whose call it waits on.
#[derive(Clone, Copy)]
enum Mark {
    Loop(LoopId),
    Site(usize),
}

fn mark(at: &[LoopAt], site: Option<usize>) -> Option<Mark> {
    match at.first() {
        Some(first) => Some(Mark::Loop(first.id())),
        None => site.map(Mark::Site),
    }
}

/// How a round's `flow` leaves loop `id`, if it does.
fn left_loop(flow: Flow, id: LoopId) -> Option<Flow> {
    match flow {
        Flow::Next | Flow::Continue(None) => None,
        Flow::Continue(Some(target)) if target == id => None,
        Flow::Break(None) => Some(Flow::Next),
        Flow::Break(Some(target)) if target == id => Some(Flow::Next),
        other => Some(other),
    }
}

fn holds(stmt: &Stmt, mark: Mark) -> bool {
    if let Mark::Site(site) = mark {
        if stmt as *const Stmt as usize == site {
            return true;
        }
    }
    match stmt {
        Stmt::If(_, then, otherwise) => block_holds(then, mark) || block_holds(otherwise, mark),
        Stmt::While(own, _, body) | Stmt::For(own, _, _, body) => {
            matches!(mark, Mark::Loop(id) if *own == id) || block_holds(body, mark)
        }
        _ => false,
    }
}

fn block_holds(block: &Block, mark: Mark) -> bool {
    block.stmts.iter().any(|s| holds(s, mark))
}

/// Whether `at`, then `site`, names a place in `block` a run can start
/// again from.
fn locates(block: &Block, at: &[LoopAt], site: Option<usize>) -> bool {
    let Some(mark) = mark(at, site) else {
        return false;
    };
    let Some(stmt) = block.stmts.iter().find(|s| holds(s, mark)) else {
        return false;
    };
    match (stmt, at.first()) {
        (_, None) if stmt as *const Stmt as usize == site.unwrap_or(0) => true,
        (Stmt::If(_, then, otherwise), _) => locates(
            if block_holds(then, mark) {
                then
            } else {
                otherwise
            },
            at,
            site,
        ),
        (Stmt::While(id, _, body), Some(&LoopAt::While(want))) => {
            *id == want && locates_inside(body, &at[1..], site)
        }
        (
            Stmt::For(
                id,
                _,
                Iter::Range {
                    step: None,
                    rev: false,
                    ..
                },
                body,
            ),
            Some(&LoopAt::Range { id: want, .. }),
        ) => *id == want && locates_inside(body, &at[1..], site),
        (Stmt::For(id, _, Iter::Seq(..), body), Some(&LoopAt::Seq { id: want, .. })) => {
            *id == want && locates_inside(body, &at[1..], site)
        }
        _ => false,
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Marks in `out` the locals surely declared by the place `at` (loop ids
/// only), then `site`, in `block`: those the statements before it in each
/// block on the way declare, and the variables of the loops it is in.
fn declared(block: &Block, at: &[LoopId], site: Option<usize>, out: &mut [bool]) {
    let mark = match at.first() {
        Some(&id) => Mark::Loop(id),
        None => match site {
            Some(site) => Mark::Site(site),
            None => return,
        },
    };
    let Some(i) = block.stmts.iter().position(|s| holds(s, mark)) else {
        return;
    };
    for stmt in &block.stmts[..i] {
        if let Stmt::Let(slot, _) = stmt {
            out[*slot] = true;
        }
    }
    let inside = |body: &Block, out: &mut [bool]| {
        if at.len() > 1 || site.is_some() {
            declared(body, &at[1..], site, out);
        }
    };
    match &block.stmts[i] {
        Stmt::If(_, then, otherwise) => {
            let branch = if block_holds(then, mark) {
                then
            } else {
                otherwise
            };
            declared(branch, at, site, out);
        }
        Stmt::While(_, _, body) if !at.is_empty() => inside(body, out),
        Stmt::For(_, slot, _, body) if !at.is_empty() => {
            out[*slot] = true;
            inside(body, out);
        }
        _ => {}
    }
}

/// Whether the registers at `loc` hold nothing yet.
#[cfg(not(target_arch = "wasm32"))]
fn unset(loc: &SlotLoc, regs_i: &[i64], regs_v: &[Value]) -> bool {
    match *loc {
        SlotLoc::V(r) => matches!(regs_v[r as usize], Value::Unit),
        SlotLoc::I(r, _) => regs_i[r as usize] == 0,
        SlotLoc::OptI { val, some, .. } => regs_i[val as usize] == 0 && regs_i[some as usize] == 0,
        SlotLoc::SmallTuple { base, len, .. } => regs_i
            [base as usize..(base + len as u16) as usize]
            .iter()
            .all(|&r| r == 0),
    }
}

/// Inside a loop's body: the innermost frame may stop at the loop itself.
fn locates_inside(body: &Block, rest: &[LoopAt], site: Option<usize>) -> bool {
    (rest.is_empty() && site.is_none()) || locates(body, rest, site)
}

/// The positions a `for` over `n` items walks, backwards when `rev`, from
/// `from` on (by default the first).
pub(crate) fn rounds(n: usize, rev: bool, from: Option<i64>) -> impl Iterator<Item = usize> {
    let last = n as i64 - 1;
    let (first, count) = match (rev, from) {
        (false, None) => (0, n as i64),
        (true, None) => (last, n as i64),
        (false, Some(f)) => (f, n as i64 - f),
        (true, Some(f)) => (f.min(last), f.min(last) + 1),
    };
    (0..count.max(0)).map(move |i| (if rev { first - i } else { first + i }) as usize)
}
