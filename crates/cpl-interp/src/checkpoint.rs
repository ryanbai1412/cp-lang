//! Checkpoints: the state of a run at a round of one of main's loops, taken
//! on the untraced run, and a run that starts again from one.
//!
//! Every value is shared and copied on write, so a checkpoint copies main's
//! locals and the globals, not the heap. A run from a checkpoint runs on the
//! tree-walker, traced or not, and reads and prints exactly what the whole
//! run did from there on. Checkpoints are taken only at a jump back to the
//! top of a `while` or a `for v in lo..hi` of main's whose enclosing loops
//! are the same kinds; a run spent inside one call takes none until it
//! returns (an explicit VM frame stack would lift that).

use crate::observe::{Observer, Unobserved};
use crate::value::Value;
use crate::{trace, trap, Flow, Interp, Io, Limits, Observed, Res};
use cpl_diag::Span;
use cpl_ir::{Block, Iter, LoopId, Program, Stmt};
use std::io::Write;

/// Where one of main's loops was, outermost first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoopAt {
    While(LoopId),
    /// In the round with `v`; for the innermost loop, the round about to start.
    Range {
        id: LoopId,
        v: i64,
        hi: i64,
    },
}

impl LoopAt {
    fn id(self) -> LoopId {
        match self {
            LoopAt::While(id) | LoopAt::Range { id, .. } => id,
        }
    }
}

/// A run's state at the start of a round of one of main's loops.
#[derive(Clone)]
pub struct Checkpoint {
    /// Loop rounds and calls before it: the run's step clock.
    pub ticks: u64,
    /// Bytes the run had printed.
    pub printed: u64,
    loops: Vec<LoopAt>,
    stack: Vec<Value>,
    globals: Vec<Value>,
    domains: Vec<(i64, i64)>,
    input_at: usize,
    rng_state: u64,
    queries: i64,
}

/// The step clock, the checkpoints taken, and the one a run starts from.
pub(crate) struct Clock {
    done: u64,
    loaded: u32,
    every: u64,
    next_at: u64,
    taken: Vec<Checkpoint>,
    pub(crate) printed: u64,
    pub(crate) resume: Option<Checkpoint>,
    pub(crate) range_from: Option<(i64, i64)>,
}

impl Default for Clock {
    fn default() -> Clock {
        Clock {
            done: 0,
            loaded: crate::stop::EVERY,
            every: 0,
            next_at: 0,
            taken: Vec::new(),
            printed: 0,
            resume: None,
            range_from: None,
        }
    }
}

impl Clock {
    pub(crate) fn on(&self) -> bool {
        self.every > 0
    }

    /// Counts the ticks just used up and loads `next` more.
    pub(crate) fn reload(&mut self, next: u32) -> u32 {
        self.done += u64::from(self.loaded);
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

/// The untraced run, with a checkpoint at a round of one of main's loops
/// about every `every` loop rounds and calls.
pub fn record_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
    every: u64,
) -> (Observed, Vec<Checkpoint>) {
    let every = every.max(1);
    let ckpt = Clock {
        every,
        next_at: every,
        ..Clock::default()
    };
    let io = Io::fixed(input);
    let (observed, mut clock) =
        crate::run_from::<Unobserved>(program, io, output, limits, None, false, ckpt);
    clock.taken.retain(|c| locates(&program.main, &c.loops));
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
    /// At the jump back at `pc` of main's chunk, once a checkpoint is due.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn take_checkpoint(
        &mut self,
        chunk: &crate::vm::Chunk<'p>,
        pc: usize,
        regs_i: &[i64],
        regs_v: &[Value],
    ) {
        use crate::vm::LoopReg;
        let Some(edge) = chunk.back_edges.iter().find(|e| e.pc as usize == pc) else {
            return;
        };
        if !self.fed_loops.is_empty() || self.peer.is_some() || self.program.interactive {
            return;
        }
        let mut stack = self.stack[..self.program.main_locals.len()].to_vec();
        for (held, loc) in stack.iter_mut().zip(&chunk.slot_map) {
            *held = crate::vm::read_slot(loc, regs_i, regs_v);
        }
        let loops = (edge.loops.iter())
            .map(|l| match *l {
                LoopReg::While(id) => LoopAt::While(id),
                LoopReg::Range { id, r_var, r_hi } => LoopAt::Range {
                    id,
                    v: regs_i[r_var as usize],
                    hi: regs_i[r_hi as usize],
                },
            })
            .collect();
        let ticks = self.ckpt.ticks(self.countdown);
        self.ckpt.taken.push(Checkpoint {
            ticks,
            printed: self.ckpt.printed + self.out.len() as u64,
            loops,
            stack,
            globals: self.globals.clone(),
            domains: self.domains.clone(),
            input_at: self.input.cursor(),
            rng_state: self.rng_state,
            queries: self.queries,
        });
        self.ckpt.next_at = ticks + self.ckpt.every;
    }

