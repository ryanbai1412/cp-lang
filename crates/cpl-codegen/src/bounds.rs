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

use crate::emit::{pad, Emitter};
use crate::walk;
use cpl_ir::{
    Arg, BinOp, Block, Builtin, Cast, CmpOp, Expr, ExprKind, Iter, LoopId, Place, PlaceKind, Stmt,
    Ty, UnOp,
};
use std::collections::{BTreeMap, BTreeSet};

/// The constant offsets, low and high, each list is indexed at.
type Offsets = BTreeMap<List, (i64, i64)>;

/// A variable, or one field of a struct variable: `(global, slot or id,
/// field)`.
pub(crate) type Var = (bool, usize, Option<usize>);

/// A loop-invariant `Int` an index adds (`false`) or takes away (`true`).
pub(crate) type Term = (bool, Var);

/// What a check before a loop covers.
#[derive(Clone, Debug)]
pub(crate) enum Inside {
    /// `(list, index, term, lo, hi)`: `list[index ± term + c]` is inside
    /// the list for every `c` in `lo..=hi`.
    List(Var, usize, Option<Term>, i64, i64),
    /// `(grid, index, dims)`: `grid[e0, e1, ..]` is inside the grid when
    /// each `e` fits its `Dim`.
    Grid(Var, usize, Vec<Dim>),
    /// A shift by this C++ amount is inside `0..64`.
    Shift(String),
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

/// The most C++ a loop's copy without index checks may take; a larger
/// body keeps one copy, with its checks.
const VERSIONED_MAX: usize = 2500;

fn is_grid(ty: &Ty) -> bool {
    matches!(ty, Ty::Array { dims, .. } if dims.len() > 1)
}

fn dim_fits(dim: &Dim, slot: usize, e: &Expr) -> bool {
    match dim {
        Dim::Proven => true,
        Dim::Fixed(f) => same(f, e),
        Dim::Slot(lo, hi) => {
            offset(e, slot).is_some_and(|(t, c)| t.is_none() && *lo <= c && c <= *hi)
        }
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
        ExprKind::Local(slot) => Some((false, *slot, None)),
        ExprKind::Global(id) => Some((true, *id, None)),
        ExprKind::Field(x, f) if matches!(x.ty, Ty::Struct(_)) => match var_of(&x.kind)? {
            (global, at, None) => Some((global, at, Some(*f))),
            _ => None,
        },
        _ => None,
    }
}

fn place_var(p: &PlaceKind) -> Option<Var> {
    match p {
        PlaceKind::Local(slot) => Some((false, *slot, None)),
        PlaceKind::Global(id) => Some((true, *id, None)),
        PlaceKind::Field(x, f) if matches!(x.ty, Ty::Struct(_)) => match place_var(&x.kind)? {
            (global, at, None) => Some((global, at, Some(*f))),
            _ => None,
        },
        _ => None,
    }
}

/// Whether the body may change `v`: it stores the variable, or a field of
/// that name in any variable (two references may be one struct).
fn moved(stored: &BTreeSet<Var>, v: &Var) -> bool {
    stored.contains(&(v.0, v.1, None)) || v.2.is_some() && stored.iter().any(|s| s.2 == v.2)
}

impl<'p> Emitter<'p> {
    /// True when `list[indices]` (or a grid's cell) is inside without a check.
    pub(crate) fn inside_at(&self, list: Option<Var>, indices: &[Expr]) -> bool {
        list.is_some()
            && self.inside.iter().any(|cover| match cover {
                Inside::List(l, i, t, lo, hi) => {
                    Some(*l) == list
                        && indices.len() == 1
                        && offset(&indices[0], *i)
                            .is_some_and(|(u, c)| u == *t && *lo <= c && c <= *hi)
                }
                Inside::Grid(g, i, dims) => {
                    Some(*g) == list
                        && dims.len() == indices.len()
                        && dims.iter().zip(indices).all(|(d, e)| dim_fits(d, *i, e))
                }
                Inside::Shift(_) => false,
            })
    }

