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
    }

    fn stmt(&mut self) -> Stmt {
        let start = self.start();
        let mut decorators = Vec::new();
        while self.at(T::At) {
            decorators.push(self.decorator());
            if !self.eat(T::Newline) && !self.at_line_end() && decorators.last().is_some_and(|d| d.name.text.is_empty()) {
                break;
            }
        }
        let kind = self.stmt_kind();
        self.end_stmt(start);
        Stmt {
            kind,
            span: self.span_from(start),
            decorators,
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
        Decorator {
            name,
            args,
            span: self.span_from(start),
        }
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
                if found.is_closer() { codes::UNMATCHED_CLOSER } else { codes::UNEXPECTED_TOKEN },
                span,
                if found.is_closer() {
                    format!("this {} has no matching opener", found.describe())
                } else {
                    format!("expected the end of the statement, found {}", found.describe())
                },
            );
            if found.is_closer() {
                diag = diag.with_fix("remove it", span, "");
            } else if found == T::Semi {
                diag = diag
                    .with_help("statements end at the end of the line; `;` is only used inside `{ ... }`")
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

    fn stmt_kind(&mut self) -> StmtKind {
        if self.at(T::Pub) {
            self.bump();
        }
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
                let from = self.at_ident("from") && !matches!(self.peek_n(1), T::Newline | T::Dot | T::LParen | T::LBracket);
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
            T::Ident if self.text() == "pure" && self.peek_n(1) == T::Fn => {
                let span = self.span();
                self.error(
                    Diagnostic::error(codes::UNEXPECTED_TOKEN, span, "`pure fn` was removed; write `fn`")
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
        StmtKind::Let { mutable, pat, ty, init }
    }

    fn const_stmt(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a name after `const`");
        let ty = self.eat(T::Colon).then(|| self.ty());
        let init = if self.eat(T::Eq) {
            Some(self.expr())
        } else {
            if !self.in_trait {
                self.error_expected(codes::EXPECTED_TOKEN, "`=` and a value for the `const`", None);
            }
            None
        };
        StmtKind::Const { name, ty, init }
    }

    fn for_stmt(&mut self, label: Option<Name>) -> StmtKind {
        self.bump();
        let pat = self.pattern();
        self.expect(T::In, "`in` after the loop variable");
        let iter = self.expr();
        let body = self.colon_block("the `for` header");
        StmtKind::For { label, pat, iter, body }
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
                    _ => BinOp::Shr,
                })
            }
            // `best max= x`.
            T::Ident if self.peek_n(1) == T::Eq && matches!(self.text(), "max" | "min") => {
                let op = if self.text() == "max" { AssignOp::Max } else { AssignOp::Min };
                self.bump();
                op
            }
            _ => return StmtKind::Expr(target),
        };
        self.bump();
        let value = self.expr();
        StmtKind::Assign { target, op, value }
    }

    pub(super) fn cond(&mut self) -> Cond {
        if self.at(T::Let) {
            self.bump();
            let pat = self.pattern();
            self.expect(T::Eq, "`=` after the pattern in `if let`");
            return Cond::Let { pat, value: self.expr() };
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
