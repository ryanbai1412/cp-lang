//! Resolving written types, and building the `Def` of a declaration.

use super::pattern::{domain_binding, typed_pattern};
use super::*;
use crate::def::DomainInfo;
use crate::ty::{human_int, Param};
use cpl_syntax::ast::{self, Expr, ExprKind, TypeKind};

/// Types by the name they are filed under: a body's method signatures.
type Named = Vec<(String, Ty)>;
impl<'a> Checker<'a> {
    /// R04.2.11: for `P = (A, B)`, `[P]T` and `[A, B]T` are one type, kept
    /// as one index per factor.
    pub(crate) fn flat_index(&mut self, index: Vec<Ty>) -> Vec<Ty> {
        let mut flat = Vec::with_capacity(index.len());
        for ty in index {
            match (self.position_factors(&ty), &ty) {
                (Some(parts), _) => flat.extend(parts),
                (None, Ty::Tuple(items))
                    if items.len() > 1 && items.iter().all(|t| matches!(t, Ty::Pos(_))) =>
                {
                    flat.extend(items.iter().cloned())
                }
                (None, _) => flat.push(ty),
            }
        }
        flat
    }

    /// `grid[p]` or `d[p, q]` with `p, q: Cell = (Row, Col)`: each product
    /// position stands for its factor indices.
    /// A key the checker could not infer (`[_]K`) takes any indices; lowering checks them.
    pub(crate) fn indexes_by_factors(&mut self, got: &[Ty], index: &[Ty]) -> bool {
        if index.contains(&Ty::Unknown) {
            return true;
        }
        let flat = self.flat_index(got.to_vec());
        flat.len() == index.len()
            && flat
                .iter()
                .zip(index)
                .all(|(got, want)| self.fits(got, want))
    }

    /// The factor positions of a product-domain position, when every
    /// factor is itself a position.
    pub(crate) fn position_factors(&mut self, ty: &Ty) -> Option<Vec<Ty>> {
        let Ty::Pos(dom) = ty else {
            return None;
        };
        let parts: Vec<Ty> = self
            .product_factors(dom)?
            .into_iter()
            .map(|(_, t)| t)
            .collect();
        parts
            .iter()
            .all(|t| matches!(t, Ty::Pos(_)))
            .then_some(parts)
    }

    pub(crate) fn resolve_type(&mut self, ty: &ast::Type) -> Ty {
        match &ty.kind {
            TypeKind::Named { name, args } => {
                let args: Vec<Ty> = args.iter().map(|a| self.type_arg(a)).collect();
                self.named_type(&name.text, name.span, args)
            }
            TypeKind::Array { index, elem } => {
                let index = index.iter().map(|i| self.resolve_type(i)).collect();
                Ty::Array {
                    index: self.flat_index(index),
                    elem: Box::new(self.resolve_type(elem)),
                }
            }
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
                let has_binder = items
                    .iter()
                    .any(|i| matches!(i.kind, TypeKind::DomainBinder(_)));
                if has_binder {
                    self.push_scope();
                    let resolved = items
                        .iter()
                        .map(|i| match &i.kind {
                            TypeKind::DomainBinder(name) => {
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
                            _ => self.resolve_type(i),
                        })
                        .collect();
                    self.pop_scope();
                    Ty::Tuple(resolved)
                } else {
                    Ty::Tuple(items.iter().map(|i| self.resolve_type(i)).collect())
                }
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
                let lo = lo.as_deref().and_then(|lo| {
                    self.infer(lo);
                    self.bounds(lo).0
                });
                let hi = hi.as_deref().and_then(|hi| {
                    self.infer(hi);
                    self.bounds(hi)
                        .1
                        .map(|h| if *inclusive { h } else { h - 1 })
                });
                Ty::Range {
                    lo,
                    hi,
                    text: self.text(ty.span).to_string(),
                }
            }
            TypeKind::Fn { params, ret } => Ty::Fn {
                params: params
                    .iter()
                    .map(|p| {
                        let (mutable, inner) = match &p.kind {
                            TypeKind::Mut(inner) => (true, inner.as_ref()),
                            _ => (false, p),
                        };
                        Param {
                            name: String::new(),
                            ty: self.resolve_type(inner),
                            mutable,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }
                    })
                    .collect(),
                ret: Box::new(self.resolve_type(ret)),
            },
            TypeKind::Mut(inner) => self.resolve_type(inner),
            TypeKind::Yields(elem) => Ty::Named {
                name: "Gen".into(),
                args: vec![self.resolve_type(elem)],
            },
            TypeKind::DomainBinder(name) => Ty::Domain(name.text.clone()),
            TypeKind::ConstrainedStr { len, .. } => self.bounded_str(len.as_deref()),
            TypeKind::Row(fields) => {
                let mut fs: Vec<(String, Ty)> = fields
                    .iter()
                    .map(|(n, t)| (n.text.clone(), self.resolve_type(t)))
                    .collect();
                fs.push(("..".into(), Ty::Unknown));
                Ty::Record(fs)
            }
            TypeKind::RowWithout {
                prefix,
                base,
                removed,
            } => {
                let mut out_fields: Vec<(String, Ty)> = prefix
                    .iter()
                    .map(|(n, t)| (n.text.clone(), self.resolve_type(t)))
                    .collect();
                let base_ty = self.resolve_type(base);
                let rem_names: Vec<String> = removed.iter().map(|n| n.text.clone()).collect();
                if let Ty::Param(p) = &base_ty {
                    if let Some(req) = self.row_bounds.get(p).cloned() {
                        for r in removed {
                            if !req.iter().any(|(fnm, _)| fnm == &r.text) {
                                self.error(
                                    codes::UNKNOWN_FIELD,
                                    r.span,
                                    format!("row `{p}` has no named field `{}` to remove", r.text),
                                );
                            }
                        }
                    }
                }
                match base_ty {
                    Ty::Record(fs) => {
                        out_fields.extend(
                            fs.into_iter()
                                .filter(|(n, _)| !rem_names.iter().any(|r| r == n)),
                        );
                        Ty::Record(out_fields)
                    }
                    other => Ty::RowWithout(out_fields, Box::new(other), rem_names),
                }
            }
            TypeKind::LiteralUnion(lits) => match lits.first().map(|s| s.as_str()) {
                Some(s) if s.starts_with('\'') => Ty::Char,
                Some(s) if s.starts_with('"') => Ty::Str,
                _ => Ty::Int,
            },
            TypeKind::Value(e) => match e.kind {
                ExprKind::Int(v) => Ty::Const(v),
                _ => Ty::Unknown,
            },
            TypeKind::NewDomain | TypeKind::Infer => Ty::Unknown,
            TypeKind::Error => Ty::Error,
        }
    }

