//! `manual_mod_reduce` (L0531): `a * b % 1000000007`, C++'s way of working
//! modulo a prime, reduces by hand after each product; `Mod<P>` (`std.mod`)
//! reduces after every operation. Reported once per file, at the first.

use super::sentinel::constant;
use crate::walk::Visitor;
use crate::{File, Finding, MANUAL_MOD_REDUCE};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

/// The smallest modulus treated as a contest prime (`998244353`, `10**9 + 7`).
const PRIME: i128 = 100_000_000;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Reduces {
        base: u32,
        /// The modulus and each `x * y % P`'s span.
        found: Vec<(i128, Span)>,
    }
    impl Visitor for Reduces {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Binary {
                op: BinOp::Rem,
                lhs,
                rhs,
                ..
            } = &expr.kind
            else {
                return;
            };
            let product = matches!(
                &crate::walk::unparen(lhs).kind,
                ExprKind::Binary { op: BinOp::Mul, .. }
            );
            if let Some(p) = constant(rhs).filter(|&p| product && p >= PRIME) {
                self.found.push((p, expr.span.shift(self.base)));
            }
        }
    }
    let mut v = Reduces {
        base: 0,
        found: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let Some(&(p, span)) = v.found.iter().min_by_key(|(_, s)| s.start) else {
        return Vec::new();
    };
    let more = v.found.len() - 1;
    let also = match more {
        0 => String::new(),
        1 => ", and once more in this file".into(),
        n => format!(", and {n} more times in this file"),
    };
    let diag = Diagnostic::warning(
        MANUAL_MOD_REDUCE.code,
        span,
        format!("`% {p}` reduces a product by hand{also}"),
    )
    .with_help(format!(
        "declare `type Z = Mod<{p}>` (`std.mod`) and compute in `Z`: it reduces after every `*`, `+` and `-`, so no `% {p}` can be forgotten"
    ));
    vec![Finding {
        lint: &MANUAL_MOD_REDUCE,
        diag,
    }]
}
