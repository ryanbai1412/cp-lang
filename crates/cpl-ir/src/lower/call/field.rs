//! Fields and members, and the items/pairs/projection heads of a walk.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn field(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        if let Some(alias) = self.subsets_alias(base) {
            return self.field(whole, &alias, name, hint);
        }
        let then = matches!(name.text.as_str(), "len" | "is_empty").then_some(name);
        if let Some(e) = then.and_then(|n| self.map_view_piece(base, Some(n))) {
            return e;
        }
        if let Some(piece) = self.map_view_piece(whole, None) {
            return piece;
        }
        if matches!(name.text.as_str(), "first" | "last" | "len" | "is_empty")
            && self.is_sorted_range_expr(base)
        {
            return self.sorted_view_field(base, &name.text, span);
        }
        if let ast::ExprKind::Field { name: bounds, .. } = &base.kind {
            // `V.cuts` is a domain, and a domain's `range` is all of it.
            if matches!(bounds.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "range" | "all")
            {
                let cuts = self.expr(base, None);
                if matches!(cuts.kind, ExprKind::Range { .. }) {
                    return cuts;
                }
            }
            if matches!(bounds.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "count" | "len" | "size")
            {
                let cuts = self.expr(base, None);
                if let (ExprKind::Range { lo, hi }, Ty::Range(_)) = (&cuts.kind, &cuts.ty) {
                    return self.mk(
                        ExprKind::Binary(BinOp::Sub, hi.clone(), lo.clone()),
                        Ty::Int,
                        span,
                    );
                }
            }
            if matches!(bounds.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "first" | "last")
            {
                let cuts = self.expr(base, None);
                if let (ExprKind::Range { lo, hi }, Ty::Range(cut)) = (&cuts.kind, &cuts.ty) {
                    let end = if name.text == "first" {
                        (**lo).clone()
                    } else {
                        self.int_add((**hi).clone(), -1)
                    };
                    let end = self.cast(Cast::Trust, end, (**cut).clone());
                    return self.spec_opt(None, end, span);
                }
            }
        }
        // Handle `GenericBase<...>.member` (e.g., `Mod<7>.zero`, `Sum<Int>.id`, `Less<Int>.rev`).
        if let ast::ExprKind::Generic {
            base: gbase,
            args: gtypes,
        } = &base.kind
        {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                let lookup = self.arity_name(gname, gtypes.len());
                if let Some(Sym::GenStruct(gid)) = self.get_sym(&lookup) {
                    let gargs = self.resolve_gen_args_for_struct(gid, gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    // A method taking `self` is the value's: `Subset<B>.first` is the domain's.
                    if let Some(fn_id) = self.own_method(sid, &name.text).filter(|&f| {
                        self.sigs[f].receiver.is_none() || self.structs[sid].index_domain.is_none()
                    }) {
                        return self.method_call(fn_id, None, &[], span);
                    }
                    if let Some(number) = self.struct_number_param(sid, &name.text, span) {
                        return number;
                    }
                    if name.text == "count" {
                        if let Some(d) = self.structs[sid].index_domain {
                            return self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        }
                    }
                    if self.structs[sid].index_domain.is_some() {
                        return self.library_domain_seq(base, sid, name, None, span);
                    }
                }
            }
        }
        // Local type alias pointing to a domain (e.g. `type Group = Subset<Rabbit>` inside a block).
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                if let Some(&Ty::Pos(id)) = self.cx.local_type_aliases.get(n) {
                    return self.domain_field(id, name, span);
                }
            }
        }
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none()
                && (n == "Char" || matches!(self.gen_env.get(n), Some(GenArg::Ty(Ty::Char))))
                && matches!(name.text.as_str(), "len" | "size" | "count")
            {
                return self.int_lit(128, span);
            }
        }
        // `Char` and `Bool` are bounded and ordered, so they have ends as `MIN` and `MAX` (R02.2.1).
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                let ty = match self.gen_env.get(n) {
                    Some(GenArg::Ty(t @ (Ty::Char | Ty::Bool))) => Some(t.clone()),
                    _ if n == "Char" => Some(Ty::Char),
                    _ if n == "Bool" => Some(Ty::Bool),
                    _ => None,
                };
                let max = match name.text.as_str() {
                    "MIN" => Some(false),
                    "MAX" => Some(true),
                    _ => None,
                };
                if let (Some(ty), Some(max)) = (ty, max) {
                    let kind = match ty {
                        Ty::Char => ExprKind::Char(if max { 127 } else { 0 }),
                        _ => ExprKind::Bool(max),
                    };
                    return self.mk(kind, ty, span);
                }
            }
        }
        // Generic parameter bound to a `.cuts` domain (`D = V.cuts`):
        // `D.K` / `D.len` / `D.count` is `V.K + 1`, `D.first` is `0`, `D.last` is `V.K`.
        if let ast::ExprKind::Name(n) = &base.kind {
            if self.lookup(n).is_none() {
                if let Some(GenArg::Ty(Ty::Bound(d))) = self.gen_env.get(n).cloned() {
                    match name.text.as_str() {
                        "len" | "size" | "count" => {
                            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                            return self.int_add(len, 1);
                        }
                        "first" => {
                            let zero = self.int_lit(0, span);
                            return self.cast(Cast::Trust, zero, Ty::Bound(d));
                        }
                        "last" => {
                            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                            return self.cast(Cast::Trust, len, Ty::Bound(d));
                        }
                        _ => {}
                    }
                }
            }
        }
        match self.sym(base) {
            Some(Sym::Mod(target_mod)) => {
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                match sym {
                    Some(Sym::ModConst(mcid)) => return self.ensure_mod_const(mcid, span),
                    Some(Sym::Global(id)) => {
                        return self.mk(ExprKind::Global(id), self.globals[id].ty.clone(), span);
                    }
                    Some(Sym::Fn(id)) => return self.fn_as_closure(id, hint, span),
                    _ => {
                        self.report(
                            name.span,
                            msgs::ir::UNKNOWN_FIELD_MODULE_HAS_NO_MEMBER,
                            &[&name.text],
                        );
                        return self.err_expr(span);
                    }
                }
            }
            Some(Sym::Struct(id)) => {
                if name.text == "count" {
                    if let Some(d) = self.structs[id].index_domain {
                        return self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    }
                }
                if let Some(sc) = self.structs[id].consts.get(&name.text).cloned() {
                    return self.eval_struct_const(id, sc, span);
                }
                // A method taking `self` is the value's: `Subset<B>.first` is the domain's.
                if let Some(fn_id) = self.own_method(id, &name.text).filter(|&f| {
                    self.sigs[f].receiver.is_none() || self.structs[id].index_domain.is_none()
                }) {
                    if let Some(value) = self.static_fn_value(fn_id, hint, span) {
                        return value;
                    }
                    return self.method_call(fn_id, None, &[], span);
                }
                if let Some(number) = self.struct_number_param(id, &name.text, span) {
                    return number;
                }
                if self.structs[id].index_domain.is_some() {
                    return self.library_domain_seq(base, id, name, None, span);
                }
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                match resolved {
                    Ty::Struct(id) => {
                        if let Some(sc) = self.structs[id].consts.get(&name.text).cloned() {
                            return self.eval_struct_const(id, sc, span);
                        }
                        // A method taking `self` is the value's: `Subset<B>.first` is the domain's.
                        if let Some(fn_id) = self.own_method(id, &name.text).filter(|&f| {
                            self.sigs[f].receiver.is_none()
                                || self.structs[id].index_domain.is_none()
                        }) {
                            if let Some(value) = self.static_fn_value(fn_id, hint, span) {
                                return value;
                            }
                            return self.method_call(fn_id, None, &[], span);
                        }
                        if let Some(number) = self.struct_number_param(id, &name.text, span) {
                            return number;
                        }
                    }
                    Ty::Pos(id) => return self.domain_field(id, name, span),
                    _ => {}
                }
            }
            Some(Sym::GenStruct(gid)) => {
                if let Some(Ty::Struct(sid)) = hint {
                    if self.structs[*sid]
                        .origin_gen
                        .as_ref()
                        .is_some_and(|(g, _)| *g == gid)
                    {
                        if let Some(sc) = self.structs[*sid].consts.get(&name.text).cloned() {
                            return self.eval_struct_const(*sid, sc, span);
                        }
                        if self.structs[*sid].getters.contains(&name.text) {
                            if let Some(fid) = self.own_method(*sid, &name.text) {
                                self.queue_fn(fid);
                                let ret = self.ret_of(fid, span);
                                return self.mk(ExprKind::Call(fid, Vec::new()), ret, span);
                            }
                        }
                    }
                }
            }
            Some(Sym::Domain(id)) => return self.domain_field(id, name, span),
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) if self.enums[id].variants[tag].1.is_empty() => {
                        self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span)
                    }
                    Some(_) => {
                        self.report(
                            span,
                            msgs::ir::WRONG_ARG_COUNT_THE_VARIANT_HOLDS_VALUES,
                            &[&name.text, &name.text],
                        );
                        self.err_expr(span)
                    }
                    None if self.unit_enum(id) => self.enum_seq(base, id, name, None, span),
                    None => {
                        let message = cpl_diag::msgs::ir::HAS_NO_VARIANT
                            .at(&[&self.enums[id].name, &name.text]);
                        self.report_said(name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            _ => {}
        }
        if let ast::ExprKind::Name(type_name) = &base.kind {
            if self.lookup(type_name).is_none() {
                let ends = self
                    .gen_env
                    .get(&mono::range_ends_marker(type_name))
                    .cloned();
                if ends.is_none()
                    && (type_name == "Int"
                        || matches!(self.gen_env.get(type_name), Some(GenArg::Ty(Ty::Int))))
                {
                    match name.text.as_str() {
                        "MAX" => return self.mk(ExprKind::Int(i64::MAX), Ty::Int, span),
                        "MIN" => return self.mk(ExprKind::Int(INT_MIN), Ty::Int, span),
                        "zero" => return self.mk(ExprKind::Int(0), Ty::Int, span),
                        "one" => return self.mk(ExprKind::Int(1), Ty::Int, span),
                        _ => {}
                    }
                }
                // `T.zero` and `T.one` where `T` is one of the wide integers.
                let wide = match (type_name.as_str(), self.gen_env.get(type_name)) {
                    ("U64", _) | (_, Some(GenArg::Ty(Ty::U64))) => Some((Cast::IntToU64, Ty::U64)),
                    ("I128", _) | (_, Some(GenArg::Ty(Ty::I128))) => {
                        Some((Cast::IntToI128, Ty::I128))
                    }
                    _ => None,
                };
                if let (Some((cast, ty)), Some(unit)) =
                    (wide, ["zero", "one"].iter().position(|c| *c == name.text))
                {
                    let unit = self.mk(ExprKind::Int(unit as i64), Ty::Int, span);
                    return self.mk(ExprKind::Cast(cast, Box::new(unit)), ty, span);
                }
                if let Some(GenArg::Ty(Ty::Struct(sid))) = self.gen_env.get(type_name).cloned() {
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    // A method taking `self` is the value's: `Subset<B>.first` is the domain's.
                    if let Some(fn_id) = self.own_method(sid, &name.text).filter(|&f| {
                        self.sigs[f].receiver.is_none() || self.structs[sid].index_domain.is_none()
                    }) {
                        return self.method_call(fn_id, None, &[], span);
                    }
                }
                // A range type, or a `T` bound to one, has its ends as `MIN` and `MAX` (R02.6.2).
                let ends = ends.or_else(|| self.named_gen_arg(type_name));
                if let Some(GenArg::Range(lo, hi)) = ends {
                    let v = match name.text.as_str() {
                        "MIN" => Some(lo.unwrap_or(INT_MIN)),
                        "MAX" => Some(hi.unwrap_or(i64::MAX)),
                        "count" => lo.zip(hi).map(|(lo, hi)| (hi - lo + 1).max(0)),
                        "zero" => Some(0),
                        "one" => Some(1),
                        _ => None,
                    };
                    if let Some(v) = v {
                        return self.mk(ExprKind::Int(v), Ty::Int, span);
                    }
                }
                if let Some(GenArg::Ty(Ty::Opt(inner))) = self.gen_env.get(type_name).cloned() {
                    if matches!(name.text.as_str(), "MAX" | "MIN") {
                        return self.mk(ExprKind::None, Ty::Opt(inner), span);
                    }
                }
                if type_name == "F64"
                    || matches!(self.gen_env.get(type_name), Some(GenArg::Ty(Ty::Float)))
                {
                    match name.text.as_str() {
                        "zero" => return self.mk(ExprKind::Float(0.0), Ty::Float, span),
                        "one" => return self.mk(ExprKind::Float(1.0), Ty::Float, span),
                        "MAX" => return self.mk(ExprKind::Float(f64::MAX), Ty::Float, span),
                        "MIN" => return self.mk(ExprKind::Float(f64::MIN), Ty::Float, span),
                        _ => {}
                    }
                }
                // `U.zero` in a generic body whose `U` is a `Str`: the type
                // it was given lacks the constant, so the call can't use it.
                if let Some(GenArg::Ty(ty)) = self.gen_env.get(type_name).cloned() {
                    if matches!(name.text.as_str(), "zero" | "one" | "MAX" | "MIN")
                        && !matches!(ty, Ty::Error)
                    {
                        self.report(
                            span,
                            msgs::ir::MISSING_IMPL_HAS_NO,
                            &[&self.a_ty(&ty), &name.text],
                        );
                        return self.err_expr(span);
                    }
                }
            }
        }
        if let Some(kept) = self.filtered_items(base, name, span) {
            return kept;
        }
        let recv_mark = self.cx.pending.len();
        let value = self.expr(base, None);
        if let Some((index, ty)) = self.field_of(&value.ty, &name.text) {
            return self.mk_field(value, index, ty, span);
        }
        let ty = value.ty.clone();
        let builtin = |lw: &mut Self, op: Builtin, args: Vec<Expr>, ty: Ty| {
            lw.mk(ExprKind::Builtin(op, args), ty, span)
        };
        // A domain reached through a value (`vt.K`) has a named domain's members.
        if let ExprKind::DomainVal(id) = value.kind {
            if !matches!(name.text.as_str(), "len" | "size" | "count") {
                return self.domain_field(id, name, span);
            }
        }
        match (&ty, name.text.as_str()) {
            (Ty::Error, _) => value,
            (Ty::Struct(id), method) if self.structs[*id].methods.contains_key(method) => {
                let fn_id = self.structs[*id].methods[method];
                if !self.ensure_sig(fn_id, Some(span)) {
                    return self.err_expr(span);
                }
                if self.structs[*id].getters.contains(method) || self.sigs[fn_id].params.is_empty()
                {
                    self.method_call(fn_id, Some((base, value)), &[], span)
                } else {
                    self.bind_method_closure(fn_id, value, span)
                }
            }
            (Ty::Pos(_) | Ty::Bound(_), "ord" | "idx") => self.cast(Cast::Ord, value, Ty::Int),
            // A cell of a product is that product's position (R04.2.11), so it has its `ord`.
            (Ty::Tuple(ts), "ord" | "idx")
                if ts.len() > 1 && ts.iter().all(|t| matches!(t, Ty::Pos(_))) =>
            {
                let factors = ts
                    .iter()
                    .filter_map(|t| self.ty_to_product_factor(t).map(|f| (None, f)))
                    .collect();
                let d = self.intern_anon_product_domain(factors);
                let mut defs = Vec::new();
                self.def_anon_product_domain(d, span, &mut defs);
                self.cx.pending.extend(defs);
                let pos = self.coerce(value, &Ty::Pos(d), span);
                self.cast(Cast::Ord, pos, Ty::Int)
            }
            (Ty::Int, "ord" | "idx" | "r" | "right") => value,
            (Ty::Int, "l" | "left") => self.int_add(value, -1),
            (Ty::Pos(d), "label") => self.cast(Cast::Label(*d), value, Ty::Int),
            (Ty::Pos(_), "rank") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                self.int_add(ord, 1)
            }
            (Ty::Enum(e), "ord" | "idx" | "label") if self.unit_enum(*e) => {
                self.cast(Cast::EnumOrd(*e), value, Ty::Int)
            }
            // The cuts on either side of a fieldless enum's variant, by number.
            (Ty::Enum(e), "l" | "r") if self.unit_enum(*e) => {
                let ord = self.cast(Cast::EnumOrd(*e), value, Ty::Int);
                self.int_add(ord, i64::from(name.text == "r"))
            }
            (Ty::Enum(e), "next" | "prev") if self.unit_enum(*e) => {
                self.enum_step(*e, value, name.text == "next", span)
            }
            (Ty::Pos(d), "next") => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![value, len],
                    Ty::opt(ty.clone()),
                )
            }
            (Ty::Pos(_), "prev") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(ty.clone()))
            }
            (Ty::Pos(d), "before" | "after") => {
                let d = *d;
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let (lo, hi) = if name.text == "after" {
                    (
                        self.int_add(ord, 1),
                        self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                    )
                } else {
                    (self.int_lit(0, span), ord)
                };
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(lo),
                        hi: Box::new(hi),
                    },
                    Ty::range(Ty::Pos(d)),
                    span,
                )
            }
            (Ty::Pos(d), "l") => self.cast(Cast::Trust, value, Ty::Bound(*d)),
            (Ty::Pos(d), "r") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let next = self.int_add(ord, 1);
                self.cast(Cast::Trust, next, Ty::Bound(*d))
            }
            (Ty::Pos(d), field) if self.product_domains.contains_key(d) => {
                let d = *d;
                let factors = self.product_domains[&d].clone();
                if let Some(idx) = factors
                    .iter()
                    .position(|(fnm, _)| fnm.as_deref() == Some(field))
                    .or_else(|| field.parse::<usize>().ok().filter(|&i| i < factors.len()))
                {
                    self.decode_product_field(d, value, idx, span)
                } else if let Some(res) =
                    self.try_ufcs_call(field, base, &[], span, Some(recv_mark), None)
                {
                    res
                } else if self.self_getter(field) {
                    // A product position is a plain vertex: its own `Target::to`.
                    value
                } else {
                    self.report(
                        name.span,
                        msgs::ir::UNKNOWN_FIELD_HAS_NO_FIELD,
                        &[&self.show(&ty), &field],
                    );
                    self.err_expr(span)
                }
            }
            (Ty::Bound(d), "next") => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let cuts = self.int_add(len, 1);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![value, cuts],
                    Ty::opt(ty.clone()),
                )
            }
            (Ty::Bound(_), "prev") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(ty.clone()))
            }
            (Ty::Bound(d), "left" | "l") => {
                builtin(self, Builtin::PosPrev, vec![value], Ty::opt(Ty::Pos(*d)))
            }
            (Ty::Bound(d), "right" | "r") => {
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let before = self.int_add(ord, -1);
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                builtin(
                    self,
                    Builtin::PosNext,
                    vec![before, len],
                    Ty::opt(Ty::Pos(*d)),
                )
            }
            (Ty::Range(inner), "start" | "l") => {
                let inner = (**inner).clone();
                let res_ty = match inner {
                    Ty::I128 | Ty::Float => inner.clone(),
                    _ => Ty::Int,
                };
                let lo = builtin(self, Builtin::RangeLo, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, lo, Ty::Bound(d)),
                    _ => lo,
                }
            }
            (Ty::Range(inner), "end" | "r") => {
                let inner = (**inner).clone();
                let res_ty = match inner {
                    Ty::I128 | Ty::Float => inner.clone(),
                    _ => Ty::Int,
                };
                let hi = builtin(self, Builtin::RangeHi, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, hi, Ty::Bound(d)),
                    _ => hi,
                }
            }
            (Ty::Struct(id), field_name) if self.per_value_domain(*id, field_name) => {
                match self.value_domain(value.clone(), field_name, span) {
                    Some(d) => self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::Struct(id), field_name)
                if let Some(GenArg::Domain(d)) = self.structs[*id].env.get(field_name).cloned() =>
            {
                self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span)
            }
            (Ty::Range(inner), "first" | "last") => {
                let inner = (**inner).clone();
                let piece = matches!(self.expr_range(&value), Some(RangeInfo::Piece));
                let op = if name.text == "first" {
                    Builtin::First
                } else {
                    Builtin::Last
                };
                let end = builtin(self, op, vec![value], Ty::opt(inner.clone()));
                match piece {
                    true => self.mk_unwrap(end, false, inner, span),
                    false => end,
                }
            }
            (Ty::DomainVal, "len" | "size" | "count") => {
                // If this is `xs.D.K` where `xs` is a struct holding a `[D]`-indexed array field,
                // read the instance's own array length so multiple instances of `compress` don't alias one global domain length.
                if let ast::ExprKind::Field {
                    base: owner_ast,
                    name: dom_field,
                } = &base.kind
                {
                    if let Some(Ty::Struct(sid)) = self.probe_ty_silent(owner_ast) {
                        if let Some(GenArg::Domain(d)) =
                            self.structs[sid].env.get(&dom_field.text).cloned()
                        {
                            if let Some((fidx, fty)) = self.structs[sid]
                                .def
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, (_, ty))| {
                                    matches!(ty, Ty::Array { dims, .. } if dims.first() == Some(&Dim::Pos(d)))
                                })
                                .map(|(i, (_, ty))| (i, ty.clone()))
                            {
                                let owner_val = self.expr(owner_ast, None);
                                let arr = self.mk_field(owner_val, fidx, fty, span);
                                return builtin(self, Builtin::Len, vec![arr], Ty::Int);
                            }
                        }
                    }
                }
                builtin(self, Builtin::Len, vec![value], Ty::Int)
            }
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
                | Ty::Str
                | Ty::StrView
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Range(_),
                "len",
            ) => self.len_of(value, span),
            (Ty::Range(inner), "is_empty") => {
                let bound_ty = if **inner == Ty::I128 {
                    Ty::I128
                } else {
                    Ty::Int
                };
                let r_ref = if matches!(value.kind, ExprKind::Local(_)) {
                    value
                } else {
                    let slot = self.new_slot("r", value.ty.clone());
                    self.cx.pending.push_let(slot, value);
                    self.local(slot, span)
                };
                let lo = builtin(
                    self,
                    Builtin::RangeLo,
                    vec![r_ref.clone()],
                    bound_ty.clone(),
                );
                let hi = builtin(self, Builtin::RangeHi, vec![r_ref], bound_ty);
                self.mk_cmp(CmpOp::Ge, lo, hi, Ty::Bool, span)
            }
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
                | Ty::Str
                | Ty::StrView
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_),
                "is_empty",
            ) => {
                let len = self.len_of(value, span);
                let zero = self.int_lit(0, span);
                self.mk_cmp(CmpOp::Eq, len, zero, Ty::Bool, span)
            }
            // The ends of a sequence are optionals, `none` when it is
            // empty: a list, and an array over one domain or count.
            (Ty::Array { dims, elem }, "first" | "last") if dims.len() == 1 => {
                let opt = Ty::opt((**elem).clone());
                let end = if name.text == "last" {
                    Builtin::Last
                } else {
                    Builtin::First
                };
                builtin(self, end, vec![value], opt)
            }
            // Of a grid they are `g[R.first, C.first]` and `g[R.last, C.last]`.
            (Ty::Array { dims, elem }, "first" | "last")
                if let Some(axes) = dims
                    .iter()
                    .map(|dim| match dim {
                        Dim::Bound(d) => Some((*d, 0, Ty::Bound(*d))),
                        Dim::Pos(d) => Some((*d, -1, Ty::Pos(*d))),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>() =>
            {
                let last = name.text == "last";
                let mut filled = self.mk(ExprKind::Bool(true), Ty::Bool, span);
                let mut indices = Vec::new();
                for (d, extra, ty) in axes {
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let zero = self.int_lit(0, span);
                    let some = self.mk_cmp(CmpOp::Gt, len.clone(), zero.clone(), Ty::Bool, span);
                    filled = self.mk_binary(BinOp::And, filled, some, Ty::Bool, span);
                    let at = if last { self.int_add(len, extra) } else { zero };
                    indices.push(self.mk(ExprKind::Cast(Cast::Trust, Box::new(at)), ty, span));
                }
                let opt = Ty::opt((**elem).clone());
                let at = self.mk_index(value, indices, (**elem).clone(), span);
                let some = self.mk_some(at, opt.clone(), span);
                let none = self.mk(ExprKind::None, opt.clone(), span);
                self.mk_if(filled, some, none, opt, span)
            }
            // `xs.items` kept as a value: the list of `(index, element)`.
            (Ty::Map(key, held) | Ty::HashMap(key, held), "items") => {
                let (key, held) = ((**key).clone(), (**held).clone());
                self.pairs_of_map(value, key, held, span)
            }
            (Ty::Map(key, held) | Ty::HashMap(key, held), "keys") => {
                let kv = ((**key).clone(), (**held).clone());
                self.proj_of_map(value, kv, 0, span)
            }
            // A range's keys are its own elements (R04.4.31): `r.keys` is `r`.
            (Ty::Range(_), "keys") => value,
            (Ty::ListView(..), "keys") if self.is_reversed(&value) => {
                self.reversed_keys(value, span)
            }
            (Ty::ListView(..), "src") if self.is_reversed(&value) => {
                self.reversed_source(value, span)
            }
            (Ty::List(_) | Ty::ListView(..) | Ty::MutView(..) | Ty::Str | Ty::StrView, "keys") => {
                self.seq_keys(value, span)
            }
            // A grid's keys are its cells, positions of the product of its axes (R04.2.10).
            (Ty::Array { dims, .. }, "keys") if dims.len() > 1 => {
                let factors: Option<Vec<_>> = dims
                    .iter()
                    .map(|d| match d {
                        Dim::Pos(d) => Some((None, ProductFactor::Domain(*d))),
                        _ => None,
                    })
                    .collect();
                let Some(factors) = factors else {
                    self.report(span, msgs::ir::UNSUPPORTED_THE_KEYS_OF_A, &[]);
                    return self.err_expr(span);
                };
                let p = self.intern_anon_product_domain(factors);
                let mut defs = Vec::new();
                self.def_anon_product_domain(p, span, &mut defs);
                self.cx.pending.extend(defs);
                self.domain_keys(p, span)
            }
            (Ty::Array { dims, .. }, "keys") if dims.len() == 1 => match dims[0] {
                // The bounds are ends, not positions: `hi` is one past the last key
                // and `lo` is no key at all when the domain is empty.
                Dim::Pos(d) => self.domain_keys(d, span),
                // `[D.cuts]T` is keyed by `D`'s cuts, one more than it has positions.
                Dim::Bound(d) => {
                    let lo = self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(self.int_lit(0, span))),
                        Ty::Bound(d),
                        span,
                    );
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let n = self.int_add(len, 1);
                    let hi = self.mk(ExprKind::Cast(Cast::Trust, Box::new(n)), Ty::Bound(d), span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Bound(d)),
                        span,
                    )
                }
                dim => {
                    let first = if let Dim::IntFrom(first) = dim {
                        first
                    } else {
                        0
                    };
                    let lo = self.int_lit(first, span);
                    let len = self.len_of(value, span);
                    let hi = self.int_add(len, first);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Int),
                        span,
                    )
                }
            },
            (Ty::Map(key, held) | Ty::HashMap(key, held), "values") => {
                let kv = ((**key).clone(), (**held).clone());
                self.proj_of_map(value, kv, 1, span)
            }
            (Ty::Array { dims, elem }, "items")
                if dims.len() > 1 && dims.iter().all(|d| !matches!(d, Dim::Bound(_))) =>
            {
                let shape = (dims.clone(), (**elem).clone());
                self.cells_of_grid(value, shape, span)
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView,
                "items",
            ) if let Some(kind) = Self::items_kind(&value.ty) => {
                let shape = self.items_in(&value, kind.clone(), span);
                let pair = |items: &Items| Ty::Tuple(vec![items.index.clone(), items.elem.clone()]);
                let held_ty = value.ty.clone();
                let list_ty = Ty::list(pair(&shape));
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let held = lw.temp(held_ty);
                    out.push_let(held, value);
                    let seq = lw.local(held, span);
                    let items = lw.items_in(&seq, kind, span);
                    let acc = lw.temp(list_ty.clone());
                    out.push_let(
                        acc,
                        lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span),
                    );
                    let k = lw.temp(Ty::Int);
                    let (at, cell) = lw.item_at(&items, k, span);
                    let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair(&items), span);
                    let place = Place {
                        kind: PlaceKind::Local(acc),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    };
                    let push = lw.mk_mutate(Mutation::Push, place, vec![both], Ty::Unit, span);
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(
                        lid,
                        k,
                        items.range(),
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push_give(lw.local(acc, span));
                })
            }
            (
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _),
                "first",
            ) => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::First, vec![value], opt)
            }
            (
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _),
                "last",
            ) => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::Last, vec![value], opt)
            }
            (Ty::Keyed(_, _), "keys") => {
                let seq = if Self::keyed_view(&ty) {
                    self.hold_view(value, span)
                } else {
                    value
                };
                let lo = self.key_lo(&seq, span);
                let len = self.len_of(seq, span);
                let hi = if matches!(lo.kind, ExprKind::Int(0)) {
                    len
                } else {
                    self.int_sum(lo.clone(), len)
                };
                let key = ty.key_ty();
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(lo),
                        hi: Box::new(hi),
                    },
                    Ty::range(key),
                    span,
                )
            }
            (Ty::Str | Ty::StrView, "first") => {
                builtin(self, Builtin::First, vec![value], Ty::opt(Ty::Char))
            }
            (Ty::Str | Ty::StrView, "last") => {
                builtin(self, Builtin::Last, vec![value], Ty::opt(Ty::Char))
            }
            (Ty::Str | Ty::StrView, "rev") => {
                let value = self.hoist_view_base(value, span);
                builtin(self, Builtin::Reversed, vec![value], Ty::StrView)
            }
            (
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _),
                "rev",
            ) => {
                // A reversed slice keeps its parent's keys (R07.3.5).
                let (elem, dim) = ((**elem).clone(), ty.seq_key().unwrap_or(Dim::Int));
                let value = self.hoist_view_base(value, span);
                builtin(
                    self,
                    Builtin::Reversed,
                    vec![value],
                    Ty::ListView(Box::new(elem), dim),
                )
            }
            // `a.rev` keeps the keys of its source (R07.3.5).
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 => {
                let (elem, dim) = ((**elem).clone(), dims[0]);
                let value = self.hoist_view_base(value, span);
                builtin(
                    self,
                    Builtin::Reversed,
                    vec![value],
                    Ty::ListView(Box::new(elem), dim),
                )
            }
            (Ty::Range(of), "rev") => {
                let of = (**of).clone();
                self.reversed_range(value, of, span)
            }
            (Ty::Struct(sid), "len" | "distinct") if self.counter_key.contains_key(sid) => {
                let map_ty = self.structs[*sid].def.fields[0].1.clone();
                let map_expr = self.mk_field(value, 0, map_ty, span);
                self.len_of(map_expr, span)
            }
            (Ty::Char, "ord" | "idx") => self.cast(Cast::CharToInt, value, Ty::Int),
            (Ty::Pos(d), "len" | "count") => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            (Ty::DomainVal, "first") => {
                let dom_id = self.gen_env.values().find_map(|ga| match ga {
                    GenArg::Domain(id) => Some(*id),
                    _ => None,
                });
                if let Some(id) = dom_id {
                    self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(self.int_lit(0, span))),
                        Ty::Pos(id),
                        span,
                    )
                } else {
                    self.err_expr(span)
                }
            }
            (Ty::DomainVal, "last") => {
                let dom_id = self.gen_env.values().find_map(|ga| match ga {
                    GenArg::Domain(id) => Some(*id),
                    _ => None,
                });
                if let Some(id) = dom_id {
                    let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                    let last = self.int_add(len, -1);
                    self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(last)),
                        Ty::Pos(id),
                        span,
                    )
                } else {
                    self.err_expr(span)
                }
            }
            // The smallest and largest key of an ordered set (v2.2; `.min()`
            // and `.max()` are the older spelling).
            (Ty::Set(_) | Ty::Map(..), "first" | "last") => {
                let op = if name.text == "first" {
                    Builtin::SetMin
                } else {
                    Builtin::SetMax
                };
                let ty = Ty::opt(match &value.ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    Ty::Set(elem) => (**elem).clone(),
                    _ => Ty::Error,
                });
                self.mk_builtin(op, [value], ty, span)
            }
            (Ty::Opt(_), "is_some") => self.mk_is_some(value, Ty::Bool, span),
            (Ty::Opt(_), "is_none") => {
                let some = self.mk_is_some(value, Ty::Bool, span);
                self.mk_unary(UnOp::Not, some, Ty::Bool, span)
            }
            // A trait getter defaulted to `= self` on a plain value: a vertex's `Target::to`.
            (_, field) if self.self_getter(field) => value,
            // `Hashable::hash_token` of a position, until a blanket domain impl's getter reaches it.
            _ => {
                // A getter an impl gives a non-struct type (`'a'.hash_token`, `v.tk` for a domain).
                if let Some(fid) = self.value_impl_method(&ty, &name.text) {
                    if self.sigs[fid].decl.params.len() == 1 {
                        self.cx.pending.truncate(recv_mark);
                        let recv = ast::Arg {
                            name: None,
                            mutable: false,
                            value: base.clone(),
                        };
                        return self.user_call(fid, None, &[recv], span);
                    }
                }
                // UFCS `@get` free function (e.g. `p.adj4`, `g.transposed`, `g.flipped`).
                if let Some(res) =
                    self.try_ufcs_call(&name.text, base, &[], span, Some(recv_mark), None)
                {
                    return res;
                }
                self.report(
                    name.span,
                    msgs::check::UNKNOWN_FIELD_HAS_NO_MEMBER,
                    &[&self.a_ty(&ty), &name.text],
                );
                self.member_typo(&ty, &name.text, name.span);
                self.position_member_hint(&ty, &name.text);
                self.err_expr(span)
            }
        }
    }

    /// The outer shape of a type, for telling whose method a call means:
    /// a list and an array are one shape, as a map and a hash map are.
    pub(in crate::lower) fn shape(&self, ty: &Ty) -> Option<String> {
        let plain = match ty {
            Ty::Int | Ty::I128 | Ty::U64 => "Int",
            Ty::Float => "F64",
            Ty::Bool => "Bool",
            Ty::Char => "Char",
            Ty::Str => "Str",
            Ty::List(_) | Ty::Array { .. } => "List",
            Ty::Map(..) | Ty::HashMap(..) => "Map",
            Ty::Set(_) | Ty::HashSet(_) => "Set",
            Ty::Opt(_) => "?",
            Ty::Tuple(_) | Ty::Record(_) => "()",
            // A struct or enum is its own shape: `foo(self: A)` is not
            // for a `B`.
            Ty::Struct(id) => {
                return Some(match &self.structs[*id].origin_gen {
                    Some((gid, _)) => format!("struct<> {gid}"),
                    None => format!("struct {id}"),
                });
            }
            Ty::Enum(id) => return Some(format!("enum {id}")),
            _ => return None,
        };
        Some(plain.to_string())
    }

    /// The shape of a type as a fn's `self` writes it; `None` for one of
    /// the fn's own type parameters, which takes any.
    pub(in crate::lower) fn written_shape(
        &self,
        ty: &ast::Type,
        generics: &[ast::Name],
    ) -> Option<String> {
        let plain = match &ty.kind {
            ast::TypeKind::Array { .. } | ast::TypeKind::OwnDomain { .. } => "List",
            ast::TypeKind::Optional(_) => "?",
            ast::TypeKind::Tuple(_) | ast::TypeKind::Record(_) => "()",
            ast::TypeKind::Range { .. } => "Int",
            ast::TypeKind::Named { name, .. } => match name.text.as_str() {
                "Int" | "I128" | "U64" => "Int",
                "F64" => "F64",
                "Bool" => "Bool",
                "Char" => "Char",
                "Str" => "Str",
                "List" => "List",
                "Map" | "HashMap" => "Map",
                "Set" | "HashSet" => "Set",
                other if generics.iter().any(|g| g.text == other) => return None,
                other => {
                    return match self.get_sym(other)? {
                        Sym::Struct(id) => Some(format!("struct {id}")),
                        Sym::GenStruct(gid) => Some(format!("struct<> {gid}")),
                        Sym::Enum(id) => Some(format!("enum {id}")),
                        _ => None,
                    };
                }
            },
            _ => return None,
        };
        Some(plain.to_string())
    }

    /// Every top-level `fn name(self: ..)` in reach: the module's own
    /// `name`, then each module's under its `method_key` (R11.3.4).
    pub(in crate::lower) fn method_syms(&self, name: &str) -> Vec<Sym> {
        let prefix = cpl_check::method_key(name, "");
        let mut keys: Vec<&String> = (self.mod_syms[self.cur_mod].keys())
            .filter(|k| k.starts_with(&prefix))
            .collect();
        keys.sort();
        let keyed = keys.into_iter().filter_map(|k| self.get_sym(k));
        self.get_sym(name).into_iter().chain(keyed).collect()
    }

    /// R06.1.6: whether a call with `args`' labels fits the fn `sym`.
    pub(in crate::lower) fn sym_labels_fit(&self, sym: Sym, args: &[ast::Arg]) -> bool {
        let decl = match sym {
            Sym::Fn(fid) => self.sigs[fid].decl,
            Sym::GenFn(gfid) => self.gen_fns[gfid].decl,
            _ => return true,
        };
        let given: Vec<&str> = (args.iter())
            .filter_map(|a| a.name.as_ref().map(|n| n.text.as_str()))
            .collect();
        cpl_check::labels_fit(&decl.params, &given)
    }

    /// R06.1.6: the key of the overload of `name(..)` whose labels the call
    /// gives, when the name has several and that one is not the first.
    pub(in crate::lower) fn overload_for(&self, name: &str, args: &[ast::Arg]) -> Option<String> {
        if cpl_check::overload_part(name).is_some() {
            return None;
        }
        let prefix = cpl_check::method_key(name, "");
        let mut keys: Vec<String> = (self.mod_syms[self.cur_mod].keys())
            .filter(|k| k.starts_with(&prefix) && cpl_check::overload_part(k).is_some())
            .cloned()
            .collect();
        if keys.is_empty() {
            return None;
        }
        keys.sort();
        keys.insert(0, name.to_string());
        let fits: Vec<String> = (keys.into_iter())
            .filter(|k| {
                self.get_sym(k)
                    .is_some_and(|s| self.sym_labels_fit(s, args))
            })
            .collect();
        match fits.as_slice() {
            [one] if one != name => Some(one.clone()),
            _ => None,
        }
    }

    /// Whether the checker sent `x.name(..)` past the built-in member of the
    /// name, whose parameters can't take the call's arguments.
    pub(in crate::lower) fn past_builtin_row(&self, name: Span) -> bool {
        let items = &self.mods[self.cur_mod].items;
        (items.binary_search_by_key(&self.offset, |i| i.offset).ok())
            .is_some_and(|at| items[at].check.exprs.past_rows.contains(&name))
    }

    /// The module of the fn the checker resolved `base.name(..)` to, `""`
    /// for this file's.
    pub(in crate::lower) fn resolved_module(
        &self,
        name: &str,
        base: &ast::Expr,
        span: Span,
    ) -> Option<String> {
        let items = &self.mods[self.cur_mod].items;
        let item = &items[items
            .binary_search_by_key(&self.offset, |i| i.offset)
            .ok()?];
        let (_, found) = (item.check.resolved.iter())
            .filter(|(at, r)| at.start >= base.span.end && at.end <= span.end && r.name == name)
            .filter(|(_, r)| r.owner.is_none())
            .min_by_key(|(at, _)| at.start)?;
        Some(found.module.clone())
    }

    /// The fn the checker resolved `base.name(..)` to: the `Resolved` at
    /// the method's name, the first one after `base` inside `span`.
    pub(in crate::lower) fn resolved_method(
        &self,
        name: &str,
        base: &ast::Expr,
        span: Span,
    ) -> Option<Sym> {
        match self.resolved_module(name, base, span)?.as_str() {
            "" => self.get_sym(name),
            module => {
                (self.get_sym(&cpl_check::method_key(name, module))).or_else(|| self.get_sym(name))
            }
        }
    }

    /// Where the checker left no choice (a receiver of a type parameter):
    /// the first candidate whose written `self` shape the receiver may have.
    /// A fn that can't take `arg_count` arguments after `self` is not the call:
    /// `xs.fold()` is then the built-in `fold`, reported as such.
    pub(in crate::lower) fn fitting_method(
        &mut self,
        candidates: &[Sym],
        base: &ast::Expr,
        arg_count: usize,
    ) -> Option<Sym> {
        let given = self.probe_ty_silent(base).and_then(|ty| self.shape(&ty));
        let fits = candidates.iter().copied().find(|&sym| {
            let decl = match sym {
                Sym::Fn(fid) => self.sigs[fid].decl,
                Sym::GenFn(gfid) => self.gen_fns[gfid].decl,
                _ => return false,
            };
            let Some(first) = decl.params.first().filter(|p| p.name.text == "self") else {
                return false;
            };
            let required = (decl.params.iter())
                .filter(|p| p.default.is_none() && !p.variadic)
                .count();
            let variadic = decl.params.iter().any(|p| p.variadic);
            if arg_count + 1 < required || (!variadic && arg_count + 1 > decl.params.len()) {
                return false;
            }
            let takes = first
                .ty
                .as_ref()
                .and_then(|t| self.written_shape(t, &decl.generics));
            !matches!((takes, &given), (Some(takes), Some(given)) if takes != *given)
        });
        fits
    }

    /// Tries calling a free function or generic free function whose first parameter is `self`
    /// using UFCS (`base.name(args...)` -> `name(base, args...)`).
    /// `recv_mark` is where `pending` stood before the caller lowered `base`
    /// itself: the call lowers `base` again, so what that first lowering
    /// hoisted is dropped and `base` runs once.
    pub(in crate::lower) fn try_ufcs_call(
        &mut self,
        name: &str,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
        recv_mark: Option<usize>,
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        let candidates: Vec<Sym> = (self.method_syms(name).into_iter())
            .filter(|&s| self.sym_labels_fit(s, args))
            .collect();
        let chosen = (self.resolved_method(name, base, span))
            .filter(|&s| self.sym_labels_fit(s, args))
            .or_else(|| self.fitting_method(&candidates, base, args.len()));
        let sym = match chosen {
            Some(s) => s,
            None => {
                // Fall back to a UFCS free function declared in the receiver struct's defining module.
                let Some(Ty::Struct(sid)) = self.probe_ty_silent(base) else {
                    return None;
                };
                let mod_id = self.structs[sid].mod_id;
                *self.mod_syms[mod_id].get(name)?
            }
        };
        let (has_self, self_mut) = match sym {
            Sym::Fn(fid) => match self.sigs[fid].decl.params.first() {
                Some(p) if p.name.text == "self" => (true, p.mutable),
                _ => (false, false),
            },
            Sym::GenFn(gfid) => match self.gen_fns[gfid].decl.params.first() {
                Some(p) if p.name.text == "self" => (true, p.mutable),
                _ => (false, false),
            },
            _ => (false, false),
        };
        if !has_self || self.char_list_as_str(sym, base) {
            return has_self.then(|| self.err_expr(span));
        }
        if let Some(mark) = recv_mark {
            self.cx.pending.truncate(mark);
        }
        let mut all_args = Vec::with_capacity(args.len() + 1);
        all_args.push(ast::Arg {
            name: None,
            mutable: self_mut,
            value: base.clone(),
        });
        all_args.extend(args.iter().cloned());
        match sym {
            Sym::Fn(fid) => Some(self.user_call(fid, None, &all_args, span)),
            Sym::GenFn(gfid) => {
                self.call_hint.0 = hint.cloned();
                Some(self.call_gen_fn(gfid, Vec::new(), None, &all_args, span))
            }
            _ => None,
        }
    }
    /// `xs.items` in a `for` or a comprehension: the loop counts through the indices and reads
    /// each element in place, so no list of pairs is built.
    pub(in crate::lower) fn items_head(&mut self, e: &ast::Expr) -> Option<Items> {
        let ast::ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        if name.text != "items" || Self::filter_call(base).is_some() {
            return None;
        }
        // A domain's items are its range's: `I.items` walks `(i, i)`.
        if let Some(Sym::Domain(d)) = self.sym(base) {
            let len = self.mk(ExprKind::DomainLen(d), Ty::Int, e.span);
            let all = ExprKind::Range {
                lo: Box::new(self.int_lit(0, e.span)),
                hi: Box::new(len),
            };
            let all = self.mk(all, Ty::range(Ty::Pos(d)), e.span);
            return self.items_of(&all, e.span);
        }
        if !matches!(
            self.probe_ty_silent(base),
            Some(
                Ty::List(_)
                    | Ty::ListView(_, _)
                    | Ty::Keyed(_, _)
                    | Ty::MutView(..)
                    | Ty::Array { .. }
                    | Ty::Str
                    | Ty::StrView
                    | Ty::Range(_)
            )
        ) {
            return None;
        }
        let seq = self.expr(base, None);
        // A view or a temporary (`(0..n).items`, `f(a).items`) is held
        // once, so the loop reads it without building it again.
        let view = matches!(
            seq.ty,
            Ty::StrView | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
        );
        let seq = if (view || !is_place(base))
            && !matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_))
        {
            let held = self.temp(seq.ty.clone());
            self.cx.pending.push_let(held, seq);
            self.local(held, e.span)
        } else {
            seq
        };
        self.items_of(&seq, e.span)
    }

    /// The indices and element type `seq.items` goes through; `None`
    /// when `seq` is not a list or an array of one dimension.
    pub(in crate::lower) fn items_of(&mut self, seq: &Expr, span: Span) -> Option<Items> {
        let kind = Self::items_kind(&seq.ty)?;
        Some(self.items_in(seq, kind, span))
    }

    /// How a sequence of type `ty` walks its items, as its type alone says;
    /// `None` for a type without items.
    pub(in crate::lower) fn items_kind(ty: &Ty) -> Option<ItemsKind> {
        // A range's keys are its elements (R04.4.31): `(2..5).items` walks
        // `(2, 2) (3, 3) (4, 4)`.
        if let Ty::Range(of) = ty {
            if matches!(**of, Ty::Int | Ty::Pos(_)) {
                return Some(ItemsKind::Range((**of).clone()));
            }
        }
        let (dim, elem) = match ty {
            Ty::List(elem) => (Dim::Int, (**elem).clone()),
            Ty::ListView(elem, key) | Ty::Keyed(elem, key) | Ty::MutView(elem, key) => {
                (*key, (**elem).clone())
            }
            Ty::Str | Ty::StrView => (Dim::Int, Ty::Char),
            Ty::Array { dims, elem } if dims.len() == 1 => (dims[0], (**elem).clone()),
            _ => return None,
        };
        let count = match dim {
            Dim::Pos(d) => ItemsCount::Pos(d),
            Dim::Bound(d) => ItemsCount::Bound(d),
            Dim::Int => ItemsCount::Int,
            Dim::IntFrom(first) => ItemsCount::IntFrom(first),
            Dim::Enum(_) | Dim::Sym(_) => return None,
        };
        let keyed = Self::keyed_view(ty);
        Some(ItemsKind::Cells { count, elem, keyed })
    }

    /// The items of `seq`, walked as `kind` says.
    pub(in crate::lower) fn items_in(&mut self, seq: &Expr, kind: ItemsKind, span: Span) -> Items {
        let (count, elem, keyed) = match kind {
            ItemsKind::Range(of) => {
                let (lo, len) = self.range_lo_len(seq.clone(), span);
                return Items {
                    seq: seq.clone(),
                    len,
                    index: of.clone(),
                    first: 0,
                    keyed: false,
                    elem: of,
                    lo: Some(lo),
                };
            }
            ItemsKind::Cells { count, elem, keyed } => (count, elem, keyed),
        };
        // The loop counts the cells from 0; `[1..=9]T` names them from 1.
        let (len, index, first) = match count {
            ItemsCount::Pos(d) if keyed => (self.len_of(seq.clone(), span), Ty::Pos(d), 0),
            ItemsCount::Pos(d) => (
                self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                Ty::Pos(d),
                0,
            ),
            // `[D.cuts]T` has a cell per cut, one more than `D` has positions.
            ItemsCount::Bound(d) => {
                let n = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                (self.int_add(n, 1), Ty::Bound(d), 0)
            }
            ItemsCount::Int => (self.len_of(seq.clone(), span), Ty::Int, 0),
            ItemsCount::IntFrom(first) => (self.len_of(seq.clone(), span), Ty::Int, first),
        };
        Items {
            seq: seq.clone(),
            len,
            index,
            first,
            keyed,
            elem,
            lo: None,
        }
    }

    /// `m.items` kept as a value: the list of `(key, value)`, in the
    /// order a `for` over the map goes.
    pub(in crate::lower) fn pairs_of_map(
        &mut self,
        map: Expr,
        key: Ty,
        held: Ty,
        span: Span,
    ) -> Expr {
        let pair = Ty::Tuple(vec![key, held]);
        self.collected(pair.clone(), span, |lw, out, push| {
            let map = lw.hold(map, out);
            let each = lw.temp(pair);
            let body = Block {
                stmts: vec![push(lw, lw.local(each, span))],
            };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, each, Iter::Seq(map, false), body));
        })
    }

    /// `m.keys` / `m.values` kept as a value: the list of keys (`idx == 0`) or
    /// values (`idx == 1`), in the order a `for` over the map goes.
    pub(in crate::lower) fn proj_of_map(
        &mut self,
        map: Expr,
        (key, held): (Ty, Ty),
        idx: usize,
        span: Span,
    ) -> Expr {
        let pair = Ty::Tuple(vec![key.clone(), held.clone()]);
        let elem = if idx == 0 { key } else { held };
        self.collected(elem.clone(), span, |lw, out, push| {
            let map = lw.hold(map, out);
            let each = lw.temp(pair);
            let proj = lw.mk_field(lw.local(each, span), idx, elem.clone(), span);
            let body = Block {
                stmts: vec![push(lw, proj)],
            };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, each, Iter::Seq(map, false), body));
        })
    }

    /// `m.keys` / `m.values` (and `.rev` on an ordered `Map`) in a `for` or a
    /// comprehension: walks the map in place with no copied list.
    pub(in crate::lower) fn map_proj_head(&mut self, e: &ast::Expr) -> Option<MapProj> {
        let (field_expr, rev) = match &e.kind {
            ast::ExprKind::Field { base, name } if name.text == "rev" => (base.as_ref(), true),
            _ => (e, false),
        };
        let ast::ExprKind::Field { base, name } = &field_expr.kind else {
            return None;
        };
        let idx = match name.text.as_str() {
            "keys" => 0,
            "values" => 1,
            _ => return None,
        };
        let base_ty = self.probe_ty_silent(base)?;
        match &base_ty {
            Ty::Map(..) => {}
            Ty::HashMap(..) if !rev => {}
            _ => return None,
        }
        let map = self.expr(base, None);
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            return None;
        };
        let pair_ty = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
        let elem_ty = if idx == 0 {
            (**key).clone()
        } else {
            (**held).clone()
        };
        Some(MapProj {
            map,
            pair_ty,
            elem_ty,
            idx,
            rev,
        })
    }

    pub(in crate::lower) fn map_proj_bind(
        &mut self,
        pat: &ast::Pattern,
        proj: &MapProj,
        body: &mut Vec<Stmt>,
    ) -> Slot {
        let each = self.temp(proj.pair_ty.clone());
        let val = self.mk_field(
            self.local(each, pat.span),
            proj.idx,
            proj.elem_ty.clone(),
            pat.span,
        );
        self.bind(pat, val, body);
        each
    }

    /// `g.items` on an array of two or more dimensions: every cell with
    /// the tuple of its indices, `((r, c), g[r, c])`, row by row.
    pub(in crate::lower) fn cells_of_grid(
        &mut self,
        grid: Expr,
        (dims, elem): (Vec<Dim>, Ty),
        span: Span,
    ) -> Expr {
        let index_tys: Vec<Ty> = dims
            .iter()
            .map(|dim| match dim {
                Dim::Pos(d) => Ty::Pos(*d),
                _ => Ty::Int,
            })
            .collect();
        let at_ty = Ty::Tuple(index_tys);
        let pair = Ty::Tuple(vec![at_ty.clone(), elem.clone()]);
        self.collected(pair.clone(), span, |lw, out, push| {
            let grid = lw.hold(grid, out);
            let counters: Vec<Slot> = dims.iter().map(|_| lw.temp(Ty::Int)).collect();
            let mut named = Vec::new();
            let mut stored = Vec::new();
            for (dim, &k) in dims.iter().zip(&counters) {
                let k = lw.local(k, span);
                match *dim {
                    Dim::Pos(d) => {
                        let at = lw.mk(
                            ExprKind::Cast(Cast::FromOrd(d), Box::new(k)),
                            Ty::Pos(d),
                            span,
                        );
                        named.push(at.clone());
                        stored.push(at);
                    }
                    Dim::IntFrom(first) => {
                        named.push(lw.int_add(k.clone(), first));
                        stored.push(k);
                    }
                    _ => {
                        named.push(k.clone());
                        stored.push(k);
                    }
                }
            }
            let at = lw.mk(ExprKind::Tuple(named), at_ty, span);
            let cell = lw.mk_index(grid.clone(), stored, elem, span);
            let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair, span);
            let mut body = vec![push(lw, both)];
            for (axis, &k) in counters.iter().enumerate().rev() {
                let len = lw.mk_builtin(
                    Builtin::AxisLen,
                    [grid.clone(), lw.int_lit(axis as i64, span)],
                    Ty::Int,
                    span,
                );
                let iter = Iter::Range {
                    lo: lw.int_lit(0, span),
                    hi: len,
                    step: None,
                    rev: false,
                    closed: false,
                };
                let lid = lw.fresh_loop();
                body = vec![Stmt::For(lid, k, iter, Block { stmts: body })];
            }
            out.extend(body);
        })
    }
}

/// How a sequence walks its items, from its type: a range's are its
/// values, anything else's are its cells.
#[derive(Clone)]
pub(in crate::lower) enum ItemsKind {
    Range(Ty),
    Cells {
        count: ItemsCount,
        elem: Ty,
        keyed: bool,
    },
}

/// What a sequence's cells are counted by.
#[derive(Clone, Copy)]
pub(in crate::lower) enum ItemsCount {
    Pos(DomainId),
    Bound(DomainId),
    Int,
    IntFrom(i64),
}

impl Lowerer<'_> {
    /// A trait declares `@get fn name(self) -> P = self`, so a value with no impl of it is its
    /// own `name`: a vertex is its own `Target::to`.
    fn self_getter(&self, name: &str) -> bool {
        (self.traits.iter().flat_map(|t| &t.methods)).any(|f| {
            f.name.text == name
                && f.params.len() == 1
                && matches!(&f.body, ast::FnBody::Expr(e) if matches!(e.kind, ast::ExprKind::SelfValue))
        })
    }
}
