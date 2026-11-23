//! `peek_then_pop` (L0525): `let x = s.last!` then `s.pop()` reads the top
//! and then throws it away; `let x = s.pop()!` takes it in one step.

use crate::walk::Visitor;
use crate::{File, Finding, PEEK_THEN_POP};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Pops<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Pops<'_> {
        fn stmts(&mut self, stmts: &[Stmt]) {
            for pair in stmts.windows(2) {
                self.pair(&pair[0], &pair[1]);
            }
        }
    }
    impl Pops<'_> {
        fn pair(&mut self, peek: &Stmt, pop: &Stmt) {
            let StmtKind::Let {
                init: Some(init),
                otherwise: None,
                guard: None,
                ..
            } = &peek.kind
            else {
                return;
            };
            let ExprKind::Unwrap(top) = &init.kind else {
                return;
            };
            let ExprKind::Field { base: list, name } = &top.kind else {
                return;
            };
            let StmtKind::Expr(Expr {
                kind: ExprKind::Call { callee, args },
                ..
            }) = &pop.kind
            else {
                return;
            };
            let ExprKind::Field {
                base: popped,
                name: pop_name,
            } = &callee.kind
            else {
                return;
            };
            if (name.text.as_str(), pop_name.text.as_str()) != ("last", "pop") || !args.is_empty() {
                return;
            }
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let place = text(list);
            let path = |s: &str| {
                s.chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            };
            let between = Span::new(peek.span.end, pop.span.start).shift(self.base);
            if place != text(popped) || !path(place) || !between.text(self.src).trim().is_empty() {
                return;
            }
            let head = Span::new(peek.span.start, init.span.start).shift(self.base);
            let take = format!("{place}.pop()!");
            let mut diag = Diagnostic::warning(
                PEEK_THEN_POP.code,
                Span::new(peek.span.start, pop.span.end).shift(self.base),
                format!(
                    "`{place}.last!` then `{place}.pop()`: the pop already gives the last element"
                ),
            )
            .with_label(popped.span.shift(self.base), "the list")
            .with_help(format!("write `{}{take}`", head.text(self.src)));
            diag.fixes.push(Fix {
                title: format!("use `{take}`"),
                edits: vec![
                    (init.span.shift(self.base), take),
                    (
                        Span::new(peek.span.end, pop.span.end).shift(self.base),
                        String::new(),
                    ),
                ],
            });
            self.out.push(Finding {
                lint: &PEEK_THEN_POP,
                diag,
            });
        }
    }
    let mut v = Pops {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
