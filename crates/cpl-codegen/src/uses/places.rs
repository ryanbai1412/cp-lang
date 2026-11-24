//! Where code puts values: the places it writes, the structs it builds,
//! the fns it passes them to and the closures it makes.

use crate::walk;
use cpl_ir::{
    Arg, AssignOp, Block, ClosureId, Expr, ExprKind, FnId, Mutation, Place, PlaceKind, Slot, Stmt,
    Ty,
};

/// Where a statement puts a whole value.
#[derive(Clone, Copy)]
pub(crate) enum Put<'a> {
    /// `let slot = v`.
    Let(Slot),
    /// `place = v` or `place op= v`.
    Store(&'a Place, &'a AssignOp),
}

/// Every `let` with a value and every store in `block`, at any depth and in
/// order: where it puts the value, and the value.
pub(crate) fn each_put<'p>(block: &'p Block, f: &mut dyn FnMut(Put<'p>, &'p Expr)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(v)) => f(Put::Let(*slot), v),
        Stmt::Assign(target, op, v) => f(Put::Store(target, op), v),
        _ => {}
    });
}

/// How a write changes its place.
#[derive(Clone, Copy)]
pub(crate) enum Write<'a> {
    /// `place = v` or `place op= v`.
    Store,
    /// A push, pop, insert or other mutation in place.
    Mutate(&'a Mutation),
    /// Sorted in place.
    Sort,
    /// Viewed `mut`: a slice or a line of it.
    InPlace,
    /// Passed `mut` to a call.
    Mut,
}

/// Every place `block` writes, at any depth, and how: each target a write
/// could change.
pub(crate) fn each_write<'p>(block: &'p Block, f: &mut dyn FnMut(Write<'p>, &'p Place)) {
    for stmt in &block.stmts {
        each_write_in(stmt, f);
    }
}

/// [`each_write`] for one statement and everything under it.
pub(crate) fn each_write_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(Write<'p>, &'p Place)) {
    stores_in(stmt, &mut |target| f(Write::Store, target));
    stmt_deep_exprs(stmt, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    f(Write::Mut, target);
                }
            }
        }
        ExprKind::Mutate(how, target, _) => f(Write::Mutate(how), target),
        ExprKind::Sort(target, _, _) => f(Write::Sort, target),
        ExprKind::MutSlice(target, ..) | ExprKind::MutLine(target, ..) => f(Write::InPlace, target),
        _ => {}
    });
}

fn stores_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Place)) {
    if let Stmt::Assign(target, ..) = stmt {
        f(target);
    }
    walk::stmt_exprs(stmt, &mut |e| {
        walk::deep_expr(e, &mut |inner| {
            if let ExprKind::Do(block) = &inner.kind {
                block.stmts.iter().for_each(|s| stores_in(s, f));
            }
        })
    });
    walk::stmt_blocks(stmt, &mut |inner| {
        inner.stmts.iter().for_each(|s| stores_in(s, f));
    });
}

fn stmt_deep_exprs<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Expr)) {
    walk::stmt_exprs(stmt, &mut |e| walk::deep_expr(e, f));
    walk::stmt_blocks(stmt, &mut |inner| walk::deep_exprs(inner, f));
}

/// Every tuple or struct literal in `stmt`, at any depth: its items.
pub(crate) fn each_tuple_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p [Expr])) {
    stmt_deep_exprs(stmt, &mut |e| {
        if let ExprKind::Tuple(items) = &e.kind {
            f(items);
        }
    });
}

/// [`each_write`], whatever the write.
pub(crate) fn each_written(block: &Block, f: &mut dyn FnMut(&Place)) {
    each_write(block, &mut |_, target| f(target));
}

/// Every struct literal in `block`, at any depth: the struct and its
/// fields. A struct value is only ever made by one (every other value of
/// it is a copy of one).
pub(crate) fn each_built(block: &Block, f: &mut dyn FnMut(usize, &[Expr])) {
    walk::deep_exprs(block, &mut |e| {
        if let (ExprKind::Tuple(fields), Ty::Struct(s)) = (&e.kind, &e.ty) {
            f(*s, fields);
        }
    });
}

