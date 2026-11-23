//! `i + 1` and other `?I` values used where a position of `I` is wanted.

use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
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
