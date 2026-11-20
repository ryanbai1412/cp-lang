//! Types.

use super::*;

impl<'a> Parser<'a> {
    /// A type; `A -> B` is a fn type, as in `adj: V -> Iter<A>`.
    pub(super) fn ty(&mut self) -> Type {
        let start = self.start();
        let first = self.ty_atom();
        if !self.at(T::Arrow) || matches!(first.kind, TypeKind::Fn { .. }) {
            return first;
        }
        self.bump();
        let ret = Box::new(self.ty());
        Type {
            kind: TypeKind::Fn {
                params: vec![first],
                ret,
            },
            span: self.span_from(start),
        }
    }

    fn ty_atom(&mut self) -> Type {
        let start = self.start();
        let kind = match self.peek() {
            T::Question => {
                self.bump();
                TypeKind::Optional(Box::new(self.ty()))
            }
            T::LBracket => {
                self.bump();
                if self.at(T::Domain) {
                    self.bump();
                    let name = self.expect_name("a domain name after `domain`");
                    self.expect(T::RBracket, "`]` after the domain name");
                    TypeKind::OwnDomain {
                        name,
                        elem: Box::new(self.ty()),
                    }
                } else {
                    let index = self.comma_list(T::RBracket, "the index list", Self::index_ty);
                    TypeKind::Array {
                        index,
                        elem: Box::new(self.ty()),
                    }
                }
            }
            T::LParen => {
                self.bump();
                if self.at(T::Domain) || self.at(T::Ident) && self.peek_n(1) == T::Colon {
                    let fields = self.comma_list(T::RParen, "the record type", |p| {
                        // `(domain C, comp: [V]C)`: a fresh domain for the caller.
                        if p.at(T::Domain) {
                            let start = p.start();
                            p.bump();
                            let name = p.expect_name("a domain name after `domain`");
                            let span = p.span_from(start);
                            return (
                                name,
                                Type {
                                    kind: TypeKind::NewDomain,
                                    span,
                                },
                            );
                        }
                        let name = p.expect_name("a field name");
                        p.expect(T::Colon, "`:` after the field name");
                        (name, p.ty())
                    });
                    TypeKind::Record(fields)
                } else {
                    let mut items = self.comma_list(T::RParen, "the tuple type", Self::ty);
                    if self.eat(T::Arrow) {
                        TypeKind::Fn {
                            params: items,
                            ret: Box::new(self.ty()),
                        }
                    } else if items.len() == 1 {
                        return items.pop().unwrap();
                    } else {
                        TypeKind::Tuple(items)
                    }
                }
            }
            T::Fn => {
                self.bump();
                self.expect(T::LParen, "`(` after `fn` in a fn type");
                let params = self.comma_list(T::RParen, "the fn type", Self::ty);
                let ret = if self.eat(T::Arrow) {
                    self.ty()
                } else {
                    Type {
                        kind: TypeKind::Tuple(Vec::new()),
                        span: Span::point(self.prev_end()),
                    }
                };
                TypeKind::Fn {
                    params,
                    ret: Box::new(ret),
                }
            }
            T::Ident
                if self.text() == "yields"
                    && !matches!(self.peek_n(1), T::DotDot | T::DotDotEq) =>
            {
                self.bump();
                TypeKind::Yields(Box::new(self.ty()))
            }
            T::Ident
                if self.text().starts_with(|c: char| c.is_ascii_uppercase())
                    && !self.range_follows() =>
            {
                let mut name = self.name();
                // `V.bounds`: a domain derived from another.
                while self.at(T::Dot) && self.peek_n(1) == T::Ident {
                    self.bump();
                    let member = self.name();
                    name.text = format!("{}.{}", name.text, member.text);
                    name.span = name.span.to(member.span);
                }
                let args = if self.at(T::Lt) {
                    self.type_args()
                } else {
                    Vec::new()
                };
                // `Rooted<V>.Pre`: a domain owned by another type.
                while self.at(T::Dot)
                    && self.peek_n(1) == T::Ident
                    && self.tokens[self.pos + 1]
                        .span
                        .text(self.src)
                        .starts_with(|c: char| c.is_ascii_uppercase())
                {
                    self.bump();
                    let member = self.name();
                    name.text = format!("{}.{}", name.text, member.text);
                    name.span = name.span.to(member.span);
                }
                TypeKind::Named { name, args }
            }
            T::Ident | T::Int | T::Minus | T::Float => {
                // A range type: `0..=1e9`, `1..=n`.
                let lo = self.additive();
                let inclusive = self.at(T::DotDotEq);
                if !inclusive && !self.at(T::DotDot) {
                    let span = lo.span;
                    self.error(
                        Diagnostic::error(codes::EXPECTED_TYPE, span, "expected a type").with_help(
                            "types are capitalized (`Int`, `V`), or ranges such as `0..=1e9`",
                        ),
                    );
                    TypeKind::Error
                } else {
                    self.bump();
                    let hi = self.additive();
                    TypeKind::Range {
                        lo: Box::new(lo),
                        hi: Box::new(hi),
                        inclusive,
                    }
                }
            }
            _ => {
                let after = self.prev_kind().map_or("here", |k| k.describe());
                self.error_expected(codes::EXPECTED_TYPE, &format!("a type after {after}"), None);
                TypeKind::Error
            }
        };
        Type {
            kind,
            span: self.span_from(start),
        }
    }

