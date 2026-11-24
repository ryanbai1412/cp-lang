//! The compiled program: its chunks, call sites and loop back edges.

use super::compile::Compiler;
use crate::value::Value;
use cpl_diag::Span;
use cpl_ir::{Arg, BinOp, Builtin, Expr, ExprKind, LoopId, Program, Stmt, Ty};

use super::ir::*;

/// A jump back to the top of one of main's loops, where a checkpoint can
/// be taken: the loops around it, outermost first, ending with its own.
#[derive(Clone, Debug)]
pub(crate) struct BackEdge {
    pub pc: u32,
    pub loops: Vec<LoopReg>,
}

/// The op of `call`, the one effect of statement `stmt` before its
/// body, where a checkpoint taken inside the callee can stand: the loops
/// around it, outermost first.
#[derive(Clone, Debug)]
pub(crate) struct CallSite<'p> {
    pub pc: u32,
    /// The first op of `stmt`, where a run from a checkpoint enters it.
    pub start: u32,
    pub loops: Vec<LoopReg>,
    pub stmt: &'p Stmt,
    pub call: &'p Expr,
    /// The calls `stmt` evaluates before `call`: a run from a checkpoint
    /// takes what they returned instead of making them again.
    pub before: Vec<&'p Expr>,
}

pub(crate) struct Chunk<'p> {
    pub ops: Vec<Op>,
    pub spans: Vec<Span>,
    pub consts_i: Vec<i64>,
    pub range_texts: Vec<&'p str>,
    pub places: Vec<PlacePath>,
    pub closure_specs: Vec<ClosureMakeSpec>,
    pub feed_calls: Vec<FeedCallSpec>,
    /// In a traced chunk, the bodies of fed loops it leaves to the
    /// tree-walker, which runs each from here as the fn yields.
    pub fed_bodies: Vec<(LoopId, u32)>,
    /// The `CallTraced` ops, by pc, that run a fed loop.
    pub traced_feeds: Vec<(u32, TracedFeed<'p>)>,
    pub consts_v: Vec<Value>,
    pub formats: Vec<FormatSpec<'p>>,
    pub applies: Vec<ApplySpec<'p>>,
    pub fallback_stmts: Vec<&'p Stmt>,
    /// The statement each `Step` starts.
    pub steps: Vec<&'p Stmt>,
    /// The `assert`s and `Trap`s each `Fail` reports.
    pub fails: Vec<&'p Stmt>,
    pub fallback_exprs: Vec<&'p Expr>,
    /// The arguments of each `CallTraced` that has `mut` ones.
    pub lent_calls: Vec<&'p [Arg]>,
    pub trace_points: Vec<TracePoint<'p>>,
    pub traced: bool,
    pub fill_tys: Vec<&'p Ty>,
    pub num_i: usize,
    pub num_v: usize,
    pub num_local_v: usize,
    pub slot_map: Vec<SlotLoc>,
    /// The named locals a trace shows, by slot, and whether each is an `Int`.
    pub named_locals: Vec<(usize, &'p str, bool)>,
    pub param_count: usize,
    pub ret_kind: Option<ScalarKind>,
    pub guarded_domains: Vec<cpl_ir::DomainId>,
    pub needs_stack: bool,
    /// Runs no code but its own (no call, closure, feed, fallback, print or
    /// format) and needs no stack frame, so nothing it runs can read a
    /// variable while it is lent to the chunk.
    pub leaf: bool,
    pub has_by_ref_closure: bool,
    /// A param holds a fn value, which may reach a place the caller holds lent.
    pub takes_fn: bool,
    /// The slots `has_by_ref_closure` syncs around a call; `None` is all.
    pub ref_slots: Option<Vec<u16>>,
    /// The locals a `mut` view made here writes into, kept on the stack too
    /// once synced so that reads through the view find them.
    pub view_roots: Vec<u16>,
    pub back_edges: Vec<BackEdge>,
    pub call_sites: Vec<CallSite<'p>>,
    /// The call ops, by pc in order, whose result a call site's `before`
    /// needs: where each one's op puts it (`None` for `Unit`), and the call's
    /// address.
    pub keeps: Vec<(u32, Option<SlotLoc>, usize)>,
}

#[derive(Clone, Copy)]
pub(crate) enum FastClosure {
    Param(u8),
    Add(u8, u8),
    Sub(u8, u8),
    Mul(u8, u8),
    Min(u8, u8),
    Max(u8, u8),
    BitAnd(u8, u8),
    BitOr(u8, u8),
    BitXor(u8, u8),
    MulAdd(u8, u8, u8),
}

pub(super) fn recognize_fast_closure(
    def: &cpl_ir::ClosureDef,
    program: &Program,
) -> Option<FastClosure> {
    if !def.captures.is_empty()
        || !def.muts.is_empty()
        || def.params.len() > 4
        || !def.params.iter().enumerate().all(|(i, &s)| s == i)
        || ScalarKind::of_ty(&def.ret, program) != Some(ScalarKind::Int)
        || !def
            .params
            .iter()
            .all(|&s| ScalarKind::of_ty(&def.locals[s].ty, program) == Some(ScalarKind::Int))
    {
        return None;
    }
    if def.body.stmts.len() != 1 {
        return None;
    }
    let expr = match &def.body.stmts[0] {
        Stmt::Return(Some(e)) | Stmt::Expr(e) => e,
        _ => return None,
    };
    let param_of = |e: &Expr| -> Option<u8> {
        if let ExprKind::Local(s) = e.kind {
            if s < def.params.len() {
                return Some(s as u8);
            }
        }
        None
    };
    if let Some(p) = param_of(expr) {
        return Some(FastClosure::Param(p));
    }
    match &expr.kind {
        ExprKind::Binary(op, a, b) => {
            if let (Some(pa), Some(pb)) = (param_of(a), param_of(b)) {
                return match op {
                    BinOp::Add => Some(FastClosure::Add(pa, pb)),
                    BinOp::Sub => Some(FastClosure::Sub(pa, pb)),
                    BinOp::Mul => Some(FastClosure::Mul(pa, pb)),
                    BinOp::BitAnd => Some(FastClosure::BitAnd(pa, pb)),
                    BinOp::BitOr => Some(FastClosure::BitOr(pa, pb)),
                    BinOp::BitXor => Some(FastClosure::BitXor(pa, pb)),
                    _ => None,
                };
            }
            if matches!(op, BinOp::Add) {
                if let (Some(pc), ExprKind::Binary(BinOp::Mul, ma, mb)) = (param_of(a), &b.kind) {
                    if let (Some(pa), Some(pb)) = (param_of(ma), param_of(mb)) {
                        return Some(FastClosure::MulAdd(pa, pb, pc));
                    }
                }
                if let (ExprKind::Binary(BinOp::Mul, ma, mb), Some(pc)) = (&a.kind, param_of(b)) {
                    if let (Some(pa), Some(pb)) = (param_of(ma), param_of(mb)) {
                        return Some(FastClosure::MulAdd(pa, pb, pc));
                    }
                }
            }
            None
        }
        ExprKind::Builtin(Builtin::Min, args) if args.len() == 2 => {
            Some(FastClosure::Min(param_of(&args[0])?, param_of(&args[1])?))
        }
        ExprKind::Builtin(Builtin::Max, args) if args.len() == 2 => {
            Some(FastClosure::Max(param_of(&args[0])?, param_of(&args[1])?))
        }
        _ => None,
    }
}

/// The fn `def`'s body calls with its params in order, when that is all it
/// does and the fn takes them as the closure's caller lays them out.
fn forward_of(def: &cpl_ir::ClosureDef, fns: &[Option<Chunk<'_>>]) -> Option<u16> {
    if !def.captures.is_empty() || !def.muts.is_empty() || def.body.stmts.len() != 1 {
        return None;
    }
    let (Stmt::Return(Some(e)) | Stmt::Expr(e)) = &def.body.stmts[0] else {
        return None;
    };
    let ExprKind::Call(fid, args) = &e.kind else {
        return None;
    };
    let in_order = args.len() == def.params.len()
        && args.iter().zip(&def.params).all(|(a, &p)| {
            matches!(a, cpl_ir::Arg::Value(x) if matches!(x.kind, ExprKind::Local(s) if s == p))
        });
    fns.get(*fid)?.as_ref()?;
    in_order.then_some(*fid as u16)
}

pub(crate) struct CompiledProgram<'p> {
    pub main: Chunk<'p>,
    /// What is above `@!multitest`. It ends in `RetI` of 1, so a run
    /// that falls off its end is told apart from a top-level `return`.
    pub setup: Chunk<'p>,
    pub fns: Vec<Option<Chunk<'p>>>,
    pub fed_fns: Vec<Option<Chunk<'p>>>,
    pub closures: Vec<Option<Chunk<'p>>>,
    pub fast_closures: Vec<Option<FastClosure>>,
    /// The fn each closure only passes its params on to, as a fn value
    /// does: calling it calls the fn.
    pub forwards: Vec<Option<u16>>,
}

impl<'p> CompiledProgram<'p> {
    /// Every compiled chunk: main, setup, fns, fed fns and closures.
    pub(crate) fn chunks(&self) -> impl Iterator<Item = &Chunk<'p>> {
        let bodies = self.fns.iter().chain(&self.fed_fns).chain(&self.closures);
        std::iter::once(&self.main)
            .chain([&self.setup])
            .chain(bodies.flatten())
    }

    /// With `traced`, every write, print and call is reported to the
    /// tracer as the tree-walker reports it; with `profiled`, each
    /// statement's start is a step for the profiler.
    pub fn compile(
        program: &'p Program,
        feeds: &cpl_ir::pushed::Feeds,
        traced: bool,
        profiled: bool,
    ) -> Self {
        let fed_fn = |id: usize, fed_ok: &[bool]| -> Option<Chunk<'p>> {
            let f = &program.fns[id];
            if traced || program.judge_fns.contains(&id) {
                return None;
            }
            let &acc_slot = feeds.pushers.get(&id)?;
            let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
            let mut fc = Compiler::new(
                program,
                feeds,
                &f.locals,
                Some(id),
                f.params.len(),
                None,
                profiled,
            );
            fc.fed_ok = fed_ok;
            fc.feed_acc = Some(acc_slot);
            fc.profiled = profiled;
            fc.compile_block(&f.body, None, false);
            fc.emit(Op::FeedEnd, Span::default());
            let mut fchunk = fc.finish();
            // A fallback the tree-walker runs (an assert's trap) is fine
            // as long as it never yields.
            let yields_on_vm = (fchunk.fallback_stmts.iter())
                .all(|s| !Compiler::stmt_mentions_slot(s, acc_slot))
                && (fchunk.fallback_exprs.iter())
                    .all(|e| !Compiler::expr_mentions_slot(e, acc_slot));
            if !yields_on_vm {
                return None;
            }
            fchunk.param_count = f.params.len();
            fchunk.takes_fn = (f.locals.iter().take(f.params.len())).any(|l| l.ty.has_fn(program));
            fchunk.guarded_domains = guarded_domains;
            Some(fchunk)
        };
        // Every `yields` fn is first taken to compile, so one that feeds
        // itself or another does; one that doesn't is dropped and the rest
        // are compiled again, until none drops.
        let mut known: Vec<bool> = (0..program.fns.len())
            .map(|id| feeds.pushers.contains_key(&id))
            .collect();
        let fed_fns: Vec<Option<Chunk<'p>>> = loop {
            let fed_fns: Vec<_> = (0..program.fns.len())
                .map(|id| known[id].then(|| fed_fn(id, &known)).flatten())
                .collect();
            let now: Vec<bool> = fed_fns.iter().map(Option::is_some).collect();
            if now == known {
                break fed_fns;
            }
            known = now;
        };
        let fed_ok: Vec<bool> = fed_fns.iter().map(Option::is_some).collect();
        let body_chunk = |locals, cur_fn, params, ret, body, def, captures: &[cpl_ir::Capture]| {
            let mut c = Compiler::new(program, feeds, locals, cur_fn, params, def, profiled);
            c.fed_ok = &fed_ok;
            let refs = captures.iter().filter(|cap| cap.by_ref);
            c.held.extend(refs.map(|cap| cap.inner as u16));
            c.traced = traced;
            let ret_kind = ScalarKind::of_ty(ret, program);
            if !traced && !profiled {
                c.tuple_ret = small_tuple_shape(ret, program);
            }
            c.compile_block(body, ret_kind, true);
            let mut chunk = c.finish();
            chunk.param_count = params;
            chunk.takes_fn =
                (locals.iter().take(params)).any(|l: &cpl_ir::Local| l.ty.has_fn(program));
            chunk.ret_kind = ret_kind;
            chunk.guarded_domains = program.fn_guarded_domains(ret, body);
            chunk
        };
        let mut fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            if program.judge_fns.contains(&id) {
                fns.push(None);
            } else {
                let chunk = body_chunk(
                    &f.locals,
                    Some(id),
                    f.params.len(),
                    &f.ret,
                    &f.body,
                    Some(f),
                    &[],
                );
                fns.push(Some(chunk));
            }
        }
        let mut closures = Vec::with_capacity(program.closures.len());
        let mut fast_closures = Vec::with_capacity(program.closures.len());
        for def in &program.closures {
            // A fast closure runs no statements, so a profiled run has none.
            fast_closures.push(recognize_fast_closure(def, program).filter(|_| !profiled));
            let can_compile =
                def.muts.is_empty() && def.params.iter().enumerate().all(|(i, &s)| s == i);
            if !can_compile {
                closures.push(None);
            } else {
                let chunk = body_chunk(
                    &def.locals,
                    None,
                    def.params.len(),
                    &def.ret,
                    &def.body,
                    None,
                    &def.captures,
                );
                // A variable captured by reference is lent to the chunk's slot.
                let lendable = (def.captures.iter()).all(|cap| {
                    !cap.by_ref || !matches!(chunk.slot_map[cap.inner], SlotLoc::SmallTuple { .. })
                });
                closures.push(lendable.then_some(chunk));
            }
        }
        let mut main_c = Compiler::new(
            program,
            feeds,
            &program.main_locals,
            None,
            0,
            None,
            profiled,
        );
        main_c.fed_ok = &fed_ok;
        main_c.traced = traced;
        main_c.compile_block(&program.main, None, false);
        main_c.emit(Op::RetUnit, Span::default());
        let main = main_c.finish();
        let setup = {
            let mut c = Compiler::new(
                program,
                feeds,
                &program.main_locals,
                None,
                0,
                None,
                profiled,
            );
            c.fed_ok = &fed_ok;
            c.traced = traced;
            c.compile_block(&program.setup, None, false);
            let done = c.alloc_i();
            c.emit_const_i(done, 1, Span::default());
            c.emit(Op::RetI { src: done }, Span::default());
            c.finish()
        };
        let forwards = (program.closures.iter())
            .map(|def| forward_of(def, &fns).filter(|_| !profiled && !traced))
            .collect();
        CompiledProgram {
            forwards,
            main,
            setup,
            fns,
            fed_fns,
            closures,
            fast_closures,
        }
    }
}
