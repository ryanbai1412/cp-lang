//! Declarations: fns, structs, enums, domains, aliases, traits, imports.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn fn_decl(&mut self) -> FnDecl {
        let start = self.start();
        self.bump();
        let name = self.expect_name("a fn name after `fn`");
        let (generics, mut bounds) = self.generic_params();
        let params = if self.eat(T::LParen) {
            self.params()
        } else {
            self.error_expected(
                codes::EXPECTED_TOKEN,
                "`(` to start the parameter list",
                Some("()"),
            );
            Vec::new()
        };
        let copies = std::mem::take(&mut self.var_params);
        let ret = self.eat(T::Arrow).then(|| self.ty());
        // `fn at(self, i: V) lends mut Int`: a projection.
        let lends =
            self.at(T::Lends).then(|| {
                self.bump();
                let mutable = self.eat(T::Mut);
                Lends {
                    mutable,
                    ty: self.ty(),
                }
            });
        self.where_clause(&mut bounds);
        let sig_span = self.span_from(start);
        let body = if self.eat(T::Eq) {
            FnBody::Expr(self.expr_body(&name))
        } else if self.at(T::Colon) {
            self.bump();
            FnBody::Block(self.block(&format!("the signature of `{}`", name.text)))
        } else if (self.in_trait || self.in_grader) && self.at_line_end() && self.peek() != T::Indent
        {
            FnBody::None
        } else {
            let at = self.error_span();
            let found = self.peek().describe();
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_TOKEN,
                    at,
                    format!(
                        "expected `:` or `=` to start the body of `{}`, found {found}",
                        name.text
                    ),
                )
                .with_fix("insert `:`", Span::point(at.start), ":"),
            );
            if self.at(T::Newline) && self.peek_n(1) == T::Indent {
                FnBody::Block(self.block("the signature"))
            } else {
                self.skip_line();
                FnBody::None
            }
        };
        // A `lends` body ends with the place it hands out.
        let mut body = body;
        if let (Some(_), FnBody::Block(block)) = (&lends, &mut body) {
            let explicit = block.stmts.iter().any(|s| matches!(s.kind, StmtKind::Lend { .. }));
            if let Some(last) = block.stmts.last_mut().filter(|_| !explicit) {
                if let StmtKind::Expr(place) = &last.kind {
                    last.kind = StmtKind::Lend {
                        mutable: false,
                        place: place.clone(),
                    };
                }
            }
        }
        // `fn grow(var b: Int)`: `b` is the fn's own copy, which it may
        // assign. It is the body opening with `var b = b`.
        if !copies.is_empty() {
            let mut block = match body {
                FnBody::Block(block) => block,
                FnBody::Expr(value) => Block {
                    span: value.span,
                    stmts: vec![Stmt {
                        span: value.span,
                        kind: StmtKind::Expr(value),
                        decorators: Vec::new(),
                        public: false,
                    }],
                },
                FnBody::None => Block {
                    span: sig_span,
                    stmts: Vec::new(),
                },
            };
            let copy = |name: Name| Stmt {
                span: name.span,
                decorators: Vec::new(),
                public: false,
                kind: StmtKind::Let {
                    mutable: true,
                    pat: Pattern {
                        kind: PatternKind::Name(name.text.clone()),
                        span: name.span,
                    },
                    ty: None,
                    init: Some(Expr {
                        kind: ExprKind::Name(name.text),
                        span: name.span,
                    }),
                    otherwise: None,
                    guard: None,
                },
            };
            block.stmts.splice(0..0, copies.into_iter().map(copy));
            body = FnBody::Block(block);
        }
        FnDecl {
            name,
            generics,
            bounds,
            params,
            ret,
            lends,
            body,
            sig_span,
        }
    }

    /// The value after `=` in a fn: on the same line, or alone in an
    /// indented block on the next lines.
    ///
    /// ```text
    /// fn count_brute() -> Int =
    ///     sum(1 for l in I for r in I if l <= r)
    /// ```
    fn expr_body(&mut self, name: &Name) -> Expr {
        if !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
            return self.expr();
        }
        self.bump();
        self.bump();
        let value = self.expr();
        self.eat(T::Newline);
        if !matches!(self.peek(), T::Dedent | T::Eof) {
            let span = self.span();
            self.error(
                Diagnostic::error(
                    codes::UNEXPECTED_TOKEN,
                    span,
                    format!(
                        "`{}` is defined with `=`, so its body is one expression",
                        name.text
                    ),
                )
                .with_help("use `:` in place of `=` for a body with several statements"),
            );
            let mut extra = Vec::new();
            while !matches!(self.peek(), T::Dedent | T::Eof) {
                let before = self.pos;
                self.stmt_into(&mut extra);
                if self.pos == before {
                    self.bump();
                }
            }
        }
        self.eat(T::Dedent);
        value
    }

    pub(super) fn params(&mut self) -> Vec<Param> {
        let mut named_only = false;
        let mut out = Vec::new();
        loop {
            if self.at(T::RParen) || self.at_line_end() {
                break;
            }
            if self.eat(T::Semi) {
                named_only = true;
                continue;
            }
            let before = self.pos;
            out.push(self.param(named_only));
            if self.eat(T::Comma) || self.at(T::Semi) {
                continue;
            }
            if self.at(T::RParen) {
                break;
            }
            self.error_expected(
                codes::EXPECTED_TOKEN,
                "`,` or `)` in the parameter list",
                None,
            );
            self.skip_to(T::RParen);
            if self.pos == before && !self.at(T::RParen) && !self.at_line_end() {
                self.bump();
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.expect(T::RParen, "`)` to close the parameter list");
        out
    }

    fn param(&mut self, named_only: bool) -> Param {
        let start = self.start();
        let variadic = self.eat(T::DotDot);
        let mutable = self.eat(T::Mut);
        let copied = !mutable && self.eat(T::Var);
        let name = if self.at(T::SelfKw) {
            self.name()
        } else {
            self.expect_name("a parameter name")
        };
        if copied {
            self.var_params.push(name.clone());
        }
        let ty = self.eat(T::Colon).then(|| self.ty());
        let default = self.eat(T::Eq).then(|| self.expr());
        Param {
            mutable,
            variadic,
            named_only,
            name,
            ty,
            default,
            span: self.span_from(start),
        }
    }

    /// Parses the members of a `struct`, `enum` or `trait` body with
    /// `member`, one per line, and returns the methods among them.
    fn members(&mut self, after: &str, mut member: impl FnMut(&mut Self)) -> Vec<Stmt> {
        let mut methods = Vec::new();
        if !self.expect(T::Colon, &format!("`:` after {after}"))
            && !(self.at(T::Newline) && self.peek_n(1) == T::Indent)
        {
            self.skip_line();
            return methods;
        }
        if !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
            let at = self.prev_end();
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_BLOCK,
                    Span::point(at),
                    format!("expected an indented block after {after}"),
                )
                .with_help("indent the next line by 4 spaces"),
            );
            self.skip_line();
            return methods;
        }
        self.bump();
        self.bump();
        let was_recovering = std::mem::replace(&mut self.recovering, false);
        while !matches!(self.peek(), T::Dedent | T::Eof) {
            let before = self.pos;
            let declaration = matches!(
                self.peek(),
                T::Fn | T::At | T::Pub | T::Const | T::Domain | T::Type | T::Let
            ) || self.at_ident("law") && self.peek_n(1) == T::Ident;
            if declaration {
                self.stmt_into(&mut methods);
            } else if matches!(self.peek(), T::Indent | T::Newline) {
                let mut stray = Vec::new();
                self.stmt_into(&mut stray);
            } else {
                let start = self.start();
                member(self);
                self.end_stmt(start);
            }
            if self.pos == before {
                self.bump();
            }
        }
        self.eat(T::Dedent);
        self.recovering = was_recovering;
        methods
    }

    pub(super) fn struct_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a struct name after `struct`");
        let (generics, mut bounds) = self.generic_params();
        let mut fields = Vec::new();
        // `struct Monoid<T>(id: T, op: (T, T) -> T)`: fields in the header.
        let header = self.eat(T::LParen);
        if header {
            fields = self.comma_list(T::RParen, "the struct's fields", |p| {
                let name = p.expect_name("a field name");
                p.expect(T::Colon, "`:` and the field's type");
                let ty = p.ty();
                let default = p.eat(T::Eq).then(|| p.expr());
                FieldDecl { name, ty, default }
            });
        }
        self.where_clause(&mut bounds);
        if !header && self.at(T::Colon) && !matches!(self.peek_n(1), T::Newline | T::Eof) {
            self.bump();
            loop {
                let mut names = vec![self.expect_name("a field name")];
                while self.eat(T::Comma) {
                    names.push(self.expect_name("a field name"));
                }
                self.expect(T::Colon, "`:` and the field's type");
                let ty = self.ty();
                let default = self.eat(T::Eq).then(|| self.expr());
                for name in names {
                    fields.push(FieldDecl {
                        name,
                        ty: ty.clone(),
                        default: default.clone(),
                    });
                }
                if !self.eat(T::Comma) || self.at_line_end() {
                    break;
                }
            }
            return StmtKind::Struct(StructDecl {
                name,
                generics,
                bounds,
                fields,
                methods: Vec::new(),
            });
        }
        if header && self.at_line_end() && self.peek_n(1) != T::Indent {
            return StmtKind::Struct(StructDecl {
                name,
                generics,
                bounds,
                fields,
                methods: Vec::new(),
            });
        }
        let methods = self.members(&format!("`struct {}`", name.text), |p| {
            // `u, v: V` declares two fields of one type.
            let mut names = vec![p.expect_name("a field name")];
            while p.eat(T::Comma) {
                names.push(p.expect_name("a field name"));
            }
            p.expect(T::Colon, "`:` and the field's type");
            let ty = p.ty();
            let default = p.eat(T::Eq).then(|| p.expr());
            for name in names {
                fields.push(FieldDecl {
                    name,
                    ty: ty.clone(),
                    default: default.clone(),
                });
            }
        });
        StmtKind::Struct(StructDecl {
            name,
            generics,
            bounds,
            fields,
            methods,
        })
    }

    pub(super) fn enum_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("an enum name after `enum`");
        let (generics, _) = self.generic_params();
        let mut variants = Vec::new();
        let mut methods = Vec::new();
        let one_line = self.at(T::Colon) && self.peek_n(1) != T::Newline;
        if one_line || self.at(T::Eq) {
            // `enum Dir = L | R | U | D`; after a `:`, `,` also separates.
            self.bump();
            loop {
                variants.push(self.variant());
                if !(self.eat(T::Pipe) || (one_line && self.eat(T::Comma))) {
                    break;
                }
            }
        } else {
            methods = self.members(&format!("`enum {}`", name.text), |p| {
                variants.push(p.variant())
            });
        }
        StmtKind::Enum(EnumDecl {
            name,
            generics,
            variants,
            methods,
        })
    }

    fn variant(&mut self) -> Variant {
        let name = self.expect_name("a variant name");
        let fields = if self.eat(T::LParen) {
            self.comma_list(T::RParen, "the variant's fields", |p| {
                // `Merge(a, b: Ship)`: several labels share one type.
                let mut labels = Vec::new();
                while p.at(T::Ident) && p.peek_n(1) == T::Comma && !p.name_is_type_like() {
                    labels.push(p.name());
                    p.bump();
                }
                if p.at(T::Ident) && p.peek_n(1) == T::Colon {
                    labels.push(p.name());
                    p.bump();
                }
                let ty = p.ty();
                if labels.is_empty() {
                    return vec![(None, ty)];
                }
                labels
                    .into_iter()
                    .map(|label| (Some(label), ty.clone()))
                    .collect()
            })
            .into_iter()
            .flatten()
            .collect()
        } else {
            Vec::new()
        };
        Variant { name, fields }
    }

    pub(super) fn domain_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a domain name after `domain`");
        self.expect(
            T::Eq,
            "`=` and the domain's range, as in `domain V = 1..=n`",
        );
        let def = self.expr();
        StmtKind::Domain { name, def }
    }

    pub(super) fn type_alias(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a type name after `type`");
        let (generics, _) = self.generic_params();
        // `type Elem` in a trait: the implementor says which type.
        if self.in_trait && self.at_line_end() {
            let ty = Type {
                kind: TypeKind::Error,
                span: Span::point(self.prev_end()),
            };
            return StmtKind::TypeAlias { name, generics, ty };
        }
        self.expect(T::Eq, "`=` and the type this names");
        let ty = self.ty();
        StmtKind::TypeAlias { name, generics, ty }
    }

    pub(super) fn trait_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a trait name after `trait`");
        let (generics, _) = self.generic_params();
        // `trait Seq: Index + Ord`.
        let mut supers = Vec::new();
        if self.at(T::Colon) && self.peek_n(1) == T::Ident {
            self.bump();
            supers.push(self.ty());
            while self.eat(T::Plus) {
                supers.push(self.ty());
            }
        }
        let was_in_trait = std::mem::replace(&mut self.in_trait, true);
        let body = self.optional_body(&format!("`trait {}`", name.text));
        self.in_trait = was_in_trait;
        StmtKind::Trait {
            name,
            generics,
            supers,
            body,
        }
    }

    pub(super) fn impl_decl(&mut self) -> StmtKind {
        self.bump();
        let (_, mut bounds) = self.generic_params();
        let first = self.ty();
        let (trait_name, target) = if self.eat(T::For) {
            (Some(first), self.ty())
        } else {
            (None, first)
        };
        self.where_clause(&mut bounds);
        let body = self.optional_body("the `impl` header");
        StmtKind::Impl {
            trait_name,
            target,
            bounds,
            body,
        }
    }

    /// The body of a trait or impl, which may be left out: `impl Seq for V`
    /// takes every default method.
    fn optional_body(&mut self, after: &str) -> Block {
        if self.at_line_end() && self.peek_n(1) != T::Indent {
            return Block {
                stmts: Vec::new(),
                span: Span::point(self.prev_end()),
            };
        }
        self.colon_block(after)
    }

    fn name_is_type_like(&self) -> bool {
        self.text().starts_with(|c: char| c.is_ascii_uppercase())
    }

    pub(super) fn use_decl(&mut self) -> StmtKind {
        self.bump();
        let mut path = vec![self.expect_name("a module path after `use`, as in `use std.segtree`")];
        let mut names = Vec::new();
        let mut glob = false;
        while self.eat(T::Dot) {
            if self.eat(T::Star) {
                glob = true;
                break;
            }
            if self.eat(T::LBrace) {
                names = self.comma_list(T::RBrace, "the import list", |p| {
                    p.expect_name("a name to import")
                });
                break;
            }
            path.push(self.expect_name("a module name after `.`"));
        }
        let alias = self.at_ident("as").then(|| {
            self.bump();
            self.expect_name("a name after `as`")
        });
        StmtKind::Use {
            path,
            names,
            glob,
            alias,
        }
    }
}
