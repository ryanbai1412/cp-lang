//! Headers written as another language writes them, read as ours.

use super::*;

impl<'a> Parser<'a> {
    /// `let [a, b] = xs`: a list has no pattern in CP Lang. One error,
    /// with the names still bound so their uses don't cascade.
    pub(super) fn list_pattern(&mut self) -> PatternKind {
        let start = self.start();
        self.bump();
        let mut items = Vec::new();
        while !matches!(self.peek(), T::RBracket | T::Newline | T::Eof) {
            items.push(self.pattern());
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.expect(T::RBracket, "`]` after the list pattern");
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_PATTERN,
                self.span_from(start),
                "a list can't be unpacked in CP Lang: only tuples can",
            )
            .with_help(
                "index it, `let a = xs[0]`, or unpack a tuple: `let (a, b) = (xs[0], xs[1])`",
            ),
        );
        PatternKind::Tuple(items)
    }

    /// `0..10 step 2`: the stride is `by` in CP Lang. One error with the
    /// fix, read as `by`.
    pub(super) fn foreign_step(&mut self) -> bool {
        if !self.at_ident("step") || !matches!(self.peek_n(1), T::Int | T::Ident | T::LParen) {
            return false;
        }
        let span = self.span();
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                span,
                "a range's stride is `by` in CP Lang, not `step`",
            )
            .with_help("write `0..10 by 2`")
            .with_fix("write `by`", span, "by"),
        );
        self.bump();
        true
    }

    /// Python's `is not None`. `x is None` is our own pattern test, so it stays.
    pub(super) fn at_python_is(&self) -> bool {
        self.at_ident("is")
            && self.peek_n(1) == T::Not
            && self
                .tokens
                .get(self.pos + 2)
                .is_some_and(|t| t.kind == T::Ident && t.span.text(self.src) == "None")
    }

    /// `xs[::-1]`: Python's reversed copy. One error with the `xs.rev` fix.
    fn reversed_slice(&mut self) -> Option<Expr> {
        let t = |k: usize| self.tokens.get(self.pos + k);
        let one = t(2).is_some_and(|t| {
            t.kind == T::Int && &self.src[t.span.start as usize..t.span.end as usize] == "1"
        });
        let open = self.pos.checked_sub(1).map(|p| self.tokens[p]);
        if !self.at(T::ColonColon)
            || self.peek_n(1) != T::Minus
            || !one
            || self.peek_n(3) != T::RBracket
            || open.is_none_or(|o| o.kind != T::LBracket)
        {
            return None;
        }
        let close = self.tokens[self.pos + 3].span;
        let whole = self.rel(Span::new(open?.span.start, close.end));
        let span = self.span();
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                span,
                "CP Lang reverses with `.rev`, not `[::-1]`",
            )
            .with_help("`xs.rev` is the reversed view; `List(xs.rev)` copies it")
            .with_fix("write `.rev`", whole, ".rev"),
        );
        for _ in 0..3 {
            self.bump();
        }
        Some(Expr {
            kind: ExprKind::Error,
            span,
        })
    }

    /// Steps over a comparison operator, reading `!in` as `not in` and
    /// Python's `x is None` and `x is not None` as `==` and `!=`, each one
    /// error with a fix.
    pub(super) fn cmp_token(&mut self) {
        if self.at(T::Bang) {
            let span = self.span();
            self.error(
                Diagnostic::error(codes::UNEXPECTED_TOKEN, span, "`!` doesn't negate in cp")
                    .with_help("write `not in` instead of `!in`")
                    .with_fix("replace with `not `", span, "not "),
            );
        }
        if !self.at_ident("is") {
            self.bump();
            return;
        }
        let start = self.span();
        self.bump();
        let (foreign, ours) = if self.eat(T::Not) {
            ("is not", "!=")
        } else {
            ("is", "==")
        };
        let span = Span::new(start.start, self.rel(self.tokens[self.pos - 1].span).end);
        self.recovered(
            Diagnostic::error(
                codes::UNEXPECTED_TOKEN,
                span,
                format!("CP Lang compares with `{ours}`, not `{foreign}`"),
            )
            .with_help(format!(
                "write `x {ours} None`; values are compared, there is no identity"
            ))
            .with_fix(format!("write `{ours}`"), span, ours),
        );
    }

    /// `if c: a else: b`, as the ternaries of other languages read.
    pub(super) fn inline_if(cond: Expr, yes: Expr, no: Expr, span: Span) -> Expr {
        let block = |e: Expr| Block {
            stmts: vec![Stmt {
                public: false,
                decorators: Vec::new(),
                span: e.span,
                kind: StmtKind::Expr(e),
            }],
            span,
        };
        Expr {
            kind: ExprKind::If {
                branches: vec![(Cond::Expr(cond), block(yes))],
                otherwise: Some(block(no)),
            },
            span,
        }
    }

    /// `a if c else b`: Python's conditional expression. One error with the
    /// `if c: a else: b` fix, read as that inline `if`.
    pub(super) fn python_ternary(&mut self, start: u32, first: usize, yes: Expr) -> Expr {
        let mut depth = 0i32;
        let rest = self.tokens.get(self.pos + 1..).unwrap_or_default();
        let else_at = rest.iter().position(|t| {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                _ => {}
            }
            depth < 0
                || matches!(t.kind, T::Newline | T::Eof | T::Colon)
                || (depth == 0 && t.kind == T::Else)
        });
        let else_at = else_at.map(|k| self.pos + 1 + k);
        let Some(else_at) = else_at
            .filter(|&e| self.at(T::If) && self.tokens[e].kind == T::Else && e > self.pos + 1)
        else {
            return yes;
        };
        let iff = self.pos;
        let if_span = self.span();
        self.bump();
        let cond = self.coalesce();
        if self.pos != else_at {
            self.pos = iff;
            return yes;
        }
        self.bump();
        let no = self.expr();
        let text = |p: &Self, a: usize, b: usize| {
            let (a, b) = (p.tokens[a].span.start, p.tokens[b - 1].span.end);
            p.src[a as usize..b as usize].to_string()
        };
        let (a, c, b) = (
            text(self, first, iff),
            text(self, iff + 1, else_at),
            text(self, else_at + 1, self.pos),
        );
        let whole = self.rel(Span::new(
            self.tokens[first].span.start,
            self.tokens[self.pos - 1].span.end,
        ));
        self.recovered(
            Diagnostic::error(
                codes::REMOVED_SYNTAX,
                if_span,
                "CP Lang writes `a if c else b` as `if c: a else: b`",
            )
            .with_help("an inline `if` takes `:` after the condition and after `else`")
            .with_fix(
                "write it as an inline `if`",
                whole,
                format!("if {c}: {a} else: {b}"),
            ),
        );
        Self::inline_if(cond, yes, no, self.span_from(start))
    }

    /// `lambda x: x * 2`: Python's lambda. One error with the `x => x * 2`
    /// fix, read as that lambda.
    pub(super) fn python_lambda(&mut self) -> Expr {
        let start = self.start();
        let kw = self.span();
        self.bump();
        let (mut params, mut names) = (Vec::new(), Vec::new());
        while self.at(T::Ident) {
            names.push(self.text().to_string());
            params.push(self.pattern());
            if !self.eat(T::Comma) {
                break;
            }
        }
        let colon = self.span();
        if !self.eat(T::Colon) {
            self.error_expected(
                codes::EXPECTED_TOKEN,
                "`:` after the lambda's parameters",
                None,
            );
            return Expr {
                kind: ExprKind::Error,
                span: self.span_from(start),
            };
        }
        let ours = match &names[..] {
            [one] => format!("{one} =>"),
            _ => format!("({}) =>", names.join(", ")),
        };
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                kw,
                "CP Lang writes a lambda with `=>`, not `lambda`",
            )
            .with_help("write `x => x * 2`, or `(a, b) => a + b`")
            .with_fix(
                format!("write `{ours}`"),
                Span::new(kw.start, colon.end),
                ours,
            ),
        );
        let body = self.lambda_body();
        Expr {
            kind: ExprKind::Lambda {
                params,
                param_tys: Vec::new(),
                param_mut: Vec::new(),
                ret_ty: None,
                body: Box::new(body),
            },
            span: self.span_from(start),
        }
    }

    /// `f"{n}"`: Python's f-string. Every CP Lang string interpolates, so
    /// one error with the fix that drops the `f`, read as the string.
    pub(super) fn f_string_prefix(&mut self) -> bool {
        let next = self.tokens.get(self.pos + 1);
        let adjacent = next
            .is_some_and(|t| t.kind == T::Str && t.span.start == self.tokens[self.pos].span.end);
        if !self.at_ident("f") || !adjacent {
            return false;
        }
        let span = self.span();
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                span,
                "CP Lang strings interpolate without an `f` prefix",
            )
            .with_help("write `\"n = {n}\"`")
            .with_fix("remove the `f`", span, ""),
        );
        self.bump();
        true
    }

    /// `let a, b = t`, `for i, x in ...`: Python's unpacking without parentheses. One error
    /// with the fix that adds them, read as the tuple pattern.
    pub(super) fn bare_tuple_pattern(&mut self, first: Pattern, end: T) -> Pattern {
        let mut n = 0;
        while self.peek_n(n) == T::Comma && self.peek_n(n + 1) == T::Ident {
            n += 2;
        }
        if n == 0 || self.peek_n(n) != end {
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
        .with_help(if end == T::In {
            "write `for (i, x) in xs.items:`"
        } else {
            "write `let (a, b) = t`"
        });
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

    /// `if x == 3 then`, `while n > 0 do`: Lua's and Pascal's word at the
    /// end of a header.
    /// One error with the `:` fix, read as the `:`.
    pub(super) fn foreign_then(&mut self) -> bool {
        let Some(word) = ["then", "do"].into_iter().find(|w| self.at_ident(w)) else {
            return false;
        };
        if !matches!(self.peek_n(1), T::Newline | T::Eof) {
            return false;
        }
        let end = self.tokens[self.pos].span.end;
        let start = self
            .pos
            .checked_sub(1)
            .map_or(end - word.len() as u32, |p| self.tokens[p].span.end);
        let gap = self.rel(Span::new(start, end));
        self.recovered(
            Diagnostic::error(
                codes::EXPECTED_TOKEN,
                self.span(),
                format!("a header ends with `:` in CP Lang, not `{word}`"),
            )
            .with_help(if word == "do" {
                "write `while n > 0:`"
            } else {
                "write `if x == 3:`"
            })
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
        if let Some(e) = self.reversed_slice() {
            return e;
        }
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
