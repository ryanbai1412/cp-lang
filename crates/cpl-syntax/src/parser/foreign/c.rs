//! C's and C++'s habits: declarations, `auto`, `&`, casts, `*`, `?:`, `npos`,
//! temporaries, `for (;;)`, range `for`, `while (n--)`, `case` and `new`.

use super::*;

impl<'a> Parser<'a> {
    /// `int x;`, `long long s = 0;` or `vector<int> a(n);` as a whole
    /// line: C++'s declaration, type first. The line's span, the type's
    /// span and text, and the line as `let` or `var`.
    pub(in crate::parser) fn c_decl(&self) -> Option<(Span, Span, &'a str, String)> {
        let (begin, line) = self.this_line()?;
        let whole = line.strip_suffix(';').unwrap_or(line).trim_end();
        let (ty_len, decl) = match whole.split_once(';') {
            Some((decl, rest)) => {
                let (len, decl) = c_declaration(decl.trim_end())?;
                let indent = &self.src[self.src[..begin].rfind('\n').map_or(0, |i| i + 1)..begin];
                let decl = c_declared_read(&decl, rest)
                    .unwrap_or_else(|| format!("{decl}\n{indent}{};", rest.trim()));
                (len, decl)
            }
            None => c_declaration(whole)?,
        };
        // `long long a, b;` then `cin >> a >> b;` on the next line: one read.
        let mut end = begin + line.len();
        let next = self
            .src
            .get(end + 1..)
            .map(|r| r.split('\n').next().unwrap_or(r));
        let (decl, end) = match next.and_then(|n| Some((n, c_declared_read(&decl, n)?))) {
            Some((n, read))
                if n.len() - n.trim_start().len() == begin - line_start(self.src, begin) =>
            {
                end += 1 + n.trim_end().len();
                (read, end)
            }
            _ => (decl, end),
        };
        let at = |from: usize, to: usize| self.rel(Span::new(from as u32, to as u32));
        let written = &line[..ty_len];
        Some((at(begin, end), at(begin, begin + ty_len), written, decl))
    }

    /// `auto x = 0`, `int x = 0` or `auto (a, b) = ..`: a C++ keyword
    /// where `let` goes.
    pub(in crate::parser) fn c_auto_ahead(&self) -> bool {
        matches!(
            self.text(),
            "auto" | "int" | "long" | "double" | "bool" | "string" | "char"
        ) && (self.peek_n(1) == T::Ident && matches!(self.peek_n(2), T::Eq | T::Colon)
            || self.text() == "auto" && matches!(self.peek_n(1), T::LParen | T::LBracket))
    }

    /// One error, then the line is read as `var`: a C++ variable may change.
    pub(in crate::parser) fn c_auto(&mut self) -> StmtKind {
        let kw = self.span();
        self.error(
            cpl_diag::msgs::foreign::LET_KEYWORD
                .error(kw, &[&self.text()])
                .with_help(habits::C_AUTO.say(&[]))
                .with_fix(habits::WRITE.say(&[&"let"]), kw, "let"),
        );
        let mut stmt = self.let_stmt();
        if let StmtKind::Let { mutable, .. } = &mut stmt {
            *mutable = true;
        }
        stmt
    }

    /// `cin >> a >> b;` under `long long a, b;`, which the declaration's
    /// fix already reads: the line's span, so it gets no error of its own.
    pub(in crate::parser) fn skip_read_after_c_decl(&mut self) -> bool {
        let Some(line) = self.read_after_c_decl() else {
            return false;
        };
        self.sink.poison(line);
        self.skip_line();
        self.eat(T::Newline);
        true
    }

    fn read_after_c_decl(&self) -> Option<Span> {
        let (begin, line) = self.this_line()?;
        let start = line_start(self.src, begin);
        let prev_start = line_start(self.src, start.checked_sub(1)?);
        let prev = &self.src[prev_start..start - 1];
        let indent = prev.len() - prev.trim_start().len();
        let prev = prev.trim();
        let (_, decl) = c_declaration(prev.strip_suffix(';').unwrap_or(prev).trim_end())?;
        (indent == begin - start && c_declared_read(&decl, line).is_some())
            .then(|| self.rel(Span::new(begin as u32, (begin + line.len()) as u32)))
    }

