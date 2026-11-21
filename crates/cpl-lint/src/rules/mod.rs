//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `@allow` afterwards.

mod bounds;
mod fields;
mod floats;
mod imports;
mod lists;
mod memo;
mod mod_div;
mod scopes;
mod taste;
mod types;

use crate::{Config, File, Finding};

pub(crate) fn run(file: &File, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(scopes::check(file));
    findings.extend(types::check(file));
    findings.extend(floats::check(file));
    findings.extend(imports::check(file));
    findings.extend(fields::check(file));
    findings.extend(memo::check(file, config.memory_mb));
    findings.extend(lists::check(file));
    findings.extend(mod_div::check(file));
    findings.extend(taste::check(file));
    findings
}

/// Each immutable `let` without a written type, and each loop variable over
/// a range, whose value the declared bounds prove, as the name's offset and
/// the inclusive range. Inside a fn, a typed range parameter is a bound too.
pub(crate) fn proven_ranges(file: &File) -> Vec<(u32, i128, i128)> {
    proven(file).names
}

/// Each `+`, `-`, `*`, `/`, `%` and `**` whose operands the declared bounds prove, with the
/// range of its value, under the same scoping as [`proven_ranges`].
pub(crate) fn proven_expr_ranges(file: &File) -> Vec<crate::ProvenExpr> {
    proven(file).exprs
}

#[derive(Default)]
struct Proven {
    names: Vec<(u32, i128, i128)>,
    exprs: Vec<crate::ProvenExpr>,
}

fn proven(file: &File) -> Proven {
    let bounds = bounds::Bounds::of(file);
    let mut out = Proven::default();
    // A fn declared above a top-level `var` can assign it too.
    let top: Vec<&cpl_syntax::ast::Stmt> = file.top_level().map(|(_, s)| s).collect();
    for (base, stmt) in file.top_level() {
        ranges_in(
            base,
            std::slice::from_ref(stmt),
            &top,
            &mut bounds.clone(),
            &mut out,
        );
    }
    out.names.sort();
    out.names.dedup();
    out.exprs.sort_by_key(|e| e.op);
    out.exprs.dedup_by_key(|e| e.op);
    out
}

/// The arithmetic in `expr` the bounds prove, without entering anything
/// that binds names of its own (lambdas, comprehensions, match arms, blocks).
fn arith(base: u32, expr: &cpl_syntax::ast::Expr, bounds: &bounds::Bounds, out: &mut Vec<crate::ProvenExpr>) {
    use cpl_syntax::ast::{BinOp, ExprKind};
    match &expr.kind {
        ExprKind::Binary { op, op_span, lhs, rhs } => {
            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow
            ) {
                if let Some(v) = bounds.expr(expr).filter(|v| v.lo != v.hi) {
                    let int = i64::MAX as i128;
                    let leaves = v.lo < -int - 1 || v.hi > int;
                    out.push(crate::ProvenExpr {
                        op: (base + op_span.start, base + op_span.end),
                        expr: (base + expr.span.start, base + expr.span.end),
                        lo: v.lo,
                        hi: v.hi,
                        overflow: leaves.then(|| bounds.overflow(expr)).flatten(),
                    });
                }
            }
            arith(base, lhs, bounds, out);
            arith(base, rhs, bounds, out);
        }
        ExprKind::Unary { expr: inner, .. }
        | ExprKind::Paren(inner)
        | ExprKind::Try(inner)
        | ExprKind::Unwrap(inner)
        | ExprKind::Field { base: inner, .. } => arith(base, inner, bounds, out),
        ExprKind::Compare { first, rest } => {
            arith(base, first, bounds, out);
            rest.iter().for_each(|(_, e)| arith(base, e, bounds, out));
        }
        ExprKind::Range { lo, hi, step, .. } => {
            for e in [lo, hi, step].into_iter().flatten() {
                arith(base, e, bounds, out);
            }
        }
        ExprKind::Call { callee, args } => {
            arith(base, callee, bounds, out);
            args.iter().for_each(|a| arith(base, &a.value, bounds, out));
        }
        ExprKind::Index { base: seq, indices } => {
            arith(base, seq, bounds, out);
            indices.iter().for_each(|e| arith(base, e, bounds, out));
        }
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            items.iter().for_each(|e| arith(base, e, bounds, out));
        }
        _ => {}
    }
}

