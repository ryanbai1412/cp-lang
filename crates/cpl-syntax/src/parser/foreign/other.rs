//! Habits from Java, Kotlin, JavaScript, TypeScript, Swift, Lua and Pascal:
//! `->` lambdas, `: T` return types, `for of`, parenthesized `for` headers,
//! `[K: V]()`, `then`/`do`, `step` and list patterns.

use super::*;

impl<'a> Parser<'a> {
    /// `fn f(x: Int): Int = x`, TypeScript's return type: CP Lang writes
    /// `->`. One error, then the type is read as the return type.
    pub(in crate::parser) fn colon_return_type(&mut self) -> bool {
        let type_name = self.peek_n(1) == T::Ident
            && self.tokens.get(self.pos + 1).is_some_and(|t| {
                t.span
                    .text(self.src)
                    .starts_with(|c: char| c.is_ascii_uppercase())
            });
        if !(self.at(T::Colon) && type_name && matches!(self.peek_n(2), T::Eq | T::Colon | T::Lt)) {
            return false;
        }
        let colon = self.span();
        let ty = self.tokens[self.pos + 1].span.text(self.src);
        self.recovered(
            cpl_diag::msgs::foreign::COLON_RETURN_TYPE
                .error(colon, &[])
                .with_help(habits::COLON_RETURN_TYPE.say(&[&ty]))
                .with_fix(habits::COLON_RETURN_TYPE_2.say(&[]), colon, " ->"),
        );
        self.bump();
        true
    }

    /// `[Str: Int]()`, Swift's empty dictionary.
    pub(in crate::parser) fn swift_map_ahead(&self, callee: &Expr) -> bool {
        let type_name = |e: &Expr| matches!(&e.kind, ExprKind::Name(n) if n.starts_with(|c: char| c.is_ascii_uppercase()));
        self.peek_n(1) == T::RParen
            && matches!(&callee.kind, ExprKind::Map(entries)
                if entries.len() == 1 && type_name(&entries[0].0) && type_name(&entries[0].1))
    }

    /// `[Str: Int]()`: one error, with CP Lang's `Map<Str, Int>()`.
    pub(in crate::parser) fn swift_map(&mut self, callee: Expr) -> ExprKind {
        let ExprKind::Map(entries) = &callee.kind else {
            return ExprKind::Error;
        };
        let (ExprKind::Name(k), ExprKind::Name(v)) = (&entries[0].0.kind, &entries[0].1.kind)
        else {
            return ExprKind::Error;
        };
        let new = format!("Map<{k}, {v}>()");
        self.bump();
        let span = Span::new(callee.span.start, self.span().end);
        self.bump();
        self.recovered(
            cpl_diag::msgs::foreign::EMPTY_MAP
                .error(span, &[])
                .with_help(habits::WRITE.say(&[&new]))
                .with_fix(habits::WRITE.say(&[&new]), span, new),
        );
        ExprKind::Error
    }

