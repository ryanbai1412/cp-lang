//! Where code puts values: the places it writes, the structs it builds,
//! the fns it passes them to and the closures it makes.

use crate::walk;
use cpl_ir::{
    Arg, AssignOp, Block, ClosureId, Expr, ExprKind, FnId, Mutation, Place, Slot, Stmt, Ty,
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
pub(crate) fn each_call<'a>(block: &'a Block, f: &mut dyn FnMut(FnId, &'a [Arg])) {
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
pub(crate) fn each_return<'p>(block: &'p Block, f: &mut dyn FnMut(Option<&'p Expr>)) {
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

/// Every closure `block` makes, at any depth, and where.
pub(crate) fn each_closure_at(block: &Block, f: &mut dyn FnMut(ClosureId, cpl_diag::Span)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            f(cid, e.span);
        }
    });
}

/// Every local or global `stmt` reads, at any depth, as `(is_global, slot)`.
pub(crate) fn each_variable_in(stmt: &Stmt, f: &mut dyn FnMut((bool, Slot))) {
    stmt_deep_exprs(stmt, &mut |e| match e.kind {
        ExprKind::Local(s) => f((false, s)),
        ExprKind::Global(g) => f((true, g)),
        _ => {}
    });
}

/// Every `for` loop in `block`, at any depth: its variable and body.
pub(crate) fn each_for_body<'p>(block: &'p Block, f: &mut dyn FnMut(Slot, &'p Block)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::For(_, slot, _, body) = stmt {
            f(*slot, body);
        }
    });
}

/// Every `let` in `block`, at any depth, with or without a value: its slot.
pub(crate) fn each_declared(block: &Block, f: &mut dyn FnMut(Slot)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Let(slot, _) = stmt {
            f(*slot);
        }
    });
}

/// Every optional `block` tests (`x != None`), at any depth: the optional.
pub(crate) fn each_tested<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::IsSome(x) = &e.kind {
            f(x);
        }
    });
}

/// Every `opt ?? default` in `block`, at any depth: the whole and the optional.
pub(crate) fn each_coalesce<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr, &'p Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Binary(cpl_ir::BinOp::Coalesce, opt, _) = &e.kind {
            f(e, opt);
        }
    });
}

/// Every sort by a key fn in `block`, at any depth: the key fn.
pub(crate) fn each_sort_key<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Sort(_, cpl_ir::SortBy::Fn { func, .. }, _) = &e.kind {
            f(func);
        }
    });
}

/// Every local or global `block` reads, at any depth, as `(is_global, slot)`.
pub(crate) fn each_variable(block: &Block, f: &mut dyn FnMut((bool, Slot))) {
    walk::deep_exprs(block, &mut |e| match e.kind {
        ExprKind::Local(s) => f((false, s)),
        ExprKind::Global(g) => f((true, g)),
        _ => {}
    });
}

/// Every field read `base.field` in `block`, at any depth: the base and field.
pub(crate) fn each_field<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr, usize)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Field(base, field) = &e.kind {
            f(base, *field);
        }
    });
}

/// Every payload read in `block`, at any depth: the read and its base.
pub(crate) fn each_payload<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr, &'p Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Payload(base, ..) = &e.kind {
            f(e, base);
        }
    });
}

/// The base of every read of a part of a value in `block`, at any depth: an
/// index, a field, a payload, a variant test or a length.
pub(crate) fn each_part_base<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Builtin(cpl_ir::Builtin::Len, args) if args.len() == 1 => f(&args[0]),
        ExprKind::Index(base, _)
        | ExprKind::Field(base, _)
        | ExprKind::Payload(base, ..)
        | ExprKind::IsVariant(base, _) => f(base),
        _ => {}
    });
}
