//! performance/R13. A fn parameter `cmp: ?F` that every call fills the same way is known
//! inside the fn. When every call passes `None`, the parameter goes and
//! each read of it is `None`. When every call passes `Some(f)`, the
//! parameter holds `f` itself, so a closure argument keeps its own type
//! instead of a `std::function`. Then `if let c = cmp` picks its branch
//! and `c` is the parameter. It majorizes: the same expressions run in the
//! same order, and a name for a value no one changes is that value.

use crate::walk;
use cpl_ir::{
    Arg, Block, ClosureId, CmpOp, Expr, ExprKind, FnId, Iter, Local, Program, Slot, Stmt, Ty,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    None,
    Some,
    Mixed,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    Fn(FnId),
    Closure(ClosureId),
}

pub(crate) fn known_options(program: &mut Program) {
    // Each round may make a forwarded `Some(cmp)` known one call deeper.
    for _ in 0..4 {
        let known = shapes(program);
        if known.is_empty() {
            return;
        }
        let mut touched = BTreeSet::new();
        for (&(id, slot), &shape) in &known {
            rewrite_calls(program, id, slot, shape);
            rewrite_param(program, id, slot, shape, &mut touched);
        }
        for owner in touched {
            fold(program, owner);
        }
    }
}

/// `(fn, slot)` of each `?F` parameter whose every call fills it the same way.
fn shapes(program: &Program) -> BTreeMap<(FnId, Slot), Shape> {
    let mut out: BTreeMap<(FnId, Slot), Shape> = BTreeMap::new();
    let mut walked: BTreeSet<FnId> = BTreeSet::new();
    let mut see = |block: &Block, out: &mut BTreeMap<(FnId, Slot), Shape>| {
        walk::deep_stmts(block, &mut |s| {
            if let Stmt::For(_, _, Iter::Walk { nth, .. }, _) = s {
                walked.insert(*nth);
            }
        });
        walk::deep_exprs(block, &mut |e| {
            let ExprKind::Call(callee, args) = &e.kind else {
                return;
            };
            let f = &program.fns[*callee];
            for (slot, arg) in args.iter().enumerate().take(f.params.len()) {
                if !optional_fn(&f.locals[slot].ty) {
                    continue;
                }
                let shape = match arg {
                    Arg::Value(Expr {
                        kind: ExprKind::None,
                        ..
                    }) => Shape::None,
                    Arg::Value(Expr {
                        kind: ExprKind::Some(_),
                        ..
                    }) => Shape::Some,
                    _ => Shape::Mixed,
                };
                let seen = out.entry((*callee, slot)).or_insert(shape);
                if *seen != shape {
                    *seen = Shape::Mixed;
                }
            }
        });
    };
    see(&program.setup, &mut out);
    see(&program.main, &mut out);
    for f in &program.fns {
        see(&f.body, &mut out);
    }
    for c in &program.closures {
        see(&c.body, &mut out);
    }
    out.retain(|&(id, slot), shape| {
        *shape != Shape::Mixed
            && !program.fns[id].params[slot].mutable
            && !program.entry_fns.contains(&id)
            && !program.judge_fns.contains(&id)
            && !program.is_hook(id)
            && !walked.contains(&id)
            && !assigned(&program.fns[id].body).contains(&slot)
    });
    out
}

fn optional_fn(ty: &Ty) -> bool {
    matches!(ty, Ty::Opt(inner) if matches!(**inner, Ty::Fn(..)))
}

fn rewrite_calls(program: &mut Program, id: FnId, slot: Slot, shape: Shape) {
    let fix = |block: &mut Block| {
        exprs_mut(block, &mut |e| {
            let ExprKind::Call(callee, args) = &mut e.kind else {
                return;
            };
            if *callee != id {
                return;
            }
            let Some(Arg::Value(arg)) = args.get_mut(slot) else {
                return;
            };
            let span = arg.span;
            *arg = match (shape, std::mem::replace(&mut arg.kind, ExprKind::Unit)) {
                (Shape::Some, ExprKind::Some(inner)) => *inner,
                _ => unit(span),
            };
        });
    };
    fix(&mut program.setup);
    fix(&mut program.main);
    for f in &mut program.fns {
        fix(&mut f.body);
    }
    for c in &mut program.closures {
        fix(&mut c.body);
    }
}

