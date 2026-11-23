//! `sentinel_scalar` (L0536): a scalar `var` started at `-1` or at a value of
//! size at least `10**9` to mean "nothing yet", then set inside a `for` loop
//! (a found index, or a running best). `?Int` starting at `none` says it in
//! the type. Tables are `sentinel_table`'s.

use super::sentinel::{constant, LARGE};
use crate::walk::{walk_stmt, Visitor};
use crate::{File, Finding, SENTINEL_SCALAR};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{AssignOp, ExprKind, PatternKind, Stmt, StmtKind, TypeKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Scalars {
        base: u32,
        /// Name, start value, and the start's span.
        vars: Vec<(String, i128, Span)>,
        /// Names set inside a `for` body, and whether the value was a bare name.
        set_in_for: Vec<(String, bool, Span)>,
    }
    impl Visitor for Scalars {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Let {
                    mutable: true,
                    pat,
                    ty,
                    init: Some(init),
                    ..
                } => {
                    let PatternKind::Name(name) = &pat.kind else {
                        return;
                    };
                    let int = ty.as_ref().is_none_or(|t| {
                        matches!(&t.kind, TypeKind::Named { name, args } if name.text == "Int" && args.is_empty())
                    });
                    if let Some(v) =
                        constant(init).filter(|&v| int && (v == -1 || v.abs() >= LARGE))
                    {
                        self.vars
                            .push((name.clone(), v, init.span.shift(self.base)));
                    }
                }
                StmtKind::For { body, .. } => {
                    struct Sets<'a>(&'a mut Vec<(String, bool, Span)>, u32);
                    impl Visitor for Sets<'_> {
                        fn stmt(&mut self, stmt: &Stmt) {
                            if let StmtKind::Assign { target, op, value } = &stmt.kind {
                                if let ExprKind::Name(name) = &target.kind {
                                    let bare = matches!(op, AssignOp::Set)
                                        && matches!(&value.kind, ExprKind::Name(_));
                                    self.0.push((name.clone(), bare, stmt.span.shift(self.1)));
                                }
                            }
                        }
                    }
                    for s in &body.stmts {
                        walk_stmt(&mut Sets(&mut self.set_in_for, self.base), s);
                    }
                }
                _ => {}
            }
        }
    }
    let mut v = Scalars {
        base: 0,
        vars: Vec::new(),
        set_in_for: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let mut out = Vec::new();
    for (name, start, span) in &v.vars {
        let index = *start == -1;
        let Some((_, _, set)) = v
            .set_in_for
            .iter()
            .find(|(n, bare, _)| n == name && (*bare || !index))
        else {
            continue;
        };
        let shown = span.text(file.src);
        let help = if index {
            format!("declare `var {name}: ?Int = none` and read it with `?? -1`, `if let` or `!`: `-1` passes for a real index, `none` cannot")
        } else {
            format!("declare `var {name}: ?Int = none`, or reduce with `.max()` / `.min()` (`std.seq`): `{shown}` guesses a bound the data may pass, `none` is no number at all")
        };
        let diag = Diagnostic::warning(
            SENTINEL_SCALAR.code,
            *span,
            format!("`{name}` starts at `{shown}` to mean \"nothing yet\""),
        )
        .with_label(*set, "set here")
        .with_help(help);
        out.push(Finding {
            lint: &SENTINEL_SCALAR,
            diag,
        });
    }
    out
}
