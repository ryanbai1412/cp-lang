//! Python's habits: tuple assignment, `*` unpacking, `[::-1]`, `a if c else b`,
//! `lambda`, f-strings, `.format`, bare tuple patterns, keyword arguments,
//! slices and `is None`.

use super::*;

impl<'a> Parser<'a> {
    /// `a, b = b, a`, Python's swap: the line as a tuple assignment.
    /// `n, m = map(int, input().split())` reads the names as a tuple.
    pub(super) fn tuple_assign(&self) -> Option<(Span, String, &'static str)> {
        let (begin, line) = self.this_line()?;
        let (lhs, rhs) = line.split_once(" = ")?;
        let place = |c: char| c.is_alphanumeric() || matches!(c, '_' | ',' | ' ' | '.' | '[' | ']');
        if !lhs.contains(',')
            || !lhs.chars().all(place)
            || rhs.contains(" = ")
            || rhs.contains("//")
        {
            return None;
        }
        let span = Span::new(begin as u32, (begin + line.len()) as u32);
        if let Some(read) = python_read(lhs, rhs) {
            let help = habits::TUPLE_ASSIGN.help;
            return Some((self.rel(span), read, help));
        }
        let rhs = if rhs.starts_with('(') {
            rhs.to_string()
        } else {
            format!("({rhs})")
        };
        let help = habits::TUPLE_ASSIGN_2.help;
        Some((self.rel(span), format!("({lhs}) = {rhs}"), help))
    }

    /// `print(*a)`: Python's unpacking. `print` already prints a list's
    /// items, so there the `*` just goes.
    pub(super) fn python_splat(&mut self, star: Span) {
        let print = self.callee() == Some("print");
        self.bump();
        let diag = cpl_diag::msgs::foreign::PYTHON_SPLAT.error(star, &[]);
        self.recovered(if print {
            diag.with_help(habits::PYTHON_SPLAT.say(&[])).with_fix(
                habits::C_DEREF.say(&[]),
                star,
                "",
            )
        } else {
            diag.with_help(habits::PYTHON_SPLAT_2.say(&[]))
        });
    }

