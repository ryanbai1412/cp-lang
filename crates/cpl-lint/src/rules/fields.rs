//! `unread_field`: a struct field that is set and never looked at.
//!
//! A field is read when `.name` appears anywhere other than as the target
//! of a plain `=`. Reads are matched by name, not by type, so a field
//! sharing its name with one that is read is never reported. A file with
//! no statements of its own is a library, whose fields are read by the
//! programs that use it, and is skipped.

use crate::walk::Visitor;
use crate::{File, Finding, UNREAD_FIELD};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{AssignOp, Expr, ExprKind, Stmt, StmtKind};
use std::collections::HashSet;

struct Field {
    owner: String,
    name: String,
    span: Span,
}

#[derive(Default)]
struct Fields {
    base: u32,
    declared: Vec<Field>,
    read: HashSet<String>,
    /// Field accesses that are assignment targets: writes, not reads.
    /// Keyed by position, since two items can hold equal spans.
    written: HashSet<(u32, Span)>,
}

impl Visitor for Fields {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Struct(decl) => {
                for field in &decl.fields {
                    self.declared.push(Field {
                        owner: decl.name.text.clone(),
                        name: field.name.text.clone(),
                        span: field.name.span.shift(self.base),
                    });
                }
            }
            StmtKind::Assign {
                target,
                op: AssignOp::Set,
                ..
            } => {
                if matches!(target.kind, ExprKind::Field { .. }) {
                    self.written.insert((self.base, target.span));
                }
            }
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Field { name, .. } = &expr.kind {
            if !self.written.contains(&(self.base, expr.span)) {
                self.read.insert(name.text.clone());
            }
        }
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let is_program = file.top_level().any(|(_, stmt)| {
        !matches!(
            stmt.kind,
            StmtKind::Fn(_)
                | StmtKind::Struct(_)
                | StmtKind::Enum(_)
                | StmtKind::Domain { .. }
                | StmtKind::TypeAlias { .. }
                | StmtKind::Trait { .. }
                | StmtKind::Impl { .. }
                | StmtKind::Use { .. }
                | StmtKind::Const { .. }
        )
    });
    if !is_program {
        return Vec::new();
    }
    let mut fields = Fields::default();
    file.walk(&mut fields, |fields, base| fields.base = base);
    fields
        .declared
        .iter()
        .filter(|field| !fields.read.contains(&field.name))
        .map(|field| {
            let diag = Diagnostic::warning(
                UNREAD_FIELD.code,
                field.span,
                format!("field `{}` of `{}` is never read", field.name, field.owner),
            )
            .with_help(format!(
                "remove it, or add `@allow(unread_field)` to `{}` if it is only read as part of the whole value (printed, compared, sorted)",
                field.owner
            ));
            Finding {
                lint: &UNREAD_FIELD,
                diag,
            }
        })
        .collect()
}