    /// True when a shift by `by` needs no test that it is inside `0..width`.
    pub(crate) fn shift_fits(&self, by: &str, width: i64) -> bool {
        by.parse::<i64>().is_ok_and(|n| (0..width).contains(&n))
            || width == 64
                && self
                    .inside
                    .iter()
                    .any(|c| matches!(c, Inside::Shift(s) if s == by))
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

    fn var_expr(&self, (global, at, field): Var) -> Expr {
        let root = Expr {
            kind: if global {
                ExprKind::Global(at)
            } else {
                ExprKind::Local(at)
            },
            ty: if global {
                self.program.globals[at].ty.clone()
            } else {
                self.locals()[at].ty.clone()
            },
            span: Default::default(),
        };
        match (field, &root.ty) {
            (Some(f), Ty::Struct(id)) => Expr {
                ty: self.program.structs[*id].fields[f].1.clone(),
                kind: ExprKind::Field(Box::new(root), f),
                span: Default::default(),
            },
            _ => root,
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
    ) -> Ahead {
        let none = Ahead {
            inside: Vec::new(),
            version: None,
        };
        let Iter::Range {
            lo,
            hi,
            step: None,
            closed,
            ..
        } = iter
        else {
            return none;
        };
        if self.closure_id.is_some() || self.locals()[slot].ty != Ty::Int {
            return none;
        }
        let mut scan = Scan {
            emitter: self,
            slot,
            stored: BTreeSet::new(),
            every: BTreeMap::new(),
            every_grid: Vec::new(),
            any: BTreeMap::new(),
            any_grid: Vec::new(),
            lists: false,
            grids: false,
            sometimes: false,
            shifts: Vec::new(),
            floors: Vec::new(),
            floored: Vec::new(),
        };
        if !scan.block(body, true) {
            return none;
        }
        let Scan {
            stored,
            every,
            every_grid,
            any,
            any_grid,
            lists,
            grids,
            sometimes,
            shifts,
            floored,
            ..
        } = scan;
        let mut moving = stored.clone();
        moving.insert((false, slot, None));
        if stored.contains(&(false, slot, None))
            || !end(lo, &stored)
            || !end(hi, &stored)
            || !shifts.iter().all(|e| bound(e, &moving))
        {
            return none;
        }
        // An index under `if i > e` is checked from `e + 1` when `e` stays put.
        let mut any = any;
        let floored: Vec<(Expr, Offsets)> = floored
            .into_iter()
            .filter_map(|(floor, key, c)| {
                if bound(&floor, &moving) {
                    return Some((floor, BTreeMap::from([(key, (c, c))])));
                }
                let range = any.entry(key).or_insert((c, c));
                *range = (range.0.min(c), range.1.max(c));
                None
            })
            .collect();
        let (lo, hi) = (self.expr(lo).text, self.expr(hi).text);
        let (run, last) = if *closed {
            (format!("(ll)({lo}) <= (ll)({hi})"), format!("(ll)({hi})"))
        } else {
            (
                format!("(ll)({lo}) < (ll)({hi})"),
                format!("(ll)({hi}) - 1"),
            )
        };
        let bounds = Bounds {
            slot,
            lo: &lo,
            last: &last,
        };
        // A list and a grid stop with different messages, so a check moved
        // ahead could change which one a run prints.
        let mixed = lists && grids;
        // A shift by too much stops with its own message, so with shifts
        // every check goes into the test that picks the unchecked copy.
        let shifty = !shifts.is_empty();
        let mut inside = Vec::new();
        if !mixed && !shifty {
            let lists = fixed_lists(every, &stored);
            let grids = merge(every_grid, &stored, slot);
            if !lists.is_empty() || !grids.is_empty() {
                self.need("fail");
            }
            for (bad, cover) in self.covers(&bounds, lists, grids) {
                let what = if matches!(cover, Inside::List(..)) {
                    "list"
                } else {
                    "array"
                };
                if let Some(bad) = bad.filter(|_| self.target.checks != crate::Checks::None) {
                    self.line(
                        out,
                        &format!("if ({run} && ({bad})) cpl::fail(\"index outside the {what}\");"),
                    );
                }
                inside.push(cover);
            }
        }
        // Indexes only some rounds reach: one test before the loop picks a
        // copy of it that skips their checks, or the copy that keeps them.
        let version = (sometimes || shifty || (mixed && !any_grid.is_empty())).then(|| {
            let lists = fixed_lists(any, &stored);
            let grids = merge(any_grid, &stored, slot);
            let covers = self.covers(&bounds, lists, grids);
            let mut covers = covers;
            for (floor, list) in floored {
                let floor = self.expr(&floor).text;
                let lo = format!("std::max((ll)({lo}), (ll)({floor}))");
                let at = Bounds { lo: &lo, ..bounds };
                covers.extend(self.covers(&at, fixed_lists(list, &stored), Vec::new()));
            }
            for by in &shifts {
                let by = self.expr(by).text;
                covers.push((
                    Some(format!("(unsigned long long)({by}) >= 64")),
                    Inside::Shift(by),
                ));
            }
            let bad: Vec<_> = covers.iter().filter_map(|(b, _)| b.clone()).collect();
            let guard = if bad.is_empty() {
                "true".to_string()
            } else {
                format!("!({run} && ({}))", bad.join(" || "))
            };
            (
                guard,
                covers.into_iter().map(|(_, c)| c).collect::<Vec<_>>(),
            )
        });
        Ahead {
            inside,
            version: version.filter(|(_, c)| !c.is_empty()),
        }
    }

    /// `for slot in iter: body` with its index checks taken ahead where
    /// they can be.
    pub(crate) fn checked_for(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &Iter,
        body: &Block,
        out: &mut String,
    ) {
        let ahead = self.check_ahead(slot, iter, body, out);
        let mut slow = ahead.inside;
        slow.extend(
            walk::inside_list(slot, iter, body, self.locals())
                .map(|(l, i)| Inside::List((false, l, None), i, None, 0, 0)),
        );
        let fast = ahead.version.and_then(|(guard, mut fast)| {
            fast.extend(slow.iter().cloned());
            let mut code = String::new();
            self.indent += 1;
            self.covered_for(id, slot, iter, body, fast, &mut code);
            self.indent -= 1;
            (code.len() <= VERSIONED_MAX).then_some((guard, code))
        });
        match fast {
            Some((guard, code)) => {
                self.line(out, &format!("if ({guard}) {{"));
                out.push_str(&code);
                self.line(out, "} else {");
                self.indent += 1;
                self.covered_for(id, slot, iter, body, slow, out);
                self.indent -= 1;
                self.line(out, "}");
            }
            None => self.covered_for(id, slot, iter, body, slow, out),
        }
    }

    /// `for slot in iter: body` with the indexes `inside` covers unchecked.
    fn covered_for(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &Iter,
        body: &Block,
        inside: Vec<Inside>,
        out: &mut String,
    ) {
        let pushed = inside.len();
        self.inside.extend(inside);
        let rows = self.loop_rows(slot, body);
        let (start, mark) = (out.len(), self.row_ptrs.len());
        self.row_ptrs.extend(rows);
        self.for_stmt(id, slot, iter, body, out);
        let rows = self.row_ptrs.split_off(mark);
        if !rows.is_empty() {
            self.need("lazy");
            let lead = pad(self.indent);
            let mut head = format!("{lead}{{\n");
            for (_, name, code) in &rows {
                head.push_str(&format!(
                    "{lead}auto {name} = cpl::lazy([&] {{ return &{code}; }});\n"
                ));
            }
            out.insert_str(start, &head);
            self.line(out, "}");
        }
        self.inside.truncate(self.inside.len() - pushed);
    }

    /// Each cover with the C++ test that it fails, if it can.
    fn covers(
        &mut self,
        b: &Bounds,
        lists: Vec<(List, (i64, i64))>,
        grids: Vec<(Var, Vec<Dim>)>,
    ) -> Vec<(Option<String>, Inside)> {
        let (lo, last) = (b.lo, b.last);
        let mut covers = Vec::new();
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
            let bad = (!bad.is_empty()).then(|| bad.join(" || "));
            covers.push((bad, Inside::Grid(grid, b.slot, dims)));
        }
        for ((list, term), (min, max)) in lists {
            let of = self.var_expr(list);
            let len = self
                .expr(&Expr {
                    kind: ExprKind::Builtin(Builtin::Len, vec![of]),
                    ty: Ty::Int,
                    span: Default::default(),
                })
                .text;
            let plus = match term {
                Some((neg, t)) => {
                    let sign = if neg { '-' } else { '+' };
                    format!(" {sign} (ll)({})", self.expr(&self.var_expr(t)).text)
                }
                None => String::new(),
            };
            let bad =
                format!("(ll)({lo}){plus} + {min} < 0 || {last}{plus} + {max} >= (ll)({len})");
            covers.push((Some(bad), Inside::List(list, b.slot, term, min, max)));
        }
        covers
    }
}

/// What `check_ahead` found: the covers its checks stand for, and, when
/// some indexes are reached on only some rounds, the test that picks a
/// copy of the loop with them covered too.
pub(crate) struct Ahead {
    pub inside: Vec<Inside>,
    pub version: Option<(String, Vec<Inside>)>,
}

#[derive(Clone, Copy)]
struct Bounds<'a> {
    slot: usize,
    lo: &'a str,
    last: &'a str,
}

/// A list, with the loop-invariant term its indexes add to the loop's.
type List = (Var, Option<Term>);

fn fixed_lists(
    lists: BTreeMap<List, (i64, i64)>,
    stored: &BTreeSet<Var>,
) -> Vec<(List, (i64, i64))> {
    lists
        .into_iter()
        .filter(|((list, term), _)| {
            !moved(stored, list) && term.is_none_or(|(_, t)| !moved(stored, &t))
        })
        .collect()
}

/// The cells of grids the body does not replace, one cover per shape.
fn merge(cells: Vec<(Var, Vec<Dim>)>, stored: &BTreeSet<Var>, slot: usize) -> Vec<(Var, Vec<Dim>)> {
    let mut moving = stored.clone();
    moving.insert((false, slot, None));
    let mut grids: Vec<(Var, Vec<Dim>)> = Vec::new();
    for (grid, dims) in cells {
        if moved(stored, &grid)
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
    grids
}

/// `slot + c`, or `slot ± v + c` for one other `Int` variable `v`: the
/// term and `c`.
fn offset(e: &Expr, slot: usize) -> Option<(Option<Term>, i64)> {
    let mut terms = Vec::new();
    let c = linear(e, false, &mut terms)?;
    let at = terms
        .iter()
        .position(|&(neg, v)| !neg && v == (false, slot, None))?;
    terms.remove(at);
    match terms.as_slice() {
        [] => Some((None, c)),
        [t] if t.1 != (false, slot, None) => Some((Some(*t), c)),
        _ => None,
    }
}

/// `e` as a sum of `Int` variables and a constant, which it gives.
fn linear(e: &Expr, neg: bool, terms: &mut Vec<Term>) -> Option<i64> {
    match &e.kind {
        ExprKind::Int(c) if neg => c.checked_neg(),
        ExprKind::Int(c) => Some(*c),
        ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub), a, b) if e.ty == Ty::Int => {
            let l = linear(a, neg, terms)?;
            l.checked_add(linear(b, neg != (*op == BinOp::Sub), terms)?)
        }
        _ if e.ty == Ty::Int && terms.len() < 2 => {
            terms.push((neg, var_of(&e.kind)?));
            Some(0)
        }
        _ => None,
    }
}

