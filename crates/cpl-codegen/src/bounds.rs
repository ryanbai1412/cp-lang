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

/// What a check before a loop covers.
#[derive(Clone, Debug)]
pub(crate) enum Inside {
    /// `(list, index, lo, hi)`: `list[index + c]` is inside the list for
    /// every `c` in `lo..=hi`.
    List(Var, usize, i64, i64),
    /// `(grid, index, dims)`: `grid[e0, e1, ..]` is inside the grid when
    /// each `e` fits its `Dim`.
    Grid(Var, usize, Vec<Dim>),
}

/// One index of a grid cell the check covers.
#[derive(Clone, Debug)]
pub(crate) enum Dim {
    /// `index + c` for `c` in `lo..=hi`.
    Slot(i64, i64),
    /// This expression, which the loop does not change.
    Fixed(Expr),
    /// Proved by its type.
    Proven,
}

fn is_grid(ty: &Ty) -> bool {
    matches!(ty, Ty::Array { dims, .. } if dims.len() > 1)
}

fn dim_fits(dim: &Dim, slot: usize, e: &Expr) -> bool {
    match dim {
        Dim::Proven => true,
        Dim::Fixed(f) => same(f, e),
        Dim::Slot(lo, hi) => offset(e).is_some_and(|(s, c)| s == slot && *lo <= c && c <= *hi),
    }
}

/// The same expression, spans aside, among those `bound` takes.
fn same(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Local(x), ExprKind::Local(y)) | (ExprKind::Global(x), ExprKind::Global(y)) => {
            x == y
        }
        (ExprKind::Binary(o, l, r), ExprKind::Binary(p, m, q)) => {
            o == p && same(l, m) && same(r, q)
        }
        (ExprKind::Builtin(Builtin::Len, x), ExprKind::Builtin(Builtin::Len, y)) => {
            x.len() == 1 && y.len() == 1 && same(&x[0], &y[0])
        }
        _ => false,
    }
}

