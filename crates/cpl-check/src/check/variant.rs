//! A bare variant that two enums declare (R02.7.1).

use super::*;
use cpl_syntax::ast::{Block, Expr, ExprKind, Pattern, StmtKind, Type};

impl<'a> Checker<'a> {
    /// The type `let pat: ty = init` declares. With no annotation nothing says
    /// which enum a bare variant in `init`'s value means, so one that two enums
    /// declare is E0201 rather than a pick of one.
    pub(super) fn let_declared(
        &mut self,
        pat: &Pattern,
        ty: Option<&Type>,
        init: Option<&Expr>,
    ) -> Option<Ty> {
        match (ty, init) {
            (Some(ty), _) => Some(self.let_annotation(pat, ty)),
            (None, Some(init)) => {
                self.shared_bare_variants(init);
                None
            }
            (None, None) => None,
        }
    }

    /// `e`'s value positions: itself, a tuple's items, and what a block, `if`
    /// or `match` ends with.
    fn shared_bare_variants(&mut self, e: &Expr) {
        let tail = |b: &Block| match b.stmts.last().map(|s| &s.kind) {
            Some(StmtKind::Expr(e)) => Some(e.clone()),
            _ => None,
        };
        let values: Vec<Expr> = match &e.kind {
            ExprKind::Name(name) => return self.shared_bare_variant(name, e.span),
            ExprKind::Paren(inner) => vec![(**inner).clone()],
            ExprKind::Tuple(items) => items.clone(),
            ExprKind::Block(b) => tail(b).into_iter().collect(),
            ExprKind::If {
                branches,
                otherwise,
            } => branches
                .iter()
                .map(|(_, b)| b)
                .chain(otherwise)
                .filter_map(tail)
                .collect(),
            ExprKind::Match { arms, .. } => arms.iter().filter_map(|a| tail(&a.body)).collect(),
            _ => Vec::new(),
        };
        for value in &values {
            self.shared_bare_variants(value);
        }
    }

    fn shared_bare_variant(&mut self, name: &str, span: Span) {
        if self.local(name).is_some()
            || !matches!(self.global(name).as_deref(), Some(Def::Variant { .. }))
        {
            return;
        }
        let enums = self.env.variant_enums(name);
        let [a, b, ..] = enums.as_slice() else {
            return;
        };
        let message =
            format!("`{name}` is a variant of both `{a}` and `{b}`, and nothing here says which");
        let help = format!("write `{a}.{name}`, or give the `let` a type: `let x: {a} = {name}`");
        if let Some(diag) = self.error(codes::UNRESOLVED_NAME, span, message) {
            diag.help = Some(help);
        }
    }
}
