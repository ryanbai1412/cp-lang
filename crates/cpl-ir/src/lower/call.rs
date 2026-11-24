//! Calls, methods and members: the user's fns and the ones the language
//! provides.

use super::*;
use cpl_diag::msgs;

mod collect;
mod field;
mod fold;
mod gen;
mod gen_take;
mod io_base;
mod laws;
pub(super) use gen::fixed_domain;
pub use laws::lower_sampling_laws;
pub(in crate::lower) use laws::Stress;
mod cuts;
mod items;
mod map_view;
mod mut_receiver;
mod prelude;
mod range_room;
mod rev;
mod seq;
mod struct_ops;
mod user;
mod value_method;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Reduce {
    Sum,
    Count,
    All,
    Any,
    Min,
    Max,
}

impl<'a> Lowerer<'a> {
    /// The range ends a struct's type arguments carry in this instance: a
    /// written `T` that is an `Int` with known ends gives its parameter those
    /// ends, without making the struct a type of its own.
    fn written_ranges(&self, params: &[ast::Name], written: &[ast::Type]) -> Vec<(String, GenArg)> {
        params
            .iter()
            .zip(written)
            .filter_map(|(p, t)| {
                let ast::TypeKind::Named { name, .. } = &t.kind else {
                    return None;
                };
                let ends = self.gen_env.get(&mono::range_ends_marker(&name.text))?;
                (self.gen_env.get(&name.text) == Some(&GenArg::Ty(Ty::Int)))
                    .then(|| (mono::range_ends_marker(&p.text), ends.clone()))
            })
            .collect()
    }

