//! `char_offset_index` (L0574): `cnt[c - 'a']` turns letters into `Int`
//! offsets; a `domain Letter = 'a'..='z'` keeps them an index space.

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, CHAR_OFFSET_INDEX};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Offsets<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Offsets<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Index { indices, .. } = &expr.kind else {
                return;
            };
            let [index] = &indices[..] else { return };
            let ExprKind::Binary {
                op: BinOp::Sub,
                lhs,
                rhs,
                ..
            } = &unparen(index).kind
            else {
                return;
            };
            let (first, last) = match unparen(rhs).kind {
                ExprKind::Char('a') => ('a', 'z'),
                ExprKind::Char('A') => ('A', 'Z'),
                _ => return,
            };
            let c = lhs.span.shift(self.base).text(self.src);
            let diag = Diagnostic::warning(
                CHAR_OFFSET_INDEX.code,
                index.span.shift(self.base),
                format!("`{c} - '{first}'` turns a letter into an `Int` index"),
            )
            .with_help(format!(
                "declare `domain Letter = '{first}'..='{last}'`, make the array `[Letter]T` and index it with `Letter({c})`: one cell per letter, and any other character traps where it enters"
            ));
            self.out.push(Finding {
                lint: &CHAR_OFFSET_INDEX,
                diag,
            });
        }
    }
    let mut v = Offsets {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
