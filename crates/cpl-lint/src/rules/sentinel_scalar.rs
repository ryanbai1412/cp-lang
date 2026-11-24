//! `sentinel_scalar` (L0536): a scalar `var` started at `-1` or at a value of
//! size at least `10**9` to mean "nothing yet", then set inside a `for` loop
//! (a found index, or a running best). `?Int` starting at `None` says it in
//! the type. Tables are `sentinel_table`'s.

use super::sentinel::{constant, LARGE};
use crate::walk::{walk_stmt, Visitor};
use crate::{File, Finding, SENTINEL_SCALAR};
use cpl_check::Ty;
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{AssignOp, ExprKind, PatternKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Scalars {
        base: u32,
        /// Name, start value, the start's span, and the name's span when
        /// a type is written (the checker's facts say whether it is `Int`).
        vars: Vec<(String, i128, Span, Option<Span>)>,
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
                    if let Some(v) = constant(init).filter(|&v| v == -1 || v.abs() >= LARGE) {
                        let written = ty.as_ref().map(|_| pat.span.shift(self.base));
                        self.vars
                            .push((name.clone(), v, init.span.shift(self.base), written));
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
    for (name, start, span, written) in &v.vars {
        let int = written.is_none_or(|at| {
            file.facts
                .is_some_and(|f| matches!(f.place_type(at), Some(Ty::Int)))
        });
        if !int {
            continue;
        }
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
            format!("declare `var {name}: ?Int = None` and read it with `?? -1`, `if let` or `!`: `-1` passes for a real index, `None` cannot")
        } else {
            format!("declare `var {name}: ?Int = None`, or reduce with `.max()` / `.min()` (`std.seq`): `{shown}` guesses a bound the data may pass, `None` is no number at all")
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
