//! Rust's habits: print macros, `[x; n]`, `|x|` closures and struct literals.

use super::*;

impl<'a> Parser<'a> {
    /// `println!(...)`, Rust's print macros.
    pub(in crate::parser) fn rust_print_ahead(&self) -> bool {
        matches!(self.text(), "println" | "print" | "eprintln")
            && self.peek_n(1) == T::Bang
            && self.peek_n(2) == T::LParen
    }

    /// `println!("{} {}", a, b)`: one error, with `print(a, b)` when the
    /// format string only places each value. The arguments are skipped,
    /// not parsed: Rust's `{}` is not an interpolation.
    pub(in crate::parser) fn rust_print(&mut self) -> ExprKind {
        let start = self.start();
        let name = self.text();
        self.bump();
        self.bump();
        self.bump();
        let mut args: Vec<(usize, usize)> = Vec::new();
        let mut first = None;
        let mut depth = 0usize;
        loop {
            match self.peek() {
                T::Newline | T::Eof => break,
                T::RParen if depth == 0 => break,
                T::Comma if depth == 0 => {
                    args.extend(first.take().map(|f| (f, self.pos - 1)));
                    self.bump();
                    continue;
                }
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            first.get_or_insert(self.pos);
            self.bump();
        }
        args.extend(first.map(|f| (f, self.pos - 1)));
        self.expect_token(T::RParen, "`)` after the arguments");
        let span = self.span_from(start);
        let texts: Vec<&str> = args
            .iter()
            .map(|&(a, b)| {
                Span::new(self.tokens[a].span.start, self.tokens[b].span.end).text(self.src)
            })
            .collect();
        let mut diag = cpl_diag::msgs::foreign::RUST_PRINT
            .error(span, &[&name])
            .with_help(habits::RUST_PRINT.say(&[]));
        if let ("println", Some(new)) = (name, Self::print_for(&texts)) {
            diag = diag.with_fix(habits::WRITE.say(&[&new]), span, new);
        }
        self.recovered(diag);
        ExprKind::Error
    }

    /// The `print` call for `println!`'s arguments, if its format string
    /// only places them: `"{}"`, `"{} {}"`, or a string with no `{}`.
    fn print_for(args: &[&str]) -> Option<String> {
        let (format, values) = args.split_first()?;
        let inner = format.strip_prefix('"')?.strip_suffix('"')?;
        if values.is_empty() && !inner.contains("{}") && !inner.contains("{:") {
            return Some(format!("print({format})"));
        }
        let placed = inner.split(' ').all(|piece| piece == "{}");
        let count = inner.split(' ').count();
        (placed && count == values.len()).then(|| format!("print({})", values.join(", ")))
    }

    /// At the `;` of Rust's `[x; n]`: a fill of the typed `[n]T`,
    /// `[n].filled(x)` (R01.11.1). `[x] * n` is not offered while the
    /// checker rejects `List * Int` (compiler worklog).
    pub(in crate::parser) fn rust_repeat(&mut self, items: Vec<Expr>) -> ExprKind {
        let semi = self.span();
        self.bump();
        let count = self.expr();
        let close = self.span();
        let mut diag = cpl_diag::msgs::foreign::RUST_REPEAT
            .error(semi, &[])
            .with_help(habits::RUST_REPEAT.say(&[]));
        if let ([item], true) = (items.as_slice(), self.at(T::RBracket)) {
            let item_text = self.item_text(item.span);
            if !item_text.contains(';') {
                diag.fixes.push(cpl_diag::Fix {
                    title: habits::RUST_REPEAT_2.say(&[&self.item_text(count.span), &item_text]),
                    edits: vec![
                        (
                            Span::new(item.span.start, count.span.end),
                            self.item_text(count.span),
                        ),
                        (Span::point(close.end), format!(".filled({item_text})")),
                    ],
                });
            }
        }
        self.error(diag);
        self.eat(T::RBracket);
        ExprKind::List(items)
    }

    /// `P { x: 1 }` after a type's name: Rust's struct literal.
    pub(in crate::parser) fn struct_literal_ahead(&self, callee: &Expr) -> bool {
        let fields = self.peek_n(1) == T::RBrace
            || (self.peek_n(1) == T::Ident && self.peek_n(2) == T::Colon);
        fields
            && matches!(&callee.kind, ExprKind::Name(n) if n.starts_with(|c: char| c.is_ascii_uppercase()))
    }

    /// `P { x: 1 }`, read as the call CP Lang builds a struct with, `P(x: 1)`.
    pub(in crate::parser) fn struct_literal(&mut self, callee: Expr) -> ExprKind {
        let open = self.span();
        self.bump();
        let mut args = Vec::new();
        while self.at(T::Ident) && self.peek_n(1) == T::Colon {
            let name = self.name();
            self.bump();
            let value = self.expr();
            args.push(Arg {
                name: Some(name),
                mutable: false,
                value,
            });
            if !self.eat(T::Comma) {
                break;
            }
        }
        let close = self.span();
        if self.expect_token(T::RBrace, "`}` after the struct's fields") {
            let first = args
                .first()
                .and_then(|a: &Arg| a.name.as_ref())
                .map(|n| n.span.start);
            let last = args.last().map_or(close.start, |a| a.value.span.end);
            let fields = first.map_or(String::new(), |f| self.item_text(Span::new(f, last)));
            let mut diag = cpl_diag::msgs::foreign::STRUCT_LITERAL
                .error(Span::new(open.start, close.end), &[])
                .with_help(habits::STRUCT_LITERAL.say(&[&self.item_text(callee.span), &fields]));
            let open = Span::new(callee.span.end, first.unwrap_or(last));
            diag.fixes.push(cpl_diag::Fix {
                title: habits::STRUCT_LITERAL_2.say(&[]),
                edits: vec![(open, "(".into()), (Span::new(last, close.end), ")".into())],
            });
            self.recovered(diag);
        }
        ExprKind::Call {
            callee: Box::new(callee),
            args,
        }
    }

    /// `|x| x + 1` or `|a, b| a + b`: Rust's closure. One error with the
    /// `=>` fix, read as the lambda.
    pub(super) fn rust_closure(&mut self) -> Option<Expr> {
        if !self.at(T::Pipe) {
            return None;
        }
        let (mut n, mut names) = (1, Vec::new());
        loop {
            let t = self.tokens.get(self.pos + n)?;
            if t.kind != T::Ident {
                return None;
            }
            names.push(t.span.text(self.src).to_string());
            match self.tokens.get(self.pos + n + 1)?.kind {
                T::Comma => n += 2,
                T::Pipe => break,
                _ => return None,
            }
        }
        let start = self.start();
        let bars = Span::new(self.span().start, self.span_n(n + 1).end);
        self.bump();
        let mut params = Vec::new();
        while self.at(T::Ident) {
            params.push(self.pattern_atom());
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.bump();
        let ours = match &names[..] {
            [one] => format!("{one} =>"),
            _ => format!("({}) =>", names.join(", ")),
        };
        self.recovered(
            cpl_diag::msgs::foreign::RUST_CLOSURE
                .error(bars, &[])
                .with_help(habits::RUST_CLOSURE.say(&[]))
                .with_fix(habits::WRITE.say(&[&ours]), bars, ours),
        );
        let body = self.lambda_body();
        Some(Expr {
            kind: ExprKind::Lambda {
                params,
                param_tys: Vec::new(),
                param_mut: Vec::new(),
                ret_ty: None,
                body: Box::new(body),
            },
            span: self.span_from(start),
        })
    }
}
