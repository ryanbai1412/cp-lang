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
use crate::uses::spliced::{
    captures_a_local, local_reads, may_write_global, param_reads, return_count, sorts_or_calls,
    writes_any,
};
use cpl_ir::{
    Arg, Block, Builtin, ClosureDef, ClosureId, Expr, ExprKind, FnId, Function, Iter, Local,
    LoopId, Mutation, Place, PlaceKind, Program, Slot, SortBy, Stmt, Ty,
};
use std::collections::{BTreeMap, BTreeSet};

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
                frozen: Vec::new(),
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
    /// Per parameter of an inlined fn: whether it is a value nothing may write.
    frozen: Vec<bool>,
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
                if self.frozen.get(*s) == Some(&true) {
                    return false;
                }
                match self.consts.get(*s).cloned().flatten() {
                    Some(Expr {
                        kind: ExprKind::Global(g),
                        ..
                    }) if !self.frozen.is_empty() => {
                        p.kind = PlaceKind::Global(g);
                        true
                    }
                    Some(_) => false,
                    None => {
                        *s = self.map[*s];
                        true
                    }
                }
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
                    // Inlined, a parameter is its argument, where it was written.
                    Some(value) if !self.frozen.is_empty() => *e = value,
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
            ExprKind::CallExpr(f, args) => {
                self.expr(f)
                    && args.iter_mut().all(|arg| match arg {
                        Arg::Value(v) => self.expr(v),
                        Arg::Mut(p) => self.place(p),
                    })
            }
            ExprKind::Sort(place, by, desc) => {
                self.place(place)
                    && match by {
                        SortBy::Natural => true,
                        SortBy::Key(slot, key) => {
                            *slot = self.map[*slot];
                            self.expr(key)
                        }
                        SortBy::Fn { func, .. } => self.expr(func),
                    }
                    && desc.as_mut().is_none_or(|d| self.expr(d))
            }
            // Inlined, a fn's closures capture only globals (`inlinable`).
            ExprKind::Closure(_) => !self.frozen.is_empty(),
            _ => false,
        }
    }
}

/// Statements, at any depth, of a fn a call may run in place.
const MAX_INLINED: usize = 6;

/// A call to a small non-recursive fn, std or user alike, whose arguments
/// are variables, constants or closures runs the fn's body in place, so the
/// analyses after it see what the fn does: `xs.sort(by: k)` is a sort of
/// `xs` by `k`, and `sorted` the copy it sorts.
pub(crate) fn inline_small(program: &mut Program) {
    let recursive = crate::devirt::recursive_fns(program);
    let bodies: BTreeMap<FnId, Function> = (program.fns.iter().enumerate())
        .filter(|(id, f)| !recursive.contains(id) && inlinable(f, &program.closures))
        .map(|(id, f)| (id, f.clone()))
        .collect();
    if bodies.is_empty() {
        return;
    }
    let seeds = sorting(&bodies);
    if seeds.is_empty() {
        return;
    }
    let keys: Vec<Option<KeyFn>> = program.closures.iter().map(key_fn).collect();
    let turners = turners(program);
    let mut next = max_loop(program) + 1;
    let mut run = |body: &mut Block, locals: &mut Vec<Local>| {
        inline_in(body, locals, &bodies, Some(&seeds), &mut next, 3);
        each_expr_mut(body, &mut |e| {
            simplify(e, &keys, &turners, locals, &mut next)
        });
        cpl_ir::walk::blocks_mut(body, &mut flatten);
    };
    run(&mut program.main, &mut program.main_locals);
    run(&mut program.setup, &mut program.main_locals);
    for f in &mut program.fns {
        run(&mut f.body, &mut f.locals);
    }
    for c in &mut program.closures {
        run(&mut c.body, &mut c.locals);
    }
}