    /// One entry of `[...]T`: a domain, or a count as in `[n]Int`, which
    /// is the range `0..n`.
    fn index_ty(&mut self) -> Type {
        // A count is an `Int` expression, `n` or `n - 1`; a range has
        // its `..`, and a domain starts with a capital.
        let entry = self.tokens[self.pos..]
            .iter()
            .take_while(|t| !matches!(t.kind, T::RBracket | T::Comma | T::Newline | T::Eof));
        let counted = matches!(self.peek(), T::Ident | T::Int)
            && !self.text().starts_with(|c: char| c.is_ascii_uppercase())
            && !entry
                .into_iter()
                .any(|t| matches!(t.kind, T::DotDot | T::DotDotEq | T::LParen | T::LBracket));
        if !counted {
            return self.ty();
        }
        let hi = self.primary_for_count();
        let span = hi.span;
        let lo = Expr {
            kind: ExprKind::Int(0),
            span: Span::point(span.start),
        };
        Type {
            kind: TypeKind::Range {
                lo: Box::new(lo),
                hi: Box::new(hi),
                inclusive: false,
            },
            span,
        }
    }

    fn primary_for_count(&mut self) -> Expr {
        self.additive()
    }

    /// True when a capitalized name starts a range bound (`N..=M`, with `N`
    /// a constant) and not a type name.
    fn range_follows(&self) -> bool {
        matches!(self.peek_n(1), T::DotDot | T::DotDotEq)
    }

    /// Parses `<T, U>`. A closing `>>` counts as two closers.
    pub(super) fn type_args(&mut self) -> Vec<Type> {
        self.bump();
        let mut args = Vec::new();
        loop {
            if self.at_type_args_end() || self.at_line_end() {
                break;
            }
            // `Mod<7>`: a value argument.
            let value = matches!(self.peek(), T::Int)
                && matches!(self.peek_n(1), T::Comma | T::Gt | T::Shr);
            if value {
                let expr = self.unary();
                args.push(Type {
                    span: expr.span,
                    kind: TypeKind::Value(Box::new(expr)),
                });
            } else {
                args.push(self.ty());
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        if self.at(T::Gt) {
            self.bump();
        } else if self.at(T::Shr) {
            if self.half_shr {
                self.bump();
            }
            self.half_shr = !self.half_shr;
        } else {
            self.error_expected(
                codes::EXPECTED_TOKEN,
                "`>` to close the type arguments",
                Some(">"),
            );
        }
        args
    }

    fn at_type_args_end(&self) -> bool {
        matches!(self.peek(), T::Gt | T::Shr)
    }

    /// Parses `<D, T: Ord + Add, const N: Int>` after a fn or type name:
    /// the parameter names and the bounds written on them.
    pub(super) fn generic_params(&mut self) -> (Vec<Name>, Vec<Bound>) {
        let mut bounds = Vec::new();
        if !self.eat(T::Lt) {
            return (Vec::new(), bounds);
        }
        let names = self.comma_list(T::Gt, "the type parameters", |p| {
            // `const N: Int` is a value parameter; its type is not a bound.
            let value = p.eat(T::Const);
            let name = p.expect_name("a type parameter name");
            if p.eat(T::Colon) {
                let traits = p.bound_traits();
                if !value {
                    bounds.push(Bound {
                        name: name.clone(),
                        traits,
                    });
                }
            }
            name
        });
        (names, bounds)
    }

    /// `Ord + Bounded`.
    fn bound_traits(&mut self) -> Vec<Type> {
        let mut traits = vec![self.ty_atom()];
        while self.eat(T::Plus) {
            traits.push(self.ty_atom());
        }
        traits
    }

    /// Parses `where T: Ord, U: Add + Mul` if it is here.
    pub(super) fn where_clause(&mut self, bounds: &mut Vec<Bound>) {
        if !self.eat(T::Where) {
            return;
        }
        loop {
            let name = self.expect_name("a type parameter name after `where`");
            self.expect(T::Colon, "`:` and the traits the type must have");
            let traits = self.bound_traits();
            bounds.push(Bound { name, traits });
            if !self.eat(T::Comma) {
                break;
            }
        }
    }
}
