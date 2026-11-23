//! Headers written as another language writes them, read as ours.

use super::*;

impl<'a> Parser<'a> {
    /// `let a, b = t`: Python's unpacking without parentheses. One error
    /// with the fix that adds them, read as the tuple pattern.
    pub(super) fn bare_tuple_pattern(&mut self, first: Pattern) -> Pattern {
        let mut n = 0;
        while self.peek_n(n) == T::Comma && self.peek_n(n + 1) == T::Ident {
            n += 2;
        }
        if n == 0 || self.peek_n(n) != T::Eq {
            return first;
        }
        let start = first.span.start;
        let mut items = vec![first];
        while self.eat(T::Comma) {
            items.push(self.pattern());
        }
        let span = self.span_from(start);
        let mut diag = Diagnostic::error(
            codes::EXPECTED_TOKEN,
            span,
            "unpacking a tuple takes parentheses in CP Lang",
        )
        .with_help("write `let (a, b) = t`");
        diag.fixes.push(cpl_diag::Fix {
            title: "add the parentheses".into(),
            edits: vec![
                (Span::new(span.start, span.start), "(".into()),
                (Span::new(span.end, span.end), ")".into()),
            ],
        });
        self.recovered(diag);
        Pattern {
            kind: PatternKind::Tuple(items),
            span,
        }
    }

    /// `if x == 3 then`: Lua's and Pascal's `then` at the end of a header.
    /// One error with the `:` fix, read as the `:`.
    pub(super) fn foreign_then(&mut self) -> bool {
        if !self.at_ident("then") || !matches!(self.peek_n(1), T::Newline | T::Eof) {
            return false;
        }
        let end = self.tokens[self.pos].span.end;
        let start = self
            .pos
            .checked_sub(1)
            .map_or(end - 4, |p| self.tokens[p].span.end);
        let gap = self.rel(Span::new(start, end));
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                self.span(),
                "a header ends with `:` in CP Lang, not `then`",
            )
            .with_help("write `if x == 3:`")
            .with_fix("write `:`", gap, ":"),
        );
        self.bump();
        true
    }

    /// `sorted(xs, reverse=True)`: Python's keyword argument. One error
    /// with the `:` fix, read as the named argument.
    pub(super) fn keyword_arg(&mut self) -> Name {
        let (name_end, eq) = (
            self.tokens[self.pos].span.end,
            self.tokens[self.pos + 1].span,
        );
        let value = self
            .tokens
            .get(self.pos + 2)
            .map_or(eq.end, |t| t.span.start);
        let gap = self.rel(Span::new(name_end, value));
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                self.rel(eq),
                "a named argument takes `:`, not `=`",
            )
            .with_help("write `name: value`, as in `print(xs, sep: \", \")`")
            .with_fix("write `:`", gap, ": "),
        );
        let name = self.name();
        self.bump();
        name
    }

    /// `for (x in xs):`: parentheses around the header, as in C and
    /// JavaScript. One error with the fix that drops them, read without them.
    pub(super) fn paren_for(&mut self) -> bool {
        let open = self.pos;
        let end = after_group(self.tokens, open);
        let Some(inner) = end
            .checked_sub(1)
            .and_then(|e| self.tokens.get(open + 1..e))
        else {
            return false;
        };
        let mut depth = 0i32;
        let has_in = inner.iter().any(|t| {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                _ => {}
            }
            depth == 0 && t.kind == T::In
        });
        if !self.at(T::LParen) || !has_in || self.tokens.get(end).map(|t| t.kind) != Some(T::Colon)
        {
            return false;
        }
        let (o, c) = (self.span(), self.rel(self.tokens[end - 1].span));
        let mut diag = Diagnostic::error(
            codes::REMOVED_SYNTAX,
            o,
            "a `for` header takes no parentheses in CP Lang",
        )
        .with_help("write `for x in xs:`");
        diag.fixes.push(cpl_diag::Fix {
            title: "remove the parentheses".into(),
            edits: vec![(o, String::new()), (c, String::new())],
        });
        self.recovered(diag);
        self.bump();
        true
    }

    /// `for x of xs`: JavaScript's `of`. One error with the `in` fix.
    pub(super) fn for_of(&mut self) -> bool {
        if !self.at_ident("of") {
            return false;
        }
        let span = self.span();
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                span,
                "CP Lang loops with `in`, not `of`",
            )
            .with_help("write `for x in xs:`")
            .with_fix("replace `of` with `in`", span, "in"),
        );
        self.bump();
        true
    }

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
