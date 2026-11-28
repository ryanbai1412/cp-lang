//! Statements and blocks. A broken statement ends at its `Newline`; the
//! block's other statements, and a broken header's block, still parse.

use super::*;
use crate::grammar::{contextual, Contextual};

impl Parser<'_> {
    /// Statements up to `end`, or a `Dedent` or the end of input.
    pub(super) fn stmts_until(&mut self, end: TokenKind) {
        while !matches!(self.peek(), T::Dedent | T::Eof) && !self.at(end) {
            let before = self.pos;
            self.stmt();
            if self.pos == before {
                self.start(NodeKind::Error);
                self.bump();
                self.finish();
            }
        }
    }

    /// `@trusted:`, a statement and not a decorator (R09.5.4).
    pub(super) fn trusted_ahead(&self) -> bool {
        self.at(T::At)
            && self.glued(1)
            && self.nth(1) == T::Ident
            && contextual(self.nth_text(1)) == Some(Contextual::Trusted)
            && self.nth(2) == T::Colon
    }

    pub(super) fn word(&self) -> Option<Contextual> {
        if self.at(T::Ident) {
            contextual(self.text())
        } else {
            None
        }
    }

    pub(super) fn stmt(&mut self) {
        match self.peek() {
            T::Newline => self.bump(),
            T::Indent => self.stray_indent(|p| p.stmts_until(T::Dedent)),
            // Kept whole so that decls says where a directive belongs.
            T::At if self.nth(1) == T::Bang && self.glued(1) => self.directive(),
            T::At if self.trusted_ahead() => {
                self.start(NodeKind::TrustedStmt);
                self.bump();
                self.bump();
                self.block();
                self.finish();
            }
            _ if self.item_ahead() => self.item(),
            T::If => self.if_clauses(NodeKind::IfStmt, Self::block),
            T::While => self.while_stmt(),
            T::For => self.for_stmt(),
            T::Label if self.nth(1) == T::Colon && matches!(self.nth(2), T::For | T::While) => {
                if self.nth(2) == T::For {
                    self.for_stmt()
                } else {
                    self.while_stmt()
                }
            }
            T::Match => self.match_expr(),
            T::Unchecked => self.headed(NodeKind::UncheckedStmt, |_| {}),
            T::Ident if self.word() == Some(Contextual::Repeat) && self.nth(1) != T::Eq => {
                self.headed(NodeKind::RepeatStmt, |p| p.expr())
            }
            T::Ident if self.word() == Some(Contextual::Guard) && self.nth(1) == T::Let => {
                self.guard_let()
            }
            T::Ident if self.word() == Some(Contextual::Loop) && self.nth(1) == T::Colon => {
                let span = self.span();
                self.diagnostics
                    .push(msgs::syntax::LOOP_KEYWORD.error(span, &[]).with_fix(
                        cpl_diag::habits::REPLACE_WITH.say(&[&"while true"]),
                        span,
                        "while true",
                    ));
                self.start(NodeKind::WhileStmt);
                self.bump();
                self.block();
                self.loop_else();
                self.finish();
            }
            // `with io(base: 1)` with no block covers the rest of the file.
            T::Ident if self.word() == Some(Contextual::With) && self.nth(1) == T::Ident => {
                self.start(NodeKind::WithStmt);
                self.bump();
                self.expr();
                while self.eat(T::Comma) {
                    self.expr();
                }
                if self.at(T::Colon) {
                    self.block();
                }
                self.finish();
                self.end_line();
            }
            _ => {
                self.simple();
                while self.eat(T::Semi) {
                    if !matches!(self.peek(), T::Newline | T::Eof | T::Dedent) {
                        self.simple();
                    }
                }
                self.end_line();
            }
        }
        // A compound statement whose block ended on its own line.
        if self.at(T::Newline) {
            self.bump();
        }
    }

    /// An indented run where a statement was wanted: one diagnostic, and
    /// its statements are kept.
    fn stray_indent(&mut self, inner: fn(&mut Self)) {
        self.diagnostics.push(
            msgs::stmt::STRAY_INDENT
                .error(Span::point(self.span().start), &[])
                .with_help("remove the extra indentation, or end the line above with `:`"),
        );
        self.bump();
        inner(self);
        self.eat(T::Dedent);
    }

    /// The end of a simple statement's line. Anything left on it is one
    /// `Error` node with one diagnostic.
    pub(super) fn end_line(&mut self) {
        if matches!(self.peek(), T::Newline | T::Eof | T::Dedent) || self.after_line_end() {
            return;
        }
        self.error_here(codes::UNEXPECTED_TOKEN, "the end of the line");
        self.start(NodeKind::Error);
        while !matches!(self.peek(), T::Newline | T::Eof | T::Dedent) {
            self.bump();
        }
        self.finish();
    }

    /// Whether the last token taken ended a line: a construct whose block
    /// closed with its `Dedent` has nothing left on its line.
    fn after_line_end(&self) -> bool {
        self.tokens[..self.pos]
            .iter()
            .rev()
            .find(|t| !t.kind.is_trivia())
            .is_none_or(|t| matches!(t.kind, T::Newline | T::Dedent))
    }

    /// `keyword header block`.
    fn headed(&mut self, node: NodeKind, header: impl FnOnce(&mut Self)) {
        self.start(node);
        self.bump();
        header(self);
        self.block();
        self.finish();
    }

    pub(super) fn simple(&mut self) {
        match self.peek() {
            T::Let | T::Var => {
                let at = self.checkpoint();
                self.let_stmt(at);
            }
            T::Const => {
                let at = self.checkpoint();
                self.const_decl(at);
            }
            T::Return => self.keyword_stmt(NodeKind::ReturnStmt, true),
            T::Yield => self.keyword_stmt(NodeKind::YieldStmt, true),
            T::Break => self.keyword_stmt(NodeKind::BreakStmt, false),
            T::Continue => self.keyword_stmt(NodeKind::ContinueStmt, false),
            T::Ident
                if self.word() == Some(Contextual::Lend)
                    && matches!(self.nth(1), T::Mut | T::Ident | T::SelfKw) =>
            {
                self.start(NodeKind::LendStmt);
                self.bump();
                self.eat(T::Mut);
                self.expr();
                self.finish();
            }
            T::Assert => {
                self.start(NodeKind::AssertStmt);
                self.bump();
                self.expr();
                if self.eat(T::Comma) {
                    self.expr();
                }
                self.finish();
            }
            _ => self.expr_or_assign(true),
        }
    }

    /// `return e`, `yield e`, `break 'outer`, `continue`.
    fn keyword_stmt(&mut self, node: NodeKind, value: bool) {
        self.start(node);
        self.bump();
        if value && !matches!(self.peek(), T::Newline | T::Eof | T::Dedent | T::Semi) {
            self.expr();
        }
        if !value {
            self.eat(T::Label);
        }
        self.finish();
    }

    pub(super) fn let_stmt(&mut self, at: Checkpoint) {
        if self.at(T::Let) && self.nth(1) == T::LParen && self.nth(2) == T::Domain {
            return self.domain_let();
        }
        self.start_at(at, NodeKind::LetStmt);
        self.bump();
        self.pattern();
        while self.eat(T::Comma) {
            self.pattern();
        }
        if self.eat(T::Colon) {
            self.ty();
        }
        if self.want(T::Eq, "`=` and a value") {
            self.expr();
            if self.at(T::Else) {
                self.start(NodeKind::ElseClause);
                self.bump();
                self.block();
                self.finish();
            }
        }
        self.finish();
    }

    /// An expression, or an assignment to it: `p = e`, `p += e`, `p f= e`.
    /// As a statement (`stmt`), a plain expression is an `ExprStmt`; as an
    /// arm's or a lambda's body it is the value itself.
    fn expr_or_assign(&mut self, stmt: bool) {
        let at = self.checkpoint();
        self.expr();
        let named = self.at(T::Ident) && self.nth(1) == T::Eq && self.glued(1);
        if matches!(self.peek(), T::Eq | T::OpEq) || named {
            self.start_at(at, NodeKind::AssignStmt);
            if named {
                self.bump();
            }
            self.bump();
            self.expr();
        } else if stmt {
            self.start_at(at, NodeKind::ExprStmt);
        } else {
            return;
        }
        self.finish();
    }

    /// `:` then the statements on the line, or a newline and an indented
    /// run. A missing `:` is one diagnostic, and an indented body after it
    /// still parses as the block.
    pub(super) fn block(&mut self) {
        self.start(NodeKind::Block);
        let colon = self.at(T::Colon);
        if colon {
            self.bump();
        } else {
            self.error_here(codes::EXPECTED_TOKEN, "`:` and a block");
        }
        if self.at(T::Newline) && self.nth(1) == T::Indent {
            self.bump();
            self.bump();
            self.stmts_until(T::Dedent);
            self.want(T::Dedent, "the end of the block");
        } else if colon && !matches!(self.peek(), T::Newline | T::Eof | T::Dedent) {
            self.simple();
            while self.eat(T::Semi) {
                self.simple();
            }
        } else if colon {
            self.error_here(codes::EXPECTED_TOKEN, "a statement after `:`");
        }
        self.finish();
    }

    /// `if c <body> (elif c <body>)* (else <body>)?`, as a statement with
    /// blocks or a value with `: expr` bodies.
    pub(super) fn if_clauses(&mut self, node: NodeKind, body: fn(&mut Self)) {
        self.start(node);
        self.bump();
        self.eat(T::Const);
        self.cond();
        body(self);
        // A value's `elif`/`else` may sit on deeper lines below it (R01.2.4):
        // the parser reads that layout, so the shared lexer stays as the
        // frozen `cpl` reads it.
        let mut indented = false;
        loop {
            let next = match (self.peek(), self.nth(1), self.nth(2)) {
                (T::Elif | T::Else, _, _) => 0,
                (T::Newline, T::Elif | T::Else, _) => 1,
                (T::Newline, T::Indent, T::Elif | T::Else)
                    if node == NodeKind::IfExpr && !indented =>
                {
                    2
                }
                _ => break,
            };
            for _ in 0..next {
                self.bump();
            }
            indented |= next == 2;
            let elif = self.at(T::Elif);
            self.start(if elif {
                NodeKind::ElifClause
            } else {
                NodeKind::ElseClause
            });
            self.bump();
            if elif {
                self.cond();
            }
            body(self);
            self.finish();
            if !elif {
                break;
            }
        }
        if indented {
            if self.at(T::Newline) && self.nth(1) == T::Dedent {
                self.bump();
            }
            self.want(T::Dedent, "the end of the indented `elif`/`else` lines");
        }
        self.finish();
    }

    /// `c, let p = e, d`.
    /// `T: Ord + Hash<K>:`, a bound test (R08), told from a one-line
    /// `if ok: x` by the `:` that ends the bound.
    fn bound_cond_ahead(&self) -> bool {
        if !(self.at(T::Ident) && self.nth(1) == T::Colon) {
            return false;
        }
        let mut n = 2;
        loop {
            if self.nth(n) != T::Ident {
                return false;
            }
            n += 1;
            while self.nth(n) == T::Dot && self.nth(n + 1) == T::Ident {
                n += 2;
            }
            if self.nth(n) == T::Lt {
                let mut depth = 0i32;
                loop {
                    match self.nth(n) {
                        T::Lt => depth += 1,
                        T::Gt => depth -= 1,
                        T::Shr => depth -= 2,
                        T::Newline | T::Eof => return false,
                        _ => {}
                    }
                    n += 1;
                    if depth <= 0 {
                        break;
                    }
                }
            }
            if self.nth(n) != T::Plus {
                return self.nth(n) == T::Colon;
            }
            n += 1;
        }
    }

    pub(super) fn cond(&mut self) {
        self.start(NodeKind::Condition);
        self.cond_part();
        while self.eat(T::Comma) {
            self.cond_part();
        }
        self.finish();
    }

    /// A clause's `if` filter (01:356): a condition, where a `,`
    /// before `p in e` starts the next clause instead.
    pub(super) fn filter(&mut self) {
        self.start(NodeKind::Condition);
        self.cond_part();
        while self.at(T::Comma) && !self.clause_after_comma() {
            self.bump();
            self.cond_part();
        }
        self.finish();
    }

    /// `, x in e` or `, (a, b) in e` at the cursor.
    pub(super) fn clause_after_comma(&self) -> bool {
        match self.nth(1) {
            T::Ident => self.nth(2) == T::In,
            T::LParen => {
                let (mut depth, mut n) = (0usize, 1);
                loop {
                    match self.nth(n) {
                        T::LParen => depth += 1,
                        T::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                return self.nth(n + 1) == T::In;
                            }
                        }
                        T::Eof | T::Newline => return false,
                        _ => {}
                    }
                    n += 1;
                }
            }
            _ => false,
        }
    }

    /// One part of a condition: `let p = e`, a bound test, or a `Bool`.
    pub(super) fn cond_part(&mut self) {
        if self.at(T::Let) {
            self.start(NodeKind::LetCond);
            self.bump();
            self.pattern();
            if self.want(T::Eq, "`=` and the value to match") {
                let outer = std::mem::replace(&mut self.no_and, true);
                self.expr();
                self.no_and = outer;
            }
            self.finish();
            if self.at(T::And) {
                self.and_after_let();
                self.cond_part();
            }
        } else if self.bound_cond_ahead() {
            self.start(NodeKind::BoundCond);
            self.start(NodeKind::NameRef);
            self.bump();
            self.finish();
            self.bump();
            self.bound();
            self.finish();
        } else {
            self.expr();
        }
    }

    /// `guard let p = e else: leave`: one spelling, the let-else (R01.6.2);
    /// the fix deletes `guard`.
    fn guard_let(&mut self) {
        let word = self.span();
        let to_let = Span::new(word.start, self.nth_span(1).start);
        let err = msgs::syntax::GUARD_LET.error(word, &[]);
        self.diagnostics
            .push(err.with_fix("delete `guard`", to_let, ""));
        self.start(NodeKind::Error);
        self.bump();
        self.finish();
        self.stmt();
    }

    /// `if let p = e and c:`: a `let` joins what follows with `,` (01:505);
    /// the fix writes `,`.
    fn and_after_let(&mut self) {
        let and = self.span();
        let prev = (self.tokens[..self.pos].iter().rev())
            .find(|t| !t.kind.is_trivia())
            .map_or(and.start, |t| t.span.end);
        let err = msgs::syntax::AND_LET.error(and, &[]);
        let fix = Span::new(prev, and.end);
        self.diagnostics.push(err.with_fix("write `,`", fix, ","));
        self.start(NodeKind::Error);
        self.bump();
        self.finish();
    }

    fn while_stmt(&mut self) {
        self.start(NodeKind::WhileStmt);
        if self.eat(T::Label) {
            self.bump();
        }
        self.bump();
        self.cond();
        self.block();
        self.loop_else();
        self.finish();
    }

    fn for_stmt(&mut self) {
        self.start(NodeKind::ForStmt);
        if self.eat(T::Label) {
            self.bump();
        }
        self.bump();
        loop {
            self.start(NodeKind::ForClause);
            self.pattern();
            if self.want(T::In, "`in` and what to walk") {
                self.expr();
            }
            if self.eat(T::If) {
                self.filter();
            }
            self.finish();
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.block();
        self.loop_else();
        self.finish();
    }

    fn loop_else(&mut self) {
        let next = usize::from(self.at(T::Newline));
        if self.nth(next) == T::Else {
            if next == 1 {
                self.bump();
            }
            self.start(NodeKind::ElseClause);
            self.bump();
            self.block();
            self.finish();
        }
    }

    /// `match e:` and its arms, on the line separated by `,` or one per
    /// indented line.
    pub(super) fn match_expr(&mut self) {
        self.start(NodeKind::MatchExpr);
        self.bump();
        self.expr();
        self.want(T::Colon, "`:` and the arms");
        if self.at(T::Newline) && self.nth(1) == T::Indent {
            self.bump();
            self.bump();
            self.arm_lines();
            self.want(T::Dedent, "the end of the arms");
        } else {
            self.arm();
            while self.eat(T::Comma) {
                self.arm();
            }
        }
        self.finish();
    }

    /// One arm per line up to the `Dedent`; a stray indented run is one
    /// diagnostic and its lines are arms too.
    fn arm_lines(&mut self) {
        while !matches!(self.peek(), T::Dedent | T::Eof) {
            if self.eat(T::Newline) {
                continue;
            }
            if self.at(T::Indent) {
                self.stray_indent(Self::arm_lines);
                continue;
            }
            let before = self.pos;
            self.arm();
            self.end_line();
            if self.pos == before {
                self.start(NodeKind::Error);
                self.bump();
                self.finish();
            }
        }
    }

    fn arm(&mut self) {
        self.start(NodeKind::MatchArm);
        self.pattern();
        // A guard is never a lambda: its `=>` starts the arm's value.
        if self.eat(T::If) {
            self.expr_at(crate::ops::Level::Expr);
        }
        // `p: body` is not an arm (decision 4.1): one error whose fix
        // writes `=>`, and the body after it is still kept.
        if self.at(T::Colon) {
            let span = self.span();
            self.diagnostics
                .push(msgs::syntax::COLON_ARM.error(span, &[]).with_fix(
                    cpl_diag::habits::REPLACE_WITH.say(&[&"=>"]),
                    span,
                    " =>",
                ));
            self.block();
        } else if self.want(T::FatArrow, "`=>` and the arm's value") {
            self.arm_body();
        }
        self.finish();
    }

    /// An arm's or a lambda's body: an expression or an assignment on the
    /// line, a statement that leaves (`break`, `continue`, `return`,
    /// `assert`, typed Never), or an indented run of statements.
    pub(super) fn arm_body(&mut self) {
        if self.word() == Some(Contextual::Do) && self.nth(1) == T::Colon {
            return self.do_block();
        }
        if self.indented_block() {
            return;
        }
        match self.peek() {
            T::Return | T::Break | T::Continue | T::Assert => self.simple(),
            _ => self.expr_or_assign(false),
        }
    }

    /// `=> do: ...`: the block follows `=>` itself (decision 4.5).
    fn do_block(&mut self) {
        let word = self.span();
        let prev = (self.tokens[..self.pos].iter().rev())
            .find(|t| !t.kind.is_trivia())
            .map_or(word.start, |t| t.span.end);
        let err = msgs::syntax::DO_BLOCK.error(word, &[]);
        let fix = Span::new(prev, self.nth_span(1).end);
        self.diagnostics.push(err.with_fix("delete `do:`", fix, ""));
        self.start(NodeKind::Error);
        self.bump();
        self.finish();
        self.block();
    }

    /// An `if` expression's branch: a value on the line or an indented
    /// block, as an arm's (decision 4.5).
    pub(super) fn value_body(&mut self) {
        if !self.indented_block() {
            self.expr();
        }
    }

    fn indented_block(&mut self) -> bool {
        if !(self.at(T::Newline) && self.nth(1) == T::Indent) {
            return false;
        }
        self.start(NodeKind::Block);
        self.bump();
        self.bump();
        self.stmts_until(T::Dedent);
        self.want(T::Dedent, "the end of the block");
        self.finish();
        true
    }
}