/// The fns that sort a place in place, themselves or through one another.
/// Only calls to them are run in place: their sort is what the analyses
/// after want to see; other calls stay, so a fn's C++ stays its own.
fn sorting(bodies: &BTreeMap<FnId, Function>) -> BTreeSet<FnId> {
    let mut seeds = BTreeSet::new();
    loop {
        let before = seeds.len();
        for (id, f) in bodies {
            if sorts_or_calls(&f.body, &seeds) {
                seeds.insert(*id);
            }
        }
        if seeds.len() == before {
            return seeds;
        }
    }
}

/// Runs in place each call in `body` to a fn of `only` (any inlinable fn
/// when `None`), and inside what it runs, any inlinable fn.
fn inline_in(
    body: &mut Block,
    locals: &mut Vec<Local>,
    bodies: &BTreeMap<FnId, Function>,
    only: Option<&BTreeSet<FnId>>,
    next: &mut LoopId,
    depth: usize,
) {
    if depth == 0 {
        return;
    }
    each_expr_mut(body, &mut |e| {
        let ExprKind::Call(f, args) = &e.kind else {
            return;
        };
        // At the top, a call is run in place only when it hands the callee a place to change
        // or a closure: what the caller's analyses must see happen to them.
        let hands = args.iter().any(|a| match a {
            Arg::Mut(_) => true,
            Arg::Value(v) => matches!(v.kind, ExprKind::Closure(_)),
        });
        if only.is_some_and(|s| !s.contains(f) || !hands) {
            return;
        }
        let Some(g) = bodies.get(f) else {
            return;
        };
        let mark = (locals.len(), *next);
        if let Some(mut b) = inlined(g, args, e.span, locals, next) {
            inline_in(&mut b, locals, bodies, None, next, depth - 1);
            if read_globals(args).any(|gid| may_write_global(&b, gid)) {
                locals.truncate(mark.0);
                *next = mark.1;
            } else {
                e.kind = ExprKind::Do(b);
            }
        }
    });
}

fn read_globals(args: &[Arg]) -> impl Iterator<Item = cpl_ir::GlobalId> + '_ {
    args.iter().filter_map(|a| match a {
        Arg::Value(Expr {
            kind: ExprKind::Global(g),
            ..
        }) => Some(*g),
        _ => None,
    })
}

fn inlinable(f: &Function, closures: &[ClosureDef]) -> bool {
    let returns_last = matches!(f.body.stmts.last(), Some(Stmt::Return(_)));
    // A closure that captures a local of `f` would capture another's once run in place.
    return_count(&f.body) == returns_last as usize
        && !captures_a_local(&f.body, closures)
        && stmt_count(&f.body) <= MAX_INLINED
        && !f.locals.iter().any(|l| l.name == "__yields")
}

