//! `float_equality`: `==` and `!=` on floats compare every bit, and two
//! ways of computing the same number rarely agree to the last one.
//!
//! An operand is a float when the checker types it `F64`: a float literal,
//! arithmetic on a float, or a name, field, call or index of type `F64`.

use crate::walk::Visitor;
use crate::{Facts, File, Finding, FLOAT_EQUALITY};
use cpl_check::Ty;
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{CmpOp, Expr, ExprKind, UnOp};

struct Report<'a> {
    src: &'a str,
    base: u32,
    facts: &'a Facts,
    findings: Vec<Finding>,
}

impl Report<'_> {
    fn is_float(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Float(_) => true,
            ExprKind::Paren(inner)
            | ExprKind::Unary {
                op: UnOp::Neg,
                expr: inner,
            } => self.is_float(inner),
            ExprKind::Binary { lhs, rhs, .. } => self.is_float(lhs) || self.is_float(rhs),
            _ => self.facts.place_type(e.span.shift(self.base)) == Some(&Ty::Float),
        }
    }
}

impl Visitor for Report<'_> {
    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Compare { first, rest } = &expr.kind else {
            return;
        };
        let mut lhs: &Expr = first;
        for (op, rhs) in rest {
            let float = [lhs, rhs].into_iter().find(|e| self.is_float(e));
            if let (true, Some(float)) = (matches!(op, CmpOp::Eq | CmpOp::Ne), float) {
                let text = |e: &Expr| e.span.shift(self.base).text(self.src);
                let (a, b) = (text(lhs), text(rhs));
                let (symbol, instead) = match op {
                    CmpOp::Eq => ("==", format!("abs({a} - {b}) < EPS")),
                    _ => ("!=", format!("abs({a} - {b}) >= EPS")),
                };
                let diag = Diagnostic::warning(
                    FLOAT_EQUALITY.code,
                    lhs.span.to(rhs.span).shift(self.base),
                    format!("`{symbol}` compares floats bit for bit"),
                )
                // The operand read as a float, for the checker to confirm: a
                // field name a float struct also uses can be an `Int` here.
                .with_label(float.span.shift(self.base), "a float")
                .with_help(format!(
                    "compare within a tolerance: `{instead}`, with `EPS` chosen for the answer's precision"
                ));
                self.findings.push(Finding {
                    lint: &FLOAT_EQUALITY,
                    diag,
                });
            }
            lhs = rhs;
        }
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let Some(facts) = file.facts else {
        return Vec::new();
    };
    let mut report = Report {
        src: file.src,
        base: 0,
        facts,
        findings: Vec::new(),
    };
    file.walk(&mut report, |report, base| report.base = base);
    report.findings
}
