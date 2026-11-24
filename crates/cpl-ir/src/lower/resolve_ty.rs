//! Resolving written types to IR types: named types, array dimensions,
//! domains, and the ranges a read struct's fields carry.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn resolve_ty(&mut self, ty: &ast::Type) -> Ty {
        self.resolve(ty, false).ty()
    }

    /// The type `ty` names. With `values`, also the lengths and ranges
    /// written in it, lowered where the type stands.
    /// The type `ty` names; a library type it instantiates is asked for here.
    pub(in crate::lower) fn resolve(&mut self, ty: &ast::Type, values: bool) -> Spec {
        let outer = self
            .asked_at
            .enter(ty.span.shift(self.offset), self.cur_mod);
        let spec = self.resolve_at(ty, values);
        self.asked_at.leave(outer);
        spec
    }

    pub(in crate::lower) fn resolve_at(&mut self, ty: &ast::Type, values: bool) -> Spec {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => self.named(name, args, values),
            ast::TypeKind::Array { index, elem }
                if matches!(index.as_slice(), [ast::Type { kind: ast::TypeKind::Named { name, args }, .. }]
                    if args.is_empty() && self.gen_env.contains_key(&mono::int_key_marker(&name.text))) =>
            {
                // `[K]T` keyed by a sequence's `Int` keys (R04b.5.3): those start
                // wherever the sequence's do, so the table carries its first key.
                let elem = self.resolve(elem, values).ty();
                Spec::Plain(Ty::Keyed(Box::new(elem), Dim::Int))
            }
            // `[D]T` with `D = Int` is a list (R07.3.5): `List<T>` is `Indexed<Int, T>`.
            ast::TypeKind::Array { index, elem }
                if matches!(index.as_slice(), [ast::Type { kind: ast::TypeKind::Named { name, args }, .. }]
                    if args.is_empty() && self.gen_env.get(&name.text) == Some(&GenArg::Ty(Ty::Int))
                        && !self.gen_env.contains_key(&mono::own_generic_marker(&name.text))) =>
            {
                Spec::Plain(Ty::list(self.resolve(elem, values).ty()))
            }
            ast::TypeKind::Array { index, elem } => {
                // `[V]T` where `V` stands for `(R, C)`: one dimension for
                // each part, as `[R, C]T`.
                let mut dims: Vec<Option<DimSpec>> = Vec::new();
                for d in index {
                    let parts = self.product_dims(d);
                    match parts {
                        Some(parts) => dims.extend(parts.into_iter().map(Some)),
                        None => dims.push(self.dim(d, values)),
                    }
                }
                let elem = self.resolve(elem, values);
                match dims.into_iter().collect::<Option<Vec<_>>>() {
                    Some(dims) => Spec::Array {
                        dims,
                        elem: Box::new(elem),
                    },
                    None => Spec::Plain(Ty::Error),
                }
            }
            ast::TypeKind::Optional(inner) => {
                Spec::Opt(Box::new(self.resolve(inner, values)), None)
            }
            ast::TypeKind::Tuple(items) if items.is_empty() => Spec::Plain(Ty::Unit),
            ast::TypeKind::Tuple(items) => {
                let specs: Vec<Spec> = items.iter().map(|t| self.resolve(t, values)).collect();
                Spec::Fields(Ty::Tuple(specs.iter().map(Spec::ty).collect()), specs)
            }
            ast::TypeKind::Record(fields) => {
                let specs: Vec<Spec> = fields
                    .iter()
                    .map(|(_, t)| self.resolve(t, values))
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&specs)
                        .map(|((name, _), spec)| (name.text.clone(), spec.ty()))
                        .collect(),
                );
                Spec::Fields(ty, specs)
            }
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !values {
                    return Spec::Plain(Ty::Int);
                }
                let lo = lo.as_deref().map(|lo| Box::new(self.expr_as(lo, &Ty::Int)));
                let hi = hi.as_deref().map(|hi| {
                    let mut hi = self.expr_as(hi, &Ty::Int);
                    if !inclusive {
                        hi = self.int_add(hi, -1);
                    }
                    Box::new(hi)
                });
                Spec::Int {
                    lo,
                    hi,
                    text: self.source(ty.span),
                }
            }
            ast::TypeKind::OwnDomain { name, elem } => {
                let dom_id = self.domains.len();
                self.domains.push(name.text.clone());
                self.own_domains.insert(dom_id);
                if self.cx.in_body {
                    let scope = self.cx.local_syms.top_mut();
                    scope.insert(name.text.clone(), Sym::Domain(dom_id));
                } else {
                    self.mod_syms[self.cur_mod].insert(name.text.clone(), Sym::Domain(dom_id));
                }
                let elem = self.resolve(elem, values);
                Spec::Array {
                    dims: vec![DimSpec::Pos(dom_id)],
                    elem: Box::new(elem),
                }
            }
            ast::TypeKind::Fn { params, ret } => {
                let ps: Vec<Ty> = params.iter().map(|p| self.resolve_ty(p)).collect();
                let r = self.resolve_ty(ret);
                Spec::Plain(Ty::Fn(ps, Box::new(r)))
            }
            ast::TypeKind::Mut(inner) => Spec::Plain(Ty::Mut(Box::new(self.resolve_ty(inner)))),
            ast::TypeKind::Value(_) => Spec::Plain(Ty::Int),
            ast::TypeKind::NewDomain | ast::TypeKind::DomainBinder(_) => Spec::Plain(Ty::DomainVal),
            ast::TypeKind::Yields(inner) => Spec::Plain(Ty::list(self.resolve_ty(inner))),
            ast::TypeKind::ConstrainedStr { .. } => self.str_spec(ty, values),
            ast::TypeKind::Row(_) => Spec::Plain(Ty::Error),
            ast::TypeKind::RowWithout {
                prefix,
                base,
                removed,
            } => {
                let mut out_fields: Vec<(String, Ty)> = prefix
                    .iter()
                    .map(|(n, t)| (n.text.clone(), self.resolve_ty(t)))
                    .collect();
                let bty = self.resolve_ty(base);
                let ty = match bty {
                    Ty::Record(fs) => {
                        out_fields.extend(
                            fs.into_iter()
                                .filter(|(n, _)| !removed.iter().any(|r| r.text == *n)),
                        );
                        Ty::Record(out_fields)
                    }
                    other if out_fields.is_empty() => other,
                    _ => Ty::Record(out_fields),
                };
                Spec::Plain(ty)
            }
            ast::TypeKind::LiteralUnion(lits) => {
                let ty = match lits.first().map(|s| s.as_str()) {
                    Some(s) if s.starts_with('\'') => Ty::Char,
                    Some(s) if s.starts_with('"') => Ty::Str,
                    _ => Ty::Int,
                };
                Spec::Plain(ty)
            }
            ast::TypeKind::ItemOf(item) => Spec::Plain(match item.fn_item_ident() {
                Some(ident) => Ty::Struct(self.item_type(&ident)),
                None => Ty::Error,
            }),
            ast::TypeKind::Infer | ast::TypeKind::Error => Spec::Plain(Ty::Error),
        }
    }

    pub(in crate::lower) fn named(
        &mut self,
        name: &ast::Name,
        args: &[ast::Type],
        values: bool,
    ) -> Spec {
        if let ([elem], true, "List") = (args, values, name.text.as_str()) {
            return Spec::List(Box::new(self.resolve(elem, true)));
        }
        let arg = |lw: &mut Self, i: usize| match args.get(i) {
            Some(ty) => lw.resolve_ty(ty),
            None => {
                lw.report_said(name.span, typo::missing_type_args(&name.text, i));
                Ty::Error
            }
        };
        // Check `Self` and `gen_env` first.
        if name.text == "Self" {
            if let Some(sid) = self.self_struct {
                return if values {
                    self.struct_spec(sid)
                } else {
                    Spec::Plain(Ty::Struct(sid))
                };
            }
        }
        if args.is_empty() {
            if let Some(ga) = self.gen_env.get(&name.text).cloned() {
                // `read<T>()` in a generic fn reads the type `T` stands for.
                if let (true, GenArg::Ty(ty)) = (values, &ga) {
                    if let Some(spec) = self.spec_of(ty) {
                        return spec;
                    }
                }
                if let (true, GenArg::Range(lo, hi)) = (values, &ga) {
                    let bounds = RangeInfo::Scalar(self.const_bounds(*lo, *hi, name.span));
                    if let Some(spec) = bounds.spec() {
                        return spec;
                    }
                }
                if let GenArg::Sized(ty, lens) = &ga {
                    if let (true, Ty::Array { dims, elem }) = (values, ty) {
                        if let Some(elem) = self.spec_of(elem) {
                            let dims = dims
                                .iter()
                                .zip(lens)
                                .map(|(d, &n)| {
                                    let len = Box::new(self.int_lit(n, name.span));
                                    match *d {
                                        Dim::IntFrom(first) => DimSpec::IntFrom(first, len),
                                        _ => DimSpec::Int(len),
                                    }
                                })
                                .collect();
                            return Spec::Array {
                                dims,
                                elem: Box::new(elem),
                            };
                        }
                    }
                }
                return Spec::Plain(match ga {
                    GenArg::Ty(t) | GenArg::Sized(t, _) => t,
                    GenArg::Domain(d) => Ty::Pos(d),
                    GenArg::Int(_) | GenArg::RuntimeInt(_) | GenArg::Range(_, _) => Ty::Int,
                    GenArg::OptRange(_, _) => Ty::opt(Ty::Int),
                    GenArg::Op(op) => Ty::Struct(self.op_type(op)),
                });
            }
            let local = self
                .cx
                .local_type_aliases
                .get(&name.text)
                .cloned()
                .or_else(|| {
                    self.outer_cxs
                        .iter()
                        .rev()
                        .find_map(|ocx| ocx.local_type_aliases.get(&name.text).cloned())
                });
            // A local `type Z = Mod(p)` reads by its struct's input format, as a top-level one does.
            match local {
                Some(Ty::Struct(sid)) if values => return self.struct_spec(sid),
                Some(ty) => return Spec::Plain(ty),
                None => {}
            }
        }
        // Check qualified module type `mod_alias.Type` or `D.cuts` or `Rooted.Pre`.
        if let Some((prefix, rest)) = name.text.split_once('.') {
            if rest == "cuts" {
                if let Some(GenArg::Domain(d)) = self.gen_env.get(prefix).cloned() {
                    return Spec::Plain(Ty::Bound(d));
                }
                if let Some(Sym::Domain(d)) = self.get_sym(prefix) {
                    return Spec::Plain(Ty::Bound(d));
                }
            }
            if rest == "ranges" {
                if let Some(GenArg::Domain(d)) = self.gen_env.get(prefix).cloned() {
                    return Spec::Plain(Ty::range(Ty::Pos(d)));
                }
                if let Some(Sym::Domain(d)) = self.get_sym(prefix) {
                    return Spec::Plain(Ty::range(Ty::Pos(d)));
                }
            }
            if let Some(of) = cpl_check::domain_member(rest) {
                if let Some(d) = self.resolve_domain_name(prefix) {
                    return Spec::Plain(self.member_domain_ty(of, d, name.span));
                }
            }
            let projected = match self.gen_env.get(prefix).cloned() {
                Some(GenArg::Ty(t)) => Some(t),
                Some(GenArg::Domain(d)) => Some(Ty::Pos(d)),
                _ => None,
            };
            if let Some(ty) = projected.and_then(|t| self.assoc_type(&t, rest)) {
                return Spec::Plain(ty);
            }
            if let Some(Sym::Mod(target_mod)) = self.get_sym(prefix) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let inner_name = ast::Name {
                    text: rest.to_string(),
                    span: name.span,
                };
                let res = self.named(&inner_name, args, values);
                self.cur_mod = saved_mod;
                return res;
            }
            if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                let gargs = self.resolve_gen_args_for_struct(gid, args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
            if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
            // `xs.D`: the domain a value such as `compress(all)` brought.
            if let Some(dom_id) = self.domain_of_value(prefix, rest) {
                return Spec::Plain(Ty::Pos(dom_id));
            }
        }

        // `Slice<S>` and `Rev<S>` are views of `S`; a `Rev` binding walks backwards.
        if matches!(name.text.as_str(), "Slice" | "Rev") && args.len() == 1 {
            match self.resolve_ty(&args[0]) {
                Ty::List(e) => return Spec::Plain(Ty::ListView(e, Dim::Int)),
                Ty::ListView(e, key) | Ty::Keyed(e, key) | Ty::MutView(e, key) => {
                    return Spec::Plain(Ty::ListView(e, key))
                }
                Ty::Str | Ty::StrView => return Spec::Plain(Ty::StrView),
                _ => {}
            }
        }
        let ty = match name.text.as_str() {
            // The solution an `@interactor` talks to holds nothing.
            "Interactor" if self.judge => Ty::Unit,
            "Int" | "I64" => Ty::Int,
            "F64" => Ty::Float,
            "Bool" => Ty::Bool,
            "Char" => Ty::Char,
            "Str" => Ty::Str,
            "List" | "Iter" => Ty::list(arg(self, 0)),
            "Slice" | "Rev" => match arg(self, 0) {
                Ty::List(e) => Ty::ListView(e, Dim::Int),
                Ty::ListView(e, d) | Ty::MutView(e, d) => Ty::ListView(e, d),
                Ty::Str | Ty::StrView => Ty::StrView,
                Ty::Array { elem, dims } if dims.len() == 1 => Ty::ListView(elem, dims[0]),
                Ty::Array { elem, .. } => Ty::List(elem),
                other => other,
            },
            "Opt" => Ty::opt(arg(self, 0)),
            "Set" => Ty::Set(Box::new(arg(self, 0))),
            "HashSet" => Ty::HashSet(Box::new(arg(self, 0))),
            "Map" => {
                let key = arg(self, 0);
                Ty::Map(Box::new(key), Box::new(arg(self, 1)))
            }
            "HashMap" => {
                let key = arg(self, 0);
                Ty::HashMap(Box::new(key), Box::new(arg(self, 1)))
            }
            "Range" => Ty::range(arg(self, 0)),
            "Cut" if !args.is_empty() => match arg(self, 0) {
                Ty::Pos(d) => Ty::Bound(d),
                other => other,
            },
            "I128" => Ty::I128,
            "U64" => Ty::U64,
            _ => match self.get_sym(&self.arity_name(&name.text, args.len())) {
                Some(Sym::Struct(id)) if values => return self.struct_spec(id),
                Some(Sym::Struct(id)) => Ty::Struct(id),
                Some(Sym::GenStruct(gid)) => {
                    let gargs = self.resolve_gen_args_for_struct(gid, args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if values {
                        return self.struct_spec(sid);
                    }
                    Ty::Struct(sid)
                }
                Some(Sym::Enum(id)) => Ty::Enum(id),
                Some(Sym::GenEnum(gid)) => {
                    let gargs = self.resolve_gen_args(args);
                    Ty::Enum(self.instantiate_enum(gid, gargs))
                }
                Some(Sym::Domain(id)) => Ty::Pos(id),
                Some(Sym::Alias(id)) => return self.in_alias(id, |lw, ty| lw.resolve(ty, values)),
                Some(Sym::GenAlias(gaid)) => {
                    return self.gen_alias_spec(gaid, args, name.span, values)
                }
                _ => {
                    self.report(
                        name.span,
                        msgs::ir::UNKNOWN_TYPE_IS_NOT_A_TYPE,
                        &[&name.text],
                    );
                    Ty::Error
                }
            },
        };
        Spec::Plain(ty)
    }

    /// A list built by walking the ordinals of `domain`: `first` goes in
    /// before the walk, and `each` gives what an ordinal adds, if anything.
    pub(in crate::lower) fn listed(
        &mut self,
        domain: DomainId,
        elem: Ty,
        first: Option<Expr>,
        span: Span,
        each: impl FnOnce(&mut Self, Expr) -> (Option<Expr>, Expr),
    ) -> Expr {
        let list_ty = Ty::list(elem);
        let acc = self.temp(list_ty.clone());
        let at = self.temp(Ty::Int);
        let id = self.fresh_loop();
        let push = |lw: &mut Self, value: Expr| {
            Stmt::Expr(lw.mk_mutate(
                Mutation::Push,
                Place {
                    kind: PlaceKind::Local(acc),
                    ty: list_ty.clone(),
                    span,
                },
                vec![value],
                Ty::Unit,
                span,
            ))
        };
        let empty = self.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
        let mut stmts = vec![Stmt::Let(acc, Some(empty))];
        if let Some(first) = first {
            stmts.push(push(self, first));
        }
        let ord = self.local(at, span);
        let (keep, value) = each(self, ord);
        let add = push(self, value);
        let body = match keep {
            Some(keep) => Stmt::If(keep, Block { stmts: vec![add] }, Block::default()),
            None => add,
        };
        let lo = self.int_lit(0, span);
        let hi = self.mk(ExprKind::DomainLen(domain), Ty::Int, span);
        stmts.push(Stmt::For(
            id,
            at,
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            },
            Block { stmts: vec![body] },
        ));
        stmts.push_give(self.local(acc, span));
        self.mk(ExprKind::Do(Block { stmts }), list_ty, span)
    }

    /// True when `id` is declared in another module and has a field
    /// without `pub`, which code here can't name (R11.1.4). A generic fn
    /// reads it where a fn of its module asked for the instance.
    pub(in crate::lower) fn private_fields(&self, id: StructId) -> bool {
        self.structs[id].mod_id != self.cur_mod
            && !self.lowered_for(self.structs[id].mod_id)
            && self.structs[id]
                .decl
                .is_some_and(|decl| decl.fields.iter().any(|f| !f.public))
    }

    /// A struct's fields as they are read from input: with their ranges.
    pub(in crate::lower) fn struct_spec(&mut self, id: StructId) -> Spec {
        if let Some(fn_id) = self.own_method(id, "read") {
            if self.sigs[fn_id].params.is_empty() {
                let call = self.method_call(fn_id, None, &[], Span::default());
                return Spec::Call(Box::new(call));
            }
        }
        if self.structs[id].def.fields.len() == 1 {
            if let Some(fn_id) = self.own_method(id, "new") {
                // A defaulted parameter (`new(cap: Int = 0)`) is a setting,
                // not the value the struct is built from.
                let sig = &self.sigs[fn_id];
                if sig.params.len() == 1 && sig.decl.params.iter().all(|p| p.default.is_none()) {
                    let param_ty = self.sigs[fn_id].params[0].ty.clone();
                    let arg_spec = self.spec_of(&param_ty).map(|spec| self.readable(spec));
                    if let Some(arg_spec) =
                        arg_spec.filter(|spec| read_spec::unreadable(spec).is_none())
                    {
                        let read_arg = self.mk_read(arg_spec, param_ty, Span::default());
                        let call = self.mk(
                            ExprKind::Call(fn_id, vec![Arg::Value(read_arg)]),
                            Ty::Struct(id),
                            Span::default(),
                        );
                        return Spec::Call(Box::new(call));
                    }
                }
            }
        }
        let Some(decl) = self.structs[id].decl else {
            return Spec::Fields(Ty::Struct(id), Vec::new());
        };
        // Outside its module a struct reads field by field only when every
        // field is `pub`: `read()` builds nothing user code couldn't (R11.1.4).
        if self.private_fields(id) {
            return Spec::Plain(Ty::Struct(id));
        }
        let (m, offset, env) = (
            self.structs[id].mod_id,
            self.structs[id].offset,
            self.structs[id].env.clone(),
        );
        let saved_mod = std::mem::replace(&mut self.cur_mod, m);
        let saved_offset = std::mem::replace(&mut self.offset, offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let fields = decl
            .fields
            .iter()
            .map(|f| self.resolve(&f.ty, true))
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        Spec::Fields(Ty::Struct(id), fields)
    }

    /// A product domain's grid as array dimensions, one position axis per factor.
    fn product_grid_dims(&self, id: DomainId) -> Option<Vec<DimSpec>> {
        let axes = super::comprehension::pos_axes(&self.product_grid(id)?)?;
        Some(axes.into_iter().map(DimSpec::Pos).collect())
    }

    /// One dimension of an array type: a domain, its bounds, a custom `Index` struct, or a count.
    /// The dimensions of an index type that stands for a tuple or record of
    /// domains, `(R, C)` or `(r: R, c: C)`, named directly, by an alias or by
    /// a generic: one for each field, so `[P]T` is laid out as `[R, C]T`.
    pub(in crate::lower) fn product_dims(&mut self, d: &ast::Type) -> Option<Vec<DimSpec>> {
        let ty = match &d.kind {
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                match self.gen_env.get(&name.text) {
                    Some(GenArg::Ty(ty)) => ty.clone(),
                    // `[V]T` with `V` a product domain is its grid.
                    Some(GenArg::Domain(id)) => {
                        return self.product_grid_dims(*id);
                    }
                    Some(_) => return None,
                    None if matches!(self.get_sym(&name.text), Some(Sym::Alias(_)))
                        || self.cx.local_type_aliases.contains_key(&name.text) =>
                    {
                        self.resolve_ty(d)
                    }
                    // `domain P = (A, B)` names its grid too.
                    None => {
                        return match self.get_sym(&name.text) {
                            Some(Sym::Domain(id)) => self.product_grid_dims(id),
                            _ => None,
                        }
                    }
                }
            }
            ast::TypeKind::Tuple(items) if items.len() >= 2 => self.resolve_ty(d),
            ast::TypeKind::Record(fields) if fields.len() >= 2 => self.resolve_ty(d),
            _ => return None,
        };
        let items: Vec<Ty> = match ty {
            Ty::Tuple(items) => items,
            Ty::Record(fields) => fields.into_iter().map(|(_, ty)| ty).collect(),
            _ => return None,
        };
        items
            .iter()
            .map(|t| match t {
                Ty::Pos(id) => Some(DimSpec::Pos(*id)),
                _ => None,
            })
            .collect()
    }

    pub(in crate::lower) fn dim(&mut self, ty: &ast::Type, values: bool) -> Option<DimSpec> {
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            if args.is_empty() {
                if let Some(GenArg::Domain(id)) = self.gen_env.get(&name.text).cloned() {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(&name.text).cloned() {
                    return Some(DimSpec::Bound(id));
                }
                if let Some(Sym::Domain(id)) = self.get_sym(&name.text) {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(sid) = self.index_struct(&name.text) {
                    if let Some(dom_id) = self.structs[sid].index_domain {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(Sym::Enum(eid)) = self.get_sym(&name.text) {
                    if self.enums[eid].variants.iter().all(|(_, f)| f.is_empty()) {
                        return Some(DimSpec::Enum(eid));
                    }
                }
                // `type Val = xs.D`, then `[Val]Int`.
                if matches!(self.get_sym(&name.text), Some(Sym::Alias(_)))
                    || self.cx.local_type_aliases.contains_key(&name.text)
                {
                    match self.resolve_ty(ty) {
                        Ty::Pos(id) => return Some(DimSpec::Pos(id)),
                        Ty::Bound(id) => return Some(DimSpec::Bound(id)),
                        _ => {}
                    }
                }
                // `[xs.D]`: the domain a value such as `compress(all)` brought.
                if let Some((owner, member)) = name.text.split_once('.') {
                    if let Some(id) = self.domain_of_value(owner, member) {
                        return Some(DimSpec::Pos(id));
                    }
                }
                if let Some(domain) = name.text.strip_suffix(".cuts") {
                    if let Some(GenArg::Domain(id)) = self.gen_env.get(domain).cloned() {
                        return Some(DimSpec::Bound(id));
                    }
                    if let Some(Sym::Domain(id)) = self.get_sym(domain) {
                        return Some(DimSpec::Bound(id));
                    }
                }
                // `[V.ranges]T`: a non-empty range `l..r` is the pair `(l, r - 1)`, `l <= r - 1`.
                if let Some(domain) = name.text.strip_suffix(".ranges") {
                    if let Some(GenArg::Domain(id)) = self.gen_env.get(domain).cloned() {
                        return Some(DimSpec::Sym(id));
                    }
                    if let Some(Sym::Domain(id)) = self.get_sym(domain) {
                        return Some(DimSpec::Sym(id));
                    }
                }
                if let Some(dim) = self.subsets_dim(&name.text, name.span, values) {
                    return dim;
                }
            } else if name.text == "Cut" && args.len() == 1 {
                if let Ty::Pos(id) = self.resolve_ty(&args[0]) {
                    return Some(DimSpec::Bound(id));
                }
            } else if name.text == "Sym" && args.len() == 1 {
                if let Ty::Pos(id) = self.resolve_ty(&args[0]) {
                    return Some(DimSpec::Sym(id));
                }
            } else if let Some(Sym::GenStruct(gid)) = self.get_sym(&name.text) {
                let gargs = self.resolve_gen_args_for_struct(gid, args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(dom_id) = self.structs[sid].index_domain {
                    return Some(DimSpec::Pos(dom_id));
                }
            } else if let Some((prefix, rest)) = name.text.split_once('.') {
                if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                    let gargs = self.resolve_gen_args_for_struct(gid, args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                // `[xs.D]`: the domain a value such as `compress(all)` brought.
                if let Some(dom_id) = self.domain_of_value(prefix, rest) {
                    return Some(DimSpec::Pos(dom_id));
                }
            }
        }
        // Also handle `[xs.D]T` parsed as `TypeKind::Value(Field { base: Name(prefix), name: rest })`.
        if let ast::TypeKind::Value(e) = &ty.kind {
            if let ast::ExprKind::Field { base, name } = &e.kind {
                if let ast::ExprKind::Name(prefix) = &base.kind {
                    if let Some(dom_id) = self.domain_of_value(prefix, &name.text) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                    if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                        if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                            return Some(DimSpec::Pos(dom_id));
                        }
                    }
                }
            }
        }
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            let arg = self.gen_env.get(&name.text).cloned();
            let tuple = matches!(arg, Some(GenArg::Ty(Ty::Tuple(_))));
            if let (true, Some(arg @ (GenArg::Ty(_) | GenArg::Range(..)))) = (
                args.is_empty() && (values || tuple || self.range_generic(&name.text)),
                arg,
            ) {
                return self.generic_dim(&name.text, &arg, name.span);
            }
        }
        // Anything else is a length: `[n]Int`, `[2 * n]Int`, `[0..n]Int`, `[V.cuts]Int`.
        let text = self.source(ty.span);
        if let Some(domain) = text.strip_suffix(".cuts") {
            let dname = domain.trim();
            if let Some(GenArg::Domain(id)) = self.gen_env.get(dname).cloned() {
                return Some(DimSpec::Bound(id));
            }
            if let Some(Sym::Domain(id)) = self.get_sym(dname) {
                return Some(DimSpec::Bound(id));
            }
        }
        match &ty.kind {
            ast::TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => {
                let first = match &lo.kind {
                    ast::ExprKind::Int(first) => i64::try_from(*first).ok(),
                    ast::ExprKind::Unary {
                        op: ast::UnOp::Neg,
                        expr,
                    } => match &expr.kind {
                        ast::ExprKind::Int(first) => i64::try_from(*first).ok().map(|first| -first),
                        _ => None,
                    },
                    _ => None,
                };
                let Some(first) = first else {
                    self.unsupported(
                        lo.span,
                        "an array dimension that starts at anything but a number",
                    );
                    self.help("name the range: `domain D = lo..=hi`, then `[D]T`");
                    return None;
                };
                if first != 0 {
                    if !values {
                        return Some(DimSpec::IntFrom(first, Box::new(self.int_lit(0, ty.span))));
                    }
                    let mut len = self.expr_as(hi, &Ty::Int);
                    len = self.int_add(len, if *inclusive { 1 - first } else { -first });
                    return Some(DimSpec::IntFrom(first, Box::new(len)));
                }
                if !*inclusive {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let probed = self.expr(hi, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let Some(dim) = self.probed_dim(&probed, hi, values) {
                        return Some(dim);
                    }
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let mut len = self.expr_as(hi, &Ty::Int);
                if *inclusive {
                    len = self.int_add(len, 1);
                }
                Some(DimSpec::Int(Box::new(len)))
            }
            // `[Bool]T` has two cells, `false` then `true`.
            ast::TypeKind::Named { name, args }
                if args.is_empty() && name.text == "Bool" && self.get_sym(&name.text).is_none() =>
            {
                Some(DimSpec::Int(Box::new(self.int_lit(2, ty.span))))
            }
            // `[Sign]T` for a fieldless enum `Sign`: one cell per variant.
            ast::TypeKind::Named { name, args }
                if args.is_empty()
                    && let Some(Sym::Enum(eid)) = self.get_sym(&name.text)
                    && !self.enums[eid].variants.is_empty()
                    && self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) =>
            {
                let n = self.enums[eid].variants.len() as i64;
                Some(DimSpec::Int(Box::new(self.int_lit(n, ty.span))))
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(Sym::Alias(aid)) = self.get_sym(&name.text) {
                    let (m, aliased_ty, offset) = self.aliases[aid];
                    let saved_mod = std::mem::replace(&mut self.cur_mod, m);
                    let saved_offset = std::mem::replace(&mut self.offset, offset);
                    let d = self.dim(aliased_ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    return d;
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let expr = ast::Expr {
                    kind: ast::ExprKind::Name(name.text.clone()),
                    span: name.span,
                };
                Some(DimSpec::Int(Box::new(self.expr_as(&expr, &Ty::Int))))
            }
            ast::TypeKind::Value(expr) => {
                if !values {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let v = self.expr(expr, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let ExprKind::DomainVal(dom_id) = v.kind {
                        return Some(DimSpec::Pos(dom_id));
                    }
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let v = self.expr(expr, None);
                if let ExprKind::DomainVal(dom_id) = v.kind {
                    return Some(DimSpec::Pos(dom_id));
                }
                Some(DimSpec::Int(Box::new(self.coerce(v, &Ty::Int, expr.span))))
            }
            _ => {
                self.report(ty.span, msgs::ir::UNKNOWN_TYPE_AN_ARRAY_IS_INDEXED, &[]);
                None
            }
        }
    }
}
