//! A loop over a call to a fn value that can only be one closure, whose
//! body builds and returns a list, runs that body in place:
//! `for a in g(u)` with `g = c => [q for q in c.adj4 if ..]` becomes
//! `for a in (do { let v = []; for q in u.adj4 { if ..: v.push(q) }; give v })`,
//! which `crate::fuse` then runs without building the list. It majorizes:
//! the same work in the same order, minus a call.
//!
//! A loop over a direct call to a small non-recursive fn that builds and
//! returns a list, as a `yields` fn does, runs its body in place the same
//! way: `for (x, y) in r.splits` walks its cuts as a loop of its own, with no
//! callback per cut.

use crate::devirt::{skippable, Devirt};
use crate::uses::effects::{trap_free_block, writes_own, Touch};
use crate::uses::loops::{each_loop, stmt_count, Loop};
use cpl_ir::{
    Arg, Block, ClosureDef, ClosureId, Expr, ExprKind, FnId, Iter, Local, LoopId, Place, PlaceKind,
    Program, Slot, Stmt, Ty,
};
use std::collections::BTreeMap;

/// Statements a spliced body may have, at any depth.
const MAX_STMTS: usize = 12;

/// Where a loop lives: a fn, or (`None`) main, setup or a closure.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    Fn(FnId),
    Main,
    Setup,
    Closure(ClosureId),
}

/// The body a loop runs in place: a closure's, or a fn's.
#[derive(Clone, Copy)]
enum Source {
    Closure(ClosureId),
    Fn(FnId),
}

pub(crate) fn splice(program: &mut Program, traps: bool, fns: bool) {
    let mut found: BTreeMap<(Owner, LoopId), Source> = BTreeMap::new();
    {
        let devirt = Devirt::collect(program);
        let mut scan = |owner: Owner, home: Option<FnId>, body: &Block| {
            each_loop(body, &mut |id, head| {
                let Loop::For(_, Iter::Seq(seq, false)) = head else {
                    return;
                };
                match &seq.kind {
                    ExprKind::CallExpr(callee, _) => {
                        if let Some(cid) = devirt.target(home, callee) {
                            if skippable(callee) {
                                found.insert((owner, id), Source::Closure(cid));
                            }
                        }
                    }
                    ExprKind::Call(f, _)
                        if owner != Owner::Fn(*f) && !devirt.recursive_fns.contains(f) =>
                    {
                        found.insert((owner, id), Source::Fn(*f));
                    }
                    _ => {}
                }
            });
        };
        scan(Owner::Main, None, &program.main);
        scan(Owner::Setup, None, &program.setup);
        for (id, f) in program.fns.iter().enumerate() {
            scan(Owner::Fn(id), Some(id), &f.body);
        }
        for (cid, c) in program.closures.iter().enumerate() {
            scan(Owner::Closure(cid), None, &c.body);
        }
    }
    let body_of = |program: &Program, source: Source| match source {
        Source::Closure(cid) => closure_producer(&program.closures[cid]),
        Source::Fn(_) if !fns => None,
        Source::Fn(f) => fn_producer(&program.fns, &program.fns[f], traps),
    };
    found.retain(|_, source| body_of(program, *source).is_some());
    if found.is_empty() {
        return;
    }
    let mut next_loop = max_loop(program) + 1;
    // Each source's body, its parameter count and its locals.
    let sources: BTreeMap<(bool, usize), (Block, usize, Vec<Local>)> = found
        .values()
        .filter_map(|&source| {
            let block = body_of(program, source)?;
            Some(match source {
                Source::Closure(cid) => {
                    let c = &program.closures[cid];
                    ((true, cid), (block, c.params.len(), c.locals.clone()))
                }
                Source::Fn(f) => {
                    let f_ = &program.fns[f];
                    ((false, f), (block, f_.params.len(), f_.locals.clone()))
                }
            })
        })
        .collect();
    let mut run = |owner: Owner, body: &mut Block, locals: &mut Vec<Local>| {
        let mut todo = |id: LoopId, seq: &mut Expr, body: &Block| {
            let Some(&source) = found.get(&(owner, id)) else {
                return;
            };
            let (key, args) = match (&seq.kind, source) {
                (ExprKind::CallExpr(_, args), Source::Closure(cid)) => ((true, cid), args),
                (ExprKind::Call(_, args), Source::Fn(f)) => ((false, f), args),
                _ => return,
            };
            let Some((block, params, c_locals)) = sources.get(&key) else {
                return;
            };
            let (mut block, params) = (block.clone(), *params);
            if args.len() != params {
                return;
            }
            // The fused body runs between the producer's steps, so it must
            // leave what the producer reads alone.
            let touch = Touch::of_block(body);
            let read = |a: &Arg| {
                matches!(a, Arg::Value(Expr { kind: ExprKind::Local(s), .. })
                    if touch.writes.contains(&(false, *s)) || touch.binds.contains(s))
            };
            if !key.0 && args.iter().any(read) {
                return;
            }
            let mut ren = Ren {
                map: Vec::with_capacity(c_locals.len()),
                consts: Vec::with_capacity(params),
                loops: BTreeMap::new(),
                next: next_loop,
            };
            for arg in args {
                let Arg::Value(value) = arg else {
                    return;
                };
                match value.kind {
                    ExprKind::Local(s) => {
                        ren.map.push(s);
                        ren.consts.push(None);
                    }
                    ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Char(_) | ExprKind::Unit => {
                        ren.map.push(usize::MAX);
                        ren.consts.push(Some(value.clone()));
                    }
                    _ => return,
                }
            }
            let base = locals.len();
            for i in params..c_locals.len() {
                ren.map.push(base + i - params);
            }
            let ok = ren.block(&mut block);
            if !ok {
                return;
            }
            next_loop = ren.next;
            locals.extend(c_locals[params..].iter().cloned());
            seq.kind = ExprKind::Do(block);
        };
        for_loops_mut(body, &mut todo);
    };
    run(Owner::Main, &mut program.main, &mut program.main_locals);
    run(Owner::Setup, &mut program.setup, &mut program.main_locals);
    for (id, f) in program.fns.iter_mut().enumerate() {
        run(Owner::Fn(id), &mut f.body, &mut f.locals);
    }
    for (cid, c) in program.closures.iter_mut().enumerate() {
        run(Owner::Closure(cid), &mut c.body, &mut c.locals);
    }
}