    /// A field's declared type, or its default's when written `total = 0`.
    pub(crate) fn field_type(&mut self, f: &ast::FieldDecl) -> Ty {
        match (&f.ty.kind, &f.default) {
            (TypeKind::Infer, Some(default)) => self.quietly(|c| c.infer(default)).widen(),
            _ => self.resolve_type(&f.ty),
        }
    }

    /// An argument of a generic type: a type, or the name of a number
    /// (`Mod<P>` with `const P = 998244353`).
    pub(crate) fn type_arg(&mut self, arg: &ast::Type) -> Ty {
        if let TypeKind::Named { name, args } = &arg.kind {
            if args.is_empty() {
                let value = match self.local(&name.text) {
                    Some(local) => Some(local.ty.clone()),
                    None => match self.global(&name.text).as_deref() {
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
        if let Some(member) = name.strip_prefix("Self.") {
            if let Some((_, chosen)) = self.self_assoc.iter().find(|(n, _)| n == member) {
                return chosen.clone();
            }
        }
        if self.generics.iter().any(|g| g == name) {
            return Ty::Param(name.to_string());
        }
        // `V.cuts` is a domain of its own, derived from `V`, and
        // `Rooted<V>.Pre` is a domain the struct declares.
        if let Some((head, member)) = name.split_once('.') {
            let head_span = Span::new(span.start, span.start + head.len() as u32);
            // `treap.Treap<Int>` after `use lib.treap`.
            if self.local(head).is_none() && self.env.is_module(head) {
                return match self.global(name) {
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
            let head_ty = self.named_type(head, head_span, Vec::new());
            if !matches!(head_ty, Ty::Pos(_) | Ty::Param(_) | Ty::Error | Ty::Unknown) {
                if let Some(chosen) = self.project(&head_ty, member) {
                    return chosen;
                }
            }
            return match head_ty {
                // `D.cuts` over a generic domain: lowering knows which one.
                Ty::Param(_) if matches!(member, "cuts" | "subsets" | "ranges") => Ty::Unknown,
                Ty::Param(p) => Ty::Assoc {
                    of: Box::new(Ty::Param(p)),
                    name: member.to_string(),
                },
                Ty::Pos(_) if member == "Cut" => {
                    let member_span = Span::new(head_span.end + 1, span.end);
                    self.planned(member_span, &cpl_syntax::planned::CUT_DOMAIN);
                    Ty::Error
                }
                // `V.ranges`: its cells are the non-empty ranges of `V`.
                Ty::Pos(domain) if member == "ranges" => Ty::Named {
                    name: "Range".into(),
                    args: vec![Ty::Pos(domain)],
                },
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
            self.hovers.push(HoverEntry {
                span,
                name: name.to_string(),
                ty: Ty::TypeName(name.to_string()),
                keyword: "type",
                decl: None,
                owner: None,
            });
            return Ty::Named {
                name: name.to_string(),
                args,
            };
        }
        if let Some(local) = self.local(name) {
            let resolved = match &local.ty {
                // `type Len = S.cuts` inside a fn.
                ty if local.keyword == "type" => Some(ty.clone()),
                Ty::Domain(domain) => Some(Ty::Pos(domain.clone())),
                _ => None,
            };
            if let Some(resolved) = resolved {
                let entry = HoverEntry {
                    span,
                    name: name.to_string(),
                    ty: local.ty.clone(),
                    keyword: local.keyword,
                    decl: Some(local.span),
                    owner: None,
                };
                self.hovers.push(entry);
                return resolved;
            }
        }
        let lookup_name = prefix_shape_struct(name, args.len()).unwrap_or(name);
        if self.defining_aliases.iter().any(|a| a == lookup_name) {
            if let Some(def) = self.env.outer_global(lookup_name) {
                return self.def_type(&def, lookup_name, span, args);
            }
            return self.alias_cycle(lookup_name, span);
        }
        if let Some(def) = self.global(lookup_name) {
            self.refs.push(GlobalRef {
                span,
                name: lookup_name.to_string(),
                top_level: false,
                in_lambda: false,
            });
            self.hovers.push(HoverEntry {
                span,
                name: name.to_string(),
                ty: Ty::TypeName(lookup_name.to_string()),
                keyword: "type",
                decl: None,
                owner: None,
            });
            return self.def_type(&def, lookup_name, span, args);
        }
        if self.env.declares_type(name) {
            let name = name.to_string();
            return Ty::Named { name, args };
        }
        if self.env.declares_alias(lookup_name) {
            return self.alias_cycle(lookup_name, span);
        }
        if let Some(replacement) = renamed_prefix_name(name) {
            let msg = format!("`{name}` was replaced by `{replacement}`");
            if let Some(diag) = self.error(codes::UNKNOWN_TYPE, span, msg) {
                diag.help = Some(format!(
                    "write `{replacement}` (or run `cpl fix` to migrate)"
                ));
                diag.fixes.push(cpl_diag::Fix {
                    title: format!("replace `{name}` with `{replacement}`"),
                    edits: vec![(span, replacement.to_string())],
                });
            }
            return Ty::Error;
        }
        if let Some(ty) = prelude::ty(name) {
            let expected_arity = match name {
                "List" | "Set" | "HashSet" | "Range" | "SortedRange" | "Iter" | "Gen" | "Seq" => {
                    Some(1)
                }
                "Map" | "HashMap" | "Indexed" => Some(2),
                _ => None,
            };
            if let Some(want) = expected_arity {
                if args.is_empty() {
                    let plural = if want == 1 { "" } else { "s" };
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        format!(
                            "`{name}` is a built-in type that takes {want} type argument{plural}"
                        ),
                    );
                    return Ty::Error;
                }
            }
            return match ty {
                Ty::Named { name, .. } => Ty::Named { name, args },
                other => other,
            };
        }
        // A declared alias with no definition yet is one whose right side
        // reaches back to it: `type A = B` with `type B = A`.
        if self.env.declares_alias(lookup_name) {
            return self.alias_cycle(lookup_name, span);
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
                    None => match self.global(name).as_deref() {
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
            ExprKind::Call { .. } => match self.quietly(|c| c.infer(expr)) {
                Ty::Range { lo, hi, .. } => (lo, hi),
                _ => (None, None),
            },
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
        for p in f
            .params
            .iter()
            .filter(|p| !(self_ty.is_some() && p.name.text == "self"))
        {
            let ty = match &p.ty {
                Some(ty) => self.resolve_type(ty),
                None => Ty::Unknown,
            };
            if p.variadic && !matches!(ty, Ty::Param(_) | Ty::Error) {
                let message = format!(
                    "`..{}` collects its arguments into a tuple, so its type is a type parameter, not `{ty}`",
                    p.name.text
                );
                if let Some(diag) = self.error(codes::TYPE_MISMATCH, p.span, message) {
                    diag.help = Some(format!(
                        "write `fn {}<Ts>(..{}: Ts)`, or take a `List<{ty}>` and pass `[a, b, c]`",
                        f.name.text, p.name.text
                    ));
                }
            }
            if p.lazy
                && !matches!(&ty, Ty::Fn { params, .. } if params.is_empty())
                && ty != Ty::Error
            {
                let message = format!(
                    "`lazy {}` is passed as an expression the fn runs when it calls `{}()`, so its type is `() -> T`, not `{ty}`",
                    p.name.text, p.name.text
                );
                if let Some(diag) = self.error(codes::TYPE_MISMATCH, p.span, message) {
                    diag.help = Some(format!("write `lazy {}: () -> {ty}`", p.name.text));
                }
            }
            let seen = Local {
                ty: ty.clone(),
                mutable: p.mutable,
                span: p.name.span,
                keyword: "param",
            };
            self.scopes
                .last_mut()
                .unwrap()
                .insert(p.name.text.clone(), seen);
            params.push(Param {
                name: p.label_text().to_string(),
                ty,
                mutable: p.mutable || self.note_opt_mut(p),
                has_default: p.default.is_some(),
                named_only: p.named_only,
                variadic: p.variadic,
                lazy: p.lazy,
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
    pub(super) fn def_type(&mut self, def: &Def, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        match def {
            Def::Domain(_) => Ty::Pos(name.to_string()),
            // `let (C, comp) = scc(..)`: a domain a fn made and returned.
            Def::Value {
                ty: Ty::Domain(domain),
                ..
            } => Ty::Pos(domain.clone()),
            Def::Alias { ty, generics } => ty.subst(&generics.iter().cloned().zip(args).collect()),
            Def::Struct {
                generics, defaults, ..
            } => {
                let mut args = args;
                if args.len() < defaults.len() {
                    let mut bindings: HashMap<String, Ty> = generics
                        .iter()
                        .zip(&args)
                        .map(|(g, a)| (g.clone(), a.clone()))
                        .collect();
                    for (i, default_ty) in defaults.iter().enumerate().skip(args.len()) {
                        if let Some(default_ty) = default_ty {
                            let filled = default_ty.subst(&bindings);
                            if let Some(g) = generics.get(i) {
                                bindings.insert(g.clone(), filled.clone());
                            }
                            args.push(filled);
                        } else {
                            break;
                        }
                    }
                }
                Ty::Named {
                    name: name.to_string(),
                    args,
                }
            }
            Def::Enum { .. } => Ty::Named {
                name: name.to_string(),
                args,
            },
            Def::Trait { .. } => Ty::Unknown,
            // Declared by a broken statement: its error is already reported.
            Def::Value { ty: Ty::Error, .. } | Def::Fn { ty: Ty::Error, .. } => Ty::Error,
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
        if let Some(Def::Struct { owned, .. }) = self.type_def(owner).as_deref() {
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
    ) -> (Named, Vec<String>, Vec<String>) {
        let (sigs, getters, mutators, _) = self.method_sigs_and_selves(methods, self_ty);
        (sigs, getters, mutators)
    }

    /// `method_sigs`, with the type each method that writes `self: ..`
    /// gives its receiver, in the method's own type parameters.
    pub(crate) fn method_sigs_and_selves(
        &mut self,
        methods: &[ast::Stmt],
        self_ty: &Ty,
    ) -> (Named, Vec<String>, Vec<String>, Named) {
        let mut getters = Vec::new();
        let mut mutators = Vec::new();
        let mut selves = Vec::new();
        let sigs = methods
            .iter()
            .filter_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) => {
                    if m.decorators.iter().any(|d| d.name.text == "get") {
                        getters.push(f.name.text.clone());
                    }
                    if f.params
                        .first()
                        .is_some_and(|p| p.mutable && p.name.text == "self")
                    {
                        mutators.push(f.name.text.clone());
                    }
                    let written = f.params.first().filter(|p| p.name.text == "self");
                    if let Some(ty) = written.and_then(|p| p.ty.as_ref()) {
                        let outer = self.generics.len();
                        self.generics
                            .extend(f.generics.iter().map(|g| g.text.clone()));
                        let ty = self.resolve_type(ty);
                        self.generics.truncate(outer);
                        selves.push((f.name.text.clone(), ty));
                    }
                    Some((f.name.text.clone(), self.fn_sig(f, Some(self_ty))))
                }
                _ => None,
            })
            .collect();
        (sigs, getters, mutators, selves)
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
                otherwise,
                ..
            } => {
                let keyword = if *mutable { "var" } else { "let" };
                let value_ty = match (ty, init) {
                    (Some(ty), _) => self.resolve_type(ty),
                    (None, Some(init)) => {
                        let inferred = self.infer(init);
                        let rebased = match &pat.kind {
                            ast::PatternKind::Name(name) => self.rebased(name, init),
                            _ => None,
                        };
                        if let Some(rebased) = rebased {
                            rebased
                        } else if *mutable && matches!(inferred, Ty::Range { .. }) {
                            Ty::Int
                        } else if !*mutable && inferred == Ty::Int {
                            let (lo, hi) = self.bounds(init);
                            if let (Some(l), Some(h)) = (lo, hi) {
                                Ty::Range {
                                    lo: Some(l),
                                    hi: Some(h),
                                    text: format!("{l}..={h}"),
                                }
                            } else {
                                inferred
                            }
                        } else {
                            inferred
                        }
                    }
                    (None, None) => Ty::Error,
                };
                let value_ty = match (ty, init, otherwise) {
                    (None, Some(init), Some(_)) => {
                        self.quietly(|c| c.let_else(pat, init, value_ty))
                    }
                    _ => value_ty,
                };
                let mut out = Vec::new();
                let is_mut = *mutable
                    || matches!(
                        init,
                        Some(ast::Expr {
                            kind: ast::ExprKind::Mut(_),
                            ..
                        })
                    );
                let mut value = |name: &str, ty: Ty| {
                    out.push((
                        name.to_string(),
                        Def::Value {
                            ty,
                            mutable: is_mut,
                            keyword,
                            generics: Vec::new(),
                            settled: None,
                        },
                    ))
                };
                match domain_binding(pat) {
                    // `let LIM = 100`: a capitalized name bound to a value.
                    Some(name) => value(name, value_ty.clone()),
                    None => {
                        let mut payload =
                            |path: &[ast::Name], ty: &Ty| self.variant_payload(path, ty);
                        typed_pattern(pat, &value_ty, &mut payload, &mut value)
                    }
                }
                if let Some(ty) = ty {
                    own_domains(ty, &mut |name| {
                        out.push((
                            name.text.clone(),
                            Def::Domain(DomainInfo {
                                text: format!("the positions of `{}`", self.text(pat.span)),
                                max_size: None,
                                min_pos: None,
                                max_pos: None,
                                bound: None,
                                factors: Vec::new(),
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
                let mut row_bounds = Vec::new();
                let mut trait_bounds = Vec::new();
                for b in &f.bounds {
                    for t in &b.traits {
                        match &t.kind {
                            TypeKind::Row(fields) => {
                                let resolved = fields
                                    .iter()
                                    .map(|(n, ty)| (n.text.clone(), self.resolve_type(ty)))
                                    .collect();
                                row_bounds.push((b.name.text.clone(), resolved));
                            }
                            TypeKind::Named { name: tname, .. } => {
                                trait_bounds.push((b.name.text.clone(), tname.text.clone()));
                            }
                            _ => {}
                        }
                    }
                }
                out.push((
                    f.name.text.clone(),
                    Def::Fn {
                        ty,
                        generics: f.generics.iter().map(|g| g.text.clone()).collect(),
                        bounds: super::views::by_walk_bounds(f, extract_bounds(&f.bounds)),
                        lends: f.lends.is_some(),
                        row_bounds,
                        trait_bounds,
                        droppable: stmt.decorators.iter().any(|d| d.name.text == "droppable"),
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
                let defaults: Vec<Option<Ty>> = s
                    .generic_defaults
                    .iter()
                    .map(|d| d.as_ref().map(|ty| self.resolve_type(ty)))
                    .collect();
                let owned = self.owned(&s.name.text, &s.methods);
                let fields = s
                    .fields
                    .iter()
                    .map(|f| Param {
                        name: f.name.text.clone(),
                        ty: self.field_type(f),
                        mutable: false,
                        has_default: f.default.is_some(),
                        named_only: false,
                        variadic: false,
                        lazy: false,
                    })
                    .collect();
                let var_fields: Vec<String> = s
                    .fields
                    .iter()
                    .filter(|f| f.mutable)
                    .map(|f| f.name.text.clone())
                    .collect();
                let (methods, getters, mutators, selves) =
                    self.method_sigs_and_selves(&s.methods, &self_ty);
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
                let receivers = s
                    .methods
                    .iter()
                    .filter_map(|m| match &m.kind {
                        ast::StmtKind::Fn(f)
                            if f.params.first().is_some_and(|p| p.name.text == "self") =>
                        {
                            Some(f.name.text.clone())
                        }
                        _ => None,
                    })
                    .collect();
                vec![(
                    s.name.text.clone(),
                    Def::Struct {
                        generics,
                        defaults,
                        fields,
                        var_fields,
                        methods,
                        ctor,
                        getters,
                        mutators,
                        receivers,
                        selves,
                        owned,
                        projections: self.projections(&s.methods),
                        cached_fields: s
                            .fields
                            .iter()
                            .filter(|f| f.cached)
                            .map(|f| f.name.text.clone())
                            .collect(),
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
                                generics: generics.clone(),
                            },
                        )
                    })
                    .collect();
                let tagged = e.variants.iter().any(|v| v.tag.is_some())
                    || stmt.decorators.iter().any(|d| d.name.text == "tags");
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
            S::TypeAlias {
                name, generics, ty, ..
            } => {
                let ty = self.resolve_alias(name, generics, ty);
                let generics = generics.iter().map(|g| g.text.clone()).collect();
                vec![(name.text.clone(), Def::Alias { ty, generics })]
            }
            S::Trait {
                name,
                generics,
                supers,
                body,
                alias,
                ..
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
                let fields: Vec<(String, Ty)> = body
                    .stmts
                    .iter()
                    .filter_map(|s| match &s.kind {
                        S::Let {
                            pat:
                                ast::Pattern {
                                    kind: ast::PatternKind::Name(fname),
                                    ..
                                },
                            ty: Some(fty),
                            init: None,
                            ..
                        } => Some((fname.clone(), self.resolve_type(fty))),
                        _ => None,
                    })
                    .collect();
                let assoc = super::assoc::declared(&body.stmts)
                    .into_iter()
                    .map(|(n, bounds, default)| {
                        let default = default.map(|t| self.quietly(|c| c.resolve_type(t)));
                        (n, bounds, default)
                    })
                    .collect();
                self.leave_body(scope);
                vec![(
                    name.text.clone(),
                    Def::Trait {
                        supers: supers.iter().filter_map(type_head).collect(),
                        methods,
                        fields,
                        alias: *alias,
                        assoc,
                    },
                )]
            }
            S::Impl {
                trait_name,
                target,
                bounds,
                body,
            } => {
                let Some(target_name) = impl_target_head(target) else {
                    return Vec::new();
                };
                let generics = self.impl_generics(trait_name.as_ref(), target, bounds);
                let scope = self.enter_body(&generics, Ty::Unknown);
                let self_ty = self.quietly(|c| c.resolve_type(target));
                self.self_ty = Some(self_ty.clone());
                let trait_args: Vec<Ty> = match trait_name.as_ref().map(|t| &t.kind) {
                    Some(TypeKind::Named { args, .. }) => args
                        .iter()
                        .map(|a| self.quietly(|c| c.resolve_type(a)))
                        .collect(),
                    _ => Vec::new(),
                };
                let out_ty: Option<Ty> = body.stmts.iter().find_map(|s| match &s.kind {
                    ast::StmtKind::TypeAlias { name, ty, .. } if name.text == "Out" => {
                        Some(self.quietly(|c| c.resolve_type(ty)))
                    }
                    _ => None,
                });
                let assoc: Vec<(String, Ty)> = body
                    .stmts
                    .iter()
                    .filter_map(|s| match &s.kind {
                        S::TypeAlias {
                            name, generics, ty, ..
                        } if generics.is_empty() => {
                            Some((name.text.clone(), self.quietly(|c| c.resolve_type(ty))))
                        }
                        _ => None,
                    })
                    .collect();
                let assoc = self.with_assoc_defaults(trait_name.as_ref(), &self_ty, assoc);
                let outer_assoc = std::mem::replace(&mut self.self_assoc, assoc.clone());
                let owned = self.owned(&target_name, &body.stmts);
                let (methods, getters, mutators) = self.method_sigs(&body.stmts, &self_ty);
                self.self_assoc = outer_assoc;
                self.leave_body(scope);
                let trait_head = trait_name.as_ref().and_then(trait_impl_key);
                vec![(
                    crate::def::impl_name(trait_head.as_deref(), &target_name),
                    Def::Impl {
                        trait_args,
                        out_ty,
                        methods,
                        getters,
                        mutators,
                        owned,
                        projections: self.projections(&body.stmts),
                        target: self_ty,
                        assoc,
                    },
                )]
            }
            ast::StmtKind::Context { name, fields } => {
                let fs = fields
                    .iter()
                    .map(|f| (f.name.text.clone(), self.field_type(f)))
                    .collect();
                vec![(
                    name.text.clone(),
                    Def::Value {
                        ty: Ty::Record(fs),
                        mutable: false,
                        keyword: "context",
                        generics: Vec::new(),
                        settled: None,
                    },
                )]
            }
            _ => Vec::new(),
        }
    }

    /// Whether `name` is a finite-index enum (§2.16): fieldless or carrying only
    /// finite-index payloads (`Ty::Pos`, `Ty::Bool`, or another finite-index enum).
    /// Returns `Some(max_size)` when so (`max_size` is `None` if any factor is unbounded).
    pub(crate) fn finite_index_enum_size(&mut self, name: &str) -> Option<Option<i128>> {
        fn go(
            env: &mut dyn crate::check::Env,
            name: &str,
            seen: &mut crate::fx::HashSet<String>,
        ) -> Option<Option<i128>> {
            if !seen.insert(name.to_string()) {
                return None;
            }
            let Some(Def::Enum { variants, .. }) = env.type_def(name).as_deref().cloned() else {
                seen.remove(name);
                return None;
            };
            if variants.is_empty() {
                seen.remove(name);
                return None;
            }
            let mut total: Option<i128> = Some(0);
            for (_, fs) in &variants {
                let mut prod: Option<i128> = Some(1);
                for f in fs {
                    let fmax = match f {
                        Ty::Bool => Some(2),
                        Ty::Pos(d) => match env.global(d).as_deref() {
                            Some(Def::Domain(info)) => info.max_size,
                            _ => None,
                        },
                        Ty::Named { name: inner, args } if args.is_empty() => {
                            go(env, inner.as_str(), seen)?
                        }
                        _ => {
                            seen.remove(name);
                            return None;
                        }
                    };
                    prod = prod.zip(fmax).and_then(|(a, b)| a.checked_mul(b));
                }
                total = total.zip(prod).and_then(|(a, b)| a.checked_add(b));
            }
            seen.remove(name);
            Some(total)
        }
        go(self.env, name, &mut std::collections::HashSet::default())
    }

    fn product_factor_info(&mut self, e: &ast::Expr) -> Option<(Ty, Option<i128>)> {
        match &e.kind {
            ExprKind::Name(name) if name == "Bool" => Some((Ty::Bool, Some(2))),
            ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => {
                let size = match (self.bounds(lo).0, self.bounds(hi).1) {
                    (Some(lo), Some(hi)) => Some((hi - lo + i128::from(*inclusive)).max(0)),
                    _ => None,
                };
                Some((Ty::Int, size))
            }
            ExprKind::Name(name) => {
                if let Some(local) = self.local(name) {
                    if let Ty::Domain(d) = &local.ty {
                        return Some((Ty::Pos(d.clone()), None));
                    }
                }
                match self.global(name).as_deref() {
                    Some(Def::Domain(info)) => Some((Ty::Pos(name.clone()), info.max_size)),
                    _ => self.finite_index_enum_size(name).map(|max| {
                        (
                            Ty::Named {
                                name: name.clone(),
                                args: Vec::new(),
                            },
                            max,
                        )
                    }),
                }
            }
            _ => None,
        }
    }

    /// Size and provenance of `domain V = <def>`.
    pub(crate) fn domain_info(&mut self, def: &ast::Expr) -> DomainInfo {
        let text = self.text(def.span).to_string();
        let mut factors = Vec::new();
        let mut min_pos = None;
        let mut max_pos = None;
        let (max_size, bound) = match &def.kind {
            ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => {
                let lo_min = self.bounds(lo).0;
                let hi_max = self.bounds(hi).1;
                min_pos = lo_min;
                max_pos = hi_max.map(|h| if *inclusive { h } else { h - 1 });
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
            // `domain State = (V, K)`: a Cartesian product of finite domains/enums/Bool.
            ExprKind::Tuple(items) if items.len() >= 2 => {
                let mut resolved = Vec::with_capacity(items.len());
                let mut prod: Option<i128> = Some(1);
                let mut all_ok = true;
                for item in items {
                    if let Some((ty, sz)) = self.product_factor_info(item) {
                        resolved.push((None, ty));
                        prod = prod.zip(sz).and_then(|(a, b)| a.checked_mul(b));
                    } else {
                        all_ok = false;
                        break;
                    }
                }
                if all_ok {
                    factors = resolved;
                    (prod, None)
                } else {
                    (None, None)
                }
            }
            // `domain State = (v: V, k: K)`: a named Cartesian product domain.
            ExprKind::Record(fields) if fields.len() >= 2 => {
                let mut resolved = Vec::with_capacity(fields.len());
                let mut prod: Option<i128> = Some(1);
                let mut all_ok = true;
                for (fname, fval) in fields {
                    if let Some((ty, sz)) = self.product_factor_info(fval) {
                        resolved.push((Some(fname.text.clone()), ty));
                        prod = prod.zip(sz).and_then(|(a, b)| a.checked_mul(b));
                    } else {
                        all_ok = false;
                        break;
                    }
                }
                if all_ok {
                    factors = resolved;
                    (prod, None)
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };
        DomainInfo {
            text,
            max_size,
            min_pos,
            max_pos,
            bound,
            factors,
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

/// Key for a trait in an `impl` header, preserving type arguments so
/// `impl Mul for Vec` and `impl Mul<F64> for Vec` get distinct names.
pub(crate) fn trait_impl_key(ty: &ast::Type) -> Option<String> {
    match &ty.kind {
        TypeKind::Named { name, args } if !name.text.is_empty() => {
            if args.is_empty() {
                Some(name.text.clone())
            } else {
                let parts: Vec<String> = args
                    .iter()
                    .map(|a| trait_impl_key(a).unwrap_or_else(|| "_".to_string()))
                    .collect();
                Some(format!("{}<{}>", name.text, parts.join(",")))
            }
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
        TypeKind::Optional(inner) | TypeKind::Mut(inner) => own_domains(inner, f),
        TypeKind::Tuple(items) => items.iter().for_each(|i| own_domains(i, f)),
        TypeKind::Record(fields) => fields.iter().for_each(|(_, t)| own_domains(t, f)),
        TypeKind::Named { args, .. } => args.iter().for_each(|a| own_domains(a, f)),
        _ => {}
    }
}
/// Extracts `(param_name, [trait_names])` from AST `bounds`.
pub(crate) fn extract_bounds(bounds: &[ast::Bound]) -> Vec<(String, Vec<String>)> {
    bounds
        .iter()
        .map(|b| {
            let traits = b.traits.iter().filter_map(type_head).collect();
            (b.name.text.clone(), traits)
        })
        .collect()
}

impl<'a> Checker<'a> {
    /// Checks a struct's or impl's methods with its `bounds` in scope, as its generics are.
    pub(crate) fn bounded_methods(
        &mut self,
        methods: &[ast::Stmt],
        self_ty: &Ty,
        bounds: &[ast::Bound],
    ) {
        let outer = self.param_bounds.len();
        self.param_bounds.extend(extract_bounds(bounds));
        self.methods(methods, self_ty);
        self.param_bounds.truncate(outer);
    }

    /// True when trait `sub` is `want` or transitively inherits from `want`.
    pub(crate) fn trait_implies(&mut self, sub: &str, want: &str) -> bool {
        let mut stack = vec![sub.to_string()];
        let mut seen = Vec::new();
        while let Some(cur) = stack.pop() {
            if cur == want {
                return true;
            }
            if seen.iter().any(|s| s == &cur) {
                continue;
            }
            seen.push(cur.clone());
            match cur.as_str() {
                "Ord" => stack.push("Eq".into()),
                "IndexedMut" => stack.push("Indexed".into()),
                "Indexed" => {
                    stack.push("Sized".into());
                    stack.push("Bidir".into());
                    stack.push("Keyed".into());
                }
                "Sized" | "Bidir" | "Keyed" => stack.push("Seq".into()),
                _ => {}
            }
            if let Some(Def::Trait { supers, .. }) = self.global(&cur).as_deref() {
                stack.extend(supers.iter().cloned());
            }
        }
        false
    }

    /// True when the in-scope type parameter `param` has a bound implying `trait_name`.
    pub(crate) fn param_has_bound(&mut self, param: &str, trait_name: &str) -> bool {
        let traits: Vec<String> = self
            .param_bounds
            .iter()
            .filter(|(p, _)| p == param)
            .flat_map(|(_, ts)| ts.iter().cloned())
            .collect();
        traits.iter().any(|t| self.trait_implies(t, trait_name))
    }

    /// True when `ty` implements `trait_name`, via explicit `impl`, structural
    /// derivation (`Eq`/`Ord`/`Hash`/`Print`), built-in sequence capability, or
    /// type-parameter bounds.
    pub(crate) fn implements_trait(&mut self, ty: &Ty, trait_name: &str) -> bool {
        let mut seen = Vec::new();
        self.implements_trait_inner(ty, trait_name, &mut seen)
    }

    fn implements_trait_inner(
        &mut self,
        ty: &Ty,
        trait_name: &str,
        seen: &mut Vec<String>,
    ) -> bool {
        match ty {
            Ty::Error | Ty::Unknown => return true,
            Ty::Param(p) => {
                if !self.generics.iter().any(|g| g == p) {
                    return true;
                }
                return self.param_has_bound(p, trait_name);
            }
            _ => {}
        }
        if let Some(Def::Trait {
            supers,
            alias: true,
            ..
        }) = self.type_def(trait_name).as_deref().cloned()
        {
            return supers
                .iter()
                .all(|s| self.implements_trait_inner(ty, s, seen));
        }
        let impl_key = match ty {
            Ty::Named { name, .. } => Some(name.as_str()),
            Ty::Int => Some("Int"),
            Ty::I128 => Some("I128"),
            Ty::U64 => Some("U64"),
            Ty::Float => Some("Float"),
            Ty::Bool => Some("Bool"),
            Ty::Char => Some("Char"),
            Ty::Str => Some("Str"),
            Ty::Fn { .. } => Some("fn"),
            _ => None,
        };
        if impl_key.is_some_and(|name| self.has_impl(name, trait_name)) {
            return true;
        }
        match trait_name {
            "Eq" | "Ord" | "Hash" => self.derives_structural(ty, trait_name, seen),
            "Show" => self.derives_print(ty, false, seen),
            "Seq" => ty.is_seq() || self.walks_by_each(ty),
            "Sized" => ty.is_sized(),
            "Indexed" => ty.is_indexed(),
            super::views::INDEXED_BY_WALK => ty.rev_source().unwrap_or(ty).is_indexed(),
            "Bidir" => ty.is_bidir(),
            "Keyed" => ty.is_keyed(),
            "IndexedMut" => ty.is_indexed_mut(),
            "Contains" => match ty {
                Ty::Str | Ty::Domain(_) => true,
                Ty::Named { name, .. } => matches!(
                    name.as_str(),
                    "Set"
                        | "HashSet"
                        | "SortedSet"
                        | "SortedBag"
                        | "Map"
                        | "HashMap"
                        | "SortedMap"
                        | "Bitset"
                        | "Range"
                        | "Counter"
                        | "Bag"
                ),
                _ => false,
            },
            "Add" => ty.is_numeric() || matches!(ty, Ty::Str),
            "Sub" | "Mul" | "Div" | "Rem" | "Pow" | "Neg" | "Num" => ty.is_numeric(),
            "Index" => ty.is_int() || matches!(ty, Ty::Pos(_) | Ty::Char),
            "Domain" => match ty {
                Ty::Pos(_) | Ty::Domain(_) => true,
                Ty::Named { name, .. } => self.has_impl(name, "Index"),
                _ => false,
            },
            "Target" => match ty {
                Ty::Pos(_) | Ty::Int => true,
                Ty::Record(fields) => fields.iter().any(|(n, _)| n == "to"),
                _ => false,
            },
            "Weighted" => match ty {
                Ty::Record(fields) => fields.iter().any(|(n, _)| n == "w"),
                _ => false,
            },
            "Next" | "WeightedNext" | "EdgedNext" => match ty {
                Ty::Array { .. } | Ty::Fn { .. } => true,
                Ty::Named { name, .. } => matches!(
                    name.as_str(),
                    "Graph" | "Reweighted" | "RevGraph" | "FilteredGraph" | "Adj"
                ),
                _ => false,
            },
            _ => match ty {
                Ty::Record(fields) => self.meets_by_shape(fields, trait_name),
                _ => false,
            },
        }
    }

    /// R04.2.10: the factor positions one array axis stands for, `[P]T` with
    /// `domain P = (R, C)` or `type P = (R, C)`.
    pub(crate) fn axis_factors(&mut self, axis: &Ty) -> Option<Vec<Ty>> {
        match axis {
            Ty::Pos(d) => Some(
                self.product_factors(d)?
                    .into_iter()
                    .map(|(_, t)| t)
                    .collect(),
            ),
            Ty::Tuple(items) if items.len() > 1 => Some(items.clone()),
            _ => None,
        }
    }

    /// R08.1.7: a record meets a trait when a field meets each member the trait leaves without a
    /// body, a field `w: W` or a bodiless `@get fn w(self) -> W` alike, with its type.
    pub(crate) fn meets_by_shape(&mut self, fields: &[(String, Ty)], trait_name: &str) -> bool {
        let Some(Def::Trait {
            methods,
            fields: wanted,
            ..
        }) = self.global(trait_name).as_deref().cloned()
        else {
            return true;
        };
        let getters = methods
            .into_iter()
            .filter(|m| m.required)
            .map(|m| match m.ty {
                Ty::Fn { params, ret } if params.len() <= 1 => (m.name, *ret),
                _ => (m.name, Ty::Unknown),
            });
        let wanted: Vec<(String, Ty)> = getters.chain(wanted).collect();
        wanted.iter().all(|(name, want)| {
            fields.iter().any(|(f, got)| {
                f == name && (matches!(want, Ty::Param(_) | Ty::Unknown) || got.fits(want))
            })
        })
    }

    fn derives_structural(&mut self, ty: &Ty, trait_name: &str, seen: &mut Vec<String>) -> bool {
        match ty {
            Ty::Error | Ty::Unknown | Ty::Const(_) => true,
            Ty::Param(p) => self.param_has_bound(p, trait_name),
            // Its trait's `type X: Bound` is not tracked yet, so it is taken as met.
            Ty::Assoc { .. } => true,
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Bool
            | Ty::Char
            | Ty::Str
            | Ty::Unit
            | Ty::Range { .. }
            | Ty::Pos(_)
            | Ty::Domain(_) => true,
            Ty::Float => trait_name != "Hash",
            Ty::Fn { .. } => false,
            Ty::Optional(inner) => self.implements_trait_inner(inner, trait_name, seen),
            Ty::Tuple(items) => items
                .iter()
                .all(|t| self.implements_trait_inner(t, trait_name, seen)),
            Ty::Record(fields) => fields
                .iter()
                .all(|(_, t)| self.implements_trait_inner(t, trait_name, seen)),
            Ty::Array { elem, .. } => self.implements_trait_inner(elem, trait_name, seen),
            Ty::Named { name, args } => {
                if seen.iter().any(|s| s == name) {
                    return true;
                }
                if trait_name == "Ord" && matches!(name.as_str(), "HashMap" | "HashSet") {
                    return false;
                }
                match self.type_def(name).as_deref().cloned() {
                    Some(Def::Struct {
                        generics, fields, ..
                    }) => {
                        seen.push(name.clone());
                        let mut bindings = crate::ty::Bindings::default();
                        for (g, a) in generics.iter().zip(args) {
                            bindings.insert(g.clone(), a.clone());
                        }
                        let ok = fields.iter().all(|f| {
                            let fty = f.ty.subst(&bindings);
                            self.implements_trait_inner(&fty, trait_name, seen)
                        });
                        seen.pop();
                        ok
                    }
                    Some(Def::Enum { variants, .. }) => {
                        seen.push(name.clone());
                        let ok = variants.iter().all(|(_, fs)| {
                            fs.iter()
                                .all(|fty| self.implements_trait_inner(fty, trait_name, seen))
                        });
                        seen.pop();
                        ok
                    }
                    _ => args
                        .iter()
                        .all(|a| self.implements_trait_inner(a, trait_name, seen)),
                }
            }
            Ty::RowWithout(_, inner, _) => self.implements_trait_inner(inner, trait_name, seen),
            Ty::TypeName(_) => false,
        }
    }

    /// `expect_fits` for a move into a binding or place: an array's range
    /// elements don't widen, as that rewrites every cell (R04.4.24).
    pub(crate) fn expect_moves(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: impl FnOnce(&Self, &Ty, &Ty) -> String,
    ) -> bool {
        if let (Ty::Array { elem: ge, .. }, Ty::Array { elem: we, .. }) = (got, want) {
            if matches!(**ge, Ty::Range { .. }) && we.is_int() && ge != we {
                let msg = message(self, got, want);
                let help = format!(
                    "convert each cell: `{}.map(x => {we}(x))`",
                    self.quote(at.span)
                );
                if let Some(diag) = self.error(codes::TYPE_MISMATCH, at.span, msg) {
                    diag.help = Some(help);
                }
                return false;
            }
        }
        self.expect_fits(got, want, at, message)
    }

    /// Is there a written `impl trait_name for name`?
    pub(super) fn has_impl(&mut self, name: &str, trait_name: &str) -> bool {
        self.global(&crate::def::impl_name(Some(trait_name), name))
            .is_some()
            || self
                .env
                .impls(name)
                .into_iter()
                .filter_map(|(t, _)| t)
                .any(|t| t.split('<').next().unwrap_or(&t) == trait_name)
    }

    /// True when `ty` can be passed to `print(...)`: a written `impl Show`
    /// is its print hook, else it prints field by field.
    /// When `allow_none` is true (`print(x, none: "...")`), `?T` is printable if `T` is.
    pub(super) fn derives_print(&mut self, ty: &Ty, none: bool, seen: &mut Vec<String>) -> bool {
        let allow_none = none;
        match ty {
            Ty::Error | Ty::Unknown | Ty::Const(_) => true,
            Ty::Param(p) => self.param_has_bound(p, "Show"),
            // Its trait's `type X: Bound` is not tracked yet, so it is taken as met.
            Ty::Assoc { .. } => true,
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Char
            | Ty::Str
            | Ty::Range { .. }
            | Ty::Pos(_)
            | Ty::Unit => true,
            Ty::Bool => false,
            Ty::Optional(inner) => {
                if matches!(inner.as_ref(), Ty::Pos(_)) {
                    return true;
                }
                allow_none && self.derives_print(inner, allow_none, seen)
            }
            Ty::Fn { params, ret } => {
                params.is_empty() && self.derives_print(ret, allow_none, seen)
            }
            Ty::Tuple(items) => items
                .iter()
                .all(|t| self.derives_print(t, allow_none, seen)),
            Ty::Record(fields) => fields
                .iter()
                .all(|(_, t)| self.derives_print(t, allow_none, seen)),
            Ty::Array { elem, .. } => self.derives_print(elem, allow_none, seen),
            Ty::Named { name, args } => {
                if seen.iter().any(|s| s == name) || self.has_impl(name, "Show") {
                    return true;
                }
                if matches!(name.as_str(), "HashMap" | "HashSet") {
                    return false;
                }
                if name == "List" && args.len() == 1 {
                    return self.derives_print(&args[0], allow_none, seen);
                }
                match self.type_def(name).as_deref().cloned() {
                    Some(Def::Struct {
                        generics, fields, ..
                    }) => {
                        seen.push(name.clone());
                        let mut bindings = crate::ty::Bindings::default();
                        for (g, a) in generics.iter().zip(args) {
                            bindings.insert(g.clone(), a.clone());
                        }
                        let ok = fields.iter().all(|f| {
                            let fty = f.ty.subst(&bindings);
                            self.derives_print(&fty, allow_none, seen)
                        });
                        seen.pop();
                        ok
                    }
                    Some(Def::Enum {
                        variants, tagged, ..
                    }) => {
                        if tagged {
                            return true;
                        }
                        seen.push(name.clone());
                        let ok = variants.iter().all(|(_, fs)| {
                            fs.iter()
                                .all(|fty| self.derives_print(fty, allow_none, seen))
                        });
                        seen.pop();
                        ok
                    }
                    _ => true,
                }
            }
            Ty::RowWithout(_, inner, _) => self.derives_print(inner, allow_none, seen),
            Ty::Domain(_) | Ty::TypeName(_) => true,
        }
    }

    /// Reports `expr` if it is known not to be a `Bool`.
    pub(crate) fn require_bool(&mut self, ty: &Ty, expr: &Expr, what: &str) {
        if ty.is_vague() || *ty == Ty::Bool {
            return;
        }
        if !(ty.is_numeric() || matches!(ty, Ty::Str | Ty::Char | Ty::Pos(_) | Ty::Optional(_))) {
            return;
        }
        let quoted = self.quote(expr.span);
        let message = format!(
            "{what} needs a Bool, but `{quoted}` is {}",
            ty.widen().with_article()
        );
        let zero = match ty {
            Ty::Optional(_) => Some("None"),
            t if t.is_numeric() => Some("0"),
            _ => None,
        };
        let simple = matches!(
            expr.kind,
            ExprKind::Name(_)
                | ExprKind::Field { .. }
                | ExprKind::Call { .. }
                | ExprKind::Index { .. }
                | ExprKind::Unary { .. }
        ) || matches!(&expr.kind, ExprKind::Binary { op, .. }
            if !matches!(op, ast::BinOp::And | ast::BinOp::Or | ast::BinOp::Coalesce));
        if let Some(diag) = self.error(codes::CONDITION_NOT_BOOL, expr.span, message) {
            let Some(zero) = zero else {
                diag.help = Some("write a comparison".to_string());
                return;
            };
            diag.help = Some(format!("compare it: `{quoted} != {zero}`"));
            if simple {
                diag.fix(
                    format!("compare with `!= {zero}`"),
                    Span::point(expr.span.end),
                    format!(" != {zero}"),
                );
            }
        }
    }
}
