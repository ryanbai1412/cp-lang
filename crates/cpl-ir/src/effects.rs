//! What a call can do beyond computing its value: which globals it writes,
//! whether it does I/O, whether it may trap, and whether it calls a fn value
//! whose target is unknown. The one source both back ends and the tools ask.

use crate::pushed::{summarize, FnSummary};
use crate::walk::{self, Effects as Stores};
use crate::{
    Arg, BinOp, Block, Builtin, Cast, ClosureId, CmpOp, Dim, Expr, ExprKind, FnId, Iter, Mutation,
    PlaceKind, Program, Stmt, Ty, UnOp,
};
use std::collections::BTreeSet;

/// What calling a fn or closure can do, through everything it calls.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Effects {
    /// The globals it may write.
    pub writes: BTreeSet<usize>,
    /// It reads input, writes output or sizes a domain.
    pub io: bool,
    /// It is not known to be pure: it may trap in this build, read a global
    /// or take a `mut` parameter.
    pub traps: bool,
    /// It calls a fn value whose target is not known, which may write what
    /// that value captured.
    pub unknown_call: bool,
}

/// The closure a fn value called in `home` (`None` for a closure body or
/// top-level code) is known to be, when it is.
pub type Target<'a> = &'a dyn Fn(Option<FnId>, &Expr) -> Option<ClosureId>;

/// Each fn's [`Effects`], computed once per program.
#[derive(Clone, Default, Debug)]
pub struct Table {
    fns: Vec<Effects>,
    summaries: Vec<FnSummary>,
    pure: BTreeSet<FnId>,
    traps: bool,
}

impl Table {
    /// The table for a build that checks (`traps`) or not, knowing no fn
    /// value's target.
    pub fn of(program: &Program, traps: bool) -> Table {
        Table::with_targets(program, traps, &|_, _| None)
    }

    /// The table, with `target` naming the closures fn values are known to be.
    pub fn with_targets(program: &Program, traps: bool, target: Target) -> Table {
        let summaries = summarize(program);
        let pure = pure_fns(program, traps, target);
        let fns = (0..program.fns.len())
            .map(|f| of_summary(&summaries[f], !pure.contains(&f)))
            .collect();
        Table {
            fns,
            summaries,
            pure,
            traps,
        }
    }

    /// What calling fn `f` can do.
    pub fn effects(&self, f: FnId) -> &Effects {
        &self.fns[f]
    }

    /// The fns known to be pure: [`Effects::traps`] is false for these.
    pub fn pure(&self) -> &BTreeSet<FnId> {
        &self.pure
    }

    /// What calling closure `cid` can do: its body's effects and those of
    /// the fns it calls. A fn value it calls may run any closure.
    pub fn closure_effects(&self, program: &Program, cid: ClosureId) -> Effects {
        let c = &program.closures[cid];
        let stores = Stores::of_block(&c.body);
        let mut summary = FnSummary {
            writes: stores.globals,
            opaque: c.captures.iter().any(|cap| cap.by_ref),
            ..FnSummary::default()
        };
        walk::deep_exprs(&c.body, &mut |e| match &e.kind {
            ExprKind::Read(_) | ExprKind::Print(..) => summary.io = true,
            ExprKind::CallExpr(..) => summary.merge(&self.summaries[program.fns.len()]),
            _ => {}
        });
        walk::deep_stmts(&c.body, &mut |stmt| {
            summary.io |= matches!(stmt, Stmt::DefDomain { .. });
        });
        for f in stores.calls {
            summary.merge(&self.summaries[f]);
        }
        let ok = |e: &Expr| pure_call(program, &|_, _| None, &self.pure, None, self.traps, e);
        let pure = c.muts.is_empty() && pure_body(&c.body, self.traps, &ok);
        of_summary(&summary, !pure)
    }
}

fn of_summary(s: &FnSummary, traps: bool) -> Effects {
    Effects {
        writes: s.writes.clone(),
        io: s.io,
        traps,
        unknown_call: s.opaque || (s.calls_values && traps),
    }
}

/// The fns that can neither trap, loop forever nor be seen: no mutable
/// parameter, no global read, and a body of trap-free statements whose only
/// calls are to such fns or to closures that are. Built bottom-up, so a
/// recursive fn is never in it.
pub fn pure_fns(program: &Program, traps: bool, target: Target) -> BTreeSet<FnId> {
    let mut pure = BTreeSet::new();
    loop {
        let found: Vec<usize> = (0..program.fns.len())
            .filter(|fid| !pure.contains(fid))
            .filter(|&fid| {
                let f = &program.fns[fid];
                !f.params.iter().any(|p| p.mutable)
                    && !program.judge_fns.contains(&fid)
                    && !program.entry_fns.contains(&fid)
                    && pure_body(&f.body, traps, &|e| {
                        pure_call(program, target, &pure, Some(fid), traps, e)
                    })
            })
            .collect();
        if found.is_empty() {
            return pure;
        }
        pure.extend(found);
    }
}