/// `stmts` is a block, or one top-level statement whose `var`s are in
/// scope across all of `top`.
fn ranges_in(
    base: u32,
    stmts: &[cpl_syntax::ast::Stmt],
    top: &[&cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
    proven: &mut Proven,
) {
    use cpl_syntax::ast::{Cond, ExprKind, FnBody, PatternKind, StmtKind};
    for (i, stmt) in stmts.iter().enumerate() {
        match &stmt.kind {
            StmtKind::Let { init: Some(e), .. }
            | StmtKind::Return(Some(e))
            | StmtKind::Yield { value: e, .. }
            | StmtKind::Assert { cond: e, .. }
            | StmtKind::For { iter: e, .. } => arith(base, e, bounds, &mut proven.exprs),
            StmtKind::Assign { target, value, .. } => {
                arith(base, target, bounds, &mut proven.exprs);
                arith(base, value, bounds, &mut proven.exprs);
            }
            StmtKind::While { cond: Cond::Expr(e), .. } => arith(base, e, bounds, &mut proven.exprs),
            StmtKind::Expr(e) => match &e.kind {
                ExprKind::If { branches, .. } => {
                    for (cond, _) in branches {
                        if let Cond::Expr(c) = cond {
                            arith(base, c, bounds, &mut proven.exprs);
                        }
                    }
                }
                _ => arith(base, e, bounds, &mut proven.exprs),
            },
            _ => {}
        }
        let out = &mut proven.names;
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
                ..
            } => {
                let value = match (&pat.kind, ty) {
                    (PatternKind::Name(_), Some(ty)) => bounds.range(ty),
                    (PatternKind::Name(_), None) if !mutable => {
                        init.as_ref().and_then(|e| bounds.expr(e))
                    }
                    (PatternKind::Name(name), None) => {
                        let scope: Vec<&cpl_syntax::ast::Stmt> = if top.is_empty() {
                            stmts[i + 1..].iter().collect()
                        } else {
                            let at = top.iter().position(|s| std::ptr::eq(*s, stmt)).unwrap_or(0);
                            let earlier = top[..at].iter().filter(|s| {
                                matches!(s.kind, StmtKind::Fn(_) | StmtKind::Struct(_) | StmtKind::Impl { .. })
                            });
                            earlier.chain(&top[at + 1..]).copied().collect()
                        };
                        init.as_ref()
                            .and_then(|e| bounds.expr(e))
                            .and_then(|start| var_range(name, start, &scope, bounds))
                    }
                    _ => None,
                };
                if let (Some(v), None) = (value, ty) {
                    out.push((base + pat.span.start, v.lo, v.hi));
                }
                forget(pat, bounds);
                if let PatternKind::Name(name) = &pat.kind {
                    match (value, ty, init) {
                        (Some(v), Some(_), _) => bounds.input(name, v),
                        (_, None, Some(init)) if !mutable => {
                            bounds.bind(name, value);
                            bounds.define(name, init);
                        }
                        _ => bounds.bind(name, value),
                    }
                }
            }
            StmtKind::Const { name, init, .. } => {
                let value = init.as_ref().and_then(|e| bounds.expr(e));
                bounds.bind(&name.text, value);
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                let mut inner = bounds.clone();
                forget(pat, &mut inner);
                if let (
                    PatternKind::Name(name),
                    ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    },
                ) = (&pat.kind, &iter.kind)
                {
                    let value = inner.span(lo, hi, *inclusive).filter(|v| v.lo <= v.hi);
                    if let Some(v) = value {
                        out.push((base + pat.span.start, v.lo, v.hi));
                    }
                    inner.bind(name, value);
                }
                ranges_in(base, &body.stmts, &[], &mut inner, proven);
            }
            StmtKind::While { cond, body, .. } => {
                let mut inner = bounds.clone();
                if let Cond::Expr(test) = cond {
                    narrow(test, true, &body.stmts, &mut inner);
                }
                ranges_in(base, &body.stmts, &[], &mut inner, proven);
            }
            StmtKind::Unchecked(body) => {
                ranges_in(base, &body.stmts, &[], &mut bounds.clone(), proven);
            }
            StmtKind::Expr(expr) => {
                if let ExprKind::If {
                    branches,
                    otherwise,
                } = &expr.kind
                {
                    // An `if let` binds names the branch would see through.
                    let mut failed = Vec::new();
                    for (cond, block) in branches {
                        let Cond::Expr(test) = cond else {
                            continue;
                        };
                        let mut inner = bounds.clone();
                        for &earlier in &failed {
                            narrow(earlier, false, &block.stmts, &mut inner);
                        }
                        narrow(test, true, &block.stmts, &mut inner);
                        ranges_in(base, &block.stmts, &[], &mut inner, proven);
                        failed.push(test);
                    }
                    if let Some(block) = otherwise {
                        let mut inner = bounds.clone();
                        for &earlier in &failed {
                            narrow(earlier, false, &block.stmts, &mut inner);
                        }
                        ranges_in(base, &block.stmts, &[], &mut inner, proven);
                    }
                }
            }
            StmtKind::Fn(decl) => {
                bounds.declare_fn(&decl.name.text);
                let mut inner = bounds.clone();
                for param in &decl.params {
                    match param.ty.as_ref().and_then(|t| inner.range(t)) {
                        Some(range) => inner.input(&param.name.text, range),
                        None => inner.bind(&param.name.text, None),
                    }
                }
                if let FnBody::Block(block) = &decl.body {
                    inner.declare_fn(&decl.name.text);
                    ranges_in(base, &block.stmts, &[], &mut inner, proven);
                }
            }
            _ => {}
        }
    }
}