    /// Python's `is not None`. `x is None` is our own pattern test, so it stays.
    pub(in crate::parser) fn at_python_is(&self) -> bool {
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
            cpl_diag::msgs::foreign::REVERSED_SLICE
                .error(span, &[])
                .with_help(habits::REVERSED_SLICE.say(&[]))
                .with_fix(habits::WRITE.say(&[&".rev"]), whole, ".rev"),
        );
        for _ in 0..3 {
            self.bump();
        }
        Some(Expr {
            kind: ExprKind::Error,
            span,
        })
    }

    /// `a if c else b`: Python's conditional expression. One error with the
    /// `if c: a else: b` fix, read as that inline `if`.
    pub(in crate::parser) fn python_ternary(
        &mut self,
        start: u32,
        first: usize,
        yes: Expr,
    ) -> Expr {
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
            cpl_diag::msgs::foreign::PYTHON_TERNARY
                .error(if_span, &[])
                .with_help(habits::PYTHON_TERNARY.say(&[]))
                .with_fix(
                    habits::C_TERNARY_2.say(&[]),
                    whole,
                    format!("if {c}: {a} else: {b}"),
                ),
        );
        Self::inline_if(cond, yes, no, self.span_from(start))
    }

    pub(in crate::parser) fn python_lambda(&mut self) -> Expr {
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
            cpl_diag::msgs::foreign::PYTHON_LAMBDA
                .error(kw, &[])
                .with_help(habits::RUST_CLOSURE.say(&[]))
                .with_fix(
                    habits::WRITE.say(&[&ours]),
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
    pub(in crate::parser) fn f_string_prefix(&mut self) -> bool {
        let next = self.tokens.get(self.pos + 1);
        let adjacent = next
            .is_some_and(|t| t.kind == T::Str && t.span.start == self.tokens[self.pos].span.end);
        if !self.at_ident("f") || !adjacent {
            return false;
        }
        let span = self.span();
        self.recovered(
            cpl_diag::msgs::foreign::F_STRING_PREFIX
                .error(span, &[])
                .with_help(habits::F_STRING_PREFIX.say(&[]))
                .with_fix(habits::F_STRING_PREFIX_2.say(&[]), span, ""),
        );
        self.bump();
        true
    }

    /// `"{} {:.2f}".format(a, x)`: Python's format. A CP Lang string names
    /// its values in its holes, so one error with the fix `"{a} {x:.2f}"`,
    /// read as that string.
    pub(in crate::parser) fn python_format(&mut self) -> Option<ExprKind> {
        let format = self.tokens.get(self.pos + 2).is_some_and(|t| {
            t.kind == T::Ident
                && self.src.get(t.span.start as usize..t.span.end as usize) == Some("format")
        });
        if self.peek_n(1) != T::Dot || !format || self.peek_n(3) != T::LParen {
            return None;
        }
        let text = self.text();
        let parts = literal::string_parts(text);
        let holes: Vec<Span> = parts
            .iter()
            .filter_map(|p| match p {
                StrPart::Expr(span) => Some(*span),
                StrPart::Text(_) => None,
            })
            .collect();
        let unnamed = |h: &Span| {
            let inner = h.text(text);
            inner.is_empty() || inner.starts_with(':')
        };
        if holes.is_empty() || !holes.iter().all(unnamed) {
            return None;
        }
        let start = self.start();
        for _ in 0..4 {
            self.bump();
        }
        let mut args = Vec::new();
        while !matches!(self.peek(), T::RParen | T::Newline | T::Eof) {
            args.push(self.expr());
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.eat(T::RParen);
        let span = self.span_from(start);
        let mut diag = cpl_diag::msgs::foreign::PYTHON_FORMAT.error(span, &[]);
        if args.len() != holes.len() {
            let help = habits::PYTHON_FORMAT.say(&[]);
            self.recovered(diag.with_help(help));
            let text_only = parts.iter().filter_map(|p| match p {
                StrPart::Text(t) => Some(StrPiece::Text(unescape(t.text(text)))),
                StrPart::Expr(_) => None,
            });
            return Some(ExprKind::Str(text_only.collect()));
        }
        let (mut fixed, mut prev, mut pieces) = (String::new(), 0, Vec::new());
        let mut args = args.into_iter();
        for part in &parts {
            match part {
                StrPart::Text(t) => pieces.push(StrPiece::Text(unescape(t.text(text)))),
                StrPart::Expr(h) => {
                    let Some(arg) = args.next() else {
                        break;
                    };
                    let inner = h.text(text);
                    let arg_text = self.item_text(arg.span);
                    let plain = arg_text
                        .strip_prefix('"')
                        .and_then(|t| t.strip_suffix('"'))
                        .filter(|t| inner.is_empty() && !t.contains(['{', '}', '"', '\\']));
                    if let Some(plain) = plain {
                        fixed.push_str(&text[prev..h.start as usize - 1]);
                        fixed.push_str(plain);
                        prev = h.start as usize + 1;
                        pieces.push(StrPiece::Text(plain.to_string()));
                        continue;
                    }
                    fixed.push_str(&text[prev..h.start as usize]);
                    fixed.push_str(&arg_text);
                    fixed.push_str(inner);
                    prev = h.start as usize + inner.len();
                    pieces.push(match inner.strip_prefix(':').and_then(FmtSpec::parse) {
                        Some(spec) => StrPiece::Spec(arg, spec),
                        None => StrPiece::Expr(arg),
                    });
                }
            }
        }
        fixed.push_str(&text[prev..]);
        diag = diag.with_help(habits::WRITE.say(&[&fixed])).with_fix(
            habits::WRITE.say(&[&fixed]),
            span,
            fixed,
        );
        self.recovered(diag);
        Some(ExprKind::Str(pieces))
    }

    /// `let a, b = t`, `for i, x in ...`: Python's unpacking without parentheses. One error
    /// with the fix that adds them, read as the tuple pattern.
    pub(in crate::parser) fn bare_tuple_pattern(&mut self, first: Pattern, end: T) -> Pattern {
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
        let mut diag = cpl_diag::msgs::foreign::BARE_TUPLE_PATTERN
            .error(span, &[])
            .with_help(habits::BARE_TUPLE_PATTERN.say(&[&self.item_text(span)]));
        diag.fixes.push(cpl_diag::Fix {
            title: habits::BARE_TUPLE_PATTERN_2.say(&[]),
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

    /// `sorted(xs, reverse=True)`: Python's keyword argument. One error
    /// with the `:` fix, read as the named argument.
    pub(in crate::parser) fn keyword_arg(&mut self) -> Name {
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
            cpl_diag::msgs::foreign::KEYWORD_ARG
                .error(self.rel(eq), &[])
                .with_help(habits::KEYWORD_ARG.say(&[]))
                .with_fix(habits::WRITE.say(&[&":"]), gap, ": "),
        );
        let name = self.name();
        self.bump();
        name
    }

    /// One index in `xs[...]`; Python's slice `xs[a:b]` is one error with
    /// the `xs[a..b]` fix, read as that range.
    pub(in crate::parser) fn index_item(&mut self) -> Expr {
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
            cpl_diag::msgs::foreign::SLICE_COLON
                .error(colon, &[])
                .with_help(habits::INDEX_ITEM.say(&[]))
                .with_fix(habits::INDEX_ITEM_2.say(&[]), colon, ".."),
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

    /// `List[Int]`, Python's generic brackets after any type name: read as
    /// `List<Int>`, with one error that turns the brackets into `<>`.
    pub(in crate::parser) fn bracket_type_args(&mut self, name: &str) -> Vec<Type> {
        if !self.at(T::LBracket) {
            return Vec::new();
        }
        let open = self.span();
        self.bump();
        let mut args = Vec::new();
        while !self.at(T::RBracket) && !self.at_line_end() {
            args.push(self.ty());
            if !self.eat(T::Comma) {
                break;
            }
        }
        let close = self.span();
        if !self.eat(T::RBracket) {
            return args;
        }
        let mut diag = msgs::syntax::TYPE_ARGS_IN_ANGLES
            .error(open, &[&name, &name])
            .with_help(habits::BRACKET_TYPE_ARGS.say(&[]));
        diag.fixes.push(cpl_diag::Fix {
            title: habits::WRITE.say(&[&"<...>"]),
            edits: vec![(open, "<".into()), (close, ">".into())],
        });
        self.error(diag);
        args
    }
}

/// `n, m = map(int, input().split())` as `let (n, m): (Int, Int) = read()`.
fn python_read(lhs: &str, rhs: &str) -> Option<String> {
    let rest = rhs.trim().strip_prefix("map(")?;
    let (f, rest) = rest.split_once(',')?;
    let ty = match f.trim() {
        "int" => "Int",
        "float" => "F64",
        _ => return None,
    };
    let line = rest.trim().strip_prefix("input().split(")?;
    if !matches!(line, "))" | "' '))" | "\" \"))") {
        return None;
    }
    let names: Vec<&str> = lhs.split(',').map(str::trim).collect();
    let plain = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_');
    if names.len() < 2 || !names.iter().all(|n| plain(n)) {
        return None;
    }
    let tys = vec![ty; names.len()].join(", ");
    Some(format!("let ({}): ({tys}) = read()", names.join(", ")))
}