    /// `let [a, b] = xs`: a list has no pattern in CP Lang. One error,
    /// with the names still bound so their uses don't cascade.
    pub(in crate::parser) fn list_pattern(&mut self) -> PatternKind {
        let start = self.start();
        self.bump();
        let mut items = Vec::new();
        while !matches!(self.peek(), T::RBracket | T::Newline | T::Eof) {
            items.push(self.pattern());
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.expect_token(T::RBracket, "`]` after the list pattern");
        self.recovered(
            cpl_diag::msgs::foreign::LIST_PATTERN
                .error(self.span_from(start), &[])
                .with_help(habits::LIST_PATTERN.say(&[])),
        );
        PatternKind::Tuple(items)
    }

    /// `0..10 step 2`: the stride is `by` in CP Lang. One error with the
    /// fix, read as `by`.
    pub(in crate::parser) fn foreign_step(&mut self) -> bool {
        if !self.at_ident("step") || !matches!(self.peek_n(1), T::Int | T::Ident | T::LParen) {
            return false;
        }
        let span = self.span();
        self.recovered(
            cpl_diag::msgs::foreign::FOREIGN_STEP
                .error(span, &[])
                .with_help(habits::FOREIGN_STEP.say(&[]))
                .with_fix(habits::FOREIGN_STEP_2.say(&[]), span, "by"),
        );
        self.bump();
        true
    }

    /// `x -> x * 2`: Java's and Kotlin's lambda arrow. One error with the
    /// `=>` fix, read as `x => x * 2`.
    pub(super) fn arrow_lambda(&mut self) -> Expr {
        let start = self.start();
        let params = vec![self.pattern_atom()];
        let arrow = self.span();
        self.bump();
        self.recovered(
            cpl_diag::msgs::foreign::ARROW_LAMBDA
                .error(arrow, &[])
                .with_help(habits::ARROW_LAMBDA.say(&[]))
                .with_fix(habits::ARROW_LAMBDA_2.say(&[]), arrow, "=>"),
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

    /// `if x == 3 then`, `while n > 0 do`: Lua's and Pascal's word at the
    /// end of a header.
    /// One error with the `:` fix, read as the `:`.
    pub(in crate::parser) fn foreign_then(&mut self) -> bool {
        let Some(word) = ["then", "do"].into_iter().find(|w| self.at_ident(w)) else {
            return false;
        };
        if !matches!(self.peek_n(1), T::Newline | T::Eof) {
            return false;
        }
        let end = self.tokens[self.pos].span.end;
        let start = self
            .pos
            .checked_sub(1)
            .map_or(end - word.len() as u32, |p| self.tokens[p].span.end);
        let gap = self.rel(Span::new(start, end));
        self.recovered(
            cpl_diag::msgs::foreign::FOREIGN_THEN
                .error(self.span(), &[&word])
                .with_help(habits::FOREIGN_THEN.say(&[&word]))
                .with_fix(habits::FOREIGN_THEN_2.say(&[]), gap, ":"),
        );
        self.bump();
        true
    }

    /// `for (x in xs):`: parentheses around the header, as in C and
    /// JavaScript. One error with the fix that drops them, read without them.
    pub(in crate::parser) fn paren_for(&mut self) -> bool {
        let open = self.pos;
        let end = after_group(self.tokens, open);
        let Some(inner) = end
            .checked_sub(1)
            .and_then(|e| self.tokens.get(open + 1..e))
        else {
            return false;
        };
        let mut depth = 0i32;
        let has_in = inner.iter().any(|t| {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                _ => {}
            }
            depth == 0 && t.kind == T::In
        });
        if !self.at(T::LParen) || !has_in || self.tokens.get(end).map(|t| t.kind) != Some(T::Colon)
        {
            return false;
        }
        let (o, c) = (self.span(), self.rel(self.tokens[end - 1].span));
        let mut diag = cpl_diag::msgs::foreign::PAREN_FOR
            .error(o, &[])
            .with_help(habits::PAREN_FOR.say(&[]));
        diag.fixes.push(cpl_diag::Fix {
            title: habits::PAREN_FOR_2.say(&[]),
            edits: vec![(o, String::new()), (c, String::new())],
        });
        self.recovered(diag);
        self.bump();
        true
    }

    /// `for x of xs`: JavaScript's `of`. One error with the `in` fix.
    pub(in crate::parser) fn for_of(&mut self) -> bool {
        if !self.at_ident("of") {
            return false;
        }
        let span = self.span();
        self.recovered(
            cpl_diag::msgs::foreign::FOR_OF
                .error(span, &[])
                .with_help(habits::PAREN_FOR.say(&[]))
                .with_fix(habits::FOR_OF.say(&[]), span, "in"),
        );
        self.bump();
        true
    }

    /// `Int?`, Kotlin's and C#'s optional: CP Lang writes `?Int` (R02.4).
    pub(in crate::parser) fn postfix_optional(&mut self, inner: Type) -> Type {
        let q = self.span();
        self.bump();
        let span = Span::new(inner.span.start, q.end);
        let mut diag = msgs::syntax::OPTIONAL_MARK_FIRST
            .error(span, &[])
            .with_help(habits::POSTFIX_OPTIONAL.say(&[]));
        diag.fixes.push(cpl_diag::Fix {
            title: habits::POSTFIX_OPTIONAL_2.say(&[]),
            edits: vec![
                (Span::point(inner.span.start), "?".into()),
                (q, String::new()),
            ],
        });
        self.recovered(diag);
        Type {
            kind: TypeKind::Optional(Box::new(inner)),
            span,
        }
    }
}