/// Trap-free statements up to a `return` of a trap-free value, reading no
/// global; of an `if` decided at compile time only the branch taken counts.
fn pure_body(body: &Block, traps: bool, call_ok: &dyn Fn(&Expr) -> bool) -> bool {
    let mut global = false;
    walk::deep_exprs(body, &mut |e| {
        global |= matches!(e.kind, ExprKind::Global(_))
    });
    !global && pure_stmts(&body.stmts, traps, call_ok)
}

fn pure_stmts(stmts: &[Stmt], traps: bool, call_ok: &dyn Fn(&Expr) -> bool) -> bool {
    let Some((first, rest)) = stmts.split_first() else {
        return true;
    };
    match first {
        Stmt::Return(Some(r)) => trap_free_with(r, traps, call_ok),
        Stmt::If(cond, then, otherwise) if known(cond).is_some() => {
            let taken = if known(cond) == Some(true) {
                then
            } else {
                otherwise
            };
            pure_stmts(&taken.stmts, traps, call_ok)
                && (matches!(taken.stmts.last(), Some(Stmt::Return(_)))
                    || pure_stmts(rest, traps, call_ok))
        }
        _ => {
            trap_free_stmts(std::slice::from_ref(first), usize::MAX, traps, call_ok)
                && pure_stmts(rest, traps, call_ok)
        }
    }
}

/// A call of a fn in `pure`, or of a closure known from `home` to capture
/// nothing and have a pure body, with trap-free arguments.
pub fn pure_call(
    program: &Program,
    target: Target,
    pure: &BTreeSet<FnId>,
    home: Option<FnId>,
    traps: bool,
    e: &Expr,
) -> bool {
    let inner = |x: &Expr| pure_call(program, target, pure, None, traps, x);
    let (known, args) = match &e.kind {
        ExprKind::Call(fid, args) => (pure.contains(fid), args),
        ExprKind::CallExpr(callee, args) => {
            let here = |x: &Expr| pure_call(program, target, pure, home, traps, x);
            let known = trap_free_with(callee, traps, &here)
                && target(home, callee).is_some_and(|cid| {
                    let c = &program.closures[cid];
                    c.muts.is_empty() && c.captures.is_empty() && pure_body(&c.body, traps, &inner)
                });
            (known, args)
        }
        _ => return false,
    };
    let here = |x: &Expr| pure_call(program, target, pure, home, traps, x);
    known
        && args
            .iter()
            .all(|a| matches!(a, Arg::Value(v) if trap_free_with(v, traps, &here)))
}

/// What `e` is at compile time, when that is known.
pub fn known(e: &Expr) -> Option<bool> {
    known_where(e, &|_| false)
}

/// `trap_free_block` over `stmts`, where `call_ok` says which calls cannot trap.
pub fn trap_free_stmts(
    stmts: &[Stmt],
    vec: usize,
    traps: bool,
    call_ok: &dyn Fn(&Expr) -> bool,
) -> bool {
    let free = |e: &Expr| trap_free_with(e, traps, call_ok);
    let block_free = |b: &Block| trap_free_stmts(&b.stmts, vec, traps, call_ok);
    stmts.iter().all(|stmt| match stmt {
        Stmt::Let(_, init) => init.as_ref().is_none_or(free),
        Stmt::Assign(place, _, val) => matches!(place.kind, PlaceKind::Local(_)) && free(val),
        Stmt::Expr(e) => match &e.kind {
            ExprKind::Mutate(Mutation::Push, place, args) if matches!(place.kind, PlaceKind::Local(s) if s == vec) => {
                args.iter().all(free)
            }
            _ => free(e),
        },
        Stmt::If(cond, then, otherwise) => match known(cond) {
            Some(taken) => block_free(if taken { then } else { otherwise }),
            None => free(cond) && block_free(then) && block_free(otherwise),
        },
        Stmt::For(_, _, Iter::Range { lo, hi, step: None, .. }, body) => {
            free(lo) && free(hi) && block_free(body)
        }
        Stmt::For(_, _, Iter::Seq(seq, _), body) => {
            matches!(seq.kind, ExprKind::Local(_)) && block_free(body)
        }
        Stmt::Continue(None) => true,
        _ => false,
    })
}

