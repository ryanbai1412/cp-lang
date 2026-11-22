//! Methods that keep a cache in their struct.
//!
//! `Comb<P>` grows its factorial tables the first time a larger `n` is
//! asked for, and yet `c.choose(n, k)` takes plain `self`: to its caller
//! the value never changes, it only remembers more. A method like that
//! is handed the caller's own struct rather than a copy of it, or what it
//! remembered would be thrown away when it returned.

use std::collections::HashSet;

use cpl_syntax::ast::{self, ExprKind, StmtKind};
use cpl_syntax::visit::{walk_block, walk_expr, Visitor};

/// Whether `method`, which takes `self` without `mut`, appends to a list
/// field of `owner`, itself or through another such method.
pub(super) fn grows_cache(owner: &ast::StructDecl, method: &ast::FnDecl) -> bool {
    if takes_plain_self(method) && owner.fields.iter().any(|f| f.cached) {
        return true;
    }
    grows(owner, method, &mut HashSet::new())
}

fn takes_plain_self(f: &ast::FnDecl) -> bool {
    f.params
        .first()
        .is_some_and(|p| p.name.text == "self" && !p.mutable)
}

fn grows<'a>(
    owner: &'a ast::StructDecl,
    method: &'a ast::FnDecl,
    seen: &mut HashSet<&'a str>,
) -> bool {
    if !takes_plain_self(method) || !seen.insert(method.name.text.as_str()) {
        return false;
    }
    let mut body = Body {
        owner,
        // A parameter hides a field of the same name.
        hidden: method.params.iter().map(|p| p.name.text.as_str()).collect(),
        pushes: false,
        calls: Vec::new(),
    };
    match &method.body {
        ast::FnBody::Block(block) => walk_block(block, &mut body),
        ast::FnBody::Expr(expr) => walk_expr(expr, &mut body),
        ast::FnBody::None => {}
    }
    if body.pushes {
        return true;
    }
    let calls = body.calls;
    owner.methods.iter().any(|stmt| match &stmt.kind {
        StmtKind::Fn(f) => calls.contains(&f.name.text) && grows(owner, f, seen),
        _ => false,
    })
}

struct Body<'a> {
    owner: &'a ast::StructDecl,
    hidden: Vec<&'a str>,
    pushes: bool,
    /// The methods of `self` this one calls.
    calls: Vec<String>,
}

impl Body<'_> {
    /// `f` or `self.f`, for a field `f`.
    fn is_field(&self, e: &ast::Expr) -> bool {
        let name = match &e.kind {
            ExprKind::Name(name) if !self.hidden.contains(&name.as_str()) => name.as_str(),
            ExprKind::Field { base, name } if matches!(base.kind, ExprKind::SelfValue) => {
                &name.text
            }
            _ => return false,
        };
        self.owner.fields.iter().any(|f| f.name.text == name)
    }
}

impl Visitor for Body<'_> {
    fn expr(&mut self, e: &ast::Expr) {
        let ExprKind::Call { callee, .. } = &e.kind else {
            return;
        };
        match &callee.kind {
            ExprKind::Field { base, name } if name.text == "push" && self.is_field(base) => {
                self.pushes = true;
            }
            ExprKind::Field { base, name } if matches!(base.kind, ExprKind::SelfValue) => {
                self.calls.push(name.text.clone());
            }
            ExprKind::Name(name) => self.calls.push(name.clone()),
            _ => {}
        }
    }
}
