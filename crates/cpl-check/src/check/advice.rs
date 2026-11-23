//! The words and fixes of a few errors, kept out of the passes that find them.

use super::*;
use cpl_diag::Fix;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl Checker<'_> {
    /// `let N = 5` as `const N = 5`: only a literal can be a constant as is.
    pub(super) fn const_fix(&self, keyword: &str, name: Span, init: Option<&Expr>) -> Option<Fix> {
        let literal = |e: &Expr| {
            matches!(
                e.kind,
                ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_)
            )
        };
        let at = Span::new(name.start.checked_sub(4)?, name.start - 1);
        (keyword == "let" && init.is_some_and(literal) && self.text(at) == "let").then(|| Fix {
            title: "make it a `const`".into(),
            edits: vec![(at, "const".into())],
        })
    }

    /// `"a" + n` as `"a{n}"`, and `n + "a"` as `"{n}a"`.
    pub(super) fn string_fix(&self, lhs: &Expr, rhs: &Expr) -> Option<(Span, String)> {
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
        let reason = if why.contains("only when") {
            why
        } else {
            "a call can't change one place through two names"
        };
        return (
            format!("`{first}` is lent to this call twice, {how}: {reason}"),
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

/// `True` or `null` from another language, as CP Lang spells the value.
pub(super) fn foreign_value(name: &str) -> Option<&'static str> {
    match name {
        "True" | "TRUE" => Some("true"),
        "False" | "FALSE" => Some("false"),
        "null" | "nil" | "None" | "NULL" | "nullptr" => Some("none"),
        _ => None,
    }
}

/// Python's conversion builtins and `math.`, as CP Lang writes them.
pub(super) fn foreign_call(name: &str) -> Option<&'static str> {
    match name {
        "str" | "to_string" | "toString" => Some("make a string by interpolation: `\"{x}\"`"),
        "int" | "stoi" | "atoi" | "parseInt" => {
            Some("read a number from a string with `s.parse<Int>()!`")
        }
        "float" | "stod" | "parseFloat" => {
            Some("read a number from a string with `s.parse<F64>()!`")
        }
        "math" | "Math" => Some("math functions are methods here: `x.sqrt`, `x.abs`, `x.isqrt`"),
        _ => None,
    }
}
