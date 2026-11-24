//! Text-level helpers shared by the taste rules: receivers, sequences and names.

use cpl_syntax::ast::*;
use std::collections::HashSet;

pub(super) use crate::rules::shapes::mentions;

/// `0..` as `(0..)`, so a following `if` is not read as its end.
pub(super) fn iter_text<'s>(iter: &Expr, text: &'s str) -> std::borrow::Cow<'s, str> {
    match &iter.kind {
        ExprKind::Range { hi: None, .. } => format!("({text})").into(),
        _ => text.into(),
    }
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

pub(super) fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

pub(super) fn idents(src: &str) -> HashSet<&str> {
    words(src).collect()
}
