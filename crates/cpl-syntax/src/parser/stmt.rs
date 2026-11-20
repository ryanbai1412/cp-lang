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
        out.push(self.stmt());
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
        let kind = self.stmt_kind();
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
        let name = self.expect_name("a decorator name after `@`");
        let args = if self.eat(T::LParen) {
            self.comma_list(T::RParen, "the decorator's arguments", Self::arg)
        } else {
            Vec::new()
        };
        let span = self.span_from(start);
        // Decorator arguments v2.2 took away.
        let removed = match name.text.as_str() {
            "memo" => args.iter().find_map(|arg| match &arg.value.kind {
                ExprKind::Name(kind) if arg.name.is_none() && (kind == "flat" || kind == "hash") => Some((
                    format!("`@memo({kind})` was removed in CP Lang v2.2"),
                    "write bare `@memo`: the table is flat when the arguments are small and hashed otherwise",
                )),
                _ => None,
            }),
            "multitest" => args
                .iter()
                .any(|arg| arg.name.as_ref().is_some_and(|n| n.text == "count"))
                .then(|| {
                    (
                        "`@multitest(count:)` was removed in CP Lang v2.2".to_string(),
                        "read the count and loop: `for _ in 0..tests:`",
                    )
                }),
            "grader" if self.at(T::Const) => Some((
                "`@grader const` was removed in CP Lang v2.2".to_string(),
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
        Decorator { name, args, span }
    }

    /// Ends a statement: expects the end of the line (or `;` in braces).
    pub(super) fn end_stmt(&mut self, start: u32) {
        let ended = self.eat(T::Newline)
            || self.prev_kind() == Some(T::Dedent)
            || matches!(self.peek(), T::Dedent | T::Eof)
            || (self.brace_depth > 0 && matches!(self.peek(), T::Semi | T::RBrace));
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
                        format!("`{word}` was removed in CP Lang v2.2"),
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
        let pat = self.pattern();
        let ty = self.eat(T::Colon).then(|| self.ty());
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
                    "`where` guards were removed in CP Lang v2.2",
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
                    "`for mut` was removed in CP Lang v2.2",
                )
                .with_help("loop over the indexes and assign through them: `for i in 0..xs.len: xs[i] += 1`"),
            );
            self.bump();
        }
        let pat = self.pattern();
        self.expect(T::In, "`in` after the loop variable");
        let iter = self.expr();
        let body = self.colon_block("the `for` header");
        StmtKind::For {
            label,
            pat,
            iter,
            body,
        }
    }

    fn expr_stmt(&mut self) -> StmtKind {
        let target = self.expr();
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
    fn same_line(&self) -> bool {
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
            let pat = self.pattern();
            self.expect(T::Eq, "`=` after the pattern in `if let`");
            // The value ends before `and`, which adds a condition on
            // what the pattern bound.
            let value = self.not();
            let and = self.eat(T::And).then(|| {
                Box::new(if self.at(T::Let) {
                    self.cond()
                } else {
                    Cond::Expr(self.and())
                })
            });
            return Cond::Let { pat, value, and };
        }
        if self.at(T::Const) {
            self.bump();
            return Cond::Const(self.expr());
        }
        Cond::Expr(self.expr())
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
        let start = self.start();
        let kind = self.stmt_kind();
        let span = self.span_from(start);
        Block {
            stmts: vec![Stmt {
                kind,
                span,
                decorators: Vec::new(),
                public: false,
            }],
            span,
        }
    }

    /// Parses `{ stmt; stmt; expr }`. The `{` is already consumed.
    pub(super) fn brace_block(&mut self, start: u32) -> Block {
        self.brace_depth += 1;
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
        self.expect(T::RBrace, "`}` to close the block");
        Block {
            stmts,
            span: self.span_from(start),
        }
    }
}
