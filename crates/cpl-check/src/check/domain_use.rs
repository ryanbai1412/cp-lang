//! Type parameters a fn uses as domains (R04.3.3): `[R]T` or `Range<R>` in
//! its signature, `R.at(i)` or `for v in R` in its body. Inside the body such
//! a parameter holds `R: Domain`, so `p + 1` on a position is an `Int` with
//! no operator bound, as on a declared domain.

use super::{Checker, Ty};
use cpl_syntax::ast::{self, ExprKind, FnBody, StmtKind, Type, TypeKind};
use cpl_syntax::visit::{walk_block, walk_expr, Visitor};

/// The members only a domain has, read off a type parameter as `R.at(i)`.
const MEMBERS: [&str; 6] = ["at", "count", "cuts", "nth", "wrap", "ranges"];

/// `f`'s declared bounds, plus `Domain` on each type parameter it uses as one.
pub(crate) fn body_bounds(f: &ast::FnDecl) -> Vec<(String, Vec<String>)> {
    let mut uses = Uses {
        generics: f.generics.iter().map(|g| g.text.as_str()).collect(),
        found: Vec::new(),
    };
    for ty in f.params.iter().filter_map(|p| p.ty.as_ref()).chain(&f.ret) {
        uses.ty(ty, false);
    }
    match &f.body {
        FnBody::Block(block) => walk_block(block, &mut uses),
        FnBody::Expr(expr) => walk_expr(expr, &mut uses),
        FnBody::None => {}
    }
    let mut bounds = super::types::extract_bounds(&f.bounds);
    let found = uses.found;
    bounds.extend(found.into_iter().map(|g| (g, vec!["Domain".to_string()])));
    bounds
}

struct Uses<'f> {
    generics: Vec<&'f str>,
    found: Vec<String>,
}

impl Uses<'_> {
    fn note(&mut self, name: &str) {
        if self.generics.contains(&name) && !self.found.iter().any(|g| g == name) {
            self.found.push(name.to_string());
        }
    }

    /// `index` is true for an array dimension or a `Range` argument.
    fn ty(&mut self, ty: &Type, index: bool) {
        match &ty.kind {
            TypeKind::Named { name, args } => {
                if index && args.is_empty() {
                    self.note(&name.text);
                }
                for arg in args {
                    self.ty(arg, name.text == "Range");
                }
            }
            TypeKind::Array { index, elem } => {
                for i in index {
                    self.ty(i, true);
                }
                self.ty(elem, false);
            }
            TypeKind::OwnDomain { elem: t, .. }
            | TypeKind::Optional(t)
            | TypeKind::Mut(t)
            | TypeKind::Yields(t) => self.ty(t, false),
            TypeKind::Tuple(ts) => ts.iter().for_each(|t| self.ty(t, false)),
            TypeKind::Record(fs) | TypeKind::Row(fs) => {
                fs.iter().for_each(|(_, t)| self.ty(t, false));
            }
            TypeKind::Fn { params, ret } => {
                params.iter().for_each(|t| self.ty(t, false));
                self.ty(ret, false);
            }
            _ => {}
        }
    }
}

impl Visitor for Uses<'_> {
    fn stmt(&mut self, stmt: &ast::Stmt) {
        if let StmtKind::For { iter, .. } = &stmt.kind {
            if let ExprKind::Name(n) = &iter.kind {
                self.note(n);
            }
        }
    }

    fn expr(&mut self, expr: &ast::Expr) {
        if let ExprKind::Field { base, name } = &expr.kind {
            if let ExprKind::Name(n) = &base.kind {
                if MEMBERS.contains(&name.text.as_str()) {
                    self.note(n);
                }
            }
        }
    }
}

impl Checker<'_> {
    /// The branch `body` of `cond`, checked with the bounds `cond` tests:
    /// inside `if T: Add:` every `T` is an `Add`.
    pub(super) fn tested_block(&mut self, cond: &ast::Cond, body: &ast::Block) -> Ty {
        let outer = self.param_bounds.len();
        self.param_bounds
            .extend(cpl_syntax::narrow::tested_bounds(cond));
        let ty = self.block(body);
        self.param_bounds.truncate(outer);
        ty
    }
}
