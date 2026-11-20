//! Walking the direct sub-expressions of an expression.

use cpl_ir::{Arg, Block, DimSpec, Expr, ExprKind, FmtPiece, Iter, Place, PlaceKind, Spec, Stmt};
use std::collections::BTreeSet;

/// Calls `f` on each index expression along `place`.
pub(crate) fn place_exprs<'p>(place: &'p Place, f: &mut dyn FnMut(&'p Expr)) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => place_exprs(base, f),
        PlaceKind::Index(base, indices) => {
            place_exprs(base, f);
            indices.iter().for_each(f);
        }
    }
}

/// Calls `f` on each expression directly inside `e`. Statements of a
/// `Do` block are not expressions and are left to the caller.
pub(crate) fn children<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Expr)) {
    match &e.kind {
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
            indices.iter().for_each(f);
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
            items.iter().for_each(f)
        }
        ExprKind::Call(_, args) => {
            for arg in args {
                match arg {
                    Arg::Value(value) => f(value),
                    Arg::Mut(place) => place_exprs(place, f),
                }
            }
        }
        ExprKind::CallExpr(callee, args) => {
            f(callee);
            args.iter().for_each(f);
        }
        ExprKind::Range { lo, hi } => {
            f(lo);
            f(hi);
        }
        ExprKind::Mutate(_, place, args) => {
            place_exprs(place, f);
            args.iter().for_each(f);
        }
        ExprKind::Sort(place, sort_by) => {
            place_exprs(place, f);
            match sort_by {
                cpl_ir::SortBy::Natural => {}
                cpl_ir::SortBy::Key(_, key) => f(key),
                cpl_ir::SortBy::Fn { func, .. } => f(func),
            }
        }
        ExprKind::Read(s) => {
            // Bounds and lengths inside a spec live as long as the spec.
            let s: &'p Spec = s;
            spec_p(s, f);
        }
        ExprKind::Print(args) => args.iter().for_each(|arg| f(&arg.value)),
        ExprKind::Format(pieces) => {
            for piece in pieces {
                if let FmtPiece::Value(arg) = piece {
                    f(&arg.value);
                }
            }
        }
        ExprKind::Fill(ds, value) => {
            for dim in ds {
                if let DimSpec::Int(len) = dim {
                    f(len);
                }
            }
            if let Some(value) = value {
                f(value);
            }
        }
    }
}

fn spec_p<'p>(s: &'p Spec, f: &mut dyn FnMut(&'p Expr)) {
    match s {
        Spec::Plain(_) => {}
        Spec::Int { lo, hi, .. } => {
            for bound in [lo, hi].into_iter().flatten() {
                f(bound);
            }
        }
        Spec::Fields(_, fields) => fields.iter().for_each(|field| spec_p(field, f)),
        Spec::Array { dims: ds, elem } => {
            for dim in ds {
                if let DimSpec::Int(len) = dim {
                    f(len);
                }
            }
            spec_p(elem, f);
        }
        Spec::Opt(inner) => spec_p(inner, f),
    }
}

/// Calls `f` on each expression a statement holds directly: not those
/// inside its nested blocks.
fn stmt_exprs<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Expr)) {
    match stmt {
        Stmt::Let(_, init) => init.iter().for_each(f),
        Stmt::Assign(place, _, value) => {
            f(value);
            place_exprs(place, f);
        }
        Stmt::Expr(e) | Stmt::Give(e) | Stmt::If(e, ..) | Stmt::While(_, e, _) => f(e),
        Stmt::Return(value) => value.iter().for_each(f),
        Stmt::For(_, _, iter, _) => match iter {
            Iter::Range { lo, hi, step, .. } => {
                f(lo);
                f(hi);
                step.iter().for_each(f);
            }
            Iter::RangeVal(e, _) | Iter::Seq(e, _) => f(e),
        },
        Stmt::Assert { cond, message, .. } => {
            f(cond);
            message.iter().for_each(f);
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
fn stmt_blocks<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Block)) {
    match stmt {
        Stmt::If(_, then, otherwise) => {
            f(then);
            f(otherwise);
        }
        Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => f(body),
        _ => {}
    }
}

/// Calls `f` on `e` and on every expression inside it, at any depth,
/// including those in the statements of a `Do` block.
pub(crate) fn deep_expr<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Expr)) {
    f(e);
    if let ExprKind::Do(block) = &e.kind {
        deep_exprs(block, f);
    }
    children(e, &mut |child| deep_expr(child, f));
}

/// Calls `f` on every expression in `block`, at any depth.
pub(crate) fn deep_exprs<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    for stmt in &block.stmts {
        stmt_exprs(stmt, &mut |e| deep_expr(e, f));
        stmt_blocks(stmt, &mut |inner| deep_exprs(inner, f));
    }
}

/// Calls `f` on every statement in `block`, at any depth.
pub(crate) fn deep_stmts<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Stmt)) {
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
pub(crate) fn root(place: &Place) -> (bool, usize) {
    match &place.kind {
        PlaceKind::Local(slot) => (false, *slot),
        PlaceKind::Global(id) => (true, *id),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => root(base),
    }
}

/// The variable a path expression reads from, if `e` is one: a variable,
/// or fields and elements of one.
pub(crate) fn path_root(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(id) => Some((true, *id)),
        ExprKind::Field(base, _) => path_root(base),
        ExprKind::Index(base, indices) if indices.iter().all(is_simple) => path_root(base),
        _ => None,
    }
}

/// A literal or a variable: nothing that runs code.
pub(crate) fn is_simple(e: &Expr) -> bool {
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
pub(crate) struct Effects {
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
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _) => self.store(place),
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
pub(crate) fn has_give(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| {
        let mut found = matches!(stmt, Stmt::Give(_));
        stmt_blocks(stmt, &mut |inner| found |= has_give(&inner.stmts));
        found
    })
}