    /// `base.name(args)`.
    /// `G.m(..)` or `G<V>.m(..)` on a generic struct: the arguments not written
    /// (all of them, or the last ones) come from the hint, the call's
    /// arguments and the method's bounds.
    fn gen_struct_static(
        &mut self,
        gid: GenStructId,
        mut inferred: GenEnv,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let decl = self.gen_structs[gid].decl;
        let tpl_mod = self.gen_structs[gid].mod_id;
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
        if let Some(call) = self.list_form_call(gid, name, &[], args, span) {
            return call;
        }
        if let Some(Ty::Struct(hsid)) = hint {
            if let Some((hgid, hargs)) = &self.structs[*hsid].origin_gen {
                if *hgid == gid {
                    for (g, a) in gen_names.iter().zip(hargs) {
                        inferred.entry(g.clone()).or_insert_with(|| a.clone());
                    }
                }
            }
        }
        if inferred.len() < gen_names.len() {
            if let Some(mdecl) = decl.methods.iter().find_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) if f.name.text == name.text => Some(f),
                _ => None,
            }) {
                let mut all_gen_names = gen_names.clone();
                for g in &mdecl.generics {
                    if !all_gen_names.contains(&g.text) {
                        all_gen_names.push(g.text.clone());
                    }
                }
                for (p, arg) in mdecl.params.iter().zip(args) {
                    // `Lift.of((v: V) => ..)`: a lambda's written param types bind the
                    // generics its fn-typed param names, before any default fills them.
                    if let ast::ExprKind::Lambda { param_tys, .. } = &arg.value.kind {
                        if let Some(ast::Type {
                            kind:
                                ast::TypeKind::Fn {
                                    params: fn_params, ..
                                },
                            ..
                        }) = &p.ty
                        {
                            for (fp, written) in fn_params.iter().zip(param_tys) {
                                if let Some(written) = written {
                                    let ty = self.resolve_ty(written);
                                    self.unify_ast_ty(
                                        fp,
                                        &ty,
                                        tpl_mod,
                                        &all_gen_names,
                                        &mut inferred,
                                    );
                                }
                            }
                        }
                    } else if let Some(pty) = &p.ty {
                        self.bind_range_domain(
                            pty,
                            &arg.value,
                            &all_gen_names,
                            &mut inferred,
                            span,
                            None,
                        );
                        let probed = self.probe_expr(&arg.value, None);
                        if probed.ty != Ty::Error {
                            self.unify_ast_ty(
                                pty,
                                &probed.ty,
                                tpl_mod,
                                &all_gen_names,
                                &mut inferred,
                            );
                        }
                    }
                }
                // `S: Indexed<E, R>` binds R, then `R: (u: V, ..)` binds the struct's V.
                loop {
                    let before = inferred.len();
                    self.unify_bounds(&mdecl.bounds, tpl_mod, &all_gen_names, &mut inferred);
                    self.unify_bounds(&decl.bounds, tpl_mod, &all_gen_names, &mut inferred);
                    self.bind_impl_bounds(&mdecl.bounds, tpl_mod, &mut inferred);
                    if inferred.len() == before {
                        break;
                    }
                }
            }
        }
        let mut final_args = Vec::with_capacity(gen_names.len());
        for (i, g) in gen_names.iter().enumerate() {
            let arg = match (inferred.get(g), decl.generic_defaults.get(i)) {
                (Some(a), _) => a.clone(),
                (None, Some(Some(def_ty))) => {
                    let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                    let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                    let arg = self.resolve_gen_arg(def_ty);
                    self.gen_env = saved_env;
                    self.cur_mod = saved_mod;
                    inferred.insert(g.clone(), arg.clone());
                    arg
                }
                (None, _) => GenArg::Ty(Ty::Int),
            };
            final_args.push(arg);
        }
        let sid = self.instantiate_struct(gid, final_args);
        if let Some(fn_id) = self.own_method(sid, &name.text) {
            return self.method_call(fn_id, None, args, span);
        }
        if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
            return self.call_gen_fn(gfid, Vec::new(), None, args, span);
        }
        self.report(
            name.span,
            msgs::check::UNKNOWN_FIELD_HAS_NO_FN,
            &[&decl.name.text, &name.text],
        );
        self.err_expr(span)
    }

    fn method(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        if let [op] = args {
            let relower = |lw: &mut Self| lw.method(whole, base, name, args, hint);
            if let Some(checked) = self.checking_laws(base, op, span, relower) {
                return checked;
            }
        }
        if args.is_empty() && name.text == "parse" && self.parse_without_type(base, name, hint) {
            return self.err_expr(span);
        }
        self.intern_inline_product(base, span);
        if let Some(alias) = self.subsets_alias(base) {
            return self.method(whole, &alias, name, args, hint);
        }
        if let Some(taken) = self.take_from_generator(base, name, args, span) {
            return taken;
        }
        // `x.inv()`: a `@get` method reads the same with empty parentheses.
        if args.is_empty() {
            if let Some(Ty::Struct(id)) = self.probe_ty_silent(base) {
                if self.structs[id].getters.contains(&name.text) {
                    return self.field(whole, base, name, hint);
                }
            }
        }
        // `opt.map(f)` / `opt.filter(p)` on `?T`.
        if let ([arg], op @ ("map" | "filter")) = (args, name.text.as_str()) {
            if arg.name.is_none() && matches!(self.probe_ty_silent(base), Some(Ty::Opt(_))) {
                return self.lower_opt_map_or_filter(base, &arg.value, op == "map", span);
            }
        }
        // `a.inverse()`: partial injection `[D]?E -> [E]?D` or bijection `[D]E -> [E]D`.
        if args.is_empty() && name.text == "inverse" {
            if let Some(Ty::Array { dims, .. }) = self.probe_ty_silent(base) {
                if dims.len() == 1 {
                    return self.lower_array_inverse(base, span);
                }
            }
        }
        let is_seq_like = |ty: &Ty| ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_));
        let base_is_seq = self
            .probe_ty_silent(base)
            .as_ref()
            .is_some_and(&is_seq_like)
            || matches!(self.sym(base), Some(Sym::Domain(_)));
        // `xs.map(x => f(x))` returns `[D...]U` on fixed arrays `[D...]T`
        // (preserving index domains), and `List<U>` across every other sequence.
        if let ([func], "map") = (args, name.text.as_str()) {
            if func.name.is_none() && !func.mutable {
                if matches!(self.probe_ty_silent(base), Some(Ty::Array { .. })) {
                    return self.lower_array_map(base, &func.value, span);
                }
                if base_is_seq {
                    return self.lower_map(base, &func.value, span);
                }
            }
        }
        if name.text == "keyed" {
            if let Some(keyed) = self.lower_keyed(base, args, span) {
                return keyed;
            }
        }
        // `xs.filter(x => cond)` returns a `List<T>` of elements satisfying `cond`.
        if let ([test], "filter") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_filter(base, &test.value, span);
            }
        }
        // `xs.partition(x => cond)` returns `(List<T>, List<T>)`.
        if let ([test], "partition") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_partition(base, &test.value, span);
            }
        }
        // `xs.first(x => cond)` / `xs.last(x => cond)` returns `?T`.
        if let ([test], "first") = (args, name.text.as_str()) {
            if test.name.is_none() && base_is_seq {
                return self.lower_first_or_last_where(base, &test.value, true, span);
            }
        }
        if let ([test], "last") = (args, name.text.as_str()) {
            if test.name.is_none() && base_is_seq && self.is_test(&test.value) {
                return self.lower_first_or_last_where(base, &test.value, false, span);
            }
        }
        // `xs.reach(while: (a, b) => ...; cyclic: Bool = false)` returns `List<Int>`.
        if name.text == "reach" && base_is_seq {
            return self.lower_reach(base, args, span);
        }
        // `D.sort(...)`: sort all positions of domain `D` into a `List<D>`.
        if name.text == "sort" {
            if let Some(Sym::Domain(id)) = self.sym(base) {
                let list = self.positions(id, span);
                let list_ty = list.ty.clone();
                let elem = Ty::Pos(id);
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    out.push_let(slot, list);
                    let place = Place {
                        kind: PlaceKind::Local(slot),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    };
                    let sort_expr = lw.sort_place(place, &elem, args, span);
                    out.push(Stmt::Expr(sort_expr));
                    out.push_give(lw.local(slot, span));
                });
            }
        }
        // A fold over a sequence reads as its method: `xs.sum()` is
        // `sum(xs)`, `xs.count(x => x > 2)` is `count(xs, x => x > 2)`.
        let seq_ty = self.probe_ty_silent(base);
        if self.is_sorted_range_expr(base)
            || matches!(self.sym(base), Some(Sym::Domain(_)))
            || matches!(
                seq_ty,
                Some(
                    Ty::List(_)
                        | Ty::ListView(_, _)
                        | Ty::Keyed(_, _)
                        | Ty::MutView(..)
                        | Ty::Array { .. }
                        | Ty::Range(_)
                        | Ty::Set(_)
                        | Ty::HashSet(_)
                        | Ty::Str
                        | Ty::StrView
                )
            )
        {
            let seq = ast::Arg {
                name: None,
                mutable: false,
                value: base.clone(),
            };
            match (args, name.text.as_str()) {
                ([], "sum") => return self.reduce(Reduce::Sum, base, span),
                (_, "sorted")
                    if !matches!(
                        seq_ty,
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_, _)
                                | Ty::Keyed(_, _)
                                | Ty::MutView(..)
                                | Ty::Array { .. }
                        )
                    ) =>
                {
                    let value = match self.walked(base) {
                        Some(values) => self.expr(&values, None),
                        None => self.expr(base, None),
                    };
                    return if args.is_empty() {
                        self.sorted(value, span)
                    } else {
                        self.sorted_by(value, args, span)
                    };
                }
                ([], "min" | "max") if !matches!(seq_ty, Some(Ty::Set(_))) => {
                    return self.min_max_reduce(name.text == "max", base, true, span);
                }
                ([by_arg], "min" | "max")
                    if by_arg.name.as_ref().is_some_and(|n| n.text == "by") && !by_arg.mutable =>
                {
                    return self.min_max_by(name.text == "max", base, &by_arg.value, span);
                }
                ([test], "first") => {
                    if let Some(walk) = self.tested_walk("first", &[seq, test.clone()], true) {
                        return self.lower_first(&walk, span);
                    }
                }
                ([test], "count" | "all" | "any") => {
                    if let Some(walk) = self.tested_walk(&name.text, &[seq, test.clone()], false) {
                        let kind = match name.text.as_str() {
                            "count" => Reduce::Count,
                            "all" => Reduce::All,
                            _ => Reduce::Any,
                        };
                        return self.reduce(kind, &walk, span);
                    }
                }
                _ => {}
            }
        }
        // `sol.send(x)` and `sol.recv()` in an `@interactor`.
        if self.judge && matches!(name.text.as_str(), "send" | "recv") {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self
                    .lookup(n)
                    .is_some_and(|slot| self.cx.locals[slot].ty == Ty::Unit)
                {
                    return self.talk(name, args, hint, span);
                }
            }
        }
        // `[V].filled(x)`, `[n].filled(x)`: an array with every cell `x`.
        if let ast::ExprKind::List(index) = &base.kind {
            if name.text == "filled" && !index.is_empty() {
                return self.array_filled(index, args, hint, span);
            }
        }
        // `List.filled(n, val)`
        if let ast::ExprKind::Name(n) = &base.kind {
            if n == "List" && self.lookup(n).is_none() && name.text == "filled" {
                let Some(exprs) = self.positional_any("filled", args, 2, span) else {
                    return self.err_expr(span);
                };
                let count = self.expr_as(exprs[0], &Ty::Int);
                let val_hint = match hint {
                    Some(Ty::List(e)) => Some(&**e),
                    _ => None,
                };
                // Under a known element type the value is one of it:
                // `let a: List<Z> = List.filled(n, 0)` holds `Z(0)`.
                let val = match val_hint {
                    Some(elem) if !matches!(elem, Ty::Error) => {
                        let elem = elem.clone();
                        self.expr_as(exprs[1], &elem)
                    }
                    _ => self.expr(exprs[1], None),
                };
                let list_ty = Ty::list(val.ty.clone());
                return self.mk_builtin(Builtin::ListFilled, [count, val], list_ty, span);
            }
            if n == "List" && self.lookup(n).is_none() {
                return self.list_static_typo(name, span);
            }
        }
        // `D.nth(k)` where `D` is bound to `V.cuts`.
        if name.text == "nth" {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self.lookup(n).is_none() {
                    if let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(n).cloned() {
                        if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                            return self.mk(
                                ExprKind::Cast(Cast::Trust, Box::new(val)),
                                Ty::Bound(id),
                                span,
                            );
                        }
                        return self.err_expr(span);
                    }
                    // `D` bound to `Int` by `Range<Int>` arguments: positions are the `Int`s.
                    if let Some(GenArg::Ty(Ty::Int)) = self.gen_env.get(n) {
                        return self
                            .one_arg("nth", args, &Ty::Int, span)
                            .unwrap_or_else(|| self.err_expr(span));
                    }
                }
            }
        }
        // `D.cuts.nth(k)`
        if let ast::ExprKind::Field {
            base: dbase,
            name: fname,
        } = &base.kind
        {
            if matches!(fname.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "nth" | "at")
            {
                let dom_id = if let Some(Sym::Domain(id)) = self.sym(dbase) {
                    Some(id)
                } else if let ast::ExprKind::Name(n) = &dbase.kind {
                    if self.lookup(n).is_none() {
                        match self.gen_env.get(n) {
                            Some(GenArg::Domain(id)) | Some(GenArg::Ty(Ty::Bound(id))) => Some(*id),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    let dv = self.expr(dbase, None);
                    match dv.kind {
                        ExprKind::DomainVal(id) => Some(id),
                        _ => match dv.ty {
                            Ty::Pos(id) | Ty::Bound(id) => Some(id),
                            Ty::DomainVal => self.gen_env.values().find_map(|ga| match ga {
                                GenArg::Domain(id) => Some(*id),
                                _ => None,
                            }),
                            _ => None,
                        },
                    }
                };
                if let Some(id) = dom_id {
                    if name.text == "at" {
                        return match self.one_arg("at", args, &Ty::Int, span) {
                            Some(k) => self.domain_cut(id, k, false, span),
                            None => self.err_expr(span),
                        };
                    }
                    if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                        return self.mk(
                            ExprKind::Cast(Cast::Trust, Box::new(val)),
                            Ty::Bound(id),
                            span,
                        );
                    }
                    return self.err_expr(span);
                }
            }
        }
        // `GenericStruct<T>.method(...)` (e.g. `SegTree<V, Int>.from(...)`, `Graph<V, Int>.directed(...)`).
        if let ast::ExprKind::Generic {
            base: gbase,
            args: gtypes,
        } = &base.kind
        {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                let lookup = self.arity_name(gname, gtypes.len());
                if let Some(Sym::GenStruct(gid)) = self.get_sym(&lookup) {
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    let gargs = self.resolve_gen_args_for_struct(gid, gtypes);
                    let decl = self.gen_structs[gid].decl;
                    if gargs.len() < decl.generics.len() {
                        let seed = decl
                            .generics
                            .iter()
                            .map(|g| g.text.clone())
                            .zip(gargs)
                            .collect();
                        return self.gen_struct_static(gid, seed, name, args, hint, span);
                    }
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(fn_id) = self.own_method(sid, &name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                        self.owner_ranges = self.written_ranges(&decl.generics, gtypes);
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                    if self.structs[sid].index_domain.is_some() {
                        return self.library_domain_seq(base, sid, name, Some(args), span);
                    }
                    self.report_said(
                        name.span,
                        msgs::check::UNKNOWN_FIELD_HAS_NO_FN
                            .at(&[&self.structs[sid].def.name, &name.text]),
                    );
                    return self.err_expr(span);
                }
            }
        }
        if let Some(e) = self.builtin_type_call(base, name, args, span) {
            return e;
        }
        // `Tr.m(x)`: the `m` of `x`'s `impl Tr`.
        if let (Some(Sym::Trait(tid)), Some((first, rest))) = (self.sym(base), args.split_first()) {
            if first.name.is_none() {
                if let Some(Ty::Struct(sid)) = self.probe_ty_silent(&first.value) {
                    let key = (self.traits[tid].name.clone(), name.text.clone());
                    if let Some(&fn_id) = self.structs[sid].trait_methods.get(&key) {
                        let value = self.expr(&first.value, None);
                        return self.method_call(fn_id, Some((&first.value, value)), rest, span);
                    }
                }
            }
        }
        // `w.Idx.wrap(4)`: the domain the value `w` sized from its fields.
        let value_domain = match &base.kind {
            ast::ExprKind::Field {
                base: owner,
                name: member,
            } => self.value_domain_named(owner, &member.text),
            _ => None,
        };
        match value_domain.map(Sym::Domain).or_else(|| self.sym(base)) {
            Some(Sym::Mod(target_mod)) => {
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                return match sym {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::GenFn(gfid)) => self.call_gen_fn(gfid, Vec::new(), None, args, span),
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(gid)) => {
                        self.call_gen_struct(gid, Vec::new(), args, hint, span)
                    }
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    _ => {
                        self.report(
                            name.span,
                            msgs::ir::UNKNOWN_FIELD_MODULE_HAS_NO_FN,
                            &[&name.text],
                        );
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Struct(id)) => {
                if let Some(fn_id) = self.own_method(id, &name.text) {
                    let call = self.method_call(fn_id, None, args, span);
                    return match self.structs[id].index_domain {
                        Some(d) if name.text == "nth" => self.checked_nth(d, call, span),
                        _ => call,
                    };
                }
                if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                    return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                }
                if self.structs[id].index_domain.is_some() {
                    return self.library_domain_seq(base, id, name, Some(args), span);
                }
                self.report_said(
                    name.span,
                    msgs::check::UNKNOWN_FIELD_HAS_NO_FN
                        .at(&[&self.structs[id].def.name, &name.text]),
                );
                return self.err_expr(span);
            }
            Some(Sym::GenStruct(gid)) => {
                return self.gen_struct_static(gid, HashMap::new(), name, args, hint, span);
            }
            Some(Sym::Alias(aid)) => {
                let resolved = self.in_alias(aid, |lw, ty| lw.resolve_ty(ty));
                if let Ty::Struct(id) = resolved {
                    if let Some(fn_id) = self.own_method(id, &name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                }
            }
            Some(Sym::Domain(id))
                if let Some(tag) = self
                    .sum_enums
                    .get(&id)
                    .and_then(|info| info.variants.iter().position(|(v, _)| *v == name.text)) =>
            {
                let info = self.sum_enums[&id].clone();
                return self.variant(info.enum_id, tag, args, span);
            }
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) => self.variant(id, tag, args, span),
                    None if self.unit_enum(id) => self.enum_seq(base, id, name, Some(args), span),
                    None => {
                        let message = cpl_diag::msgs::ir::HAS_NO_VARIANT
                            .at(&[&self.enums[id].name, &name.text]);
                        self.report_said(name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::GenEnum(gid)) => {
                let id = match hint {
                    Some(Ty::Enum(eid)) => Some(*eid),
                    _ => {
                        let decl = self.gen_enums[gid].decl;
                        let tpl_mod = self.gen_enums[gid].mod_id;
                        let gen_names: Vec<String> =
                            decl.generics.iter().map(|g| g.text.clone()).collect();
                        let mut inferred: GenEnv = HashMap::new();
                        if let Some(variant) =
                            decl.variants.iter().find(|v| v.name.text == name.text)
                        {
                            for (f, arg) in variant.fields.iter().zip(args) {
                                let probed = self.probe_expr(&arg.value, None);
                                if probed.ty != Ty::Error {
                                    self.unify_ast_ty(
                                        &f.1,
                                        &probed.ty,
                                        tpl_mod,
                                        &gen_names,
                                        &mut inferred,
                                    );
                                }
                            }
                        }
                        if inferred.len() == gen_names.len() {
                            let gargs: Vec<GenArg> =
                                gen_names.iter().map(|g| inferred[g].clone()).collect();
                            Some(self.instantiate_enum(gid, gargs))
                        } else {
                            None
                        }
                    }
                };
                if let Some(id) = id {
                    return match self.enums[id]
                        .variants
                        .iter()
                        .position(|(v, _)| *v == name.text)
                    {
                        Some(tag) => self.variant(id, tag, args, span),
                        None => {
                            let message = cpl_diag::msgs::ir::HAS_NO_VARIANT
                                .at(&[&self.enums[id].name, &name.text]);
                            self.report_said(name.span, message);
                            self.err_expr(span)
                        }
                    };
                }
            }
            Some(Sym::Domain(id)) => match name.text.as_str() {
                "cuts" => {
                    return match self.one_arg("cuts", args, &Ty::Int, span) {
                        Some(k) => self.domain_cut(id, k, true, span),
                        None => self.err_expr(span),
                    };
                }
                "at" => return self.domain_at(id, args, span),
                "wrap" | "nth" | "try_at" => {
                    let space = super::ord_space::OrdSpace::Domain(id);
                    if let Some(found) = self.space_member(space, &name.text, args, span) {
                        return found;
                    }
                }
                "after" => {
                    return match self.one_arg("after", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_add(self.cast(Cast::Ord, pos, Ty::Int), 1);
                            let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::range(Ty::Pos(id)),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "before" => {
                    return match self.one_arg("before", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_lit(0, span);
                            let hi = self.cast(Cast::Ord, pos, Ty::Int);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::range(Ty::Pos(id)),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "pairs" if args.is_empty() => {
                    return self.domain_pairs_list(id, span);
                }
                _ => {}
            },
            _ => {}
        }
        // Check `lends` projection read before lowering `base` as a plain value.
        if let Some(proj) = self.find_projection_on(base, &name.text) {
            return self.lower_projection_read(proj, base, args, span);
        }
        let field = ast::ExprKind::Field {
            base: Box::new(base.clone()),
            name: name.clone(),
        };
        let op = self.detect_bare_op(&ast::Expr { kind: field, span });
        if let Some(op) = op.or_else(|| self.field_op(base, &name.text)) {
            if let Some(e) = self.bare_op_call(op, name.span, args, span, hint) {
                return e;
            }
        }
        let recv_mark = self.cx.pending.len();
        let value = self.expr(base, None);
        let value = match self.rev_method(value, name, args, span) {
            Ok(call) => return call,
            Err(value) => value,
        };
        let ty = value.ty.clone();
        let method = name.text.as_str();
        let value = match self.call_field(value, name, args, span, hint) {
            Ok(call) => return call,
            Err(value) => value,
        };
        if let Ty::Struct(id) = &ty {
            if method == "get" {
                if let Some(key_ty) = self.counter_key.get(id).cloned() {
                    let Some(key) = self.one_arg(method, args, &key_ty, span) else {
                        return self.err_expr(span);
                    };
                    let map_ty = self.structs[*id].def.fields[0].1.clone();
                    let map_expr = self.mk_field(value, 0, map_ty, span);
                    let found =
                        self.mk_builtin(Builtin::MapGet, [map_expr, key], Ty::opt(Ty::Int), span);
                    let zero = self.int_lit(0, span);
                    return self.mk_binary(BinOp::Coalesce, found, zero, Ty::Int, span);
                }
            }
            if let Some(fn_id) = self.own_method(*id, method) {
                return self.method_call(fn_id, Some((base, value)), args, span);
            }
            if let Some(fn_id) = self.open_impl_method(*id, method) {
                return self.method_call(fn_id, Some((base, value)), args, span);
            }
            if let Some(gfid) = self.structs[*id].gen_methods.get(method).copied() {
                let is_mut = self.gen_fns[gfid]
                    .decl
                    .params
                    .first()
                    .is_some_and(|p| p.name.text == "self" && p.mutable)
                    || self.structs[*id]
                        .decl
                        .is_some_and(|sd| super::cache::grows_cache(sd, self.gen_fns[gfid].decl));
                let recv = if is_mut {
                    Arg::Mut(self.mut_receiver(base, method))
                } else {
                    Arg::Value(value)
                };
                self.call_hint.0 = hint.cloned();
                return self.call_gen_fn(gfid, Vec::new(), Some(recv), args, span);
            }
        }
        // Members that read like fields can be called too: `xs.len()`.
        if args.is_empty()
            && matches!(
                method,
                "len" | "is_empty" | "first" | "last" | "items" | "keys" | "values"
            )
            && !matches!(ty, Ty::Struct(_))
        {
            return self.field(whole, base, name, hint);
        }
        let elem = match &ty {
            Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _)
            | Ty::MutView(elem, _)
            | Ty::Set(elem)
            | Ty::HashSet(elem)
            | Ty::Range(elem) => (**elem).clone(),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
            Ty::Str | Ty::StrView => Ty::Char,
            _ => Ty::Error,
        };
        self.value_method(whole, base, name, args, hint, value, ty, elem, recv_mark)
    }
}

/// Turns the `break` that ends an `all`/`any` early into one that leaves
/// the outermost loop of the comprehension.
fn retarget_break(stmts: &mut [Stmt], outer: LoopId, depth: usize) {
    for stmt in stmts {
        match stmt {
            Stmt::For(_, _, _, body) => retarget_break(&mut body.stmts, outer, depth + 1),
            Stmt::If(_, then, _) => retarget_break(&mut then.stmts, outer, depth),
            Stmt::Break(target @ None) if depth > 1 => *target = Some(outer),
            _ => {}
        }
    }
}

/// Whether `e` names somewhere a value lives: a variable, a field or an
/// element of one.
pub(super) fn is_place(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => true,
        ast::ExprKind::Paren(inner) => is_place(inner),
        ast::ExprKind::Field { base, .. } | ast::ExprKind::Index { base, .. } => is_place(base),
        _ => false,
    }
}

/// Makes every `?T` of `spec` read `absent` as `none`; false when it has no
/// `?T`.
pub(super) fn reads_none_as(spec: &mut Spec, absent: &Expr) -> bool {
    match spec {
        Spec::Fields(_, fields) => {
            let mut any = false;
            for field in fields {
                any |= reads_none_as(field, absent);
            }
            any
        }
        Spec::Array { elem, .. } | Spec::List(elem) => reads_none_as(elem, absent),
        Spec::Opt(_, slot) => {
            *slot = Some(Box::new(absent.clone()));
            true
        }
        _ => false,
    }
}

/// What `for .. in g.items` goes through on a grid: `total` cells, and a
/// counter per axis that says which one the loop is at.
pub(super) struct GridItems {
    grid: Expr,
    dims: Vec<Dim>,
    elem: Ty,
    lens: Vec<Expr>,
    total: Expr,
    counters: Vec<Slot>,
}

impl GridItems {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.total.span,
        };
        Iter::Range {
            lo,
            hi: self.total.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

pub(super) struct MapProj {
    map: Expr,
    pair_ty: Ty,
    elem_ty: Ty,
    idx: usize,
    rev: bool,
}

impl MapProj {
    pub(super) fn iter(&self) -> Iter {
        Iter::Seq(self.map.clone(), self.rev)
    }
}

/// What `xs.items` goes through: each of the `len` cells of `seq`, with
/// its index. An `Int` index starts at `first`.
pub(super) struct Items {
    seq: Expr,
    len: Expr,
    index: Ty,
    first: i64,
    /// The view names its cells by source keys, not walk positions.
    keyed: bool,
    elem: Ty,
    /// A range's first element: its cells are `lo + k`, not `seq[k]`.
    lo: Option<Expr>,
}

impl Items {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.len.span,
        };
        Iter::Range {
            lo,
            hi: self.len.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

impl<'a> Lowerer<'a> {
    pub(super) fn sorted_container_key(&self, sid: StructId) -> Option<Ty> {
        let info = &self.structs[sid];
        if !info.methods.contains_key("range_each") {
            return None;
        }
        let (_, gargs) = info.origin_gen.as_ref()?;
        match gargs.first()? {
            GenArg::Ty(k) => Some(k.clone()),
            GenArg::Domain(d) => Some(Ty::Pos(*d)),
            _ => None,
        }
    }

    /// The slot of the `b.range(r)` view named `n`, captured into the closure being lowered
    /// when a fn around it holds the view.
    fn sorted_view_slot(&mut self, n: &str) -> Option<Slot> {
        let is_view = |cx: &FnCx, slot: Slot| cx.sorted_views.contains_key(&slot);
        if let Some(slot) = self
            .lookup(n)
            .or_else(|| self.cx.captured_names.get(n).copied())
        {
            return is_view(&self.cx, slot).then_some(slot);
        }
        if !self.cx.is_closure {
            return None;
        }
        let outer = self.outer_cxs.iter().rev().find_map(|cx| {
            let slot = cx.scopes.iter().rev().find_map(|s| s.get(n).copied())?;
            Some(is_view(cx, slot))
        });
        if outer != Some(true) {
            return None;
        }
        self.capture_from_outer(n)
    }

    pub(super) fn is_sorted_range_expr(&mut self, mut e: &ast::Expr) -> bool {
        loop {
            match &e.kind {
                ast::ExprKind::Paren(inner) => e = inner,
                ast::ExprKind::Name(n) => {
                    return self.sorted_view_slot(n).is_some();
                }
                ast::ExprKind::Field { base, name } if name.text == "rev" => e = base,
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    if let ast::ExprKind::Field { base, name } = &callee.kind {
                        if name.text == "range" {
                            return match self.probe_ty_silent(base) {
                                Some(Ty::Struct(sid)) => self.sorted_container_key(sid).is_some(),
                                Some(Ty::Set(_) | Ty::Map(_, _)) => true,
                                _ => false,
                            };
                        }
                    }
                    return false;
                }
                _ => return false,
            }
        }
    }

    fn sorted_domain_bounds(
        &mut self,
        d: DomainId,
        lo_ord: Expr,
        hi_ord: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> (Expr, Expr, Expr, Expr, Option<Expr>) {
        let lo_s = self.temp(Ty::Int);
        out.push_let(lo_s, lo_ord);
        let hi_s = self.temp(Ty::Int);
        out.push_let(hi_s, hi_ord);
        let dlen_s = self.temp(Ty::Int);
        let dlen = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        out.push_let(dlen_s, dlen);

        let lo_l = self.local(lo_s, span);
        let hi_l = self.local(hi_s, span);
        let dlen_l = self.local(dlen_s, span);
        let zero = self.int_lit(0, span);

        let c1 = self.mk_cmp(CmpOp::Ge, lo_l.clone(), hi_l.clone(), Ty::Bool, span);
        let c2 = self.mk_cmp(CmpOp::Ge, lo_l.clone(), dlen_l.clone(), Ty::Bool, span);
        let c3 = self.mk_cmp(CmpOp::Le, hi_l.clone(), zero.clone(), Ty::Bool, span);
        let c12 = self.mk_binary(BinOp::Or, c1, c2, Ty::Bool, span);
        let empty = self.mk_binary(BinOp::Or, c12, c3, Ty::Bool, span);
        let empty_s = self.temp(Ty::Bool);
        out.push_let(empty_s, empty);
        let empty_l = self.local(empty_s, span);
        let not_empty = self.mk_unary(UnOp::Not, empty_l.clone(), Ty::Bool, span);

        let opt_k = Ty::opt(Ty::Pos(d));
        let lo_gt_zero = self.mk_cmp(CmpOp::Gt, lo_l.clone(), zero, Ty::Bool, span);
        let has_lo = self.mk_binary(BinOp::And, not_empty.clone(), lo_gt_zero, Ty::Bool, span);
        let lo_pos = self.cast(Cast::Trust, lo_l, Ty::Pos(d));
        let lo_some = self.mk_some(lo_pos, opt_k.clone(), span);
        let lo_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let lo_expr = self.mk_if(has_lo, lo_some, lo_none, opt_k.clone(), span);

        let hi_lt_len = self.mk_cmp(CmpOp::Lt, hi_l.clone(), dlen_l, Ty::Bool, span);
        let has_hi = self.mk_binary(BinOp::And, not_empty, hi_lt_len, Ty::Bool, span);
        let hi_pos = self.cast(Cast::Trust, hi_l, Ty::Pos(d));
        let hi_some = self.mk_some(hi_pos, opt_k.clone(), span);
        let hi_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let hi_expr = self.mk_if(has_hi, hi_some, hi_none, opt_k, span);

        let lo_inc = self.mk(ExprKind::Bool(true), Ty::Bool, span);
        let hi_inc = self.mk(ExprKind::Bool(false), Ty::Bool, span);
        (lo_expr, lo_inc, hi_expr, hi_inc, Some(empty_l))
    }

    pub(super) fn sorted_view_from_expr(
        &mut self,
        e: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<SortedView> {
        let mut cur = e;
        let mut flip = false;
        loop {
            match &cur.kind {
                ast::ExprKind::Paren(inner) => cur = inner,
                ast::ExprKind::Field { base, name } if name.text == "rev" => {
                    flip = !flip;
                    cur = base;
                }
                ast::ExprKind::Name(n) => {
                    let slot = self.sorted_view_slot(n)?;
                    let mut v = self.cx.sorted_views.get(&slot)?.clone();
                    if flip {
                        v.rev = !v.rev;
                    }
                    return Some(v);
                }
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    let ast::ExprKind::Field { base, name } = &callee.kind else {
                        return None;
                    };
                    if name.text != "range" {
                        return None;
                    }
                    let (kind, key_ty, elem_ty) = match self.probe_ty_silent(base) {
                        Some(Ty::Struct(sid)) => {
                            let k = self.sorted_container_key(sid)?;
                            (SortedViewKind::Bag(sid), k.clone(), k)
                        }
                        Some(Ty::Set(k)) => (SortedViewKind::Set, (*k).clone(), *k),
                        Some(Ty::Map(k, v)) => {
                            let kt = (*k).clone();
                            let et = Ty::Tuple(vec![*k, *v]);
                            (SortedViewKind::Map, kt, et)
                        }
                        _ => return None,
                    };
                    let span = cur.span;
                    let raw_place = self.held_place(base, out);
                    let place = self.settle_indexes(raw_place, out);
                    let arg = &args[0].value;
                    let opt_k = Ty::opt(key_ty.clone());

                    let (lo_expr, lo_inc_expr, hi_expr, hi_inc_expr, empty_expr) = match &arg.kind {
                        ast::ExprKind::Range {
                            lo,
                            hi,
                            inclusive,
                            step: None,
                        } => {
                            let domain_ords = if let Ty::Pos(d) = key_ty {
                                let lo_is_bound = lo
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                let hi_is_bound = hi
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                (lo_is_bound || hi_is_bound).then_some(d)
                            } else {
                                None
                            };
                            if let Some(d) = domain_ords {
                                let lo_ord = match lo.as_deref() {
                                    Some(l) => {
                                        let le = self.expr(l, None);
                                        self.cast(Cast::Ord, le, Ty::Int)
                                    }
                                    None => self.int_lit(0, span),
                                };
                                let hi_ord = match hi.as_deref() {
                                    Some(h) => {
                                        let he = self.expr(h, None);
                                        let is_pos = he.ty == Ty::Pos(d);
                                        let ho = self.cast(Cast::Ord, he, Ty::Int);
                                        if is_pos && *inclusive {
                                            self.int_add(ho, 1)
                                        } else {
                                            ho
                                        }
                                    }
                                    None => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                };
                                self.sorted_domain_bounds(d, lo_ord, hi_ord, span, out)
                            } else {
                                let lo_e = match lo.as_deref() {
                                    Some(l) => {
                                        let lv = self.expr_as(l, &key_ty);
                                        self.mk_some(lv, opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                let hi_e = match hi.as_deref() {
                                    Some(h) => {
                                        let hv = self.expr_as(h, &key_ty);
                                        self.mk_some(hv, opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                (
                                    lo_e,
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    hi_e,
                                    self.mk(ExprKind::Bool(*inclusive), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                        _ => {
                            let want_r = Ty::range(key_ty.clone());
                            let rv = self.expr(arg, Some(&want_r));
                            let r_slot = self.temp(rv.ty.clone());
                            out.push_let(r_slot, rv);
                            let r_l = self.local(r_slot, span);
                            let r_lo =
                                self.mk_builtin(Builtin::RangeLo, [r_l.clone()], Ty::Int, span);
                            let r_hi = self.mk_builtin(Builtin::RangeHi, [r_l], Ty::Int, span);
                            if let Ty::Pos(d) = key_ty {
                                self.sorted_domain_bounds(d, r_lo, r_hi, span, out)
                            } else {
                                let r_lo = self.coerce(r_lo, &key_ty, span);
                                let r_hi = self.coerce(r_hi, &key_ty, span);
                                (
                                    self.mk_some(r_lo, opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    self.mk_some(r_hi, opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(false), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                    };

                    let lo_s = self.view_bound(lo_expr, out);
                    let lo_inc_s = self.view_bound(lo_inc_expr, out);
                    let hi_s = self.view_bound(hi_expr, out);
                    let hi_inc_s = self.view_bound(hi_inc_expr, out);
                    let empty_s = empty_expr.map(|ee| self.view_bound(ee, out));
                    return Some(SortedView {
                        base: place,
                        kind,
                        key_ty,
                        elem_ty,
                        lo: lo_s,
                        lo_inc: lo_inc_s,
                        hi: hi_s,
                        hi_inc: hi_inc_s,
                        empty: empty_s,
                        rev: flip,
                    });
                }
                _ => return None,
            }
        }
    }

    /// A bound of a view as each of its uses reads it: a constant as it is,
    /// so `b.range(2..)` passes `Some(2), true, None, false` straight to the
    /// one shared walk, and any other value once, in a local.
    fn view_bound(&mut self, e: Expr, out: &mut Vec<Stmt>) -> Expr {
        fn constant(e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Char(_) | ExprKind::None => true,
                ExprKind::Some(inner) | ExprKind::Cast(_, inner) => constant(inner),
                ExprKind::Unary(UnOp::Neg, inner) => constant(inner),
                _ => false,
            }
        }
        if constant(&e) || matches!(e.kind, ExprKind::Local(_)) && e.ty == Ty::Bool {
            return e;
        }
        let span = e.span;
        let slot = self.temp(e.ty.clone());
        out.push_let(slot, e);
        self.local(slot, span)
    }

    pub(super) fn call_sorted_view(
        &mut self,
        view: &SortedView,
        sid: StructId,
        method: &str,
        ret_ty: Ty,
        empty_default: Expr,
        span: Span,
    ) -> Expr {
        let Some(fn_id) = self.struct_op_fn(sid, method) else {
            return empty_default;
        };
        let args = vec![
            Arg::Value(super::stmt::place_to_expr(&view.base)),
            Arg::Value(view.lo.clone()),
            Arg::Value(view.lo_inc.clone()),
            Arg::Value(view.hi.clone()),
            Arg::Value(view.hi_inc.clone()),
        ];
        let call = self.mk(ExprKind::Call(fn_id, args), ret_ty.clone(), span);
        if let Some(emp) = &view.empty {
            let cond = emp.clone();
            self.mk_if(cond, empty_default, call, ret_ty, span)
        } else {
            call
        }
    }

    fn tree_view_key(&mut self, view: &SortedView, val: Expr, span: Span) -> Expr {
        match view.kind {
            SortedViewKind::Map => self.mk_field(val, 0, view.key_ty.clone(), span),
            _ => val,
        }
    }

    /// Filter an optional candidate `cand` against the upper bound (`hi`, `hi_inc`)
    /// when `check_hi`, or the lower bound (`lo`, `lo_inc`) when `!check_hi`.
    fn tree_view_clip(
        &mut self,
        view: &SortedView,
        cand: Expr,
        check_hi: bool,
        span: Span,
    ) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let bound = if check_hi { &view.hi } else { &view.lo }.clone();
        let inc = if check_hi { &view.hi_inc } else { &view.lo_inc }.clone();
        self.do_expr(Some(&opt_e), span, |lw, out| {
            let cs = lw.temp(opt_e.clone());
            out.push_let(cs, cand);
            let c_ref = lw.local(cs, span);
            let c_some = lw.mk_is_some(c_ref.clone(), Ty::Bool, span);
            let b_ref = bound;
            let b_some = lw.mk_is_some(b_ref.clone(), Ty::Bool, span);
            let both = lw.mk_binary(BinOp::And, c_some, b_some, Ty::Bool, span);
            let elem_val = lw.mk_unwrap(c_ref.clone(), false, view.elem_ty.clone(), span);
            let k_val = lw.tree_view_key(view, elem_val, span);
            let b_val = lw.mk_unwrap(b_ref, false, view.key_ty.clone(), span);
            let (strict_op, inc_op) = if check_hi {
                (CmpOp::Lt, CmpOp::Le)
            } else {
                (CmpOp::Gt, CmpOp::Ge)
            };
            let inc_cmp = lw.mk_cmp(inc_op, k_val.clone(), b_val.clone(), Ty::Bool, span);
            let strict_cmp = lw.mk_cmp(strict_op, k_val, b_val, Ty::Bool, span);
            let in_bounds = lw.mk_if(inc, inc_cmp, strict_cmp, Ty::Bool, span);
            let keep = lw.mk_if(
                in_bounds,
                c_ref.clone(),
                lw.mk(ExprKind::None, opt_e.clone(), span),
                opt_e.clone(),
                span,
            );
            let res = lw.mk_if(both, keep, c_ref, opt_e.clone(), span);
            out.push_give(res);
        })
    }

    fn tree_view_end(&mut self, view: &SortedView, want_last: bool, span: Span) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let none_e = self.mk(ExprKind::None, opt_e.clone(), span);
        let body = self.do_expr(Some(&opt_e), span, |lw, out| {
            let base_e = super::stmt::place_to_expr(&view.base);
            let (start_slot, start_inc, end_op, ge_op, gt_op, check_hi) = if want_last {
                (
                    view.hi.clone(),
                    view.hi_inc.clone(),
                    Builtin::SetMax,
                    Builtin::LastLe,
                    Builtin::LastLt,
                    false,
                )
            } else {
                (
                    view.lo.clone(),
                    view.lo_inc.clone(),
                    Builtin::SetMin,
                    Builtin::FirstGe,
                    Builtin::FirstGt,
                    true,
                )
            };
            let s_ref = start_slot;
            let s_some = lw.mk_is_some(s_ref.clone(), Ty::Bool, span);
            let s_val = lw.mk_unwrap(s_ref, false, view.key_ty.clone(), span);
            let call_ge =
                lw.mk_builtin(ge_op, [base_e.clone(), s_val.clone()], opt_e.clone(), span);
            let call_gt = lw.mk_builtin(gt_op, [base_e.clone(), s_val], opt_e.clone(), span);
            let bounded = lw.mk_if(start_inc, call_ge, call_gt, opt_e.clone(), span);
            let open = lw.mk_builtin(end_op, [base_e], opt_e.clone(), span);
            let cand = lw.mk_if(s_some, bounded, open, opt_e.clone(), span);
            let clipped = lw.tree_view_clip(view, cand, check_hi, span);
            out.push_give(clipped);
        });
        if let Some(emp) = &view.empty {
            self.mk_if(emp.clone(), none_e, body, opt_e, span)
        } else {
            body
        }
    }

    fn tree_view_collect(&mut self, view: &SortedView, span: Span) -> Expr {
        let list_ty = Ty::list(view.elem_ty.clone());
        let opt_e = Ty::opt(view.elem_ty.clone());
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let out_s = lw.temp(list_ty.clone());
            out.push_let(
                out_s,
                lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span),
            );
            let cur_s = lw.temp(opt_e.clone());
            let first = lw.tree_view_end(view, view.rev, span);
            out.push_let(cur_s, first);
            let cur_ref = lw.local(cur_s, span);
            let cond = lw.mk_is_some(cur_ref.clone(), Ty::Bool, span);
            let v_s = lw.temp(view.elem_ty.clone());
            let v_init = lw.mk_unwrap(cur_ref, false, view.elem_ty.clone(), span);
            let v_ref = lw.local(v_s, span);
            let push_stmt = Stmt::Expr(lw.mk_mutate(
                Mutation::Push,
                Place {
                    kind: PlaceKind::Local(out_s),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                },
                vec![v_ref.clone()],
                Ty::Unit,
                span,
            ));
            let k_val = lw.tree_view_key(view, v_ref, span);
            let step_op = if view.rev {
                Builtin::LastLt
            } else {
                Builtin::FirstGt
            };
            let next_raw = lw.mk_builtin(
                step_op,
                [super::stmt::place_to_expr(&view.base), k_val],
                opt_e.clone(),
                span,
            );
            let next_clipped = lw.tree_view_clip(view, next_raw, !view.rev, span);
            let step_stmt = Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(cur_s),
                    ty: opt_e.clone(),
                    span: lw.span(span),
                },
                AssignOp::Set,
                next_clipped,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::While(
                lid,
                cond,
                Block {
                    stmts: vec![Stmt::Let(v_s, Some(v_init)), push_stmt, step_stmt],
                },
            ));
            out.push_give(lw.local(out_s, span));
        })
    }

    fn sorted_view_field(&mut self, base: &ast::Expr, field: &str, span: Span) -> Expr {
        let mut res_ty = Ty::Error;
        let mut e = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(base, out) else {
                out.push_give(lw.err_expr(span));
                return;
            };
            let opt_e = Ty::opt(view.elem_ty.clone());
            let res = match view.kind {
                SortedViewKind::Bag(sid) => match field {
                    "first" => {
                        let m = if view.rev {
                            "range_last"
                        } else {
                            "range_first"
                        };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "last" => {
                        let m = if view.rev {
                            "range_first"
                        } else {
                            "range_last"
                        };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "len" => {
                        let d = lw.int_lit(0, span);
                        lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span)
                    }
                    "is_empty" => {
                        let d = lw.int_lit(0, span);
                        let len = lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span);
                        let zero = lw.int_lit(0, span);
                        lw.mk_cmp(CmpOp::Eq, len, zero, Ty::Bool, span)
                    }
                    _ => lw.err_expr(span),
                },
                SortedViewKind::Set | SortedViewKind::Map => match field {
                    "first" => lw.tree_view_end(&view, view.rev, span),
                    "last" => lw.tree_view_end(&view, !view.rev, span),
                    "is_empty" => {
                        let first = lw.tree_view_end(&view, false, span);
                        let some = lw.mk_is_some(first, Ty::Bool, span);
                        lw.mk_unary(UnOp::Not, some, Ty::Bool, span)
                    }
                    _ => lw.err_expr(span),
                },
            };
            res_ty = res.ty.clone();
            out.push_give(res);
        });
        e.ty = res_ty;
        match e.kind {
            ExprKind::Do(Block { stmts }) => match <[Stmt; 1]>::try_from(stmts) {
                Ok([Stmt::Give(res)]) => res,
                Ok([stmt]) => Expr {
                    kind: ExprKind::Do(Block { stmts: vec![stmt] }),
                    ..e
                },
                Err(stmts) => Expr {
                    kind: ExprKind::Do(Block { stmts }),
                    ..e
                },
            },
            _ => e,
        }
    }

    pub(super) fn sorted_view_iter(&mut self, e: &ast::Expr, span: Span) -> Option<(Iter, Ty)> {
        if !self.is_sorted_range_expr(e) {
            return None;
        }
        // Zero-allocation direct iterator walk for `Set.range(r)` / `Map.range(r)`
        // when `r` is a syntactic range literal (`a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..`).
        let mut cur = e;
        let mut rev = false;
        while let ast::ExprKind::Field { base, name } = &cur.kind {
            if name.text == "rev" {
                rev = !rev;
                cur = base;
            } else {
                break;
            }
        }
        if let ast::ExprKind::Call { callee, args } = &cur.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if name.text == "range" && args.len() == 1 {
                    let base_ty = self.probe_ty_silent(base);
                    if let Some((key_ty, elem_ty)) = match base_ty {
                        Some(Ty::Set(k)) => Some(((*k).clone(), *k)),
                        Some(Ty::Map(k, v)) => {
                            let kt = *k;
                            let et = Ty::Tuple(vec![kt.clone(), *v]);
                            Some((kt, et))
                        }
                        _ => None,
                    } {
                        if let ast::ExprKind::Range {
                            lo, hi, inclusive, ..
                        } = &args[0].value.kind
                        {
                            let container = self.expr(base, None);
                            let lo = lo.as_ref().map(|x| (self.expr_as(x, &key_ty), true));
                            let hi = hi.as_ref().map(|x| (self.expr_as(x, &key_ty), *inclusive));
                            return Some((
                                Iter::TreeRange {
                                    container,
                                    lo,
                                    hi,
                                    rev,
                                },
                                elem_ty,
                            ));
                        }
                    }
                }
            }
        }
        let mut elem_ty = Ty::Error;
        let mut seq_ty = Ty::Error;
        let mut seq = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(e, out) else {
                out.push_give(lw.err_expr(span));
                return;
            };
            elem_ty = view.elem_ty.clone();
            let list_ty = Ty::list(view.elem_ty.clone());
            let res = match view.kind {
                SortedViewKind::Bag(sid) => {
                    let m = if view.rev {
                        "range_each_rev"
                    } else {
                        "range_each"
                    };
                    let empty_list = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    lw.call_sorted_view(&view, sid, m, list_ty.clone(), empty_list, span)
                }
                SortedViewKind::Set | SortedViewKind::Map => lw.tree_view_collect(&view, span),
            };
            seq_ty = list_ty;
            out.push_give(res);
        });
        seq.ty = seq_ty;
        Some((Iter::Seq(seq, false), elem_ty))
    }
}
