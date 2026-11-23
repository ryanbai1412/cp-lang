//! A loop over a call to a fn value that can only be one closure, whose
//! body builds and returns a list, runs that body in place:
//! `for a in g(u)` with `g = c => [q for q in c.adj4 if ..]` becomes
//! `for a in (do { let v = []; for q in u.adj4 { if ..: v.push(q) }; give v })`,
//! which `crate::fuse` then runs without building the list. It majorizes:
//! the same work in the same order, minus a call.

use crate::devirt::{skippable, Devirt};
use crate::walk;
use cpl_ir::{
    Arg, Block, ClosureDef, ClosureId, Expr, ExprKind, FnId, Iter, Local, LoopId, Place, PlaceKind,
    Program, Slot, Stmt,
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

pub(crate) fn splice(program: &mut Program) {
    if program.closures.is_empty() {
        return;
    }
    let mut found: BTreeMap<(Owner, LoopId), ClosureId> = BTreeMap::new();
    {
        let devirt = Devirt::collect(program);
        let mut scan = |owner: Owner, home: Option<FnId>, body: &Block| {
            walk::deep_stmts(body, &mut |stmt| {
                if let Stmt::For(id, _, Iter::Seq(seq, false), _) = stmt {
                    if let ExprKind::CallExpr(callee, _) = &seq.kind {
                        if let Some(cid) = devirt.target(home, callee) {
                            if skippable(callee) {
                                found.insert((owner, *id), cid);
                            }
                        }
                    }
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
    found.retain(|_, cid| producer_body(&program.closures[*cid]).is_some());
    if found.is_empty() {
        return;
    }
    let mut next_loop = max_loop(program) + 1;
    let closures = program.closures.clone();
    let mut run = |owner: Owner, body: &mut Block, locals: &mut Vec<Local>| {
        let mut todo = |id: LoopId, seq: &mut Expr| {
            let Some(&cid) = found.get(&(owner, id)) else {
                return;
            };
            let ExprKind::CallExpr(_, args) = &seq.kind else {
                return;
            };
            let c = &closures[cid];
            let Some(mut block) = producer_body(c) else {
                return;
            };
            if args.len() != c.params.len() {
                return;
            }
            let mut ren = Ren {
                map: Vec::with_capacity(c.locals.len()),
                consts: Vec::with_capacity(args.len()),
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
            for i in c.params.len()..c.locals.len() {
                ren.map.push(base + i - c.params.len());
            }
            if !ren.block(&mut block) {
                return;
            }
            next_loop = ren.next;
            locals.extend(c.locals[c.params.len()..].iter().cloned());
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

/// The list-building block closure `cid` returns, when it takes its
/// arguments by value, captures nothing and is small.
fn producer_body(c: &ClosureDef) -> Option<Block> {
    if !c.captures.is_empty() || !c.muts.is_empty() {
        return None;
    }
    let block = match c.body.stmts.as_slice() {
        [Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Expr(e)] => match &e.kind {
            ExprKind::Do(b) => b.clone(),
            _ => return None,
        },
        [Stmt::Let(v, Some(_)), Stmt::For(..), Stmt::Return(Some(e)) | Stmt::Give(e)]
            if e.kind == ExprKind::Local(*v) =>
        {
            let mut stmts = c.body.stmts.clone();
            stmts[2] = Stmt::Give(e.clone());
            Block { stmts }
        }
        _ => return None,
    };
    let mut count = 0;
    walk::deep_stmts(&block, &mut |_| count += 1);
    let seq = Expr {
        kind: ExprKind::Do(block),
        ty: c.ret.clone(),
        span: Default::default(),
    };
    if count > MAX_STMTS || crate::fuse::producer(&seq).is_none() {
        return None;
    }
    let ExprKind::Do(block) = seq.kind else {
        unreachable!()
    };
    Some(block)
}

fn max_loop(program: &Program) -> LoopId {
    let mut max = 0;
    let mut see = |b: &Block| {
        walk::deep_stmts(b, &mut |s| match s {
            Stmt::For(id, ..) | Stmt::While(id, ..) => max = max.max(*id),
            _ => {}
        })
    };
    see(&program.main);
    see(&program.setup);
    program.fns.iter().for_each(|f| see(&f.body));
    program.closures.iter().for_each(|c| see(&c.body));
    max
}

/// Calls `f` on the id and sequence of each `for x in seq` in `b`.
fn for_loops_mut(b: &mut Block, f: &mut dyn FnMut(LoopId, &mut Expr)) {
    for stmt in &mut b.stmts {
        match stmt {
            Stmt::For(id, _, iter, body) => {
                if let Iter::Seq(seq, false) = iter {
                    f(*id, seq);
                }
                for_loops_mut(body, f);
            }
            Stmt::While(_, _, body) => for_loops_mut(body, f),
            Stmt::If(_, then, otherwise) => {
                for_loops_mut(then, f);
                for_loops_mut(otherwise, f);
            }
            _ => {}
        }
    }
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
                    Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
                    Iter::Range { lo, hi, step, .. } => {
                        self.expr(lo) && self.expr(hi) && step.as_mut().is_none_or(|s| self.expr(s))
                    }
                    Iter::RangeVal(e, _)
                    | Iter::Seq(e, _)
                    | Iter::Subsets(e)
                    | Iter::Members(e) => self.expr(e),
                    Iter::TreeRange { .. } => false,
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