/// The range a `var` named `name` that starts in `start` stays in, through
/// every assignment in `scope`: the smallest hull each assignment maps into
/// itself, found in a few rounds, or `None`. Anything else that could change
/// it (a `mut` argument, a tuple assignment, a `lend`) or a second binding of
/// the name gives `None`, and so does an assigned value that reads a name
/// bound in `scope`, since that name is not the one `bounds` knows.
fn var_range(
    name: &str,
    start: bounds::Interval,
    scope: &[&cpl_syntax::ast::Stmt],
    bounds: &bounds::Bounds,
) -> Option<bounds::Interval> {
    use cpl_syntax::ast::{AssignOp, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};

    struct Scan<'a> {
        name: &'a str,
        writes: Vec<(AssignOp, Expr)>,
        bound: Vec<String>,
        opaque: bool,
    }
    fn names(pat: &Pattern, out: &mut Vec<String>) {
        match &pat.kind {
            PatternKind::Name(n) => out.push(n.clone()),
            PatternKind::Domain(n) => out.push(n.text.clone()),
            PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
                parts.iter().for_each(|p| names(p, out))
            }
            PatternKind::Variant { args, .. } => args.iter().for_each(|p| names(p, out)),
            PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
        }
    }
    impl Scan<'_> {
        fn mentions(&self, e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Name(n) => n == self.name,
                ExprKind::Paren(inner) => self.mentions(inner),
                ExprKind::Tuple(items) | ExprKind::List(items) => {
                    items.iter().any(|i| self.mentions(i))
                }
                _ => false,
            }
        }
    }
    impl Visitor for Scan<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign { target, op, value } => {
                    if matches!(&target.kind, ExprKind::Name(n) if n == self.name) {
                        self.writes.push((*op, value.clone()));
                    } else if self.mentions(target) {
                        self.opaque = true;
                    }
                }
                StmtKind::Lend { place, .. } => self.opaque |= self.mentions(place),
                StmtKind::Fn(decl) => {
                    self.bound.extend(decl.params.iter().map(|p| p.name.text.clone()))
                }
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { args, .. } = &expr.kind {
                self.opaque |= args.iter().any(|a| a.mutable && self.mentions(&a.value));
            }
        }
        fn pattern(&mut self, pat: &Pattern) {
            names(pat, &mut self.bound);
        }
    }

    let mut scan = Scan {
        name,
        writes: Vec::new(),
        bound: Vec::new(),
        opaque: false,
    };
    scope.iter().for_each(|s| walk_stmt(s, &mut scan));
    if scan.opaque || scan.bound.iter().any(|b| b == name) {
        return None;
    }
    let mut known = bounds.clone();
    for b in &scan.bound {
        known.bind(b, None);
    }
    let mut hull = start;
    for _ in 0..4 {
        known.bind(name, Some(hull));
        let mut next = hull;
        for (op, value) in &scan.writes {
            let v = known.expr(value)?;
            let after = match op {
                AssignOp::Set => v,
                AssignOp::Binary(op) => bounds::Bounds::combine(*op, hull, v)?,
                AssignOp::Max => bounds::Interval {
                    lo: hull.lo.max(v.lo),
                    hi: hull.hi.max(v.hi),
                },
                AssignOp::Min => bounds::Interval {
                    lo: hull.lo.min(v.lo),
                    hi: hull.hi.min(v.hi),
                },
            };
            next = bounds::Interval {
                lo: next.lo.min(after.lo),
                hi: next.hi.max(after.hi),
            };
        }
        if next == hull {
            return Some(hull);
        }
        hull = next;
    }
    None
}

