//! Statements and blocks.

use super::*;

impl<'a> Parser<'a> {
    /// Parses one statement into `out`. An unexpectedly indented run of
    /// lines is reported once and its statements are kept at this level.
    pub(super) fn stmt_into(&mut self, out: &mut Vec<Stmt>) {
        let continues = |k: TokenKind| {
            matches!(k, T::Dot | T::QuestionQuestion | T::And | T::Or)
                || super::expr::BINARY_OPS.contains(&k)
        };
        if self.at(T::Indent) && continues(self.peek_n(1)) {
            let start = self.start();
            self.skip_indented();
            let span = self.span_from(start);
            self.planned(span, &crate::planned::CONTINUED_LINE);
            return;
        }
        if self.at(T::Indent) {
            let span = Span::point(self.span().start);
            self.error(
                Diagnostic::error(
                    codes::UNEXPECTED_INDENT,
                    span,
                    "this line is indented, but no block starts above it",
                )
                .with_help("remove the extra indentation, or end the line above with `:`"),
            );
            self.dedent_fix();
            self.recovering = false;
            self.bump();
            while !matches!(self.peek(), T::Dedent | T::Eof) {
                let before = self.pos;
                self.stmt_into(out);
                if self.pos == before {
                    self.bump();
                }
            }
            self.eat(T::Dedent);
            return;
        }
        if self.eat(T::Newline) {
            return;
        }
        let stmt = self.stmt();
        out.append(&mut self.lead_stmts);
        out.push(stmt);
        out.append(&mut self.extra_stmts);
    }

