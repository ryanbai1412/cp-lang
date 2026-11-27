//! Walking a syntax tree.
//!
//! [`walk_stmt`] and [`walk_expr`] call a [`Visitor`] on every statement
//! and expression under a node, outermost first, in source order. Checks
//! that ask a question of a whole subtree ("is `part` named below?", "what
//! does this loop assign?") are written as a visitor.

use crate::ast::*;
use cpl_diag::Span;

pub trait Visitor {
    fn stmt(&mut self, _stmt: &Stmt) {}
    fn expr(&mut self, _expr: &Expr) {}
    fn pattern(&mut self, _pat: &Pattern) {}
    /// A block, before its statements.
    fn block(&mut self, _block: &Block) {}
}

pub fn walk_block(block: &Block, v: &mut dyn Visitor) {
    v.block(block);
    for stmt in &block.stmts {
        walk_stmt(stmt, v);
    }
}

fn walk_cond(cond: &Cond, v: &mut dyn Visitor) {
    match cond {
        Cond::Expr(expr) | Cond::Const(expr) => walk_expr(expr, v),
        Cond::Guard { test, and } => {
            walk_expr(test, v);
            walk_cond(and, v);
        }
        Cond::Let { pat, value, and } => {
            v.pattern(pat);
            walk_expr(value, v);
            if let Some(and) = and {
                walk_cond(and, v);
            }
        }
    }
}

fn walk_fn(f: &FnDecl, v: &mut dyn Visitor) {
    for param in &f.params {
        if let Some(default) = &param.default {
            walk_expr(default, v);
        }
    }
    match &f.body {
        FnBody::Block(block) => walk_block(block, v),
        FnBody::Expr(expr) => walk_expr(expr, v),
        FnBody::None => {}
    }
}

pub fn walk_stmt(stmt: &Stmt, v: &mut dyn Visitor) {
    v.stmt(stmt);
    for decorator in &stmt.decorators {
        for arg in &decorator.args {
            walk_expr(&arg.value, v);
        }
    }
    match &stmt.kind {
        StmtKind::Let {
            pat,
            init,
            otherwise,
            guard,
            ..
        } => {
            v.pattern(pat);
            if let Some(init) = init {
                walk_expr(init, v);
            }
            if let Some(otherwise) = otherwise {
                walk_block(otherwise, v);
            }
            if let Some(guard) = guard {
                walk_expr(guard, v);
            }
        }
        StmtKind::Const { init, .. } => {
            if let Some(init) = init {
                walk_expr(init, v);
            }
        }
        StmtKind::Fn(f) => walk_fn(f, v),
        StmtKind::Struct(s) => {
            for field in &s.fields {
                if let Some(default) = &field.default {
                    walk_expr(default, v);
                }
            }
            for method in &s.methods {
                walk_stmt(method, v);
            }
        }
        StmtKind::Enum(e) => {
            for method in &e.methods {
                walk_stmt(method, v);
            }
        }
        StmtKind::Domain { def, .. } => walk_expr(def, v),
        StmtKind::Trait { body, .. } | StmtKind::Impl { body, .. } | StmtKind::Mod { body, .. } => {
            walk_block(body, v)
        }
        StmtKind::Law { body, .. } => walk_expr(body, v),
        StmtKind::Lend { place, .. } => walk_expr(place, v),
        StmtKind::Expr(expr) => walk_expr(expr, v),
        StmtKind::Assign { target, value, .. } => {
            walk_expr(target, v);
            walk_expr(value, v);
        }
        StmtKind::While {
            cond,
            body,
            else_body,
        } => {
            walk_cond(cond, v);
            walk_block(body, v);
            if let Some(else_body) = else_body {
                walk_block(else_body, v);
            }
        }
        StmtKind::For {
            pat,
            iter,
            body,
            else_body,
            ..
        } => {
            v.pattern(pat);
            walk_expr(iter, v);
            walk_block(body, v);
            if let Some(else_body) = else_body {
                walk_block(else_body, v);
            }
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                walk_expr(value, v);
            }
        }
        StmtKind::Yield { value, .. } => walk_expr(value, v),
        StmtKind::Assert { cond, message } => {
            walk_expr(cond, v);
            if let Some(message) = message {
                walk_expr(message, v);
            }
        }
        StmtKind::Unchecked(block) => walk_block(block, v),
        StmtKind::With { args, body, .. } => {
            for arg in args {
                walk_expr(&arg.value, v);
            }
            if let Some(body) = body {
                walk_block(body, v);
            }
        }
        StmtKind::Context { fields, .. } => {
            for f in fields {
                if let Some(d) = &f.default {
                    walk_expr(d, v);
                }
            }
        }
        StmtKind::TypeAlias { .. }
        | StmtKind::Use { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::Error => {}
    }
}