/// The body of `g` run with `args`, its locals moved into `locals`.
fn inlined(
    g: &Function,
    args: &[Arg],
    at: cpl_diag::Span,
    locals: &mut Vec<Local>,
    next: &mut LoopId,
) -> Option<Block> {
    let params = g.params.len();
    if args.len() != params {
        return None;
    }
    let uses = param_reads(&g.body, params);
    let mut ren = Ren {
        map: Vec::with_capacity(g.locals.len()),
        consts: Vec::with_capacity(params),
        frozen: g.params.iter().map(|p| !p.mutable).collect(),
        loops: BTreeMap::new(),
        next: *next,
    };
    let (mut muts, mut values, mut temps) = (Vec::new(), Vec::new(), Vec::new());
    for (p, arg) in args.iter().enumerate() {
        let (slot, value) = match arg {
            Arg::Mut(Place {
                kind: PlaceKind::Local(s),
                ..
            }) => {
                muts.push(*s);
                (*s, None)
            }
            Arg::Mut(Place {
                kind: PlaceKind::Global(gid),
                ty,
                span,
            }) => {
                let global = Expr {
                    kind: ExprKind::Global(*gid),
                    ty: ty.clone(),
                    span: *span,
                };
                (0, Some(global))
            }
            Arg::Value(v) => match v.kind {
                ExprKind::Local(s) => {
                    values.push(s);
                    (s, Some(v.clone()))
                }
                ExprKind::Unit
                | ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Global(_) => (0, Some(v.clone())),
                ExprKind::Closure(_) if uses[p] <= 1 => (0, Some(v.clone())),
                // Any other value is worked out first, into a local of its own.
                _ => {
                    temps.push((p, v));
                    (usize::MAX, None)
                }
            },
            Arg::Mut(_) => return None,
        };
        ren.map.push(slot);
        ren.consts.push(value);
    }
    if values.iter().any(|s| muts.contains(s)) {
        return None;
    }
    if temps.iter().any(|(_, v)| writes_any(v, &values)) {
        return None;
    }
    let base = locals.len();
    ren.map
        .extend((params..g.locals.len()).map(|i| base + i - params));
    let first_temp = base + g.locals.len() - params;
    for (k, (p, _)) in temps.iter().enumerate() {
        ren.map[*p] = first_temp + k;
    }
    // What the body does is reported at the call running it, the args at their own spans.
    let mut block = g.body.clone();
    each_expr_mut(&mut block, &mut |e| e.span = at);
    match block.stmts.pop() {
        Some(Stmt::Return(Some(e))) => block.stmts.push(Stmt::Give(e)),
        Some(Stmt::Return(None)) | None => {}
        Some(last) => block.stmts.push(last),
    }
    if !ren.block(&mut block) {
        return None;
    }
    *next = ren.next;
    locals.extend(g.locals[params..].iter().cloned());
    let lets = temps.iter().enumerate().map(|(k, (p, v))| {
        locals.push(Local {
            ty: v.ty.clone(),
            ..g.locals[*p].clone()
        });
        Stmt::Let(first_temp + k, Some((*v).clone()))
    });
    block.stmts.splice(0..0, lets.collect::<Vec<_>>());
    Some(block)
}

/// A closure of one parameter that gives an expression of it and of what it
/// captures: a sort key.
struct KeyFn {
    param: Slot,
    local: Local,
    tys: Vec<Ty>,
    captures: Vec<cpl_ir::Capture>,
    body: Expr,
}

fn key_fn(c: &ClosureDef) -> Option<KeyFn> {
    let ([param], [Stmt::Return(Some(body)) | Stmt::Expr(body)]) =
        (c.params.as_slice(), c.body.stmts.as_slice())
    else {
        return None;
    };
    (c.muts.is_empty() && c.locals.len() == 1 + c.captures.len()).then(|| KeyFn {
        param: *param,
        local: c.locals[*param].clone(),
        tys: c.locals.iter().map(|l| l.ty.clone()).collect(),
        captures: c.captures.clone(),
        body: body.clone(),
    })
}

/// `k` as a key read in place: its parameter a new local of the host.
fn keyed(k: &KeyFn, locals: &mut Vec<Local>, next: &mut LoopId) -> Option<SortBy> {
    let slot = locals.len();
    let mut ren = Ren {
        map: vec![slot; k.tys.len()],
        consts: vec![None; k.tys.len()],
        frozen: Vec::new(),
        loops: BTreeMap::new(),
        next: *next,
    };
    for cap in &k.captures {
        if cap.is_global {
            let ty = k.tys[cap.inner].clone();
            ren.consts[cap.inner] = Some(Expr {
                kind: ExprKind::Global(cap.outer),
                ty,
                span: k.body.span,
            });
        } else {
            ren.map[cap.inner] = cap.outer;
        }
    }
    let mut body = k.body.clone();
    if !ren.expr(&mut body) {
        return None;
    }
    *next = ren.next;
    locals.push(k.local.clone());
    Some(SortBy::Key(slot, Box::new(body)))
}

