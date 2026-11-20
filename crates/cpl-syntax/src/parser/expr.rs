//! Expressions.

use super::*;
use crate::literal::{self, StrPart};

impl<'a> Parser<'a> {
    pub(super) fn expr(&mut self) -> Expr {
        let start = self.start();
        // Lambdas: `x => x + 1`, `(a, b) => a + b`.
        let lambda = match self.peek() {
            T::Ident => self.peek_n(1) == T::FatArrow,
            T::LParen => {
                self.tokens
                    .get(after_group(self.tokens, self.pos))
                    .map(|t| t.kind)
                    == Some(T::FatArrow)
            }
            _ => false,
        };
        if lambda {
            let params = if self.eat(T::LParen) {
                self.comma_list(T::RParen, "the lambda's parameters", Self::pattern)
            } else {
                vec![self.pattern()]
            };
            self.bump();
            let body = Box::new(self.expr());
            return Expr {
                kind: ExprKind::Lambda { params, body },
                span: self.span_from(start),
            };
        }
        let base = self.coalesce();
        if !self.at(T::With) {
            return base;
        }
        self.bump();
        let mut updates = Vec::new();
        loop {
            let name = self.expect_name("a field name after `with`");
            self.expect(T::Colon, "`:` and the field's new value");
            updates.push((name, self.coalesce()));
            // A following `name:` is another update; anything else belongs
            // to the enclosing list.
            if self.at(T::Comma) && self.peek_n(1) == T::Ident && self.peek_n(2) == T::Colon {
                self.bump();
            } else {
                break;
            }
        }
        Expr {
            kind: ExprKind::With {
                base: Box::new(base),
                updates,
            },
            span: self.span_from(start),
        }
    }

