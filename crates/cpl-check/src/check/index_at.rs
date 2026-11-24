//! `s[k]` on a struct with an `at` method reads as `s.at(k)` (R12.4.13); beside a `get`,
//! only as a place written through a `lends mut` `at` (R05.6.3).

use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind};

impl Checker<'_> {
    /// Marks each `s[k]` on the path of `e` (`a[i].kids[j]`) as written through, giving
    /// the ones it newly marked.
    pub(super) fn lend_path(&mut self, mut e: &Expr) -> Vec<Span> {
        let mut out = Vec::new();
        loop {
            match &e.kind {
                ExprKind::Index { base, .. } => {
                    if self.lent_places.insert(e.span) {
                        out.push(e.span);
                    }
                    e = base;
                }
                ExprKind::Field { base, .. } => e = base,
                ExprKind::Paren(inner) => e = inner,
                _ => return out,
            }
        }
    }

    /// Whether calling `name` on a `base_ty` changes it: a `mut self` method.
    pub(super) fn changes_self(&mut self, base_ty: &Ty, name: &str) -> bool {
        let members = match base_ty {
            Ty::Named { name, .. } => self.members_of(name),
            _ => None,
        };
        match &members {
            Some(m) if m.members.iter().any(|(n, _)| n == name) => {
                m.mutators.iter().any(|m| m == name)
            }
            _ => self.takes_mut_self(base_ty, name),
        }
    }

    /// `a[i].c.bump(1)`, `inc(mut a[i].c)`: a `mut self` call's receiver and a `mut`
    /// argument are places, so each `s[k]` on their paths is written through (R05.6.3).
    pub(super) fn lent_receiver(&mut self, callee: &Expr, args: &[ast::Arg]) {
        for arg in args.iter().filter(|a| a.mutable) {
            self.lend_path(&arg.value);
        }
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        let path = self.lend_path(base);
        if path.is_empty() {
            return;
        }
        // `p[0].kids.swap(0, 1)`: a built-in row that changes its receiver is one too.
        let base_ty = self.quietly(|c| c.infer(base));
        if !self.call_mutator(&base_ty, &name.text, args) {
            for s in path {
                self.lent_places.remove(&s);
            }
        }
    }

    /// R12.4.13: `s[k]` on a struct with an `at` method is `s.at(k)`; beside a `get`,
    /// a read is `s.get(k)` and only a place a `lends mut` `at` lends is `s.at(k)`
    /// (R05.6.3), as in lowering.
    pub(super) fn index_by_at(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> Option<Ty> {
        if matches!(index.kind, ExprKind::Range { .. }) {
            return None;
        }
        let base_ty = self.infer_quietly(base);
        let Ty::Named { name, .. } = &base_ty else {
            return None;
        };
        let name = name.clone();
        let def = self.type_def(&name)?;
        let Def::Struct {
            methods,
            projections,
            ..
        } = &*def
        else {
            return None;
        };
        let has = |m: &str| methods.iter().any(|(n, _)| n == m);
        let lent = self.lent_places.contains(&expr.span)
            && (projections.iter()).any(|p| p.name == "at" && p.mutable);
        // A struct's own `get` is its `[]` when read, as in lowering (`Compressed`).
        let method = match (has("get"), has("at")) {
            (true, true) if lent => "at",
            (true, _) => "get",
            (false, true) => "at",
            (false, false) => return None,
        };
        // A position given for a cut is its right cut (R07.1.6), as plain indexing reads it.
        let takes_cut = methods.iter().any(|(m, ty)| {
            m == method
                && matches!(ty, Ty::Fn { params, .. } if params.last().is_some_and(|p|
                    matches!(&p.ty, Ty::Named { name, .. } if name == "Cut")))
        });
        if takes_cut && matches!(self.infer_quietly(index), Ty::Pos(_)) {
            return None;
        }
        let callee = Expr {
            kind: ExprKind::Field {
                base: Box::new(base.clone()),
                name: ast::Name {
                    text: method.to_string(),
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
        let (hovers, resolved) = (self.hovers.len(), self.resolved.len());
        let ty = self.infer(&call);
        // The method the call names is not written: the index in its place
        // keeps its own hover and refs.
        let mut i = 0;
        self.hovers.retain(|h| {
            i += 1;
            i <= hovers || !(h.span == index.span && h.keyword == "field" && h.name == method)
        });
        let mut i = 0;
        self.resolved.retain(|(span, r)| {
            i += 1;
            i <= resolved || !(*span == index.span && r.name == method && r.owner.is_some())
        });
        self.exprs.accessors.push((expr.span, method == "at"));
        Some(ty)
    }
}