/// A loop bound the body cannot change: literals, variables it does not
/// store, list lengths and sums of them.
/// The `e` of each `index >= e` the condition's `and` chain states.
fn floors_of(cond: &Expr, slot: usize) -> Vec<Expr> {
    let is_slot = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if s == slot);
    let plus_one = |e: &Expr| Expr {
        kind: ExprKind::Binary(BinOp::Add, Box::new(e.clone()), Box::new(int(1))),
        ty: Ty::Int,
        span: e.span,
    };
    match &cond.kind {
        ExprKind::Binary(BinOp::And, a, b) => {
            let mut out = floors_of(a, slot);
            out.extend(floors_of(b, slot));
            out
        }
        ExprKind::Cmp(CmpOp::Ge, i, e) | ExprKind::Cmp(CmpOp::Le, e, i) if is_slot(i) => {
            vec![(**e).clone()]
        }
        ExprKind::Cmp(CmpOp::Gt, i, e) | ExprKind::Cmp(CmpOp::Lt, e, i) if is_slot(i) => {
            vec![plus_one(e)]
        }
        _ => Vec::new(),
    }
}

fn int(n: i64) -> Expr {
    Expr {
        kind: ExprKind::Int(n),
        ty: Ty::Int,
        span: Default::default(),
    }
}