    /// Main, from `from` on.
    pub(crate) fn resume(&mut self, from: Checkpoint) -> Res<()> {
        self.stack = from.stack;
        self.globals = from.globals;
        self.domains = from.domains;
        self.input.rewind(from.input_at);
        self.rng_state = from.rng_state;
        self.queries = from.queries;
        self.ckpt.printed = from.printed;
        let program = self.program;
        self.resume_block(&program.main, &from.loops).map(|_| ())
    }

    fn resume_block(&mut self, block: &'p Block, at: &[LoopAt]) -> Res<Flow> {
        let Some(k) = block.stmts.iter().position(|s| holds(s, at[0].id())) else {
            return trap(Span::default(), "the checkpoint's loop is not in main");
        };
        let mut flow = self.resume_stmt(&block.stmts[k], at)?;
        for stmt in &block.stmts[k + 1..] {
            if !matches!(flow, Flow::Next) {
                break;
            }
            flow = self.stmt(stmt)?;
        }
        Ok(flow)
    }

    fn resume_stmt(&mut self, stmt: &'p Stmt, at: &[LoopAt]) -> Res<Flow> {
        let rest = &at[1..];
        match (stmt, at[0]) {
            (Stmt::If(_, then, otherwise), _) => {
                let branch = if block_holds(then, at[0].id()) {
                    then
                } else {
                    otherwise
                };
                self.resume_block(branch, at)
            }
            (Stmt::While(id, _, body), LoopAt::While(_)) => {
                if !rest.is_empty() {
                    if let Some(left) = left_loop(self.resume_block(body, rest)?, *id) {
                        return Ok(left);
                    }
                }
                self.stmt(stmt)
            }
            (Stmt::For(id, _, iter, body), LoopAt::Range { v, hi, .. }) => {
                let mut from = v;
                if !rest.is_empty() {
                    if let Some(left) = left_loop(self.resume_block(body, rest)?, *id) {
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
            _ => trap(Span::default(), "the checkpoint does not match main"),
        }
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

fn holds(stmt: &Stmt, id: LoopId) -> bool {
    match stmt {
        Stmt::If(_, then, otherwise) => block_holds(then, id) || block_holds(otherwise, id),
        Stmt::While(own, _, body) | Stmt::For(own, _, _, body) => {
            *own == id || block_holds(body, id)
        }
        _ => false,
    }
}

fn block_holds(block: &Block, id: LoopId) -> bool {
    block.stmts.iter().any(|s| holds(s, id))
}

/// Whether `at` names a nest of loops in `block` a run can start again in.
fn locates(block: &Block, at: &[LoopAt]) -> bool {
    let Some(&first) = at.first() else {
        return true;
    };
    let Some(stmt) = block.stmts.iter().find(|s| holds(s, first.id())) else {
        return false;
    };
    match (stmt, first) {
        (Stmt::If(_, then, otherwise), _) => locates(
            if block_holds(then, first.id()) {
                then
            } else {
                otherwise
            },
            at,
        ),
        (Stmt::While(id, _, body), LoopAt::While(want)) => *id == want && locates(body, &at[1..]),
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
            LoopAt::Range { id: want, .. },
        ) => *id == want && locates(body, &at[1..]),
        _ => false,
    }
}