    /// Offers to move an unexpectedly indented run of lines back to the
    /// indentation of the line above it, keeping the run's own nesting.
    fn dedent_fix(&mut self) {
        let mut depth = 0;
        let mut end = None;
        for t in &self.tokens[self.pos..] {
            match t.kind {
                T::Indent => depth += 1,
                T::Dedent if depth == 1 => break,
                T::Dedent => depth -= 1,
                T::Newline => {}
                _ => end = Some(t.span.end),
            }
        }
        let (Some(first), Some(end)) = (self.tokens.get(self.pos + 1), end) else {
            return;
        };
        let src = self.src;
        let start = src[..first.span.start as usize]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let indent = |line: &str| line.len() - line.trim_start_matches(' ').len();
        let above = src[..start.saturating_sub(1)]
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with("//"))
            .map_or(0, indent);
        let Some(extra) = indent(&src[start..]).checked_sub(above).filter(|&e| e > 0) else {
            return;
        };
        let mut edits = Vec::new();
        let mut at = start;
        while at <= end as usize {
            let Some(line) = src.get(at..).and_then(|r| r.split('\n').next()) else {
                return;
            };
            let Ok(rel) = u32::try_from(at as i64 + self.delta) else {
                return;
            };
            if indent(line) >= extra {
                edits.push((Span::new(rel, rel + extra as u32), String::new()));
            }
            at += line.len() + 1;
        }
        let last = self.sink.last_mut();
        if let Some(diag) =
            last.filter(|d| d.code == codes::UNEXPECTED_INDENT && d.fixes.is_empty())
        {
            diag.fixes.push(cpl_diag::Fix {
                title: "remove the extra indentation".into(),
                edits,
            });
        }
    }

    fn stmt(&mut self) -> Stmt {
        let start = self.start();
        // `pub @get fn count(self)`: `pub` may come before the decorators.
        let mut public = self.at(T::Pub) && self.peek_n(1) == T::At;
        if public {
            self.bump();
        }
        let mut decorators = Vec::new();
        // `@trusted:` block (opt out of runtime checks in this block).
        if self.at(T::At)
            && self.peek_n(1) == T::Ident
            && self.peek_n(2) == T::Colon
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| t.span.text(self.src) == "trusted")
        {
            self.bump();
            self.bump();
            let block = self.colon_block("the `@trusted` block");
            self.end_stmt(start);
            return Stmt {
                kind: StmtKind::Unchecked(block),
                span: self.span_from(start),
                decorators,
                public,
            };
        }
        // `@law assoc(a, b, c) = op(op(a, b), c) == op(a, op(b, c))`.
        if self.at(T::At)
            && self.peek_n(1) == T::Ident
            && self.peek_n(2) == T::Ident
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| t.span.text(self.src) == "law")
        {
            self.bump();
            self.bump();
            let kind = self.law();
            self.end_stmt(start);
            return Stmt {
                kind,
                span: self.span_from(start),
                decorators,
                public,
            };
        }
        while self.at(T::At)
            || (self.at(T::Minus) && self.peek_n(1) == T::Minus && self.peek_n(2) == T::Minus)
        {
            if self.at(T::At) {
                decorators.push(self.decorator());
                if !self.eat(T::Newline)
                    && !self.at_line_end()
                    && decorators.last().is_some_and(|d| d.name.text.is_empty())
                {
                    break;
                }
            } else {
                decorators.push(self.wrap_line());
                self.eat(T::Newline);
            }
        }
        if self.eat(T::Pub) {
            public = true;
        }
        let grader = decorators.iter().any(|d| d.name.text == "grader");
        let was_in_grader = std::mem::replace(&mut self.in_grader, grader);
        let kind = self.stmt_kind();
        self.in_grader = was_in_grader;
        self.end_stmt(start);
        Stmt {
            kind,
            span: self.span_from(start),
            decorators,
            public,
        }
    }

    /// `--- multitest ---` or `--- with multitest() ---`: removed wrap-line
    /// spelling of `@!multitest` / `@!interactive`.
    fn wrap_line(&mut self) -> Decorator {
        let start = self.start();
        while self.eat(T::Minus) {}
        if self.at_ident("with") {
            self.bump();
        }
        let name = self.expect_name("a directive name in the `--- ... ---` wrap-line");
        let args = if self.eat(T::LParen) {
            self.comma_list(T::RParen, "the directive arguments", Self::arg)
        } else {
            Vec::new()
        };
        while self.eat(T::Minus) {}
        let span = self.span_from(start);
        if matches!(name.text.as_str(), "multitest" | "interactive") {
            let replacement = if args.is_empty() {
                format!("@!{}", name.text)
            } else {
                let rendered: Vec<&str> =
                    args.iter().map(|a| a.value.span.text(self.src)).collect();
                format!("@!{}({})", name.text, rendered.join(", "))
            };
            self.error(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    format!("use `{replacement}` instead of `{}`", span.text(self.src)),
                )
                .with_help(format!("replace with `{replacement}`"))
                .with_fix(
                    format!("replace with `{replacement}`"),
                    span,
                    replacement,
                ),
            );
        } else if !name.text.is_empty() {
            self.planned(span, &crate::planned::WRAP_LINE);
        }
        Decorator {
            name,
            args,
            directive: true,
            span,
        }
    }

    fn decorator(&mut self) -> Decorator {
        let start = self.start();
        self.bump();
        let directive = self.eat(T::Bang);
        let name = self.expect_name(if directive {
            "a directive name after `@!`"
        } else {
            "a decorator name after `@`"
        });
        let args = if self.eat(T::LParen) {
            self.comma_list(T::RParen, "the decorator's arguments", Self::arg)
        } else {
            Vec::new()
        };
        let span = self.span_from(start);
        if directive && !name.text.is_empty() && self.speculating == 0 {
            // The file-level directives: `@!multitest` divides the file
            // (decision 20.5), `@!interactive` writes each `print` out at once.
            let known = matches!(name.text.as_str(), "multitest" | "interactive");
            if !known {
                self.sink.push(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        format!("unknown directive `@!{}`", name.text),
                    )
                    .with_help(format!(
                        "the file-level directives are `@!multitest` and `@!interactive`; a decorator is `@{}`",
                        name.text
                    )),
                );
            } else if self.block_depth > 0 {
                let help = if name.text == "multitest" {
                    "move it out of the block: what is below it runs once per test case"
                } else {
                    "move it out of the block, to the top of the file: it holds for every `print`"
                };
                self.sink.push(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        format!("`@!{}` stands at the top level of the file", name.text),
                    )
                    .with_help(help),
                );
            } else if name.text == "interactive" && !args.is_empty() {
                self.sink.push(Diagnostic::error(
                    codes::UNEXPECTED_TOKEN,
                    span,
                    "`@!interactive` takes nothing",
                ));
            }
        }
        if !directive && name.text == "allow" && self.speculating == 0 {
            let lints: Vec<String> = args
                .iter()
                .map(|a| format!("{}: allow", a.value.span.text(self.src)))
                .collect();
            let with = format!("with lint({})", lints.join(", "));
            self.sink.push(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    "`@allow` was removed in CP Lang 0.3; lints are contexts",
                )
                .with_help(format!(
                    "write `{with}` as the first line of the body, or around the statement"
                )),
            );
        }
        // Decorator arguments v2.2 took away.
        let removed = match name.text.as_str() {
            "memo" => args.iter().find_map(|arg| match &arg.value.kind {
                ExprKind::Name(kind) if arg.name.is_none() && (kind == "flat" || kind == "hash") => Some((
                    format!("`@memo({kind})` was removed in CP Lang 0.2.2"),
                    "write bare `@memo`: the table is flat when the arguments are small and hashed otherwise",
                )),
                _ => None,
            }),
            "multitest" => args
                .iter()
                .any(|arg| arg.name.as_ref().is_some_and(|n| n.text == "count"))
                .then(|| {
                    (
                        "`@multitest(count:)` was removed in CP Lang 0.2.2".to_string(),
                        "read the count and loop: `for _ in 0..tests:`",
                    )
                }),
            "grader" if self.at(T::Const) => Some((
                "`@grader const` was removed in CP Lang 0.2.2".to_string(),
                "take the value as a parameter of the `@grader fn`",
            )),
            _ => None,
        };
        if let Some((message, help)) = removed {
            if self.speculating == 0 {
                self.sink
                    .push(Diagnostic::error(codes::REMOVED_SYNTAX, span, message).with_help(help));
            }
        }
        Decorator {
            directive,
            name,
            args,
            span,
        }
    }

    /// Ends a statement: expects the end of the line (or `;` in braces).
    pub(super) fn end_stmt(&mut self, start: u32) {
        let ended = self.eat(T::Newline)
            || self.prev_kind() == Some(T::Dedent)
            || matches!(self.peek(), T::Dedent | T::Eof)
            || (self.brace_depth > 0 && matches!(self.peek(), T::Semi | T::RBrace));
        // `a; b` on one line is two statements (v2.2 cheat sheet).
        let ended = ended
            || (self.at(T::Semi)
                && !matches!(self.peek_n(1), T::Newline | T::Eof | T::Dedent)
                && {
                    self.bump();
                    true
                });
        if !ended && self.at(T::At) && self.peek_n(1) == T::Ident {
            let at = self.start();
            self.decorator();
            let span = self.span_from(at);
            self.planned(span, &crate::planned::VALUE_DECORATOR);
            return self.end_stmt(start);
        }
        if !ended {
            if self.at(T::Indent) {
                // Reported by `stmt_into` as an unexpected indent.
                self.end_recovery(start);
                return;
            }
            let found = self.peek();
            let mut span = self.span();
            // `...`, lexed `..` then `.`: an elided body, which cp spells `todo`.
            let prev = self
                .pos
                .checked_sub(1)
                .map(|i| self.rel(self.tokens[i].span));
            let ellipsis = found == T::Dot && self.prev_kind() == Some(T::DotDot);
            if let (true, Some(prev)) = (ellipsis, prev.filter(|p| p.end == span.start)) {
                span = Span::new(prev.start, span.end);
                let message = "`...` is not an expression; write `todo` for an unwritten part";
                let diag = Diagnostic::error(codes::UNEXPECTED_TOKEN, span, message)
                    .with_help("`todo` has every type and traps when it runs")
                    .with_fix("replace with `todo`", span, "todo");
                self.error(diag);
                self.skip_line();
                self.eat(T::Newline);
                return self.end_recovery(start);
            }
            let mut diag = Diagnostic::error(
                if found.is_closer() {
                    codes::UNMATCHED_CLOSER
                } else {
                    codes::UNEXPECTED_TOKEN
                },
                span,
                if found.is_closer() {
                    format!("this {} has no matching opener", found.describe())
                } else {
                    format!(
                        "expected the end of the statement, found {}",
                        found.describe()
                    )
                },
            );
            if found.is_closer() {
                diag = diag.with_fix("remove it", span, "");
            } else if found == T::Semi {
                diag = diag
                    .with_help(
                        "statements end at the end of the line; `;` is only used inside `{ ... }`",
                    )
                    .with_fix("remove `;`", span, "");
            }
            self.error(diag);
            self.skip_line();
            if !self.eat(T::Newline) && self.brace_depth == 0 {
                self.eat(T::Semi);
            }
        }
        self.end_recovery(start);
    }

    pub(super) fn stmt_kind(&mut self) -> StmtKind {
        match self.peek() {
            T::Let | T::Var => self.let_stmt(),
            // `auto x = 0`, `int x = 0`: C++'s declarations, read as `let`.
            T::Ident
                if matches!(
                    self.text(),
                    "auto" | "int" | "long" | "double" | "bool" | "string" | "char"
                ) && (self.peek_n(1) == T::Ident
                    && matches!(self.peek_n(2), T::Eq | T::Colon)
                    || self.text() == "auto"
                        && matches!(self.peek_n(1), T::LParen | T::LBracket)) =>
            {
                let kw = self.span();
                self.error(
                    Diagnostic::error(
                        codes::REMOVED_SYNTAX,
                        kw,
                        format!("CP Lang declares a variable with `let`, not `{}`", self.text()),
                    )
                    .with_help("`let x = 0` for a value that stays, `var x = 0` for one that changes; the type comes from the value")
                    .with_fix("write `let`", kw, "let"),
                );
                self.let_stmt()
            }
            // `extern fn transaction(m: Int) -> Int`: the judge supplies the body.
            T::Ident if self.text() == "extern" && self.peek_n(1) == T::Fn => {
                self.bump();
                let was_in_grader = std::mem::replace(&mut self.in_grader, true);
                let kind = self.stmt_kind();
                self.in_grader = was_in_grader;
                kind
            }
            // `mod name:` with an indented body: inline module.
            T::Ident
                if self.text() == "mod"
                    && self.peek_n(1) == T::Ident
                    && self.peek_n(2) == T::Colon =>
            {
                self.bump();
                let name = Name {
                    text: self.text().to_string(),
                    span: self.span(),
                };
                self.bump();
                let body = self.colon_block("the module name");
                StmtKind::Mod { name, body }
            }
            T::Ident if self.text() == "macro" && self.peek_n(1) == T::Ident => {
                let start = self.start();
                self.skip_line();
                let span = self.span_from(start);
                self.error(Diagnostic::error(
                    codes::UNSUPPORTED,
                    span,
                    "CP Lang has no macros; use a generic fn or inline helper instead",
                ));
                StmtKind::Error
            }
            T::Ident if self.text() == "guard" && self.peek_n(1) == T::Let => {
                self.bump();
                self.let_stmt()
            }
            T::Ident if self.text() == "context" && self.peek_n(1) == T::Ident => {
                self.bump();
                let name = self.expect_name("a context name after `context`");
                let mut fields = Vec::new();
                if self.at(T::Colon) {
                    self.members(&format!("the fields of `{}`", name.text), |p| {
                        let fname = p.expect_name("a context field name");
                        p.expect(T::Colon, "`:` and the field's type");
                        let ty = p.ty();
                        let default = p.eat(T::Eq).then(|| p.expr());
                        fields.push(FieldDecl {
                            name: fname,
                            ty,
                            default,
                            mutable: false,
                            cached: false,
                        });
                    });
                }
                StmtKind::Context { name, fields }
            }
            T::Ident if self.text() == "with" && self.peek_n(1) == T::Ident => {
                self.bump();
                let head = self.expr();
                let (ctx, args) = match head.kind {
                    ExprKind::Call { callee, args } => match callee.kind {
                        ExprKind::Name(text) => (
                            Name {
                                text,
                                span: callee.span,
                            },
                            args,
                        ),
                        _ => return StmtKind::Error,
                    },
                    ExprKind::Name(text) => (
                        Name {
                            text,
                            span: head.span,
                        },
                        Vec::new(),
                    ),
                    _ => return StmtKind::Error,
                };
                let body = self.at(T::Colon).then(|| self.colon_block("the context"));
                StmtKind::With { ctx, args, body }
            }
            T::Ident if self.text() == "macro" && self.peek_n(1) == T::Ident => {
                let span = self.span();
                self.not_in_language(
                    span,
                    "there are no macros (R08.5.4)",
                    "write a generic fn; a lambda argument covers code that takes code",
                );
                self.skip_line();
                StmtKind::Error
            }
            T::Ident
                if self.text() == "record"
                    && self.peek_n(1) == T::Ident
                    && self.peek_n(2) == T::LParen =>
            {
                let span = self.span();
                self.planned(span, &crate::planned::RECORD_DECL);
                self.struct_decl()
            }
            T::Const => self.const_stmt(),
            T::Fn => self.fn_stmt(),
            T::Struct => self.struct_decl(),
            T::Enum => self.enum_decl(),
            T::Domain => self.domain_decl(),
            T::Type => self.type_alias(),
            T::Trait => self.trait_decl(),
            T::Impl => self.impl_decl(),
            T::Use => self.use_decl(),
            T::While => {
                self.bump();
                let cond = self.cond();
                let body = self.colon_block("the `while` condition");
                let else_body = self.loop_else();
                StmtKind::While {
                    cond,
                    body,
                    else_body,
                }
            }
            T::For => self.for_stmt(None),
            T::Label if self.peek_n(1) == T::Colon && self.peek_n(2) == T::For => {
                let label = self.name();
                self.bump();
                self.for_stmt(Some(label))
            }
            T::Return => {
                self.bump();
                let value = (!self.at_stmt_end()).then(|| self.expr());
                StmtKind::Return(value)
            }
            T::Break => {
                self.bump();
                StmtKind::Break(self.at(T::Label).then(|| self.name()))
            }
            T::Continue => {
                self.bump();
                StmtKind::Continue(self.at(T::Label).then(|| self.name()))
            }
            T::Yield => {
                self.bump();
                let from = self.at_ident("from")
                    && !matches!(
                        self.peek_n(1),
                        T::Newline | T::Dot | T::LParen | T::LBracket
                    );
                if from {
                    self.bump();
                }
                StmtKind::Yield {
                    value: self.expr(),
                    from,
                }
            }
            T::Assert => {
                self.bump();
                let cond = self.expr();
                let message = self.eat(T::Comma).then(|| self.expr());
                StmtKind::Assert { cond, message }
            }
            T::Unchecked if self.peek_n(1) == T::Colon => {
                self.bump();
                StmtKind::Unchecked(self.colon_block("`unchecked`"))
            }
            T::Elif | T::Else => {
                let word = self.text().to_string();
                let span = self.span();
                let target = if word == "else" {
                    "`if` or loop"
                } else {
                    "`if`"
                };
                self.error(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        format!("this `{word}` has no {target} above it at the same indentation"),
                    )
                    .with_help(format!("line `{word}` up with its {target}")),
                );
                self.skip_line();
                StmtKind::Error
            }
            T::RParen | T::RBracket | T::RBrace | T::Error => {
                // `end_stmt` reports the stray token.
                if self.at(T::Error) {
                    self.recovering = true;
                    self.skip_line();
                }
                StmtKind::Error
            }
            T::Ident if self.text() == "loop" && self.peek_n(1) == T::Colon => {
                let span = self.span();
                self.bump();
                let body = self.colon_block("`loop`");
                let else_body = self.loop_else();
                StmtKind::While {
                    cond: Cond::Expr(Expr {
                        kind: ExprKind::Bool(true),
                        span,
                    }),
                    body,
                    else_body,
                }
            }
            T::Ident
                if self.text() == "repeat"
                    && !matches!(
                        self.peek_n(1),
                        T::LParen
                            | T::Dot
                            | T::LBracket
                            | T::Colon
                            | T::Eq
                            | T::Newline
                            | T::Eof
                            | T::Comma
                            | T::RParen
                    ) =>
            {
                // `repeat n:` is `for _ in 0..n:`.
                let word = self.span();
                self.bump();
                let count = self.expr();
                let span = self.span_from(word.start);
                let body = self.colon_block("the `repeat` count");
                let else_body = self.loop_else();
                StmtKind::For {
                    label: None,
                    pat: Pattern {
                        kind: PatternKind::Wildcard,
                        span: word,
                    },
                    iter: Expr {
                        kind: ExprKind::Range {
                            lo: Some(Box::new(Expr {
                                kind: ExprKind::Int(0),
                                span: word,
                            })),
                            hi: Some(Box::new(count)),
                            inclusive: false,
                            step: None,
                        },
                        span,
                    },
                    body,
                    else_body,
                }
            }
            T::Ident
                if self.text() == "law"
                    && self.peek_n(1) == T::Ident
                    && self.peek_n(2) == T::LParen =>
            {
                self.bump();
                self.law()
            }
            T::Ident
                if self.text() == "lend"
                    && matches!(self.peek_n(1), T::Mut | T::Ident | T::SelfKw) =>
            {
                self.bump();
                let mutable = self.eat(T::Mut);
                StmtKind::Lend {
                    mutable,
                    place: self.expr(),
                }
            }
            // The place a `lends mut` body hands out: `mut self.data[i]`.
            T::Mut if matches!(self.peek_n(1), T::Ident | T::SelfKw) => {
                self.bump();
                StmtKind::Lend {
                    mutable: true,
                    place: self.expr(),
                }
            }
            T::Ident
                if matches!(self.text(), "defer" | "assume" | "inline")
                    && matches!(
                        self.peek_n(1),
                        T::Ident | T::Fn | T::SelfKw | T::Not | T::Int | T::True | T::False
                    ) =>
            {
                let span = self.span();
                let word = self.text();
                let help = crate::token::removed_keyword(word).unwrap_or_default();
                self.error(
                    Diagnostic::error(
                        codes::REMOVED_SYNTAX,
                        span,
                        format!("`{word}` was removed in CP Lang 0.2.2"),
                    )
                    .with_help(help),
                );
                self.recovering = true;
                self.skip_line();
                StmtKind::Error
            }
            T::Mut if self.peek_n(1) == T::Fn => {
                let span = self.span();
                self.error(
                    Diagnostic::error(
                        codes::EXPECTED_EXPR,
                        span,
                        "a fn is not marked `mut`: mark the parameter it changes, `fn f(mut x: T)`",
                    )
                    .with_fix("remove `mut`", span, ""),
                );
                self.recovering = false;
                self.bump();
                StmtKind::Fn(self.fn_decl())
            }
            T::Ident if self.text() == "pure" && self.peek_n(1) == T::Fn => {
                let span = self.span();
                self.error(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        "`pure fn` was removed; write `fn`",
                    )
                    .with_fix("remove `pure`", span, ""),
                );
                self.recovering = false;
                self.bump();
                StmtKind::Fn(self.fn_decl())
            }
            _ => self.expr_stmt(),
        }
    }

    fn at_stmt_end(&self) -> bool {
        self.at_line_end() || (self.brace_depth > 0 && matches!(self.peek(), T::Semi | T::RBrace))
    }

    fn has_typed_tuple_pattern(&self) -> bool {
        let end = after_group(self.tokens, self.pos);
        let mut depth = 0usize;
        for t in &self.tokens[self.pos + 1..end.saturating_sub(1)] {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth = depth.saturating_sub(1),
                T::Colon if depth == 0 => return true,
                _ => {}
            }
        }
        false
    }

    fn let_stmt(&mut self) -> StmtKind {
        let stmt_start = self.start();
        let mut mutable = self.at(T::Var);
        self.bump();
        if !mutable && self.at(T::Mut) {
            let span = Span::new(stmt_start, self.span().end);
            self.error(
                Diagnostic::error(
                    codes::UNEXPECTED_TOKEN,
                    span,
                    "`let mut` is `var` in CP Lang",
                )
                .with_fix("replace with `var`", span, "var"),
            );
            self.bump();
            mutable = true;
        }
        // `let (a: Int, b: Str) = ...`: typed tuple pattern desugars to
        // `let (a, b): (Int, Str) = ...`.
        let mut tuple_elem_tys: Option<Type> = None;
        let mut pat =
            if self.at(T::LParen) && !self.at_record_pattern() && self.has_typed_tuple_pattern() {
                let pstart = self.start();
                self.bump();
                let mut pats = Vec::new();
                let mut tys = Vec::new();
                let items = self.comma_list(T::RParen, "the typed tuple pattern", |p| {
                    let pat = p.pattern();
                    let ty = p.eat(T::Colon).then(|| p.ty());
                    (pat, ty)
                });
                for (p, t) in items {
                    pats.push(p);
                    if let Some(t) = t {
                        tys.push(t);
                    }
                }
                if tys.len() == pats.len() && !tys.is_empty() {
                    tuple_elem_tys = Some(Type {
                        kind: TypeKind::Tuple(tys),
                        span: self.span_from(pstart),
                    });
                }
                Pattern {
                    kind: PatternKind::Tuple(pats),
                    span: self.span_from(pstart),
                }
            } else {
                self.pattern()
            };
        // `var ones, open, ans: Z = 0`, `var ones, one_twos, ans: Z = (0, 0, 0)`,
        // and `let n, m: 1..=20 = read()`: homogeneous multi-binding.
        let mut shared_pats: Vec<Pattern> = Vec::new();
        if matches!(&pat.kind, PatternKind::Name(_))
            && self.at(T::Comma)
            && self.peek_n(1) == T::Ident
            && matches!(self.peek_n(2), T::Comma | T::Colon)
        {
            let start = pat.span.start;
            shared_pats.push(pat);
            while self.eat(T::Comma) {
                shared_pats.push(self.pattern());
            }
            pat = Pattern {
                kind: PatternKind::Tuple(shared_pats.clone()),
                span: self.span_from(start),
            };
        }
        if matches!(&pat.kind, PatternKind::Name(_)) && self.at(T::Comma) {
            pat = self.bare_tuple_pattern(pat, T::Eq);
        }
        let shared_elem_ty = self.eat(T::Colon).then(|| self.ty());
        let ty = tuple_elem_tys.or_else(|| {
            shared_elem_ty.as_ref().map(|elem| {
                if shared_pats.len() > 1 {
                    let span = elem.span;
                    Type {
                        kind: TypeKind::Tuple(vec![elem.clone(); shared_pats.len()]),
                        span,
                    }
                } else {
                    elem.clone()
                }
            })
        });
        let init = if self.eat(T::Eq) {
            if let Some(value) = self.continued_value() {
                Some(value)
            } else if self.at(T::Mut) {
                let mstart = self.start();
                self.bump();
                let inner = self.expr();
                Some(Expr {
                    kind: ExprKind::Mut(Box::new(inner)),
                    span: self.span_from(mstart),
                })
            } else if self.at(T::Newline)
                && self.peek_n(1) == T::Indent
                && matches!(self.peek_n(2), T::If | T::Match)
            {
                self.bump();
                self.bump();
                let value = self.expr();
                if self.at(T::Newline) && self.peek_n(1) == T::Dedent {
                    self.bump();
                }
                self.eat(T::Dedent);
                Some(value)
            } else {
                Some(self.expr())
            }
        } else {
            if ty.is_none() {
                let word = if mutable { "var" } else { "let" };
                self.error_expected(
                    codes::EXPECTED_TOKEN,
                    &format!("`=` and a value, or `: Type`, after the name in `{word}`"),
                    None,
                );
            } else {
                let word = if mutable { "var" } else { "let" };
                let span = self.span_from(stmt_start);
                self.error(Diagnostic::error(
                    codes::EXPECTED_TOKEN,
                    span,
                    format!(
                        "`{word}` with a type and no value: write `=` and the value it starts with (R01.10.2)"
                    ),
                ));
            }
            None
        };
        // `var ones, open, ans: Z = 0` or `var ways, next: [1..=m]Z = 0`:
        // a single non-tuple, non-`read()` initializer fills each binding with
        // its own copy of the declared type and value.
        let is_tuple_or_read = init.as_ref().is_some_and(|e| {
            matches!(e.kind, ExprKind::Tuple(_))
                || matches!(
                    &e.kind,
                    ExprKind::Call { callee, .. }
                        if matches!(&callee.kind, ExprKind::Name(n) if n == "read")
                )
        });
        if shared_pats.len() > 1 && init.is_some() && !is_tuple_or_read {
            let full_span = self.span_from(stmt_start);
            let mut iter = shared_pats.into_iter();
            let first = iter.next().unwrap();
            for next_pat in iter {
                self.extra_stmts.push(Stmt {
                    kind: StmtKind::Let {
                        mutable,
                        pat: next_pat,
                        ty: shared_elem_ty.clone(),
                        init: init.clone(),
                        guard: None,
                        otherwise: None,
                    },
                    span: full_span,
                    decorators: Vec::new(),
                    public: false,
                });
            }
            return StmtKind::Let {
                mutable,
                pat: first,
                ty: shared_elem_ty,
                init,
                guard: None,
                otherwise: None,
            };
        }
        // `var best = 0, ways = 1` is two bindings.
        while init.is_some()
            && self.at(T::Comma)
            && self.peek_n(1) == T::Ident
            && matches!(self.peek_n(2), T::Eq | T::Colon)
        {
            self.bump();
            let start = self.start();
            let pat = self.pattern();
            let ty = self.eat(T::Colon).then(|| self.ty());
            self.expect(T::Eq, "`=` and a value after the name");
            let value = self.expr();
            self.extra_stmts.push(Stmt {
                kind: StmtKind::Let {
                    mutable,
                    pat,
                    ty,
                    init: Some(value),
                    otherwise: None,
                    guard: None,
                },
                span: self.span_from(start),
                decorators: Vec::new(),
                public: false,
            });
        }
        // `let a = x and let b = y else: return` is one `let ... else`
        // after another, with the same `else`.
        let mut more = Vec::new();
        while init.is_some() && self.at(T::And) && self.peek_n(1) == T::Let {
            self.bump();
            let start = self.start();
            self.bump();
            let pat = self.pattern();
            self.expect(T::Eq, "`=` and a value after the name in `let`");
            let value = self.expr();
            more.push((pat, value, self.span_from(start)));
        }
        // `let v = u else: return`.
        let otherwise = (init.is_some() && self.at(T::Else)).then(|| {
            self.bump();
            self.colon_block("`else`")
        });
        for (pat, value, span) in more {
            self.extra_stmts.push(Stmt {
                kind: StmtKind::Let {
                    mutable,
                    pat,
                    ty: None,
                    init: Some(value),
                    otherwise: otherwise.clone(),
                    guard: None,
                },
                span,
                decorators: Vec::new(),
                public: false,
            });
        }
        // `let k: 0..n = read() where k < n`.
        let guard = if self.at_ident("where") {
            let span = self.span();
            self.error(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    "`where` guards were removed in CP Lang 0.2.2",
                )
                .with_help("put the constraint in the type (`let k: 0..n = read()`) or `assert` it on the next line"),
            );
            self.bump();
            Some(self.expr())
        } else {
            None
        };
        StmtKind::Let {
            mutable,
            pat,
            ty,
            init,
            otherwise,
            guard,
        }
    }

    fn const_stmt(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a name after `const`");
        let (generics, mut bounds) = self.generic_params();
        let ty = self.eat(T::Colon).then(|| self.ty());
        let init = if self.eat(T::Eq) {
            Some(self.expr())
        } else {
            if !self.in_trait {
                self.error_expected(
                    codes::EXPECTED_TOKEN,
                    "`=` and a value for the `const`",
                    None,
                );
            }
            None
        };
        self.where_clause(&mut bounds);
        // `const N = 3e5, B = 1200` is two constants, as `let a = 0, b = 1`
        // is two bindings.
        while init.is_some()
            && self.at(T::Comma)
            && self.peek_n(1) == T::Ident
            && matches!(self.peek_n(2), T::Eq | T::Colon)
        {
            self.bump();
            let start = self.start();
            let name = self.name();
            let ty = self.eat(T::Colon).then(|| self.ty());
            self.expect(T::Eq, "`=` and a value after the name");
            let value = self.expr();
            self.extra_stmts.push(Stmt {
                kind: StmtKind::Const {
                    name,
                    generics: Vec::new(),
                    bounds: Vec::new(),
                    ty,
                    init: Some(value),
                },
                span: self.span_from(start),
                decorators: Vec::new(),
                public: false,
            });
        }
        StmtKind::Const {
            name,
            generics,
            bounds,
            ty,
            init,
        }
    }

    /// The rest of a law, from its name: `assoc(a, b, c) = cond`.
    fn law(&mut self) -> StmtKind {
        let name = self.name();
        self.expect(T::LParen, "`(` and the law's arguments");
        let params = self.params();
        self.expect(T::Eq, "`=` and the law's condition");
        StmtKind::Law {
            name,
            params,
            body: self.expr(),
        }
    }

    fn for_stmt(&mut self, label: Option<Name>) -> StmtKind {
        self.bump();
        if self.at(T::Mut) {
            let span = self.span();
            self.error(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    "`for mut` was removed in CP Lang 0.2.2",
                )
                .with_help("loop over the indexes and assign through them: `for i in 0..xs.len: xs[i] += 1`"),
            );
            self.bump();
        }
        let mut clauses = Vec::new();
        if let Some((pat, iter)) = self.c_for() {
            clauses.push((pat, iter, None));
        }
        let c_style = !clauses.is_empty();
        if !c_style {
            loop {
                let paren = self.paren_for();
                let mut pat = self.pattern();
                if matches!(&pat.kind, PatternKind::Name(_)) && self.at(T::Comma) {
                    pat = self.bare_tuple_pattern(pat, T::In);
                }
                if !self.for_of() {
                    self.expect(T::In, "`in` after the loop variable");
                }
                let iter = if self.at(T::Question) {
                    self.optional_type_iter()
                } else {
                    self.expr()
                };
                if paren {
                    self.eat(T::RParen);
                }
                // `for x in xs if cond:` runs the body for the `x` that pass:
                // the body is an `if`.
                let has_filter = self.eat(T::If)
                    || (self.at_ident("where") && {
                        let span = self.span();
                        self.error(
                            Diagnostic::error(
                                codes::REMOVED_SYNTAX,
                                span,
                                "loop filters use `if`, not `where`",
                            )
                            .with_help("use `if` to filter a `for` loop")
                            .with_fix(
                                "replace `where` with `if`",
                                span,
                                "if",
                            ),
                        );
                        self.bump();
                        true
                    });
                let filter = has_filter.then(|| (self.start(), self.cond()));
                clauses.push((pat, iter, filter));
                if !self.eat(T::Comma) {
                    break;
                }
            }
        }
        let body = self.colon_block("the `for` header");
        let mut out_stmt: Option<StmtKind> = None;
        for (idx, (pat, iter, filter)) in clauses.into_iter().enumerate().rev() {
            let mut inner_body = if let Some(prev) = out_stmt.take() {
                let span = body.span;
                Block {
                    stmts: vec![Stmt {
                        kind: prev,
                        span,
                        decorators: Vec::new(),
                        public: false,
                    }],
                    span,
                }
            } else {
                body.clone()
            };
            if let Some((start, cond)) = filter {
                let span = self.span_from(start);
                let test = Expr {
                    kind: ExprKind::If {
                        branches: vec![(cond, inner_body)],
                        otherwise: None,
                    },
                    span,
                };
                inner_body = Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(test),
                        span,
                        decorators: Vec::new(),
                        public: false,
                    }],
                    span,
                };
            }
            let clause_label = if idx == 0 { label.clone() } else { None };
            let clause_else = if idx == 0 { self.loop_else() } else { None };
            out_stmt = Some(StmtKind::For {
                label: clause_label,
                pat,
                iter,
                body: inner_body,
                else_body: clause_else,
            });
        }
        out_stmt.unwrap()
    }

    /// Optional trailing `else:` on a `for` / `while` loop (decision 125),
    /// at the loop's indentation.
    fn loop_else(&mut self) -> Option<Block> {
        if self.at(T::Newline) && self.peek_n(1) == T::Else {
            self.bump();
        }
        if self.eat(T::Else) {
            Some(self.colon_block("`else`"))
        } else {
            None
        }
    }

    /// `?Col` where a `for` walks: every value of the type, `none` first.
    /// It is held as `Opt<Col>`, the way `Subset<Row>` is.
    pub(super) fn optional_type_iter(&mut self) -> Expr {
        let start = self.start();
        let name_span = self.span();
        self.bump();
        let inner = self.ty();
        Expr {
            kind: ExprKind::Generic {
                base: Box::new(Expr {
                    kind: ExprKind::Name("Opt".to_string()),
                    span: name_span,
                }),
                args: vec![inner],
            },
            span: self.span_from(start),
        }
    }

    /// True at the `(` of `(place ??= init)`.
    fn at_filled_place(&self) -> bool {
        if !self.at(T::LParen) {
            return false;
        }
        let mut depth = 0u32;
        for t in &self.tokens[self.pos + 1..] {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace if depth == 0 => return false,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                T::OpEq if depth == 0 => return t.span.text(self.src) == "??=",
                T::Newline | T::Indent | T::Dedent | T::Eof => return false,
                _ => {}
            }
        }
        false
    }

    /// `(m[k] ??= init)` at the start of a statement: `m[k] ??= init` runs
    /// first, then the statement goes on with `m[k]`. An index that is
    /// more than a name or a number is worked out once, into a hidden
    /// `let`, so `(m[f(x)] ??= []).push(y)` calls `f` once.
    fn filled_place(&mut self) -> Expr {
        let start = self.start();
        self.bump();
        let mut target = self.postfix();
        self.name_indices(&mut target);
        self.bump();
        let value = self.expr();
        self.expect(T::RParen, "`)` to close the parenthesis");
        let span = self.span_from(start);
        self.lead_stmts.push(Stmt {
            kind: StmtKind::Assign {
                target: target.clone(),
                op: AssignOp::Binary(BinOp::Coalesce),
                value,
            },
            span,
            decorators: Vec::new(),
            public: false,
        });
        Expr {
            kind: ExprKind::Paren(Box::new(target)),
            span,
        }
    }

    fn name_indices(&mut self, place: &mut Expr) {
        match &mut place.kind {
            ExprKind::Field { base, .. } | ExprKind::Paren(base) => self.name_indices(base),
            ExprKind::Index { base, indices } => {
                self.name_indices(base);
                for index in indices {
                    if matches!(index.kind, ExprKind::Name(_) | ExprKind::Int(_)) {
                        continue;
                    }
                    let span = index.span;
                    let name = format!("__key_{}", span.start);
                    let init = std::mem::replace(
                        index,
                        Expr {
                            kind: ExprKind::Name(name.clone()),
                            span,
                        },
                    );
                    self.lead_stmts.push(Stmt {
                        kind: StmtKind::Let {
                            mutable: false,
                            pat: Pattern {
                                kind: PatternKind::Name(name),
                                span,
                            },
                            ty: None,
                            init: Some(init),
                            otherwise: None,
                            guard: None,
                        },
                        span,
                        decorators: Vec::new(),
                        public: false,
                    });
                }
            }
            _ => {}
        }
    }

    pub(super) fn expr_stmt(&mut self) -> StmtKind {
        let target = if self.at_filled_place() {
            let start = self.start();
            let place = self.filled_place();
            self.postfix_from(start, place)
        } else {
            self.stmt_head = true;
            let target = self.expr();
            self.stmt_head = false;
            target
        };
        let op = match self.peek() {
            T::Eq => AssignOp::Set,
            T::OpEq => {
                let text = self.text();
                AssignOp::Binary(match &text[..text.len() - 1] {
                    "+" => BinOp::Add,
                    "-" => BinOp::Sub,
                    "*" => BinOp::Mul,
                    "/" => BinOp::Div,
                    "%" => BinOp::Rem,
                    "**" => BinOp::Pow,
                    "^" => BinOp::BitXor,
                    "&" => BinOp::BitAnd,
                    "|" => BinOp::BitOr,
                    "<<" => BinOp::Shl,
                    "??" => BinOp::Coalesce,
                    _ => BinOp::Shr,
                })
            }
            // `best max= x`.
            T::Ident
                if self.peek_n(1) == T::Eq
                    && self.same_line()
                    && matches!(self.text(), "max" | "min") =>
            {
                let op = if self.text() == "max" {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                self.bump();
                op
            }
            // `l lcm= x` is `l = lcm(l, x)`, for any fn of two values.
            T::Ident if self.peek_n(1) == T::Eq && self.same_line() => {
                let name = Expr {
                    kind: ExprKind::Name(self.text().to_string()),
                    span: self.span(),
                };
                self.bump();
                self.bump();
                let operand = self.expr();
                let span = name.span.to(operand.span);
                let args = [target.clone(), operand]
                    .into_iter()
                    .map(|value| Arg {
                        name: None,
                        mutable: false,
                        value,
                    })
                    .collect();
                let value = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(name),
                        args,
                    },
                    span,
                };
                return StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                };
            }
            _ => return StmtKind::Expr(target),
        };
        self.bump();
        let value = self.expr();
        StmtKind::Assign { target, op, value }
    }

    /// Whether the next token continues a line that `target` is on,
    /// rather than starting a line of its own. A target that ends with a
    /// block (`if c:` and its body) is followed by the next statement
    /// with no line break token between them.
    /// `c ? a : b` from C, after the condition: inline `if` here.
    pub(super) fn c_ternary_ahead(&self, cond: &Expr) -> bool {
        let rest = self.tokens.get(self.pos + 1..).unwrap_or_default();
        let mut line = rest.iter().take_while(|t| t.kind != T::Newline);
        self.at(T::Question)
            && self.span().start > cond.span.end
            && !matches!(self.peek_n(1), T::RParen | T::RBracket | T::Comma | T::Dot)
            && line.any(|t| t.kind == T::Colon)
    }

    /// Reads `c ? a : b` as `if c: a else: b`, with one error that says so
    /// and rewrites it, so the rest of the program sees an inline `if`.
    pub(super) fn c_ternary(&mut self, start: u32, first: usize, cond: Expr) -> Expr {
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
        let mut diag = Diagnostic::error(codes::REMOVED_SYNTAX, q, "CP Lang has no `c ? a : b`")
            .with_help("write `if c: a else: b`");
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
                "write it as an inline `if`",
                whole,
                format!("if {c}: {a} else: {b}"),
            );
        }
        self.error(diag);
        Self::inline_if(cond, yes, no, span)
    }

    pub(super) fn same_line(&self) -> bool {
        // Token spans index the source; the spans of the tree do not.
        let Some(before) = self
            .tokens
            .get(self.pos)
            .and_then(|token| self.src.get(..token.span.start as usize))
        else {
            return false;
        };
        let line = &before[before.rfind('\n').map_or(0, |i| i + 1)..];
        !line.trim().is_empty()
    }

    pub(super) fn cond(&mut self) -> Cond {
        if self.at(T::Ident)
            && self.peek_n(1) == T::Colon
            && self.peek_n(2) == T::Ident
            && self.peek_n(3) == T::Colon
        {
            let start = self.start();
            let param = self.name();
            self.bump();
            let trait_name = self.name();
            let span = self.span_from(start);
            return Cond::Expr(Expr {
                kind: ExprKind::HasBound { param, trait_name },
                span,
            });
        }
        if self.at(T::Let) {
            self.bump();
            return self.let_cond();
        }
        if self.at(T::Const) {
            self.bump();
            return Cond::Const(self.expr());
        }
        let test = self.expr();
        if self.eat(T::Comma) {
            return Cond::Guard {
                test,
                and: Box::new(self.more_clauses()),
            };
        }
        fn unwrap_is_cond(expr: Expr) -> Cond {
            let is_match = matches!(
                &expr.kind,
                ExprKind::If {
                    branches,
                    otherwise: Some(otherwise),
                } if branches.len() == 1
                    && matches!(branches[0].0, Cond::Let { and: None, .. })
                    && branches[0].1.stmts.len() == 1
                    && matches!(branches[0].1.stmts[0].kind, StmtKind::Expr(_))
                    && otherwise.stmts.len() == 1
                    && matches!(otherwise.stmts[0].kind, StmtKind::Expr(Expr { kind: ExprKind::Bool(false), .. }))
            );
            if is_match {
                let ExprKind::If { mut branches, .. } = expr.kind else {
                    unreachable!()
                };
                let (Cond::Let { pat, value, .. }, mut then) = branches.pop().unwrap() else {
                    unreachable!()
                };
                let StmtKind::Expr(rhs) = then.stmts.pop().unwrap().kind else {
                    unreachable!()
                };
                let and = if matches!(rhs.kind, ExprKind::Bool(true)) {
                    None
                } else {
                    Some(Box::new(unwrap_is_cond(rhs)))
                };
                return Cond::Let { pat, value, and };
            }
            Cond::Expr(expr)
        }
        unwrap_is_cond(test)
    }

    /// Parses `pat = value` after `let` (or after `,` in multi-binding
    /// `if let a = x, b = y where cond:`).
    fn let_cond(&mut self) -> Cond {
        let pat = self.pattern();
        self.expect(T::Eq, "`=` after the pattern in `if let`");
        // The value ends before `and`, `,`, or `where`, which adds a
        // binding or condition on what the pattern bound. After `,` or
        // `where` the condition is whole, `or` and all; after `and` an
        // `or` would read two ways.
        let value = self.not();
        let and = if self.eat(T::Comma) {
            Some(Box::new(if self.at(T::Let) {
                self.bump();
                self.let_cond()
            } else if self.at_let_binding() {
                self.let_cond()
            } else {
                self.more_clauses()
            }))
        } else if self.eat(T::And) || self.c_and() {
            Some(Box::new(if self.at(T::Let) {
                self.bump();
                self.let_cond()
            } else {
                Cond::Expr(self.and())
            }))
        } else if self.at_ident("where") {
            let span = self.span();
            self.error(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    "use `and`, not `where`, after `if let`",
                )
                .with_help("write `if let pat = value and cond`")
                .with_fix("replace `where` with `and`", span, "and"),
            );
            self.bump();
            Some(Box::new(self.more_clauses()))
        } else {
            None
        };
        Cond::Let { pat, value, and }
    }

    /// True when the upcoming tokens form `pat = ...` (without a leading
    /// `let`), as in `if let a = x, b = y:` or `if let (a, b) = p, (c, d) = q:`.
    fn at_let_binding(&self) -> bool {
        let mut i = self.pos;
        match self.tokens.get(i).map(|t| t.kind) {
            Some(T::Ident) => {
                i += 1;
                if self.tokens.get(i).map(|t| t.kind) == Some(T::LParen) {
                    i = self.skip_parens_from(i);
                }
            }
            Some(T::LParen) => {
                i = self.skip_parens_from(i);
            }
            _ => return false,
        }
        self.tokens.get(i).map(|t| t.kind) == Some(T::Eq)
    }

    fn skip_parens_from(&self, start: usize) -> usize {
        let mut depth = 0u32;
        let mut i = start;
        while let Some(t) = self.tokens.get(i) {
            match t.kind {
                T::LParen => depth += 1,
                T::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return i + 1;
                    }
                }
                T::Newline | T::Eof => return i,
                _ => {}
            }
            i += 1;
        }
        i
    }

    /// What follows a `,` or `where` in a condition: `let`s and whole
    /// tests, any number, each holding only if those before it did.
    fn more_clauses(&mut self) -> Cond {
        if self.at(T::Let) {
            self.bump();
            return self.let_cond();
        }
        let test = self.or();
        if self.eat(T::Comma) {
            return Cond::Guard {
                test,
                and: Box::new(self.more_clauses()),
            };
        }
        Cond::Expr(test)
    }

    /// Parses `: block`. `after` names what the colon follows.
    pub(super) fn colon_block(&mut self, after: &str) -> Block {
        if !(self.eat(T::Colon) || self.foreign_then()) {
            let at = self.error_span();
            let found = self.peek().describe();
            let next_after_nl = if self.at(T::Newline) {
                if self.peek_n(1) == T::Indent {
                    self.peek_n(2)
                } else {
                    self.peek_n(1)
                }
            } else {
                T::Eof
            };
            if self.at(T::Newline)
                && (next_after_nl.continues_line() || matches!(next_after_nl, T::Gt | T::Shr))
            {
                self.error(
                    Diagnostic::error(
                        codes::EXPECTED_TOKEN,
                        at,
                        format!("expected `:` after {after}, found {found}"),
                    )
                    .with_help(
                        "for a multi-line condition, wrap it in `(...)` or put the operator at the end of the previous line",
                    ),
                );
            } else {
                let mut diag = Diagnostic::error(
                    codes::EXPECTED_TOKEN,
                    at,
                    format!("expected `:` after {after}, found {found}"),
                );
                // A line that already ends in `:` lost something else, so
                // a second `:` would not fix it.
                let ends_in_colon = self.tokens[self.pos..]
                    .iter()
                    .take_while(|t| !matches!(t.kind, T::Newline | T::Eof))
                    .last()
                    .is_some_and(|t| t.kind == T::Colon);
                if self.at(T::Eq)
                    && after.contains("condition")
                    && expr::begins_expr(self.peek_n(1))
                {
                    diag = diag
                        .with_help("`=` assigns; to compare, write `==`")
                        .with_fix("change to `==`", at, "==");
                } else if let Some(edits) = self.brace_header_edits() {
                    diag.help = Some("the body is indented under `:`, with no braces".into());
                    let title = "write `:` and drop the braces".into();
                    diag.fixes.push(cpl_diag::Fix { title, edits });
                } else if !ends_in_colon {
                    diag = diag.with_fix("insert `:`", Span::point(at.start), ":");
                }
                self.error(diag);
            }
            // The body is still read if it is there.
            if !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
                self.skip_line();
                return Block {
                    stmts: Vec::new(),
                    span: Span::point(self.prev_end()),
                };
            }
        }
        self.block(after)
    }

    /// Parses the block after a `:`: an indented block on the following
    /// lines, or one statement on the same line.
    pub(super) fn block(&mut self, after: &str) -> Block {
        self.block_depth += 1;
        let block = self.block_body(after);
        self.block_depth -= 1;
        block
    }

    fn block_body(&mut self, after: &str) -> Block {
        let header_end = self.prev_end();
        if self.at(T::Newline) && self.peek_n(1) == T::Indent {
            self.bump();
            self.bump();
            // Whatever was wrong with the header stays with the header.
            let was_recovering = std::mem::replace(&mut self.recovering, false);
            let start = self.start();
            let mut stmts = Vec::new();
            while !matches!(self.peek(), T::Dedent | T::Eof) {
                let before = self.pos;
                self.stmt_into(&mut stmts);
                if self.pos == before {
                    self.bump();
                }
            }
            let span = self.span_from(start);
            self.eat(T::Dedent);
            self.recovering = was_recovering;
            return Block { stmts, span };
        }
        if self.at_stmt_end() {
            let at = self.block_expected_at(header_end);
            let mut diag = Diagnostic::error(
                codes::EXPECTED_BLOCK,
                at,
                format!("expected an indented block after {after}"),
            )
            .with_help("indent the next line by 4 spaces");
            // A next line no deeper than the header becomes its body.
            let abs = |p: u32| (p as i64 - self.delta) as usize;
            let line = |p: usize| self.src[..p].rfind('\n').map_or(0, |i| i + 1);
            let header = &self.src[line(abs(header_end))..];
            let indent = header.len() - header.trim_start_matches(' ').len();
            let lead = abs(at.start) - line(abs(at.start));
            if at.start != at.end && lead <= indent {
                let span = Span::new(at.start - lead as u32, at.start);
                diag = diag.with_fix("indent the next line", span, " ".repeat(indent + 4));
            }
            let simple = lead == indent && at.start != at.end;
            if simple
                && self.at(T::Newline)
                && !matches!(self.peek_n(1), T::Indent | T::Dedent | T::Eof)
            {
                self.error(diag);
                self.bump();
                let start = self.start();
                let mut stmts = Vec::new();
                self.stmt_into(&mut stmts);
                // The header's own statement ends at the body's newline.
                if self.tokens[self.pos - 1].kind == T::Newline {
                    self.pos -= 1;
                }
                return Block {
                    stmts,
                    span: self.span_from(start),
                };
            }
            self.error(diag);
            return Block {
                stmts: Vec::new(),
                span: Span::point(header_end),
            };
        }
        // One statement on the same line: `if x < 0: return none`.
        // `if c: a; b` keeps both in the block.
        let start = self.start();
        let outer = std::mem::take(&mut self.lead_stmts);
        let mut stmts = Vec::new();
        loop {
            let at = self.start();
            let kind = self.stmt_kind();
            stmts.append(&mut self.lead_stmts);
            stmts.push(Stmt {
                kind,
                span: self.span_from(at),
                decorators: Vec::new(),
                public: false,
            });
            let more = self.brace_depth == 0
                && self.at(T::Semi)
                && !matches!(self.peek_n(1), T::Newline | T::Eof | T::Dedent);
            if !more {
                break;
            }
            self.bump();
        }
        self.lead_stmts = outer;
        Block {
            stmts,
            span: self.span_from(start),
        }
    }

    /// Parses `{ stmt; stmt; expr }`. The `{` is already consumed.
    pub(super) fn brace_block(&mut self, start: u32) -> Block {
        self.brace_depth += 1;
        self.block_depth += 1;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), T::RBrace | T::Eof | T::Newline | T::Dedent) {
            let before = self.pos;
            if !self.eat(T::Semi) {
                stmts.push(self.stmt());
            }
            if self.pos == before {
                self.bump();
            }
        }
        self.brace_depth -= 1;
        self.block_depth -= 1;
        self.expect(T::RBrace, "`}` to close the block");
        Block {
            stmts,
            span: self.span_from(start),
        }
    }
}
