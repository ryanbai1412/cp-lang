//! A walk over the syntax tree that tells a rule about scopes.
//!
//! [`walk_stmt`] visits every statement, expression, type and binding in
//! source order, and calls [`Visitor::enter`] and [`Visitor::exit`] around
//! everything that opens a scope, so a rule that needs to know what a name
//! refers to keeps a stack and nothing else.

use cpl_diag::Span;
use cpl_syntax::ast::*;

/// What opens a scope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Scope {
    Fn,
    /// The body of a `for` or `while`, and a comprehension.
    Loop,
    Block,
}

/// How a name comes into being.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Bind {
    Let,
    Var,
    Const,
    Param,
    /// A loop variable, a `match` or `if let` binding, a lambda parameter.
    Pattern,
}

/// A rule's view of the walk. Spans are relative to the item being walked.
pub(crate) trait Visitor {
    fn stmt(&mut self, _stmt: &Stmt) {}
    fn expr(&mut self, _expr: &Expr) {}
    fn ty(&mut self, _ty: &Type) {}
    fn bind(&mut self, _name: &str, _span: Span, _how: Bind) {}
    fn enter(&mut self, _scope: Scope) {}
    fn exit(&mut self, _scope: Scope) {}
    /// A `for` loop, after its scope is entered and before its variable is
    /// bound.
    fn for_loop(&mut self, _pat: &Pattern, _iter: &Expr) {}
}