fn unit(span: cpl_diag::Span) -> Expr {
    Expr {
        kind: ExprKind::Unit,
        ty: Ty::Unit,
        span,
    }
}

/// The parameter, and every closure capture that copies it, now holds the
/// fn (`Some`) or nothing (`None`); each read gives back the optional.
fn rewrite_param(
    program: &mut Program,
    id: FnId,
    slot: Slot,
    shape: Shape,
    touched: &mut BTreeSet<Owner>,
) {
    let mut work = vec![(Owner::Fn(id), slot)];
    while let Some((owner, slot)) = work.pop() {
        touched.insert(owner);
        let (locals, body) = match owner {
            Owner::Fn(f) => {
                let f = &mut program.fns[f];
                (&mut f.locals, &mut f.body)
            }
            Owner::Closure(c) => {
                let c = &mut program.closures[c];
                (&mut c.locals, &mut c.body)
            }
        };
        let opt = locals[slot].ty.clone();
        let Ty::Opt(inner) = &opt else {
            continue;
        };
        locals[slot].ty = match shape {
            Shape::Some => (**inner).clone(),
            _ => Ty::Unit,
        };
        exprs_mut(body, &mut |e| {
            if e.kind == ExprKind::Local(slot) {
                let span = e.span;
                *e = match shape {
                    Shape::Some => Expr {
                        kind: ExprKind::Some(Box::new(Expr {
                            kind: ExprKind::Local(slot),
                            ty: (**inner).clone(),
                            span,
                        })),
                        ty: opt.clone(),
                        span,
                    },
                    _ => Expr {
                        kind: ExprKind::None,
                        ty: opt.clone(),
                        span,
                    },
                };
            }
        });
        let made: Vec<ClosureId> = closures_in(body).into_iter().collect();
        for cid in made {
            let captures = &mut program.closures[cid].captures;
            for cap in captures.iter().filter(|c| !c.is_global && c.outer == slot) {
                work.push((Owner::Closure(cid), cap.inner));
            }
            if shape == Shape::None {
                captures.retain(|c| c.is_global || c.outer != slot);
            }
        }
    }
}

fn closures_in(body: &Block) -> BTreeSet<ClosureId> {
    let mut out = BTreeSet::new();
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            out.insert(cid);
        }
    });
    out
}

/// Local slots a statement of `body` stores to, loops over, or passes `mut`.
fn assigned(body: &Block) -> BTreeSet<Slot> {
    let mut out = BTreeSet::new();
    walk::deep_stmts(body, &mut |s| match s {
        Stmt::Assign(place, ..) => {
            if let (false, slot) = walk::root(place) {
                out.insert(slot);
            }
        }
        Stmt::For(_, slot, ..) => {
            out.insert(*slot);
        }
        _ => {}
    });
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) = &e.kind {
            for arg in args {
                if let Arg::Mut(place) = arg {
                    if let (false, slot) = walk::root(place) {
                        out.insert(slot);
                    }
                }
            }
        }
    });
    out
}

