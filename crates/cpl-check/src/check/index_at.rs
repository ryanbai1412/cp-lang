//! `s[k]` on a struct with an `at` method reads as `s.at(k)` (R12.4.13).

use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind};

impl Checker<'_> {
    /// R12.4.13: `s[k]` on a struct with an `at` method is `s.at(k)`.
    pub(super) fn index_by_at(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> Option<Ty> {
        if matches!(index.kind, ExprKind::Range { .. }) {
            return None;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        let Ty::Named { name, .. } = &base_ty else {
            return None;
        };
        let name = name.clone();
        let def = self.type_def(&name)?;
        let Def::Struct { methods, .. } = &*def else {
            return None;
        };
        // A struct's own `get` is its `[]`, as in lowering (`Compressed`).
        if !methods.iter().any(|(m, _)| m == "at") || methods.iter().any(|(m, _)| m == "get") {
            return None;
        }
        // A position given for a cut is its right cut (R07.1.6), as plain indexing reads it.
        let takes_cut = methods.iter().any(|(m, ty)| {
            m == "at"
                && matches!(ty, Ty::Fn { params, .. } if params.last().is_some_and(|p|
                    matches!(&p.ty, Ty::Named { name, .. } if name == "Cut")))
        });
        if takes_cut && matches!(self.quietly(|c| c.infer(index)), Ty::Pos(_)) {
            return None;
        }
        let callee = Expr {
            kind: ExprKind::Field {
                base: Box::new(base.clone()),
                name: ast::Name {
                    text: "at".to_string(),
                    span: index.span,
                },
            },
            span: expr.span,
        };
        let args = vec![ast::Arg {
            name: None,
            mutable: false,
            value: index.clone(),
        }];
        let call = Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            span: expr.span,
        };
        Some(self.infer(&call))
    }
}