/// Every call by name in `block`, at any depth: the fn and its arguments.
pub(crate) fn each_call(block: &Block, f: &mut dyn FnMut(FnId, &[Arg])) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Call(callee, args) = &e.kind {
            f(*callee, args);
        }
    });
}

/// Every closure `block` makes, at any depth.
pub(crate) fn each_closure(block: &Block, f: &mut dyn FnMut(ClosureId)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            f(cid);
        }
    });
}

/// Every call of a fn value in `block`, at any depth: the callee and its
/// arguments.
pub(crate) fn each_value_call(block: &Block, f: &mut dyn FnMut(&Expr, &[Arg])) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::CallExpr(callee, args) = &e.kind {
            f(callee, args);
        }
    });
}

/// Every optional `block` tests (`x != None`) or reads through (`x!`), at
/// any depth: the optional.
pub(crate) fn each_opened(block: &Block, f: &mut dyn FnMut(&Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::IsSome(x) | ExprKind::Unwrap(x, false) = &e.kind {
            f(x);
        }
    });
}

/// Every expression inside `e`, `e` itself included, at any depth.
pub(crate) fn each_part<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_expr(e, f);
}

/// Every expression statement in `block`, at any depth: the expression.
pub(crate) fn each_stmt_expr<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Expr(e) = stmt {
            f(e);
        }
    });
}

/// Every local `block` reads, at any depth.
pub(crate) fn each_local(block: &Block, f: &mut dyn FnMut(Slot)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            f(s);
        }
    });
}

/// Every `return` in `block`, at any depth: its value, if any.
pub(crate) fn each_return(block: &Block, f: &mut dyn FnMut(Option<&Expr>)) {
    walk::deep_stmts(block, &mut |s| {
        if let Stmt::Return(value) = s {
            f(value.as_ref());
        }
    });
}

/// Whether every statement of `block`, at any depth, only evaluates, binds,
/// branches, returns or gives: no store, loop, `break`, `continue`, assert,
/// domain or trap.
pub(crate) fn branches_and_binds_only(block: &Block) -> bool {
    let mut only = true;
    walk::deep_stmts(block, &mut |stmt| {
        only &= matches!(
            stmt,
            Stmt::Expr(_) | Stmt::Return(_) | Stmt::Give(_) | Stmt::If(..) | Stmt::Let(..)
        );
    });
    only
}

/// The base of every index read `base[..]` in `block`, at any depth, outermost
/// first.
pub(crate) fn each_index<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Index(base, _) = &e.kind {
            f(base);
        }
    });
}

/// Whether `block` defines a domain at any depth: its sizes change as it runs.
pub(crate) fn defines_domain(block: &Block) -> bool {
    let mut found = false;
    walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. });
    });
    found
}

/// Whether `block` has an `assert` with a note, at any depth.
pub(crate) fn notes_an_assert(block: &Block) -> bool {
    let mut found = false;
    walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::Assert { note: Some(_), .. });
    });
    found
}

/// One step down from a variable to a part of it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Step {
    Field(usize),
    /// Any element of a list or map: two indices may name the same one.
    Elem,
    /// The value inside an optional.
    Opt,
    /// A field of one variant's payload.
    Payload(usize, usize),
}

/// A variable and the steps from it to the part a read or write reaches.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Path {
    pub(crate) root: (bool, usize),
    pub(crate) steps: Vec<Step>,
}

impl Path {
    /// Whether a write to one can change what the other reads: same root,
    /// and one is the other or a part of it. A write above a part may move
    /// or reallocate what holds it.
    pub(crate) fn overlaps(&self, other: &Path) -> bool {
        self.root == other.root
            && self.steps.iter().zip(&other.steps).all(|pair| match pair {
                (Step::Field(a), Step::Field(b)) => a == b,
                (Step::Payload(v, a), Step::Payload(w, b)) => v != w || a == b,
                _ => true,
            })
    }
}

