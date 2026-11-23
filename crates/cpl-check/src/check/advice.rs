//! The words and fixes of a few errors, kept out of the passes that find them.

use super::*;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl Checker<'_> {
    /// `"a" + n` as `"a{n}"`, and `n + "a"` as `"{n}a"`.
    pub(super) fn into_string(&self, lhs: &Expr, rhs: &Expr) -> Option<(Span, String)> {
        let literal = |e: &Expr| {
            let text = self.quote(e.span);
            let plain = matches!(e.kind, ExprKind::Str(_))
                && text.len() >= 2
                && text.starts_with('"')
                && text.ends_with('"')
                && !text.starts_with("\"\"\"");
            plain.then_some(text)
        };
        let span = Span::new(lhs.span.start, rhs.span.end);
        if let Some(text) = literal(lhs) {
            let value = self.quote(rhs.span);
            return Some((span, format!("{}{{{value}}}\"", &text[..text.len() - 1])));
        }
        let text = literal(rhs)?;
        Some((
            span,
            format!("\"{{{}}}{}", self.quote(lhs.span), &text[1..]),
        ))
    }
}

/// E0601's message and help: one place passed twice is said plainly.
pub(super) fn lent_twice(a: &Arg, b: &Arg, why: &str) -> (String, String) {
    let (first, second) = (super::lends::show(&a.value), super::lends::show(&b.value));
    if first == second {
        let how = if a.mutable && b.mutable {
            "both times `mut`"
        } else {
            "once `mut`"
        };
        return (
            format!("`{first}` is lent to this call twice, {how}: a call can't change one place through two names"),
            format!("pass `{first}` once"),
        );
    }
    let how = if a.mutable && b.mutable {
        "both are"
    } else {
        "one is"
    };
    (
        format!("`{first}` and `{second}` may be the same storage, and {how} lent `mut` to this call: {why}"),
        "guard it (`if i != j: ...`), state it (`assert i != j`), or copy the read-only one first".into(),
    )
}

/// E0407: point at the first variant with the tag, and say what to change.
pub(super) fn duplicate_tag(diag: &mut Diagnostic, first: Span) {
    diag.labels.push(cpl_diag::Label {
        span: first,
        message: "first `@tag` here".to_string(),
    });
    diag.help = Some("input names one variant per tag: give one of them another `@tag`".into());
}
