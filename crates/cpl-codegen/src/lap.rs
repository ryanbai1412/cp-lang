//! A call of a fn that only computes a value, with an argument that is
//! another such call, is written as the one expression both compute when
//! that takes fewer reductions: `add(x, mul(y, z))` over residues becomes
//! `(x + y * z) % P` where the bounds keep `x + y * z` inside an `Int`.
//!
//! Decided on the bodies put in, from bounds alone. A lap reduction, `if a
//! < 0: a + n else: a` with `a` in `-n..n` (or `if a >= n: a - n else: a`
//! with `a` in `0..2n`), is `a % n`. A sum taken `% n` drops terms that
//! are `n` and the `% n` of its terms. Field bounds come from the struct's
//! declared ranges (`Mod<P>(v: 0..P)`), which every value holds.

use crate::emit::{Code, Emitter};
use cpl_ir::{Arg, BinOp, Builtin, CmpOp, Expr, ExprKind, Stmt, Ty};

/// `[lo, hi]`, wide enough that no step of a sum or product of `Int`s overflows.
type Bounds = (i128, i128);

impl<'p> Emitter<'p> {
    /// The fused form of the call `id(args)`, when it saves a reduction.
    pub(crate) fn try_lap_fusion(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Option<Code> {
        let nested = args.iter().any(|a| {
            matches!(a, Arg::Value(v) if matches!(v.kind, ExprKind::Call(g, _) if self.value_body(g).is_some()))
        });
        if !nested {
            return None;
        }
        let before = self.call_value(id, args)?;
        let after = self.canon(&before);
        (reductions(&after) < reductions(&before) && self.all_fit(&after)).then_some(())?;
        let _ = e;
        Some(self.expr(&after))
    }

    /// The lets and returned value of fn `id`, when that is all its body is.
    fn value_body(&self, id: usize) -> Option<(&'p [Stmt], &'p Expr)> {
        let f = self.program.fns.get(id)?;
        let (last, lets) = f.body.stmts.split_last()?;
        let Stmt::Return(Some(ret)) = last else {
            return None;
        };
        lets.iter()
            .all(|s| matches!(s, Stmt::Let(_, Some(_))))
            .then_some((lets, ret))
    }

    /// The value `id(args)` computes with the arguments put in; a nested call
    /// of such a fn is put in too, at any depth. `None` unless every other
    /// argument only reads.
    fn call_value(&self, id: usize, args: &[Arg]) -> Option<Expr> {
        let (lets, ret) = self.value_body(id)?;
        let params = self.program.fns[id].params.len();
        let mut vals: Vec<Option<Expr>> = vec![None; self.program.fns[id].locals.len()];
        for (slot, arg) in args.iter().enumerate().take(params) {
            let Arg::Value(v) = arg else { return None };
            vals[slot] = Some(match &v.kind {
                ExprKind::Call(g, a) => self.call_value(*g, a)?,
                _ if reads_only(v) => v.clone(),
                _ => return None,
            });
        }
        for stmt in lets {
            let Stmt::Let(slot, Some(init)) = stmt else {
                return None;
            };
            let v = put(init, &vals)?;
            *vals.get_mut(*slot)? = Some(v);
        }
        put(ret, &vals)
    }

    /// `e` with lap reductions as `%` and each `%` over the fewest terms.
    fn canon(&self, e: &Expr) -> Expr {
        let mut kids = e.clone();
        match &mut kids.kind {
            ExprKind::Binary(_, l, r) | ExprKind::Cmp(_, l, r) => {
                **l = self.canon(l);
                **r = self.canon(r);
            }
            ExprKind::If(c, a, b) => {
                **c = self.canon(c);
                **a = self.canon(a);
                **b = self.canon(b);
            }
            ExprKind::Cast(_, x) | ExprKind::Unary(_, x) | ExprKind::Field(x, _) => {
                **x = self.canon(x)
            }
            ExprKind::Tuple(items) => {
                for x in items.iter_mut() {
                    *x = self.canon(x);
                }
            }
            _ => {}
        }
        if let ExprKind::Field(base, i) = &kids.kind {
            if let ExprKind::Tuple(items) = &base.kind {
                return items[*i].clone();
            }
        }
        match self.lap(&kids) {
            Some((a, n)) => self.fold_rem(&a, n, &kids),
            None => match &kids.kind {
                ExprKind::Binary(BinOp::Rem, a, m) if kids.ty == Ty::Int => match m.kind {
                    ExprKind::Int(n) if n > 0 => self.fold_rem(a, n, &kids),
                    _ => kids,
                },
                _ => kids,
            },
        }
    }

    /// `(a, n)` when `e` is a lap reduction of `a` by the constant `n`
    /// that `a`'s bounds make equal to `a % n`.
    fn lap(&self, e: &Expr) -> Option<(Expr, i64)> {
        let ExprKind::If(c, then, otherwise) = &e.kind else {
            return None;
        };
        let ExprKind::Cmp(op, a, k) = &c.kind else {
            return None;
        };
        let same = |x: &Expr| crate::stmt::same_path(x, a) || x == &**a;
        let (n, from) = match (op, &k.kind, &then.kind) {
            (CmpOp::Lt, ExprKind::Int(0), ExprKind::Binary(BinOp::Add, x, m)) if same(x) => {
                (const_of(m)?, -1)
            }
            (CmpOp::Ge, ExprKind::Int(n), ExprKind::Binary(BinOp::Sub, x, m))
                if same(x) && const_of(m) == Some(*n) =>
            {
                (*n, 0)
            }
            _ => return None,
        };
        (n > 0 && same(otherwise)).then_some(())?;
        let (lo, hi) = self.bounds(a)?;
        let n128 = i128::from(n);
        (lo >= from * n128 && hi < (from + 2) * n128).then(|| ((**a).clone(), n))
    }

    /// `a % n` over the terms of `a` that change it: a term `n` or `-n` goes,
    /// and a term `t % n` with `t >= 0` is `t`. `orig` stays when that is not
    /// a sum `>= 0`.
    fn fold_rem(&self, a: &Expr, n: i64, orig: &Expr) -> Expr {
        let mut terms = Vec::new();
        flatten(a, true, &mut terms);
        let kept: Vec<(bool, Expr)> = terms
            .into_iter()
            .filter(|(_, t)| const_of(t) != Some(n))
            .map(|(plus, t)| match &t.kind {
                ExprKind::Binary(BinOp::Rem, x, m)
                    if const_of(m) == Some(n) && self.bounds(x).is_some_and(|(lo, _)| lo >= 0) =>
                {
                    (plus, (**x).clone())
                }
                _ => (plus, t),
            })
            .collect();
        let Some(sum) = rebuild(kept, a) else {
            return orig.clone();
        };
        if !self
            .bounds(&sum)
            .is_some_and(|(lo, hi)| lo >= 0 && hi <= i128::from(i64::MAX))
        {
            return orig.clone();
        }
        let n = Expr {
            kind: ExprKind::Int(n),
            ty: Ty::Int,
            span: a.span,
        };
        Expr {
            kind: ExprKind::Binary(BinOp::Rem, Box::new(sum), Box::new(n)),
            ty: Ty::Int,
            span: orig.span,
        }
    }

    /// Whether every `+`, `-` and `*` in `e` stays inside an `Int`.
    fn all_fit(&self, e: &Expr) -> bool {
        let int = i128::from(i64::MIN) + 1..=i128::from(i64::MAX);
        let ok = match &e.kind {
            ExprKind::Binary(BinOp::Add | BinOp::Sub | BinOp::Mul, ..) => self
                .bounds(e)
                .is_some_and(|(lo, hi)| int.contains(&lo) && int.contains(&hi)),
            _ => true,
        };
        ok && match &e.kind {
            ExprKind::Binary(_, l, r) | ExprKind::Cmp(_, l, r) => {
                self.all_fit(l) && self.all_fit(r)
            }
            ExprKind::If(c, a, b) => self.all_fit(c) && self.all_fit(a) && self.all_fit(b),
            ExprKind::Cast(_, x) | ExprKind::Unary(_, x) => self.all_fit(x),
            ExprKind::Tuple(items) => items.iter().all(|x| self.all_fit(x)),
            _ => true,
        }
    }

    /// Constant bounds of the `Int` `e`: sums and products in 128 bits over
    /// what the range facts prove of each leaf.
    fn bounds(&self, e: &Expr) -> Option<Bounds> {
        let of = |x: &Expr| self.bounds(x);
        match &e.kind {
            ExprKind::Int(n) => Some((i128::from(*n), i128::from(*n))),
            ExprKind::Binary(BinOp::Add, l, r) => {
                let ((a, b), (c, d)) = (of(l)?, of(r)?);
                Some((a + c, b + d))
            }
            ExprKind::Binary(BinOp::Sub, l, r) => {
                let ((a, b), (c, d)) = (of(l)?, of(r)?);
                Some((a - d, b - c))
            }
            ExprKind::Binary(BinOp::Mul, l, r) => {
                let ((a, b), (c, d)) = (of(l)?, of(r)?);
                let xs = [a * c, a * d, b * c, b * d];
                Some((*xs.iter().min()?, *xs.iter().max()?))
            }
            ExprKind::Binary(BinOp::Rem, _, m) => match m.kind {
                ExprKind::Int(n) if n > 0 => Some((0, i128::from(n) - 1)),
                _ => None,
            },
            ExprKind::If(_, a, b) => {
                let ((p, q), (r, s)) = (of(a)?, of(b)?);
                Some((p.min(r), q.max(s)))
            }
            _ => {
                let home = self.facts_home();
                Some((
                    i128::from(self.facts.lower_bound(home, e, 0)?),
                    i128::from(self.facts.upper_bound(home, e, 0)?),
                ))
            }
        }
    }
}

fn const_of(e: &Expr) -> Option<i64> {
    match e.kind {
        ExprKind::Int(n) => Some(n),
        _ => None,
    }
}

/// How many `%`s and lap reductions `e` runs.
fn reductions(e: &Expr) -> usize {
    let own = usize::from(matches!(
        e.kind,
        ExprKind::Binary(BinOp::Rem, ..) | ExprKind::If(..)
    ));
    own + match &e.kind {
        ExprKind::Binary(_, l, r) | ExprKind::Cmp(_, l, r) => reductions(l) + reductions(r),
        ExprKind::If(c, a, b) => reductions(c) + reductions(a) + reductions(b),
        ExprKind::Cast(_, x) | ExprKind::Unary(_, x) | ExprKind::Field(x, _) => reductions(x),
        ExprKind::Tuple(items) => items.iter().map(reductions).sum(),
        _ => 0,
    }
}

/// Whether evaluating `e` only reads: no call, no store, nothing printed.
fn reads_only(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(x, _) | ExprKind::Cast(_, x) | ExprKind::Unary(_, x) => reads_only(x),
        ExprKind::Binary(_, l, r) | ExprKind::Cmp(_, l, r) => reads_only(l) && reads_only(r),
        ExprKind::Index(base, idx) => reads_only(base) && idx.iter().all(reads_only),
        ExprKind::Builtin(Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul, a) => {
            a.iter().all(reads_only)
        }
        _ => false,
    }
}

