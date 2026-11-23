//! `pop_min_loop` (L0576): `let x = s.first!` then `s.remove(x)` is
//! `let x = s.pop_first()!` (`std.bag`). The set is labelled so the fact
//! table keeps the finding only on a `SortedBag` or `SortedSet` (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, POP_MIN_LOOP};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind};

use super::shapes::name;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Ends<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Ends<'_> {
        fn stmts(&mut self, stmts: &[Stmt]) {
            for pair in stmts.windows(2) {
                self.pair(&pair[0], &pair[1]);
            }
        }
    }
    impl Ends<'_> {
        fn pair(&mut self, read: &Stmt, remove: &Stmt) {
            let StmtKind::Let {
                pat: pattern,
                init: Some(init),
                otherwise: None,
                guard: None,
                ..
            } = &read.kind
            else {
                return;
            };
            let Some(bound) = binding(pattern) else {
                return;
            };
            let ExprKind::Unwrap(end) = &init.kind else {
                return;
            };
            let ExprKind::Field {
                base: set,
                name: end,
            } = &unparen(end).kind
            else {
                return;
            };
            let pop = match end.text.as_str() {
                "first" => "pop_first",
                "last" => "pop_last",
                _ => return,
            };
            let StmtKind::Expr(Expr {
                kind: ExprKind::Call { callee, args },
                ..
            }) = &remove.kind
            else {
                return;
            };
            let ExprKind::Field {
                base: removed,
                name: method,
            } = &callee.kind
            else {
                return;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let between = Span::new(read.span.end, remove.span.start).shift(self.base);
            let same =
                matches!(&args[..], [a] if a.name.is_none() && name(&a.value) == Some(bound));
            if method.text != "remove"
                || !same
                || text(set) != text(removed)
                || !between.text(self.src).trim().is_empty()
            {
                return;
            }
            let take = format!("{}.{pop}()!", text(set));
            let mut diag = Diagnostic::warning(
                POP_MIN_LOOP.code,
                Span::new(read.span.start, remove.span.end).shift(self.base),
                format!("`{}.{}!` then `{}.remove({bound})` is `{take}`", text(set), end.text, text(set)),
            )
            .with_label(set.span.shift(self.base), "the set")
            .with_help(format!(
                "write `let {bound} = {take}` (`std.bag`): it reads and removes the end in one call instead of finding it twice"
            ));
            diag.fixes.push(Fix {
                title: format!("use `{take}`"),
                edits: vec![
                    (init.span.shift(self.base), take),
                    (
                        Span::new(read.span.end, remove.span.end).shift(self.base),
                        String::new(),
                    ),
                ],
            });
            self.out.push(Finding {
                lint: &POP_MIN_LOOP,
                diag,
            });
        }
    }
    let mut v = Ends {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// The name a `let` binds, when it binds one plain name.
fn binding(p: &Pattern) -> Option<&str> {
    match &p.kind {
        PatternKind::Name(n) => Some(n.as_str()),
        _ => None,
    }
}