    fn binary(
        &mut self,
        start: u32,
        op: BinOp,
        lhs: Expr,
        rhs: impl FnOnce(&mut Self) -> Expr,
    ) -> Expr {
        let op_span = self.span();
        self.bump();
        let rhs = rhs(self);
        Expr {
            kind: ExprKind::Binary {
                op,
                op_span,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span: self.span_from(start),
        }
    }

    /// `a ?? b`, right-associative, binds loosest.
    fn coalesce(&mut self) -> Expr {
        let start = self.start();
        let lhs = self.or();
        if self.at(T::QuestionQuestion) {
            return self.binary(start, BinOp::Coalesce, lhs, Self::coalesce);
        }
        lhs
    }

    fn or(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.and();
        while self.at(T::Or) {
            lhs = self.binary(start, BinOp::Or, lhs, Self::and);
        }
        lhs
    }

    fn and(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.not();
        while self.at(T::And) {
            lhs = self.binary(start, BinOp::And, lhs, Self::not);
        }
        lhs
    }

    fn not(&mut self) -> Expr {
        if self.at(T::Not) {
            let start = self.start();
            self.bump();
            let expr = Box::new(self.not());
            return Expr {
                kind: ExprKind::Unary {
                    op: UnOp::Not,
                    expr,
                },
                span: self.span_from(start),
            };
        }
        self.comparison()
    }

    fn cmp_op(&self) -> Option<CmpOp> {
        Some(match self.peek() {
            T::EqEq => CmpOp::Eq,
            T::NotEq => CmpOp::Ne,
            T::Lt => CmpOp::Lt,
            T::LtEq => CmpOp::Le,
            T::Gt => CmpOp::Gt,
            T::GtEq => CmpOp::Ge,
            T::Spaceship => CmpOp::Order,
            T::In => CmpOp::In,
            T::Not if self.peek_n(1) == T::In => CmpOp::NotIn,
            _ => return None,
        })
    }

    /// `a < b`, and chains such as `lo <= x < hi`.
    fn comparison(&mut self) -> Expr {
        let start = self.start();
        let first = self.range();
        let mut rest = Vec::new();
        while let Some(op) = self.cmp_op() {
            self.bump();
            if op == CmpOp::NotIn {
                self.bump();
            }
            rest.push((op, self.range()));
        }
        if rest.is_empty() {
            return first;
        }
        Expr {
            kind: ExprKind::Compare {
                first: Box::new(first),
                rest,
            },
            span: self.span_from(start),
        }
    }

    fn starts_expr(&self) -> bool {
        matches!(
            self.peek(),
            T::Ident
                | T::Int
                | T::Float
                | T::Str
                | T::RawStr
                | T::Char
                | T::True
                | T::False
                | T::None
                | T::SelfKw
                | T::LParen
                | T::LBracket
                | T::LBrace
                | T::Minus
                | T::Tilde
                | T::If
                | T::Match
        )
    }

    /// `lo..hi`, `lo..=hi`, open ends, and `by step`.
    fn range(&mut self) -> Expr {
        let start = self.start();
        let lo = if matches!(self.peek(), T::DotDot | T::DotDotEq) {
            None
        } else {
            let lo = self.bit_or();
            if !matches!(self.peek(), T::DotDot | T::DotDotEq) {
                return lo;
            }
            Some(Box::new(lo))
        };
        let inclusive = self.at(T::DotDotEq);
        self.bump();
        let hi = if self.starts_expr() {
            Some(Box::new(self.bit_or()))
        } else {
            if inclusive {
                self.error_expected(
                    codes::EXPECTED_EXPR,
                    "the end of the range after `..=`",
                    None,
                );
            }
            None
        };
        let step = self.eat(T::By).then(|| Box::new(self.bit_or()));
        Expr {
            kind: ExprKind::Range {
                lo,
                hi,
                inclusive,
                step,
            },
            span: self.span_from(start),
        }
    }

    fn bit_or(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.bit_xor();
        while self.at(T::Pipe) {
            lhs = self.binary(start, BinOp::BitOr, lhs, Self::bit_xor);
        }
        lhs
    }

    fn bit_xor(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.bit_and();
        while self.at(T::Caret) {
            lhs = self.binary(start, BinOp::BitXor, lhs, Self::bit_and);
        }
        lhs
    }

    fn bit_and(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.shift();
        while self.at(T::Amp) {
            lhs = self.binary(start, BinOp::BitAnd, lhs, Self::shift);
        }
        lhs
    }

    fn shift(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.additive();
        loop {
            let op = match self.peek() {
                T::Shl => BinOp::Shl,
                T::Shr => BinOp::Shr,
                _ => return lhs,
            };
            lhs = self.binary(start, op, lhs, Self::additive);
        }
    }

    pub(super) fn additive(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.multiplicative();
        loop {
            let op = match self.peek() {
                T::Plus => BinOp::Add,
                // After a block, `-x` on the next line is a new statement.
                T::Minus if self.prev_kind() != Some(T::Dedent) => BinOp::Sub,
                _ => return lhs,
            };
            lhs = self.binary(start, op, lhs, Self::multiplicative);
        }
    }

    fn multiplicative(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.unary();
        loop {
            let op = match self.peek() {
                T::Star => BinOp::Mul,
                T::Slash => BinOp::Div,
                T::Percent => BinOp::Rem,
                _ => return lhs,
            };
            lhs = self.binary(start, op, lhs, Self::unary);
        }
    }

    pub(super) fn unary(&mut self) -> Expr {
        let op = match self.peek() {
            T::Minus => UnOp::Neg,
            T::Tilde => UnOp::BitNot,
            _ => return self.power(),
        };
        let start = self.start();
        self.bump();
        let expr = Box::new(self.unary());
        Expr {
            kind: ExprKind::Unary { op, expr },
            span: self.span_from(start),
        }
    }

    /// `x ** 2`, right-associative, binds tighter than unary minus on its
    /// left: `-x ** 2` is `-(x ** 2)`.
    fn power(&mut self) -> Expr {
        let start = self.start();
        let lhs = self.postfix();
        if self.at(T::StarStar) {
            return self.binary(start, BinOp::Pow, lhs, Self::unary);
        }
        lhs
    }

    pub(super) fn arg(&mut self) -> Arg {
        let name = self.at_label().then(|| {
            let name = self.name();
            self.bump();
            name
        });
        let mutable = self.eat(T::Mut);
        let start = self.start();
        let mut value = self.expr();
        // `max(dist[v] for v in V)`.
        if self.at(T::For) {
            let clauses = self.clauses();
            value = Expr {
                kind: ExprKind::Comprehension {
                    body: Box::new(value),
                    clauses,
                },
                span: self.span_from(start),
            };
        }
        Arg {
            name,
            mutable,
            value,
        }
    }

    fn postfix(&mut self) -> Expr {
        let start = self.start();
        let mut expr = self.primary();
        loop {
            // An expression that ended with a block ended there: a `(` or
            // `[` on the next line starts a new statement.
            if self.prev_kind() == Some(T::Dedent) {
                return expr;
            }
            let kind = match self.peek() {
                T::LParen => {
                    self.bump();
                    let args = self.comma_list(T::RParen, "the argument list", Self::arg);
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    }
                }
                T::LBracket => {
                    self.bump();
                    let indices = self.comma_list(T::RBracket, "the index", Self::expr);
                    ExprKind::Index {
                        base: Box::new(expr),
                        indices,
                    }
                }
                T::Dot => {
                    self.bump();
                    let name = if self.at(T::Ident) || self.at(T::Int) || self.field_keyword() {
                        self.name()
                    } else {
                        // Mid-edit `xs.`: keep the access so completion and
                        // hover still see the receiver.
                        let span = Span::point(self.prev_end());
                        self.error(Diagnostic::error(
                            codes::EXPECTED_NAME,
                            span,
                            "expected a field or method name after `.`",
                        ));
                        Name {
                            text: String::new(),
                            span,
                        }
                    };
                    ExprKind::Field {
                        base: Box::new(expr),
                        name,
                    }
                }
                T::Question => {
                    self.bump();
                    ExprKind::Try(Box::new(expr))
                }
                T::Bang => {
                    self.bump();
                    ExprKind::Unwrap(Box::new(expr))
                }
                T::ColonColon if self.peek_n(1) == T::Lt => {
                    self.bump();
                    let args = self.type_args();
                    ExprKind::Generic {
                        base: Box::new(expr),
                        args,
                    }
                }
                _ => return expr,
            };
            expr = Expr {
                kind,
                span: self.span_from(start),
            };
        }
    }

