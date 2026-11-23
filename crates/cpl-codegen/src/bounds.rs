//! Index checks taken out of a loop: `for i in 0..n: k[i + 1] -= k[i]`
//! checks once, before the loop, that `0 + 0 >= 0` and `n - 1 + 1 <
//! k.len`, and then indexes `k` without a check.
//!
//! The one check stands for the checks every round would make, so it is
//! taken only where nothing can tell them apart: the body cannot print,
//! leave the loop early, or stop the program with a different message (a
//! call counts only if its fn is one `return` of such an expression); it
//! keeps the lists' lengths and the loop's bounds; and each index the
//! check covers is reached on every round. A list index that fails stops
//! the program with the same message before the loop as during it.

use crate::emit::Emitter;
use crate::walk;
use cpl_ir::{
    Arg, BinOp, Block, Builtin, Cast, Expr, ExprKind, Iter, Place, PlaceKind, Stmt, Ty, UnOp,
};
use std::collections::{BTreeMap, BTreeSet};

/// A variable: `(global, slot or id)`.
pub(crate) type Var = (bool, usize);

/// `(list, index, lo, hi)`: `list[index + c]` is inside the list for
/// every `c` in `lo..=hi`.
pub(crate) type Inside = (Var, usize, i64, i64);

pub(crate) fn var_of(e: &ExprKind) -> Option<Var> {
    match e {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(id) => Some((true, *id)),
        _ => None,
    }
}

fn place_var(p: &PlaceKind) -> Option<Var> {
    match p {
        PlaceKind::Local(slot) => Some((false, *slot)),
        PlaceKind::Global(id) => Some((true, *id)),
        _ => None,
    }
}

impl<'p> Emitter<'p> {
    /// True when `list[index]` is inside the list without a check.
    pub(crate) fn inside_at(&self, list: Option<Var>, index: &Expr) -> bool {
        offset(index).is_some_and(|(slot, c)| {
            self.inside
                .iter()
                .any(|&(l, i, lo, hi)| Some(l) == list && i == slot && lo <= c && c <= hi)
        })
    }

    pub(crate) fn inside_place(&self, list: &PlaceKind, index: &Expr) -> bool {
        self.inside_at(place_var(list), index)
    }

    fn var_ty(&self, (global, at): Var) -> &'p Ty {
        if global {
            &self.program.globals[at].ty
        } else {
            &self.locals()[at].ty
        }
    }

    /// Writes the check, before `for slot in iter: body`, of the indexes
    /// into lists that every round reaches, and gives what it covers.
    pub(crate) fn check_ahead(
        &mut self,
        slot: usize,
        iter: &Iter,
        body: &Block,
        out: &mut String,
    ) -> Vec<Inside> {
        let Iter::Range {
            lo,
            hi,
            step: None,
            closed,
            ..
        } = iter
        else {
            return Vec::new();
        };
        if self.closure_id.is_some() || self.locals()[slot].ty != Ty::Int {
            return Vec::new();
        }
        let mut scan = Scan {
            emitter: self,
            slot,
            stored: BTreeSet::new(),
            every: BTreeMap::new(),
        };
        if !scan.block(body, true) {
            return Vec::new();
        }
        let (stored, every) = (scan.stored, scan.every);
        let fixed = |e: &Expr| bound(e, &stored);
        if stored.contains(&(false, slot)) || !fixed(lo) || !fixed(hi) {
            return Vec::new();
        }
        let every: Vec<_> = every
            .into_iter()
            .filter(|(list, _)| !stored.contains(list))
            .collect();
        if every.is_empty() {
            return Vec::new();
        }
        let (lo, hi) = (self.expr(lo).text, self.expr(hi).text);
        let (run, last) = if *closed {
            (format!("(ll)({lo}) <= (ll)({hi})"), format!("(ll)({hi})"))
        } else {
            (
                format!("(ll)({lo}) < (ll)({hi})"),
                format!("(ll)({hi}) - 1"),
            )
        };
        self.need("fail");
        let mut covered = Vec::new();
        for (list, (min, max)) in every {
            let of = Expr {
                kind: match list {
                    (true, id) => ExprKind::Global(id),
                    (false, at) => ExprKind::Local(at),
                },
                ty: self.var_ty(list).clone(),
                span: Default::default(),
            };
            let len = self
                .expr(&Expr {
                    kind: ExprKind::Builtin(Builtin::Len, vec![of]),
                    ty: Ty::Int,
                    span: Default::default(),
                })
                .text;
            self.line(
                out,
                &format!(
                    "if ({run} && ((ll)({lo}) + {min} < 0 || {last} + {max} >= (ll)({len}))) \
                     cpl::fail(\"index outside the list\");"
                ),
            );
            covered.push((list, slot, min, max));
        }
        covered
    }
}