/// True when `e` cannot trap: no checked index, call, unwrap or division by
/// what may be zero. A build without `traps` (`--checks none`) checks no
/// index and gives 0 for `/ 0`, so only calls and unwraps are left.
pub fn trap_free_with(e: &Expr, traps: bool, call_ok: &dyn Fn(&Expr) -> bool) -> bool {
    let free = |x: &Expr| trap_free_with(x, traps, call_ok);
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Some(x)
        | ExprKind::Field(x, _)
        | ExprKind::IsSome(x)
        | ExprKind::IsVariant(x, _)
        | ExprKind::Unwrap(x, false)
        | ExprKind::Cast(Cast::IntToFloat, x) => free(x),
        ExprKind::Unary(op, x) => *op != UnOp::Neg && free(x),
        ExprKind::Binary(op, l, r) => {
            let safe = match op {
                BinOp::Div | BinOp::Rem => !traps || matches!(r.kind, ExprKind::Int(k) if k != 0),
                BinOp::Pow | BinOp::Shl | BinOp::Shr => false,
                _ => true,
            };
            safe && l.ty != Ty::Str && free(l) && free(r)
        }
        ExprKind::Cmp(_, l, r) => free(l) && free(r),
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().all(free),
        ExprKind::If(c, t, f) => match known(c) {
            Some(taken) => free(if taken { t } else { f }),
            None => free(c) && free(t) && free(f),
        },
        ExprKind::Closure(_) => true,
        ExprKind::Index(base, idxs) if !traps || in_domain(&base.ty, idxs) => {
            free(base) && idxs.iter().all(free)
        }
        _ => call_ok(e),
    }
}

/// Indexing an array of type `ty` by `idxs` cannot trap: each index is a
/// position of the domain its axis is over.
pub fn in_domain(ty: &Ty, idxs: &[Expr]) -> bool {
    let Ty::Array { dims, .. } = ty else {
        return false;
    };
    dims.len() == idxs.len()
        && dims.iter().zip(idxs).all(|(dim, idx)| {
            matches!((dim, &idx.ty), (Dim::Pos(d), Ty::Pos(e)) if d == e) && pure_path(idx)
        })
}

pub fn pure_path(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) => pure_path(base),
        _ => false,
    }
}

/// What `e` is at compile time, when that is known; `is_none` names the
/// optionals known to be absent.
pub fn known_where(e: &Expr, is_none: &dyn Fn(&Expr) -> bool) -> Option<bool> {
    match &e.kind {
        ExprKind::IsSome(x) if is_none(x) => Some(false),
        ExprKind::Bool(b) => Some(*b),
        ExprKind::Cmp(op, a, b) => {
            let (ExprKind::Int(a), ExprKind::Int(b)) = (&a.kind, &b.kind) else {
                return None;
            };
            match op {
                CmpOp::Eq => Some(a == b),
                CmpOp::Ne => Some(a != b),
                CmpOp::Lt => Some(a < b),
                CmpOp::Le => Some(a <= b),
                CmpOp::Gt => Some(a > b),
                CmpOp::Ge => Some(a >= b),
                _ => None,
            }
        }
        ExprKind::Unary(UnOp::Not, x) => known_where(x, is_none).map(|b| !b),
        ExprKind::Binary(BinOp::And, a, b) => match known_where(a, is_none)? {
            false => Some(false),
            true => known_where(b, is_none),
        },
        ExprKind::Binary(BinOp::Or, a, b) => match known_where(a, is_none)? {
            true => Some(true),
            false => known_where(b, is_none),
        },
        _ => None,
    }
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod tests;

/// What running an expression or block may touch, through the fns it
/// calls: the variables it reads and writes, by `(global, index)`, the
/// domains it reads and sizes, and whether it does I/O or runs code that
/// is not seen. Locals of the code itself count as written where bound.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Access {
    /// Reads input, writes output, sizes a domain or may stop the run.
    pub io: bool,
    /// Calls a fn value it can't see into, which may read or write
    /// anything it captured.
    pub opaque: bool,
    pub reads: BTreeSet<(bool, usize)>,
    pub writes: BTreeSet<(bool, usize)>,
    pub reads_domain: BTreeSet<usize>,
    pub writes_domain: BTreeSet<usize>,
}

impl Access {
    /// True when running `self` before `later` can differ from after.
    pub fn conflicts(&self, later: &Access) -> bool {
        let effects =
            |t: &Access| t.io || t.opaque || !t.writes.is_empty() || !t.writes_domain.is_empty();
        (self.io && later.io)
            || (self.opaque && effects(later))
            || (later.opaque && effects(self))
            || self
                .writes
                .iter()
                .any(|v| later.reads.contains(v) || later.writes.contains(v))
            || later.writes.iter().any(|v| self.reads.contains(v))
            || self
                .writes_domain
                .iter()
                .any(|d| later.reads_domain.contains(d) || later.writes_domain.contains(d))
            || later
                .writes_domain
                .iter()
                .any(|d| self.reads_domain.contains(d))
    }

