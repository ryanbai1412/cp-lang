//! Calls to generic functions and methods: inferring omitted type
//! arguments, binding range domains, and local generic fns.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Calls a generic function or generic struct method, inferring any omitted type arguments
    /// via two-pass unification (non-lambda arguments first, then lambda arguments).
    pub(in crate::lower) fn call_gen_fn(
        &mut self,
        gfid: GenFnId,
        explicit: Vec<GenArg>,
        receiver: Option<Arg>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let (hint, want) = std::mem::take(&mut self.call_hint);
        let owner_ranges = std::mem::take(&mut self.owner_ranges);
        let tpl_mod = self.gen_fns[gfid].mod_id;
        let tpl_offset = self.gen_fns[gfid].offset;
        let owner_env = self.gen_fns[gfid].owner_env.clone();
        let decl = self.gen_fns[gfid].decl;
        let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();

        // Map call args to AST param definitions (skipping `self` if method).
        let ast_params: Vec<&ast::Param> = decl
            .params
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                !(*i == 0 && p.name.text == "self" && self.gen_fns[gfid].owner.is_some())
            })
            .map(|(_, p)| p)
            .collect();

        let thunked = thunk_lazy_args(&ast_params, args);
        let args = thunked.as_deref().unwrap_or(args);
        let packed = pack_args(&ast_params, args, span);
        let args = packed.as_deref().unwrap_or(args);
        let closed = self.gen_fn_values_as_lambdas(&ast_params, args);
        let args = closed.as_deref().unwrap_or(args);

        let mut inferred = self.seed_gens(owner_env.clone(), &explicit, decl, want);

        // Match call args to ast_params.
        let mut matched: Vec<Option<&ast::Arg>> = vec![None; ast_params.len()];
        let mut next = 0;
        for arg in args {
            let idx = match &arg.name {
                Some(n) => ast_params.iter().position(|p| p.label_text() == n.text),
                None => {
                    while next < ast_params.len() && matched[next].is_some() {
                        next += 1;
                    }
                    (next < ast_params.len()).then_some(next)
                }
            };
            if let Some(i) = idx {
                matched[i] = Some(arg);
            }
        }

        // If a UFCS call passes a concrete `DomainVal(d)` as receiver (e.g. `D.prefix_range(4)`),
        // bind the first param's generic domain name (`self: D`) to `GenArg::Domain(d)`.
        if let Some(Arg::Value(Expr {
            kind: ExprKind::DomainVal(d),
            ..
        })) = &receiver
        {
            if let Some(first_param) = decl.params.first() {
                if let Some(ast::Type {
                    kind:
                        ast::TypeKind::Named {
                            name: pname,
                            args: targs,
                        },
                    ..
                }) = &first_param.ty
                {
                    if targs.is_empty() && gen_names.contains(&pname.text) {
                        inferred
                            .entry(pname.text.clone())
                            .or_insert(GenArg::Domain(*d));
                    }
                }
            }
        }

        for (param, arg) in ast_params.iter().zip(&matched) {
            if let (Some(pty), Some(arg)) = (&param.ty, arg) {
                self.bind_range_domain(
                    pty,
                    &arg.value,
                    &gen_names,
                    &mut inferred,
                    span,
                    Some((gfid, decl.ret.as_ref())),
                );
            }
        }

        // Pass 1: unify from non-lambda arguments (and detect bare operators passed for unbound type params).
        let mut op_params: Vec<(String, BareOp)> = Vec::new();
        let mut waiting_ops = Vec::new();
        for (param, arg) in ast_params.iter().zip(&matched) {
            // An omitted `over: M = (+)` binds `M` to the operator its default names.
            if let (Some(ast::TypeKind::Named { name, args }), None, Some(default)) =
                (param.ty.as_ref().map(|t| &t.kind), arg, &param.default)
            {
                if let Some(op) = (args.is_empty() && gen_names.contains(&name.text))
                    .then(|| self.detect_bare_op(default))
                    .flatten()
                {
                    inferred.entry(name.text.clone()).or_insert(GenArg::Op(op));
                } else if args.is_empty()
                    && gen_names.contains(&name.text)
                    && !inferred.contains_key(&name.text)
                    && !matches!(default.kind, ast::ExprKind::Lambda { .. })
                {
                    // An omitted `op: M = NoOp()` binds `M` before the bounds, so `M.Fold` resolves.
                    let saved = (
                        std::mem::replace(&mut self.cur_mod, tpl_mod),
                        std::mem::replace(&mut self.offset, tpl_offset),
                        std::mem::replace(&mut self.gen_env, inferred.clone()),
                    );
                    let probed = self.probe_ty_silent(default);
                    (self.cur_mod, self.offset, self.gen_env) = saved;
                    if let Some(dty) = probed.filter(|t| *t != Ty::Error) {
                        inferred.insert(name.text.clone(), GenArg::Ty(dty));
                    }
                }
            }
            let (Some(pty), Some(arg)) = (&param.ty, arg) else {
                continue;
            };
            if matches!(pty.kind, ast::TypeKind::Fn { .. }) {
                if let Some(op) = self.detect_bare_op(&arg.value) {
                    op_params.push((param.name.text.clone(), op));
                }
            }
            // `op: ?M = None` binds `M` to a bare operator as `op: M` does.
            let bare_ty = match &pty.kind {
                ast::TypeKind::Optional(inner) => inner,
                _ => pty,
            };
            if let ast::TypeKind::Named { name, args: targs } = &bare_ty.kind {
                if targs.is_empty()
                    && gen_names.contains(&name.text)
                    && !inferred.contains_key(&name.text)
                {
                    if let Some(g) = self.item_arg_for(&decl.bounds, &name.text, &arg.value) {
                        inferred.insert(name.text.clone(), g);
                        continue;
                    }
                    // `go(+, [2, 3])` with `M: Op<E>`: `+` takes `Monoid<E>` once
                    // a later argument gives `E`.
                    if self.monoid_bare_op(&arg.value).is_some()
                        && self.bound_has_monoid_impl(&decl.bounds, &name.text)
                        && self
                            .monoid_impl_shape(
                                &decl.bounds,
                                &name.text,
                                (tpl_mod, tpl_offset),
                                &inferred,
                            )
                            .is_none()
                    {
                        waiting_ops.push((pty, &arg.value, name.text.clone()));
                        continue;
                    }
                    let tpl = (tpl_mod, tpl_offset, &inferred);
                    if let Some(g) = self.bare_op_for(&decl.bounds, &name.text, &arg.value, tpl) {
                        inferred.insert(name.text.clone(), g);
                        continue;
                    }
                    // A domain passed as a value is the walk of its positions:
                    // `walk(R)` binds `A` to `Range<R>`, not to one position.
                    if let Some(Sym::Domain(d)) = self.sym(&arg.value) {
                        inferred.insert(
                            name.text.clone(),
                            GenArg::Ty(Ty::Range(Box::new(Ty::Pos(d)))),
                        );
                        continue;
                    }
                }
            }
            if matches!(arg.value.kind, ast::ExprKind::Lambda { .. })
                || matches!(&arg.value.kind, ast::ExprKind::Name(n) if matches!(n.as_str(), "min" | "max"))
            {
                self.unify_lambda_params(pty, &arg.value, tpl_mod, &gen_names, &mut inferred);
                continue;
            }
            self.call_hint.1 = self.want_bound(pty, decl, tpl_mod, tpl_offset, &inferred);
            if let Some(concrete) = self
                .probe_expr_ty(&arg.value, pty, tpl_mod, tpl_offset, &gen_names, &inferred)
                .map(|ty| if arg.mutable { ty.lent_mut() } else { ty })
            {
                if concrete == Ty::DomainVal {
                    let dom_id = if let Some(Sym::Domain(d)) = self.sym(&arg.value) {
                        Some(d)
                    } else {
                        let ev = self.probe_expr(&arg.value, None);
                        if let ExprKind::DomainVal(d) = ev.kind {
                            Some(d)
                        } else {
                            None
                        }
                    };
                    if let Some(d) = dom_id {
                        if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                            if targs.is_empty() && gen_names.contains(&name.text) {
                                inferred
                                    .entry(name.text.clone())
                                    .or_insert(GenArg::Ty(Ty::Range(Box::new(Ty::Pos(d)))));
                                continue;
                            }
                            if name.text == "List" {
                                self.unify_ast_ty(
                                    pty,
                                    &Ty::list(Ty::Pos(d)),
                                    tpl_mod,
                                    &gen_names,
                                    &mut inferred,
                                );
                                continue;
                            }
                            if name.text == "Range" {
                                self.unify_ast_ty(
                                    pty,
                                    &Ty::range(Ty::Pos(d)),
                                    tpl_mod,
                                    &gen_names,
                                    &mut inferred,
                                );
                                continue;
                            }
                        }
                    }
                }
                self.unify_arg(
                    decl,
                    pty,
                    &concrete,
                    arg,
                    tpl_mod,
                    &gen_names,
                    &mut inferred,
                );
                // If this argument bound a `Seq`/`Indexed`/`IndexedMut` container generic
                // (e.g. `resized<S: Indexed, T>(self: S, n: Int, fill: T)`), deduce `T` now so
                // subsequent arguments (like `0` for `Mod<M>`) get the element type as hint.
                if gen_names.iter().any(|g| g == "T") && !inferred.contains_key("T") {
                    let elem_ty = decl.bounds.iter().find_map(|b| {
                        // `A: Indexed<K, U>` names its element `U`, not `T`.
                        let is_seq_bound = b.traits.iter().any(|tr| {
                            matches!(&tr.kind, ast::TypeKind::Named { name, args }
                                if matches!(name.text.as_str(), "Seq" | "Keyed" | "Bidir" | "Positional" | "Indexed" | "IndexedMut")
                                    && args.last().is_none_or(|a| ty_mentions(a, "T")))
                        });
                        if !is_seq_bound {
                            return None;
                        }
                        // `S: Seq<?T>` names the element `?T`: unify against it, not `T` itself.
                        let pat = b.traits.iter().find_map(|tr| match &tr.kind {
                            ast::TypeKind::Named { args, .. } => args.last(),
                            _ => None,
                        });
                        let elem = match inferred.get(&b.name.text).cloned() {
                            Some(GenArg::Ty(t)) => self.walk_elem(&t, span),
                            Some(GenArg::Domain(d)) => Some(Ty::Pos(d)),
                            _ => None,
                        };
                        elem.map(|e| (e, pat))
                    });
                    let (elem_ty, mut pat) = elem_ty.map_or((None, None), |(e, p)| (Some(e), p));
                    // `flatten<S: Seq, U: Seq, T>(self: S)`: a `Seq` generic no parameter
                    // names is the element of `S`, and `T` is the element of that.
                    let inner = decl.bounds.iter().find(|b| {
                        !inferred.contains_key(&b.name.text)
                            && b.traits.iter().any(|tr| matches!(&tr.kind, ast::TypeKind::Named { name, .. } if name.text == "Seq"))
                            && !ast_params.iter().any(|p| p.ty.as_ref().is_some_and(|t| ty_mentions(t, &b.name.text)))
                    });
                    let elem_ty = match (inner, elem_ty) {
                        (Some(u), Some(ety)) => {
                            pat = None;
                            let inner_elem = ety.seq_elem();
                            inferred.insert(u.name.text.clone(), GenArg::Ty(ety));
                            inner_elem
                        }
                        (_, ety) => ety,
                    };
                    let bound = elem_ty.is_some();
                    match (elem_ty, pat) {
                        (Some(ety), Some(p)) => {
                            self.unify_ast_ty(p, &ety, tpl_mod, &gen_names, &mut inferred)
                        }
                        (Some(ety), None) => {
                            inferred.insert("T".to_string(), GenArg::Ty(ety));
                        }
                        _ => {}
                    }
                    if bound {
                        self.bind_elem_range(decl, arg, pat, &gen_names, &mut inferred);
                    }
                }
            }
        }

        if let Some(sid) = self.gen_fns[gfid].owner {
            for (_, _, param) in &waiting_ops {
                self.bind_op_from_owner(sid, decl, param, &owner_env, &mut inferred);
            }
        }
        for (pty, value, param) in waiting_ops {
            let tpl = (tpl_mod, tpl_offset, &inferred);
            match self.bare_op_for(&decl.bounds, &param, value, tpl) {
                Some(g) => {
                    inferred.insert(param, g);
                }
                None => self.unify_lambda_params(pty, value, tpl_mod, &gen_names, &mut inferred),
            }
        }
        // Propagate bindings implied by row/trait bounds (e.g. `R: (u: V, v: V, ..)`,
        // `G: Next<V>`).
        self.unify_bounds(&decl.bounds, tpl_mod, &gen_names, &mut inferred);
        self.fn_graph_vertex_domains(&decl.bounds, tpl_mod, &mut inferred);
        let tpl = (tpl_mod, tpl_offset, explicit.is_empty());
        self.infer_yielded(decl, tpl, &gen_names, &mut inferred);

        // Propagate domain param `V` from a bound `Graph<V, E>` (or any struct with a domain param),
        // and arc element type `A` from a bound `next: N` when `N` is a `Graph<V, E>` or fn.
        for g in &gen_names {
            if !inferred.contains_key(g) && (g == "V" || g == "D") {
                // The struct's own parameter of that name first, whether
                // it was given as a domain or as the type of its positions
                // (`Graph.directed` of a plain list of `(V, V)`).
                let as_domain = |a: &GenArg| match a {
                    GenArg::Domain(d) | GenArg::Ty(Ty::Pos(d)) => Some(*d),
                    _ => None,
                };
                let dom_from_struct = inferred.values().find_map(|ga| match ga {
                    GenArg::Ty(Ty::Struct(sid)) => {
                        let info = &self.structs[*sid];
                        info.env.get(g).and_then(as_domain).or_else(|| {
                            info.origin_gen
                                .as_ref()
                                .and_then(|(_, gargs)| gargs.iter().find_map(as_domain))
                        })
                    }
                    // A table of neighbour lists, `[V]List<..>`, is a graph
                    // over the domain that indexes it.
                    GenArg::Ty(Ty::Array { dims, elem }) if matches!(**elem, Ty::List(_)) => {
                        match dims.as_slice() {
                            [Dim::Pos(d)] => Some(*d),
                            _ => None,
                        }
                    }
                    _ => None,
                });
                if let Some(d) = dom_from_struct {
                    inferred.insert(g.clone(), GenArg::Domain(d));
                }
            }
        }

        // `argmax<S: Keyed, D, T>(self: S) -> ?D`: `D` is what `S` is keyed by.
        if gen_names.iter().any(|g| g == "D") && !inferred.contains_key("D") {
            let key = decl.bounds.iter().find_map(|b| {
                let keyed = b.traits.iter().any(|tr| {
                    matches!(&tr.kind, ast::TypeKind::Named { name, .. }
                        if matches!(name.text.as_str(), "Keyed" | "Indexed" | "IndexedMut"))
                });
                match inferred.get(&b.name.text) {
                    Some(GenArg::Ty(t)) if keyed => t.seq_key(),
                    _ => None,
                }
            });
            if let Some(arg) = key.and_then(GenArg::of_key) {
                inferred.insert("D".to_string(), arg);
            }
        }

        // `S: Indexed<K, T>`: `K` is what `S` is keyed by and `T` its element (R04b.5.3).
        // A domain passed as `S` is the sequence of its positions.
        for b in &decl.bounds {
            let (key, elem) = match inferred.get(&b.name.text) {
                Some(GenArg::Ty(seq)) => (seq.seq_key(), seq.seq_elem()),
                Some(GenArg::Domain(d)) => (None, Some(Ty::Pos(*d))),
                _ => continue,
            };
            for tr in &b.traits {
                let ast::TypeKind::Named { name, args: targs } = &tr.kind else {
                    continue;
                };
                let named = name.text.as_str();
                if !matches!(named, "Keyed" | "Indexed" | "IndexedMut" | "Positional") {
                    continue;
                }
                let free = |t: Option<&ast::Type>| match t.map(|t| &t.kind) {
                    Some(ast::TypeKind::Named { name, args })
                        if args.is_empty()
                            && gen_names.contains(&name.text)
                            && !inferred.contains_key(&name.text) =>
                    {
                        Some(name.text.clone())
                    }
                    _ => None,
                };
                let (k, t) = if named == "Positional" {
                    (None, free(targs.first()))
                } else {
                    (free(targs.first()), free(targs.get(1)))
                };
                if let (Some(k), Some(arg)) = (k, key.and_then(GenArg::of_key)) {
                    inferred.insert(k, arg);
                }
                if let (Some(t), Some(elem)) = (t, elem.clone()) {
                    inferred.insert(t, GenArg::Ty(elem));
                }
            }
        }

        // Pass 2 (up to 4 rounds for chained lambdas like `dijkstra(adj, from, w)`):
        // unify from lambda arguments whose parameter types are now known.
        for _ in 0..4 {
            let before = inferred.len();
            // If an unbound generic (e.g. `T` in `fold_path<S, T>(..., ds: S)`) is present in a bound
            // struct argument's generic environment (e.g. `SegTree<Pre, Word>` has `T -> Word`), inherit it.
            for g in &gen_names {
                if !inferred.contains_key(g) {
                    let from_struct = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::Struct(sid)) => self.structs[*sid].env.get(g).cloned(),
                        _ => None,
                    });
                    if let Some(ga) = from_struct {
                        inferred.insert(g.clone(), ga);
                    }
                }
            }
            // If an unbound generic `A` is used as a lambda parameter type (e.g. `w: A -> W` in `dijkstra`),
            // deduce `A` from a bound `N` (`Graph<V, E>`'s `out` element type, or `Fn(_, List<Elem>)`).
            for g in &gen_names {
                if !inferred.contains_key(g) && g == "A" {
                    let arc_ty = inferred.values().find_map(|ga| match ga {
                        GenArg::Ty(Ty::Struct(sid)) => {
                            let out_fn = self.own_method(*sid, "out")?;
                            if let Some(Ty::List(elem)) = self.sigs[out_fn].ret.clone() {
                                Some((*elem).clone())
                            } else {
                                None
                            }
                        }
                        GenArg::Ty(Ty::Fn(_, ret) | Ty::Array { elem: ret, .. }) => {
                            if let Ty::List(elem) = ret.as_ref() {
                                Some((**elem).clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });
                    if let Some(aty) = arc_ty {
                        inferred.insert(g.clone(), GenArg::Ty(aty));
                    }
                }
            }
            for (param, arg) in ast_params.iter().zip(&matched) {
                let Some(pty) = param.ty.as_ref().map(lambda_infer::unwrap_optional) else {
                    continue;
                };
                let is_default = arg.is_none();
                let expr = match arg {
                    Some(a) => &a.value,
                    None => match &param.default {
                        Some(d) => d,
                        None => continue,
                    },
                };
                if is_default && !matches!(&expr.kind, ast::ExprKind::Lambda { .. }) {
                    if self.ast_ty_has_unbound(pty, &gen_names, &inferred) {
                        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                        let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                        let probed = self.probe_ty_silent(expr);
                        self.cur_mod = saved_mod;
                        self.offset = saved_offset;
                        self.gen_env = saved_env;
                        if let Some(dty) = probed {
                            self.unify_ast_ty(pty, &dty, tpl_mod, &gen_names, &mut inferred);
                        }
                    }
                    continue;
                }
                if let (ast::ExprKind::Name(_), ast::TypeKind::Named { name, args: targs }) =
                    (&expr.kind, &pty.kind)
                {
                    let open = targs.is_empty()
                        && gen_names.contains(&name.text)
                        && !inferred.contains_key(&name.text);
                    if open && self.detect_bare_op(expr).is_some() {
                        let tpl = (tpl_mod, tpl_offset);
                        if let Some(f) =
                            self.fn_impl_shape(&decl.bounds, &name.text, tpl, &inferred)
                        {
                            inferred.insert(name.text.clone(), GenArg::Ty(f));
                        }
                    }
                }
                let ast::ExprKind::Lambda {
                    params: lparams,
                    body: lbody,
                    ..
                } = &expr.kind
                else {
                    continue;
                };
                if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                    if targs.is_empty()
                        && gen_names.contains(&name.text)
                        && !inferred.contains_key(&name.text)
                    {
                        let tpl = (tpl_mod, tpl_offset);
                        if let Some(f) =
                            self.fn_impl_shape(&decl.bounds, &name.text, tpl, &inferred)
                        {
                            inferred.insert(name.text.clone(), GenArg::Ty(f));
                            continue;
                        }
                        let vertex = inferred.get("V").or_else(|| inferred.get("D"));
                        let vertex = vertex.and_then(|ga| match ga {
                            GenArg::Domain(d) => Some(Ty::Pos(*d)),
                            GenArg::Ty(t) => Some(t.clone()),
                            _ => None,
                        });
                        if let Some(param) = self.one_param(lparams, vertex.as_ref()) {
                            // `peel(v => adj[v])`: with nothing else to say what a
                            // vertex is, it is what the lambda indexes with `v`.
                            let ann = vertex.is_none().then(|| self.annotated_param_ty(expr));
                            let annotated = matches!(ann, Some(Some(_)));
                            let dom_ty = vertex
                                .or(ann.flatten())
                                .or_else(|| self.indexed_by_param(&param, lbody));
                            if let (Some(Ty::Pos(d)), false) = (&dom_ty, inferred.contains_key("V"))
                            {
                                if gen_names.iter().any(|g| g == "V") {
                                    inferred.insert("V".to_string(), GenArg::Domain(*d));
                                }
                            }
                            if let Some(vty) = dom_ty {
                                if let Some(ret_ty) = self.probe_lambda_ret(
                                    std::slice::from_ref(&param),
                                    std::slice::from_ref(&vty),
                                    lbody,
                                ) {
                                    inferred.insert(
                                        name.text.clone(),
                                        GenArg::Ty(Ty::Fn(vec![vty], Box::new(ret_ty))),
                                    );
                                    if annotated {
                                        self.unify_bounds(
                                            &decl.bounds,
                                            tpl_mod,
                                            &gen_names,
                                            &mut inferred,
                                        );
                                        self.fn_graph_vertex_domains(
                                            &decl.bounds,
                                            tpl_mod,
                                            &mut inferred,
                                        );
                                    }
                                }
                            }
                            continue;
                        } else if is_default && !lparams.is_empty() {
                            let ptys = vec![Ty::Int; lparams.len()];
                            let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                            let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                            let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                            let ret_ty = self.probe_lambda_ret(lparams, &ptys, lbody);
                            self.cur_mod = saved_mod;
                            self.offset = saved_offset;
                            self.gen_env = saved_env;
                            if let Some(ret_ty) = ret_ty {
                                inferred.insert(
                                    name.text.clone(),
                                    GenArg::Ty(Ty::Fn(ptys, Box::new(ret_ty))),
                                );
                            }
                            continue;
                        }
                    }
                }
                let tpl = (tpl_mod, tpl_offset);
                self.unify_lambda_ret(pty, lparams, lbody, tpl, &gen_names, &mut inferred, false);
            }
            self.unify_bounds(&decl.bounds, tpl_mod, &gen_names, &mut inferred);
            if inferred.len() == before {
                // Only after ready lambdas have had a chance to infer `T` (e.g. `init: V -> T`),
                // fall back to deducing an unbound `T`/`E`/`X` from a bound container generic `S`.
                // A generic a lambda argument is passed as is that lambda's fn, never an element.
                let fn_valued: Vec<&String> = (ast_params.iter().zip(&matched))
                    .filter_map(
                        |(p, a)| match (p.ty.as_ref().map(|t| &t.kind), a.as_ref()) {
                            (Some(ast::TypeKind::Named { name, args }), Some(a))
                                if args.is_empty()
                                    && matches!(a.value.kind, ast::ExprKind::Lambda { .. }) =>
                            {
                                Some(&name.text)
                            }
                            _ => None,
                        },
                    )
                    .collect();
                for g in &gen_names {
                    if !inferred.contains_key(g)
                        && !fn_valued.contains(&g)
                        && (g == "T" || g == "E" || g == "X")
                    {
                        let elem_ty = self.inferred_elem(&inferred, span);
                        if let Some(ety) = elem_ty {
                            inferred.insert(g.clone(), GenArg::Ty(ety));
                        }
                    }
                }
                if inferred.len() == before {
                    if let (Some(ret_ast), Some(want)) = (
                        decl.ret.as_ref(),
                        hint.as_ref().filter(|t| **t != Ty::Error),
                    ) {
                        self.unify_ast_ty(ret_ast, want, tpl_mod, &gen_names, &mut inferred);
                    }
                }
                if inferred.len() == before {
                    let tpl = (tpl_mod, tpl_offset);
                    self.bind_from_lambda_results(
                        &ast_params,
                        &matched,
                        tpl,
                        &gen_names,
                        &mut inferred,
                    );
                }
                if inferred.len() == before {
                    // A lambda with unwritten parameter types given to an open
                    // `X` takes `Int`s, as a literal does (R08.2.11).
                    for (param, arg) in ast_params.iter().zip(&matched) {
                        let (Some(pty), Some(arg)) = (&param.ty, arg) else {
                            continue;
                        };
                        let (
                            ast::TypeKind::Named { name, args: targs },
                            ast::ExprKind::Lambda {
                                params: lparams,
                                param_tys,
                                body: lbody,
                                ..
                            },
                        ) = (&pty.kind, &arg.value.kind)
                        else {
                            continue;
                        };
                        let bounded = decl.bounds.iter().any(|b| b.name.text == name.text);
                        if bounded
                            || !targs.is_empty()
                            || !gen_names.contains(&name.text)
                            || inferred.contains_key(&name.text)
                            || lparams.is_empty()
                            || param_tys.iter().any(Option::is_some)
                        {
                            continue;
                        }
                        let ptys = vec![Ty::Int; lparams.len()];
                        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                        let saved_env = std::mem::replace(&mut self.gen_env, inferred.clone());
                        let ret_ty = self.probe_lambda_ret(lparams, &ptys, lbody);
                        self.cur_mod = saved_mod;
                        self.offset = saved_offset;
                        self.gen_env = saved_env;
                        if let Some(ret_ty) = ret_ty {
                            let f = Ty::Fn(ptys, Box::new(ret_ty));
                            inferred.insert(name.text.clone(), GenArg::Ty(f));
                        }
                    }
                }
                if inferred.len() == before {
                    break;
                }
            }
        }

        // Handle `<T, S>(it: S) -> T` where `T` is the element type of `S` (or `S` is `List<T>` when `it` is `[]`).
        for (pi, p) in ast_params.iter().enumerate() {
            if let Some(arg) = args.get(pi) {
                if matches!(&arg.value.kind, ast::ExprKind::List(items) if items.is_empty()) {
                    if let Some(pty) = p.ty.as_ref() {
                        if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                            if targs.is_empty()
                                && gen_names.contains(&name.text)
                                && !inferred.contains_key(&name.text)
                            {
                                if let Some(GenArg::Ty(elem_ty)) = decl
                                    .ret
                                    .as_ref()
                                    .and_then(|r| match &r.kind {
                                        ast::TypeKind::Named { name: rn, args: ra }
                                            if ra.is_empty() =>
                                        {
                                            inferred.get(&rn.text)
                                        }
                                        _ => None,
                                    })
                                    .cloned()
                                {
                                    inferred
                                        .insert(name.text.clone(), GenArg::Ty(Ty::list(elem_ty)));
                                }
                            }
                        }
                    }
                }
            }
        }
        if let (Some(ret_ast), Some(want)) = (decl.ret.as_ref(), hint.filter(|t| *t != Ty::Error)) {
            self.unify_ast_ty(ret_ast, &want, tpl_mod, &gen_names, &mut inferred);
        }
        if let Some(ret_ast) = decl.ret.as_ref() {
            if let ast::TypeKind::Named { name: rn, args: ra } = &ret_ast.kind {
                if ra.is_empty() && gen_names.contains(&rn.text) && !inferred.contains_key(&rn.text)
                {
                    let elem_from_container = self.inferred_elem(&inferred, span);
                    if let Some(elem_ty) = elem_from_container {
                        inferred.insert(rn.text.clone(), GenArg::Ty(elem_ty));
                    }
                }
            }
        }

        // Enforce `Seq` and `Indexed` trait bounds on generic parameters.
        for bound in &decl.bounds {
            let Some(GenArg::Ty(ty)) = inferred.get(&bound.name.text) else {
                continue;
            };
            if *ty == Ty::Error {
                continue;
            }
            for tr in &bound.traits {
                let ast::TypeKind::Named { name: tr_name, .. } = &tr.kind else {
                    continue;
                };
                let ok = match tr_name.text.as_str() {
                    "Seq" | "Keyed" | "Sized" => {
                        ty.is_seq()
                            || (matches!(ty, Ty::Struct(_))
                                && self.implements(ty, tr_name.text.as_str()))
                    }
                    "Indexed" | "Bidir" => {
                        ty.is_indexed()
                            || (matches!(ty, Ty::Struct(_))
                                && self.implements(ty, tr_name.text.as_str()))
                    }
                    "IndexedMut" => {
                        ty.is_indexed_mut()
                            || (matches!(ty, Ty::Struct(_))
                                && self.implements(ty, tr_name.text.as_str()))
                    }
                    // A fn has no order, equality, hash or text; it can be a `Next`.
                    "Ord" | "Eq" | "Show" => !matches!(ty, Ty::Fn(..)),
                    t => match self.get_sym(t) {
                        Some(Sym::Trait(tid)) if self.traits[tid].structural => {
                            !matches!(ty, Ty::Fn(..))
                        }
                        _ => true,
                    },
                };
                if !ok {
                    self.report(
                        span,
                        msgs::check::MISSING_IMPL_DOES_NOT_IMPLEMENT_REQUIRED,
                        &[&self.show(ty), &tr_name.text, &decl.name.text],
                    );
                    let own = match self.get_sym(&tr_name.text) {
                        Some(Sym::Trait(tid)) => self.traits[tid].unimplemented.clone(),
                        _ => None,
                    };
                    if let Some(words) = own {
                        let shown = self.show(ty);
                        self.help(words.replace("{T}", &shown));
                    }
                    return self.err_expr(span);
                }
            }
        }

        self.bind_from_later_calls(decl, span, tpl_mod, &gen_names, &mut inferred);
        let open = |g| inferred.get(g).cloned().unwrap_or(GenArg::Ty(Ty::Int));
        let mut final_args: Vec<GenArg> = gen_names.iter().map(open).collect();
        // A fn written for the `Map<K, V>` or `Set<T>` it is called on is
        // the same fn for the hashed kind: it gets its own copy, whose
        // `self` is the `HashMap` or `HashSet` given.
        let given = match &receiver {
            Some(Arg::Value(e)) => Some(e.ty.clone()),
            Some(Arg::Mut(place)) => Some(place.ty.clone()),
            None => match (decl.params.first(), args.first()) {
                (Some(p), Some(arg)) if p.name.text == "self" => self.probe_ty_silent(&arg.value),
                _ => None,
            },
        };
        let hashed = given.filter(|ty| {
            let declared = decl
                .params
                .first()
                .and_then(|p| p.ty.as_ref())
                .map(|ty| &ty.kind);
            match (ty, declared) {
                (Ty::HashMap(..), Some(ast::TypeKind::Named { name, .. })) => name.text == "Map",
                (Ty::HashSet(_), Some(ast::TypeKind::Named { name, .. })) => name.text == "Set",
                _ => false,
            }
        });
        let mut trait_params: Vec<(usize, Ty)> = Vec::new();
        for (decl_idx, param) in decl.params.iter().enumerate() {
            let Some(pty) = &param.ty else { continue };
            let ast::TypeKind::Named { name, .. } = &pty.kind else {
                continue;
            };
            let is_trait = matches!(self.mod_syms[tpl_mod].get(&name.text), Some(Sym::Trait(_)))
                || matches!(name.text.as_str(), "Keyed" | "Indexed" | "IndexedMut");
            if !is_trait {
                continue;
            }
            let pos = if decl.params.first().is_some_and(|p| p.name.text == "self") {
                if decl_idx == 0 {
                    continue;
                }
                decl_idx - 1
            } else {
                decl_idx
            };
            let matched_arg = args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == param.name.text))
                .or_else(|| args.get(pos).filter(|a| a.name.is_none()));
            if let Some(arg) = matched_arg {
                let hint = self
                    .trait_lambda_hint(pty, &arg.value, &inferred)
                    .or_else(|| {
                        self.trait_list_hint(pty, &arg.value, (tpl_mod, tpl_offset), &inferred)
                    });
                let probed = self.probe_expr(&arg.value, hint.as_ref());
                if probed.ty != Ty::Error {
                    trait_params.push((decl_idx, probed.ty));
                }
            } else if let Some(default) = &param.default {
                let saved = (
                    std::mem::replace(&mut self.cur_mod, tpl_mod),
                    std::mem::replace(&mut self.offset, tpl_offset),
                    std::mem::replace(&mut self.gen_env, inferred.clone()),
                );
                let probed = self.probe_ty_silent(default);
                (self.cur_mod, self.offset, self.gen_env) = saved;
                if let Some(dty) = probed.filter(|t| *t != Ty::Error) {
                    trait_params.push((decl_idx, dty));
                }
            }
        }
        for (_, ty) in &trait_params {
            final_args.push(GenArg::Ty(ty.clone()));
        }
        if hashed.is_some() {
            final_args.push(GenArg::Int(1));
        }
        final_args.extend(op_params.iter().map(|&(_, op)| GenArg::Op(op)));
        let rev = self.reversed_params(decl, &ast_params, receiver.as_ref(), args, &gen_names);
        if let Some(e) = self.growing_call(gfid, &decl.name.text, span) {
            return e;
        }
        let mut range_ends: Vec<(String, GenArg)> = gen_names
            .iter()
            .filter_map(|g| {
                let key = mono::range_ends_marker(g);
                let ends = inferred.get(&key)?.clone();
                (inferred.get(g) == Some(&GenArg::Ty(Ty::Int))).then_some((key, ends))
            })
            .collect();
        range_ends.extend(owner_ranges);
        let fn_id = self.instantiate_fn(
            gfid,
            final_args,
            &trait_params,
            &op_params,
            rev,
            &range_ends,
        );
        if let Some(ty) = hashed {
            self.sigs[fn_id].params[0].ty = ty;
        }
        self.user_call(fn_id, receiver, args, span)
    }

    /// Whether the generic fn says its parameter `name` anywhere but as the index of an array it
    /// takes: in what it returns, in another parameter, or in its body.
    pub(in crate::lower) fn gen_fn_names(&self, gfid: GenFnId, name: &str) -> bool {
        let tpl = &self.gen_fns[gfid];
        let decl = tpl.decl;
        let only_indexes = |ty: &ast::Type| {
            let ty = match &ty.kind {
                ast::TypeKind::Optional(inner) => inner,
                _ => ty,
            };
            match &ty.kind {
                ast::TypeKind::Array { elem, .. } => !type_names(elem, name),
                _ => !type_names(ty, name),
            }
        };
        if decl.ret.as_ref().is_some_and(|ret| type_names(ret, name))
            || decl
                .params
                .iter()
                .any(|p| p.ty.as_ref().is_some_and(|ty| !only_indexes(ty)))
        {
            return true;
        }
        let span = match &decl.body {
            ast::FnBody::Block(block) => block.span,
            ast::FnBody::Expr(e) => e.span,
            ast::FnBody::None => return false,
        };
        let abs = span.shift(tpl.offset);
        let text = self.mods[tpl.mod_id]
            .text
            .get(abs.start as usize..abs.end as usize)
            .unwrap_or("");
        let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
        text.match_indices(name).any(|(at, _)| {
            !text[..at].chars().next_back().is_some_and(word)
                && !text[at + name.len()..].chars().next().is_some_and(word)
        })
    }

    /// A `[0..n]T` or `[1..=n]T` passed where a `[D]T` (or `?[D]T`) is asked for: `D` is a domain
    /// sized by the array. A fn whose result does not name `D` (`of` gives it) has one such domain
    /// for all its calls, one fn in the C++, each call putting the size back when it returns; what
    /// can outlive the call (a struct built from the array, a `[D]T` given back) has a domain of
    /// its own per place it is made.
    pub(in crate::lower) fn bind_range_domain(
        &mut self,
        pty: &ast::Type,
        arg: &ast::Expr,
        gen_names: &[String],
        inferred: &mut GenEnv,
        span: Span,
        of: Option<(GenFnId, Option<&ast::Type>)>,
    ) {
        let pty = match &pty.kind {
            ast::TypeKind::Optional(inner) => inner,
            _ => pty,
        };
        let ast::TypeKind::Array { index, elem } = &pty.kind else {
            return;
        };
        if index.len() > 1 {
            return self.bind_grid_range_domains(index, arg, gen_names, inferred, span, of);
        }
        let [ast::Type {
            kind: ast::TypeKind::Named { name, args: targs },
            ..
        }] = index.as_slice()
        else {
            return;
        };
        if !targs.is_empty()
            || !gen_names.contains(&name.text)
            || inferred.contains_key(&name.text)
            || self.bind_view_domain(arg, &name.text, inferred)
        {
            return;
        }
        // `[1, 2, 3]` and `[x for ..]` written at the call are lists.
        let written = matches!(
            arg.kind,
            ast::ExprKind::List(_) | ast::ExprKind::Comprehension { .. }
        );
        // A part of an array keeps the first index of the axis it runs
        // along: `one[.., 3]` of a `[1..=2, 1..=3]T` is indexed from 1.
        let mut part = arg;
        while let ast::ExprKind::Paren(inner) = &part.kind {
            part = inner;
        }
        let part_from = match &part.kind {
            ast::ExprKind::Index { base, indices } => match self.probe_ty_silent(base) {
                Some(Ty::Array { dims, .. }) if dims.len() == indices.len() => {
                    let mut along = indices
                        .iter()
                        .zip(&dims)
                        .filter(|(index, _)| matches!(index.kind, ast::ExprKind::Range { .. }));
                    match (along.next(), along.next()) {
                        (Some((_, Dim::IntFrom(first))), None) => Some(*first),
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        };
        let (lo, list) = match self.probe_ty_silent(arg) {
            Some(
                Ty::Array { .. }
                | Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..),
            ) if part_from.is_some() => (part_from.unwrap_or(0), true),
            Some(Ty::Array { .. }) if written => (0, true),
            Some(Ty::Array { dims, .. }) => match dims.as_slice() {
                [Dim::Int] => (0, false),
                [Dim::IntFrom(lo)] => (*lo, false),
                _ => return,
            },
            // A list is indexed by its own positions, `0..len`.
            Some(
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
                | Ty::Str
                | Ty::StrView,
            ) => (0, true),
            _ => return,
        };
        // `fn f<D>(mut d: [D]Int, ..)` that never says `D` again takes an
        // array of any index: there is no domain to make, and the array
        // is indexed as it always was; a list has no index to keep (R04b.3.1).
        if !list && of.is_some_and(|(gfid, _)| !self.gen_fn_names(gfid, &name.text)) {
            return;
        }
        let d = self.range_domain_at(&name.text, lo, list, span, of);
        inferred.insert(name.text.clone(), GenArg::Domain(d));
        if let ast::TypeKind::Named {
            name: ename,
            args: eargs,
        } = &elem.kind
        {
            if eargs.is_empty()
                && gen_names.contains(&ename.text)
                && !inferred.contains_key(&ename.text)
                && matches!(
                    self.probe_ty_silent(arg).and_then(|t| t.seq_elem()),
                    Some(Ty::Int)
                )
                && of
                    .is_some_and(|(_, ret)| ret.is_some_and(|r| type_uses_as_index(r, &ename.text)))
            {
                inferred.insert(ename.text.clone(), GenArg::Domain(d));
            }
        }
    }

    pub(in crate::lower) fn call_local_gen_fn(
        &mut self,
        decl: &ast::FnDecl,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let mut arg_vals: Vec<Expr> = Vec::with_capacity(args.len());
        let mut new_env = self.gen_env.clone();
        for (i, arg) in args.iter().enumerate() {
            let val = self.expr(&arg.value, None);
            if let Some(p) = decl.params.get(i) {
                if let Some(pty) = &p.ty {
                    if let ast::TypeKind::Named { name, args: targs } = &pty.kind {
                        if targs.is_empty() && decl.generics.iter().any(|g| g.text == name.text) {
                            new_env.insert(name.text.clone(), GenArg::Ty(val.ty.clone()));
                        }
                    }
                }
            }
            arg_vals.push(val);
        }
        let saved_env = std::mem::replace(&mut self.gen_env, new_env);
        let ret_ty = decl
            .ret
            .as_ref()
            .map(|r| self.resolve_ty(r))
            .unwrap_or(Ty::Unit);
        let res = self.do_expr(Some(&ret_ty), span, |lw, out| {
            lw.cx.scopes.push(HashMap::new());
            for (p, val) in decl.params.iter().zip(arg_vals) {
                let slot = lw.temp(val.ty.clone());
                out.push_let(slot, val);
                lw.cx.scopes.top_mut().insert(p.name.text.clone(), slot);
            }
            match &decl.body {
                ast::FnBody::Expr(e) => {
                    let v = lw.expr(e, Some(&ret_ty));
                    out.push_give(v);
                }
                ast::FnBody::Block(b) => {
                    let blk = lw.block(
                        b,
                        if ret_ty == Ty::Unit {
                            Tail::Discard
                        } else {
                            Tail::Give
                        },
                    );
                    out.extend(blk.stmts);
                }
                ast::FnBody::None => {}
            }
            lw.cx.scopes.pop();
        });
        self.gen_env = saved_env;
        res
    }
}