    /// Keywords that are also ordinary member names: `xs.min`, `V.none`.
    fn field_keyword(&self) -> bool {
        matches!(
            self.peek(),
            T::None | T::In | T::By | T::Type | T::Domain | T::Use | T::With
        ) && !self.at_line_end()
    }

    fn primary(&mut self) -> Expr {
        let start = self.start();
        let kind = match self.peek() {
            T::Int => {
                let value = literal::int_value(self.text()).unwrap_or(0);
                self.bump();
                ExprKind::Int(value)
            }
            T::Float => {
                let text = self.text().to_string();
                self.bump();
                ExprKind::Float(text)
            }
            T::Str => self.string(),
            T::RawStr => {
                let text = self.text();
                let inner = text[2..].strip_suffix('"').unwrap_or(&text[2..]);
                self.bump();
                ExprKind::Str(vec![StrPiece::Text(inner.to_string())])
            }
            T::Char => {
                let text = self.text();
                let inner = &text[1..text.len() - 1];
                self.bump();
                ExprKind::Char(unescape(inner).chars().next().unwrap_or('\0'))
            }
            T::True | T::False => {
                let value = self.at(T::True);
                self.bump();
                ExprKind::Bool(value)
            }
            T::None => {
                self.bump();
                ExprKind::None
            }
            T::SelfKw => {
                self.bump();
                ExprKind::SelfValue
            }
            // `do:` runs a block for its value; `?` inside exits the block.
            T::Ident if self.text() == "do" && self.peek_n(1) == T::Colon => {
                self.bump();
                self.bump();
                ExprKind::Block(self.block("`do`"))
            }
            T::Ident => {
                let name = self.name();
                let type_like = name.is_type_like();
                let expr = Expr {
                    kind: ExprKind::Name(name.text),
                    span: name.span,
                };
                // `Name<` starts type arguments only for a capitalized name
                // whose arguments parse and close, so `INF < x` stays a
                // comparison.
                if type_like && self.at(T::Lt) {
                    let args = self.speculate(|p| {
                        let args = p.type_args();
                        if args.is_empty()
                            || !matches!(p.peek(), T::LParen | T::Dot | T::ColonColon)
                                && !p.at_line_end()
                        {
                            p.speculation_failed = true;
                        }
                        args
                    });
                    if let Some(args) = args {
                        return Expr {
                            kind: ExprKind::Generic {
                                base: Box::new(expr),
                                args,
                            },
                            span: self.span_from(start),
                        };
                    }
                }
                return expr;
            }
            T::LParen => {
                self.bump();
                if self.at(T::Ident) && self.peek_n(1) == T::Colon {
                    let fields = self.comma_list(T::RParen, "the record", |p| {
                        let name = p.expect_name("a field name");
                        p.expect(T::Colon, "`:` after the field name");
                        (name, p.expr())
                    });
                    ExprKind::Record(fields)
                } else if self.eat(T::RParen) {
                    ExprKind::Tuple(Vec::new())
                } else {
                    let first = self.expr();
                    if self.at(T::Comma) {
                        self.bump();
                        let mut items = vec![first];
                        items.extend(self.comma_list(T::RParen, "the tuple", Self::expr));
                        ExprKind::Tuple(items)
                    } else {
                        self.expect(T::RParen, "`)` to close the parenthesis");
                        ExprKind::Paren(Box::new(first))
                    }
                }
            }
            T::LBracket => self.bracket(),
            T::LBrace => {
                self.bump();
                ExprKind::Block(self.brace_block(start))
            }
            T::If => self.if_expr(),
            T::Match => self.match_expr(),
            T::Bang => {
                let span = self.span();
                self.error(
                    Diagnostic::error(codes::UNEXPECTED_TOKEN, span, "`!` doesn't negate in cp")
                        .with_help("write `not x` to negate, or `x!` to unwrap an optional")
                        .with_fix("replace with `not `", span, "not "),
                );
                self.bump();
                return self.unary();
            }
            _ => {
                let after = self
                    .prev_kind()
                    .map_or("here".to_string(), |k| format!("after {}", k.describe()));
                self.error_expected(codes::EXPECTED_EXPR, &format!("a value {after}"), None);
                return Expr {
                    kind: ExprKind::Missing,
                    span: Span::point(self.prev_end()),
                };
            }
        };
        Expr {
            kind,
            span: self.span_from(start),
        }
    }