    /// Adds what `other` touches.
    pub fn merge(&mut self, other: Access) {
        self.io |= other.io;
        self.opaque |= other.opaque;
        self.reads.extend(other.reads);
        self.writes.extend(other.writes);
        self.reads_domain.extend(other.reads_domain);
        self.writes_domain.extend(other.writes_domain);
    }
}

impl Table {
    /// What evaluating `e` in `home` touches. A fn value `target` names
    /// as one closure that captures nothing, takes nothing `mut` and calls
    /// no fn value counts as the globals that closure touches; any other
    /// is opaque.
    pub fn access(
        &self,
        program: &Program,
        home: Option<FnId>,
        target: Target,
        e: &Expr,
    ) -> Access {
        let mut touch = Access::default();
        let mut_args = |args: &[Arg], touch: &mut Access| {
            for arg in args {
                if let Arg::Mut(place) = arg {
                    touch.writes.insert(walk::root(place));
                }
            }
        };
        walk::deep_expr(e, &mut |inner| match &inner.kind {
            ExprKind::Local(slot) => {
                touch.reads.insert((false, *slot));
            }
            ExprKind::Global(id) => {
                touch.reads.insert((true, *id));
            }
            ExprKind::Read(_)
            | ExprKind::Parse(..)
            | ExprKind::Print(..)
            | ExprKind::Builtin(Builtin::CountQuery | Builtin::SwapInput | Builtin::AtEof, _) => {
                touch.io = true
            }
            ExprKind::DomainLen(d)
            | ExprKind::DomainLo(d)
            | ExprKind::Cast(Cast::Label(d) | Cast::FromLabel(d) | Cast::FromOrd(d), _) => {
                touch.reads_domain.insert(*d);
            }
            ExprKind::CallExpr(callee, args) => {
                match self.closure_access(program, target(home, callee)) {
                    Some(t) => touch.merge(t),
                    None => {
                        touch.io = true;
                        touch.opaque = true;
                    }
                }
                mut_args(args, &mut touch);
            }
            ExprKind::Call(id, args) => {
                let summary = &self.summaries[*id];
                touch.io |= summary.io;
                touch.reads.extend(summary.reads.iter().map(|g| (true, *g)));
                touch
                    .writes
                    .extend(summary.writes.iter().map(|g| (true, *g)));
                mut_args(args, &mut touch);
            }
            ExprKind::MutSlice(place, _, _)
            | ExprKind::Mutate(_, place, _)
            | ExprKind::Sort(place, _, _) => {
                touch.writes.insert(walk::root(place));
            }
            ExprKind::Do(block) => walk::deep_stmts(block, &mut |stmt| match stmt {
                Stmt::Assign(place, ..) => {
                    touch.writes.insert(walk::root(place));
                }
                Stmt::Let(slot, _) | Stmt::For(_, slot, ..) => {
                    touch.writes.insert((false, *slot));
                }
                Stmt::DefDomain { id, .. } => {
                    touch.writes_domain.insert(*id);
                    touch.io = true;
                }
                Stmt::Assert { .. } | Stmt::Trap(..) => touch.io = true,
                _ => {}
            }),
            _ => {}
        });
        touch
    }

    /// What running `block` in `home` touches: [`Table::access`] of each
    /// expression it runs, and the variables and domains it binds or sets.
    pub fn block_access(
        &self,
        program: &Program,
        home: Option<FnId>,
        target: Target,
        block: &Block,
    ) -> Access {
        let mut t = Access::default();
        walk::deep_stmts(block, &mut |stmt| {
            walk::stmt_exprs(stmt, &mut |e| {
                t.merge(self.access(program, home, target, e))
            });
            match stmt {
                Stmt::Let(slot, _) | Stmt::For(_, slot, ..) => {
                    t.writes.insert((false, *slot));
                }
                Stmt::Assign(place, ..) => {
                    t.writes.insert(walk::root(place));
                }
                Stmt::DefDomain { id, .. } | Stmt::DefDomainFromVal { id, .. } => {
                    t.writes_domain.insert(*id);
                }
                Stmt::Assert { .. } | Stmt::Trap(..) => t.io = true,
                _ => {}
            }
        });
        t
    }

    /// The globals closure `cid` touches when called, when it captures
    /// nothing, takes nothing `mut` and calls no fn value.
    fn closure_access(&self, program: &Program, cid: Option<ClosureId>) -> Option<Access> {
        let c = &program.closures[cid?];
        let mut values = false;
        walk::deep_exprs(&c.body, &mut |e| {
            values |= matches!(e.kind, ExprKind::CallExpr(..))
        });
        if values || !c.captures.is_empty() || !c.muts.is_empty() {
            return None;
        }
        let mut t = self.block_access(program, None, &|_, _| None, &c.body);
        t.reads.retain(|&(global, _)| global);
        t.writes.retain(|&(global, _)| global);
        Some(t)
    }
}