fn same_shape(a: &[Dim], b: &[Dim]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|pair| match pair {
            (Dim::Proven, Dim::Proven) | (Dim::Slot(..), Dim::Slot(..)) => true,
            (Dim::Fixed(x), Dim::Fixed(y)) => same(x, y),
            _ => false,
        })
}

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
    /// True when `list[indices]` (or a grid's cell) is inside without a check.
    pub(crate) fn inside_at(&self, list: Option<Var>, indices: &[Expr]) -> bool {
        list.is_some()
            && self.inside.iter().any(|cover| match cover {
                Inside::List(l, i, lo, hi) => {
                    Some(*l) == list
                        && indices.len() == 1
                        && offset(&indices[0]).is_some_and(|(s, c)| s == *i && *lo <= c && c <= *hi)
                }
                Inside::Grid(g, i, dims) => {
                    Some(*g) == list
                        && dims.len() == indices.len()
                        && dims.iter().zip(indices).all(|(d, e)| dim_fits(d, *i, e))
                }
            })
    }

    pub(crate) fn inside_place(&self, list: &PlaceKind, indices: &[Expr]) -> bool {
        self.inside_at(place_var(list), indices)
    }

    /// `.ck` for a grid cell that needs its check, else nothing.
    pub(crate) fn grid_ck(&self, grid: &PlaceKind, indices: &[Expr]) -> &'static str {
        if crate::expr::proven(indices) || self.inside_place(grid, indices) {
            ""
        } else {
            ".ck"
        }
    }

    fn var_expr(&self, var: Var) -> Expr {
        Expr {
            kind: match var {
                (true, id) => ExprKind::Global(id),
                (false, at) => ExprKind::Local(at),
            },
            ty: self.var_ty(var).clone(),
            span: Default::default(),
        }
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
            every_grid: Vec::new(),
            lists: false,
            grids: false,
        };
        // A list and a grid stop with different messages, so a check moved
        // ahead could change which one a run prints.
        if !scan.block(body, true) || (scan.lists && scan.grids) {
            return Vec::new();
        }
        let (stored, every, every_grid) = (scan.stored, scan.every, scan.every_grid);
        if stored.contains(&(false, slot)) || !end(lo, &stored) || !end(hi, &stored) {
            return Vec::new();
        }
        let every: Vec<_> = every
            .into_iter()
            .filter(|(list, _)| !stored.contains(list))
            .collect();
        let mut moving = stored.clone();
        moving.insert((false, slot));
        let mut grids: Vec<(Var, Vec<Dim>)> = Vec::new();
        for (grid, dims) in every_grid {
            if stored.contains(&grid)
                || dims
                    .iter()
                    .any(|d| matches!(d, Dim::Fixed(e) if !bound(e, &moving)))
            {
                continue;
            }
            match grids
                .iter_mut()
                .find(|(g, have)| *g == grid && same_shape(have, &dims))
            {
                Some((_, have)) => {
                    for pair in have.iter_mut().zip(&dims) {
                        if let (Dim::Slot(lo, hi), Dim::Slot(c, _)) = pair {
                            (*lo, *hi) = ((*lo).min(*c), (*hi).max(*c));
                        }
                    }
                }
                None => grids.push((grid, dims)),
            }
        }
        if every.is_empty() && grids.is_empty() {
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
        for (grid, dims) in grids {
            let g = self.expr(&self.var_expr(grid)).text;
            let mut bad = Vec::new();
            for (d, dim) in dims.iter().enumerate() {
                match dim {
                    Dim::Slot(min, max) => bad.push(format!(
                        "(ll)({lo}) + {min} < 0 || {last} + {max} >= {g}.n[{d}]"
                    )),
                    Dim::Fixed(e) => {
                        let at = self.expr(e).text;
                        bad.push(format!(
                            "(unsigned long long)({at}) >= (unsigned long long){g}.n[{d}]"
                        ))
                    }
                    Dim::Proven => {}
                }
            }
            if !bad.is_empty() {
                self.line(
                    out,
                    &format!(
                        "if ({run} && ({})) cpl::fail(\"index outside the array\");",
                        bad.join(" || ")
                    ),
                );
            }
            covered.push(Inside::Grid(grid, slot, dims));
        }
        for (list, (min, max)) in every {
            let of = self.var_expr(list);
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
            covered.push(Inside::List(list, slot, min, max));
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

/// A loop end the check may read again before the loop: `bound`, or an
/// element of a list or array the body does not replace. The loop reads it once
/// before its first round, so a second read fails the same way or not at all.
fn end(e: &Expr, stored: &BTreeSet<Var>) -> bool {
    match &e.kind {
        ExprKind::Index(list, at) => {
            matches!(list.ty, Ty::List(_) | Ty::Array { .. })
                && var_of(&list.kind).is_some_and(|v| !stored.contains(&v))
                && at.iter().all(|i| bound(i, stored))
        }
        ExprKind::Binary(BinOp::Add | BinOp::Sub, a, b) => end(a, stored) && end(b, stored),
        _ => bound(e, stored),
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
    /// Grid cells reached on every round, each index as a `Dim`.
    every_grid: Vec<(Var, Vec<Dim>)>,
    /// Whether the body indexes a list, and a grid.
    lists: bool,
    grids: bool,
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
                (Some(grid), at) if is_grid(&base.ty) => self.cell(grid, at, every),
                _ => false,
            },
            _ => false,
        }
    }

    fn cell(&mut self, grid: Var, at: &[Expr], every: bool) -> bool {
        self.grids = true;
        if every {
            let dims = at.iter().map(|e| {
                if crate::expr::proven(std::slice::from_ref(e)) {
                    Dim::Proven
                } else {
                    match offset(e) {
                        Some((s, c)) if s == self.slot => Dim::Slot(c, c),
                        _ => Dim::Fixed(e.clone()),
                    }
                }
            });
            self.every_grid.push((grid, dims.collect()));
        }
        at.iter().all(|e| self.expr(e, every))
    }

    fn index(&mut self, list: Var, index: &Expr, every: bool) -> bool {
        self.lists = true;
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
                (Some(grid), at) if is_grid(&base.ty) => self.cell(grid, at, every),
                _ => false,
            },
            ExprKind::Builtin(Builtin::Min | Builtin::Max, args)
                if args.len() == 2 && args.iter().all(|x| x.ty == Ty::Int) =>
            {
                args.iter().all(|x| self.expr(x, every))
            }
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

    const GRID: &str = concat!(
        "let n: Int = read()\n",
        "var g: [0..n, 0..n]Int = 0\n",
        "for y in 0..n - 1:\n",
        "    for k in 0..n:\n",
        "        g[y, k] += g[y + 1, k] + min(k, 3)\n",
        "print(g[0, 0])\n",
    );

    fn inner(code: &str) -> &str {
        let body = &code[code.find("for (ll k").unwrap()..];
        &body[..body.find('}').unwrap()]
    }

    #[test]
    fn a_grid_loop_checks_its_cells_once_before() {
        let code = cpp(GRID);
        assert!(
            code.contains("cpl::fail(\"index outside the array\")"),
            "{code}"
        );
        assert!(!inner(&code).contains(".ck("), "{code}");
    }

    #[test]
    fn a_grid_loop_that_prints_keeps_its_checks() {
        let code = cpp(&GRID.replace("+ min(k, 3)", "\n        print(k)"));
        assert!(inner(&code).contains(".ck("), "{code}");
    }

    #[test]
    fn a_loop_over_a_grid_and_a_list_keeps_its_checks() {
        let code = cpp(&GRID
            .replace("var g:", "let w = [i for i in 0..n]\nvar g:")
            .replace("min(k, 3)", "w[k]"));
        assert!(inner(&code).contains(".ck("), "{code}");
    }

    #[test]
    fn a_grid_loop_to_a_list_element_checks_ahead() {
        let code = cpp(concat!(
            "let n: Int = read()\n",
            "let a = [i for i in 0..n]\n",
            "let b: [0..n]0..n = read()\n",
            "var g: [0..n, 0..n]Int = 0\n",
            "for y in 0..n:\n",
            "    for k in 0..a[y] + b[y]:\n",
            "        g[k, y] += 1\n",
            "print(g[0, 0])\n",
        ));
        assert!(!inner(&code).contains(".ck("), "{code}");
    }
}