/// `i`, `i + c` or `i - c`: the slot and `c`.
fn offset(e: &Expr) -> Option<(usize, i64)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((*slot, 0)),
        ExprKind::Binary(BinOp::Add, a, b) => match (&a.kind, &b.kind) {
            (ExprKind::Local(slot), ExprKind::Int(c))
            | (ExprKind::Int(c), ExprKind::Local(slot)) => Some((*slot, *c)),
            _ => None,
        },
        ExprKind::Binary(BinOp::Sub, a, b) => match (&a.kind, &b.kind) {
            (ExprKind::Local(slot), ExprKind::Int(c)) => Some((*slot, c.checked_neg()?)),
            _ => None,
        },
        _ => None,
    }
}

/// A loop bound the body cannot change: literals, variables it does not
/// store, list lengths and sums of them.
fn bound(e: &Expr, stored: &BTreeSet<Var>) -> bool {
    match &e.kind {
        ExprKind::Int(_) => true,
        ExprKind::Local(_) | ExprKind::Global(_) => {
            e.ty == Ty::Int && var_of(&e.kind).is_some_and(|v| !stored.contains(&v))
        }
        ExprKind::Builtin(Builtin::Len, of) => match of.as_slice() {
            [list] => {
                matches!(list.ty, Ty::List(_))
                    && var_of(&list.kind).is_some_and(|v| !stored.contains(&v))
            }
            _ => false,
        },
        ExprKind::Binary(BinOp::Add | BinOp::Sub, a, b) => bound(a, stored) && bound(b, stored),
        _ => false,
    }
}

struct Scan<'e, 'p> {
    emitter: &'e Emitter<'p>,
    slot: usize,
    /// Variables the body stores whole.
    stored: BTreeSet<Var>,
    /// Per list, the least and greatest `c` of `list[slot + c]` reached on
    /// every round.
    every: BTreeMap<Var, (i64, i64)>,
}

impl Scan<'_, '_> {
    /// False when the body can do anything but compute and store: print,
    /// leave the loop, resize a list, or stop with another message.
    /// `every` is true while each round surely reaches the code.
    fn block(&mut self, block: &Block, every: bool) -> bool {
        block.stmts.iter().all(|stmt| match stmt {
            Stmt::Let(at, init) => {
                self.stored.insert((false, *at));
                init.as_ref().is_none_or(|e| self.expr(e, every))
            }
            Stmt::Assign(place, _, value) => self.place(place, every) && self.expr(value, every),
            Stmt::Expr(e) => self.expr(e, every),
            Stmt::If(cond, then, otherwise) => {
                self.expr(cond, every) && self.block(then, false) && self.block(otherwise, false)
            }
            Stmt::For(
                _,
                at,
                Iter::Range {
                    lo, hi, step: None, ..
                },
                body,
            ) => {
                self.stored.insert((false, *at));
                self.expr(lo, every) && self.expr(hi, every) && self.block(body, false)
            }
            _ => false,
        })
    }

