//! Resolving written types, and building the `Def` of a declaration.

use super::*;
use crate::def::DomainInfo;
use crate::ty::{human_int, Param};
use cpl_syntax::ast::{self, ExprKind, TypeKind};

impl<'a> Checker<'a> {
    pub(crate) fn resolve_type(&mut self, ty: &ast::Type) -> Ty {
        match &ty.kind {
            TypeKind::Named { name, args } => {
                self.range_as_type_arg(&name.text, args);
                let args: Vec<Ty> = args.iter().map(|a| self.type_arg(a)).collect();
                self.named_type(&name.text, name.span, args)
            }
            TypeKind::Array { index, elem } => Ty::Array {
                index: index.iter().map(|i| self.resolve_type(i)).collect(),
                elem: Box::new(self.resolve_type(elem)),
            },
            TypeKind::OwnDomain { name, elem } => {
                // `[domain S]Char` declares `S` where it is written.
                if self.local(&name.text).is_none() && !self.scopes.is_empty() {
                    self.declare(
                        &name.text,
                        name.span,
                        Ty::Domain(name.text.clone()),
                        false,
                        "domain",
                    );
                }
                Ty::Array {
                    index: vec![Ty::Pos(name.text.clone())],
                    elem: Box::new(self.resolve_type(elem)),
                }
            }
            TypeKind::Optional(inner) => Ty::Optional(Box::new(self.resolve_type(inner))),
            TypeKind::Tuple(items) if items.is_empty() => Ty::Unit,
            TypeKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|i| self.resolve_type(i)).collect())
            }
            TypeKind::Record(fields) => {
                // `(domain C, comp: [V]C)`: the fresh domain is in scope
                // for the fields after it.
                self.push_scope();
                let fields = fields
                    .iter()
                    .map(|(name, ty)| {
                        let field_ty = match &ty.kind {
                            TypeKind::NewDomain => {
                                let domain = Ty::Domain(name.text.clone());
                                self.scopes.last_mut().unwrap().insert(
                                    name.text.clone(),
                                    Local {
                                        ty: domain.clone(),
                                        mutable: false,
                                        span: name.span,
                                        keyword: "domain",
                                    },
                                );
                                domain
                            }
                            _ => self.resolve_type(ty),
                        };
                        (name.text.clone(), field_ty)
                    })
                    .collect();
                self.pop_scope();
                Ty::Record(fields)
            }
            TypeKind::Range { lo, hi, inclusive } => {
                self.infer(lo);
                self.infer(hi);
                let lo = self.bounds(lo).0;
                let hi = self
                    .bounds(hi)
                    .1
                    .map(|h| if *inclusive { h } else { h - 1 });
                Ty::Range {
                    lo,
                    hi,
                    text: self.text(ty.span).to_string(),
                }
            }
            TypeKind::Fn { params, ret } => Ty::Fn {
                params: params
                    .iter()
                    .map(|p| Param {
                        name: String::new(),
                        ty: self.resolve_type(p),
                        mutable: false,
                        has_default: false,
                        named_only: false,
                        variadic: false,
                    })
                    .collect(),
                ret: Box::new(self.resolve_type(ret)),
            },
            TypeKind::Yields(elem) => Ty::Named {
                name: "Gen".into(),
                args: vec![self.resolve_type(elem)],
            },
            TypeKind::Value(_) | TypeKind::NewDomain => Ty::Unknown,
            TypeKind::Error => Ty::Error,
        }
    }

    /// `Dsu<0..n>`: a struct's type argument is a domain by name, and a
    /// range written in place has none.
    pub(crate) fn range_as_type_arg(&mut self, name: &str, args: &[ast::Type]) {
        let Some(range) = args
            .iter()
            .find(|a| matches!(a.kind, TypeKind::Range { .. }))
        else {
            return;
        };
        if !matches!(self.env.type_def(name).as_deref(), Some(Def::Struct { .. })) {
            return;
        }
        let written = self.quote(range.span).to_string();
        let message = format!("`{name}` takes a domain by name, and `{written}` is a range");
        if let Some(diag) = self.error(codes::UNKNOWN_TYPE, range.span, message) {
            diag.help = Some(format!(
                "name it first, `domain V = {written}`, and write `V` here"
            ));
        }
    }

    /// An argument of a generic type: a type, or the name of a number
    /// (`Mod<P>` with `const P = 998244353`).
    pub(crate) fn type_arg(&mut self, arg: &ast::Type) -> Ty {
        if let TypeKind::Named { name, args } = &arg.kind {
            if args.is_empty() {
                let value = match self.local(&name.text) {
                    Some(local) => Some(local.ty.clone()),
                    None => match self.env.global(&name.text).as_deref() {
                        Some(Def::Value { ty, .. }) => Some(ty.clone()),
                        _ => None,
                    },
                };
                if value.is_some_and(|ty| ty.is_int()) {
                    self.name_ty(&name.text, name.span);
                    return Ty::Unknown;
                }
            }
        }
        self.resolve_type(arg)
    }

    fn named_type(&mut self, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        if name == "Self" {
            return self.self_ty.clone().unwrap_or(Ty::Unknown);
        }
        if self.generics.iter().any(|g| g == name) {
            return Ty::Param(name.to_string());
        }
        // `V.bounds` is a domain of its own, derived from `V`, and
        // `Rooted<V>.Pre` is a domain the struct declares.
        if let Some((head, member)) = name.split_once('.') {
            let head_span = Span::new(span.start, span.start + head.len() as u32);
            // `treap.Treap<Int>` after `use lib.treap`.
            if self.local(head).is_none() && self.env.is_module(head) {
                return match self.env.global(name) {
                    Some(def) => self.def_type(&def, member, span, args),
                    None => {
                        let message = format!("module `{head}` has no public type `{member}`");
                        self.error(codes::UNKNOWN_TYPE, span, message);
                        Ty::Error
                    }
                };
            }
            // `xs.D`: the domain a value brought with it, as `compress`
            // does. Lowering knows which one it is.
            if head.starts_with(|c: char| c.is_ascii_lowercase()) {
                return Ty::Unknown;
            }
            return match self.named_type(head, head_span, Vec::new()) {
                Ty::Pos(domain) => Ty::Pos(format!("{domain}.{member}")),
                Ty::Named { name: owner, .. } => match self.owned_member(&owner, member) {
                    Some(Ty::Domain(domain)) => Ty::Pos(domain),
                    Some(_) => Ty::Unknown,
                    None => {
                        let member_span = Span::new(head_span.end + 1, span.end);
                        let message = format!("`{owner}` declares no domain `{member}`");
                        self.error(codes::UNKNOWN_TYPE, member_span, message);
                        Ty::Error
                    }
                },
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            };
        }
        // The struct or enum whose body this is, by its own name: its
        // definition is still being computed, so it is not looked up.
        if matches!(&self.self_ty, Some(Ty::Named { name: own, .. }) if own == name) {
            return Ty::Named {
                name: name.to_string(),
                args,
            };
        }
        if let Some(local) = self.local(name) {
            // `type Len = S.bounds` inside a fn.
            if local.keyword == "type" {
                return local.ty.clone();
            }
            if let Ty::Domain(domain) = &local.ty {
                return Ty::Pos(domain.clone());
            }
        }
        if let Some(def) = self.env.global(name) {
            self.refs.push(GlobalRef {
                span,
                name: name.to_string(),
                top_level: false,
            });
            self.hovers.push(HoverEntry {
                span,
                name: name.to_string(),
                ty: Ty::TypeName(name.to_string()),
                keyword: "type",
                decl: None,
            });
            return self.def_type(&def, name, span, args);
        }
        if let Some(ty) = prelude::ty(name) {
            let expected_arity = match name {
                "List" | "Set" | "HashSet" | "Range" | "Iter" | "Gen" => Some(1),
                "Map" | "HashMap" => Some(2),
                _ => None,
            };
            if let Some(want) = expected_arity {
                if args.is_empty() {
                    let plural = if want == 1 { "" } else { "s" };
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        format!("`{name}` is a built-in type that takes {want} type argument{plural}"),
                    );
                    return Ty::Error;
                }
            }
            return match ty {
                Ty::Named { name, .. } => Ty::Named { name, args },
                other => other,
            };
        }
        if self.in_fn > 0 && is_implicit_generic(name) {
            return Ty::Unknown;
        }
        if self.env.has_imports() {
            return Ty::Named {
                name: name.to_string(),
                args,
            };
        }
        self.unresolved(codes::UNKNOWN_TYPE, name, span, false);
        Ty::Error
    }

    /// The least and greatest value an integer expression can take, where
    /// that follows from literals and range types alone.
    pub(crate) fn bounds(&mut self, expr: &ast::Expr) -> (Option<i128>, Option<i128>) {
        match &expr.kind {
            ExprKind::Int(v) => (Some(*v), Some(*v)),
            ExprKind::Paren(inner) => self.bounds(inner),
            ExprKind::Name(name) => {
                let ty = match self.local(name) {
                    Some(local) => Some(local.ty.clone()),
                    None => match self.env.global(name).as_deref() {
                        Some(Def::Value { ty, .. }) => Some(ty.clone()),
                        _ => None,
                    },
                };
                match ty {
                    Some(Ty::Range { lo, hi, .. }) => (lo, hi),
                    _ => (None, None),
                }
            }
            ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
            } => {
                let (lo, hi) = self.bounds(expr);
                (hi.map(|v| -v), lo.map(|v| -v))
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let (alo, ahi) = self.bounds(lhs);
                let (blo, bhi) = self.bounds(rhs);
                let both =
                    |a: Option<i128>, b: Option<i128>, f: fn(i128, i128) -> Option<i128>| f(a?, b?);
                match op {
                    ast::BinOp::Add => (
                        both(alo, blo, i128::checked_add),
                        both(ahi, bhi, i128::checked_add),
                    ),
                    ast::BinOp::Sub => (
                        both(alo, bhi, i128::checked_sub),
                        both(ahi, blo, i128::checked_sub),
                    ),
                    ast::BinOp::Mul => match (alo, ahi, blo, bhi) {
                        (Some(a), Some(b), Some(c), Some(d)) => {
                            let products = [
                                a.checked_mul(c),
                                a.checked_mul(d),
                                b.checked_mul(c),
                                b.checked_mul(d),
                            ];
                            if products.iter().all(Option::is_some) {
                                let values = products.map(Option::unwrap);
                                (values.iter().min().copied(), values.iter().max().copied())
                            } else {
                                (None, None)
                            }
                        }
                        _ => (None, None),
                    },
                    // `/` and `%` are Euclidean (decision 9): the
                    // remainder is never negative, whatever the signs,
                    // and the quotient is the one that goes with it.
                    ast::BinOp::Div | ast::BinOp::Rem => match (blo, bhi) {
                        (Some(d), Some(same)) if d == same && d != 0 => {
                            if *op == ast::BinOp::Rem {
                                let exact = match (alo, ahi) {
                                    (Some(a), Some(same)) if a == same => Some(a.rem_euclid(d)),
                                    _ => None,
                                };
                                (exact.or(Some(0)), exact.or(Some(d.abs() - 1)))
                            } else {
                                // Dividing by a positive number keeps the
                                // order; by a negative one reverses it.
                                let (lo, hi) = if d > 0 { (alo, ahi) } else { (ahi, alo) };
                                (
                                    lo.and_then(|a| a.checked_div_euclid(d)),
                                    hi.and_then(|a| a.checked_div_euclid(d)),
                                )
                            }
                        }
                        _ => (None, None),
                    },
                    _ => (None, None),
                }
            }
            _ => (None, None),
        }
    }

    /// The signature of a fn. `self_ty` is the receiver's type in a method;
    /// the `self` parameter is not part of the signature callers see.
    pub(crate) fn fn_sig(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>) -> Ty {
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        self.in_fn += 1;
        // A parameter's type can use the ones before it:
        // `fn valid(n: Int, seq: [0..n]Int)`. So can the result's.
        self.push_scope();
        let mut params = Vec::new();
        for p in f.params.iter().filter(|p| !(self_ty.is_some() && p.name.text == "self")) {
            let ty = match &p.ty {
                Some(ty) => self.resolve_type(ty),
                None => Ty::Unknown,
            };
            let seen = Local { ty: ty.clone(), mutable: p.mutable, span: p.name.span, keyword: "param" };
            self.scopes.last_mut().unwrap().insert(p.name.text.clone(), seen);
            params.push(Param {
                name: p.name.text.clone(),
                ty,
                mutable: p.mutable,
                has_default: p.default.is_some(),
                named_only: p.named_only,
                variadic: p.variadic,
            });
        }
        let ret = match (&f.ret, &f.body) {
            (Some(ty), _) => self.resolve_type(ty),
            // A projection's caller works with the value it lends.
            (None, _) if f.lends.is_some() => self.resolve_type(&f.lends.as_ref().unwrap().ty),
            (None, ast::FnBody::Expr(_)) => Ty::Unknown,
            (None, _) => Ty::Unit,
        };
        self.scopes.pop();
        self.in_fn -= 1;
        self.generics.truncate(outer);
        Ty::Fn {
            params,
            ret: Box::new(ret),
        }
    }

    /// The type a definition gives its name when the name is used as a type.
    fn def_type(&mut self, def: &Def, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        match def {
            Def::Domain(_) => Ty::Pos(name.to_string()),
            // `let (C, comp) = scc(..)`: a domain a fn made and returned.
            Def::Value {
                ty: Ty::Domain(domain),
                ..
            } => Ty::Pos(domain.clone()),
            Def::Alias { ty } => ty.clone(),
            Def::Struct { .. } | Def::Enum { .. } => Ty::Named {
                name: name.to_string(),
                args,
            },
            Def::Trait { .. } => Ty::Unknown,
            Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. } | Def::Impl { .. } => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    span,
                    format!("`{name}` is a value, not a type"),
                );
                Ty::Error
            }
        }
    }

    /// A domain or constant that the struct `owner`, or an impl for it,
    /// declares in its body.
    pub(crate) fn owned_member(&mut self, owner: &str, member: &str) -> Option<Ty> {
        if let Some(Def::Struct { owned, .. }) = self.env.type_def(owner).as_deref() {
            if let Some((_, ty)) = owned.iter().find(|(name, _)| name == member) {
                return Some(ty.clone());
            }
        }
        self.env
            .impls(owner)
            .iter()
            .find_map(|(_, def)| match &**def {
                Def::Impl { owned, .. } => owned
                    .iter()
                    .find(|(name, _)| name == member)
                    .map(|(_, ty)| ty.clone()),
                _ => None,
            })
    }

    /// Enters the body of a struct, enum, trait or impl: its type
    /// parameters are in scope and `Self` is `self_ty`. Returns what
    /// [`Checker::leave_body`] needs to undo it.
    pub(crate) fn enter_body(&mut self, generics: &[String], self_ty: Ty) -> (usize, Option<Ty>) {
        let outer = self.generics.len();
        self.generics.extend(generics.iter().cloned());
        self.push_scope();
        (outer, self.self_ty.replace(self_ty))
    }

    pub(crate) fn leave_body(&mut self, (generics, self_ty): (usize, Option<Ty>)) {
        self.pop_scope();
        self.generics.truncate(generics);
        self.self_ty = self_ty;
    }

    /// Declares the domains and constants of a body, which the fields and
    /// methods after them may name: `domain Pre = 0..V.K`, `let K = n`.
    pub(crate) fn owned(&mut self, owner: &str, body: &[ast::Stmt]) -> Vec<(String, Ty)> {
        let mut owned = Vec::new();
        for stmt in body {
            let (name, ty, keyword) = match &stmt.kind {
                ast::StmtKind::Domain { name, .. } => {
                    (name, Ty::Domain(format!("{owner}.{}", name.text)), "domain")
                }
                ast::StmtKind::Const { name, ty, init, .. } => {
                    (name, self.declared_or_inferred(ty, init), "const")
                }
                ast::StmtKind::Let { pat, ty, init, .. } => {
                    // `let K = n`: a capitalized name parses as a pattern
                    // that could be an enum variant; here it is the name.
                    let text = match &pat.kind {
                        ast::PatternKind::Name(text) => text.as_str(),
                        _ => match domain_binding(pat) {
                            Some(text) => text,
                            None => continue,
                        },
                    };
                    let ty = self.declared_or_inferred(ty, init);
                    self.scopes.last_mut().unwrap().insert(
                        text.to_string(),
                        Local {
                            ty: ty.clone(),
                            mutable: false,
                            span: pat.span,
                            keyword: "let",
                        },
                    );
                    owned.push((text.to_string(), ty));
                    continue;
                }
                _ => continue,
            };
            self.scopes.last_mut().unwrap().insert(
                name.text.clone(),
                Local {
                    ty: ty.clone(),
                    mutable: false,
                    span: name.span,
                    keyword,
                },
            );
            owned.push((name.text.clone(), ty));
        }
        owned
    }

    fn declared_or_inferred(&mut self, ty: &Option<ast::Type>, init: &Option<ast::Expr>) -> Ty {
        // Signatures are computed apart from bodies, so what the value's
        // expression reports is reported where the body is checked.
        match (ty, init) {
            (Some(ty), _) => self.resolve_type(ty),
            (None, Some(init)) => self.quietly(|c| c.infer(init)).widen(),
            (None, None) => Ty::Error,
        }
    }

    /// The projections among the fns of a body.
    pub(crate) fn projections(&mut self, methods: &[ast::Stmt]) -> Vec<crate::def::Projection> {
        methods
            .iter()
            .filter_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) => {
                    let lends = f.lends.as_ref()?;
                    let law = match &f.body {
                        ast::FnBody::Block(block) => {
                            block.stmts.iter().find_map(|s| match &s.kind {
                                ast::StmtKind::Law { name, .. } if name.text == "disjoint" => {
                                    Some(self.text(s.span).to_string())
                                }
                                _ => None,
                            })
                        }
                        _ => None,
                    };
                    Some(crate::def::Projection {
                        name: f.name.text.clone(),
                        mutable: lends.mutable,
                        lent: self.text(lends.ty.span).to_string(),
                        law,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// The signatures of the fns in a body, which of them are `@get`, and
    /// which take `mut self`.
    pub(crate) fn method_sigs(
        &mut self,
        methods: &[ast::Stmt],
        self_ty: &Ty,
    ) -> (Vec<(String, Ty)>, Vec<String>, Vec<String>) {
        let mut getters = Vec::new();
        let mut mutators = Vec::new();
        let sigs = methods
            .iter()
            .filter_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) => {
                    if m.decorators.iter().any(|d| d.name.text == "get") {
                        getters.push(f.name.text.clone());
                    }
                    if f.params.first().is_some_and(|p| p.mutable && p.name.text == "self") {
                        mutators.push(f.name.text.clone());
                    }
                    Some((f.name.text.clone(), self.fn_sig(f, Some(self_ty))))
                }
                _ => None,
            })
            .collect();
        (sigs, getters, mutators)
    }

    /// The type an impl is for, with the type parameters its header names:
    /// `impl Add for Mod<P>` is generic over `P`.
    pub(crate) fn impl_generics(&mut self, target: &ast::Type) -> Vec<String> {
        let TypeKind::Named { args, .. } = &target.kind else {
            return Vec::new();
        };
        args.iter()
            .filter_map(|arg| match &arg.kind {
                TypeKind::Named { name, args }
                    if args.is_empty()
                        && self.local(&name.text).is_none()
                        && self.env.global(&name.text).is_none()
                        && prelude::ty(&name.text).is_none() =>
                {
                    Some(name.text.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// The definitions a top-level statement gives to the names it declares.
    pub(crate) fn defs_of(&mut self, stmt: &ast::Stmt) -> Vec<(String, Def)> {
        use ast::StmtKind as S;
        match &stmt.kind {
            S::Let {
                mutable,
                pat,
                ty,
                init,
                ..
            } => {
                let keyword = if *mutable { "var" } else { "let" };
                let value_ty = match (ty, init) {
                    (Some(ty), _) => self.resolve_type(ty),
                    (None, Some(init)) => self.infer(init),
                    (None, None) => Ty::Error,
                };
                let mut out = Vec::new();
                let mut value = |name: &str, ty: Ty| {
                    out.push((
                        name.to_string(),
                        Def::Value {
                            ty,
                            mutable: *mutable,
                            keyword,
                            generics: Vec::new(),
                            settled: None,
                        },
                    ))
                };
                match domain_binding(pat) {
                    // `let LIM = 100`: a capitalized name bound to a value.
                    Some(name) => value(name, value_ty.clone()),
                    None => pattern_types(pat, &value_ty, &mut value),
                }
                if let Some(ty) = ty {
                    own_domains(ty, &mut |name| {
                        out.push((
                            name.text.clone(),
                            Def::Domain(DomainInfo {
                                text: format!("the positions of `{}`", self.text(pat.span)),
                                max_size: None,
                                bound: None,
                            }),
                        ))
                    });
                }
                out
            }
            S::Const {
                name,
                generics,
                ty,
                init,
                ..
            } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                let ty = match (ty, init) {
                    (Some(ty), _) => self.resolve_type(ty),
                    (None, Some(init)) => self.infer(init),
                    (None, None) => Ty::Error,
                };
                self.generics.truncate(outer);
                let settled = init.as_ref().and_then(|init| self.const_bool(init));
                vec![(
                    name.text.clone(),
                    Def::Value {
                        ty,
                        mutable: false,
                        keyword: "const",
                        generics: generics.iter().map(|g| g.text.clone()).collect(),
                        settled,
                    },
                )]
            }
            S::Fn(f) => {
                let ty = self.fn_sig(f, None);
                let mut out = Vec::new();
                // `@memo(into: dp)`: the answers, indexed by the arguments,
                // `none` where the fn has not been asked yet.
                if let (Some((table, _)), Ty::Fn { params, ret }) = (memo_table(stmt), &ty) {
                    out.push((
                        table.to_string(),
                        Def::Value {
                            ty: Ty::Array {
                                index: params.iter().map(|p| p.ty.clone()).collect(),
                                elem: Box::new(Ty::Optional(Box::new(ret.widen()))),
                            },
                            mutable: true,
                            keyword: "var",
                            generics: Vec::new(),
                            settled: None,
                        },
                    ));
                }
                out.push((
                    f.name.text.clone(),
                    Def::Fn {
                        ty,
                        generics: f.generics.iter().map(|g| g.text.clone()).collect(),
                    },
                ));
                out
            }
            S::Struct(s) => {
                let generics: Vec<String> = s.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                let owned = self.owned(&s.name.text, &s.methods);
                let fields = s
                    .fields
                    .iter()
                    .map(|f| Param {
                        name: f.name.text.clone(),
                        ty: self.resolve_type(&f.ty),
                        mutable: false,
                        has_default: f.default.is_some(),
                        named_only: false,
                        variadic: false,
                    })
                    .collect();
                let (methods, getters, mutators) = self.method_sigs(&s.methods, &self_ty);
                self.leave_body(body);
                // `fn new(..)` without `self` says how the struct is built.
                let ctor = s.methods.iter().find_map(|m| match &m.kind {
                    ast::StmtKind::Fn(f)
                        if f.name.text == "new"
                            && f.params.iter().all(|p| p.name.text != "self") =>
                    {
                        match methods.iter().find(|(name, _)| name == "new") {
                            Some((_, Ty::Fn { params, .. })) => Some(params.clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                });
                vec![(
                    s.name.text.clone(),
                    Def::Struct {
                        generics,
                        fields,
                        methods,
                        ctor,
                        getters,
                        mutators,
                        owned,
                        projections: self.projections(&s.methods),
                    },
                )]
            }
            S::Enum(e) => {
                let generics: Vec<String> = e.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                let variants: Vec<(String, Vec<Ty>)> = e
                    .variants
                    .iter()
                    .map(|v| {
                        (
                            v.name.text.clone(),
                            v.fields
                                .iter()
                                .map(|(_, ty)| self.resolve_type(ty))
                                .collect(),
                        )
                    })
                    .collect();
                let (methods, _, _) = self.method_sigs(&e.methods, &self_ty);
                self.leave_body(body);
                let mut out: Vec<(String, Def)> = variants
                    .iter()
                    .map(|(name, fields)| {
                        (
                            name.clone(),
                            Def::Variant {
                                of: e.name.text.clone(),
                                fields: fields.clone(),
                            },
                        )
                    })
                    .collect();
                let tagged = stmt.decorators.iter().any(|d| d.name.text == "tags");
                let def = Def::Enum {
                    variants,
                    methods,
                    tagged,
                };
                out.insert(0, (e.name.text.clone(), def));
                out
            }
            S::Domain { name, def } => {
                vec![(name.text.clone(), Def::Domain(self.domain_info(def)))]
            }
            S::TypeAlias { name, generics, ty } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                let ty = self.resolve_type(ty);
                self.generics.truncate(outer);
                vec![(name.text.clone(), Def::Alias { ty })]
            }
            S::Trait {
                name,
                generics,
                supers,
                body,
            } => {
                let generics: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
                let scope = self.enter_body(&generics, Ty::Param("Self".into()));
                let methods = body
                    .stmts
                    .iter()
                    .filter_map(|m| match &m.kind {
                        ast::StmtKind::Fn(f) => Some(crate::def::TraitMethod {
                            name: f.name.text.clone(),
                            ty: self.fn_sig(f, Some(&Ty::Param("Self".into()))),
                            required: matches!(f.body, ast::FnBody::None),
                        }),
                        _ => None,
                    })
                    .collect();
                self.leave_body(scope);
                vec![(
                    name.text.clone(),
                    Def::Trait {
                        supers: supers.iter().filter_map(type_head).collect(),
                        methods,
                    },
                )]
            }
            S::Impl {
                trait_name,
                target,
                body,
                ..
            } => {
                let Some(target_name) = type_head(target) else {
                    return Vec::new();
                };
                let generics = self.impl_generics(target);
                let scope = self.enter_body(&generics, Ty::Unknown);
                let self_ty = self.quietly(|c| c.resolve_type(target));
                self.self_ty = Some(self_ty.clone());
                let owned = self.owned(&target_name, &body.stmts);
                let (methods, getters, mutators) = self.method_sigs(&body.stmts, &self_ty);
                self.leave_body(scope);
                let trait_head = trait_name.as_ref().and_then(type_head);
                vec![(
                    crate::def::impl_name(trait_head.as_deref(), &target_name),
                    Def::Impl {
                        methods,
                        getters,
                        mutators,
                        owned,
                        projections: self.projections(&body.stmts),
                    },
                )]
            }
            _ => Vec::new(),
        }
    }

    /// Size and provenance of `domain V = <def>`.
    pub(crate) fn domain_info(&mut self, def: &ast::Expr) -> DomainInfo {
        let text = self.text(def.span).to_string();
        let (max_size, bound) = match &def.kind {
            ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => {
                let lo_min = self.bounds(lo).0;
                let hi_max = self.bounds(hi).1;
                let size = match (lo_min, hi_max) {
                    (Some(lo), Some(hi)) => Some((hi - lo + i128::from(*inclusive)).max(0)),
                    _ => None,
                };
                let bound = match (&hi.kind, hi_max) {
                    (ExprKind::Name(name), Some(max)) => {
                        Some(format!("{name} <= {}", human_int(max)))
                    }
                    _ => None,
                };
                (size, bound)
            }
            // `domain Node = In + Out`: a disjoint union.
            ExprKind::Binary {
                op: ast::BinOp::Add,
                lhs,
                rhs,
                ..
            } => {
                let size = |c: &mut Self, e: &ast::Expr| match &e.kind {
                    ExprKind::Name(name) => match c.env.global(name).as_deref() {
                        Some(Def::Domain(info)) => info.max_size,
                        _ => None,
                    },
                    _ => None,
                };
                let (a, b) = (size(self, lhs), size(self, rhs));
                (a.zip(b).map(|(a, b)| a + b), None)
            }
            _ => (None, None),
        };
        DomainInfo {
            text,
            max_size,
            bound,
        }
    }
}

/// Calls `f` with each name a pattern binds and the type it gets from a
/// value of type `ty`.
/// The table a `@memo(into: dp)` fn keeps its answers in: its name and
/// where the name is written.
pub(crate) fn memo_table(stmt: &ast::Stmt) -> Option<(&str, Span)> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    memo.args
        .iter()
        .find_map(|arg| match (&arg.name, &arg.value.kind) {
            (Some(label), ExprKind::Name(table)) if label.text == "into" => {
                Some((table.as_str(), arg.value.span))
            }
            _ => None,
        })
}

/// The name a type starts with: `Mod` for `Mod<P>`.
pub(crate) fn type_head(ty: &ast::Type) -> Option<String> {
    match &ty.kind {
        TypeKind::Named { name, .. } if !name.text.is_empty() => Some(name.text.clone()),
        _ => None,
    }
}

pub(crate) fn pattern_types(pat: &ast::Pattern, ty: &Ty, f: &mut dyn FnMut(&str, Ty)) {
    use ast::PatternKind as P;
    match &pat.kind {
        P::Name(name) => f(name, ty.clone()),
        P::Tuple(items) => {
            for (i, item) in items.iter().enumerate() {
                let item_ty = match ty {
                    Ty::Tuple(tys) if tys.len() == items.len() => tys[i].clone(),
                    Ty::Record(fields) if fields.len() == items.len() => fields[i].1.clone(),
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                match domain_binding(item) {
                    // `let (C, comp) = scc(..)`: the domain the fn made.
                    // Out of a tuple this checker can't see into, the
                    // capitalized name is still a domain.
                    Some(name) if item_ty == Ty::Unknown => f(name, Ty::Domain(name.to_string())),
                    Some(name) => f(name, item_ty),
                    None => pattern_types(item, &item_ty, f),
                }
            }
        }
        _ => {}
    }
}

/// A capitalized name alone inside a tuple pattern binds a domain:
/// `let (C, comp) = scc(..)`.
pub(crate) fn domain_binding(pat: &ast::Pattern) -> Option<&str> {
    match &pat.kind {
        ast::PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
            Some(&path[0].text)
        }
        _ => None,
    }
}

/// Calls `f` with each `[domain S]` name in a type.
pub(crate) fn own_domains(ty: &ast::Type, f: &mut dyn FnMut(&ast::Name)) {
    match &ty.kind {
        TypeKind::OwnDomain { name, elem } => {
            f(name);
            own_domains(elem, f);
        }
        TypeKind::Array { index, elem } => {
            index.iter().for_each(|i| own_domains(i, f));
            own_domains(elem, f);
        }
        TypeKind::Optional(inner) => own_domains(inner, f),
        TypeKind::Tuple(items) => items.iter().for_each(|i| own_domains(i, f)),
        TypeKind::Record(fields) => fields.iter().for_each(|(_, t)| own_domains(t, f)),
        TypeKind::Named { args, .. } => args.iter().for_each(|a| own_domains(a, f)),
        _ => {}
    }
}
