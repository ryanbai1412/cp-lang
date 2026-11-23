//! Fields and members, and the items/pairs/projection heads of a walk.

use super::*;

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
                let lookup = prefix_shape_struct(gname, gtypes.len()).unwrap_or(gname);
                if let Some(Sym::GenStruct(gid)) = self.get_sym(lookup) {
                    let gargs = self.resolve_gen_args_for_struct(gid, gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(sc) = self.structs[sid].consts.get(&name.text).cloned() {
                        return self.eval_struct_const(sid, sc, span);
                    }
                    // A method taking `self` is the value's: `Subset<B>.first` is the domain's.
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text).filter(|&&f| {
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
                    return self.domain_field(id, name, hint, span);
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
                        self.error(
                            codes::UNKNOWN_FIELD,
                            name.span,
                            format!("module has no member `{}`", name.text),
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
                if let Some(&fn_id) = self.structs[id].methods.get(&name.text).filter(|&&f| {
                    self.sigs[f].receiver.is_none() || self.structs[id].index_domain.is_none()
                }) {
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
                        if let Some(&fn_id) =
                            self.structs[id].methods.get(&name.text).filter(|&&f| {
                                self.sigs[f].receiver.is_none()
                                    || self.structs[id].index_domain.is_none()
                            })
                        {
                            return self.method_call(fn_id, None, &[], span);
                        }
                        if let Some(number) = self.struct_number_param(id, &name.text, span) {
                            return number;
                        }
                    }
                    Ty::Pos(id) => return self.domain_field(id, name, hint, span),
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
                            if let Some(&fid) = self.structs[*sid].methods.get(&name.text) {
                                self.queue_fn(fid);
                                let ret = self.ret_of(fid, span);
                                return self.mk(ExprKind::Call(fid, Vec::new()), ret, span);
                            }
                        }
                    }
                }
            }
            Some(Sym::Domain(id)) => return self.domain_field(id, name, hint, span),
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
                        let message = format!(
                            "the variant `{}` holds values: write `{}(...)`",
                            name.text, name.text
                        );
                        self.error(codes::WRONG_ARG_COUNT, span, message);
                        self.err_expr(span)
                    }
                    None if self.unit_enum(id) => self.enum_seq(base, id, name, None, span),
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            _ => {}
        }
        if let ast::ExprKind::Name(type_name) = &base.kind {
            if self.lookup(type_name).is_none() {
                if type_name == "Int"
                    || matches!(self.gen_env.get(type_name), Some(GenArg::Ty(Ty::Int)))
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
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text).filter(|&&f| {
                        self.sigs[f].receiver.is_none() || self.structs[sid].index_domain.is_none()
                    }) {
                        return self.method_call(fn_id, None, &[], span);
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
            }
        }
        if let Some(kept) = self.filtered_items(base, name, span) {
            return kept;
        }
        let value = self.expr(base, None);
        if let Some((index, ty)) = self.field_of(&value.ty, &name.text) {
            return self.mk(ExprKind::Field(Box::new(value), index), ty, span);
        }
        let ty = value.ty.clone();
        let builtin = |lw: &mut Self, op: Builtin, args: Vec<Expr>, ty: Ty| {
            lw.mk(ExprKind::Builtin(op, args), ty, span)
        };
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
                    Ty::Range(Box::new(Ty::Pos(d))),
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
                } else if let Some(res) = self.try_ufcs_call(field, base, &[], span) {
                    res
                } else if field == "to" {
                    // A product position is a plain vertex: its own `Target::to`.
                    value
                } else {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        format!("`{}` has no field `{}`", self.show(&ty), field),
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
                let res_ty = if inner == Ty::I128 { Ty::I128 } else { Ty::Int };
                let lo = builtin(self, Builtin::RangeLo, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, lo, Ty::Bound(d)),
                    _ => lo,
                }
            }
            (Ty::Range(inner), "end" | "r") => {
                let inner = (**inner).clone();
                let res_ty = if inner == Ty::I128 { Ty::I128 } else { Ty::Int };
                let hi = builtin(self, Builtin::RangeHi, vec![value], res_ty);
                match inner {
                    Ty::Pos(d) | Ty::Bound(d) => self.cast(Cast::Trust, hi, Ty::Bound(d)),
                    _ => hi,
                }
            }
            (Ty::Struct(id), field_name)
                if matches!(
                    self.structs[*id].env.get(field_name),
                    Some(GenArg::Domain(_))
                ) =>
            {
                let Some(GenArg::Domain(d)) = self.structs[*id].env.get(field_name).cloned() else {
                    unreachable!()
                };
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
                    true => self.mk(ExprKind::Unwrap(Box::new(end), false), inner, span),
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
                                let arr = self.mk(ExprKind::Field(Box::new(owner_val), fidx), fty, span);
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
                | Ty::MutView(_)
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
                    self.cx.pending.push(Stmt::Let(slot, Some(value)));
                    self.local(slot, span)
                };
                let lo = builtin(
                    self,
                    Builtin::RangeLo,
                    vec![r_ref.clone()],
                    bound_ty.clone(),
                );
                let hi = builtin(self, Builtin::RangeHi, vec![r_ref], bound_ty);
                self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(lo), Box::new(hi)),
                    Ty::Bool,
                    span,
                )
            }
            (
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
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
                self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
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
                if dims
                    .iter()
                    .all(|d| matches!(d, Dim::Pos(_) | Dim::Bound(_))) =>
            {
                let last = name.text == "last";
                let mut filled = self.mk(ExprKind::Bool(true), Ty::Bool, span);
                let mut indices = Vec::new();
                for dim in dims {
                    let (d, extra, ty) = match dim {
                        Dim::Bound(d) => (*d, 0, Ty::Bound(*d)),
                        Dim::Pos(d) => (*d, -1, Ty::Pos(*d)),
                        _ => unreachable!("guarded above"),
                    };
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let zero = self.int_lit(0, span);
                    let some = self.mk(
                        ExprKind::Cmp(CmpOp::Gt, Box::new(len.clone()), Box::new(zero.clone())),
                        Ty::Bool,
                        span,
                    );
                    filled = self.mk(
                        ExprKind::Binary(BinOp::And, Box::new(filled), Box::new(some)),
                        Ty::Bool,
                        span,
                    );
                    let at = if last { self.int_add(len, extra) } else { zero };
                    indices.push(self.mk(ExprKind::Cast(Cast::Trust, Box::new(at)), ty, span));
                }
                let opt = Ty::opt((**elem).clone());
                let at = self.mk(
                    ExprKind::Index(Box::new(value), indices),
                    (**elem).clone(),
                    span,
                );
                let some = self.mk(ExprKind::Some(Box::new(at)), opt.clone(), span);
                let none = self.mk(ExprKind::None, opt.clone(), span);
                self.mk(
                    ExprKind::If(Box::new(filled), Box::new(some), Box::new(none)),
                    opt,
                    span,
                )
            }
            // `xs.items` kept as a value: the list of `(index, element)`.
            (Ty::Map(..) | Ty::HashMap(..), "items") => self.pairs_of_map(value, span),
            (Ty::Map(..) | Ty::HashMap(..), "keys") => self.proj_of_map(value, 0, span),
            // A range's keys are its own elements (R04.4.31): `r.keys` is `r`.
            (Ty::Range(_), "keys") => value,
            (Ty::ListView(..), "keys")
                if Self::reversed_list(&value) || self.named_reversed(&value) =>
            {
                self.reversed_keys(value, span)
            }
            (Ty::List(_) | Ty::ListView(..) | Ty::MutView(_) | Ty::Str | Ty::StrView, "keys") => {
                self.seq_keys(value, span)
            }
            (Ty::Array { dims, .. }, "keys") if dims.len() == 1 => match dims[0] {
                // The bounds are ends, not positions: `hi` is one past the last key
                // and `lo` is no key at all when the domain is empty.
                Dim::Pos(d) => {
                    let lo = self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(self.int_lit(0, span))),
                        Ty::Pos(d),
                        span,
                    );
                    let hi = self.mk(
                        ExprKind::Cast(
                            Cast::Trust,
                            Box::new(self.mk(ExprKind::DomainLen(d), Ty::Int, span)),
                        ),
                        Ty::Pos(d),
                        span,
                    );
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                _ => {
                    let lo = self.int_lit(0, span);
                    let hi = self.len_of(value, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
            },
            (Ty::Map(..) | Ty::HashMap(..), "values") => self.proj_of_map(value, 1, span),
            (Ty::Array { dims, .. }, "items")
                if dims.len() > 1 && dims.iter().all(|d| !matches!(d, Dim::Bound(_))) =>
            {
                self.cells_of_grid(value, span)
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView,
                "items",
            ) if self.items_of(&value, span).is_some() => {
                let pair = |items: &Items| Ty::Tuple(vec![items.index.clone(), items.elem.clone()]);
                let held_ty = value.ty.clone();
                let Some(shape) = self.items_of(&value, span) else {
                    unreachable!("checked above")
                };
                let list_ty = Ty::List(Box::new(pair(&shape)));
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let held = lw.temp(held_ty);
                    out.push(Stmt::Let(held, Some(value)));
                    let seq = lw.local(held, span);
                    let items = lw.items_of(&seq, span).expect("the same type");
                    let acc = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(
                        acc,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
                    ));
                    let k = lw.temp(Ty::Int);
                    let (at, cell) = lw.item_at(&items, k, span);
                    let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair(&items), span);
                    let place = Place {
                        kind: PlaceKind::Local(acc),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    };
                    let push = lw.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![both]),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(
                        lid,
                        k,
                        items.range(),
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            (
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem),
                "first",
            ) => {
                let opt = Ty::opt((**elem).clone());
                builtin(self, Builtin::First, vec![value], opt)
            }
            (
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem),
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
                    Ty::Range(Box::new(key)),
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
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem),
                "rev",
            ) => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                builtin(
                    self,
                    Builtin::Reversed,
                    vec![value],
                    Ty::ListView(Box::new(elem), Dim::Int),
                )
            }
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                builtin(
                    self,
                    Builtin::Reversed,
                    vec![value],
                    Ty::ListView(Box::new(elem), Dim::Int),
                )
            }
            (Ty::Range(_), "rev") => self.reversed_range(value, span),
            (Ty::List(elem) | Ty::ListView(elem, _) | Ty::MutView(elem), "rebase") => {
                let elem = (**elem).clone();
                self.rebase(value, elem, span)
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Str
                | Ty::StrView,
                "adjacent",
            ) => self.lower_adjacent(value, span),
            (Ty::Array { dims, .. }, "adjacent") if dims.len() == 1 => {
                self.lower_adjacent(value, span)
            }
            (Ty::Struct(sid), "len" | "distinct") if self.counter_key.contains_key(sid) => {
                let map_ty = self.structs[*sid].def.fields[0].1.clone();
                let map_expr = self.mk(ExprKind::Field(Box::new(value), 0), map_ty, span);
                self.len_of(map_expr, span)
            }
            (Ty::Char, "ord" | "idx") => self.cast(Cast::CharToInt, value, Ty::Int),
            (Ty::Char, "is_digit") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'0' as i64, span);
                let hi = self.int_lit(b'9' as i64, span);
                let ge = self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)),
                    Ty::Bool,
                    span,
                );
                let le = self.mk(
                    ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)),
                    Ty::Bool,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Char, "is_lower") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'a' as i64, span);
                let hi = self.int_lit(b'z' as i64, span);
                let ge = self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)),
                    Ty::Bool,
                    span,
                );
                let le = self.mk(
                    ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)),
                    Ty::Bool,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Char, "is_upper") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let lo = self.int_lit(b'A' as i64, span);
                let hi = self.int_lit(b'Z' as i64, span);
                let ge = self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)),
                    Ty::Bool,
                    span,
                );
                let le = self.mk(
                    ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(hi)),
                    Ty::Bool,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(le)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Char, "is_alpha") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let a = self.int_lit(b'a' as i64, span);
                let z = self.int_lit(b'z' as i64, span);
                let cap_a = self.int_lit(b'A' as i64, span);
                let cap_z = self.int_lit(b'Z' as i64, span);
                let lo1 = self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(a)),
                    Ty::Bool,
                    span,
                );
                let hi1 = self.mk(
                    ExprKind::Cmp(CmpOp::Le, Box::new(ord.clone()), Box::new(z)),
                    Ty::Bool,
                    span,
                );
                let is_lo = self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(lo1), Box::new(hi1)),
                    Ty::Bool,
                    span,
                );
                let lo2 = self.mk(
                    ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(cap_a)),
                    Ty::Bool,
                    span,
                );
                let hi2 = self.mk(
                    ExprKind::Cmp(CmpOp::Le, Box::new(ord), Box::new(cap_z)),
                    Ty::Bool,
                    span,
                );
                let is_hi = self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(lo2), Box::new(hi2)),
                    Ty::Bool,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(is_lo), Box::new(is_hi)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Char, "is_space") => {
                let ord = self.cast(Cast::CharToInt, value, Ty::Int);
                let sp = self.int_lit(b' ' as i64, span);
                let tab = self.int_lit(b'\t' as i64, span);
                let nl = self.int_lit(b'\n' as i64, span);
                let cr = self.int_lit(b'\r' as i64, span);
                let e1 = self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(sp)),
                    Ty::Bool,
                    span,
                );
                let e2 = self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(tab)),
                    Ty::Bool,
                    span,
                );
                let e3 = self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(ord.clone()), Box::new(nl)),
                    Ty::Bool,
                    span,
                );
                let e4 = self.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(ord), Box::new(cr)),
                    Ty::Bool,
                    span,
                );
                let o1 = self.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(e1), Box::new(e2)),
                    Ty::Bool,
                    span,
                );
                let o2 = self.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(e3), Box::new(e4)),
                    Ty::Bool,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(o1), Box::new(o2)),
                    Ty::Bool,
                    span,
                )
            }
            (Ty::Pos(d), "len" | "count") => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            (Ty::DomainVal, "first") => {
                let dom_id = match value.kind {
                    ExprKind::DomainVal(id) => Some(id),
                    _ => self.gen_env.values().find_map(|ga| match ga {
                        GenArg::Domain(id) => Some(*id),
                        _ => None,
                    }),
                };
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
                let dom_id = match value.kind {
                    ExprKind::DomainVal(id) => Some(id),
                    _ => self.gen_env.values().find_map(|ga| match ga {
                        GenArg::Domain(id) => Some(*id),
                        _ => None,
                    }),
                };
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
                self.mk(ExprKind::Builtin(op, vec![value]), ty, span)
            }
            (Ty::Opt(_), "is_some") => self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span),
            (Ty::Opt(_), "is_none") => {
                let some = self.mk(ExprKind::IsSome(Box::new(value)), Ty::Bool, span);
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span)
            }
            // Default `Target::to` trait getter on plain values (e.g. vertices yielded by `u => adj[u]`).
            (_, "to") => value,
            // Default `Hashable::hash_token` trait getter (`Int(self)`).
            (Ty::Int, "hash_token") => value,
            (Ty::Char, "hash_token") => self.cast(Cast::CharToInt, value, Ty::Int),
            (Ty::Bool, "hash_token") => self.mk(
                ExprKind::If(
                    Box::new(value),
                    Box::new(self.int_lit(1, span)),
                    Box::new(self.int_lit(0, span)),
                ),
                Ty::Int,
                span,
            ),
            (Ty::Pos(_), "hash_token") => self.cast(Cast::Ord, value, Ty::Int),
            // A number's math reads either way: `sqrt(x)` or `x.sqrt`.
            (Ty::Float, "sqrt") => self.mk(
                ExprKind::Builtin(Builtin::Sqrt, vec![value]),
                Ty::Float,
                span,
            ),
            (Ty::Int, "sqrt") => {
                let value = self.coerce(value, &Ty::Float, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Sqrt, vec![value]),
                    Ty::Float,
                    span,
                )
            }
            (Ty::Float, "floor" | "ceil" | "round") => {
                let op = match name.text.as_str() {
                    "floor" => Builtin::Floor,
                    "ceil" => Builtin::Ceil,
                    _ => Builtin::Round,
                };
                self.mk(ExprKind::Builtin(op, vec![value]), Ty::Int, span)
            }
            (Ty::Float, "trunc") => self.cast(Cast::FloatToInt, value, Ty::Int),
            (Ty::Int | Ty::I128 | Ty::Float, "abs") => self.mk(
                ExprKind::Builtin(Builtin::Abs, vec![value]),
                ty.clone(),
                span,
            ),
            _ => {
                // UFCS `@get` free function (e.g. `p.adj4`, `g.transposed`, `g.flipped`).
                if let Some(res) = self.try_ufcs_call(&name.text, base, &[], span) {
                    return res;
                }
                let message = format!(
                    "{} has no member `{}` that `cpl run` knows",
                    self.a_ty(&ty),
                    name.text
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                self.member_typo(&ty, &name.text, name.span);
                match (&ty, name.text.as_str()) {
                    // An arc of a graph built from a list of pairs is the
                    // neighbour, with no record around it.
                    (Ty::Pos(_), "e") => self.help(
                        "this is the neighbour itself, and a graph built from a list of pairs numbers no edges; \
                         for `.e`, give the edges a domain: `let edges: [E](u: V, v: V) = ...`, then `Graph.directed(edges)`",
                    ),
                    (Ty::List(_), "indices") => self.help(
                        "a list is indexed by `Int` from 0: write `0..xs.len`; \
                         an array `[D]T` has `.indices`, the positions of `D`",
                    ),
                    (Ty::Tuple(parts), _) => self.help(format!(
                        "the parts of a tuple have numbers, not names: `.0` to `.{}`",
                        parts.len().saturating_sub(1)
                    )),
                    _ => {}
                }
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
                })
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
                    }
                }
            },
            _ => return None,
        };
        Some(plain.to_string())
    }

    /// Tries calling a free function or generic free function whose first parameter is `self`
    /// using UFCS (`base.name(args...)` -> `name(base, args...)`).
    pub(in crate::lower) fn try_ufcs_call(
        &mut self,
        name: &str,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let theirs = self.get_sym(&cpl_check::std_method(name));
        let mut sym = match self.get_sym(name).or(theirs) {
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
        // The program's own `find(self: List<Int>, ..)` is not what
        // `s.find(..)` on a `Str` means: that is the library's.
        if let Some(theirs) = theirs {
            let own = match sym {
                Sym::Fn(fid) => Some(self.sigs[fid].decl),
                Sym::GenFn(gfid) => Some(self.gen_fns[gfid].decl),
                _ => None,
            };
            let takes = own.and_then(|decl| {
                let first = decl.params.first().filter(|p| p.name.text == "self")?;
                self.written_shape(first.ty.as_ref()?, &decl.generics)
            });
            let given = self.probe_ty_silent(base).and_then(|ty| self.shape(&ty));
            if own.is_none()
                || matches!((takes, given), (Some(takes), Some(given)) if takes != given)
            {
                sym = theirs;
            }
        }
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
        let mut all_args = Vec::with_capacity(args.len() + 1);
        all_args.push(ast::Arg {
            name: None,
            mutable: self_mut,
            value: base.clone(),
        });
        all_args.extend(args.iter().cloned());
        match sym {
            Sym::Fn(fid) => Some(self.user_call(fid, None, &all_args, span)),
            Sym::GenFn(gfid) => Some(self.call_gen_fn(gfid, Vec::new(), None, &all_args, span)),
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
        if !matches!(
            self.probe_ty_silent(base),
            Some(
                Ty::List(_)
                    | Ty::ListView(_, _)
                    | Ty::Keyed(_, _)
                    | Ty::MutView(_)
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
            Ty::StrView | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(_)
        );
        let seq = if (view || !is_place(base))
            && !matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_))
        {
            let held = self.temp(seq.ty.clone());
            self.cx.pending.push(Stmt::Let(held, Some(seq)));
            self.local(held, e.span)
        } else {
            seq
        };
        self.items_of(&seq, e.span)
    }

    /// The indices and element type `seq.items` goes through; `None`
    /// when `seq` is not a list or an array of one dimension.
    pub(in crate::lower) fn items_of(&mut self, seq: &Expr, span: Span) -> Option<Items> {
        // A range's keys are its elements (R04.4.31): `(2..5).items` walks
        // `(2, 2) (3, 3) (4, 4)`.
        if matches!(&seq.ty, Ty::Range(of) if **of == Ty::Int) {
            let (lo, len) = self.range_lo_len(seq.clone(), span);
            let (index, elem, first, keyed) = (Ty::Int, Ty::Int, 0, false);
            let (seq, lo) = (seq.clone(), Some(lo));
            return Some(Items {
                seq,
                len,
                index,
                first,
                keyed,
                elem,
                lo,
            });
        }
        let (dim, elem) = match &seq.ty {
            Ty::List(elem) | Ty::MutView(elem) => (Dim::Int, (**elem).clone()),
            Ty::ListView(elem, key) | Ty::Keyed(elem, key) => (*key, (**elem).clone()),
            Ty::Str | Ty::StrView => (Dim::Int, Ty::Char),
            Ty::Array { dims, elem } if dims.len() == 1 => (dims[0], (**elem).clone()),
            _ => return None,
        };
        let keyed = Self::keyed_view(&seq.ty);
        // The loop counts the cells from 0; `[1..=9]T` names them from 1.
        let (len, index, first) = match dim {
            Dim::Pos(d) if keyed => (self.len_of(seq.clone(), span), Ty::Pos(d), 0),
            Dim::Pos(d) => (
                self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                Ty::Pos(d),
                0,
            ),
            Dim::Int => (self.len_of(seq.clone(), span), Ty::Int, 0),
            Dim::IntFrom(first) => (self.len_of(seq.clone(), span), Ty::Int, first),
            _ => return None,
        };
        Some(Items {
            seq: seq.clone(),
            len,
            index,
            first,
            keyed,
            elem,
            lo: None,
        })
    }

    /// `m.items` kept as a value: the list of `(key, value)`, in the
    /// order a `for` over the map goes.
    pub(in crate::lower) fn pairs_of_map(&mut self, map: Expr, span: Span) -> Expr {
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            unreachable!("the caller matched a map")
        };
        let pair = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
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
    pub(in crate::lower) fn proj_of_map(&mut self, map: Expr, idx: usize, span: Span) -> Expr {
        let (Ty::Map(key, held) | Ty::HashMap(key, held)) = &map.ty else {
            unreachable!("the caller matched a map")
        };
        let pair = Ty::Tuple(vec![(**key).clone(), (**held).clone()]);
        let elem = if idx == 0 {
            (**key).clone()
        } else {
            (**held).clone()
        };
        self.collected(elem.clone(), span, |lw, out, push| {
            let map = lw.hold(map, out);
            let each = lw.temp(pair);
            let proj = lw.mk(
                ExprKind::Field(Box::new(lw.local(each, span)), idx),
                elem.clone(),
                span,
            );
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
        let val = self.mk(
            ExprKind::Field(Box::new(self.local(each, pat.span)), proj.idx),
            proj.elem_ty.clone(),
            pat.span,
        );
        self.bind(pat, val, body);
        each
    }

    /// `g.items` on an array of two or more dimensions: every cell with
    /// the tuple of its indices, `((r, c), g[r, c])`, row by row.
    pub(in crate::lower) fn cells_of_grid(&mut self, grid: Expr, span: Span) -> Expr {
        let Ty::Array { dims, elem } = &grid.ty else {
            unreachable!("the caller matched an array")
        };
        let (dims, elem) = (dims.clone(), (**elem).clone());
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
            let cell = lw.mk(ExprKind::Index(Box::new(grid.clone()), stored), elem, span);
            let both = lw.mk(ExprKind::Tuple(vec![at, cell]), pair, span);
            let mut body = vec![push(lw, both)];
            for (axis, &k) in counters.iter().enumerate().rev() {
                let len = lw.mk(
                    ExprKind::Builtin(
                        Builtin::AxisLen,
                        vec![grid.clone(), lw.int_lit(axis as i64, span)],
                    ),
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