/// The list-building block closure `c` returns, when it takes its
/// arguments by value, captures nothing and is small.
fn closure_producer(c: &ClosureDef) -> Option<Block> {
    if !c.captures.is_empty() || !c.muts.is_empty() {
        return None;
    }
    producer_body(&c.body)
}

/// The list a `yields` fn `f` builds, when it takes small parameters by
/// value, is small, and makes it unseen: no call, trap or write but to its
/// own locals, so its steps may interleave with the loop that reads them.
fn yields(f: &cpl_ir::Function) -> bool {
    f.locals.iter().any(|l| l.name == "__yields")
}

fn fn_producer(fns: &[cpl_ir::Function], f: &cpl_ir::Function, traps: bool) -> Option<Block> {
    let generator = yields(f);
    // A parameter is bound as a local, so only ones that cost no copy.
    let small = |ty: &Ty| match ty {
        Ty::Range(e) => e.is_scalar(),
        ty => ty.is_scalar(),
    };
    let params = &f.locals[..f.params.len()];
    if !generator || f.params.iter().any(|p| p.mutable) || !params.iter().all(|l| small(&l.ty)) {
        return None;
    }
    let block = producer_body(&f.body)?;
    let seq = Expr {
        kind: ExprKind::Do(block),
        ty: f.ret.clone(),
        span: Default::default(),
    };
    let p = crate::fuse::producer(&seq)?;
    // The range it walks is worked out before the first step, as the call
    // was, so only the steps must be unseen. A walk fed by another
    // `yields` fn runs in its callback, which a `return` cannot leave.
    let fed = match p.iter {
        Iter::Seq(e, _) => match &e.kind {
            ExprKind::Call(g, _) => yields(&fns[*g]),
            ExprKind::CallExpr(..) => true,
            _ => false,
        },
        _ => false,
    };
    let unseen = !fed && trap_free_block(p.body, *p.vec, traps) && writes_own(p.body);
    let ExprKind::Do(block) = seq.kind else {
        return None;
    };
    unseen.then_some(block)
}

fn producer_body(body: &Block) -> Option<Block> {
    let block = match body.stmts.as_slice() {
        [Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Expr(e)] => match &e.kind {
            ExprKind::Do(b) => b.clone(),
            _ => return None,
        },
        [Stmt::Let(v, Some(_)), Stmt::For(..), Stmt::Return(Some(e)) | Stmt::Give(e)]
            if e.kind == ExprKind::Local(*v) =>
        {
            let mut stmts = body.stmts.clone();
            stmts[2] = Stmt::Give(e.clone());
            Block { stmts }
        }
        _ => return None,
    };
    let count = stmt_count(&block);
    if count > MAX_STMTS || crate::fuse::block_producer(&block).is_none() {
        return None;
    }
    Some(block)
}

fn max_loop(program: &Program) -> LoopId {
    let mut max = 0;
    let mut see = |b: &Block| each_loop(b, &mut |id, _| max = max.max(id));
    see(&program.main);
    see(&program.setup);
    program.fns.iter().for_each(|f| see(&f.body));
    program.closures.iter().for_each(|c| see(&c.body));
    max
}