/// The path a write to `place` changes.
pub(crate) fn place_path(place: &Place) -> Path {
    match &place.kind {
        PlaceKind::Local(slot) => Path {
            root: (false, *slot),
            steps: Vec::new(),
        },
        PlaceKind::Global(id) => Path {
            root: (true, *id),
            steps: Vec::new(),
        },
        PlaceKind::Index(base, _) => place_path(base).then(Step::Elem),
        PlaceKind::Field(base, k) => place_path(base).then(Step::Field(*k)),
        PlaceKind::Some(base) => place_path(base).then(Step::Opt),
    }
}

impl Path {
    fn then(mut self, step: Step) -> Path {
        self.steps.push(step);
        self
    }
}

/// The path `e` reads, when it is a part of a variable whose indices
/// change nothing: also through a call that lends a place inlined as
/// `{ let t = place; t }`.
pub(crate) fn read_path(e: &Expr) -> Option<Path> {
    let quiet = crate::uses::effects::writes_nothing;
    match &e.kind {
        ExprKind::Local(slot) => Some(Path {
            root: (false, *slot),
            steps: Vec::new(),
        }),
        ExprKind::Global(id) => Some(Path {
            root: (true, *id),
            steps: Vec::new(),
        }),
        ExprKind::Field(base, k) => Some(read_path(base)?.then(Step::Field(*k))),
        ExprKind::Payload(base, v, k) => Some(read_path(base)?.then(Step::Payload(*v, *k))),
        ExprKind::Unwrap(base, _) => Some(read_path(base)?.then(Step::Opt)),
        ExprKind::Index(base, indices) if indices.iter().all(quiet) => {
            Some(read_path(base)?.then(Step::Elem))
        }
        ExprKind::Do(block) => match block.stmts.as_slice() {
            [Stmt::Let(slot, Some(init)), Stmt::Give(given)] if matches!(given.kind, ExprKind::Local(s) if s == *slot) => {
                read_path(init)
            }
            _ => None,
        },
        _ => None,
    }
}

/// The paths every write in `block` changes.
pub(crate) fn write_paths(block: &Block) -> Vec<Path> {
    let mut paths = Vec::new();
    each_write(block, &mut |_, place| paths.push(place_path(place)));
    paths
}

/// Whether `e` makes a value of its own instead of naming a place: a list,
/// tuple or literal it builds, or a block's local it hands on. A yield of
/// it can hand the value on owned.
pub(crate) fn makes_own_value(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_)
        | ExprKind::Global(_)
        | ExprKind::Field(..)
        | ExprKind::Index(..)
        | ExprKind::Payload(..)
        | ExprKind::Unwrap(..)
        | ExprKind::Call(..)
        | ExprKind::CallExpr(..) => false,
        ExprKind::Do(block) => match block.stmts.last() {
            Some(Stmt::Give(v)) => match v.kind {
                ExprKind::Local(s) => block.stmts.iter().any(
                    |st| matches!(st, Stmt::Let(l, Some(init)) if *l == s && makes_own_value(init)),
                ),
                _ => makes_own_value(v),
            },
            _ => false,
        },
        _ => true,
    }
}

/// Whether every value a `yields` fn whose list is `acc` yields is one it
/// makes ([`makes_own_value`]): the callback may then take each owned.
pub(crate) fn yields_owned(body: &Block, acc: Slot) -> bool {
    let (mut any, mut all) = (false, true);
    each_write(body, &mut |how, place| {
        if matches!(place.kind, PlaceKind::Local(s) if s == acc) {
            any = true;
            all &= matches!(how, Write::Mutate(Mutation::Push));
        }
    });
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Mutate(Mutation::Push, place, items) = &e.kind {
            if matches!(place.kind, PlaceKind::Local(s) if s == acc) {
                all &= items.iter().all(makes_own_value);
            }
        }
    });
    any && all
}
