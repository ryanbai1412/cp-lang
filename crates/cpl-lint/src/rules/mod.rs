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
    let bounds = bounds::Bounds::of(file);
    let mut out = Vec::new();
    for (base, stmt) in file.top_level() {
        ranges_in(
            base,
            std::slice::from_ref(stmt),
            &mut bounds.clone(),
            &mut out,
        );
    }
    out.sort();
    out.dedup();
    out
}

fn ranges_in(
    base: u32,
    stmts: &[cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
    out: &mut Vec<(u32, i128, i128)>,
) {
    use cpl_syntax::ast::{Cond, ExprKind, FnBody, PatternKind, StmtKind};
    for stmt in stmts {
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
                ranges_in(base, &body.stmts, &mut inner, out);
            }
            StmtKind::While { body, .. } | StmtKind::Unchecked(body) => {
                ranges_in(base, &body.stmts, &mut bounds.clone(), out);
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
                            ranges_in(base, &block.stmts, &mut bounds.clone(), out);
                        }
                    }
                    if let Some(block) = otherwise {
                        ranges_in(base, &block.stmts, &mut bounds.clone(), out);
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
                    ranges_in(base, &block.stmts, &mut inner, out);
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