    /// The one error for a C++ declaration line, its fix the whole line.
    pub(in crate::parser) fn c_decl_error(&self) -> Option<Diagnostic> {
        let (at, ty, written, decl) = self.c_decl()?;
        Some(
            cpl_diag::msgs::foreign::C_DECL_TYPE
                .error(ty, &[&written])
                .with_help(habits::C_DECL_ERROR.say(&[]))
                .with_fix(habits::WRITE.say(&[&decl]), at, decl.clone()),
        )
    }

    /// `new P(x: 1)`: Java's and C++'s `new`. One error with the fix that
    /// drops it; the call after it is read as written.
    pub(in crate::parser) fn foreign_new(&mut self) {
        let Some(next) = self.tokens.get(self.pos + 1).map(|t| t.span) else {
            return;
        };
        let upper = self.peek_n(1) == T::Ident
            && next
                .text(self.src)
                .starts_with(|c: char| c.is_ascii_uppercase());
        if !(self.at_ident("new") && upper && self.peek_n(2) == T::LParen) {
            return;
        }
        let (kw, name) = (self.span(), self.span_n(1));
        self.bump();
        self.recovered(
            cpl_diag::msgs::foreign::FOREIGN_NEW
                .error(kw, &[])
                .with_help(habits::FOREIGN_NEW.say(&[&next.text(self.src)]))
                .with_fix(
                    habits::FOREIGN_NEW_2.say(&[]),
                    Span::new(kw.start, name.start),
                    "",
                ),
        );
    }

    /// `*it`, `*max_element(..)`: C's dereference. One error with the fix
    /// that drops the `*`; the value after it is read as written.
    pub(in crate::parser) fn c_deref(&mut self) {
        let star = self.span();
        if self.at(T::Amp) {
            return self.c_address(star);
        }
        let prev = self.pos.checked_sub(1).map(|i| self.tokens[i].kind);
        if matches!(prev, Some(T::LParen | T::Comma)) {
            return self.python_splat(star);
        }
        self.bump();
        self.recovered(
            cpl_diag::msgs::foreign::C_DEREF
                .error(star, &[])
                .with_help(habits::C_DEREF.say(&[]))
                .with_fix(habits::C_DEREF.say(&[]), star, ""),
        );
    }

    /// `swap(&a, &b)`, `scanf("%d", &n)`: C's address-of. In `scanf` the
    /// checker rewrites the whole call, so the `&` goes without an error.
    fn c_address(&mut self, amp: Span) {
        let scanf = self.callee() == Some("scanf");
        self.bump();
        if !scanf {
            self.recovered(
                cpl_diag::msgs::foreign::C_ADDRESS
                    .error(amp, &[])
                    .with_help(habits::C_ADDRESS.say(&[]))
                    .with_fix(habits::C_ADDRESS_2.say(&[]), amp, ""),
            );
        }
    }

    /// `swap(&a, &b)`: C++'s reference argument, which CP Lang writes
    /// `mut a`. Read as `mut`; in `scanf` the checker rewrites the call.
    pub(in crate::parser) fn c_ref_arg(&mut self) -> bool {
        if !self.at(T::Amp) || self.peek_n(1) != T::Ident {
            return false;
        }
        let amp = self.span();
        let scanf = self.callee() == Some("scanf");
        self.bump();
        if !scanf {
            let next = self.span();
            self.recovered(
                cpl_diag::msgs::foreign::C_REF_ARG
                    .error(amp, &[])
                    .with_help(habits::CHANGES.say(&[&self.tokens[self.pos].span.text(self.src)]))
                    .with_fix(
                        habits::GETS.say(&[]),
                        Span::new(amp.start, next.start),
                        "mut ",
                    ),
            );
        }
        true
    }

