//! `sentinel_table` (L0530): a `var` table or list filled with `-1` or a
//! value of size at least `10**9`, whose entries the file compares with that
//! value (or, for a large one, orders as a relaxation does). C++ marks
//! "absent" with a value; `[D]?Int` says it in the type.

use crate::facts::Facts;
use crate::walk::Visitor;
use crate::{File, Finding, SENTINEL_TABLE};
use cpl_check::Ty;
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{BinOp, CmpOp, Expr, ExprKind, PatternKind, Stmt, StmtKind, UnOp};

pub(crate) const LARGE: i128 = 1_000_000_000;

struct Table {
    name: String,
    fill: i128,
    span: Span,
    ty: String,
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Sentinels<'a> {
        src: &'a str,
        base: u32,
        facts: Option<&'a Facts>,
        tables: Vec<Table>,
        /// Name, the compared constant if any, and the comparison's span.
        tests: Vec<(String, Option<i128>, bool, Span)>,
    }
    impl Visitor for Sentinels<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            let StmtKind::Let {
                mutable: true,
                pat,
                ty,
                init: Some(init),
                ..
            } = &stmt.kind
            else {
                return;
            };
            let PatternKind::Name(name) = &pat.kind else {
                return;
            };
            // A written type, as the checker reads it; `Some(None)` when it
            // has no answer.
            let written = ty.as_ref().map(|_| {
                self.facts
                    .and_then(|f| f.place_type(pat.span.shift(self.base)))
            });
            let fill = match (written, &init.kind) {
                (Some(Some(Ty::Array { elem, .. })), _) if is_int(elem) => {
                    constant(init).map(|v| (v, init.span))
                }
                (
                    Some(Some(Ty::Named { name, args })),
                    ExprKind::Comprehension { body, clauses },
                ) if name == "List"
                    && matches!(&args[..], [a] if is_int(a))
                    && clauses.len() == 1 =>
                {
                    constant(body).map(|v| (v, body.span))
                }
                (None, ExprKind::Comprehension { body, clauses }) if clauses.len() == 1 => {
                    constant(body).map(|v| (v, body.span))
                }
                _ => None,
            };
            let Some((fill, fill_span)) = fill.filter(|&(v, _)| v == -1 || v.abs() >= LARGE) else {
                return;
            };
            let ty = match ty {
                Some(t) => {
                    let text = t.span.shift(self.base).text(self.src);
                    let at = text.rfind("Int").unwrap_or(text.len());
                    format!("{}?{}", &text[..at], &text[at..])
                }
                None => "List<?Int>".into(),
            };
            self.tables.push(Table {
                name: name.clone(),
                fill,
                span: fill_span.shift(self.base),
                ty,
            });
        }
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Compare { first, rest } = &expr.kind else {
                return;
            };
            let mut lhs = &**first;
            for (op, rhs) in rest {
                let ordered = matches!(op, CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge);
                if ordered || matches!(op, CmpOp::Eq | CmpOp::Ne) {
                    for (entry, other) in [(lhs, rhs), (rhs, lhs)] {
                        if let ExprKind::Index { base, .. } = &entry.kind {
                            if let ExprKind::Name(name) = &base.kind {
                                let span = expr.span.shift(self.base);
                                self.tests
                                    .push((name.clone(), constant(other), ordered, span));
                            }
                        }
                    }
                }
                lhs = rhs;
            }
        }
    }
    let mut v = Sentinels {
        src: file.src,
        base: 0,
        facts: file.facts,
        tables: Vec::new(),
        tests: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let mut out = Vec::new();
    for t in &v.tables {
        let Some((_, _, _, at)) = v.tests.iter().find(|(name, c, ordered, _)| {
            *name == t.name
                && (*c == Some(t.fill) || (*ordered && c.is_none() && t.fill.abs() >= LARGE))
        }) else {
            continue;
        };
        let shown = t.span.text(file.src);
        let diag = Diagnostic::warning(
            SENTINEL_TABLE.code,
            t.span,
            format!("`{}` is filled with `{shown}` to mean \"not set\"", t.name),
        )
        .with_label(*at, "the sentinel is tested here")
        .with_help(format!(
            "declare it `{}` filled with `None`, so an unset entry can't be read as a number; read one with `if let`, `!` or `??`",
            t.ty
        ));
        out.push(Finding {
            lint: &SENTINEL_TABLE,
            diag,
        });
    }
    out
}

fn is_int(ty: &Ty) -> bool {
    matches!(ty, Ty::Int)
}

/// `-1`, `10**18`, `10**9 + 7`, `1000000000000000000` and their negations.
pub(crate) fn constant(e: &Expr) -> Option<i128> {
    match &e.kind {
        ExprKind::Int(v) => Some(*v),
        ExprKind::Paren(inner) => constant(inner),
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => constant(expr).map(|v| -v),
        ExprKind::Binary {
            op: BinOp::Pow,
            lhs,
            rhs,
            ..
        } => constant(lhs)?.checked_pow(u32::try_from(constant(rhs)?).ok()?),
        ExprKind::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        } => constant(lhs)?.checked_add(constant(rhs)?),
        _ => None,
    }
}