    fn place(&mut self, place: &Place, every: bool) -> bool {
        if let Some(var) = place_var(&place.kind) {
            self.stored.insert(var);
            return place.ty.is_scalar();
        }
        match &place.kind {
            PlaceKind::Index(base, at) => match (place_var(&base.kind), at.as_slice()) {
                (Some(list), [index]) if matches!(base.ty, Ty::List(_)) => {
                    self.index(list, index, every)
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn index(&mut self, list: Var, index: &Expr, every: bool) -> bool {
        if every {
            if let Some((_, c)) = offset(index).filter(|&(at, _)| at == self.slot) {
                let range = self.every.entry(list).or_insert((c, c));
                *range = (range.0.min(c), range.1.max(c));
            }
        }
        self.expr(index, every)
    }

    /// The expression cannot trap but by indexing a list variable.
    fn expr(&mut self, e: &Expr, every: bool) -> bool {
        match &e.kind {
            ExprKind::Some(x)
            | ExprKind::Field(x, _)
            | ExprKind::IsSome(x)
            | ExprKind::IsVariant(x, _)
            | ExprKind::Cast(Cast::IntToFloat, x) => self.expr(x, every),
            ExprKind::Unary(op, x) => *op != UnOp::Neg && self.expr(x, every),
            ExprKind::Binary(op, l, r) => {
                let short = matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce);
                safe_op(*op, l, r) && self.expr(l, every) && self.expr(r, every && !short)
            }
            ExprKind::Cmp(_, l, r) => self.expr(l, every) && self.expr(r, every),
            ExprKind::Tuple(items) | ExprKind::Variant(_, items) => {
                items.iter().all(|x| self.expr(x, every))
            }
            ExprKind::If(c, t, f) => {
                self.expr(c, every) && self.expr(t, false) && self.expr(f, false)
            }
            ExprKind::Index(base, at) => match (var_of(&base.kind), at.as_slice()) {
                (Some(list), [index]) if matches!(base.ty, Ty::List(_)) => {
                    self.index(list, index, every)
                }
                _ => false,
            },
            ExprKind::Call(f, args) => {
                args.iter()
                    .all(|a| matches!(a, Arg::Value(x) if self.expr(x, every)))
                    && calm_fn(self.emitter, *f, 2)
            }
            _ => leaf(&e.kind),
        }
    }
}

fn leaf(e: &ExprKind) -> bool {
    matches!(
        e,
        ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::None
            | ExprKind::Local(_)
            | ExprKind::Global(_)
    )
}

fn safe_op(op: BinOp, l: &Expr, r: &Expr) -> bool {
    l.ty != Ty::Str
        && match op {
            BinOp::Div | BinOp::Rem => matches!(r.kind, ExprKind::Int(k) if k != 0),
            BinOp::Pow | BinOp::Shl | BinOp::Shr => false,
            _ => true,
        }
}

/// A fn whose body is one `return` that cannot trap, print or store.
fn calm_fn(emitter: &Emitter, f: usize, depth: usize) -> bool {
    let Some(func) = emitter.program.fns.get(f) else {
        return false;
    };
    match func.body.stmts.as_slice() {
        [Stmt::Return(Some(e))] => depth > 0 && calm(emitter, e, depth - 1),
        _ => false,
    }
}

fn calm(emitter: &Emitter, e: &Expr, depth: usize) -> bool {
    let here = match &e.kind {
        ExprKind::Some(_)
        | ExprKind::Field(..)
        | ExprKind::IsSome(_)
        | ExprKind::IsVariant(..)
        | ExprKind::Cmp(..)
        | ExprKind::Tuple(_)
        | ExprKind::Variant(..)
        | ExprKind::If(..)
        | ExprKind::Cast(Cast::IntToFloat, _) => true,
        ExprKind::Unary(op, _) => *op != UnOp::Neg,
        ExprKind::Binary(op, l, r) => safe_op(*op, l, r),
        ExprKind::Call(f, args) => {
            args.iter().all(|a| matches!(a, Arg::Value(_))) && calm_fn(emitter, *f, depth)
        }
        kind => leaf(kind),
    };
    let mut ok = here;
    if ok {
        walk::children(e, &mut |x| ok = ok && calm(emitter, x, depth));
    }
    ok
}

#[cfg(test)]
mod tests {
    use crate::tests::cpp;

    const SHIFT: &str = concat!(
        "let n: Int = read()\n",
        "var k = [i for i in 0..=n]\n",
        "for i in 0..n:\n    k[i + 1] -= k[i]\n",
        "print(k[n])\n",
    );

    #[test]
    fn a_loop_that_only_computes_checks_its_indexes_once_before() {
        let code = cpp(SHIFT);
        assert!(
            code.contains("cpl::fail(\"index outside the list\")"),
            "{code}"
        );
        let body = code.split("for (").nth(1).unwrap();
        let body = &body[..body.find('}').unwrap()];
        assert!(!body.contains("cpl::at("), "{body}");
    }

    #[test]
    fn a_loop_that_prints_keeps_its_checks() {
        let code = cpp(&SHIFT.replace("k[i + 1] -= k[i]", "print(k[i + 1])"));
        assert!(
            !code.contains("cpl::fail(\"index outside the list\")"),
            "{code}"
        );
    }

    #[test]
    fn an_index_reached_only_on_some_rounds_keeps_its_check() {
        let code = cpp(&SHIFT.replace("k[i + 1] -= k[i]", "if i > 2: k[i + 1] -= 1"));
        assert!(
            !code.contains("cpl::fail(\"index outside the list\")"),
            "{code}"
        );
    }

    #[test]
    fn a_loop_that_grows_the_list_keeps_its_checks() {
        let code = cpp(&SHIFT.replace("k[i + 1] -= k[i]", "k[i] += 1\n    k.push(1)"));
        assert!(
            !code.contains("cpl::fail(\"index outside the list\")"),
            "{code}"
        );
    }

    #[test]
    fn an_index_past_the_end_fails_ahead_of_the_loop() {
        let code = cpp(&SHIFT.replace("k[i + 1] -= k[i]", "k[i] += k[i + 2]"));
        assert!(code.contains("(ll)(n) - 1 + 2 >= (ll)("), "{code}");
    }

    #[test]
    fn a_check_ahead_brings_the_fail_it_calls() {
        let code = cpp(concat!(
            "fn pairs(xs: List<Int>) -> Int:\n",
            "    var same = 0\n",
            "    for i in 0..xs.len:\n",
            "        for j in 0..i:\n",
            "            if xs[i] == xs[j]: same += 1\n",
            "    same\n",
            "print(pairs([1, 2, 1, 1]))\n",
        ));
        assert!(code.contains("[[noreturn]] inline void fail("), "{code}");
    }
}
