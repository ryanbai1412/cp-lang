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

/// Each `+`, `-` and `*` whose operands the declared bounds prove, with the
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
    for (base, stmt) in file.top_level() {
        ranges_in(
            base,
            std::slice::from_ref(stmt),
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
            if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) {
                if let Some(v) = bounds.expr(expr).filter(|v| v.lo != v.hi) {
                    out.push(crate::ProvenExpr {
                        op: (base + op_span.start, base + op_span.end),
                        expr: (base + expr.span.start, base + expr.span.end),
                        lo: v.lo,
                        hi: v.hi,
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

fn ranges_in(
    base: u32,
    stmts: &[cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
    proven: &mut Proven,
) {
    use cpl_syntax::ast::{Cond, ExprKind, FnBody, PatternKind, StmtKind};
    for stmt in stmts {
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
                    _ => None,
                };
                if let (Some(v), None, false) = (value, ty, mutable) {
                    out.push((base + pat.span.start, v.lo, v.hi));
                }
                forget(pat, bounds);
                if let PatternKind::Name(name) = &pat.kind {
                    bounds.bind(name, value);
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
                ranges_in(base, &body.stmts, &mut inner, proven);
            }
            StmtKind::While { body, .. } | StmtKind::Unchecked(body) => {
                ranges_in(base, &body.stmts, &mut bounds.clone(), proven);
            }
            StmtKind::Expr(expr) => {
                if let ExprKind::If {
                    branches,
                    otherwise,
                } = &expr.kind
                {
                    // An `if let` binds names the branch would see through.
                    for (cond, block) in branches {
                        if matches!(cond, Cond::Expr(_)) {
                            ranges_in(base, &block.stmts, &mut bounds.clone(), proven);
                        }
                    }
                    if let Some(block) = otherwise {
                        ranges_in(base, &block.stmts, &mut bounds.clone(), proven);
                    }
                }
            }
            StmtKind::Fn(decl) => {
                let mut inner = bounds.clone();
                for param in &decl.params {
                    inner.bind(
                        &param.name.text,
                        param.ty.as_ref().and_then(|t| inner.range(t)),
                    );
                }
                if let FnBody::Block(block) = &decl.body {
                    ranges_in(base, &block.stmts, &mut inner, proven);
                }
            }
            _ => {}
        }
    }
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
