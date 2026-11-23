//! Walking the direct sub-expressions of an expression.

use crate::{Arg, Block, DimSpec, Expr, ExprKind, FmtPiece, Iter, Place, PlaceKind, Spec, Stmt};
use crate::{Builtin, Local, Slot, Ty};
use std::collections::BTreeSet;

macro_rules! walkers {
    (
        <$($lt:lifetime)?> [$($m:tt)*] $iter:ident,
        $place:ident, $children:ident, $spec:ident, $stmt_exprs:ident, $stmt_blocks:ident
    ) => {
        /// Calls `f` on each index expression along `place`.
        pub fn $place<$($lt)?>(place: &$($lt)? $($m)* Place, f: &mut dyn FnMut(&$($lt)? $($m)* Expr)) {
            match &$($m)* place.kind {
                PlaceKind::Local(_) | PlaceKind::Global(_) => {}
                PlaceKind::Field(base, _) | PlaceKind::Some(base) => $place(base, f),
                PlaceKind::Index(base, indices) => {
                    $place(base, f);
                    indices.$iter().for_each(f);
                }
            }
        }

        /// Calls `f` on each expression directly inside `e`. Statements of a
        /// `Do` block are not expressions and are left to the caller.
        pub fn $children<$($lt)?>(e: &$($lt)? $($m)* Expr, f: &mut dyn FnMut(&$($lt)? $($m)* Expr)) {
            match &$($m)* e.kind {
                ExprKind::Unit
                | ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Str(_)
                | ExprKind::None
                | ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::DomainLen(_)
                | ExprKind::DomainLo(_)
                | ExprKind::DomainVal(_)
                | ExprKind::Closure(_)
                | ExprKind::Do(_)
                | ExprKind::Error => {}
                ExprKind::Some(a)
                | ExprKind::Unary(_, a)
                | ExprKind::Cast(_, a)
                | ExprKind::Field(a, _)
                | ExprKind::IsVariant(a, _)
                | ExprKind::Payload(a, _, _)
                | ExprKind::IsSome(a)
                | ExprKind::Unwrap(a, _) => f(a),
                ExprKind::Binary(_, a, b) | ExprKind::Cmp(_, a, b) => {
                    f(a);
                    f(b);
                }
                ExprKind::If(a, b, c) => {
                    f(a);
                    f(b);
                    f(c);
                }
                ExprKind::Index(base, indices) => {
                    f(base);
                    indices.$iter().for_each(f);
                }
                ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
                    items.$iter().for_each(f)
                }
                ExprKind::Call(_, args) => {
                    for arg in args {
                        match arg {
                            Arg::Value(value) => f(value),
                            Arg::Mut(place) => $place(place, f),
                        }
                    }
                }
                ExprKind::CallExpr(callee, args) => {
                    f(callee);
                    for arg in args {
                        match arg {
                            Arg::Value(value) => f(value),
                            Arg::Mut(place) => $place(place, f),
                        }
                    }
                }
                ExprKind::Range { lo, hi } => {
                    f(lo);
                    f(hi);
                }
                ExprKind::RangeCheck { value, lo, hi, .. } => {
                    f(value);
                    if let Some(lo) = lo {
                        f(lo);
                    }
                    if let Some(hi) = hi {
                        f(hi);
                    }
                }
                ExprKind::MutSlice(place, lo, hi) => {
                    $place(place, f);
                    f(lo);
                    f(hi);
                }
                ExprKind::Mutate(_, place, args) => {
                    $place(place, f);
                    args.$iter().for_each(f);
                }
                ExprKind::Sort(place, sort_by, desc) => {
                    $place(place, f);
                    if let Some(desc) = desc {
                        f(desc);
                    }
                    match sort_by {
                        crate::SortBy::Natural => {}
                        crate::SortBy::Key(_, key) => f(key),
                        crate::SortBy::Fn { func, .. } => f(func),
                    }
                }
                ExprKind::Read(s) => {
                    $spec(s, f);
                }
                ExprKind::Parse(src, s) => {
                    f(src);
                    $spec(s, f);
                }
                ExprKind::Print(args, end) => {
                    args.$iter().for_each(|arg| f(&$($m)* arg.value));
                    end.$iter().for_each(|end| f(end));
                }
                // Only the variables a `debug(...)` names reach the C++.
                ExprKind::Debug(_, args) => args
                    .$iter()
                    .filter(|arg| matches!(arg.value.kind, ExprKind::Local(_) | ExprKind::Global(_)))
                    .for_each(|arg| f(&$($m)* arg.value)),
                ExprKind::Format(pieces) => {
                    for piece in pieces {
                        match piece {
                            FmtPiece::Value(arg) => f(&$($m)* arg.value),
                            FmtPiece::Number(value, _) => f(value),
                            FmtPiece::Text(_) => {}
                        }
                    }
                }
                ExprKind::Fill(ds, value) => {
                    for dim in ds {
                        if let DimSpec::Int(len) | DimSpec::IntFrom(_, len) = dim {
                            f(len);
                        }
                    }
                    if let Some(value) = value {
                        f(value);
                    }
                }
            }
        }

        fn $spec<$($lt)?>(s: &$($lt)? $($m)* Spec, f: &mut dyn FnMut(&$($lt)? $($m)* Expr)) {
            match s {
                Spec::Plain(_) | Spec::Line => {}
                Spec::Int { lo, hi, .. } => {
                    for bound in [lo, hi].into_iter().flatten() {
                        f(bound);
                    }
                }
                Spec::Fields(_, fields) => fields.$iter().for_each(|field| $spec(field, f)),
                Spec::Call(call) => f(call),
                Spec::List(each) => $spec(each, f),
                Spec::Parse(text, inner) => {
                    f(text);
                    $spec(inner, f);
                }
                Spec::Array { dims: ds, elem } => {
                    for dim in ds {
                        if let DimSpec::Int(len) | DimSpec::IntFrom(_, len) = dim {
                            f(len);
                        }
                    }
                    $spec(elem, f);
                }
                Spec::Opt(inner, absent) => {
                    $spec(inner, f);
                    if let Some(absent) = absent {
                        f(absent);
                    }
                }
            }
        }

        /// Calls `f` on each expression a statement holds directly: not those
        /// inside its nested blocks.
        pub fn $stmt_exprs<$($lt)?>(stmt: &$($lt)? $($m)* Stmt, f: &mut dyn FnMut(&$($lt)? $($m)* Expr)) {
            match stmt {
                Stmt::Let(_, init) => init.$iter().for_each(f),
                Stmt::Assign(place, _, value) => {
                    f(value);
                    $place(place, f);
                }
                Stmt::Expr(e) | Stmt::Give(e) | Stmt::If(e, ..) | Stmt::While(_, e, _) => f(e),
                Stmt::Return(value) => value.$iter().for_each(f),
                Stmt::For(_, _, iter, _) => match iter {
                    Iter::Range { lo, hi, step, .. } => {
                        f(lo);
                        f(hi);
                        step.$iter().for_each(f);
                    }
                    Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => f(e),
                    Iter::TreeRange {
                        container, lo, hi, ..
                    } => {
                        f(container);
                        if let Some((e, _)) = lo {
                            f(e);
                        }
                        if let Some((e, _)) = hi {
                            f(e);
                        }
                    }
                },
                Stmt::Assert { cond, message, .. } => {
                    f(cond);
                    message.$iter().for_each(f);
                }
                Stmt::DefDomain { lo, len, .. } => {
                    f(lo);
                    f(len);
                }
                Stmt::DefDomainFromVal { val, .. } => f(val),
                Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
            }
        }

        /// The blocks directly inside a statement.
        pub fn $stmt_blocks<$($lt)?>(stmt: &$($lt)? $($m)* Stmt, f: &mut dyn FnMut(&$($lt)? $($m)* Block)) {
            match stmt {
                Stmt::If(_, then, otherwise) => {
                    f(then);
                    f(otherwise);
                }
                Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => f(body),
                _ => {}
            }
        }
    };
}