/// Rewrites what an inlined body leaves into the forms the analyses know:
/// a sort by the identity is the natural sort, by a closure its key, a list built by pushing each
/// item of an indexed value is its copy, and a sorted copy is `sorted`.
fn simplify(
    e: &mut Expr,
    keys: &[Option<KeyFn>],
    turners: &BTreeSet<FnId>,
    locals: &mut Vec<Local>,
    next: &mut LoopId,
) {
    if let ExprKind::Sort(_, by, desc) = &mut e.kind {
        if let SortBy::Fn {
            func,
            is_order: false,
        } = by
        {
            if let ExprKind::Closure(c) = func.kind {
                if let Some(k) = &keys[c] {
                    if is_local(&k.body, k.param) {
                        *by = SortBy::Natural;
                    } else if let Some(key) = keyed(k, locals, next) {
                        *by = key;
                    }
                }
            }
        }
        if let (SortBy::Key(_, key), None) = (&mut *by, &*desc) {
            if let ExprKind::Call(f, args) = &key.kind {
                if let (true, [Arg::Value(inner)]) = (turners.contains(f), args.as_slice()) {
                    let span = key.span;
                    **key = inner.clone();
                    *desc = Some(Box::new(Expr {
                        kind: ExprKind::Bool(true),
                        ty: Ty::Bool,
                        span,
                    }));
                }
            }
        }
        return;
    }
    let kind = std::mem::replace(&mut e.kind, ExprKind::Unit);
    e.kind = match kind {
        ExprKind::Do(b) => match (b.stmts.as_slice(), &e.ty) {
            ([Stmt::Expr(only)], Ty::Unit) => only.kind.clone(),
            _ => match forward(b) {
                b => match copied(&b).or_else(|| given(&b)).or_else(|| sorted(&b)) {
                    Some(k) => k,
                    None => ExprKind::Do(b),
                },
            },
        },
        k => k,
    };
}

/// The fns that wrap their one parameter in a one-field struct ordered
/// backwards (`desc`): a key through one is the inner key, sorted descending.
fn turners(program: &Program) -> BTreeSet<FnId> {
    let field =
        |e: &Expr, slot: Slot| matches!(&e.kind, ExprKind::Field(x, 0) if is_local(x, slot));
    let backwards = |sid: usize| {
        let def = &program.structs[sid];
        let Some(cpl_ir::OrdHook::Lt(lt)) = def.ord else {
            return false;
        };
        def.fields.len() == 1
            && matches!(program.fns[lt].body.stmts.as_slice(),
                [Stmt::Return(Some(Expr { kind: ExprKind::Cmp(cpl_ir::CmpOp::Lt, b, a), .. }))]
                    if field(b, 1) && field(a, 0))
    };
    (program.fns.iter().enumerate())
        .filter(
            |(_, f)| match (f.params.len(), &f.ret, f.body.stmts.as_slice()) {
                (
                    1,
                    Ty::Struct(sid),
                    [Stmt::Return(Some(Expr {
                        kind: ExprKind::Tuple(xs),
                        ..
                    }))],
                ) => matches!(xs.as_slice(), [x] if is_local(x, 0)) && backwards(*sid),
                _ => false,
            },
        )
        .map(|(id, _)| id)
        .collect()
}

/// A value worked out into a local the next statement only copies or turns
/// into a list is that statement's own operand: a copy of a fresh value is the value.
fn forward(mut b: Block) -> Block {
    let mut i = 0;
    while i + 1 < b.stmts.len() {
        let fresh = match (&b.stmts[i], &b.stmts[i + 1]) {
            (
                Stmt::Let(t, Some(_)),
                Stmt::Let(
                    _,
                    Some(Expr {
                        kind: ExprKind::Builtin(op, src),
                        ..
                    }),
                ),
            ) if matches!(op, Builtin::Copy | Builtin::ToList | Builtin::ArrayToList)
                && matches!(src.as_slice(), [x] if is_local(x, *t)) =>
            {
                local_reads(&b, *t) == 1
            }
            _ => false,
        };
        if fresh {
            let (Stmt::Let(_, Some(value)), Stmt::Let(_, Some(init))) =
                (b.stmts.remove(i), &mut b.stmts[i])
            else {
                i += 1;
                continue;
            };
            {
                match &mut init.kind {
                    ExprKind::Builtin(Builtin::Copy, _) => *init = value,
                    ExprKind::Builtin(_, src) => src[0] = value,
                    _ => {}
                }
            }
        }
        i += 1;
    }
    b
}