    /// `[a, b]`, `[body for x in xs if c]`, or `[V] u => body`.
    fn bracket(&mut self) -> ExprKind {
        // A table is `[Domain, ...]` followed directly by its parameters.
        let close = after_group(self.tokens, self.pos);
        let after = |n: usize| self.tokens.get(close + n).map(|t| t.kind);
        let table = match after(0) {
            Some(T::Ident) => after(1) == Some(T::FatArrow),
            Some(T::LParen) => {
                self.tokens
                    .get(after_group(self.tokens, close))
                    .map(|t| t.kind)
                    == Some(T::FatArrow)
            }
            _ => false,
        };
        self.bump();
        if table {
            let domains = self.comma_list(T::RBracket, "the table's domains", Self::ty);
            let params = if self.eat(T::LParen) {
                self.comma_list(T::RParen, "the table's parameters", Self::pattern)
            } else {
                vec![self.pattern()]
            };
            self.bump();
            let body = Box::new(self.expr());
            return ExprKind::Table {
                domains,
                params,
                body,
            };
        }
        if self.eat(T::RBracket) {
            return ExprKind::List(Vec::new());
        }
        let first = self.expr();
        if self.at(T::For) {
            let clauses = self.clauses();
            self.expect(T::RBracket, "`]` to close the comprehension");
            return ExprKind::Comprehension {
                body: Box::new(first),
                clauses,
            };
        }
        let mut items = vec![first];
        if self.eat(T::Comma) {
            items.extend(self.comma_list(T::RBracket, "the list", Self::expr));
        } else {
            self.expect(T::RBracket, "`,` or `]` in the list");
            if self.recovering {
                self.skip_to(T::RBracket);
                self.eat(T::RBracket);
            }
        }
        ExprKind::List(items)
    }

    /// Parses one or more `for pat in iter [if cond]`.
    fn clauses(&mut self) -> Vec<Clause> {
        let mut clauses = Vec::new();
        while self.eat(T::For) {
            let pat = self.pattern();
            self.expect(T::In, "`in` after the comprehension variable");
            // Stop before `if`, which here is the filter.
            let iter = self.coalesce();
            let cond = self.eat(T::If).then(|| self.coalesce());
            clauses.push(Clause { pat, iter, cond });
        }
        clauses
    }

