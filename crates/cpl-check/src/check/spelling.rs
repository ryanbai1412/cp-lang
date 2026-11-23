//! Fixes for spellings other languages use where cp has its own.

use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind};

impl<'a> Checker<'a> {
    /// `a.pow(e)` rewritten to `a ** e`; none for `a.pow(e, m)`.
    pub(crate) fn pow_fix(&self, field: &Expr, name: &ast::Name) -> Option<cpl_diag::Fix> {
        let ExprKind::Field { base, .. } = &field.kind else {
            return None;
        };
        let inner = self.src.get(name.span.end as usize..)?.strip_prefix('(')?;
        let (mut depth, mut close) = (0u32, None);
        for (i, c) in inner.char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' if depth == 0 => {
                    close = Some(i);
                    break;
                }
                ')' | ']' | '}' => depth = depth.checked_sub(1)?,
                ',' | '\n' | '"' if depth == 0 => return None,
                _ => {}
            }
        }
        let close = close?;
        let arg = inner[..close].trim();
        let atom = arg
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '[' | ']'));
        if arg.is_empty() {
            return None;
        }
        let arg = if atom {
            arg.to_string()
        } else {
            format!("({arg})")
        };
        let end = name.span.end as usize + close + 2;
        let rest = self.src.get(end..).unwrap_or("");
        let mut text = format!("{} ** {arg}", self.text(base.span));
        if matches!(rest.chars().next(), Some('.' | '[' | '!' | '*')) {
            text = format!("({text})");
        }
        Some(cpl_diag::Fix {
            title: format!("replace with `{text}`"),
            edits: vec![(Span::new(base.span.start, end as u32), text)],
        })
    }
}