pub(crate) fn walk_stmt(v: &mut dyn Visitor, stmt: &Stmt) {
    v.stmt(stmt);
    for decorator in &stmt.decorators {
        walk_args(v, &decorator.args);
    }
    match &stmt.kind {
        StmtKind::Let {
            mutable,
            pat,
            ty,
            init,
            otherwise,
            guard,
        } => {
            walk_opt_ty(v, ty);
            walk_opt_expr(v, init);
            if let Some(block) = otherwise {
                walk_scoped(v, Scope::Block, &block.stmts);
            }
            let how = if *mutable { Bind::Var } else { Bind::Let };
            walk_pattern(v, pat, how);
            walk_opt_expr(v, guard);
        }
        StmtKind::Const { name, ty, init, .. } => {
            walk_opt_ty(v, ty);
            walk_opt_expr(v, init);
            v.bind(&name.text, name.span, Bind::Const);
        }
        StmtKind::Fn(decl) => walk_fn(v, decl),
        StmtKind::Struct(decl) => {
            for field in &decl.fields {
                walk_ty(v, &field.ty);
                walk_opt_expr(v, &field.default);
            }
            walk_scoped(v, Scope::Block, &decl.methods);
        }
        StmtKind::Enum(decl) => {
            for variant in &decl.variants {
                for (_, ty) in &variant.fields {
                    walk_ty(v, ty);
                }
            }
            walk_scoped(v, Scope::Block, &decl.methods);
        }
        StmtKind::Domain { def, .. } => walk_expr(v, def),
        StmtKind::TypeAlias { ty, .. } => walk_ty(v, ty),
        StmtKind::Trait { body, .. } => walk_scoped(v, Scope::Block, &body.stmts),
        StmtKind::Impl {
            trait_name,
            target,
            body,
            ..
        } => {
            walk_opt_ty(v, trait_name);
            walk_ty(v, target);
            walk_scoped(v, Scope::Block, &body.stmts);
        }
        StmtKind::Use { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::Return(None)
        | StmtKind::Error => {}
        StmtKind::Expr(expr)
        | StmtKind::Return(Some(expr))
        | StmtKind::Lend { place: expr, .. }
        | StmtKind::Yield { value: expr, .. } => walk_expr(v, expr),
        StmtKind::Law { params, body, .. } => {
            v.enter(Scope::Fn);
            for param in params {
                walk_opt_ty(v, &param.ty);
                v.bind(&param.name.text, param.name.span, Bind::Param);
            }
            walk_expr(v, body);
            v.exit(Scope::Fn);
        }
        StmtKind::Assign { target, value, .. } => {
            walk_expr(v, target);
            walk_expr(v, value);
        }
        StmtKind::While { cond, body } => {
            v.enter(Scope::Loop);
            walk_cond(v, cond);
            walk_stmts(v, &body.stmts);
            v.exit(Scope::Loop);
        }
        StmtKind::For {
            pat, iter, body, ..
        } => {
            walk_expr(v, iter);
            v.enter(Scope::Loop);
            v.for_loop(pat, iter);
            walk_pattern(v, pat, Bind::Pattern);
            walk_stmts(v, &body.stmts);
            v.exit(Scope::Loop);
        }
        StmtKind::Assert { cond, message } => {
            walk_expr(v, cond);
            if let Some(message) = message {
                walk_expr(v, message);
            }
        }
        StmtKind::Unchecked(block) => walk_scoped(v, Scope::Block, &block.stmts),
    }
}

fn walk_fn(v: &mut dyn Visitor, decl: &FnDecl) {
    v.enter(Scope::Fn);
    for param in &decl.params {
        walk_opt_ty(v, &param.ty);
        walk_opt_expr(v, &param.default);
        v.bind(&param.name.text, param.name.span, Bind::Param);
    }
    walk_opt_ty(v, &decl.ret);
    match &decl.body {
        FnBody::Block(block) => walk_stmts(v, &block.stmts),
        FnBody::Expr(expr) => walk_expr(v, expr),
        FnBody::None => {}
    }
    v.exit(Scope::Fn);
}

fn walk_stmts(v: &mut dyn Visitor, stmts: &[Stmt]) {
    for stmt in stmts {
        walk_stmt(v, stmt);
    }
}

fn walk_scoped(v: &mut dyn Visitor, scope: Scope, stmts: &[Stmt]) {
    v.enter(scope);
    walk_stmts(v, stmts);
    v.exit(scope);
}

fn walk_opt_ty(v: &mut dyn Visitor, ty: &Option<Type>) {
    if let Some(ty) = ty {
        walk_ty(v, ty);
    }
}

fn walk_opt_expr(v: &mut dyn Visitor, expr: &Option<Expr>) {
    if let Some(expr) = expr {
        walk_expr(v, expr);
    }
}

fn walk_args(v: &mut dyn Visitor, args: &[Arg]) {
    for arg in args {
        walk_expr(v, &arg.value);
    }
}

/// Visits a condition. A `let` condition binds into the scope that is open.
fn walk_cond(v: &mut dyn Visitor, cond: &Cond) {
    match cond {
        Cond::Expr(expr) | Cond::Const(expr) => walk_expr(v, expr),
        Cond::Let { pat, value, and } => {
            walk_expr(v, value);
            walk_pattern(v, pat, Bind::Pattern);
            if let Some(and) = and {
                walk_cond(v, and);
            }
        }
    }
}

pub(crate) fn walk_pattern(v: &mut dyn Visitor, pat: &Pattern, how: Bind) {
    match &pat.kind {
        PatternKind::Name(name) => v.bind(name, pat.span, how),
        PatternKind::Tuple(parts) | PatternKind::Variant { args: parts, .. } => {
            for part in parts {
                walk_pattern(v, part, how);
            }
        }
        // Every alternative binds the same names.
        PatternKind::Or(alts) => {
            if let Some(first) = alts.first() {
                walk_pattern(v, first, how);
            }
        }
        PatternKind::Literal(expr) => walk_expr(v, expr),
        PatternKind::Wildcard | PatternKind::Error => {}
    }
}

pub(crate) fn walk_ty(v: &mut dyn Visitor, ty: &Type) {
    v.ty(ty);
    match &ty.kind {
        TypeKind::Named { args, .. } | TypeKind::Tuple(args) => {
            for arg in args {
                walk_ty(v, arg);
            }
        }
        TypeKind::Array { index, elem } => {
            for ty in index {
                walk_ty(v, ty);
            }
            walk_ty(v, elem);
        }
        TypeKind::OwnDomain { elem, .. } | TypeKind::Optional(elem) | TypeKind::Yields(elem) => {
            walk_ty(v, elem)
        }
        TypeKind::Record(fields) => {
            for (_, ty) in fields {
                walk_ty(v, ty);
            }
        }
        TypeKind::Range { lo, hi, .. } => {
            walk_expr(v, lo);
            walk_expr(v, hi);
        }
        TypeKind::Fn { params, ret } => {
            for ty in params {
                walk_ty(v, ty);
            }
            walk_ty(v, ret);
        }
        TypeKind::Value(expr) => walk_expr(v, expr),
        TypeKind::NewDomain | TypeKind::Error => {}
    }
}

pub(crate) fn walk_expr(v: &mut dyn Visitor, expr: &Expr) {
    v.expr(expr);
    match &expr.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::None
        | ExprKind::Name(_)
        | ExprKind::SelfValue
        | ExprKind::Missing
        | ExprKind::Error => {}
        ExprKind::Str(pieces) => {
            for piece in pieces {
                if let StrPiece::Expr(expr) | StrPiece::Fixed(expr, _) = piece {
                    walk_expr(v, expr);
                }
            }
        }
        ExprKind::Unary { expr, .. }
        | ExprKind::Paren(expr)
        | ExprKind::Try(expr)
        | ExprKind::Unwrap(expr)
        | ExprKind::Field { base: expr, .. } => walk_expr(v, expr),
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr(v, lhs);
            walk_expr(v, rhs);
        }
        ExprKind::Compare { first, rest } => {
            walk_expr(v, first);
            for (_, operand) in rest {
                walk_expr(v, operand);
            }
        }
        ExprKind::Range { lo, hi, step, .. } => {
            for part in [lo, hi, step].into_iter().flatten() {
                walk_expr(v, part);
            }
        }
        ExprKind::Call { callee, args } => {
            walk_expr(v, callee);
            walk_args(v, args);
        }
        ExprKind::Index { base, indices } => {
            walk_expr(v, base);
            for index in indices {
                walk_expr(v, index);
            }
        }
        ExprKind::Generic { base, args } => {
            walk_expr(v, base);
            for ty in args {
                walk_ty(v, ty);
            }
        }
        ExprKind::Map(entries) => {
            for (key, value) in entries {
                walk_expr(v, key);
                walk_expr(v, value);
            }
        }
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            for item in items {
                walk_expr(v, item);
            }
        }
        ExprKind::Record(fields) => {
            for (_, value) in fields {
                walk_expr(v, value);
            }
        }
        ExprKind::Comprehension { body, clauses } => {
            v.enter(Scope::Loop);
            for clause in clauses {
                walk_expr(v, &clause.iter);
                walk_pattern(v, &clause.pat, Bind::Pattern);
                for cond in &clause.conds {
                    walk_cond(v, cond);
                }
            }
            walk_expr(v, body);
            v.exit(Scope::Loop);
        }
        ExprKind::Table {
            domains,
            params,
            body,
        } => {
            for ty in domains {
                walk_ty(v, ty);
            }
            walk_lambda(v, params, body);
        }
        ExprKind::Lambda { params, body } => walk_lambda(v, params, body),
        ExprKind::Block(block) => walk_scoped(v, Scope::Block, &block.stmts),
        ExprKind::If {
            branches,
            otherwise,
        } => {
            for (cond, block) in branches {
                v.enter(Scope::Block);
                walk_cond(v, cond);
                walk_stmts(v, &block.stmts);
                v.exit(Scope::Block);
            }
            if let Some(block) = otherwise {
                walk_scoped(v, Scope::Block, &block.stmts);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            walk_expr(v, scrutinee);
            for arm in arms {
                v.enter(Scope::Block);
                walk_pattern(v, &arm.pat, Bind::Pattern);
                walk_opt_expr(v, &arm.guard);
                walk_stmts(v, &arm.body.stmts);
                v.exit(Scope::Block);
            }
        }
        ExprKind::With { base, updates } => {
            walk_expr(v, base);
            for (_, value) in updates {
                walk_expr(v, value);
            }
        }
    }
}

fn walk_lambda(v: &mut dyn Visitor, params: &[Pattern], body: &Expr) {
    v.enter(Scope::Block);
    for param in params {
        walk_pattern(v, param, Bind::Pattern);
    }
    walk_expr(v, body);
    v.exit(Scope::Block);
}

/// The variable an expression reads or writes through: `g` for `g[u].w`.
pub(crate) fn root_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Name(name) => Some(name),
        ExprKind::Index { base, .. }
        | ExprKind::Field { base, .. }
        | ExprKind::Paren(base)
        | ExprKind::Unwrap(base) => root_name(base),
        _ => None,
    }
}

/// `expr` with any parentheses around it removed.
pub(crate) fn unparen(expr: &Expr) -> &Expr {
    match &expr.kind {
        ExprKind::Paren(inner) => unparen(inner),
        _ => expr,
    }
}