/// Whether `expr` names any of `names` anywhere inside it.
pub fn mentions_any(expr: &Expr, names: &[&str]) -> bool {
    let mut find = Find {
        names,
        found: false,
    };
    walk_expr(expr, &mut find);
    find.found
}

/// Whether `stmt` names any of `names` anywhere inside it.
pub fn stmt_mentions_any(stmt: &Stmt, names: &[&str]) -> bool {
    let mut find = Find {
        names,
        found: false,
    };
    walk_stmt(stmt, &mut find);
    find.found
}

/// The names `pat` binds, in order.
pub fn pattern_names(pat: &Pattern, out: &mut Vec<String>) {
    match &pat.kind {
        PatternKind::Name(n) => out.push(n.clone()),
        PatternKind::Domain(n) => out.push(n.text.clone()),
        PatternKind::Tuple(ps) | PatternKind::Variant { args: ps, .. } => {
            ps.iter().for_each(|p| pattern_names(p, out))
        }
        PatternKind::Record { fields, .. } => {
            fields.iter().for_each(|(_, p)| pattern_names(p, out))
        }
        // Each alternative binds the same names.
        PatternKind::Or(ps) => ps.iter().take(1).for_each(|p| pattern_names(p, out)),
        PatternKind::Var(p) => pattern_names(p, out),
        PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
    }
}

struct Find<'a> {
    names: &'a [&'a str],
    found: bool,
}

impl Visitor for Find<'_> {
    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Name(n) = &expr.kind {
            self.found |= self.names.contains(&n.as_str());
        }
    }
}