walkers!(<'p> [] iter, place_exprs, children, spec_p, stmt_exprs, stmt_blocks);
walkers!(<> [mut] iter_mut, place_exprs_mut, children_mut, spec_p_mut, stmt_exprs_mut, stmt_blocks_mut);

/// Calls `f` on every block inside `block`, at any depth (the statements
/// of a `Do` too), inner blocks first and `block` itself last.
pub fn blocks_mut(block: &mut Block, f: &mut dyn FnMut(&mut Block)) {
    for stmt in &mut block.stmts {
        stmt_exprs_mut(stmt, &mut |e| expr_blocks_mut(e, f));
        stmt_blocks_mut(stmt, &mut |inner| blocks_mut(inner, f));
    }
    f(block);
}

fn expr_blocks_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Block)) {
    if let ExprKind::Do(block) = &mut e.kind {
        blocks_mut(block, f);
    }
    children_mut(e, &mut |child| expr_blocks_mut(child, f));
}

/// Calls `f` on `e` and on every expression inside it, at any depth,
/// including those in the statements of a `Do` block.
pub fn deep_expr<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Expr)) {
    f(e);
    if let ExprKind::Do(block) = &e.kind {
        deep_exprs(block, f);
    }
    children(e, &mut |child| deep_expr(child, f));
}

/// Calls `f` on every expression in `block`, at any depth.
pub fn deep_exprs<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    for stmt in &block.stmts {
        stmt_exprs(stmt, &mut |e| deep_expr(e, f));
        stmt_blocks(stmt, &mut |inner| deep_exprs(inner, f));
    }
}

/// Calls `f` on every statement in `block`, at any depth.
pub fn deep_stmts<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Stmt)) {
    for stmt in &block.stmts {
        f(stmt);
        stmt_exprs(stmt, &mut |e| {
            deep_expr(e, &mut |inner| {
                if let ExprKind::Do(block) = &inner.kind {
                    for stmt in &block.stmts {
                        deep_stmt(stmt, f);
                    }
                }
            })
        });
        stmt_blocks(stmt, &mut |inner| deep_stmts(inner, f));
    }
}

