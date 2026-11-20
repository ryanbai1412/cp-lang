//! Declarations: fns, structs, enums, domains, aliases, traits, imports.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn fn_decl(&mut self) -> FnDecl {
        let start = self.start();
        self.bump();
        let name = self.expect_name("a fn name after `fn`");
        let generics = self.generic_params();
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
        let ret = self.eat(T::Arrow).then(|| self.ty());
        let sig_span = self.span_from(start);
        let body = if self.eat(T::Eq) {
            FnBody::Expr(self.expr())
        } else if self.at(T::Colon) {
            self.bump();
            FnBody::Block(self.block(&format!("the signature of `{}`", name.text)))
        } else if self.in_trait && self.at_line_end() && self.peek() != T::Indent {
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
        FnDecl {
            name,
            generics,
            params,
            ret,
            body,
            sig_span,
        }
    }

    fn params(&mut self) -> Vec<Param> {
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
        let name = if self.at(T::SelfKw) {
            self.name()
        } else {
            self.expect_name("a parameter name")
        };
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
            if matches!(self.peek(), T::Fn | T::At | T::Pub | T::Const) {
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
        let generics = self.generic_params();
        let mut fields = Vec::new();
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
            fields,
            methods,
        })
    }

    pub(super) fn enum_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("an enum name after `enum`");
        let generics = self.generic_params();
        let mut variants = Vec::new();
        let mut methods = Vec::new();
        if self.eat(T::Eq) {
            // `enum Dir = L | R | U | D`.
            loop {
                variants.push(self.variant());
                if !self.eat(T::Pipe) {
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
                let label = (p.at(T::Ident) && p.peek_n(1) == T::Colon).then(|| {
                    let label = p.name();
                    p.bump();
                    label
                });
                (label, p.ty())
            })
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
        let generics = self.generic_params();
        self.expect(T::Eq, "`=` and the type this names");
        let ty = self.ty();
        StmtKind::TypeAlias { name, generics, ty }
    }

    pub(super) fn trait_decl(&mut self) -> StmtKind {
        self.bump();
        let name = self.expect_name("a trait name after `trait`");
        let generics = self.generic_params();
        let was_in_trait = std::mem::replace(&mut self.in_trait, true);
        let body = self.colon_block(&format!("`trait {}`", name.text));
        self.in_trait = was_in_trait;
        StmtKind::Trait {
            name,
            generics,
            body,
        }
    }

    pub(super) fn impl_decl(&mut self) -> StmtKind {
        self.bump();
        let first = self.ty();
        let (trait_name, target) = if self.eat(T::For) {
            (Some(first), self.ty())
        } else {
            (None, first)
        };
        let body = self.colon_block("the `impl` header");
        StmtKind::Impl {
            trait_name,
            target,
            body,
        }
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