/// Inside a block that runs only when `test` is `holds`, each bounded name
/// a comparison in `test` limits is in the tighter range, unless the block
/// assigns it.
fn narrow(
    test: &cpl_syntax::ast::Expr,
    holds: bool,
    block: &[cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
) {
    use cpl_syntax::ast::{BinOp, CmpOp, ExprKind, UnOp};
    match &test.kind {
        ExprKind::Paren(inner) => narrow(inner, holds, block, bounds),
        ExprKind::Unary { op: UnOp::Not, expr } => narrow(expr, !holds, block, bounds),
        ExprKind::Binary { op, lhs, rhs, .. }
            if (*op == BinOp::And && holds) || (*op == BinOp::Or && !holds) =>
        {
            narrow(lhs, holds, block, bounds);
            narrow(rhs, holds, block, bounds);
        }
        ExprKind::Compare { first, rest } if holds || rest.len() == 1 => {
            let mut left = first.as_ref();
            for (op, right) in rest {
                let op = &if holds {
                    *op
                } else {
                    match op {
                        CmpOp::Lt => CmpOp::Ge,
                        CmpOp::Le => CmpOp::Gt,
                        CmpOp::Gt => CmpOp::Le,
                        CmpOp::Ge => CmpOp::Lt,
                        CmpOp::Ne => CmpOp::Eq,
                        _ => CmpOp::Ne,
                    }
                };
                let flipped = match op {
                    CmpOp::Lt => Some(CmpOp::Gt),
                    CmpOp::Le => Some(CmpOp::Ge),
                    CmpOp::Gt => Some(CmpOp::Lt),
                    CmpOp::Ge => Some(CmpOp::Le),
                    CmpOp::Eq => Some(CmpOp::Eq),
                    _ => None,
                };
                if let Some(flipped) = flipped {
                    for (name, op, other) in [(left, *op, right), (right, flipped, left)] {
                        let ExprKind::Name(name) = &name.kind else {
                            continue;
                        };
                        let (Some(have), Some(other)) = (bounds.expr_name(name), bounds.expr(other)) else {
                            continue;
                        };
                        if assigned_in(name, block) {
                            continue;
                        }
                        let (lo, hi) = match op {
                            CmpOp::Lt => (have.lo, have.hi.min(other.hi - 1)),
                            CmpOp::Le => (have.lo, have.hi.min(other.hi)),
                            CmpOp::Gt => (have.lo.max(other.lo + 1), have.hi),
                            CmpOp::Ge => (have.lo.max(other.lo), have.hi),
                            _ => (have.lo.max(other.lo), have.hi.min(other.hi)),
                        };
                        // A branch the bounds prove dead claims nothing.
                        bounds.bind(name, (lo <= hi).then_some(bounds::Interval { lo, hi }));
                    }
                }
                left = right;
            }
        }
        _ => {}
    }
}

/// Whether `stmts` can change the value of `name`.
fn assigned_in(name: &str, stmts: &[cpl_syntax::ast::Stmt]) -> bool {
    use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    struct Writes<'a>(&'a str, bool);
    impl Writes<'_> {
        fn mentions(&self, e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Name(n) => n == self.0,
                ExprKind::Paren(inner) => self.mentions(inner),
                ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().any(|i| self.mentions(i)),
                _ => false,
            }
        }
    }
    impl Visitor for Writes<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign { target, .. } => self.1 |= self.mentions(target),
                StmtKind::Lend { place, .. } => self.1 |= self.mentions(place),
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { args, .. } = &expr.kind {
                self.1 |= args.iter().any(|a| a.mutable && self.mentions(&a.value));
            }
        }
    }
    let mut writes = Writes(name, false);
    stmts.iter().for_each(|s| walk_stmt(s, &mut writes));
    writes.1
}

/// Every name `pat` binds stops being bounded.
fn forget(pat: &cpl_syntax::ast::Pattern, bounds: &mut bounds::Bounds) {
    use cpl_syntax::ast::PatternKind;
    match &pat.kind {
        PatternKind::Name(name) => bounds.bind(name, None),
        PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
            parts.iter().for_each(|p| forget(p, bounds))
        }
        PatternKind::Variant { args, .. } => args.iter().for_each(|p| forget(p, bounds)),
        _ => {}
    }
}
