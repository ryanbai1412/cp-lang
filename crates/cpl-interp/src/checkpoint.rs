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

use crate::observe::{Observer, Unobserved};
use crate::value::Value;
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
pub(crate) struct FrameAt {
    pub(crate) body: Body,
    pub(crate) locals: Vec<Value>,
    loops: Vec<LoopAt>,
    /// The statement, then the call in it, the frame waits on, by address;
    /// `None` for the innermost frame.
    site: Option<(usize, usize)>,
    /// Where the frame was called from.
    pub(crate) span: Span,
    /// How many saved domains there were before the call.
    pub(crate) doms: usize,
    /// What the calls before `site` in its statement returned, by address.
    before: Vec<(usize, Value)>,
    /// The sequences its `for` loops walk.
    held: Vec<Value>,
}

/// Whose body a frame runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Body {
    Main,
    /// What is above `@!multitest`, run once before the first case.
    Setup,
    Fn(FnId),
    /// A `yields` fn feeding a loop's body as it yields.
    Fed(FnId),
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
    /// The innermost frame, and up from it the frames that called it; none
    /// once the setup above `@!multitest` is done, before the cases.
    frames: Option<Rc<Chain>>,
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

/// Where a frame's values are: its registers, the loops it is in, and
/// whether its locals are to be read from the stack instead.
#[derive(Clone, Copy)]
struct Held<'a> {
    regs_i: &'a [i64],
    regs_v: &'a [Value],
    loops: &'a [LoopReg],
    from_stack: bool,
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
    /// At the end of a recorded run, every byte of input it was given.
    pub(crate) read: Vec<u8>,
    /// The test cases still to run after the current one.
    pub(crate) cases_left: Option<i64>,
    /// In a run from a checkpoint, the call (by address) the caller waits
    /// on, and the frames from its callee in.
    pending: Option<(usize, Vec<FrameAt>)>,
    /// In a run from a checkpoint, what the calls before the one waited on
    /// returned, by address.
    returned: Vec<(usize, Value)>,
    /// In a run from a checkpoint, the `mut` parameters of the fn the call
    /// waited on left, for its caller to take back.
    pub(crate) lent_back: Vec<Value>,
    /// While checkpoints are on, per call depth, what the last call at each
    /// kept call (by caller chunk and call address) returned.
    kept: Vec<Vec<Kept>>,
    /// While checkpoints are on, the VM statement running on the tree-walker.
    fallback: Option<Fallback>,
    pub(crate) printed: u64,
    pub(crate) resume: Option<Checkpoint>,
    /// How many more looks at the clock may come one tick apart while a
    /// checkpoint is due.
    soon: u32,
    /// Take checkpoints only before the run reads or prints.
    before_input: bool,
}

/// How many looks one tick apart a due checkpoint gets, after each one
/// taken.
const SOON: u32 = 4096;

impl Clock {
    /// Whether a caller waiting on a call, or running a statement on the
    /// tree-walker, has `regs_v` as its registers.
    pub(crate) fn waits_on(&self, regs_v: &[Value]) -> bool {
        let at = regs_v.as_ptr();
        let held = |p: *const Value, n: usize| n > 0 && std::ptr::eq(p, at);
        self.frames.iter().any(|s| held(s.regs_v, s.n_v))
            || self
                .fallback
                .as_ref()
                .is_some_and(|f| held(f.regs_v, f.n_v))
    }

