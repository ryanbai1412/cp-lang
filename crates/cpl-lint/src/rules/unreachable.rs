//! `unreachable_code` (L0597): statements after a `return`, `break` or
//! `continue` in the same block never run.

use crate::walk::Visitor;
use crate::{File, Finding, UNREACHABLE_CODE};
use cpl_diag::msgs::lint_rules;
use cpl_diag::{Fix, Span};
use cpl_syntax::ast::{Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Dead {
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Dead {
        fn stmts(&mut self, stmts: &[(u32, &Stmt)]) {
            let Some(at) = stmts.iter().position(|(_, s)| {
                matches!(
                    s.kind,
                    StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_)
                )
            }) else {
                return;
            };
            let (Some(&(nb, next)), Some(&(lb, last))) = (stmts.get(at + 1), stmts.last()) else {
                return;
            };
            let (ab, leave) = stmts[at];
            let word = match leave.kind {
                StmtKind::Return(_) => "return",
                StmtKind::Break(_) => "break",
                _ => "continue",
            };
            let base = self.base;
            let dead = Span::new(
                next.span.shift(base + nb).start,
                last.span.shift(base + lb).end,
            );
            let mut diag = lint_rules::UNREACHABLE_CODE
                .warning(dead, &[&word])
                .with_label(leave.span.shift(base + ab), "the block leaves here")
                .with_help(format!(
                    "delete it, or move the `{word}` below it if it was meant to run first"
                ));
            diag.fixes.push(Fix {
                title: "delete the code that never runs".into(),
                edits: vec![(
                    Span::new(
                        leave.span.shift(base + ab).end,
                        last.span.shift(base + lb).end,
                    ),
                    String::new(),
                )],
            });
            self.out.push(Finding {
                lint: &UNREACHABLE_CODE,
                diag,
            });
        }
    }
    let mut v = Dead {
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
