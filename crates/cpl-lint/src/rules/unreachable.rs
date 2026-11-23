//! `unreachable_code` (L0597): statements after a `return`, `break` or
//! `continue` in the same block never run.

use crate::walk::Visitor;
use crate::{File, Finding, UNREACHABLE_CODE};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Dead {
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Dead {
        fn stmts(&mut self, stmts: &[Stmt]) {
            let Some(at) = stmts.iter().position(|s| {
                matches!(
                    s.kind,
                    StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_)
                )
            }) else {
                return;
            };
            let (Some(next), Some(last)) = (stmts.get(at + 1), stmts.last()) else {
                return;
            };
            let word = match stmts[at].kind {
                StmtKind::Return(_) => "return",
                StmtKind::Break(_) => "break",
                _ => "continue",
            };
            let dead = Span::new(next.span.start, last.span.end).shift(self.base);
            let mut diag = Diagnostic::warning(
                UNREACHABLE_CODE.code,
                dead,
                format!("this never runs: it comes after `{word}` in the same block"),
            )
            .with_label(stmts[at].span.shift(self.base), "the block leaves here")
            .with_help(format!(
                "delete it, or move the `{word}` below it if it was meant to run first"
            ));
            diag.fixes.push(Fix {
                title: "delete the code that never runs".into(),
                edits: vec![(
                    Span::new(stmts[at].span.end, last.span.end).shift(self.base),
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
