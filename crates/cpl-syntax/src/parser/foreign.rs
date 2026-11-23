//! Headers written as another language writes them, read as ours.

use super::*;

impl<'a> Parser<'a> {
    /// `for (i = 0; i < n; i++)`: C's loop header. One error, with a fix
    /// when it counts up by one, read as `for i in <error>` so the body
    /// still checks.
    pub(super) fn c_for(&mut self) -> Option<(Pattern, Expr)> {
        let open = self.pos;
        let end = after_group(self.tokens, open);
        let inner = self.tokens.get(open + 1..end.checked_sub(1)?)?;
        if !self.at(T::LParen) || inner.iter().filter(|t| t.kind == T::Semi).count() != 2 {
            return None;
        }
        let (a, b) = (self.tokens[open].span.start, self.tokens[end - 1].span.end);
        let parts: Vec<&str> = self.src[a as usize + 1..b as usize - 1]
            .split(';')
            .map(str::trim)
            .collect();
        let whole = self.rel(Span::new(a, b));
        let mut diag = Diagnostic::error(
            codes::REMOVED_SYNTAX,
            whole,
            "CP Lang has no C-style `for (init; cond; step)`",
        )
        .with_help("loop over a range: `for i in 0..n:`");
        let (name, range) = counted(&parts);
        if let (Some(name), Some(range)) = (name, range) {
            diag = diag.with_fix("loop over the range", whole, format!("{name} in {range}"));
        }
        self.recovered(diag);
        self.pos = end;
        let kind = name.map_or(PatternKind::Wildcard, |n| PatternKind::Name(n.to_string()));
        let pat = Pattern { kind, span: whole };
        Some((
            pat,
            Expr {
                kind: ExprKind::Error,
                span: whole,
            },
        ))
    }

    /// `case 1:` or `default:` opening a `match` arm, as a `switch` writes
    /// it: one error, read as the arm `1:` or `default:` (a catch-all).
    pub(super) fn c_case(&mut self) {
        let next = self.peek_n(1);
        let (span, ours, fix) =
            if self.at_ident("case") && !matches!(next, T::FatArrow | T::Colon | T::If) {
                let span = Span::new(
                    self.span().start,
                    self.rel(self.tokens[self.pos + 1].span).start,
                );
                let pat = self.tokens[self.pos + 1].span.text(self.src).to_string();
                self.bump();
                (span, format!("{pat}:"), "")
            } else if self.at_ident("default") && next == T::Colon {
                (self.span(), "_:".to_string(), "_")
            } else {
                return;
            };
        let theirs = if fix.is_empty() { "case" } else { "default" };
        self.recovered(
            Diagnostic::error(
                codes::REMOVED_SYNTAX,
                span,
                format!("a `match` arm has no `{theirs}`"),
            )
            .with_help(format!("write the pattern itself: `{ours}`"))
            .with_fix(format!("remove `{theirs}`"), span, fix),
        );
    }

    /// One index in `xs[...]`; Python's slice `xs[a:b]` is one error with
    /// the `xs[a..b]` fix, read as that range.
    pub(super) fn index_item(&mut self) -> Expr {
        let start = self.start();
        let lo = (!self.at(T::Colon)).then(|| self.expr());
        let Some(colon) = self.at(T::Colon).then(|| self.span()) else {
            return lo.unwrap_or(Expr {
                kind: ExprKind::Error,
                span: self.span(),
            });
        };
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                colon,
                "CP Lang slices with `..`, not `:`",
            )
            .with_help("write `xs[a..b]`; `xs[a..]` and `xs[..b]` leave out an end")
            .with_fix("replace `:` with `..`", colon, ".."),
        );
        self.bump();
        let hi = (!matches!(self.peek(), T::RBracket | T::Comma)).then(|| Box::new(self.expr()));
        let kind = ExprKind::Range {
            lo: lo.map(Box::new),
            hi,
            inclusive: false,
            step: None,
        };
        Expr {
            kind,
            span: self.span_from(start),
        }
    }
}

/// The loop variable of `i = lo; i < hi; i++`, and `lo..hi` when it counts
/// up by one.
fn counted<'s>(parts: &[&'s str]) -> (Option<&'s str>, Option<String>) {
    let mut init = parts[0];
    for ty in [
        "int ",
        "long long ",
        "long ",
        "auto ",
        "size_t ",
        "var ",
        "let ",
    ] {
        init = init.strip_prefix(ty).unwrap_or(init);
    }
    let Some((name, lo)) = init.split_once('=') else {
        return (None, None);
    };
    let name = name.trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return (None, None);
    }
    let cond = parts[1].strip_prefix(name).map(str::trim_start);
    let range = match cond {
        Some(c) if c.starts_with("<=") => Some(("..=", &c[2..])),
        Some(c) if c.starts_with('<') => Some(("..", &c[1..])),
        _ => None,
    };
    let steps = [
        format!("{name}++"),
        format!("++{name}"),
        format!("{name} += 1"),
        format!("{name}+=1"),
    ];
    let range = range
        .filter(|_| steps.iter().any(|s| s == parts[2]))
        .map(|(op, hi)| format!("{}{op}{}", lo.trim(), hi.trim()));
    (Some(name), range)
}
