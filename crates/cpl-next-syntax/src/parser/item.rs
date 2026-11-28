//! Items (spec §1.10 `item`). An item's leading decorators are its first
//! children; members of a struct, enum, trait or impl recover per line.

use super::*;
use crate::grammar::Contextual as W;
use crate::ops::{self, Level};

impl Parser<'_> {
    /// Whether an item starts here: a decorator, a directive or a
    /// declaration keyword, after an optional `pub`.
    pub(super) fn item_ahead(&self) -> bool {
        let k = usize::from(self.at(T::Pub));
        match self.nth(k) {
            T::At | T::Fn | T::Struct | T::Enum | T::Trait | T::Impl | T::Use | T::Const => true,
            T::Type => self.nth(k + 1) == T::Ident,
            T::Domain => self.nth(k + 1) == T::Ident,
            T::Ident => {
                let word = crate::grammar::contextual(self.nth_text(k));
                matches!(word, Some(W::Mod) if self.nth(k + 1) == T::Ident)
                    || matches!(word, Some(W::Extern) if self.nth(k + 1) == T::Fn)
                    || matches!(word, Some(W::Context) if self.nth(k + 1) == T::Ident)
            }
            _ => false,
        }
    }

    pub(super) fn item(&mut self) {
        if self.at(T::At) && self.nth(1) == T::Bang && self.glued(1) {
            // A directive heads the item written below it.
            self.directive();
            while self.eat(T::Newline) {}
            if !matches!(self.peek(), T::Eof | T::Dedent) {
                self.item();
            }
            return;
        }
        let at = self.checkpoint();
        while self.at(T::At) {
            self.decorator();
        }
        let k = usize::from(self.at(T::Pub));
        match self.nth(k) {
            T::Fn => self.fn_decl(at),
            T::Struct => self.struct_decl(at),
            T::Enum => self.enum_decl(at),
            T::Trait => self.trait_decl(at),
            T::Impl => self.impl_decl(at),
            T::Use => self.use_decl(at),
            T::Const => self.const_decl(at),
            T::Type => self.type_alias(at, NodeKind::TypeAlias),
            T::Domain => self.domain_decl(at),
            T::Ident if self.nth_word(k) == Some(W::Extern) => {
                self.start_at(at, NodeKind::ExternDecl);
                self.bump();
                self.bump();
                self.name("the extern fn's name");
                self.params();
                self.ret();
                self.finish();
            }
            T::Ident if self.nth_word(k) == Some(W::Mod) => {
                self.start_at(at, NodeKind::ModDecl);
                self.eat(T::Pub);
                self.bump();
                self.name("the module's name");
                self.mod_items();
                self.finish();
            }
            T::Ident if self.nth_word(k) == Some(W::Context) => {
                self.start_at(at, NodeKind::ContextDecl);
                self.eat(T::Pub);
                self.bump();
                self.name("the context's name");
                if self.at(T::Colon) {
                    self.members(Self::field);
                }
                self.finish();
            }
            // Decorators on a statement, such as `@law` or a decorated `let`.
            _ => return self.stmt(),
        }
        self.end_line();
    }

    /// `domain N = e`.
    pub(super) fn domain_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::DomainDecl);
        self.bump();
        self.name("the domain's name");
        if self.want(T::Eq, "`=` and the domain's size") {
            self.expr();
        }
        self.finish();
    }

    /// `pub const N: T = e`.
    pub(super) fn const_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::ConstDecl);
        self.eat(T::Pub);
        self.bump();
        self.name("the constant's name");
        if self.eat(T::Colon) {
            self.ty();
        }
        if self.want(T::Eq, "`=` and the constant's value") {
            self.expr();
        }
        self.finish();
    }

    /// A mod's body: `:` then an indented run of items, each an `Item`
    /// node as at the top level, so nested items read like top-level ones.
    fn mod_items(&mut self) {
        self.want(T::Colon, "`:` and the module's items");
        if !(self.at(T::Newline) && self.nth(1) == T::Indent) {
            return self.error_here(codes::EXPECTED_TOKEN, "an indented run of items");
        }
        self.bump();
        self.bump();
        while !matches!(self.peek(), T::Dedent | T::Eof) {
            if self.eat(T::Newline) {
                continue;
            }
            let before = self.pos;
            self.start(NodeKind::Item);
            self.item();
            if self.pos == before {
                self.start(NodeKind::Error);
                self.bump();
                self.finish();
            }
            self.finish();
        }
        self.want(T::Dedent, "the end of the module");
    }

    /// A declared name, in a `Name` node, so readers find it by kind.
    pub(super) fn name(&mut self, what: &str) {
        self.start(NodeKind::Name);
        self.want(T::Ident, what);
        self.finish();
    }

    fn nth_text(&self, n: usize) -> &str {
        self.nth_index(n)
            .map_or("", |i| self.tokens[i].span.text(self.src))
    }

    pub(super) fn nth_word(&self, n: usize) -> Option<W> {
        if self.nth(n) == T::Ident {
            crate::grammar::contextual(self.nth_text(n))
        } else {
            None
        }
    }

    fn directive(&mut self) {
        self.start(NodeKind::Directive);
        self.bump();
        self.bump();
        self.want(T::Ident, "the directive's name");
        if self.at(T::LParen) {
            self.list(NodeKind::ArgList, T::RParen, "the argument list", Self::arg);
        }
        self.finish();
        self.end_line();
    }

    /// `@path(args)`, and its newline: the decorated item follows.
    fn decorator(&mut self) {
        self.start(NodeKind::Decorator);
        self.bump();
        self.want(T::Ident, "the decorator's name");
        while self.at(T::Dot) && self.nth(1) == T::Ident {
            self.bump();
            self.bump();
        }
        if self.at(T::LParen) {
            self.list(NodeKind::ArgList, T::RParen, "the argument list", Self::arg);
        }
        self.eat(T::Newline);
        self.finish();
    }

    pub(super) fn fn_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::FnDecl);
        self.eat(T::Pub);
        self.bump();
        if self.at(T::LParen) {
            self.start(NodeKind::OpFnName);
            self.params();
            if ops::binary(self.peek()).is_some() || self.at(T::In) {
                self.bump();
            } else {
                self.recover(codes::EXPECTED_TOKEN, "the operator this fn defines");
            }
            self.params();
            self.finish();
        } else {
            self.name("the fn's name");
            if self.at(T::Lt) {
                self.generics();
            }
            self.params();
        }
        self.ret();
        if self.word() == Some(W::Uses) {
            self.start(NodeKind::UsesClause);
            self.bump();
            self.want(T::Ident, "a context's name");
            while self.eat(T::Comma) {
                self.want(T::Ident, "a context's name");
            }
            self.finish();
        }
        self.where_clause();
        if self.at(T::Colon) {
            self.block();
        } else if self.eat(T::Eq) {
            // `= ` then the value on an indented line of its own.
            if self.at(T::Newline) && self.nth(1) == T::Indent {
                self.bump();
                self.bump();
                self.expr();
                while self.eat(T::Newline) {}
                self.want(T::Dedent, "the end of the fn's value");
            } else {
                self.expr();
            }
        }
        self.finish();
    }

    /// `(a: T, b; c = 1)`: `;` starts the named-only params.
    pub(super) fn params(&mut self) {
        self.start(NodeKind::ParamList);
        if self.want(T::LParen, "`(` and the parameters") {
            while !matches!(self.peek(), T::RParen | T::Newline | T::Eof | T::Dedent) {
                self.param();
                if !(self.eat(T::Comma) || self.eat(T::Semi)) {
                    break;
                }
            }
            self.want(T::RParen, "`)` to close the parameters");
        }
        self.finish();
    }

    fn param(&mut self) {
        self.start(NodeKind::Param);
        self.eat(T::Mut);
        if self.eat(T::DotDot) {
            self.name("the variadic parameter's name");
        } else if self.at(T::SelfKw) {
            self.bump();
        } else {
            // `while ok: K`: an outer label, then the inner name.
            if self.nth(1) == T::Ident && self.peek() != T::LParen {
                self.bump();
            }
            self.pattern();
        }
        if self.eat(T::Colon) {
            self.ty();
        }
        if self.eat(T::Eq) {
            self.expr();
        }
        self.finish();
    }

    fn ret(&mut self) {
        if !self.at(T::Arrow) {
            return;
        }
        self.start(NodeKind::RetType);
        self.bump();
        if self.word() == Some(W::Yields) {
            self.bump();
        } else if self.eat(T::Lends) {
            if self.at(T::LParen) {
                self.list(NodeKind::LendSrc, T::RParen, "the lent sources", |p| {
                    p.name("a parameter's name");
                });
            }
            self.eat(T::Mut);
        }
        self.ty();
        self.finish();
    }

    /// `<T: Bound, const N: Int, U = Int>`. A `>>` that closes a bound's
    /// type arguments closes the list too.
    fn generics(&mut self) {
        self.start(NodeKind::GenericParams);
        self.bump();
        while !matches!(self.peek(), T::Gt | T::Newline | T::Eof) && !self.half_gt {
            let before = self.pos;
            self.generic_param();
            if self.pos == before || self.half_gt || !self.eat(T::Comma) {
                break;
            }
        }
        if self.half_gt {
            self.half_gt = false;
        } else {
            self.want(T::Gt, "`>` to close the generic parameters");
        }
        self.finish();
    }

    fn generic_param(&mut self) {
        self.start(NodeKind::GenericParam);
        let constant = self.eat(T::Const);
        if !constant {
            self.eat(T::Domain);
        }
        self.name("a generic parameter's name");
        if self.eat(T::Colon) {
            if constant {
                self.ty();
            } else {
                self.bound();
            }
        }
        if self.eat(T::Eq) {
            self.ty();
        }
        self.finish();
    }

    fn bound(&mut self) {
        self.start(NodeKind::Bound);
        self.ty();
        while self.eat(T::Plus) {
            self.ty();
        }
        self.finish();
    }

    /// `where T: A, U: B`, for fns, impls and laws alike.
    fn where_clause(&mut self) {
        if self.word() != Some(W::Where) {
            return;
        }
        self.start(NodeKind::WhereClause);
        self.bump();
        loop {
            self.start(NodeKind::WherePred);
            self.ty();
            if self.want(T::Colon, "`:` and a bound") {
                self.bound();
            }
            self.finish();
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.finish();
    }

    /// `:` then an indented run of members, one per line; a broken member
    /// ends at its newline.
    fn members(&mut self, member: fn(&mut Self)) {
        self.start(NodeKind::Members);
        self.want(T::Colon, "`:` and the members");
        if self.at(T::Newline) && self.nth(1) == T::Indent {
            self.bump();
            self.bump();
            while !matches!(self.peek(), T::Dedent | T::Eof) {
                if self.eat(T::Newline) {
                    continue;
                }
                let before = self.pos;
                member(self);
                self.end_line();
                if self.pos == before {
                    self.bump();
                }
            }
            self.want(T::Dedent, "the end of the members");
        } else {
            self.error_here(codes::EXPECTED_TOKEN, "an indented run of members");
        }
        self.finish();
    }

    fn field(&mut self) {
        self.start(NodeKind::FieldDecl);
        self.eat(T::Pub);
        self.eat(T::Var);
        self.name("a field's name");
        // `var top = 0`: the default gives the type.
        if self.eat(T::Colon) {
            self.ty();
        } else if !self.at(T::Eq) {
            self.want(T::Colon, "`:` and the field's type");
        }
        if self.eat(T::Eq) {
            self.expr();
        }
        self.finish();
    }

    /// A member that is a fn, a field or (in enums) a variant.
    fn fn_or(&mut self, other: fn(&mut Self)) {
        let k = usize::from(self.at(T::Pub));
        if self.at(T::At) || self.nth(k) == T::Fn {
            let at = self.checkpoint();
            while self.at(T::At) {
                self.decorator();
            }
            if self.nth(usize::from(self.at(T::Pub))) == T::Fn {
                return self.fn_decl(at);
            }
            return other(self);
        }
        other(self);
    }

    fn struct_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::StructDecl);
        self.eat(T::Pub);
        self.bump();
        self.name("the struct's name");
        if self.at(T::Lt) {
            self.generics();
        }
        if self.at(T::LParen) {
            self.list(
                NodeKind::FieldList,
                T::RParen,
                "the struct's fields",
                Self::field,
            );
        }
        if self.at(T::Colon) && self.nth(1) != T::Newline {
            self.bump();
            self.field();
            while self.eat(T::Comma) {
                self.field();
            }
        } else if self.at(T::Colon) {
            self.members(|p| p.fn_or(Self::field));
        }
        self.finish();
    }

    fn enum_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::EnumDecl);
        self.eat(T::Pub);
        self.bump();
        self.name("the enum's name");
        if self.at(T::Lt) {
            self.generics();
        }
        if self.eat(T::Eq) {
            loop {
                self.variant();
                if !self.eat(T::Pipe) {
                    break;
                }
            }
        } else if self.at(T::Colon) && self.nth(1) != T::Newline {
            // `enum Dir: N, E` is not derived (R01.10.2); keep the variants.
            self.error_here(codes::EXPECTED_TOKEN, "`=` before variants on one line");
            self.bump();
            self.variant();
            while self.eat(T::Comma) || self.eat(T::Pipe) {
                self.variant();
            }
        } else {
            self.members(|p| p.fn_or(Self::variant));
        }
        self.finish();
    }

    fn variant(&mut self) {
        self.start(NodeKind::Variant);
        while self.at(T::At) {
            self.decorator();
        }
        self.name("a variant's name");
        if self.at(T::LParen) {
            self.bump();
            while !matches!(self.peek(), T::RParen | T::Newline | T::Eof) {
                self.field();
                if !self.eat(T::Comma) {
                    break;
                }
            }
            self.want(T::RParen, "`)` to close the variant's fields");
        }
        self.finish();
    }

    fn trait_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::TraitDecl);
        self.eat(T::Pub);
        self.bump();
        self.name("the trait's name");
        if self.at(T::Lt) {
            self.generics();
        }
        // `trait Both = A + B where ...`: an alias.
        if self.eat(T::Eq) {
            self.bound();
            self.where_clause();
            return self.finish();
        }
        if self.eat(T::LParen) {
            self.bound();
            self.want(T::RParen, "`)` to close the supertraits");
        } else if self.at(T::Colon) && self.nth(1) == T::Ident && self.nth(2) != T::Colon {
            self.bump();
            self.bound();
        }
        if self.at(T::Colon) && self.nth(1) == T::Ident && self.nth(2) == T::Colon {
            self.bump();
            self.field();
            while self.eat(T::Comma) {
                self.field();
            }
        } else {
            self.members(|p| p.assoc_member(Self::field));
        }
        self.finish();
    }

    fn impl_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::ImplDecl);
        self.bump();
        if self.at(T::Lt) {
            self.generics();
        }
        self.bound();
        if self.eat(T::For) {
            self.ty();
        }
        self.where_clause();
        if self.at(T::Colon) {
            self.members(|p| {
                p.assoc_member(|p| p.recover(codes::EXPECTED_TOKEN, "a fn, a `type` or a `law`"))
            });
        }
        self.finish();
    }

    /// A trait's or impl's member: `type`, `law`, a fn, or `other`.
    fn assoc_member(&mut self, other: fn(&mut Self)) {
        if self.at(T::Type) {
            let at = self.checkpoint();
            return self.type_alias(at, NodeKind::AssocType);
        }
        if self.word() == Some(W::Law) && self.nth(1) == T::Ident {
            self.start(NodeKind::LawDecl);
            self.bump();
            self.name("the law's name");
            self.params();
            self.where_clause();
            if self.want(T::Eq, "`=` and the law's condition") {
                self.expr();
            }
            return self.finish();
        }
        self.fn_or(other);
    }

    /// `type Name<G>: Bound = T`; a trait's associated type has no `= T`.
    fn type_alias(&mut self, at: Checkpoint, node: NodeKind) {
        self.start_at(at, node);
        self.eat(T::Pub);
        self.bump();
        self.name("the type's name");
        if self.at(T::Lt) {
            self.generics();
        }
        if self.eat(T::Colon) {
            self.bound();
        }
        if self.eat(T::Eq) {
            self.ty();
        }
        self.finish();
    }

    fn use_decl(&mut self, at: Checkpoint) {
        self.start_at(at, NodeKind::UseDecl);
        self.eat(T::Pub);
        self.bump();
        self.want(T::Ident, "the module's path");
        while self.at(T::Dot) && self.nth(1) == T::Ident {
            self.bump();
            self.bump();
        }
        if self.at(T::Dot) && self.nth(1) == T::LBrace {
            self.bump();
            self.list(NodeKind::UseNames, T::RBrace, "the imported names", |p| {
                p.want(T::Ident, "a name");
            });
        } else if self.at(T::Dot) && self.nth(1) == T::Star {
            self.bump();
            self.bump();
        }
        if self.word() == Some(W::As) {
            self.bump();
            self.name("the module's local name");
        }
        self.finish();
    }

    /// `let (domain N, i) = e`.
    pub(super) fn domain_let(&mut self) {
        self.start(NodeKind::DomainDecl);
        self.bump();
        self.bump();
        self.bump();
        self.name("the domain's name");
        while self.eat(T::Comma) {
            self.pattern();
        }
        self.want(T::RParen, "`)` to close the domain binding");
        if self.want(T::Eq, "`=` and the domain's size") {
            self.expr_at(Level::Expr);
        }
        self.finish();
    }
}