fn deep_stmt<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Stmt)) {
    f(stmt);
    stmt_blocks(stmt, &mut |inner| deep_stmts(inner, f));
}

/// The variable a place is part of: `(is_global, index)`.
pub fn root(place: &Place) -> (bool, usize) {
    match &place.kind {
        PlaceKind::Local(slot) => (false, *slot),
        PlaceKind::Global(id) => (true, *id),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => root(base),
    }
}

/// The variable a path expression reads from, if `e` is one: a variable,
/// or fields and elements of one.
pub fn path_root(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(id) => Some((true, *id)),
        ExprKind::Field(base, _) => path_root(base),
        ExprKind::Index(base, indices) if indices.iter().all(is_simple) => path_root(base),
        _ => None,
    }
}

/// A literal or a variable: nothing that runs code.
pub fn is_simple(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Cast(_, inner) => is_simple(inner),
        _ => false,
    }
}

/// What running some code can change.
#[derive(Default, Clone, Debug)]
pub struct Effects {
    pub locals: BTreeSet<usize>,
    pub globals: BTreeSet<usize>,
    pub calls: BTreeSet<usize>,
}

impl Effects {
    fn store(&mut self, place: &Place) {
        match root(place) {
            (true, id) => self.globals.insert(id),
            (false, slot) => self.locals.insert(slot),
        };
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Call(id, args) => {
                self.calls.insert(*id);
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        self.store(place);
                    }
                }
            }
            ExprKind::CallExpr(_, args) => {
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        self.store(place);
                    }
                }
            }
            ExprKind::MutSlice(place, _, _)
            | ExprKind::Mutate(_, place, _)
            | ExprKind::Sort(place, _, _) => self.store(place),
            _ => {}
        }
    }

    pub fn of_block(block: &Block) -> Effects {
        let mut effects = Effects::default();
        deep_stmts(block, &mut |stmt| {
            if let Stmt::Assign(place, ..) = stmt {
                effects.store(place);
            }
        });
        deep_exprs(block, &mut |e| effects.expr(e));
        effects
    }

    pub fn of_expr(e: &Expr) -> Effects {
        let mut effects = Effects::default();
        deep_expr(e, &mut |inner| {
            effects.expr(inner);
            if let ExprKind::Do(block) = &inner.kind {
                deep_stmts(block, &mut |stmt| {
                    if let Stmt::Assign(place, ..) = stmt {
                        effects.store(place);
                    }
                });
            }
        });
        effects
    }
}

/// True when one of `stmts`, or a statement in their loops and branches,
/// gives the enclosing block its value.
pub fn has_give(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| {
        let mut found = matches!(stmt, Stmt::Give(_));
        stmt_blocks(stmt, &mut |inner| found |= has_give(&inner.stmts));
        found
    })
}

/// `for i in 0..xs.len:` over a local list `xs` that keeps its length
/// for the whole loop: gives `(xs, i)`, and `xs[i]` in the body is inside
/// the list without a check.
///
/// The body may store to cells of `xs`. It may not store to `xs` whole or
/// to `i`, grow or shrink `xs`, or call a fn or make a closure: `xs` can
/// be a parameter that names a list the callee reaches another way.
pub fn inside_list(
    slot: Slot,
    iter: &Iter,
    body: &Block,
    locals: &[Local],
) -> Option<(Slot, Slot)> {
    let Iter::Range {
        lo,
        hi,
        step,
        closed: false,
        ..
    } = iter
    else {
        return None;
    };
    let from_zero = matches!(lo.kind, ExprKind::Int(lo) if lo >= 0);
    let forward = step
        .as_ref()
        .is_none_or(|step| matches!(step.kind, ExprKind::Int(step) if step > 0));
    let ExprKind::Builtin(Builtin::Len, of) = &hi.kind else {
        return None;
    };
    let [Expr {
        kind: ExprKind::Local(list),
        ..
    }] = of.as_slice()
    else {
        return None;
    };
    let list = *list;
    if !from_zero
        || !forward
        || !matches!(locals[list].ty, Ty::List(_))
        || locals[slot].ty != Ty::Int
    {
        return None;
    }
    let whole =
        |place: &Place| matches!(place.kind, PlaceKind::Local(at) if at == list || at == slot);
    let mut kept = true;
    deep_stmts(body, &mut |stmt| match stmt {
        Stmt::Assign(place, ..) => kept &= !whole(place),
        Stmt::Let(at, _) | Stmt::For(_, at, ..) => kept &= *at != list && *at != slot,
        _ => {}
    });
    deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Closure(_) => kept = false,
        ExprKind::MutSlice(place, _, _)
        | ExprKind::Mutate(_, place, _)
        | ExprKind::Sort(place, ..) => kept &= !whole(place),
        _ => {}
    });
    kept.then_some((list, slot))
}
