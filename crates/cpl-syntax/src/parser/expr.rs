//! Expressions.

use super::types::names_a_type;
use super::*;
use crate::literal::{self, StrPart};

impl<'a> Parser<'a> {
    pub(super) fn lambda_body(&mut self) -> Expr {
        if self.at(T::Newline) && self.peek_n(1) == T::Indent {
            let start = self.start();
            let block = self.block("the `=>`");
            Expr {
                kind: ExprKind::Block(block),
                span: self.span_from(start),
            }
        } else {
            // `(v, a) => cnt[a] -= 1`: a one-line body may be an assignment.
            let start = self.start();
            match self.expr_stmt() {
                StmtKind::Expr(e) => e,
                kind => {
                    let span = self.span_from(start);
                    let stmt = Stmt {
                        kind,
                        span,
                        decorators: Vec::new(),
                        public: false,
                    };
                    Expr {
                        kind: ExprKind::Block(Block {
                            stmts: vec![stmt],
                            span,
                        }),
                        span,
                    }
                }
            }
        }
    }

    pub(super) fn expr(&mut self) -> Expr {
        self.foreign_new();
        let start = self.start();
        if let Some(lambda) = self.foreign_lambda() {
            return lambda;
        }
        // Lambdas: `x => x + 1`, `(a, b) => a + b`, `(n: Int) -> Int => ...`.
        let lambda = match self.peek() {
            T::Ident => self.peek_n(1) == T::FatArrow,
            T::LParen => {
                let after = after_group(self.tokens, self.pos);
                matches!(
                    self.tokens.get(after).map(|t| t.kind),
                    Some(T::FatArrow) | Some(T::Arrow)
                )
            }
            _ => false,
        };
        if lambda {
            let (params, param_tys, param_mut) = if self.eat(T::LParen) {
                let items = self.comma_list(T::RParen, "the lambda's parameters", |p| {
                    let mutable = p.eat(T::Mut);
                    let pat = p.pattern();
                    let ty = p.eat(T::Colon).then(|| p.ty());
                    (pat, ty, mutable)
                });
                let has_ty = items.iter().any(|(_, t, _)| t.is_some());
                let has_mut = items.iter().any(|(_, _, m)| *m);
                let (ps, (ts, ms)): (Vec<_>, (Vec<_>, Vec<_>)) =
                    items.into_iter().map(|(p, t, m)| (p, (t, m))).unzip();
                (
                    ps,
                    if has_ty { ts } else { Vec::new() },
                    if has_mut { ms } else { Vec::new() },
                )
            } else {
                (vec![self.pattern()], Vec::new(), Vec::new())
            };
            let ret_ty = self.eat(T::Arrow).then(|| self.ty());
            if !self.eat(T::FatArrow) {
                self.error_expected(
                    codes::EXPECTED_TOKEN,
                    "`=>` to start the lambda body",
                    Some("=>"),
                );
            }
            let body = Box::new(self.lambda_body());
            return Expr {
                kind: ExprKind::Lambda {
                    params,
                    param_tys,
                    param_mut,
                    ret_ty,
                    body,
                },
                span: self.span_from(start),
            };
        }
        let first = self.pos;
        let base = self.coalesce();
        let base = self.python_ternary(start, first, base);
        if self.c_ternary_ahead(&base) {
            return self.c_ternary(start, first, base);
        }
        for (word, planned) in [
            ("default", &crate::planned::MAP_DEFAULT),
            ("then", &crate::planned::THEN_CHAIN),
        ] {
            if self.at_ident(word)
                && !matches!(
                    self.peek_n(1),
                    T::Colon | T::Eq | T::Comma | T::RParen | T::Newline | T::Eof
                )
            {
                let start = self.start();
                while self.at_ident(word) {
                    self.bump();
                    self.coalesce();
                }
                let span = self.span_from(start);
                self.planned(span, planned);
                return base;
            }
        }
        if !self.at_ident("with") {
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

    pub(super) fn binary(
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
    pub(super) fn coalesce(&mut self) -> Expr {
        let start = self.start();
        let lhs = self.or();
        if !self.at(T::QuestionQuestion) {
            return lhs;
        }
        let whole = self.binary(start, BinOp::Coalesce, lhs, Self::coalesce_rhs);
        if let ExprKind::Binary { lhs, rhs, .. } = &whole.kind {
            // `a + b ?? 0` reads two ways, so it has to say which. A way
            // out on the right (`cur + 1 ?? continue`) reads only one way.
            let diverges = matches!(rhs.kind, ExprKind::Block(_));
            // The left side is judged by the checker, which knows a step
            // along a domain (`i + 1 ?? I.last`) from a sum.
            let mixed = [rhs].into_iter().find(|side| {
                matches!(
                    side.kind,
                    ExprKind::Binary {
                        op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem,
                        ..
                    }
                )
            });
            if let (Some(side), false) = (mixed, diverges) {
                let span = side.span;
                let text = |e: &Expr| {
                    let abs = |p: u32| (p as i64 - self.delta) as usize;
                    self.src[abs(e.span.start)..abs(e.span.end)].to_string()
                };
                let help = match &side.kind {
                    ExprKind::Binary { op, lhs: l, rhs: r, .. } => format!(
                        "write `({} ?? {}) {} {}` to use whichever is taken, or `{} ?? ({})` for a default that is worked out",
                        text(lhs), text(l), op.text(), text(r), text(lhs), text(side)
                    ),
                    _ => unreachable!("`mixed` is a binary expression"),
                };
                self.error(
                    Diagnostic::error(
                        codes::UNPARENTHESIZED_COALESCE,
                        span,
                        "`??` mixed with `+ - * / %` needs parentheses",
                    )
                    .with_help(help),
                );
            }
        }
        whole
    }

    /// What follows `??`: a default, or a way out of the fn or the loop
    /// (`o ?? return -1`, `st.pop() ?? break`, `v + 1 ?? continue`).
    fn coalesce_rhs(&mut self) -> Expr {
        if !matches!(self.peek(), T::Return | T::Break | T::Continue) {
            return self.coalesce();
        }
        let start = self.start();
        let kind = self.stmt_kind();
        let span = self.span_from(start);
        let stmt = Stmt {
            kind,
            span,
            decorators: Vec::new(),
            public: false,
        };
        Expr {
            kind: ExprKind::Block(Block {
                stmts: vec![stmt],
                span,
            }),
            span,
        }
    }

    pub(super) fn or(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.and();
        while self.at(T::Or) || self.c_logic(T::Pipe, "or") {
            lhs = self.binary(start, BinOp::Or, lhs, Self::and);
        }
        lhs
    }

    fn at_is_pattern(&self) -> bool {
        self.at_ident("is") && self.same_line() && matches!(self.peek_n(1), T::Ident | T::LParen)
    }

    pub(super) fn and(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.not_or_is();
        // `and let` goes on to another binding, not another operand.
        while (self.at(T::And) && self.peek_n(1) != T::Let) || self.c_logic(T::Amp, "and") {
            lhs = self.binary(start, BinOp::And, lhs, Self::not_or_is);
        }
        lhs
    }

    /// An `and` operand, which may be a pattern test: `n >= 2 and hub is None`.
    fn not_or_is(&mut self) -> Expr {
        let start = self.start();
        let lhs = self.not();
        if !self.at_is_pattern() {
            return lhs;
        }
        self.bump();
        let pat = self.pattern();
        let rhs = if self.at(T::And) && self.peek_n(1) != T::Let {
            self.bump();
            self.and()
        } else {
            Expr {
                kind: ExprKind::Bool(true),
                span: self.span_from(start),
            }
        };
        let span = self.span_from(start);
        let wrap_block = |e: Expr| Block {
            stmts: vec![Stmt {
                public: false,
                decorators: Vec::new(),
                kind: StmtKind::Expr(e),
                span,
            }],
            span,
        };
        Expr {
            kind: ExprKind::If {
                branches: vec![(
                    Cond::Let {
                        pat,
                        value: lhs,
                        and: None,
                    },
                    wrap_block(rhs),
                )],
                otherwise: Some(wrap_block(Expr {
                    kind: ExprKind::Bool(false),
                    span,
                })),
            },
            span,
        }
    }

    /// `&&` where `and` goes on to another condition: reported, then
    /// read as `and`.
    pub(super) fn c_and(&mut self) -> bool {
        let found = self.c_logic(T::Amp, "and");
        if found {
            self.bump();
        }
        found
    }

    /// `&&` or `||` from C: reported with the word to write, then read as
    /// that word. Leaves the second character for [`Self::binary`].
    fn c_logic(&mut self, half: TokenKind, word: &str) -> bool {
        if !self.doubled(half) {
            return false;
        }
        let span = Span::new(self.start(), self.span_n(1).end);
        let op = if half == T::Amp { "&&" } else { "||" };
        self.error(
            Diagnostic::error(
                codes::UNEXPECTED_TOKEN,
                span,
                format!("`{op}` is `{word}` in CP Lang"),
            )
            .with_fix(format!("replace with `{word}`"), span, word),
        );
        self.bump();
        true
    }

    pub(super) fn not(&mut self) -> Expr {
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
            T::Ident if self.at_python_is() => CmpOp::Ne,
            T::In => CmpOp::In,
            T::Not if self.peek_n(1) == T::In => CmpOp::NotIn,
            T::Bang
                if self.peek_n(1) == T::In
                    && self
                        .tokens
                        .get(self.pos + 1)
                        .is_some_and(|t| t.span.start == self.tokens[self.pos].span.end) =>
            {
                CmpOp::NotIn
            }
            _ => return None,
        })
    }

    /// `a < b`, and chains such as `lo <= x < hi`.
    fn comparison(&mut self) -> Expr {
        let stmt_head = std::mem::take(&mut self.stmt_head);
        let start = self.start();
        let target = self.chain();
        // `if dist[v] min= d:` binds like a comparison, and is a `Bool`.
        let keep = !stmt_head
            && self.peek() == T::Ident
            && self.peek_n(1) == T::Eq
            && matches!(self.text(), "max" | "min")
            && self.same_line();
        if !keep {
            return target;
        }
        let max = self.text() == "max";
        self.bump();
        self.bump();
        let value = self.chain();
        Expr {
            kind: ExprKind::Keep {
                target: Box::new(target),
                max,
                value: Box::new(value),
            },
            span: self.span_from(start),
        }
    }

    fn chain(&mut self) -> Expr {
        let start = self.start();
        let first = self.range();
        if self.at(T::Lt)
            && self.peek_n(1) == T::DotDot
            && self.tokens[self.pos].span.end == self.tokens[self.pos + 1].span.start
        {
            let op = self.start();
            self.bump();
            self.bump();
            let span = self.span_from(op);
            self.planned(span, &crate::planned::START_EXCLUSIVE_RANGE);
            let hi = self.starts_expr().then(|| Box::new(self.bit_or()));
            return Expr {
                kind: ExprKind::Range {
                    lo: Some(Box::new(first)),
                    hi,
                    inclusive: false,
                    step: None,
                },
                span: self.span_from(start),
            };
        }
        let mut rest = Vec::new();
        while let Some(op) = self.cmp_op() {
            self.cmp_token();
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
        begins_expr(self.peek())
    }

    /// `a + * b`: an operator where a value goes, after an operator and
    /// before a value, is one too many. The span to delete.
    fn doubled_operator(&self) -> Option<Span> {
        let binary = |k: TokenKind| BINARY_OPS.contains(&k);
        let next = self.tokens.get(self.pos + 1)?;
        let after_op = self.prev_kind().is_some_and(binary);
        (after_op && binary(self.peek()) && begins_expr(next.kind))
            .then(|| Span::new(self.span().start, self.rel(next.span).start))
    }

    /// `lo..hi`, `lo..=hi`, open ends, and `by step`.
    fn range(&mut self) -> Expr {
        let start = self.start();
        let lo = if matches!(self.peek(), T::DotDot | T::DotDotEq) {
            None
        } else {
            let lo = self.bit_or();
            if !matches!(self.peek(), T::DotDot | T::DotDotEq) {
                return self.by(lo, start);
            }
            Some(Box::new(lo))
        };
        let inclusive = self.at(T::DotDotEq);
        self.range_op();
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
        let step = (self.eat_ident("by") || self.foreign_step()).then(|| Box::new(self.bit_or()));
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

    /// `(lo..hi) by step` is `lo..hi by step`; `(lo..hi).rev by step`
    /// counts down from the last value.
    fn by(&mut self, walk: Expr, start: u32) -> Expr {
        fn range(e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Paren(inner) => range(inner),
                ExprKind::Range { step, .. } => step.is_none(),
                _ => false,
            }
        }
        // `V by 2` and `V.rev by 2` step along a domain, named with a capital.
        fn domain(e: &Expr) -> bool {
            matches!(&e.kind, ExprKind::Name(name) if name.starts_with(|c: char| c.is_ascii_uppercase()))
        }
        let rev = matches!(&walk.kind, ExprKind::Field { base, name } if name.text == "rev" && (range(base) || domain(base)));
        if !(rev || range(&walk) || domain(&walk)) || !self.at_ident("by") {
            return walk;
        }
        self.bump();
        let step = Box::new(self.bit_or());
        let span = self.span_from(start);
        if rev || domain(&walk) {
            return Expr {
                kind: ExprKind::By {
                    walk: Box::new(walk),
                    step,
                },
                span,
            };
        }
        let mut inner = walk;
        while let ExprKind::Paren(e) = inner.kind {
            inner = *e;
        }
        if let ExprKind::Range { step: none, .. } = &mut inner.kind {
            *none = Some(step);
        }
        Expr {
            kind: inner.kind,
            span,
        }
    }

    fn bit_or(&mut self) -> Expr {
        let start = self.start();
        let mut lhs = self.bit_xor();
        while self.at(T::Pipe) && !self.doubled(T::Pipe) {
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
        while self.at(T::Amp) && !self.doubled(T::Amp) {
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
        if matches!(self.peek_n(1), T::Comma | T::RParen | T::RBracket) {
            return self.power();
        }
        if matches!(self.peek(), T::Star | T::Amp)
            && !self.after_operator()
            && matches!(self.peek_n(1), T::Ident | T::LParen)
        {
            self.c_deref();
            return self.unary();
        }
        let op = match self.peek() {
            T::Plus => UnOp::Pos,
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

    /// The two short ways to write a fn as an argument: `.cost` is
    /// `x => x.cost`, and an operator alone, `+`, is `(a, b) => a + b`.
    fn fn_shorthand(&mut self) -> Option<Expr> {
        let start = self.start();
        let span = self.span();
        let param = |text: &str| Pattern {
            kind: PatternKind::Name(text.into()),
            span,
        };
        let name = |text: &str| {
            Box::new(Expr {
                kind: ExprKind::Name(text.into()),
                span,
            })
        };
        let mut param_tys = Vec::new();
        let (params, body) = if self.at(T::Dot) && self.peek_n(1) == T::Ident {
            self.bump();
            let field = self.name();
            let body = ExprKind::Field {
                base: name("it"),
                name: field,
            };
            (vec![param("it")], body)
        } else if matches!(self.peek_n(1), T::Comma | T::RParen) {
            let cmp = match self.peek() {
                T::Lt => Some(CmpOp::Lt),
                T::LtEq => Some(CmpOp::Le),
                T::Gt => Some(CmpOp::Gt),
                T::GtEq => Some(CmpOp::Ge),
                T::EqEq => Some(CmpOp::Eq),
                T::NotEq => Some(CmpOp::Ne),
                _ => None,
            };
            if let Some(op) = cmp {
                self.bump();
                let body = ExprKind::Compare {
                    first: name("a"),
                    rest: vec![(op, *name("b"))],
                };
                (vec![param("a"), param("b")], body)
            } else {
                let op = match self.peek() {
                    T::Plus => BinOp::Add,
                    T::Minus => BinOp::Sub,
                    T::Star => BinOp::Mul,
                    T::Pipe => BinOp::BitOr,
                    T::Caret => BinOp::BitXor,
                    T::Amp => BinOp::BitAnd,
                    T::And => BinOp::And,
                    T::Or => BinOp::Or,
                    _ => return None,
                };
                self.bump();
                // `and` and `or` take only `Bool`s.
                if matches!(op, BinOp::And | BinOp::Or) {
                    let bool_ty = || {
                        Some(Type {
                            kind: TypeKind::Named {
                                name: Name {
                                    text: "Bool".into(),
                                    span,
                                },
                                args: Vec::new(),
                            },
                            span,
                        })
                    };
                    param_tys = vec![bool_ty(), bool_ty()];
                }
                let body = ExprKind::Binary {
                    op,
                    op_span: span,
                    lhs: name("a"),
                    rhs: name("b"),
                };
                (vec![param("a"), param("b")], body)
            }
        } else {
            return None;
        };
        let span = self.span_from(start);
        let body = Box::new(Expr { kind: body, span });
        Some(Expr {
            kind: ExprKind::Lambda {
                params,
                param_tys,
                param_mut: Vec::new(),
                ret_ty: None,
                body,
            },
            span,
        })
    }

    pub(super) fn arg(&mut self) -> Arg {
        let name = if self.at(T::Ident) && self.peek_n(1) == T::Eq {
            Some(self.keyword_arg())
        } else {
            self.at_label().then(|| {
                let name = self.name();
                self.bump();
                name
            })
        };
        if matches!(self.peek(), T::Comma | T::RParen) {
            if let Some(ref label) = name {
                return Arg {
                    name: Some(label.clone()),
                    mutable: false,
                    value: Expr {
                        kind: ExprKind::Name(label.text.clone()),
                        span: label.span,
                    },
                };
            }
        }
        let mutable = self.eat(T::Mut) || self.c_ref_arg();
        let start = self.start();
        let mut value = match self.fn_shorthand() {
            Some(value) => value,
            None => self.expr(),
        };
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

    pub(super) fn postfix(&mut self) -> Expr {
        let start = self.start();
        let expr = self.primary();
        self.postfix_from(start, expr)
    }

    /// `f(n) { x => x + 1 }`, read as `f(n, x => x + 1)`.
    fn trailing_lambda(&mut self) -> Option<Expr> {
        let params_end = (self.pos + 1..self.tokens.len())
            .take_while(|&i| matches!(self.tokens[i].kind, T::Ident | T::Comma))
            .last()?;
        if !(self.at(T::LBrace) && self.peek_n(params_end + 1 - self.pos) == T::FatArrow) {
            return None;
        }
        let start = self.start();
        self.bump();
        let mut params = vec![self.pattern()];
        while self.eat(T::Comma) {
            params.push(self.pattern());
        }
        self.bump();
        let body = Box::new(self.expr());
        self.expect(T::RBrace, "`}` to close the lambda");
        let span = self.span_from(start);
        self.planned(span, &crate::planned::TRAILING_LAMBDA);
        Some(Expr {
            kind: ExprKind::Lambda {
                params,
                param_tys: Vec::new(),
                param_mut: Vec::new(),
                ret_ty: None,
                body,
            },
            span,
        })
    }

    /// The calls, indexes and fields that follow `expr`.
    pub(super) fn postfix_from(&mut self, start: u32, mut expr: Expr) -> Expr {
        let mut indented_chain = false;
        loop {
            // An expression that ended with a block ended there: a `(` or
            // `[` on the next line starts a new statement.
            if self.prev_kind() == Some(T::Dedent) {
                break;
            }
            // Indented `.method(...)` continuation lines belong to the chain.
            if indented_chain && self.at(T::Newline) && self.peek_n(1) == T::Dot {
                self.bump();
            } else if !indented_chain
                && self.at(T::Newline)
                && self.peek_n(1) == T::Indent
                && self.peek_n(2) == T::Dot
            {
                self.bump();
                self.bump();
                indented_chain = true;
            }
            if self.c_ternary_ahead(&expr) {
                break;
            }
            let kind = match self.peek() {
                T::LBrace if self.struct_literal_ahead(&expr) => self.struct_literal(expr),
                T::LParen if self.swift_map_ahead(&expr) => self.swift_map(expr),
                T::LParen => {
                    self.bump();
                    let mut args = self.comma_list(T::RParen, "the argument list", Self::arg);
                    if let Some(lambda) = self.trailing_lambda() {
                        args.push(Arg {
                            name: None,
                            mutable: false,
                            value: lambda,
                        });
                    }
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    }
                }
                T::LBracket => {
                    self.bump();
                    let indices = self.comma_list(T::RBracket, "the index", Self::index_item);
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
                T::Bang
                    if !(self.peek_n(1) == T::In
                        && self.span().start > expr.span.end
                        && self
                            .tokens
                            .get(self.pos + 1)
                            .is_some_and(|t| t.span.start == self.tokens[self.pos].span.end)) =>
                {
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
                // `s.parse<Int>()`: a method takes type arguments as a
                // lowercase fn does, when they are all types and a call
                // follows, so `p.x < y` stays a comparison.
                T::Lt if matches!(expr.kind, ExprKind::Field { .. }) => {
                    let args = self.speculate(|p| {
                        let args = p.type_args();
                        if args.is_empty()
                            || p.peek() != T::LParen
                            || !args.iter().all(names_a_type)
                        {
                            p.speculation_failed = true;
                        }
                        args
                    });
                    match args {
                        Some(args) => ExprKind::Generic {
                            base: Box::new(expr),
                            args,
                        },
                        None => break,
                    }
                }
                _ => break,
            };
            expr = Expr {
                kind,
                span: self.span_from(start),
            };
        }
        if indented_chain {
            if self.at(T::Newline) && self.peek_n(1) == T::Dedent {
                self.bump();
            }
            self.eat(T::Dedent);
        }
        expr
    }

    /// Keywords that are also ordinary member names: `xs.min`, `V.none`.
    fn field_keyword(&self) -> bool {
        matches!(self.peek(), T::None | T::In | T::Type | T::Domain | T::Use) && !self.at_line_end()
    }

    /// At `@cpp(`: an expression even at a line's start, not a decorator.
    pub(super) fn at_cpp(&self) -> bool {
        self.at(T::At)
            && self.peek_n(2) == T::LParen
            && (self.tokens.get(self.pos + 1)).is_some_and(|t| t.span.text(self.src) == "cpp")
    }

    fn primary(&mut self) -> Expr {
        let start = self.start();
        let kind = match self.peek() {
            T::At
                if self.peek_n(1) == T::Ident
                    && self
                        .tokens
                        .get(self.pos + 1)
                        .is_some_and(|t| t.span.text(self.src) == "cpp") =>
            {
                self.bump();
                self.bump();
                self.expect(T::LParen, "`(` and the C++ snippet after `@cpp`");
                let code = if self.at(T::Str) {
                    let raw = self.text();
                    let inner = raw
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .unwrap_or(raw);
                    let s = unescape(inner);
                    self.bump();
                    s
                } else {
                    self.error(Diagnostic::error(
                        codes::EXPECTED_EXPR,
                        self.span(),
                        "expected a string literal in `@cpp(...)`",
                    ));
                    String::new()
                };
                self.expect(T::RParen, "`)`");
                ExprKind::Cpp(code)
            }
            T::Dot if self.peek_n(1) == T::Int => {
                self.bump();
                let digits = self.text().to_string();
                self.bump();
                let span = self.span_from(start);
                self.sink.push(
                    Diagnostic::error(
                        codes::BAD_NUMBER,
                        span,
                        format!("`.{digits}` isn't a number; write a leading zero: `0.{digits}`"),
                    )
                    .with_help("floats start with a digit, like `0.5`"),
                );
                ExprKind::Float(format!("0.{digits}"))
            }
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
            T::Str => self.python_format().unwrap_or_else(|| self.string()),
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
            // `by` is a keyword only after a range (`0..n by 2`); anywhere
            // a value starts it is an ordinary name.
            // `do:` runs a block for its value; `?` inside exits the block.
            T::Ident if self.text() == "do" && self.peek_n(1) == T::Colon => {
                self.bump();
                self.bump();
                ExprKind::Block(self.block("`do`"))
            }
            // `Some` and `None` are the spec spelling (decision 157); `some` and
            // `none` stay until the 0.2 ports migrate.
            T::Ident if self.rust_print_ahead() => self.rust_print(),
            T::Ident if self.text() == "None" => {
                self.bump();
                ExprKind::None
            }
            T::Ident if self.lazy_array_ahead() => {
                let span = self.span();
                self.planned(span, &crate::planned::LAZY_ARRAY);
                self.bump();
                self.bracket()
            }
            T::Ident if self.f_string_prefix() => self.primary().kind,
            T::Ident => {
                let mut name = self.name();
                if name.text == "Some" {
                    name.text = "some".into();
                }
                let type_like = name.is_type_like();
                let expr = Expr {
                    kind: ExprKind::Name(name.text),
                    span: name.span,
                };
                // `Name<` starts type arguments only for a capitalized name
                // whose arguments parse and close, so `INF < x` stays a
                // comparison.
                // A lowercase fn takes them when they are all types and a
                // call follows: `read<(Int, Int)>()`.
                self.empty_type_args();
                if self.at(T::Lt) {
                    let args = self.speculate(|p| {
                        let args = p.type_args();
                        if !type_like && (p.peek() != T::LParen || !args.iter().all(names_a_type)) {
                            p.speculation_failed = true;
                        }
                        if args.is_empty()
                            || !matches!(
                                p.peek(),
                                T::LParen
                                    | T::Dot
                                    | T::ColonColon
                                    | T::Comma
                                    | T::RParen
                                    | T::RBracket
                                    // `for mask in Subset<Row>:`
                                    | T::Colon
                                    // `[p for p in Pair<V> if p.lo.ord == 0]`
                                    | T::If
                            ) && !p.at_line_end()
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
                if let Some(operand) = self.c_cast() {
                    return operand;
                }
                self.bump();
                if matches!(self.peek(), T::Plus | T::Star | T::Caret | T::Amp | T::Pipe)
                    && self.peek_n(1) == T::Comma
                {
                    let op_text = self.text().to_string();
                    self.pos = super::after_group(self.tokens, self.pos.saturating_sub(1));
                    let span = self.span_from(start);
                    self.error(
                        Diagnostic::error(
                            codes::BARE_OP_TUPLE,
                            span,
                            format!(
                                "`({op_text}, ...)` is not an operation; write `each({op_text}, ...)` or `Semiring(...)`"
                            ),
                        )
                        .with_fix("insert `each`", Span::point(span.start), "each"),
                    );
                    ExprKind::Error
                } else if self.eat(T::DotDot) {
                    let base = self.expr();
                    if self.at_ident("without") {
                        self.bump();
                        self.expect(T::LParen, "`(` and the removed field names after `without`");
                        let removed = self.comma_list(T::RParen, "the removed fields", |p| {
                            p.expect_name("a field name")
                        });
                        self.expect(T::RParen, "`)`");
                        ExprKind::Without {
                            prefix: Vec::new(),
                            base: Box::new(base),
                            removed,
                        }
                    } else if self.eat(T::Comma) {
                        let updates =
                            self.comma_list(T::RParen, "the record update", Self::record_field);
                        ExprKind::With {
                            base: Box::new(base),
                            updates,
                        }
                    } else {
                        self.expect(T::RParen, "`)`");
                        ExprKind::Range {
                            lo: None,
                            hi: Some(Box::new(base)),
                            inclusive: false,
                            step: None,
                        }
                    }
                } else if self.at(T::Ident) && self.peek_n(1) == T::Colon {
                    let mut fields = Vec::new();
                    let mut spread: Option<(Box<Expr>, Vec<Name>)> = None;
                    while !self.at(T::RParen) && !self.at(T::Eof) {
                        if self.eat(T::DotDot) {
                            let base = Box::new(self.expr());
                            let mut removed = Vec::new();
                            if self.at_ident("without") {
                                self.bump();
                                self.expect(
                                    T::LParen,
                                    "`(` and the removed field names after `without`",
                                );
                                removed = self.comma_list(T::RParen, "the removed fields", |p| {
                                    p.expect_name("a field name")
                                });
                            }
                            spread = Some((base, removed));
                            self.eat(T::Comma);
                            break;
                        }
                        fields.push(self.record_field());
                        if !self.eat(T::Comma) {
                            break;
                        }
                    }
                    self.expect(T::RParen, "`)`");
                    if let Some((base, removed)) = spread {
                        ExprKind::Without {
                            prefix: fields,
                            base,
                            removed,
                        }
                    } else {
                        ExprKind::Record(fields)
                    }
                } else if self.eat(T::RParen) {
                    ExprKind::Tuple(Vec::new())
                } else {
                    let sequence = self.at(T::Let);
                    let first = if sequence {
                        Expr {
                            kind: ExprKind::Error,
                            span: self.span(),
                        }
                    } else {
                        self.expr()
                    };
                    if sequence || self.at(T::Semi) {
                        let mut depth = 1u32;
                        while depth > 0 && !self.at(T::Eof) {
                            match self.peek() {
                                T::LParen => depth += 1,
                                T::RParen => depth -= 1,
                                _ => {}
                            }
                            self.bump();
                        }
                        let span = self.span_from(start);
                        let mut diag = Diagnostic::error(
                            codes::REMOVED_SYNTAX,
                            span,
                            "statements in parentheses are written as a block, `{ a; b }`",
                        )
                        .with_help("a block's value is its last expression");
                        if depth == 0 {
                            diag.fixes.push(cpl_diag::Fix {
                                title: "write it as a block".into(),
                                edits: vec![
                                    (Span::new(span.start, span.start + 1), "{ ".into()),
                                    (Span::new(span.end - 1, span.end), " }".into()),
                                ],
                            });
                        }
                        self.error(diag);
                        return Expr {
                            kind: ExprKind::Error,
                            span,
                        };
                    }
                    if self.at(T::For) {
                        return self.paren_generator(start);
                    }
                    if self.at(T::Comma) {
                        self.bump();
                        if self.at(T::Ident) && self.peek_n(1) == T::Colon {
                            // `(t, need: r.len)`: a bare name among named fields is `t: t`.
                            let mut fields = vec![self.punned_field(first)];
                            fields.extend(self.comma_list(
                                T::RParen,
                                "the record",
                                Self::record_field,
                            ));
                            ExprKind::Record(fields)
                        } else {
                            let mut items = vec![first];
                            items.extend(self.comma_list(T::RParen, "the tuple", Self::expr));
                            ExprKind::Tuple(items)
                        }
                    } else {
                        self.expect(T::RParen, "`)` to close the parenthesis");
                        ExprKind::Paren(Box::new(first))
                    }
                }
            }
            T::LBracket => self.bracket(),
            T::Dot if self.peek_n(1) == T::Ident => {
                self.bump();
                self.bump();
                let span = self.span_from(start);
                self.planned(span, &crate::planned::IMPLICIT_MEMBER);
                ExprKind::Error
            }
            T::LBrace if self.brace_literal().is_some() => {
                let brace = self.brace_literal().unwrap();
                let (open, at) = (self.span(), self.pos);
                self.pos = after_group(self.tokens, self.pos);
                let span = self.span_from(start);
                let close = Span::new(span.end - 1, span.end);
                match brace {
                    Brace::Planned(planned) => self.planned(span, planned),
                    Brace::Map => self.brace_map(span, open, close),
                    Brace::Set => self.brace_set(span, open, close, self.pair_arg(at)),
                }
                ExprKind::Error
            }
            T::Let if self.let_in() => {
                while !matches!(self.peek(), T::Newline | T::Dedent | T::Eof) {
                    self.bump();
                }
                let span = self.span_from(start);
                self.planned(span, &crate::planned::LET_IN);
                ExprKind::Error
            }
            T::LBrace => {
                self.bump();
                ExprKind::Block(self.brace_block(start))
            }
            T::If => self.if_expr(),
            T::Match => self.match_expr(),
            T::Bang => {
                let span = self.span();
                self.error(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        "`!` doesn't negate in CP Lang",
                    )
                    .with_help("write `not x` to negate, or `x!` to unwrap an optional")
                    .with_fix("replace with `not `", span, "not "),
                );
                self.bump();
                return self.unary();
            }
            k if (BINARY_OPS.contains(&k) || matches!(k, T::And | T::Or))
                && matches!(self.peek_n(1), T::Comma | T::RParen | T::RBracket)
                && matches!(
                    self.prev_kind(),
                    Some(T::Colon | T::Comma | T::LParen | T::Eq)
                ) =>
            {
                // `over: M = (+)`, `(count: +, n: 2)`: an operator alone is
                // `(a, b) => a + b` wherever a value goes, as in an argument.
                if let Some(e) = self.fn_shorthand() {
                    return e;
                }
                let span = self.span();
                self.bump();
                self.planned(span, &crate::planned::OPERATOR_VALUE);
                ExprKind::Error
            }
            _ => {
                let after = self
                    .prev_kind()
                    .map_or("here".to_string(), |k| format!("after {}", k.describe()));
                if let Some(extra) = self.doubled_operator() {
                    let op = self.peek().describe();
                    let diag = Diagnostic::error(
                        codes::EXPECTED_EXPR,
                        self.span(),
                        format!("expected a value {after}, found {op}"),
                    )
                    .with_fix(format!("remove the extra {op}"), extra, "");
                    self.error(diag);
                } else {
                    self.error_expected(codes::EXPECTED_EXPR, &format!("a value {after}"), None);
                }
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
            let mut domains = self.comma_list(T::RBracket, "the table's domains", Self::index_ty);
            if domains.len() == 1 {
                if let TypeKind::Tuple(items) = &domains[0].kind {
                    if !items.is_empty() {
                        domains = items.clone();
                    }
                }
            }
            let params = if self.eat(T::LParen) {
                self.comma_list(T::RParen, "the table's parameters", Self::pattern)
            } else {
                vec![self.pattern()]
            };
            self.bump();
            let body = Box::new(self.lambda_body());
            return ExprKind::Table {
                domains,
                params,
                body,
            };
        }
        if self.eat(T::RBracket) {
            return ExprKind::List(Vec::new());
        }
        if self.at(T::Colon) && self.peek_n(1) == T::RBracket {
            self.bump();
            self.bump();
            return ExprKind::Map(Vec::new());
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
        // `[3: 4, 5: 6]`.
        if self.eat(T::Colon) {
            let mut entries = vec![(first, self.expr())];
            if self.eat(T::Comma) {
                entries.extend(self.comma_list(T::RBracket, "the map", |p| {
                    let key = p.expr();
                    p.expect(T::Colon, "`:` between a key and its value");
                    (key, p.expr())
                }));
            } else {
                self.expect(T::RBracket, "`,` or `]` in the map");
            }
            return ExprKind::Map(entries);
        }
        let mut items = vec![first];
        let had_comma = self.eat(T::Comma);
        if had_comma {
            items.extend(self.comma_list(T::RBracket, "the list", Self::expr));
        } else if self.at(T::Semi) {
            return self.rust_repeat(items);
        } else {
            self.expect(T::RBracket, "`,` or `]` in the list");
            if self.recovering {
                self.skip_to(T::RBracket);
                self.eat(T::RBracket);
            }
        }
        if !had_comma
            && items.len() == 1
            && (matches!(
                self.peek(),
                T::Int
                    | T::Float
                    | T::Str
                    | T::Char
                    | T::True
                    | T::False
                    | T::None
                    | T::Minus
                    | T::LParen
            ) || (self.at(T::Ident) && !self.at_ident("where")))
        {
            let value = self.expr();
            return ExprKind::TableFill {
                domain: Box::new(items.into_iter().next().unwrap()),
                value: Box::new(value),
            };
        }
        ExprKind::List(items)
    }

    /// `name: value`, `name:` (the variable `name`) or `name: +` (an operator fn).
    fn record_field(&mut self) -> (Name, Expr) {
        let name = self.expect_name("a field name");
        self.expect(T::Colon, "`:` after the field name");
        let value = if matches!(self.peek(), T::Comma | T::RParen) {
            Expr {
                kind: ExprKind::Name(name.text.clone()),
                span: name.span,
            }
        } else {
            self.fn_shorthand().unwrap_or_else(|| self.expr())
        };
        (name, value)
    }

    fn punned_field(&mut self, value: Expr) -> (Name, Expr) {
        let ExprKind::Name(text) = &value.kind else {
            self.error(Diagnostic::error(
                codes::EXPECTED_EXPR,
                value.span,
                "a record field needs a name: write `name: value`",
            ));
            let name = Name {
                text: "_".into(),
                span: value.span,
            };
            return (name, value);
        };
        let name = Name {
            text: text.clone(),
            span: value.span,
        };
        (name, value)
    }

    /// Parses one or more `for pat in iter [if cond]` (or comma-separated
    /// `for a in A, b in B if cond`, mirroring `for` statements).
    pub(super) fn clauses(&mut self) -> Vec<Clause> {
        let mut clauses = Vec::new();
        while self.eat(T::For) {
            loop {
                let pat = self.pattern();
                self.expect(T::In, "`in` after the comprehension variable");
                // Stop before `if`, which here is the filter.
                let iter = if self.at(T::Question) {
                    self.optional_type_iter()
                } else {
                    self.coalesce()
                };
                // Comprehension and generator filters use `if` only.
                let mut conds = Vec::new();
                while self.eat(T::If)
                    || (self.at_ident("where") && {
                        let span = self.span();
                        self.error(
                            Diagnostic::error(
                                codes::REMOVED_SYNTAX,
                                span,
                                "comprehension filters use `if`, not `where`",
                            )
                            .with_help("use `if` to filter a comprehension or generator")
                            .with_fix(
                                "replace `where` with `if`",
                                span,
                                "if",
                            ),
                        );
                        self.bump();
                        true
                    })
                {
                    conds.push(if self.at(T::Let) {
                        self.let_guard()
                    } else {
                        Cond::Expr(self.coalesce())
                    });
                }
                clauses.push(Clause { pat, iter, conds });
                if self.at(T::Comma) && self.comma_starts_clause() {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        clauses
    }

    /// True if a `,` inside a comprehension is followed by `pat in ...`.
    pub(super) fn comma_starts_clause(&self) -> bool {
        let mut i = 1;
        let mut depth = 0u32;
        loop {
            match self.peek_n(i) {
                T::LParen | T::LBracket => depth += 1,
                T::RParen | T::RBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                T::In if depth == 0 => return true,
                T::Comma | T::Colon | T::Eq | T::For | T::If | T::Newline | T::Eof
                    if depth == 0 =>
                {
                    return false;
                }
                T::Eof => return false,
                _ => {}
            }
            i += 1;
        }
    }

    /// `let pat = value`, then `and` with a test or another `let`,
    /// as in an `if let` statement. A `,` is not read as `and` here: it ends
    /// an argument or a list item.
    fn let_guard(&mut self) -> Cond {
        self.bump();
        let pat = self.pattern();
        self.expect(T::Eq, "`=` after the pattern in `if let`");
        let value = self.not();
        let and = if self.eat(T::And) || self.c_and() {
            Some(Box::new(if self.at(T::Let) {
                self.let_guard()
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
            Some(Box::new(Cond::Expr(self.coalesce())))
        } else {
            None
        };
        Cond::Let { pat, value, and }
    }

    /// `let y = x in y + 1`: an `in` after the `=` on the same line.
    fn let_in(&self) -> bool {
        let mut seen_eq = false;
        let mut depth = 0u32;
        for t in &self.tokens[self.pos..] {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth = depth.saturating_sub(1),
                T::Newline | T::Indent | T::Dedent | T::Eof => return false,
                T::Eq if depth == 0 => seen_eq = true,
                T::In if depth == 0 && seen_eq => return true,
                _ => {}
            }
        }
        false
    }

    fn if_expr(&mut self) -> ExprKind {
        let mut branches = Vec::new();
        let mut otherwise = None;
        let mut indented_tail = false;
        loop {
            // At `if` or `elif`.
            self.bump();
            let cond = self.cond();
            let body = self.colon_block("the condition");
            branches.push((cond, body));
            // `elif`/`else` may sit on the next line after a one-line branch,
            // either at the same indentation or indented under `x = if ...:`.
            if self.at(T::Newline) && matches!(self.peek_n(1), T::Elif | T::Else) {
                self.bump();
            } else if !indented_tail
                && self.at(T::Newline)
                && self.peek_n(1) == T::Indent
                && matches!(self.peek_n(2), T::Elif | T::Else)
            {
                self.bump();
                self.bump();
                indented_tail = true;
            }
            match self.peek() {
                T::Elif => continue,
                T::Else => {
                    self.bump();
                    // `else if` reads as `elif`, and says so.
                    if self.at(T::If) {
                        let span = self.span();
                        let both =
                            Span::new(self.rel(self.tokens[self.pos - 1].span).start, span.end);
                        self.error(
                            Diagnostic::error(
                                codes::REMOVED_SYNTAX,
                                span,
                                "use `elif`, not `else if`",
                            )
                            .with_help("write `elif cond:`")
                            .with_fix(
                                "replace with `elif`",
                                both,
                                "elif",
                            ),
                        );
                        continue;
                    }
                    otherwise = Some(self.colon_block("`else`"));
                }
                _ => {}
            }
            break;
        }
        if indented_tail {
            if self.at(T::Newline) && self.peek_n(1) == T::Dedent {
                self.bump();
            }
            self.eat(T::Dedent);
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
        if !self.at_line_end() {
            return self.inline_match(scrutinee);
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
            self.c_case();
            let pat = self.pattern();
            let guard = self.eat(T::If).then(|| self.expr());
            // `Pat => value` or, for an arm of statements, `Pat:` and a
            // block.
            let body = if self.eat(T::Colon) {
                self.block("`:`")
            } else if self.expect(T::FatArrow, "`=>` after the pattern") {
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

    /// `match k: 1 => a, 2 => b, _ => c` on one line. Each arm is a value;
    /// a `,` continues the match when a `=>` follows before the next `,`, so
    /// an inline match ends an argument list unless parenthesized.
    fn inline_match(&mut self, scrutinee: Box<Expr>) -> ExprKind {
        let mut arms = Vec::new();
        loop {
            let pat = self.pattern();
            let guard = self.eat(T::If).then(|| self.expr());
            self.expect(T::FatArrow, "`=>` after the pattern");
            let value = self.expr();
            let span = value.span;
            arms.push(Arm {
                pat,
                guard,
                body: Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(value),
                        span,
                        decorators: Vec::new(),
                        public: false,
                    }],
                    span,
                },
            });
            if !(self.at(T::Comma) && self.arm_follows(self.pos + 1)) {
                break;
            }
            self.bump();
        }
        ExprKind::Match { scrutinee, arms }
    }

    fn arm_follows(&self, from: usize) -> bool {
        let mut depth = 0u32;
        for t in &self.tokens[from.min(self.tokens.len())..] {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace if depth == 0 => return false,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                T::Comma if depth == 0 => return false,
                T::FatArrow if depth == 0 => return true,
                T::Newline | T::Indent | T::Dedent | T::Eof => return false,
                _ => {}
            }
        }
        false
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
                    // The value ends at a `:`; the rest is how to write it.
                    let colon = (sub.peek() == T::Colon).then(|| sub.tokens[sub.pos].span);
                    let mut spec = None;
                    if let Some(colon) = colon {
                        let written = &inner[colon.end as usize..];
                        spec = FmtSpec::parse(written);
                        if spec.is_none() {
                            let at = Span::new(colon.start, inner.len() as u32);
                            sub.sink.push(shift_signed(
                                Diagnostic::error(
                                    codes::UNEXPECTED_TOKEN,
                                    at,
                                    format!("`:{written}` is not a number format"),
                                )
                                .with_help(
                                    "a format is a width (`:5`, `:05`), digits after the point \
                                     (`:.3`, `:8.3f`), `:x` or `:b`",
                                ),
                                delta,
                            ));
                        }
                    } else if !matches!(sub.peek(), T::Newline | T::Eof) {
                        sub.error_expected(
                            codes::UNEXPECTED_TOKEN,
                            "`}` to end the interpolation",
                            None,
                        );
                    }
                    let failed = sub.speculation_failed;
                    // A format is not code: `08b` is no number, and no mistake.
                    let value_end = colon.map_or(u32::MAX, |c| c.start);
                    let mut diags: Vec<Diagnostic> = lexed
                        .diagnostics
                        .into_iter()
                        .filter(|d| d.span.start < value_end)
                        .map(|d| shift_signed(d, delta))
                        .collect();
                    diags.extend(sub.sink.finish());
                    // One string, one diagnostic.
                    if let Some(diag) = diags.into_iter().next() {
                        self.error(diag);
                    }
                    self.speculation_failed |= failed;
                    pieces.push(match spec {
                        Some(spec) => StrPiece::Spec(expr, spec),
                        None => StrPiece::Expr(expr),
                    });
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
                Some('x') => {
                    let hex: String = chars.clone().take(2).collect();
                    match u8::from_str_radix(&hex, 16) {
                        Ok(b) if hex.len() == 2 => {
                            chars.nth(1);
                            out.push(char::from(b));
                        }
                        _ => out.push('x'),
                    }
                }
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

pub(super) const BINARY_OPS: &[TokenKind] = &[
    T::Plus,
    T::Minus,
    T::Star,
    T::Slash,
    T::Percent,
    T::Shl,
    T::Shr,
    T::Amp,
    T::Pipe,
    T::Caret,
    T::Lt,
    T::Gt,
    T::LtEq,
    T::GtEq,
    T::EqEq,
    T::NotEq,
];

pub(super) fn begins_expr(kind: TokenKind) -> bool {
    matches!(
        kind,
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