/// Calls `f` on the id and sequence of each `for x in seq` in `b`, at
/// any depth, the `Do` blocks of expressions too.
fn for_loops_mut(b: &mut Block, f: &mut dyn FnMut(LoopId, &mut Expr, &Block)) {
    cpl_ir::walk::blocks_mut(b, &mut |block| {
        for stmt in &mut block.stmts {
            if let Stmt::For(id, _, Iter::Seq(seq, false), body) = stmt {
                f(*id, seq, body);
            }
        }
    });
}

/// How a spliced body's locals and loops become the host's.
struct Ren {
    /// The host slot of each local of the closure.
    map: Vec<Slot>,
    /// Parameters given a constant, which takes their place.
    consts: Vec<Option<Expr>>,
    loops: BTreeMap<LoopId, LoopId>,
    next: LoopId,
}

impl Ren {
    /// Renames locals and gives loops fresh ids; false on what it does
    /// not know how to move.
    fn block(&mut self, b: &mut Block) -> bool {
        b.stmts.iter_mut().all(|stmt| match stmt {
            Stmt::Let(slot, init) => {
                *slot = self.map[*slot];
                init.as_mut().is_none_or(|e| self.expr(e))
            }
            Stmt::Assign(place, _, value) => self.place(place) && self.expr(value),
            Stmt::Expr(e) | Stmt::Give(e) => self.expr(e),
            Stmt::If(c, then, otherwise) => {
                self.expr(c) && self.block(then) && self.block(otherwise)
            }
            Stmt::While(id, c, body) => {
                self.fresh(id);
                self.expr(c) && self.block(body)
            }
            Stmt::For(id, slot, iter, body) => {
                self.fresh(id);
                *slot = self.map[*slot];
                let ok = match iter {
                    Iter::Range { lo, hi, step, .. } => {
                        self.expr(lo) && self.expr(hi) && step.as_mut().is_none_or(|s| self.expr(s))
                    }
                    Iter::RangeVal(e, _) | Iter::Seq(e, _) => self.expr(e),
                    Iter::TreeRange { .. } | Iter::Walk { .. } => false,
                };
                ok && self.block(body)
            }
            Stmt::Break(to) | Stmt::Continue(to) => match to {
                Some(id) => self.loops.get(id).map(|new| *id = *new).is_some(),
                None => true,
            },
            _ => false,
        })
    }

    fn fresh(&mut self, id: &mut LoopId) {
        self.loops.insert(*id, self.next);
        *id = self.next;
        self.next += 1;
    }

    fn place(&mut self, p: &mut Place) -> bool {
        match &mut p.kind {
            PlaceKind::Local(s) => {
                let ok = self.consts.get(*s).is_none_or(Option::is_none);
                *s = self.map[*s];
                ok
            }
            PlaceKind::Global(_) => true,
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(base),
            PlaceKind::Index(base, idxs) => {
                self.place(base) && idxs.iter_mut().all(|i| self.expr(i))
            }
        }
    }

    fn expr(&mut self, e: &mut Expr) -> bool {
        match &mut e.kind {
            ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::Str(_)
            | ExprKind::None
            | ExprKind::Global(_)
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_)
            | ExprKind::DomainVal(_) => true,
            ExprKind::Local(s) => {
                match self.consts.get(*s).cloned().flatten() {
                    Some(value) => e.kind = value.kind,
                    None => *s = self.map[*s],
                }
                true
            }
            ExprKind::Some(a)
            | ExprKind::Unary(_, a)
            | ExprKind::Cast(_, a)
            | ExprKind::Field(a, _)
            | ExprKind::IsVariant(a, _)
            | ExprKind::Payload(a, _, _)
            | ExprKind::IsSome(a)
            | ExprKind::Unwrap(a, _) => self.expr(a),
            ExprKind::Binary(_, a, b)
            | ExprKind::Cmp(_, a, b)
            | ExprKind::Range { lo: a, hi: b } => self.expr(a) && self.expr(b),
            ExprKind::If(a, b, c) => self.expr(a) && self.expr(b) && self.expr(c),
            ExprKind::Index(base, idxs) => self.expr(base) && idxs.iter_mut().all(|i| self.expr(i)),
            ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
                items.iter_mut().all(|i| self.expr(i))
            }
            ExprKind::RangeCheck { value, lo, hi, .. } => {
                self.expr(value)
                    && lo.as_mut().is_none_or(|x| self.expr(x))
                    && hi.as_mut().is_none_or(|x| self.expr(x))
            }
            ExprKind::Call(_, args) => args.iter_mut().all(|arg| match arg {
                Arg::Value(v) => self.expr(v),
                Arg::Mut(p) => self.place(p),
            }),
            ExprKind::Mutate(_, place, args) => {
                self.place(place) && args.iter_mut().all(|a| self.expr(a))
            }
            ExprKind::Do(b) => self.block(b),
            _ => false,
        }
    }
}
