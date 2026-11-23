//! Declarations: fns, structs, enums, domains, aliases, traits, imports.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn fn_stmt(&mut self) -> StmtKind {
        let start = self.start();
        let after_fn = self.pos + 1;
        // Unary operator fn: `fn -(a: P) -> P = ...`
        let is_unary_op = self
            .tokens
            .get(after_fn)
            .is_some_and(|t| t.kind == T::Minus)
            && self
                .tokens
                .get(after_fn + 1)
                .is_some_and(|t| t.kind == T::LParen);
        // Binary operator fn: `fn (a: P) + (b: P) -> P = ...`
        let is_binary_op = self
            .tokens
            .get(after_fn)
            .is_some_and(|t| t.kind == T::LParen)
            && self
                .tokens
                .get(super::after_group(self.tokens, after_fn))
                .is_some_and(|t| {
                    matches!(
                        t.kind,
                        T::Plus
                            | T::Minus
                            | T::Star
                            | T::Slash
                            | T::Percent
                            | T::StarStar
                            | T::Lt
                            | T::EqEq
                    )
                });
        if !is_unary_op && !is_binary_op {
            return StmtKind::Fn(self.fn_decl());
        }
        self.bump(); // `fn`
        let (op_kind, op_span, params) = if is_unary_op {
            let op_span = self.span();
            self.bump(); // `-`
            self.bump(); // `(`
            let params = self.params();
            (T::Minus, op_span, params)
        } else {
            self.bump(); // `(`
            let mut params = self.params();
            let op_kind = self.peek();
            let op_span = self.span();
            self.bump(); // operator
            if self.eat(T::LParen) {
                params.extend(self.params());
            } else {
                self.error_expected(
                    codes::EXPECTED_TOKEN,
                    "`(` for the right-hand parameter of the operator fn",
                    Some("(b: T)"),
                );
            }
            (op_kind, op_span, params)
        };
        let (trait_name, method_name) = if is_unary_op {
            ("Neg", "neg")
        } else {
            match op_kind {
                T::Plus => ("Add", "add"),
                T::Minus => ("Sub", "sub"),
                T::Star => ("Mul", "mul"),
                T::Slash => ("Div", "div"),
                T::Percent => ("Rem", "rem"),
                T::StarStar => ("Pow", "pow"),
                T::Lt => ("Ord", "lt"),
                T::EqEq => ("Eq", "eq"),
                _ => ("Add", "add"),
            }
        };
        let name = Name {
            text: method_name.to_string(),
            span: op_span,
        };
        let _copies = std::mem::take(&mut self.var_params);
        let ret = self.eat(T::Arrow).then(|| self.ty());
        let mut bounds = Vec::new();
        self.where_clause(&mut bounds);
        let sig_span = self.span_from(start);
        let body = if let Some(block) = self.eq_block_body(&name) {
            FnBody::Block(block)
        } else if self.eat(T::Eq) {
            FnBody::Expr(self.expr_body(&name))
        } else if self.at(T::Colon) {
            self.bump();
            FnBody::Block(self.block(&format!("the signature of `{}`", name.text)))
        } else if (self.in_trait || self.in_grader)
            && self.at_line_end()
            && self.peek() != T::Indent
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
                        "expected `:` or `=` to start the body of operator `{}`, found {found}",
                        name.text
                    ),
                )
                .with_help("use `:` for an indented body or `=` for a one-line expression"),
            );
            FnBody::Block(Block {
                stmts: Vec::new(),
                span: at,
            })
        };
        let full_span = self.span_from(start);
        let decl = FnDecl {
            name,
            generics: Vec::new(),
            bounds: bounds.clone(),
            params,
            ret,
            lends: None,
            uses: Vec::new(),
            body,
            sig_span,
        };
        if self.in_trait || self.in_impl {
            return StmtKind::Fn(decl);
        }
        let Some(target) = decl.params.first().and_then(|p| p.ty.clone()) else {
            self.error(Diagnostic::error(
                codes::EXPECTED_TOKEN,
                sig_span,
                "an operator fn needs a type on its first parameter".to_string(),
            ));
            return StmtKind::Fn(decl);
        };
        let trait_args = if !is_unary_op
            && matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow")
        {
            match decl.params.get(1).and_then(|p| p.ty.as_ref()) {
                Some(rhs) if rhs != &target => vec![rhs.clone()],
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let trait_ty = Type {
            kind: TypeKind::Named {
                name: Name {
                    text: trait_name.to_string(),
                    span: op_span,
                },
                args: trait_args,
            },
            span: op_span,
        };
        StmtKind::Impl {
            trait_name: Some(trait_ty),
            target,
            bounds,
            body: Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Fn(decl),
                    span: full_span,
                    decorators: Vec::new(),
                    public: false,
                }],
                span: full_span,
            },
        }
    }

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
        let unpacked = std::mem::take(&mut self.param_patterns);
        let arrow = self.eat(T::Arrow) || self.colon_return_type();
        let ret = (arrow && !self.at(T::Lends)).then(|| self.ty());
        // `fn at(self, i: V) -> lends mut Int` or `fn pick(a, b) -> lends(a) List<Int>`.
        let lends = self.at(T::Lends).then(|| {
            let keyword = self.start();
            self.bump();
            let bare = (!arrow).then(|| self.span_from(keyword));
            let sources = if self.eat(T::LParen) {
                self.comma_list(T::RParen, "the lent source names", |p| {
                    p.expect_name("a parameter name")
                })
            } else {
                Vec::new()
            };
            let mutable = self.eat(T::Mut);
            Lends {
                sources,
                mutable,
                ty: self.ty(),
                bare,
            }
        });
        let mut uses = Vec::new();
        if self.at_ident("uses") {
            self.bump();
            uses.push(self.expect_name("a context name after `uses`"));
            while self.eat(T::Comma) {
                uses.push(self.expect_name("a context name"));
            }
        }
        self.where_clause(&mut bounds);
        let sig_span = self.span_from(start);
        let body = if let Some(block) = self.eq_block_body(&name) {
            FnBody::Block(block)
        } else if self.eat(T::Eq) {
            FnBody::Expr(self.expr_body(&name))
        } else if self.at(T::Colon) {
            self.bump();
            FnBody::Block(self.block(&format!("the signature of `{}`", name.text)))
        } else if (self.in_trait || self.in_grader)
            && self.at_line_end()
            && self.peek() != T::Indent
            // An indented block below is a body that lost its `:`.
            && !(self.at(T::Newline) && self.peek_n(1) == T::Indent)
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
            let explicit = block
                .stmts
                .iter()
                .any(|s| matches!(s.kind, StmtKind::Lend { .. }));
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
        // assign. It is the body opening with `var b = b`. A pattern in
        // place of a name is the body opening with `let (r, s) = param`.
        if !copies.is_empty() || !unpacked.is_empty() {
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
            let unpack = |(pat, name): (Pattern, Name)| Stmt {
                span: name.span,
                decorators: Vec::new(),
                public: false,
                kind: StmtKind::Let {
                    mutable: false,
                    pat,
                    ty: None,
                    init: Some(Expr {
                        kind: ExprKind::Name(name.text),
                        span: name.span,
                    }),
                    otherwise: None,
                    guard: None,
                },
            };
            let lets: Vec<Stmt> = unpacked.into_iter().map(unpack).collect();
            block
                .stmts
                .splice(0..0, copies.into_iter().map(copy).chain(lets));
            body = FnBody::Block(block);
        }
        FnDecl {
            name,
            generics,
            bounds,
            params,
            ret,
            lends,
            uses,
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
        let lazy = self.at(T::Ident) && self.text() == "lazy" && self.peek_n(1) == T::Ident;
        if lazy {
            self.bump();
        }
        let mut mutable = self.eat(T::Mut);
        let copied = !mutable && self.eat(T::Var);
        let label = (!self.at(T::SelfKw)
            && (self.at(T::Ident) || TokenKind::keyword(self.text()).is_some())
            && self.peek_n(1) == T::Ident)
            .then(|| self.name());
        let name = if self.at(T::SelfKw) {
            self.name()
        } else if self.at(T::LParen) && !variadic && !lazy && !mutable && !copied {
            let pat = self.pattern();
            let name = Name {
                text: format!("__param{}", self.param_patterns.len()),
                span: pat.span,
            };
            self.param_patterns.push((pat, name.clone()));
            name
        } else {
            self.expect_name("a parameter name")
        };
        if copied {
            self.var_params.push(name.clone());
        }
        let ty = self.eat(T::Colon).then(|| {
            if self.at(T::Amp) && self.peek_n(1) == T::Mut {
                self.bump();
                self.bump();
                mutable = true;
            }
            self.ty()
        });
        let default = self.eat(T::Eq).then(|| self.expr());
        Param {
            mutable,
            variadic,
            lazy,
            named_only,
            label,
            name,
            ty,
            default,
            span: self.span_from(start),
        }
    }

    /// Parses the members of a `struct`, `enum` or `trait` body with
    /// `member`, one per line, and returns the methods among them.
    pub(super) fn members(&mut self, after: &str, mut member: impl FnMut(&mut Self)) -> Vec<Stmt> {
        let mut methods = Vec::new();
        if !self.expect(T::Colon, &format!("`:` after {after}"))
            && !(self.at(T::Newline) && self.peek_n(1) == T::Indent)
        {
            self.skip_line();
            return methods;
        }
        if !self.at_line_end() {
            // Single-line body: `struct T: var x: Int` or `struct Road: w: Int, len: Int`.
            loop {
                let before = self.pos;
                member(self);
                if !self.eat(T::Comma) {
                    break;
                }
                if self.at_line_end() || self.pos == before {
                    break;
                }
            }
            return methods;
        }
        if !(self.at(T::Newline) && self.peek_n(1) == T::Indent) {
            let at = self.block_expected_at(self.prev_end());
            self.error(
                Diagnostic::error(
                    codes::EXPECTED_BLOCK,
                    at,
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
            let field_attr = self.at(T::At) && self.peek_n(1) == T::Ident && {
                let at = self.pos;
                self.pos += 1;
                let ok = matches!(self.text(), "tag" | "cache");
                self.pos = at;
                ok
            };
            let declaration = !field_attr
                && (matches!(
                    self.peek(),
                    T::Fn | T::At | T::Pub | T::Const | T::Domain | T::Type | T::Let
                ) || self.at_ident("law") && self.peek_n(1) == T::Ident);
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
        let (generics, generic_defaults, mut bounds) = self.generic_params_with_defaults();
        let mut fields = Vec::new();
        if self.at(T::LParen) && self.peek_n(1) == T::Ident && self.peek_n(2) == T::RParen {
            let start = self.start();
            self.bump();
            self.bump();
            self.bump();
            let span = self.span_from(start);
            self.not_in_language(
                span,
                &format!(
                    "`{}` can't inherit from another struct: there is no inheritance (R08.5.1)",
                    name.text
                ),
                "hold the other struct as a field, and share behaviour through a trait",
            );
        }
        // `struct Monoid<T>(id: T, op: (T, T) -> T)`: fields in the header.
        let header = self.eat(T::LParen);
        if header {
            fields = self.comma_list(T::RParen, "the struct's fields", |p| {
                let name = p.expect_name("a field name");
                p.expect(T::Colon, "`:` and the field's type");
                let ty = p.ty();
                let default = p.eat(T::Eq).then(|| p.expr());
                FieldDecl {
                    name,
                    ty,
                    default,
                    mutable: false,
                    cached: false,
                }
            });
        }
        self.where_clause(&mut bounds);
        if !header && self.at(T::Colon) && !matches!(self.peek_n(1), T::Newline | T::Eof) {
            self.bump();
            loop {
                let mut cached = false;
                if self.at(T::At) && self.peek_n(1) == T::Ident {
                    self.bump();
                    let attr = self.expect_name("a field attribute");
                    if attr.text == "cache" {
                        cached = true;
                    }
                }
                let mutable = self.eat(T::Var);
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
                        mutable,
                        cached,
                    });
                }
                if !self.eat(T::Comma) || self.at_line_end() {
                    break;
                }
            }
            return StmtKind::Struct(StructDecl {
                name,
                generics,
                generic_defaults,
                bounds,
                fields,
                methods: Vec::new(),
            });
        }
        if self.at_line_end() && self.peek_n(1) != T::Indent {
            return StmtKind::Struct(StructDecl {
                name,
                generics,
                generic_defaults,
                bounds,
                fields,
                methods: Vec::new(),
            });
        }
        let methods = self.members(&format!("`struct {}`", name.text), |p| {
            let mut cached = false;
            if p.at(T::At) && p.peek_n(1) == T::Ident {
                p.bump();
                let attr = p.expect_name("a field attribute");
                if attr.text == "cache" {
                    cached = true;
                }
            }
            let mutable = p.eat(T::Var);
            // `u, v: V` declares two fields of one type.
            let mut names = vec![p.expect_name("a field name")];
            while p.eat(T::Comma) {
                names.push(p.expect_name("a field name"));
            }
            let ty = if p.at(T::Eq) {
                Type {
                    kind: TypeKind::Infer,
                    span: names[0].span,
                }
            } else {
                p.expect(T::Colon, "`:` and the field's type");
                p.ty()
            };
            let default = p.eat(T::Eq).then(|| p.expr());
            for name in names {
                fields.push(FieldDecl {
                    name,
                    ty: ty.clone(),
                    default: default.clone(),
                    mutable,
                    cached,
                });
            }
        });
        StmtKind::Struct(StructDecl {
            name,
            generics,
            generic_defaults,
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
        // `@tag("Q") Ask(h: Int)`: the token the variant reads and prints as.
        let tag = if self.at(T::At) && self.peek_n(1) == T::Ident {
            self.bump();
            self.bump();
            self.expect(T::LParen, "`(` after `@tag`");
            let tag = self.expr();
            self.expect(T::RParen, "`)` after the tag");
            Some(tag)
        } else {
            None
        };
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
        let value = if fields.is_empty() && self.eat(T::Eq) {
            Some(self.expr())
        } else {
            None
        };
        Variant {
            name,
            fields,
            value,
            tag,
        }
    }

    pub(super) fn domain_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a domain name after `domain`");
        // `domain Row` in a grader module: sized when a grader fn first binds a `[Row]T` argument.
        if self.at_line_end() {
            let def = Expr {
                kind: ExprKind::Name("todo".into()),
                span: Span::point(self.prev_end()),
            };
            return StmtKind::Domain { name, def };
        }
        self.expect(
            T::Eq,
            "`=` and the domain's range, as in `domain V = 1..=n`",
        );
        let def = counted_domain(self.expr());
        if self.at(T::If) {
            let start = self.start();
            self.bump();
            self.expr();
            let span = self.span_from(start);
            self.planned(span, &crate::planned::FILTERED_DOMAIN);
        }
        let list_start = self.start();
        while self.at(T::Comma) && self.peek_n(1) == T::Ident && self.peek_n(2) == T::Eq {
            self.bump();
            let start = self.start();
            let name = self.expect_name("a domain name");
            self.bump();
            let def = counted_domain(self.expr());
            let span = self.span_from(start);
            self.extra_stmts.push(Stmt {
                kind: StmtKind::Domain { name, def },
                span,
                decorators: Vec::new(),
                public: false,
            });
        }
        if !self.extra_stmts.is_empty() {
            let span = self.span_from(list_start);
            self.planned(span, &crate::planned::DOMAIN_LIST);
        }
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
        // `trait Ord(Eq)` or `trait Seq: Index + Ord`.
        let mut supers = Vec::new();
        if self.eat(T::Eq) {
            supers.push(self.ty());
            while self.eat(T::Plus) {
                supers.push(self.ty());
            }
            let body = Block {
                stmts: Vec::new(),
                span: Span::point(self.prev_end()),
            };
            return StmtKind::Trait {
                name,
                generics,
                supers,
                body,
                alias: true,
            };
        }
        if self.eat(T::LParen) {
            supers = self.comma_list(T::RParen, "the supertraits", Self::ty);
        } else if self.at(T::Colon) && self.peek_n(1) == T::Ident && self.peek_n(2) != T::Colon {
            self.bump();
            supers.push(self.ty());
            while self.eat(T::Plus) {
                supers.push(self.ty());
            }
        }
        let was_in_trait = std::mem::replace(&mut self.in_trait, true);
        let body = if self.at(T::Colon) && self.peek_n(1) == T::Ident && self.peek_n(2) == T::Colon
        {
            let bstart = self.start();
            self.bump();
            let mut stmts = Vec::new();
            loop {
                let fstart = self.start();
                let fname = self.expect_name("a trait field name");
                self.expect(T::Colon, "`:` and the field's type");
                let fty = self.ty();
                let fspan = self.span_from(fstart);
                stmts.push(Stmt {
                    kind: StmtKind::Let {
                        mutable: false,
                        guard: None,
                        pat: Pattern {
                            kind: PatternKind::Name(fname.text),
                            span: fname.span,
                        },
                        ty: Some(fty),
                        init: None,
                        otherwise: None,
                    },
                    span: fspan,
                    decorators: Vec::new(),
                    public: false,
                });
                if !self.eat(T::Comma) || self.at_line_end() {
                    break;
                }
            }
            Block {
                stmts,
                span: self.span_from(bstart),
            }
        } else {
            self.optional_body(&format!("`trait {}`", name.text))
        };
        self.in_trait = was_in_trait;
        StmtKind::Trait {
            name,
            generics,
            supers,
            body,
            alias: false,
        }
    }

    pub(super) fn impl_decl(&mut self) -> StmtKind {
        let start = self.start();
        self.bump();
        let (_, mut bounds) = self.generic_params();
        let first = self.ty();
        let mut extra_traits = Vec::new();
        while self.eat(T::Plus) {
            extra_traits.push(self.ty());
        }
        let (trait_name, target) = if self.eat(T::For) {
            (Some(first), self.ty())
        } else {
            (None, first)
        };
        self.where_clause(&mut bounds);
        let was_in_impl = std::mem::replace(&mut self.in_impl, true);
        let body = self.optional_body("the `impl` header");
        self.in_impl = was_in_impl;
        for extra in extra_traits {
            self.extra_stmts.push(Stmt {
                kind: StmtKind::Impl {
                    trait_name: Some(extra),
                    target: target.clone(),
                    bounds: bounds.clone(),
                    body: body.clone(),
                },
                span: self.span_from(start),
                decorators: Vec::new(),
                public: false,
            });
        }
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

/// `domain E = [m]` declares the counted domain `0..m` (core 10.2).
fn counted_domain(def: Expr) -> Expr {
    match def.kind {
        ExprKind::List(mut items) if items.len() == 1 => {
            let count = items.pop().unwrap();
            Expr {
                kind: ExprKind::Range {
                    lo: Some(Box::new(Expr {
                        kind: ExprKind::Int(0),
                        span: Span::point(def.span.start),
                    })),
                    hi: Some(Box::new(count)),
                    inclusive: false,
                    step: None,
                },
                span: def.span,
            }
        }
        kind => Expr { kind, ..def },
    }
}
