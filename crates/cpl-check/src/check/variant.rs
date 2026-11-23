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

    /// A bare variant two enums declare, at one of `e`'s value positions.
    fn shared_bare_variants(&mut self, e: &Expr) {
        for value in value_positions(e, true) {
            if let ExprKind::Name(name) = &value.kind {
                self.shared_bare_variant(name, value.span);
            }
        }
    }

    /// Whether every value position of `at` is a unit variant of the enum
    /// `want` (or, in a list or tuple literal, of the enum its part wants),
    /// one of them a bare one another enum declares too: the expected type
    /// says which it is. (R02.7.1)
    pub(crate) fn expected_variants(&mut self, at: &Expr, want: &Ty) -> bool {
        let mut shared = false;
        self.variants_fit(at, want, &mut shared) && shared
    }

    fn variants_fit(&mut self, at: &Expr, want: &Ty, shared: &mut bool) -> bool {
        for value in value_positions(at, false) {
            let fits = match (&value.kind, want) {
                (ExprKind::Name(name), Ty::Named { name: want, args }) => {
                    self.variant_fits(name, want, args.is_empty(), shared)
                }
                (ExprKind::List(items), Ty::Named { name, args }) if name == "List" => {
                    let elem = args.first().cloned().unwrap_or(Ty::Unknown);
                    items
                        .iter()
                        .all(|item| self.variants_fit(item, &elem, shared))
                }
                (ExprKind::List(items), Ty::Array { elem, .. }) => items
                    .iter()
                    .all(|item| self.variants_fit(item, elem, shared)),
                (ExprKind::Tuple(items), Ty::Tuple(parts)) if items.len() == parts.len() => {
                    let mut all = true;
                    for (item, part) in items.iter().zip(parts) {
                        all &= self.variants_fit(item, part, shared);
                    }
                    all
                }
                _ => false,
            };
            if !fits {
                return false;
            }
        }
        true
    }

    fn variant_fits(&mut self, name: &str, want: &str, plain: bool, shared: &mut bool) -> bool {
        if self.local(name).is_some() {
            return false;
        }
        match self.global(name).as_deref() {
            Some(Def::Variant { of, fields, .. }) if of == want => fields.is_empty(),
            Some(Def::Variant { .. })
                if plain && self.env.variant_enums(name).iter().any(|e| e == want) =>
            {
                *shared = true;
                true
            }
            _ => false,
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

/// `e`'s value positions: itself, what a block, `if` or `match` ends with,
/// and with `tuples`, a tuple's items.
fn value_positions(e: &Expr, tuples: bool) -> Vec<Expr> {
    let tail = |b: &Block| match b.stmts.last().map(|s| &s.kind) {
        Some(StmtKind::Expr(e)) => Some(e.clone()),
        _ => None,
    };
    let inner: Vec<Expr> = match &e.kind {
        ExprKind::Paren(inner) => vec![(**inner).clone()],
        ExprKind::Tuple(items) if tuples => items.clone(),
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
        _ => return vec![e.clone()],
    };
    inner
        .iter()
        .flat_map(|e| value_positions(e, tuples))
        .collect()
}
