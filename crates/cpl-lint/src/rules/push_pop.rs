//! `push_pop_pair` (L0575): `h.push(x)` then `let y = h.pop()!` is
//! `let y = h.push_pop(x)` (`std.heap`), one sift. The heap is labelled so
//! the fact table keeps the finding only on a `Heap` or `MaxHeap`
//! (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, PUSH_POP_PAIR};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Pairs<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Pairs<'_> {
        fn stmts(&mut self, stmts: &[Stmt]) {
            for pair in stmts.windows(2) {
                self.pair(&pair[0], &pair[1]);
            }
        }
    }
    impl Pairs<'_> {
        fn pair(&mut self, push: &Stmt, pop: &Stmt) {
            let StmtKind::Expr(call) = &push.kind else {
                return;
            };
            let Some((heap, "push", [x])) = method(call) else {
                return;
            };
            let StmtKind::Let {
                init: Some(init),
                otherwise: None,
                guard: None,
                ..
            } = &pop.kind
            else {
                return;
            };
            let ExprKind::Unwrap(popped) = &init.kind else {
                return;
            };
            let Some((popped, "pop", [])) = method(unparen(popped)) else {
                return;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let between = Span::new(push.span.end, pop.span.start).shift(self.base);
            if text(heap) != text(popped) || !between.text(self.src).trim().is_empty() {
                return;
            }
            let call = format!("{}.push_pop({})", text(heap), text(&x.value));
            let mut diag = Diagnostic::warning(
                PUSH_POP_PAIR.code,
                Span::new(push.span.start, pop.span.end).shift(self.base),
                format!("a push and then a pop on `{}` is `{call}`", text(heap)),
            )
            .with_label(heap.span.shift(self.base), "the heap")
            .with_help(format!(
                "write `{call}` (`std.heap`): one sift instead of two, it returns the smaller of `{}` and the top, and needs no `!` because a heap is never empty right after a push",
                text(&x.value)
            ));
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits: vec![
                    (
                        Span::new(push.span.start, pop.span.start).shift(self.base),
                        String::new(),
                    ),
                    (init.span.shift(self.base), call),
                ],
            });
            self.out.push(Finding {
                lint: &PUSH_POP_PAIR,
                diag,
            });
        }
    }
    let mut v = Pairs {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// `base`, the method name and the arguments of `base.name(args)`.
fn method(e: &Expr) -> Option<(&Expr, &str, &[cpl_syntax::ast::Arg])> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name } = &callee.kind else {
        return None;
    };
    Some((base, name.text.as_str(), args))
}