pub fn walk_expr(expr: &Expr, v: &mut dyn Visitor) {
    v.expr(expr);
    match &expr.kind {
        ExprKind::Str(pieces) => {
            for piece in pieces {
                if let StrPiece::Expr(inner) | StrPiece::Spec(inner, _) = piece {
                    walk_expr(inner, v);
                }
            }
        }
        ExprKind::By { walk, step } => {
            walk_expr(walk, v);
            walk_expr(step, v);
        }
        ExprKind::Unary { expr: inner, .. }
        | ExprKind::Paren(inner)
        | ExprKind::Try(inner)
        | ExprKind::Unwrap(inner)
        | ExprKind::Mut(inner)
        | ExprKind::Generic { base: inner, .. }
        | ExprKind::Field { base: inner, .. } => walk_expr(inner, v),
        ExprKind::Binary { lhs, rhs, .. }
        | ExprKind::Keep {
            target: lhs,
            value: rhs,
            ..
        } => {
            walk_expr(lhs, v);
            walk_expr(rhs, v);
        }
        ExprKind::Compare { first, rest } => {
            walk_expr(first, v);
            for (_, operand) in rest {
                walk_expr(operand, v);
            }
        }
        ExprKind::Range { lo, hi, step, .. } => {
            for part in [lo, hi, step].into_iter().flatten() {
                walk_expr(part, v);
            }
        }
        ExprKind::Call { callee, args } => {
            walk_expr(callee, v);
            for arg in args {
                walk_expr(&arg.value, v);
            }
        }
        ExprKind::Index { base, indices } => {
            walk_expr(base, v);
            for index in indices {
                walk_expr(index, v);
            }
        }
        ExprKind::Map(entries) => {
            for (key, value) in entries {
                walk_expr(key, v);
                walk_expr(value, v);
            }
        }
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            for item in items {
                walk_expr(item, v);
            }
        }
        ExprKind::Record(fields) => {
            for (_, value) in fields {
                walk_expr(value, v);
            }
        }
        ExprKind::Comprehension { body, clauses } => {
            for clause in clauses {
                v.pattern(&clause.pat);
                walk_expr(&clause.iter, v);
                for cond in &clause.conds {
                    walk_cond(cond, v);
                }
            }
            walk_expr(body, v);
        }
        ExprKind::Table { params, body, .. } | ExprKind::Lambda { params, body, .. } => {
            for param in params {
                v.pattern(param);
            }
            walk_expr(body, v);
        }
        ExprKind::TableFill { domain, value } => {
            walk_expr(domain, v);
            walk_expr(value, v);
        }
        ExprKind::Block(block) => walk_block(block, v),
        ExprKind::If {
            branches,
            otherwise,
        } => {
            for (cond, body) in branches {
                walk_cond(cond, v);
                walk_block(body, v);
            }
            if let Some(otherwise) = otherwise {
                walk_block(otherwise, v);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            walk_expr(scrutinee, v);
            for arm in arms {
                v.pattern(&arm.pat);
                if let Some(guard) = &arm.guard {
                    walk_expr(guard, v);
                }
                walk_block(&arm.body, v);
            }
        }
        ExprKind::With { base, updates } => {
            walk_expr(base, v);
            for (_, value) in updates {
                walk_expr(value, v);
            }
        }
        ExprKind::Without { prefix, base, .. } => {
            for (_, e) in prefix {
                walk_expr(e, v);
            }
            walk_expr(base, v);
        }
        ExprKind::HasBound { .. } => {}
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::None
        | ExprKind::Name(_)
        | ExprKind::SelfValue
        | ExprKind::Cpp(_)
        | ExprKind::Intrinsic(_)
        | ExprKind::Missing
        | ExprKind::Error => {}
    }
}

/// The `return`, `break` and `continue` statements under `e` that leave
/// it: not inside a `fn` or lambda written under `e`, and, for a `break`
/// or `continue`, not aimed at a loop under `e`.
pub fn leaves_out_of(e: &Expr) -> Vec<Span> {
    #[derive(Default)]
    struct Under {
        /// Each leave, with the loop label a `break` or `continue` aims at.
        leaves: Vec<(Span, Option<Option<String>>)>,
        loops: Vec<(Span, Option<String>)>,
        fns: Vec<Span>,
    }
    impl Visitor for Under {
        fn stmt(&mut self, stmt: &Stmt) {
            let text = |label: &Option<Name>| label.as_ref().map(|l| l.text.clone());
            match &stmt.kind {
                StmtKind::Return(_) => self.leaves.push((stmt.span, None)),
                StmtKind::Break(label) | StmtKind::Continue(label) => {
                    self.leaves.push((stmt.span, Some(text(label))))
                }
                StmtKind::While { body, .. } => self.loops.push((body.span, None)),
                StmtKind::For { label, body, .. } => self.loops.push((body.span, text(label))),
                StmtKind::Fn(_) => self.fns.push(stmt.span),
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if matches!(expr.kind, ExprKind::Lambda { .. }) {
                self.fns.push(expr.span);
            }
        }
    }
    let mut under = Under::default();
    walk_expr(e, &mut under);
    let in_fn = |at: Span| under.fns.iter().any(|f| f.contains(at));
    let in_loop = |at: Span, label: &Option<String>| {
        under
            .loops
            .iter()
            .any(|(body, name)| (label.is_none() || name == label) && body.contains(at))
    };
    under
        .leaves
        .iter()
        .filter(|(at, aim)| !in_fn(*at) && !aim.as_ref().is_some_and(|label| in_loop(*at, label)))
        .map(|(at, _)| *at)
        .collect()
}
