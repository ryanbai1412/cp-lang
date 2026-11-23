//! `fermat_inverse` (L0573): `pow_mod(a, P - 2, P)`, the modular inverse by
//! Fermat's little theorem; `Mod<P>` (`std.mod`) has it as `Z(a).inv()`.
//! Only a constant modulus counts: `Mod<P>` needs one.

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, FERMAT_INVERSE};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

use super::shapes::{int, name};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Inverses<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Inverses<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Call { callee, args } = &expr.kind else {
                return;
            };
            let ([a, e, m], Some("pow_mod")) = (args.as_slice(), name(callee)) else {
                return;
            };
            let text = |x: &Expr| unparen(x).span.shift(self.base).text(self.src).to_string();
            let ExprKind::Binary {
                op: BinOp::Sub,
                lhs,
                rhs,
                ..
            } = &unparen(&e.value).kind
            else {
                return;
            };
            if int(rhs) != Some(2) || text(lhs) != text(&m.value) || !constant(&m.value) {
                return;
            }
            let (a, p) = (text(&a.value), text(&m.value));
            let diag = Diagnostic::warning(
                FERMAT_INVERSE.code,
                expr.span.shift(self.base),
                format!("`pow_mod({a}, {p} - 2, {p})` is the inverse of `{a}` modulo `{p}`, by hand"),
            )
            .with_help(format!(
                "declare `type Z = Mod<{p}>` (`std.mod`) and write `Z({a}).inv()`: it is this inverse, traps when there is none, and `Z`'s `*`, `+` and `-` reduce on their own"
            ));
            self.out.push(Finding {
                lint: &FERMAT_INVERSE,
                diag,
            });
        }
    }
    let mut v = Inverses {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// A modulus `Mod<P>` can take: a literal or an uppercase `const` name, not
/// a modulus known only at run time.
fn constant(m: &Expr) -> bool {
    int(m).is_some()
        || name(m).is_some_and(|n| {
            n.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        })
}