fn is_local(e: &Expr, slot: Slot) -> bool {
    matches!(e.kind, ExprKind::Local(s) if s == slot)
}

fn copied(b: &Block) -> Option<ExprKind> {
    let [Stmt::Let(acc, Some(empty)), Stmt::For(_, item, Iter::Seq(src, false), body), Stmt::Give(out)] =
        b.stmts.as_slice()
    else {
        return None;
    };
    let [Stmt::Expr(Expr {
        kind: ExprKind::Mutate(Mutation::Push, place, pushed),
        ..
    })] = body.stmts.as_slice()
    else {
        return None;
    };
    let pushes_item = matches!(place.kind, PlaceKind::Local(s) if s == *acc)
        && matches!(pushed.as_slice(), [x] if is_local(x, *item));
    if !(matches!(&empty.kind, ExprKind::Tuple(v) if v.is_empty())
        && pushes_item
        && is_local(out, *acc))
    {
        return None;
    }
    let op = match src.ty {
        Ty::List(_) => Builtin::Copy,
        Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(..) => Builtin::ToList,
        Ty::Array { .. } => Builtin::ArrayToList,
        _ => return None,
    };
    Some(ExprKind::Builtin(op, vec![src.clone()]))
}

fn given(b: &Block) -> Option<ExprKind> {
    match b.stmts.as_slice() {
        [Stmt::Let(v, Some(init)), Stmt::Give(out)] if is_local(out, *v) => Some(init.kind.clone()),
        _ => None,
    }
}

fn sorted(b: &Block) -> Option<ExprKind> {
    let [Stmt::Let(v, Some(init)), Stmt::Expr(Expr {
        kind: ExprKind::Sort(place, SortBy::Natural, None),
        ..
    }), Stmt::Give(out)] = b.stmts.as_slice()
    else {
        return None;
    };
    if !(matches!(place.kind, PlaceKind::Local(s) if s == *v) && is_local(out, *v)) {
        return None;
    }
    match &init.kind {
        ExprKind::Builtin(Builtin::Copy, src) => {
            Some(ExprKind::Builtin(Builtin::Sorted, src.clone()))
        }
        ExprKind::Local(_) | ExprKind::Global(_) => None,
        _ => Some(ExprKind::Builtin(Builtin::Sorted, vec![init.clone()])),
    }
}

/// A statement that is a unit `Do` giving nothing is its statements.
fn flatten(b: &mut Block) {
    if !b.stmts.iter().any(|s| {
        matches!(
            s,
            Stmt::Expr(Expr {
                kind: ExprKind::Do(_),
                ty: Ty::Unit,
                ..
            })
        )
    }) {
        return;
    }
    let stmts = std::mem::take(&mut b.stmts);
    for s in stmts {
        match s {
            Stmt::Expr(Expr {
                kind: ExprKind::Do(inner),
                ty: Ty::Unit,
                ..
            }) if !inner.stmts.iter().any(|s| matches!(s, Stmt::Give(_))) => {
                b.stmts.extend(inner.stmts)
            }
            s => b.stmts.push(s),
        }
    }
}

fn each_expr_mut(b: &mut Block, f: &mut dyn FnMut(&mut Expr)) {
    for stmt in &mut b.stmts {
        cpl_ir::walk::stmt_exprs_mut(stmt, &mut |e| expr_mut(e, f));
        cpl_ir::walk::stmt_blocks_mut(stmt, &mut |inner| each_expr_mut(inner, f));
    }
}

fn expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    if let ExprKind::Do(b) = &mut e.kind {
        each_expr_mut(b, f);
    }
    cpl_ir::walk::children_mut(e, &mut |c| expr_mut(c, f));
    f(e);
}