/// After the rewrite: `let t = Some(p)` or `let t = None` is `t`'s value
/// everywhere, `t.is_some()` and `t!` of a known optional are worked out,
/// a branch on a known condition keeps the branch it takes, and
/// `let c = p` of a fn no one stores to is `p`.
fn fold(program: &mut Program, owner: Owner) {
    let (body, fixed): (&Block, BTreeSet<Slot>) = match owner {
        Owner::Fn(f) => {
            let f = &program.fns[f];
            let fixed = (0..f.params.len())
                .filter(|&s| !f.params[s].mutable)
                .collect();
            (&f.body, fixed)
        }
        Owner::Closure(c) => {
            let def = &program.closures[c];
            let fixed = def
                .captures
                .iter()
                .filter(|c| !c.is_global && !c.by_ref)
                .map(|c| c.inner)
                .chain(def.params.iter().copied().filter(|p| !def.muts.contains(p)))
                .collect();
            (&def.body, fixed)
        }
    };
    let captured: BTreeSet<Slot> = closures_in(body)
        .into_iter()
        .flat_map(|cid| program.closures[cid].captures.iter())
        .filter(|c| !c.is_global)
        .map(|c| c.outer)
        .collect();
    let stored = assigned(body);
    let (locals, body): (&[Local], &mut Block) = match owner {
        Owner::Fn(f) => {
            let f = &mut program.fns[f];
            (&f.locals, &mut f.body)
        }
        Owner::Closure(c) => {
            let c = &mut program.closures[c];
            (&c.locals, &mut c.body)
        }
    };
    let steady = |s: Slot| fixed.contains(&s) && !stored.contains(&s);
    let known = |t: Slot, init: &Expr| {
        let steady_local = |e: &Expr| matches!(e.kind, ExprKind::Local(p) if steady(p));
        match &init.kind {
            ExprKind::None => true,
            ExprKind::Some(inner) => steady_local(inner),
            ExprKind::Local(_) => matches!(locals[t].ty, Ty::Fn(..)) && steady_local(init),
            _ => false,
        }
    };
    loop {
        let mut subst: BTreeMap<Slot, Expr> = BTreeMap::new();
        walk::blocks_mut(body, &mut |block| {
            for st in &mut block.stmts {
                let folded = match st {
                    Stmt::Let(t, Some(init))
                        if !stored.contains(t) && !captured.contains(t) && known(*t, init) =>
                    {
                        Some((*t, init.clone()))
                    }
                    _ => None,
                };
                // The `let` still runs where it is written, as a profile counts it.
                if let Some((t, init)) = folded {
                    *st = Stmt::Expr(unit(init.span));
                    subst.insert(t, init);
                }
            }
        });
        exprs_mut(body, &mut |e| {
            if let ExprKind::Local(s) = e.kind {
                if let Some(v) = subst.get(&s) {
                    let span = e.span;
                    *e = v.clone();
                    e.span = span;
                }
            }
            fold_expr(e);
        });
        walk::blocks_mut(body, &mut |block| {
            for st in &mut block.stmts {
                if let Stmt::If(cond, then, other) = st {
                    if let ExprKind::Bool(taken) = cond.kind {
                        if !taken || !other.stmts.is_empty() {
                            let chosen = if taken {
                                std::mem::take(then)
                            } else {
                                std::mem::take(other)
                            };
                            *st = Stmt::If(
                                Expr {
                                    kind: ExprKind::Bool(true),
                                    ..cond.clone()
                                },
                                chosen,
                                Block::default(),
                            );
                        }
                    }
                }
            }
        });
        if subst.is_empty() {
            break;
        }
    }
}

fn fold_expr(e: &mut Expr) {
    let folded = match &mut e.kind {
        ExprKind::IsSome(x) => match &x.kind {
            ExprKind::None => Some(ExprKind::Bool(false)),
            ExprKind::Some(inner) if matches!(inner.kind, ExprKind::Local(_)) => {
                Some(ExprKind::Bool(true))
            }
            _ => None,
        },
        ExprKind::Cmp(op @ (CmpOp::Eq | CmpOp::Ne), x, none) if none.kind == ExprKind::None => {
            let some = match &x.kind {
                ExprKind::None => false,
                ExprKind::Some(inner) if matches!(inner.kind, ExprKind::Local(_)) => true,
                _ => return,
            };
            Some(ExprKind::Bool(some == (*op == CmpOp::Ne)))
        }
        ExprKind::Unwrap(x, _) => match &mut x.kind {
            ExprKind::Some(inner) => {
                *e = std::mem::replace(&mut **inner, unit(e.span));
                return;
            }
            _ => None,
        },
        ExprKind::If(cond, then, other) => match cond.kind {
            ExprKind::Bool(taken) => {
                let chosen = if taken { &mut **then } else { &mut **other };
                *e = std::mem::replace(chosen, unit(e.span));
                return;
            }
            _ => None,
        },
        _ => None,
    };
    if let Some(kind) = folded {
        e.kind = kind;
    }
}

/// Calls `f` on every expression in `block`, at any depth, inner first.
fn exprs_mut(block: &mut Block, f: &mut dyn FnMut(&mut Expr)) {
    for stmt in &mut block.stmts {
        walk::stmt_exprs_mut(stmt, &mut |e| expr_mut(e, f));
        walk::stmt_blocks_mut(stmt, &mut |inner| exprs_mut(inner, f));
    }
}

fn expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    if let ExprKind::Do(block) = &mut e.kind {
        exprs_mut(block, f);
    }
    walk::children_mut(e, &mut |child| expr_mut(child, f));
    f(e);
}