    fn if_expr(&mut self) -> ExprKind {
        let mut branches = Vec::new();
        let mut otherwise = None;
        loop {
            // At `if` or `elif`.
            self.bump();
            let cond = self.cond();
            let body = self.colon_block("the condition");
            branches.push((cond, body));
            // `else` may sit on the next line after a one-line branch.
            if self.at(T::Newline) && matches!(self.peek_n(1), T::Elif | T::Else) {
                self.bump();
            }
            match self.peek() {
                T::Elif => continue,
                T::Else => {
                    self.bump();
                    // `else if` reads as `elif`.
                    if self.at(T::If) {
                        continue;
                    }
                    otherwise = Some(self.colon_block("`else`"));
                }
                _ => {}
            }
            break;
        }
        ExprKind::If {
            branches,
            otherwise,
        }
    }

    fn match_expr(&mut self) -> ExprKind {
        self.bump();
        let scrutinee = Box::new(self.expr());
        let mut arms = Vec::new();
        let colon = self.expect(T::Colon, "`:` after the value being matched");
        if !colon && !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
            self.skip_line();
            return ExprKind::Match { scrutinee, arms };
        }
        if !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
            let at = self.prev_end();
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_BLOCK,
                    Span::point(at),
                    "expected indented `pattern => value` arms after `match`",
                )
                .with_help("indent the next line by 4 spaces"),
            );
            return ExprKind::Match { scrutinee, arms };
        }
        self.bump();
        self.bump();
        let was_recovering = std::mem::replace(&mut self.recovering, false);
        while !matches!(self.peek(), T::Dedent | T::Eof) {
            let before = self.pos;
            let start = self.start();
            let pat = self.pattern();
            let guard = self.eat(T::If).then(|| self.expr());
            let body = if self.expect(T::FatArrow, "`=>` after the pattern") {
                self.block("`=>`")
            } else {
                self.skip_line();
                Block {
                    stmts: Vec::new(),
                    span: Span::point(self.prev_end()),
                }
            };
            self.end_stmt(start);
            arms.push(Arm { pat, guard, body });
            if self.pos == before {
                self.bump();
            }
        }
        self.eat(T::Dedent);
        self.recovering = was_recovering;
        ExprKind::Match { scrutinee, arms }
    }

    /// Splits an interpolating string and parses each `{expr}` in it.
    fn string(&mut self) -> ExprKind {
        let token = self.tokens[self.pos];
        let text = self.text();
        self.bump();
        let mut pieces = Vec::new();
        for part in literal::string_parts(text) {
            match part {
                StrPart::Text(span) => pieces.push(StrPiece::Text(unescape(span.text(text)))),
                StrPart::Expr(span) => {
                    let inner = span.text(text);
                    let lexed = crate::lexer::lex(inner);
                    // Spans from the inner lexer are relative to `inner`.
                    let delta = token.span.start as i64 + span.start as i64 + self.delta;
                    let mut sub = Parser::new(inner, &lexed.tokens, delta, Sink::new());
                    sub.speculating = self.speculating;
                    let expr = sub.expr();
                    if !matches!(sub.peek(), T::Newline | T::Eof) {
                        sub.error_expected(
                            codes::UNEXPECTED_TOKEN,
                            "`}` to end the interpolation",
                            None,
                        );
                    }
                    let failed = sub.speculation_failed;
                    let mut diags: Vec<Diagnostic> = lexed
                        .diagnostics
                        .into_iter()
                        .map(|d| shift_signed(d, delta))
                        .collect();
                    diags.extend(sub.sink.finish());
                    // One string, one diagnostic.
                    if let Some(diag) = diags.into_iter().next() {
                        self.error(diag);
                    }
                    self.speculation_failed |= failed;
                    pieces.push(StrPiece::Expr(expr));
                }
            }
        }
        ExprKind::Str(pieces)
    }
}

fn shift_signed(mut diag: Diagnostic, delta: i64) -> Diagnostic {
    let fix = |s: Span| {
        Span::new(
            (s.start as i64 + delta) as u32,
            (s.end as i64 + delta) as u32,
        )
    };
    diag.span = fix(diag.span);
    for label in &mut diag.labels {
        label.span = fix(label.span);
    }
    for f in &mut diag.fixes {
        for edit in &mut f.edits {
            edit.0 = fix(edit.0);
        }
    }
    diag
}

/// Processes backslash escapes and doubled braces in string text.
pub(crate) fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('0') => out.push('\0'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            },
            '{' | '}' if chars.peek() == Some(&c) => {
                chars.next();
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