    /// `(long long)n`, `(double)x`: a C cast. One error, with the
    /// conversion call: `Int(n)`, `F64(x)`.
    pub(in crate::parser) fn c_cast(&mut self) -> Option<Expr> {
        const INTS: [&str; 10] = [
            "int", "long", "ll", "unsigned", "signed", "short", "size_t", "int64_t", "i64", "lld",
        ];
        const FLOATS: [&str; 3] = ["double", "float", "ld"];
        let (mut n, mut float) = (1, false);
        while self.peek_n(n) == T::Ident {
            let word = self.tokens[self.pos + n].span.text(self.src);
            if !INTS.contains(&word) && !FLOATS.contains(&word) {
                return None;
            }
            float |= FLOATS.contains(&word);
            n += 1;
        }
        let operand_next = matches!(self.peek_n(n + 1), T::Ident | T::Int | T::Float | T::LParen);
        if n == 1 || self.peek_n(n) != T::RParen || !operand_next {
            return None;
        }
        let cast = Span::new(self.span().start, self.span_n(n).end);
        let group = (self.peek_n(n + 1) == T::LParen)
            .then(|| super::after_group(self.tokens, self.pos + n + 1));
        for _ in 0..=n {
            self.bump();
        }
        let operand = self.postfix();
        let wrapped = group == Some(self.pos);
        let (ty, help) = if float {
            ("F64", habits::C_CAST_FLOAT.help)
        } else {
            ("Int", habits::C_CAST_INT.help)
        };
        let edits = if wrapped {
            vec![(cast, ty.to_string())]
        } else {
            let close = (Span::point(operand.span.end), ")".to_string());
            vec![(cast, format!("{ty}(")), close]
        };
        let title = habits::C_CAST.say(&[&ty]);
        let mut diag = cpl_diag::msgs::foreign::C_CAST
            .error(cast, &[])
            .with_help(help);
        diag.fixes.push(cpl_diag::Fix {
            title: title.to_string(),
            edits,
        });
        self.recovered(diag);
        Some(operand)
    }

    /// C++'s `string::npos`: CP Lang's `find` gives `None` where C++ gives `npos`.
    pub(in crate::parser) fn cpp_npos(&mut self) -> Option<Expr> {
        let text = |k: usize| self.tokens.get(self.pos + k).map(|t| t.span.text(self.src));
        let len = match [0, 1, 2, 3, 4].map(text) {
            [Some("string"), Some("::"), Some("npos"), ..] => 3,
            [Some("std"), Some("::"), Some("string"), Some("::"), Some("npos")] => 5,
            _ => return None,
        };
        let (first, last) = (
            self.tokens[self.pos].span,
            self.tokens[self.pos + len - 1].span,
        );
        let at = self.rel(Span::new(first.start, last.end));
        self.recovered(
            cpl_diag::msgs::foreign::CPP_NPOS
                .error(at, &[])
                .with_help(habits::CPP_NPOS.say(&[]))
                .with_fix(habits::WRITE.say(&[&"None"]), at, "None"),
        );
        for _ in 0..len {
            self.bump();
        }
        Some(Expr {
            kind: ExprKind::Name("None".into()),
            span: at,
        })
    }

    /// Whether the next token continues a line that `target` is on,
    /// rather than starting a line of its own. A target that ends with a
    /// block (`if c:` and its body) is followed by the next statement
    /// with no line break token between them.
    /// `c ? a : b` from C, after the condition: inline `if` here.
    pub(in crate::parser) fn c_ternary_ahead(&self, cond: &Expr) -> bool {
        let rest = self.tokens.get(self.pos + 1..).unwrap_or_default();
        let mut line = rest.iter().take_while(|t| t.kind != T::Newline);
        self.at(T::Question)
            && self.span().start > cond.span.end
            && !matches!(self.peek_n(1), T::RParen | T::RBracket | T::Comma | T::Dot)
            && line.any(|t| t.kind == T::Colon)
    }

    /// Reads `c ? a : b` as `if c: a else: b`, with one error that says so
    /// and rewrites it, so the rest of the program sees an inline `if`.
    pub(in crate::parser) fn c_ternary(&mut self, start: u32, first: usize, cond: Expr) -> Expr {
        let question = self.pos;
        let q = self.span();
        self.bump();
        let yes = self.coalesce();
        let colon = self.pos;
        let complete = self.eat(T::Colon);
        let no = self.coalesce();
        let span = self.span_from(start);
        let text = |p: &Self, a: usize, b: usize| {
            let (a, b) = (p.tokens[a].span.start, p.tokens[b - 1].span.end);
            p.src[a as usize..b as usize].to_string()
        };
        let mut diag = cpl_diag::msgs::foreign::C_TERNARY
            .error(q, &[])
            .with_help(habits::C_TERNARY.say(&[]));
        if complete && self.pos > colon + 1 && colon > question + 1 {
            let (c, a, b) = (
                text(self, first, question),
                text(self, question + 1, colon),
                text(self, colon + 1, self.pos),
            );
            let whole = self.rel(Span::new(
                self.tokens[first].span.start,
                self.tokens[self.pos - 1].span.end,
            ));
            diag = diag.with_fix(
                habits::C_TERNARY_2.say(&[]),
                whole,
                format!("if {c}: {a} else: {b}"),
            );
        }
        self.error(diag);
        Self::inline_if(cond, yes, no, span)
    }

