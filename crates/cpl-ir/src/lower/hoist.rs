//! Items that run before the statements above them (R11.1.1).

use super::Lowerer;
use crate::Stmt;
use cpl_syntax::ast;

impl Lowerer<'_> {
    /// Defines each top-level `domain` whose bounds are compile-time
    /// integers (`domain V = 0..5`, `domain E = [N]`) before any other
    /// statement, so a use above it reads its size. A runtime domain runs
    /// where it is written. An integer `const` they may name is defined
    /// first. Returns the indices of the statements defined.
    pub(super) fn hoist_constant_domains(
        &mut self,
        items: &[(u32, &ast::Stmt)],
        out: &mut Vec<Stmt>,
    ) -> Vec<usize> {
        let mut hoisted = Vec::new();
        for (idx, &(offset, stmt)) in items.iter().enumerate() {
            let def = match &stmt.kind {
                ast::StmtKind::Domain { def, .. } => def,
                ast::StmtKind::Const {
                    generics,
                    init: Some(init),
                    ..
                } if generics.is_empty() && self.eval_const_int(init).is_some() => {
                    self.offset = offset;
                    self.stmt(stmt, out);
                    hoisted.push(idx);
                    continue;
                }
                _ => continue,
            };
            let constant = match &def.kind {
                ast::ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    step: None,
                    ..
                } => self.eval_const_int(lo).is_some() && self.eval_const_int(hi).is_some(),
                ast::ExprKind::List(items) => {
                    matches!(items.as_slice(), [n] if self.eval_const_int(n).is_some())
                }
                _ => false,
            };
            if constant {
                self.offset = offset;
                self.stmt(stmt, out);
                hoisted.push(idx);
            }
        }
        hoisted
    }
}