fn bound(e: &Expr, stored: &BTreeSet<Var>) -> bool {
    match &e.kind {
        ExprKind::Int(_) => true,
        ExprKind::Local(_) | ExprKind::Global(_) => {
            e.ty == Ty::Int && var_of(&e.kind).is_some_and(|v| !moved(stored, &v))
        }
        ExprKind::Field(..) => {
            e.ty == Ty::Int && var_of(&e.kind).is_some_and(|v| !moved(stored, &v))
        }
        ExprKind::Builtin(Builtin::Len, of) => match of.as_slice() {
            [list] => {
                matches!(list.ty, Ty::List(_))
                    && var_of(&list.kind).is_some_and(|v| !moved(stored, &v))
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
                && var_of(&list.kind).is_some_and(|v| !moved(stored, &v))
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
    every: BTreeMap<List, (i64, i64)>,
    /// Grid cells reached on every round, each index as a `Dim`.
    every_grid: Vec<(Var, Vec<Dim>)>,
    /// The same for every index, reached on every round or not.
    any: BTreeMap<List, (i64, i64)>,
    any_grid: Vec<(Var, Vec<Dim>)>,
    /// Whether an index is reached on only some rounds.
    sometimes: bool,
    /// Whether the body indexes a list, and a grid.
    lists: bool,
    grids: bool,
    /// How far each shift of a 64-bit value goes.
    shifts: Vec<Expr>,
    /// The `e` of each enclosing `if index >= e`.
    floors: Vec<Expr>,
    /// Indexes reached under a floor: `(floor, list, c)`.
    floored: Vec<(Expr, List, i64)>,
}

impl Scan<'_, '_> {
    /// False when the body can do anything but compute and store: print,
    /// leave the loop, resize a list, or stop with another message.
    /// `every` is true while each round surely reaches the code.
    fn block(&mut self, block: &Block, every: bool) -> bool {
        block.stmts.iter().all(|stmt| match stmt {
            Stmt::Let(at, init) => {
                self.stored.insert((false, *at, None));
                init.as_ref().is_none_or(|e| self.expr(e, every))
            }
            Stmt::Assign(place, _, value) => self.place(place, every) && self.expr(value, every),
            Stmt::Expr(e) | Stmt::Give(e) => self.expr(e, every),
            Stmt::If(cond, then, otherwise) => {
                let floors = self.floors.len();
                self.floors.extend(floors_of(cond, self.slot));
                let then = self.block(then, false);
                self.floors.truncate(floors);
                self.expr(cond, every) && then && self.block(otherwise, false)
            }
            Stmt::For(
                _,
                at,
                Iter::Range {
                    lo, hi, step: None, ..
                },
                body,
            ) => {
                self.stored.insert((false, *at, None));
                self.expr(lo, every) && self.expr(hi, every) && self.block(body, false)
            }
            Stmt::For(_, at, Iter::Seq(seq, _), body) => {
                self.stored.insert((false, *at, None));
                // Walking a sequence cannot fail; the body is reached on
                // every round only when the sequence is a nonempty literal.
                let full = self.emitter.inline_list_call(seq).is_some_and(
                    |l| matches!(&l.kind, ExprKind::Tuple(items) if !items.is_empty()),
                );
                self.expr(seq, every) && self.block(body, every && full)
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
        let dims: Vec<_> = at
            .iter()
            .map(|e| {
                if crate::expr::proven(std::slice::from_ref(e)) {
                    Dim::Proven
                } else {
                    match offset(e, self.slot) {
                        Some((None, c)) => Dim::Slot(c, c),
                        _ => Dim::Fixed(e.clone()),
                    }
                }
            })
            .collect();
        if every {
            self.every_grid.push((grid, dims.clone()));
        } else {
            self.sometimes = true;
        }
        self.any_grid.push((grid, dims));
        at.iter().all(|e| self.expr(e, every))
    }

    fn index(&mut self, list: Var, index: &Expr, every: bool) -> bool {
        self.lists = true;
        if let (Some((term, c)), false, Some(floor)) =
            (offset(index, self.slot), every, self.floors.last())
        {
            self.sometimes = true;
            self.floored.push((floor.clone(), (list, term), c));
        } else if let Some((term, c)) = offset(index, self.slot) {
            let mut maps = vec![&mut self.any];
            if every {
                maps.push(&mut self.every);
            } else {
                self.sometimes = true;
            }
            for map in maps {
                let range = map.entry((list, term)).or_insert((c, c));
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
            | ExprKind::Unwrap(x, false)
            | ExprKind::Cast(
                Cast::IntToFloat
                | Cast::EnumOrd(_)
                | Cast::EnumVal(_)
                | Cast::Ord
                | Cast::Label(_)
                | Cast::CharToInt
                | Cast::Trust
                | Cast::Raw,
                x,
            ) => self.expr(x, every),
            ExprKind::Unary(op, x) => *op != UnOp::Neg && self.expr(x, every),
            ExprKind::Binary(BinOp::Shl | BinOp::Shr, l, r)
                if matches!(l.ty, Ty::Int | Ty::U64) =>
            {
                if !matches!(r.kind, ExprKind::Int(0..=63)) {
                    self.shifts.push((**r).clone());
                }
                self.expr(l, every) && self.expr(r, every)
            }
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
            // A `give` before the last statement may skip the rest.
            ExprKind::Do(block) => match block.stmts.split_last() {
                Some((Stmt::Give(_), init)) => {
                    let every = every && !init.iter().any(gives);
                    self.block(block, every)
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

fn gives(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Give(_) => true,
        Stmt::If(_, then, otherwise) => then.stmts.iter().chain(&otherwise.stmts).any(gives),
        Stmt::For(.., body) => body.stmts.iter().any(gives),
        _ => false,
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
    fn a_bool_or_enum_axis_needs_no_check() {
        let code = cpp(concat!(
            "enum Side = L | R\n",
            "domain P = read(1..=500)\n",
            "var g: [P.cuts, Bool, Side]Int = 0\n",
            "fn f(j: P.cuts, s: Bool, d: Side) -> Int = g[j, s, d]\n",
            "g[P.cuts.first!, true, R] = 1\n",
            "print(f(P.cuts.first!, true, R))\n",
        ));
        assert!(!code.contains(".ck("), "{code}");
    }

    #[test]
    fn a_grid_loop_that_prints_keeps_its_checks() {
        let code = cpp(&GRID.replace("+ min(k, 3)", "\n        print(k)"));
        assert!(inner(&code).contains(".ck("), "{code}");
    }

    #[test]
    fn a_loop_over_a_grid_and_a_list_checks_in_a_copy() {
        let code = cpp(&GRID
            .replace("var g:", "let w = [i for i in 0..n]\nvar g:")
            .replace("min(k, 3)", "w[k]"));
        // Failing either check could print a different message first, so
        // the loop keeps its checks in the copy a failed test picks.
        let (fast, slow) = code.split_once("} else {").unwrap();
        assert!(!inner(fast).contains(".ck("), "{code}");
        assert!(
            slow.contains("g.ck(y, k)") && slow.contains("cpl::at(w, k)"),
            "{code}"
        );
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

    const STAGES: &str = concat!(
        "enum Stage = Low | High\n",
        "fn roles(b: Int) -> List<(Stage, Stage, Int)> = [(Low, High, b), (High, Low, 2 * b)]\n",
        "let n: Int = read()\n",
        "var g: [0..n + 1, Stage]Int = 0\n",
        "for k in (0..n).rev:\n",
        "    for (from, to, w) in SEQ:\n",
        "        g[k + 1, to] = max(g[k + 1, to], g[k, from] + w)\n",
        "print(g[0, Low])\n",
    );

    #[test]
    fn an_optional_min_assign_reads_its_cell_unchecked() {
        let code = cpp(concat!(
            "domain Stage = 0..2\n",
            "let (lo, hi) = (Stage(0), Stage(1))\n",
            "fn roles(b: Int) -> List<(Stage, Stage, Int)> = [(lo, hi, b), (hi, lo, 2 * b)]\n",
            "let n: Int = read()\n",
            "var g: [0..n + 1, Stage]?Int = None\n",
            "for k in (0..n).rev, (from, to, w) in roles(k):\n",
            "    g[k + 1, to] min= g[k, from] + w\n",
            "print(g[0, lo]!)\n",
        ));
        let body = &code[code.find("for (ll k").unwrap()..];
        assert!(body.contains("= g(k, from)"), "{code}");
        // The store runs only when the new cost is smaller: the copy of the
        // loop picked when `k + 1` fits skips its check, the other keeps it.
        let (fast, slow) = body.split_once("} else {").unwrap();
        assert!(fast.contains("g(k + 1, to) = "), "{code}");
        assert!(slow.contains("g.ck(k + 1, to) = "), "{code}");
    }

    #[test]
    fn a_list_index_some_rounds_reach_picks_a_loop_copy() {
        let code = cpp(concat!(
            "let n: Int = read()\n",
            "var a: List<Int> = read<[n]Int>()\n",
            "for i in 0..n - 1:\n",
            "    if a[i] > 0: a[i + 1] += a[i]\n",
            "print(a)\n",
        ));
        let (fast, slow) = code.split_once("} else {").unwrap();
        assert!(fast.contains("if (!("), "{code}");
        assert!(
            !fast[fast.rfind("for (ll i").unwrap()..].contains("cpl::at("),
            "{code}"
        );
        assert!(slow.contains("cpl::at(a, i + 1)"), "{code}");
    }

    #[test]
    fn a_grid_loop_over_a_literal_fn_checks_ahead() {
        let code = cpp(&STAGES.replace("SEQ", "roles(k)"));
        let body = &code[code.find("for (ll k").unwrap()..];
        assert!(
            !body[..body.find("put_int").unwrap()].contains(".ck("),
            "{code}"
        );
    }

    #[test]
    fn a_grid_loop_over_a_list_that_may_be_empty_keeps_its_checks() {
        let code = cpp(&STAGES.replace("SEQ", "[(Low, High, k) for _ in 0..k]"));
        let body = &code[code.find("for (ll k").unwrap()..];
        assert!(
            body[..body.find("put_int").unwrap()].contains(".ck("),
            "{code}"
        );
    }

    const FIELD: &str = concat!(
        "struct Bits:\n",
        "    w: List<Int>\n",
        "    fn add_from(mut self, words: Int):\n",
        "        for i in (words..w.len).rev:\n",
        "            w[i] += w[i - words]\n",
        "var b = Bits([1, 2, 3, 4])\n",
        "b.add_from(1)\n",
        "print(b.w[3])\n",
    );

    #[test]
    fn a_field_list_at_an_offset_by_a_variable_checks_ahead() {
        let code = cpp(FIELD);
        let body = &code[code.find("for (ll i").unwrap()..];
        let body = &body[..body.find('}').unwrap()];
        assert!(
            code.contains("cpl::fail(\"index outside the list\")") && !body.contains("cpl::at("),
            "{code}"
        );
    }

    #[test]
    fn an_offset_by_a_variable_the_loop_stores_keeps_its_check() {
        let code = cpp(&FIELD
            .replace(
                "add_from(mut self, words: Int):",
                "add_from(mut self, k: Int):\n        var words = k",
            )
            .replace(
                "w[i] += w[i - words]",
                "w[i] += w[i - words]\n            words = 1",
            ));
        let body = &code[code.find("for (ll i").unwrap()..];
        assert!(
            body.contains("cpl::at(self.w, cpl::isub(i, words))"),
            "{code}"
        );
    }

    #[test]
    fn a_loop_that_shifts_by_a_variable_checks_in_a_copy() {
        let code = cpp(concat!(
            "struct Bits:\n",
            "    w: List<U64>\n",
            "    fn or_up(mut self, words: Int, bits: Int):\n",
            "        for i in (words..w.len).rev:\n",
            "            w[i] |= w[i - words] << bits\n",
            "var b = Bits([U64(1), U64(2), U64(3)])\n",
            "b.or_up(1, 3)\n",
            "print(b.w[2])\n",
        ));
        let (fast, slow) = code.split_once("} else {").unwrap();
        let fast = &fast[fast.find("for (ll i").unwrap()..];
        assert!(
            code.contains("(unsigned long long)(bits) >= 64")
                && !fast[..fast.find('}').unwrap()].contains("cpl::at(")
                && slow.contains("cpl::at(self.w, cpl::isub(i, words))"),
            "{code}"
        );
    }

    #[test]
    fn an_index_under_if_i_above_checks_from_there() {
        let code = cpp(concat!(
            "fn main():\n",
            "    let n = 5\n",
            "    var xs = [1, 2, 3, 4, 5]\n",
            "    var s = 0\n",
            "    for i in 0..n:\n",
            "        if i > 0: s += xs[i - 1]\n",
            "    print(s)\n",
            "main()\n",
        ));
        let (fast, slow) = code.split_once("} else {").unwrap();
        assert!(
            fast.contains("std::max((ll)(0), (ll)(0LL + 1))")
                && fast.contains("xs[i - 1]")
                && slow.contains("cpl::at(xs, i - 1)"),
            "{code}"
        );
    }
}