    /// C++'s `vector<int>(n)` as a value: one error, fixed to `List.filled(n, 0)`.
    pub(in crate::parser) fn cpp_temporary(&mut self) -> Option<Expr> {
        if self.peek_n(1) != T::Lt {
            return None;
        }
        let start = self.tokens[self.pos].span.start;
        let (len, value) = c_temporary(&self.src[start as usize..])?;
        let at = self.rel(Span::new(start, start + len as u32));
        let written = &self.src[start as usize..start as usize + len];
        self.recovered(
            cpl_diag::msgs::foreign::CPP_VALUE
                .error(at, &[&written, &value])
                .with_help(habits::WRITE.say(&[&value]))
                .with_fix(habits::WRITE.say(&[&value]), at, value),
        );
        while self
            .tokens
            .get(self.pos)
            .is_some_and(|t| t.span.start < start + len as u32)
        {
            self.bump();
        }
        Some(Expr {
            kind: ExprKind::Error,
            span: at,
        })
    }

    /// `for (i = 0; i < n; i++)`: C's loop header. One error, with a fix
    /// when it counts up by one, read as `for i in <error>` so the body
    /// still checks.
    pub(in crate::parser) fn c_for(&mut self) -> Option<(Pattern, Expr)> {
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
        let mut diag = cpl_diag::msgs::foreign::C_FOR
            .error(whole, &[])
            .with_help(habits::C_FOR.say(&[]));
        let (name, range) = counted(&parts);
        // C's body with no braces, on the line or under it: the fix adds the `:`.
        let colon = match self.tokens.get(end).map(|t| t.kind) {
            Some(T::Colon | T::LBrace) => "",
            _ => ":",
        };
        if let (Some(name), Some(range)) = (name, range) {
            let ours = format!("{name} in {range}{colon}");
            diag = diag.with_fix(habits::C_FOR_2.say(&[]), whole, ours);
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

    /// C++'s `for (auto x : a)` or `for (auto& [u, v] : e)`: `for x in a:`.
    pub(in crate::parser) fn c_range_for(&mut self) -> Option<(Pattern, Expr)> {
        let open = self.pos;
        let end = after_group(self.tokens, open);
        let inner = self.tokens.get(open + 1..end.checked_sub(1)?)?;
        let mut depth = 0;
        let at = inner.iter().position(|t| {
            match t.kind {
                T::LParen | T::LBracket => depth += 1,
                T::RParen | T::RBracket => depth -= 1,
                _ => {}
            }
            depth == 0 && t.kind == T::Colon
        })?;
        let left = &inner[..at];
        let typed = left.len() >= 2 && matches!(left[0].kind, T::Ident | T::Const);
        if !self.at(T::LParen) || !typed || inner.iter().any(|t| t.kind == T::Semi) {
            return None;
        }
        let names: Vec<&str> = match left.last()?.kind {
            T::Ident => vec![left.last()?.span.text(self.src)],
            T::RBracket => {
                let from = left.iter().rposition(|t| t.kind == T::LBracket)?;
                (left[from..].iter().filter(|t| t.kind == T::Ident))
                    .map(|t| t.span.text(self.src))
                    .collect()
            }
            _ => return None,
        };
        let by_ref = left.iter().any(|t| matches!(t.kind, T::Amp | T::And))
            && !left.iter().any(|t| t.kind == T::Const)
            && self.body_assigns(end, &names);
        let (a, b) = (self.tokens[open].span.start, self.tokens[end - 1].span.end);
        let seq = self.src[inner[at].span.end as usize..b as usize - 1].trim();
        let pat = match names.as_slice() {
            [one] => one.to_string(),
            many => format!("({})", many.join(", ")),
        };
        let colon = match self.tokens.get(end).map(|t| t.kind) {
            Some(T::Colon | T::LBrace) => "",
            _ => ":",
        };
        let whole = self.rel(Span::new(a, b));
        let diag = cpl_diag::msgs::foreign::TYPED_FOR.error(whole, &[]);
        self.recovered(match by_ref {
            false => diag
                .with_help(habits::C_RANGE_FOR.say(&[&pat, &seq]))
                .with_fix(
                    habits::C_RANGE_FOR_2.say(&[]),
                    whole,
                    format!("{pat} in {seq}{colon}"),
                ),
            true => {
                let diag = diag.with_help(habits::C_RANGE_FOR_3.say(&[&seq, &seq]));
                match self.by_index(end, &names, seq, whole) {
                    Some(edits) => diag.with_edits("loop over the indexes", edits),
                    None => diag,
                }
            }
        });
        self.pos = end;
        let one = |n: &str| Pattern {
            kind: PatternKind::Name(n.to_string()),
            span: whole,
        };
        let kind = match names.as_slice() {
            [n] => PatternKind::Name(n.to_string()),
            many => PatternKind::Tuple(many.iter().map(|n| one(n)).collect()),
        };
        // `x += 1` through a C++ reference is reported once, above.
        let kind = match by_ref {
            true => PatternKind::Var(Box::new(Pattern { kind, span: whole })),
            false => kind,
        };
        let pat = Pattern { kind, span: whole };
        let iter = Expr {
            kind: ExprKind::Error,
            span: whole,
        };
        Some((pat, iter))
    }

    /// C++'s `greater<>()`: CP Lang infers type arguments, or they are written.
    pub(in crate::parser) fn empty_type_args(&mut self) {
        if self.at(T::Lt) && self.peek_n(1) == T::Gt && self.peek_n(2) == T::LParen {
            let lo = self.span();
            self.bump();
            let at = Span::new(lo.start, self.span().end);
            self.bump();
            self.error(
                cpl_diag::msgs::foreign::EMPTY_TYPE_ARGS
                    .error(at, &[])
                    .with_help(habits::EMPTY_TYPE_ARGS.say(&[]))
                    .with_fix(habits::EMPTY_TYPE_ARGS_2.say(&[]), at, ""),
            );
        }
    }

    /// Whether the loop body starting at token `from` (braced, on the line, or
    /// indented under it) assigns one of `names`, as `x += 1` or `cin >> x`.
    /// `for (auto &x : a) cin >> x;`: each `x` in the body becomes
    /// `a[i]`, and the header `i in 0..a.len:`.
    fn by_index(
        &self,
        from: usize,
        names: &[&str],
        seq: &str,
        header: Span,
    ) -> Option<Vec<(Span, String)>> {
        let [name] = names else {
            return None;
        };
        let toks = self.tokens;
        let simple = |s: &str| s.chars().all(|c| c.is_alphanumeric() || "_.".contains(c));
        let colon = match toks.get(from)?.kind {
            T::LBrace | T::Colon => "",
            _ => ":",
        };
        if !simple(seq) {
            return None;
        }
        let body = &toks[from..self.body_end(from)];
        let used = |n: &str| {
            body.iter()
                .any(|t| t.kind == T::Ident && t.span.text(self.src) == n)
        };
        let i = ["i", "j", "k", "idx"]
            .into_iter()
            .find(|n| !used(n) && n != name && *n != seq)?;
        let mut edits = vec![(header, format!("{i} in 0..{seq}.len{colon}"))];
        for (k, t) in body.iter().enumerate() {
            let after_dot = k > 0 && body[k - 1].kind == T::Dot;
            if t.kind == T::Ident && t.span.text(self.src) == *name && !after_dot {
                edits.push((self.rel(t.span), format!("{seq}[{i}]")));
            }
        }
        Some(edits)
    }

    fn body_assigns(&self, from: usize, names: &[&str]) -> bool {
        let toks = self.tokens;
        let end = self.body_end(from);
        let text = |i: usize| toks.get(i).map_or("", |t| t.span.text(self.src));
        (from..end).any(|i| {
            toks[i].kind == T::Ident
                && names.contains(&text(i))
                && (matches!(text(i + 1), "=" | "++" | "--" | "<<=" | ">>=")
                    || (text(i + 1).ends_with('=')
                        && !matches!(text(i + 1), "==" | "<=" | ">=" | "!="))
                    || text(i.wrapping_sub(1)) == ">>")
        })
    }

    /// Whether the braces opening at `open`, closed just before the cursor,
    /// hold two values and are a call's only argument: `m.insert({k, v})`.
    pub(in crate::parser) fn pair_arg(&self, open: usize) -> bool {
        let kind = |i: Option<usize>| i.and_then(|i| self.tokens.get(i)).map(|t| t.kind);
        let mut depth = 0;
        let commas = self.tokens[open + 1..self.pos - 1]
            .iter()
            .filter(|t| {
                match t.kind {
                    T::LParen | T::LBracket | T::LBrace => depth += 1,
                    T::RParen | T::RBracket | T::RBrace => depth -= 1,
                    _ => {}
                }
                depth == 0 && t.kind == T::Comma
            })
            .count();
        commas == 1
            && kind(open.checked_sub(1)) == Some(T::LParen)
            && kind(open.checked_sub(2)) == Some(T::Ident)
            && kind(Some(self.pos)) == Some(T::RParen)
    }

    /// `while (n--)` or `while n--:`, C's count-down loop, after the `while`: the header's
    /// span, the counter, the `:` a one-line body needs, and where the body
    /// starts. `c_countdown_report` reports it once the body is read.
    pub(in crate::parser) fn c_countdown(
        &mut self,
    ) -> Option<(Span, &'a str, &'static str, usize)> {
        let parens = [T::LParen, T::Ident, T::Minus, T::Minus, T::RParen];
        let bare = [T::Ident, T::Minus, T::Minus];
        let is = |kinds: &[T]| {
            let toks = self.tokens.get(self.pos..self.pos + kinds.len());
            toks.is_some_and(|toks| toks.iter().zip(kinds).all(|(t, k)| t.kind == *k))
        };
        let len = if is(&parens) {
            5
        } else if is(&bare) && matches!(self.peek_n(3), T::Colon | T::LBrace) {
            3
        } else {
            return None;
        };
        let toks = &self.tokens[self.pos..self.pos + len];
        let whole = Span::new(self.tokens[self.pos - 1].span.start, toks[len - 1].span.end);
        let name = toks[usize::from(len == 5)].span.text(self.src);
        self.pos += len;
        let colon = match self.peek() {
            T::Colon | T::LBrace => "",
            _ => ":",
        };
        Some((whole, name, colon, self.pos))
    }

    /// One error for `while (n--)`. The fix is `for _ in 0..n` when the body
    /// never reads `n`; a body that does sees it counted down, so it gets help.
    pub(in crate::parser) fn c_countdown_report(&mut self, header: (Span, &str, &str, usize)) {
        let (whole, name, colon, body) = header;
        let reads = self.tokens[body..self.pos]
            .iter()
            .any(|t| t.kind == T::Ident && t.span.text(self.src) == name);
        let whole = self.rel(whole);
        let diag = cpl_diag::msgs::foreign::C_DECREMENT_LOOP.error(whole, &[&name, &name]);
        self.recovered(match reads {
            false => diag
                .with_help(habits::C_COUNTDOWN_REPORT.say(&[&name]))
                .with_fix(
                    habits::C_FOR_2.say(&[]),
                    whole,
                    format!("for _ in 0..{name}{colon}"),
                ),
            true => diag.with_help(habits::C_COUNTDOWN_REPORT_2.say(&[&name, &name, &name])),
        });
    }

    /// `case 1:` or `default:` opening a `match` arm, as a `switch` writes
    /// it: one error, read as the arm `1:` or `default:` (a catch-all).
    pub(in crate::parser) fn c_case(&mut self) {
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
            cpl_diag::msgs::foreign::C_CASE
                .error(span, &[&theirs])
                .with_help(habits::C_CASE.say(&[&ours]))
                .with_fix(habits::C_CASE_2.say(&[&theirs]), span, fix),
        );
    }

    /// `&&` or `||` from C: reported with the word to write, then read as
    /// that word. Leaves the second character for [`Self::binary`].
    pub(in crate::parser) fn c_logic(&mut self, half: TokenKind, word: &str) -> bool {
        if !self.doubled(half) {
            return false;
        }
        let span = Span::new(self.start(), self.span_n(1).end);
        let op = if half == T::Amp { "&&" } else { "||" };
        self.error(
            msgs::syntax::C_LOGIC_OP
                .error(span, &[&op, &word])
                .with_fix(habits::REPLACE_WITH.say(&[&word]), span, word),
        );
        self.bump();
        true
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
