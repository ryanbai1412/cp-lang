//! `i + 1` and other `?I` values used where a position of `I` is wanted.

use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    pub(super) fn optional_compare(
        &mut self,
        op_str: &str,
        (left_expr, left): (&Expr, &Ty),
        (right_expr, right): (&Expr, &Ty),
    ) {
        let unwrap = |c: &Self, e: &Expr, ty: &Ty| match (&e.kind, ty) {
            (_, ty) if !matches!(ty, Ty::Optional(_)) => c.quote(e.span),
            (ExprKind::Name(_) | ExprKind::Field { .. }, _) => format!("{}!", c.quote(e.span)),
            _ => format!("({})!", c.quote(e.span)),
        };
        let (at, ty) = match left {
            Ty::Optional(_) => (left_expr, left),
            _ => (right_expr, right),
        };
        let message = format!(
            "`{}` is {}: optional values can't be ordered with `{op_str}`",
            self.quote(at.span),
            ty.with_article()
        );
        let (l, r) = (
            unwrap(self, left_expr, left),
            unwrap(self, right_expr, right),
        );
        if let Some(diag) = self.error(codes::OPTIONAL_COMPARE, at.span, message) {
            diag.help = Some(format!(
                "`None` has no order: unwrap first, `{l} {op_str} {r}`, or give a default with `??`"
            ));
        }
    }

    pub(super) fn with_not_first(&mut self, span: Span) {
        let message =
            "a bare `with` line comes first in its block; indent what it covers under `with ...:`";
        if let Some(diag) = self.error(codes::WITH_NOT_FIRST, span, message.to_string()) {
            diag.help = Some(super::advice::WITH_FIRST_HELP.into());
        }
    }

    /// Whether the step `e`, a `?I`, is known to stay inside `I`: a test
    /// or a loop around it orders the name it steps from against another
    /// position of `I` ([`cpl_syntax::order`]).
    pub(super) fn step_stays(&self, e: &Expr, domain: &Ty) -> bool {
        matches!(domain, Ty::Pos(_))
            && self.steps.bounds(e, 0).into_iter().any(|name| {
                match cpl_syntax::order::end_domain(name) {
                    // `i > D.first`: an end of the domain itself.
                    Some(of) => matches!(domain, Ty::Pos(d) if d == of),
                    None => self.local(name).is_some_and(|local| local.ty == *domain),
                }
            })
    }

    /// `i + 1` used where its domain is wanted: the message for indexing
    /// with it. A step is a `?I`, `none` when it leaves `I`.
    pub(super) fn unhandled_step(&self, got: &Ty, want: &Ty, at: &Expr) -> Option<String> {
        let Ty::Optional(inner) = got else {
            return None;
        };
        // `let u = xs.pop()`: a name holds the `?V`, it did not step.
        let may = match at.kind {
            ExprKind::Name(_) => "may be `None`".into(),
            _ => format!("may step outside {inner}"),
        };
        (matches!(**inner, Ty::Pos(_)) && inner.fits(want)).then_some(may)
    }

    pub(super) fn step_help(&self, at: &Expr) -> String {
        let t = self.quote(at.span);
        let (default, unwrap) = match at.kind {
            ExprKind::Name(_) => (format!("a default with `{t} ?? d`"), format!("{t}!")),
            _ => ("the edge a value with `??`".into(), format!("({t})!")),
        };
        format!("bind it with `if let j = {t}`, give {default}, or assert with `{unwrap}`")
    }
}
