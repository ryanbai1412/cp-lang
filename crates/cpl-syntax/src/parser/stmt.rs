//! Statements and blocks.

use super::*;

impl<'a> Parser<'a> {
    /// Parses one statement into `out`. An unexpectedly indented run of
    /// lines is reported once and its statements are kept at this level.
    pub(super) fn stmt_into(&mut self, out: &mut Vec<Stmt>) {
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

    fn stmt(&mut self) -> Stmt {
        let start = self.start();
        // `pub @get fn count(self)`: `pub` may come before the decorators.
        let mut public = self.at(T::Pub) && self.peek_n(1) == T::At;
        if public {
            self.bump();
        }
        let mut decorators = Vec::new();
        // `@law assoc(a, b, c) = op(op(a, b), c) == op(a, op(b, c))`.
        if self.at(T::At)
            && self.peek_n(1) == T::Ident
            && self.peek_n(2) == T::Ident
            && self.tokens.get(self.pos + 1).is_some_and(|t| t.span.text(self.src) == "law")
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
        while self.at(T::At) {
            decorators.push(self.decorator());
            if !self.eat(T::Newline)
                && !self.at_line_end()
                && decorators.last().is_some_and(|d| d.name.text.is_empty())
            {
                break;
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
            || (self.at(T::Semi) && !matches!(self.peek_n(1), T::Newline | T::Eof | T::Dedent) && {
                self.bump();
                true
            });
        if !ended {
            if self.at(T::Indent) {
                // Reported by `stmt_into` as an unexpected indent.
                self.end_recovery(start);
                return;
            }
            let found = self.peek();
            let span = self.span();
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
            T::Const => self.const_stmt(),
            T::Fn => StmtKind::Fn(self.fn_decl()),
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
                StmtKind::While { cond, body }
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
                self.error(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        format!("this `{word}` has no `if` above it at the same indentation"),
                    )
                    .with_help(format!("line `{word}` up with its `if`")),
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
                StmtKind::While {
                    cond: Cond::Expr(Expr {
                        kind: ExprKind::Bool(true),
                        span,
                    }),
                    body,
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

    fn let_stmt(&mut self) -> StmtKind {
        let mutable = self.at(T::Var);
        self.bump();
        let mut pat = self.pattern();
        // `var ones, one_twos, ans: Z = (0, 0, 0)` and `let n, m: 1..=20 = read()`:
        // homogeneous multi-binding desugars to a tuple pattern with a repeated
        // tuple type `(T, T, ...)`, so scalar broadcast (`var a, b: Int = 0`) is
        // cleanly rejected as a tuple/scalar mismatch.
        let mut shared_count = 0usize;
        if matches!(&pat.kind, PatternKind::Name(_))
            && self.at(T::Comma)
            && self.peek_n(1) == T::Ident
            && matches!(self.peek_n(2), T::Comma | T::Colon)
        {
            let start = pat.span.start;
            let mut pats = vec![pat];
            while self.eat(T::Comma) {
                pats.push(self.pattern());
            }
            shared_count = pats.len();
            pat = Pattern {
                kind: PatternKind::Tuple(pats),
                span: self.span_from(start),
            };
        }
        let ty = self.eat(T::Colon).then(|| {
            let elem = self.ty();
            if shared_count > 1 {
                let span = elem.span;
                Type {
                    kind: TypeKind::Tuple(vec![elem; shared_count]),
                    span,
                }
            } else {
                elem
            }
        });
        let init = if self.eat(T::Eq) {
            Some(self.expr())
        } else {
            if ty.is_none() {
                let word = if mutable { "var" } else { "let" };
                self.error_expected(
                    codes::EXPECTED_TOKEN,
                    &format!("`=` and a value, or `: Type`, after the name in `{word}`"),
                    None,
                );
            }
            None
        };
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
        let pat = self.pattern();
        self.expect(T::In, "`in` after the loop variable");
        let iter = if self.at(T::Question) {
            self.optional_type_iter()
        } else {
            self.expr()
        };
        // `for x in xs where cond:` (0.3) or `for x in xs if cond:` (0.2.2)
        // runs the body for the `x` that pass: the body is an `if`.
        let has_filter = self.eat(T::If)
            || (self.at_ident("where") && {
                self.bump();
                true
            });
        let filter = has_filter.then(|| (self.start(), self.cond()));
        let mut body = self.colon_block("the `for` header");
        if let Some((start, cond)) = filter {
            let span = self.span_from(start);
            let test = Expr {
                kind: ExprKind::If {
                    branches: vec![(cond, body)],
                    otherwise: None,
                },
                span,
            };
            body = Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Expr(test),
                    span,
                    decorators: Vec::new(),
                    public: false,
                }],
                span,
            };
        }
        StmtKind::For {
            label,
            pat,
            iter,
            body,
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

    fn expr_stmt(&mut self) -> StmtKind {
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
            return Cond::Guard { test, and: Box::new(self.more_clauses()) };
        }
        Cond::Expr(test)
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
        } else if self.eat(T::And) {
            Some(Box::new(if self.at(T::Let) {
                self.bump();
                self.let_cond()
            } else {
                Cond::Expr(self.and())
            }))
        } else if self.at_ident("where") {
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
            return Cond::Guard { test, and: Box::new(self.more_clauses()) };
        }
        Cond::Expr(test)
    }

    /// Parses `: block`. `after` names what the colon follows.
    pub(super) fn colon_block(&mut self, after: &str) -> Block {
        if !self.eat(T::Colon) {
            let at = self.error_span();
            let found = self.peek().describe();
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_TOKEN,
                    at,
                    format!("expected `:` after {after}, found {found}"),
                )
                .with_fix("insert `:`", Span::point(at.start), ":"),
            );
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
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_BLOCK,
                    Span::point(header_end),
                    format!("expected an indented block after {after}"),
                )
                .with_help("indent the next line by 4 spaces"),
            );
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
