//! Text-level helpers shared by the taste rules: receivers, sequences and names.

use cpl_syntax::ast::*;

pub(super) use crate::rules::shapes::mentions;

/// `0..` as `(0..)`, so a following `if` is not read as its end.
pub(super) fn iter_text<'s>(iter: &Expr, text: &'s str) -> std::borrow::Cow<'s, str> {
    match &iter.kind {
        ExprKind::Range { hi: None, .. } => format!("({text})").into(),
        _ => text.into(),
    }
}

/// A start value no element beats: `Int.MAX`, `INF`, `1_000_000_000`, `-1e18`.
pub(super) fn sentinel(init: &str) -> bool {
    let digits: String = init.chars().filter(|c| c.is_ascii_digit()).collect();
    ["MAX", "MIN", "INF", "inf", "e18", "e9", "**"]
        .iter()
        .any(|w| init.contains(w))
        || (digits.len() >= 9
            && init
                .trim_start_matches('-')
                .chars()
                .all(|c| c.is_ascii_digit() || c == '_'))
}

/// `it` as the receiver of a method call.
pub(super) fn receiver(it: &str) -> std::borrow::Cow<'_, str> {
    // A name followed by calls, indexes and fields (`adj[v]`, `divisors(x)`)
    // takes a method as it is; anything with a top-level operator needs parens.
    let mut depth = 0usize;
    let mut plain = !it.starts_with(['(', '[']) && !it.contains("..");
    for c in it.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 && !(c.is_alphanumeric() || "_.".contains(c)) => plain = false,
            _ => {}
        }
    }
    if it.starts_with('(') || plain {
        it.into()
    } else {
        format!("({it})").into()
    }
}

/// The values a `for p in it` loop visits: `it`, mapped and filtered as written.
pub(super) fn sequence(it: &str, p: &str, v: &str, cond: Option<&str>) -> String {
    let mut seq = receiver(it).into_owned();
    if let Some(c) = cond {
        seq = format!("{seq}.filter({p} => {c})");
    }
    if v != p {
        seq = format!("{seq}.map({p} => {v})");
    }
    seq
}

/// An initial value `sum` can start from: a number or a `.zero`.
pub(super) fn numeric(init: &str) -> bool {
    init.ends_with(".zero")
        || init
            .trim_start_matches('-')
            .starts_with(|c: char| c.is_ascii_digit())
}