/// `e` with each of the callee's locals replaced by its value in `vals`.
fn put(e: &Expr, vals: &[Option<Expr>]) -> Option<Expr> {
    let one = |x: &Expr| put(x, vals).map(Box::new);
    let kind = match &e.kind {
        ExprKind::Local(slot) => return vals.get(*slot)?.clone(),
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Global(_) => e.kind.clone(),
        ExprKind::Binary(op, l, r) => ExprKind::Binary(*op, one(l)?, one(r)?),
        ExprKind::Cmp(op, l, r) => ExprKind::Cmp(*op, one(l)?, one(r)?),
        ExprKind::If(c, a, b) => ExprKind::If(one(c)?, one(a)?, one(b)?),
        ExprKind::Cast(c, x) => ExprKind::Cast(*c, one(x)?),
        ExprKind::Unary(op, x) => ExprKind::Unary(*op, one(x)?),
        ExprKind::Field(x, i) => ExprKind::Field(one(x)?, *i),
        ExprKind::Tuple(items) => {
            ExprKind::Tuple(items.iter().map(|x| put(x, vals)).collect::<Option<_>>()?)
        }
        _ => return None,
    };
    Some(Expr {
        kind,
        ty: e.ty.clone(),
        span: e.span,
    })
}

/// The terms of a sum, each with its sign.
fn flatten(e: &Expr, plus: bool, out: &mut Vec<(bool, Expr)>) {
    match &e.kind {
        ExprKind::Binary(BinOp::Add, l, r) => {
            flatten(l, plus, out);
            flatten(r, plus, out);
        }
        ExprKind::Binary(BinOp::Sub, l, r) => {
            flatten(l, plus, out);
            flatten(r, !plus, out);
        }
        _ => out.push((plus, e.clone())),
    }
}

/// The sum of `terms`, left to right; `None` when it starts with a minus.
fn rebuild(terms: Vec<(bool, Expr)>, like: &Expr) -> Option<Expr> {
    let mut it = terms.into_iter();
    let (true, mut sum) = it.next()? else {
        return None;
    };
    for (plus, t) in it {
        let op = if plus { BinOp::Add } else { BinOp::Sub };
        sum = Expr {
            kind: ExprKind::Binary(op, Box::new(sum), Box::new(t)),
            ty: Ty::Int,
            span: like.span,
        };
    }
    Some(sum)
}