    /// Whether a call made in the running fallback still waits, reading
    /// that fallback's integer registers.
    fn waits_in_fallback(&self) -> bool {
        let Some(f) = self.fallback else {
            return false;
        };
        let reads = |s: &Suspended| s.fallback.is_some_and(|(at, _)| std::ptr::eq(at, f.regs_i));
        self.frames.iter().any(reads)
    }
}

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
            read: Vec::new(),
            cases_left: None,
            pending: None,
            returned: Vec::new(),
            lent_back: Vec::new(),
            kept: Vec::new(),
            fallback: None,
            printed: 0,
            resume: None,
            soon: SOON,
            before_input: false,
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

    /// Whether a run from a checkpoint still waits on a call.
    #[inline]
    pub(crate) fn waits(&self) -> bool {
        self.pending.is_some()
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

/// The checkpoints a run takes, or the one it starts from: what
/// [`observe_on_this_thread`] takes beside a run's trace.
#[derive(Clone, Default)]
pub struct Checkpoints {
    /// Take one about every this many loop rounds and calls, at a round of
    /// a loop of main's or of a fn's; 0 takes none.
    pub every: u64,
    /// Keep only the last this many, so a run's memory stays bounded by
    /// that many copies of its state however long it runs; 0 keeps all.
    pub keep: usize,
    /// Past this many, every other one goes and the rest stay twice as far
    /// apart, so they cover the whole run in bounded memory; 0 never thins.
    pub spread: usize,
    /// Start here instead of at the program's start, on the same program
    /// and input: the run prints what the whole run printed after
    /// `from.printed` bytes, and a trace holds what the whole run's trace
    /// holds from there on.
    pub from: Option<Checkpoint>,
    /// Take them only while the run has read, looked at and printed
    /// nothing: a run from one of those may be given any input.
    pub before_input: bool,
}

impl Checkpoints {
    /// One about every `every` loop rounds and calls, the last `keep` kept.
    pub fn taking(every: u64, keep: usize) -> Checkpoints {
        Checkpoints {
            every: every.max(1),
            keep,
            ..Checkpoints::default()
        }
    }

    /// The last one before the run first reads or prints, about every
    /// `every` loop rounds and calls: where a run on any input can start,
    /// past what the program works out before it reads.
    pub fn before_input(every: u64) -> Checkpoints {
        Checkpoints {
            before_input: true,
            ..Checkpoints::taking(every, 1)
        }
    }

    /// The run from `from` to its end.
    pub fn resuming(from: &Checkpoint) -> Checkpoints {
        Checkpoints {
            from: Some(from.clone()),
            ..Checkpoints::default()
        }
    }
}

/// What a checkpointed run leaves.
pub struct Checkpointed {
    /// What any run leaves.
    pub observed: Observed,
    /// The checkpoints it took, in order.
    pub taken: Vec<Checkpoint>,
    /// With checkpoints taken, every byte of input the run was given, as an
    /// interactive solution's arrives from its judge: a run from one of them
    /// is given this as its input, and sends nothing to the judge again.
    pub sent: Vec<u8>,
}

/// [`crate::observe_on_this_thread`], taking checkpoints or starting from
/// one as `want` says: the one entry for a run with checkpoints.
pub fn observe_on_this_thread(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
    trace: Option<trace::TraceConfig>,
    want: Checkpoints,
) -> Checkpointed {
    let ckpt = Clock::of(want);
    let tracer = trace.map(|config| trace::Tracer::new(config, program));
    let (observed, clock) = match crate::watch::Watch::of(program, tracer, false) {
        Some(watch) => crate::run_from(program, io, output, limits, Some(watch), ckpt),
        None => crate::run_from::<Unobserved>(program, io, output, limits, None, ckpt),
    };
    let (taken, sent) = clock.into_taken();
    Checkpointed {
        observed,
        taken,
        sent,
    }
}

impl Clock {
    pub(crate) fn of(want: Checkpoints) -> Clock {
        let all = |n: usize| if n == 0 { usize::MAX } else { n };
        Clock {
            every: want.every,
            next_at: want.every,
            keep: all(want.keep),
            spread: all(want.spread),
            resume: want.from,
            before_input: want.before_input,
            ..Clock::default()
        }
    }

    /// The checkpoints taken and, when any were asked for, the input read.
    pub(crate) fn into_taken(self) -> (Vec<Checkpoint>, Vec<u8>) {
        (self.taken, self.read)
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
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

    #[inline(always)]
    pub(crate) fn leave_fallback(&mut self, outer: Option<Fallback>) {
        if self.ckpt.on() {
            debug_assert!(
                !self.ckpt.waits_in_fallback(),
                "a call outlived its fallback"
            );
            self.ckpt.fallback = outer;
        }
    }

    /// Before the tree-walker runs fn `fn_id` on the VM, from the call at
    /// `span`, its caller's frame at `base`, with `doms` saved domains: when
    /// that call is in a statement a VM caller falls back to, that caller
    /// waits on it. Says whether it does, for `unsuspend`.
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
    pub(crate) fn keep_called(&mut self, e: &cpl_ir::Expr, value: &Value) {
        if let Some(at) = self.ckpt.fallback {
            if at.depth == self.call_frames.len() {
                let e = e as *const cpl_ir::Expr as usize;
                self.ckpt.keep(at.depth, at.caller, e, value.clone());
            }
        }
    }

    /// After that call.
    #[inline(always)]
    pub(crate) fn unsuspend(&mut self) {
        if self.ckpt.on() {
            self.unsuspend_recording();
        }
    }

    /// A `yields` fn runs its loop's body, which changes the frame waiting
    /// on it: that frame is read again at the next checkpoint.
    #[inline]
    pub(crate) fn feeding(&mut self) {
        let waiting = self.ckpt.frames.len().saturating_sub(1);
        self.ckpt.known = self.ckpt.known.min(waiting);
    }

    #[cold]
    #[inline(never)]
    fn unsuspend_recording(&mut self) {
        self.ckpt.frames.pop();
        self.ckpt.known = self.ckpt.known.min(self.ckpt.frames.len());
    }

    /// After the call op at `pc` of `chunk` put what it returned in `regs_i`
    /// or `regs_v`.
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
            self.keep_taken(taken);
        }
    }

    /// The setup above `@!multitest` just ran: a checkpoint here resumes
    /// straight into the cases, with none of the setup run again.
    pub(crate) fn setup_done(&mut self) {
        if !self.ckpt.on() || self.fed_bodies > 0 || self.peer.is_some() {
            return;
        }
        if self.ckpt.before_input
            && !(self.input.untouched() && self.ckpt.printed == 0 && self.out.is_empty())
        {
            return;
        }
        // With no loop round or call since the last one, a run from here
        // skips nothing a run from that one does not.
        let ticks = self.ckpt.ticks(self.countdown);
        if ticks == 0 || self.ckpt.taken.last().is_some_and(|t| t.ticks == ticks) {
            return;
        }
        self.keep_taken(Checkpoint {
            ticks,
            printed: self.ckpt.printed + self.out.len() as u64,
            steps: self.tracer.as_ref().map_or(0, |t| t.steps()),
            depth: 0,
            frames: None,
            globals: self.globals.clone(),
            domains: self.domains.clone(),
            saved_domains: self.saved_domains.clone(),
            input_at: self.input.cursor(),
            rng_state: self.rng_state,
            queries: self.queries,
            case: None,
        });
    }

    fn keep_taken(&mut self, taken: Checkpoint) {
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

    fn checkpoint_here(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        pc: usize,
        base_i: usize,
        vm_regs_i: &[i64],
        regs_v: &[Value],
    ) -> Option<Checkpoint> {
        // In a fed loop's body, the frames the fn's yield waits in are not
        // what a resume makes again.
        if self.fed_bodies > 0 || self.peer.is_some() {
            return None;
        }
        if self.ckpt.before_input
            && !(self.input.untouched() && self.ckpt.printed == 0 && self.out.is_empty())
        {
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
            None => {
                let (top, body) = top_of(compiled, chunk as *const Chunk<'p> as *const ());
                (None, top, body, Span::default(), 0)
            }
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
        let held = Held {
            regs_i,
            regs_v,
            loops: &edge.loops,
            from_stack: false,
        };
        let frame = self.frame_at(chunk, caller_fn, at, held, None)?;
        Some(Checkpoint {
            ticks: self.ckpt.ticks(self.countdown),
            printed: self.ckpt.printed + self.out.len() as u64,
            steps: self.tracer.as_ref().map_or(0, |t| t.steps()),
            depth,
            frames: Some(Rc::new(Chain { frame, up })),
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
    fn snap_caller(&self, compiled: &CompiledProgram<'p>, vm_regs_i: &[i64], k: usize) -> Snap {
        let s = &self.ckpt.frames[k];
        let (caller, caller_fn, span, doms, up) = match k.checked_sub(1) {
            None => {
                let (top, body) = top_of(compiled, s.caller);
                (top, body, Span::default(), 0, None)
            }
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
            Body::Fn(_) | Body::Fed(_) => false,
            Body::Closure(cid) => !self.program.closures[cid].muts.is_empty(),
            Body::Main | Body::Setup => true,
        };
        if lends {
            return Snap::No;
        }
        let Some(site) = (caller.call_sites.iter()).find(|c| c.pc == s.pc && c.call.span == s.span)
        else {
            return Snap::No;
        };
        // SAFETY: a caller waiting on its call neither moves nor touches its
        // registers until the call returns. Its integer registers, in a
        // fallback, stay borrowed by the fallback op until `leave_fallback`,
        // which asserts no call waiting in it is left; the others are freed
        // only through `return_regs_v`, which asserts no waiting caller holds
        // them.
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
        let held = Held {
            regs_i,
            regs_v,
            loops: &site.loops,
            from_stack,
        };
        let Some(mut frame) = self.frame_at(caller, caller_fn, at, held, site_at) else {
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

    /// One frame: `chunk`'s, run for `callee` on the stack
    /// at `base`, in `held.loops`, waiting on `site` unless it is the innermost.
    fn frame_at(
        &self,
        chunk: &Chunk<'p>,
        callee: Body,
        (base, span, doms): (usize, Span, usize),
        held: Held,
        site: Option<(&'p Stmt, &'p cpl_ir::Expr)>,
    ) -> Option<FrameAt> {
        let Held {
            regs_i,
            regs_v,
            loops,
            from_stack,
        } = held;
        let (n, body) = self.body_of(callee);
        let stacked = matches!(callee, Body::Main | Body::Setup) || chunk.needs_stack;
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
                Body::Main | Body::Setup => {}
                Body::Fn(id) | Body::Fed(id) => (surely.iter_mut())
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
                LoopReg::Seq {
                    id, r_idx, held: h, ..
                } => {
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
        let Some(chain) = &from.frames else {
            if let Some(tracer) = &mut self.tracer {
                tracer.on(crate::observe::Event::BeginCases);
            }
            return self.run_after_setup();
        };
        let mut frames = Vec::with_capacity(from.depth + 1);
        let mut at = Some(chain);
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
                tracer.on(crate::observe::Event::ResumeCase(case));
            }
            self.resume_frame(&frames, 0)?;
            return self.run_cases(left, case);
        }
        let flow = self.resume_frame(&frames, 0)?;
        if frames[0].body == Body::Setup && !matches!(flow, Flow::Return(_)) {
            return self.run_after_setup();
        }
        Ok(())
    }

    /// Sets `chunk`'s registers for frame `k` of `frames`: its loops as they
    /// were, and the call it waits on as the one pending. Where it starts.
    pub(crate) fn resume_at(
        &mut self,
        chunk: &Chunk<'p>,
        frames: &[FrameAt],
        k: usize,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<usize> {
        let frame = &frames[k];
        let found = match frame.site {
            None => (chunk.back_edges.iter())
                .find(|e| same_loops(&e.loops, &frame.loops))
                .and_then(|e| Some((&e.loops, loop_top(&chunk.ops[e.pc as usize])?))),
            Some((stmt, call)) => (chunk.call_sites.iter())
                .find(|c| {
                    c.start != u32::MAX
                        && c.stmt as *const Stmt as usize == stmt
                        && c.call as *const cpl_ir::Expr as usize == call
                        && same_loops(&c.loops, &frame.loops)
                })
                .map(|c| (&c.loops, c.start as usize)),
        };
        let Some((loops, start)) = found else {
            return trap(
                frame.span,
                cpl_diag::msgs::interp::CHECKPOINT_DOES_NOT_MATCH_PROGRAM.at(&[]),
            );
        };
        for (reg, at) in loops.iter().zip(&frame.loops) {
            match (*reg, *at) {
                (LoopReg::Range { r_var, r_hi, .. }, LoopAt::Range { v, hi, .. }) => {
                    regs_i[r_var as usize] = v;
                    regs_i[r_hi as usize] = hi;
                }
                (
                    LoopReg::Seq {
                        r_idx, r_len, held, ..
                    },
                    LoopAt::Seq { k, held: h, .. },
                ) => {
                    let seq = &frame.held[h];
                    regs_i[r_idx as usize] = k;
                    regs_i[r_len as usize] = seq_len(seq);
                    if let SeqHeld::Local(v) = held {
                        regs_v[v as usize] = seq.clone();
                    }
                }
                _ => {}
            }
        }
        if let Some((_, call)) = frame.site {
            self.ckpt.pending = Some((call, frames[k + 1..].to_vec()));
            self.ckpt.returned = frame.before.clone();
        }
        Ok(start)
    }

    /// At call op `pc` of `chunk` in a run from a checkpoint: what the call
    /// returned when the statement waiting there made it before, or, when
    /// it is the call waited on, what that call returns run from where it
    /// was.
    #[cold]
    pub(crate) fn resumed_call(&mut self, chunk: &Chunk<'p>, pc: usize) -> Res<Option<Value>> {
        if let Some(value) = self.returned_at(chunk, pc) {
            return Ok(Some(value));
        }
        match self.resumed_feed(chunk, pc) {
            Some(frames) => self.resume_callee(&frames, 0, None).map(Some),
            None => Ok(None),
        }
    }

    /// What the call at op `pc` of `chunk` returned, when a run from a
    /// checkpoint makes again a statement that made it before the call it
    /// waits on.
    pub(crate) fn returned_at(&mut self, chunk: &Chunk<'p>, pc: usize) -> Option<Value> {
        for site in chunk.call_sites.iter().filter(|c| c.pc as usize == pc) {
            let at = site.call as *const cpl_ir::Expr as usize;
            if let Some(i) = self.ckpt.returned.iter().position(|d| d.0 == at) {
                return Some(self.ckpt.returned.swap_remove(i).1);
            }
        }
        None
    }

    /// When the fed call at op `pc` of `chunk` is the one a run from a
    /// checkpoint waits on, the frames from its callee in.
    #[cold]
    pub(crate) fn resumed_feed(&mut self, chunk: &Chunk<'p>, pc: usize) -> Option<Vec<FrameAt>> {
        let waited = self.ckpt.pending.as_ref()?.0;
        let waits = (chunk.call_sites.iter())
            .any(|c| c.pc as usize == pc && c.call as *const cpl_ir::Expr as usize == waited);
        if !waits {
            return None;
        }
        self.ckpt.returned.clear();
        self.ckpt.pending.take().map(|p| p.1)
    }

    /// Whether a run from a checkpoint still waits on a call; it no longer
    /// does.
    pub(crate) fn unwait(&mut self) -> bool {
        self.ckpt.returned.clear();
        self.ckpt.pending.take().is_some()
    }

    /// How many locals `body` has, and its block.
    fn body_of(&self, body: Body) -> (usize, &'p Block) {
        let program = self.program;
        match body {
            Body::Main => (program.main_locals.len(), &program.main),
            Body::Setup => (program.main_locals.len(), &program.setup),
            Body::Fn(id) | Body::Fed(id) => (program.fns[id].locals.len(), &program.fns[id].body),
            Body::Closure(cid) => {
                let def = &program.closures[cid];
                (def.locals.len(), &def.body)
            }
        }
    }

    /// The call frame `k` of `frames` makes, from where it was; what it returns.
    pub(crate) fn resume_callee(
        &mut self,
        frames: &[FrameAt],
        k: usize,
        fed: Option<crate::Fed<'p>>,
    ) -> Res<Value> {
        let frame = &frames[k];
        let program = self.program;
        let (locals, name, f) = match frame.body {
            Body::Fn(id) | Body::Fed(id) => {
                let f = &program.fns[id];
                (&f.locals, f.name.as_str(), Some((id, f)))
            }
            Body::Closure(cid) => (&program.closures[cid].locals, "<closure>", None),
            Body::Main | Body::Setup => {
                return trap(
                    frame.span,
                    cpl_diag::msgs::interp::CHECKPOINT_DOES_NOT_MATCH_PROGRAM.at(&[]),
                )
            }
        };
        let span = frame.span;
        let base = self.stack.len();
        self.stack.extend(frame.locals.iter().cloned());
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(locals);
        self.call_frames.push((name, span));
        if let (Some(tracer), Some(_)) = (self.tracer.as_mut(), f) {
            tracer.on(crate::observe::Event::EnterFn(name));
        }
        let feeding = fed.is_some();
        if let Some(fed) = fed {
            self.fed_loops.push(crate::Fed {
                callee: base,
                ..fed
            });
        }
        let in_closure = std::mem::replace(&mut self.in_closure, f.is_none());
        let flow = self.resume_frame(frames, k);
        self.restore_domains(frame.doms);
        self.in_closure = in_closure;
        let flow = self.leave_frame(f.map(|(_, f)| f), span, saved, feeding, flow);
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        if let Some((id, _)) = f {
            self.after_call(id, span);
        }
        Ok(match flow {
            Flow::Return(value) => value,
            Flow::Give(value) if f.is_none() => value,
            _ => Value::Unit,
        })
    }

    /// The call `e` to `id`, kept for a checkpoint the statement takes.
    #[inline]
    pub(crate) fn call_or_resume(
        &mut self,
        e: &'p cpl_ir::Expr,
        id: FnId,
        args: &'p [cpl_ir::Arg],
        span: Span,
    ) -> Res<Value> {
        let value = self.call(id, args, span)?;
        self.keep_called(e, &value);
        Ok(value)
    }
}

/// The top-level chunk at `chunk`: what is above `@!multitest` when it is
/// that, else the file's top-level code.
fn top_of<'c, 'p>(compiled: &'c CompiledProgram<'p>, chunk: *const ()) -> (&'c Chunk<'p>, Body) {
    match &compiled.setup {
        Some(setup) if std::ptr::eq(setup as *const Chunk<'p> as *const (), chunk) => {
            (setup, Body::Setup)
        }
        _ => (&compiled.main, Body::Main),
    }
}

/// The chunk the VM runs `body` on, if it does.
pub(crate) fn chunk_of<'c, 'p>(
    compiled: &'c CompiledProgram<'p>,
    body: Body,
) -> Option<&'c Chunk<'p>> {
    match body {
        Body::Main => Some(&compiled.main),
        Body::Setup => compiled.setup.as_ref(),
        Body::Fn(id) => compiled.fns[id].as_ref(),
        // A traced program feeds a loop from the fn's own chunk.
        Body::Fed(id) => compiled.fed_fns[id].as_ref().or(compiled.fns[id].as_ref()),
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

/// Whether `regs` are the loops `at` was in.
fn same_loops(regs: &[LoopReg], at: &[LoopAt]) -> bool {
    regs.len() == at.len()
        && regs
            .iter()
            .zip(at)
            .all(|(r, a)| r.id() == a.id() && same_kind(r, a))
}

fn same_kind(reg: &LoopReg, at: &LoopAt) -> bool {
    matches!(
        (reg, at),
        (LoopReg::While(_), LoopAt::While(_))
            | (LoopReg::Range { .. }, LoopAt::Range { .. })
            | (LoopReg::Seq { .. }, LoopAt::Seq { .. })
    )
}

/// Where the back edge `op` jumps: the top of its loop.
fn loop_top(op: &crate::vm::Op) -> Option<usize> {
    use crate::vm::Op;
    match *op {
        Op::Jmp { target }
        | Op::ForInc { target, .. }
        | Op::ForIncClosed { target, .. }
        | Op::ForDec { target, .. } => Some(target as usize),
        _ => None,
    }
}

/// How many rounds a `for` over `seq` walks.
fn seq_len(seq: &Value) -> i64 {
    match seq {
        Value::List(items) => items.len() as i64,
        Value::ListView(view) => view.len as i64,
        Value::Array(arr) => arr.len() as i64,
        Value::Str(bytes) => bytes.len() as i64,
        Value::StrView(view) => view.len as i64,
        _ => 0,
    }
}
