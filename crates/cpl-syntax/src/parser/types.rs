//! Types.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn ty(&mut self) -> Type {
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
                    let index = self.comma_list(T::RBracket, "the index list", Self::ty);
                    TypeKind::Array {
                        index,
                        elem: Box::new(self.ty()),
                    }
                }
            }
            T::LParen => {
                self.bump();
                if self.at(T::Ident) && self.peek_n(1) == T::Colon {
                    let fields = self.comma_list(T::RParen, "the record type", |p| {
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
            T::Ident if self.text().starts_with(|c: char| c.is_ascii_uppercase()) && !self.range_follows() => {
                let name = self.name();
                let args = if self.at(T::Lt) { self.type_args() } else { Vec::new() };
                TypeKind::Named { name, args }
            }
            T::Ident | T::Int | T::Minus | T::Float => {
                // A range type: `0..=1e9`, `1..=n`.
                let lo = self.additive();
                let inclusive = self.at(T::DotDotEq);
                if !inclusive && !self.at(T::DotDot) {
                    let span = lo.span;
                    self.error(
                        Diagnostic::error(codes::EXPECTED_TYPE, span, "expected a type")
                            .with_help("types are capitalized (`Int`, `V`), or ranges such as `0..=1e9`"),
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
            args.push(self.ty());
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
            self.error_expected(codes::EXPECTED_TOKEN, "`>` to close the type arguments", Some(">"));
        }
        args
    }

    fn at_type_args_end(&self) -> bool {
        matches!(self.peek(), T::Gt | T::Shr)
    }

    /// Parses `<D, T>` after a fn or type name. Bounds (`T: Ord`) are read
    /// and dropped for now.
    pub(super) fn generic_params(&mut self) -> Vec<Name> {
        if !self.eat(T::Lt) {
            return Vec::new();
        }
        self.comma_list(T::Gt, "the type parameters", |p| {
            let name = p.expect_name("a type parameter name");
            if p.eat(T::Colon) {
                p.ty();
            }
            name
        })
    }
}
