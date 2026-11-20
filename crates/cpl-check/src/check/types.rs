//! Resolving written types, and building the `Def` of a declaration.

use super::*;
use crate::def::DomainInfo;
use crate::ty::{human_int, Param};
use cpl_syntax::ast::{self, ExprKind, TypeKind};

impl<'a> Checker<'a> {
    pub(crate) fn resolve_type(&mut self, ty: &ast::Type) -> Ty {
        match &ty.kind {
            TypeKind::Named { name, args } => {
                let args: Vec<Ty> = args.iter().map(|a| self.resolve_type(a)).collect();
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
            TypeKind::Record(fields) => Ty::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.text.clone(), self.resolve_type(ty)))
                    .collect(),
            ),
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

    fn named_type(&mut self, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        if name == "Self" || self.generics.iter().any(|g| g == name) {
            return Ty::Unknown;
        }
        // `V.bounds` is a domain of its own, derived from `V`.
        if let Some((head, member)) = name.split_once('.') {
            let head_span = Span::new(span.start, span.start + head.len() as u32);
            return match self.named_type(head, head_span, Vec::new()) {
                Ty::Pos(domain) => Ty::Pos(format!("{domain}.{member}")),
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            };
        }
        if let Some(local) = self.local(name) {
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
            return match &*def {
                Def::Domain(_) => Ty::Pos(name.to_string()),
                Def::Alias { ty } => ty.clone(),
                Def::Struct { .. } | Def::Enum { .. } => Ty::Named {
                    name: name.to_string(),
                    args,
                },
                Def::Trait => Ty::Unknown,
                Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. } => {
                    self.error(
                        codes::UNKNOWN_TYPE,
                        span,
                        format!("`{name}` is a value, not a type"),
                    );
                    Ty::Error
                }
            };
        }
        if let Some(ty) = prelude::ty(name) {
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
        let params = f
            .params
            .iter()
            .filter(|p| !(self_ty.is_some() && p.name.text == "self"))
            .map(|p| Param {
                name: p.name.text.clone(),
                ty: match &p.ty {
                    Some(ty) => self.resolve_type(ty),
                    None => Ty::Unknown,
                },
                mutable: p.mutable,
                has_default: p.default.is_some(),
                named_only: p.named_only,
                variadic: p.variadic,
            })
            .collect();
        let ret = match (&f.ret, &f.body) {
            (Some(ty), _) => self.resolve_type(ty),
            (None, ast::FnBody::Expr(_)) => Ty::Unknown,
            (None, _) => Ty::Unit,
        };
        self.in_fn -= 1;
        self.generics.truncate(outer);
        Ty::Fn {
            params,
            ret: Box::new(ret),
        }
    }

    fn method_sigs(&mut self, methods: &[ast::Stmt], self_ty: &Ty) -> Vec<(String, Ty)> {
        methods
            .iter()
            .filter_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) => Some((f.name.text.clone(), self.fn_sig(f, Some(self_ty)))),
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
                pattern_types(pat, &value_ty, &mut |name, ty| {
                    out.push((
                        name.to_string(),
                        Def::Value {
                            ty,
                            mutable: *mutable,
                            keyword,
                        },
                    ))
                });
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
            S::Const { name, ty, init, .. } => {
                let ty = match (ty, init) {
                    (Some(ty), _) => self.resolve_type(ty),
                    (None, Some(init)) => self.infer(init),
                    (None, None) => Ty::Error,
                };
                vec![(
                    name.text.clone(),
                    Def::Value {
                        ty,
                        mutable: false,
                        keyword: "const",
                    },
                )]
            }
            S::Fn(f) => vec![(
                f.name.text.clone(),
                Def::Fn {
                    ty: self.fn_sig(f, None),
                    generics: f.generics.iter().map(|g| g.text.clone()).collect(),
                },
            )],
            S::Struct(s) => {
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: Vec::new(),
                };
                let outer = self.generics.len();
                self.generics
                    .extend(s.generics.iter().map(|g| g.text.clone()));
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
                let methods = self.method_sigs(&s.methods, &self_ty);
                self.generics.truncate(outer);
                vec![(
                    s.name.text.clone(),
                    Def::Struct {
                        generics: s.generics.iter().map(|g| g.text.clone()).collect(),
                        fields,
                        methods,
                    },
                )]
            }
            S::Enum(e) => {
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: Vec::new(),
                };
                let outer = self.generics.len();
                self.generics
                    .extend(e.generics.iter().map(|g| g.text.clone()));
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
                let methods = self.method_sigs(&e.methods, &self_ty);
                self.generics.truncate(outer);
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
                out.insert(0, (e.name.text.clone(), Def::Enum { variants, methods }));
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
            S::Trait { name, .. } => vec![(name.text.clone(), Def::Trait)],
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
                pattern_types(item, &item_ty, f);
            }
        }
        _ => {}
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
