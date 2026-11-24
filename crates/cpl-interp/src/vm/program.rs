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
    pub consts_v: Vec<Value>,
    pub prints: Vec<PrintSpec<'p>>,
    pub formats: Vec<FormatSpec<'p>>,
    pub applies: Vec<ApplySpec<'p>>,
    pub fallback_stmts: Vec<&'p Stmt>,
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
    pub param_count: usize,
    pub ret_kind: Option<ScalarKind>,
    pub guarded_domains: Vec<cpl_ir::DomainId>,
    pub needs_stack: bool,
    pub has_by_ref_closure: bool,
    /// The slots `has_by_ref_closure` syncs around a call; `None` is all.
    pub ref_slots: Option<Vec<u16>>,
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

pub(crate) struct CompiledProgram<'p> {
    pub main: Chunk<'p>,
    /// What is above `@!multitest`. It ends in `RetI` of 1, so a run
    /// that falls off its end is told apart from a top-level `return`.
    pub setup: Option<Chunk<'p>>,
    pub fns: Vec<Option<Chunk<'p>>>,
    pub fed_fns: Vec<Option<Chunk<'p>>>,
    pub closures: Vec<Option<Chunk<'p>>>,
    pub fast_closures: Vec<Option<FastClosure>>,
}

impl<'p> CompiledProgram<'p> {
    /// With `traced`, every write, print and call is reported to the
    /// tracer as the tree-walker reports it.
    pub fn compile(program: &'p Program, feeds: &cpl_ir::pushed::Feeds, traced: bool) -> Self {
        let empty_fed: Vec<bool> = vec![false; program.fns.len()];
        let mut fed_fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            if traced || f.locals.len() > 512 || program.judge_fns.contains(&id) {
                fed_fns.push(None);
            } else if let Some(&acc_slot) = feeds.pushers.get(&id) {
                let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
                let mut fc = Compiler::new(
                    program,
                    feeds,
                    &empty_fed,
                    &f.locals,
                    Some(id),
                    f.params.len(),
                );
                fc.feed_acc = Some(acc_slot);
                fc.allow_fed_loops = false;
                fc.compile_block(&f.body, None, false);
                fc.emit(Op::FeedEnd, Span::default());
                let mut fchunk = fc.finish();
                // A fallback the tree-walker runs (an assert's trap) is fine
                // as long as it never yields.
                let yields_on_vm = (fchunk.fallback_stmts.iter())
                    .all(|s| !Compiler::stmt_mentions_slot(s, acc_slot))
                    && (fchunk.fallback_exprs.iter())
                        .all(|e| !Compiler::expr_mentions_slot(e, acc_slot));
                if yields_on_vm {
                    fchunk.param_count = f.params.len();
                    fchunk.ret_kind = None;
                    fchunk.guarded_domains = guarded_domains;
                    if fchunk.has_by_ref_closure {
                        fchunk.needs_stack = true;
                    }
                    fed_fns.push(Some(fchunk));
                } else {
                    fed_fns.push(None);
                }
            } else {
                fed_fns.push(None);
            }
        }
        let fed_ok: Vec<bool> = fed_fns.iter().map(Option::is_some).collect();
        let mut fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
            if f.locals.len() > 512 || program.judge_fns.contains(&id) {
                fns.push(None);
            } else {
                let mut c =
                    Compiler::new(program, feeds, &fed_ok, &f.locals, Some(id), f.params.len());
                c.traced = traced;
                let ret_kind = ScalarKind::of_ty(&f.ret, program);
                c.compile_block(&f.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = f.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                if chunk.has_by_ref_closure {
                    chunk.needs_stack = true;
                }
                fns.push(Some(chunk));
            }
        }
        let mut closures = Vec::with_capacity(program.closures.len());
        let mut fast_closures = Vec::with_capacity(program.closures.len());
        for def in &program.closures {
            fast_closures.push(recognize_fast_closure(def, program));
            let can_compile = def.locals.len() <= 512
                && def.muts.is_empty()
                && def.captures.iter().all(|cap| !cap.by_ref)
                && def.params.iter().enumerate().all(|(i, &s)| s == i);
            if !can_compile {
                closures.push(None);
            } else {
                let guarded_domains = program.fn_guarded_domains(&def.ret, &def.body);
                let mut c =
                    Compiler::new(program, feeds, &fed_ok, &def.locals, None, def.params.len());
                c.traced = traced;
                let ret_kind = ScalarKind::of_ty(&def.ret, program);
                c.compile_block(&def.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = def.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                if chunk.has_by_ref_closure {
                    chunk.needs_stack = true;
                }
                closures.push(Some(chunk));
            }
        }
        let mut main_c = Compiler::new(program, feeds, &fed_ok, &program.main_locals, None, 0);
        main_c.traced = traced;
        main_c.compile_block(&program.main, None, false);
        main_c.emit(Op::RetUnit, Span::default());
        let mut main = main_c.finish();
        if main.has_by_ref_closure {
            main.needs_stack = true;
        }
        let setup = program.multitest.then(|| {
            let mut c = Compiler::new(program, feeds, &fed_ok, &program.main_locals, None, 0);
            c.traced = traced;
            c.compile_block(&program.setup, None, false);
            let done = c.alloc_i();
            c.emit_const_i(done, 1, Span::default());
            c.emit(Op::RetI { src: done }, Span::default());
            let mut setup = c.finish();
            if setup.has_by_ref_closure {
                setup.needs_stack = true;
            }
            setup
        });
        CompiledProgram {
            main,
            setup,
            fns,
            fed_fns,
            closures,
            fast_closures,
        }
    }
}
