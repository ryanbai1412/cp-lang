//! Constructor calls and argument lists: matching args to params and fields.

use super::*;
use cpl_diag::msgs;

impl Checker<'_> {
    /// `P(read() for _ in fields(P))` or `P(f + 1 for f in fields(p))`:
    /// the body gives each field's value, checked once per field as the
    /// back ends unroll it (decision 28.9). `own` is the struct and its
    /// fields, when known.
    pub(crate) fn fields_ctor(&mut self, own: Option<(&str, &[Param])>, args: &[Arg]) -> bool {
        let want = own.map(|(_, f)| f);
        let [arg] = args else {
            return false;
        };
        let ExprKind::Comprehension { body, clauses } = &arg.value.kind else {
            return false;
        };
        let [clause] = clauses.as_slice() else {
            return false;
        };
        if arg.name.is_some() {
            return false;
        }
        let Some(values) = self.reflected_values(&clause.iter) else {
            return false;
        };
        let mut shapes = Vec::new();
        for v in values {
            if let ExprKind::Name(n) = &v.value.kind {
                if self.local(n).is_none() && n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    if let Some((name, _)) = own.filter(|(name, _)| n != name) {
                        let message =
                            cpl_diag::msgs::check::FIELDS_HERE_NAMES_TYPE.at(&[&n, &name]);
                        self.report_said(v.value.span, message);
                    }
                    continue;
                }
            }
            let ty = self.infer(&v.value);
            let same_arity = |r: &Result<Option<Vec<(String, Ty)>>, ()>| matches!((own, r), (Some((_, f)), Ok(Some(g))) if f.len() == g.len());
            let reflected = self.reflect(&ty);
            if let (Some((name, _)), Ty::Named { name: got, .. }) = (own, &ty) {
                if got != name && !same_arity(&reflected) {
                    self.report(
                        v.value.span,
                        msgs::check::TYPE_MISMATCH_FIELDS_HERE_WALKS_VALUES,
                        &[&name, &ty],
                    );
                }
            }
            shapes.push(reflected.ok().flatten());
        }
        if !clause.conds.is_empty() {
            let message = cpl_diag::msgs::check::FIELDS_CONSTRUCTOR_GIVES_EVERY.at(&[]);
            self.report_said(clause.iter.span, message);
        }
        if shapes.is_empty() && !matches!(clause.pat.kind, PatternKind::Wildcard) {
            if let Some(d) = self.report(
                clause.pat.span,
                msgs::check::TYPE_MISMATCH_FIELDS_OF_A_TYPE,
                &[],
            ) {
                d.help = Some("write `_`, as in `P(read() for _ in fields(P))`".to_string());
            }
        }
        let count = want.map(<[Param]>::len);
        let passes: Vec<Option<usize>> = match count {
            Some(n) => (0..n).map(Some).collect(),
            None => vec![None],
        };
        for i in passes {
            let one = |s: &Option<Vec<(String, Ty)>>| match (i, s) {
                (Some(i), Some(f)) => f.get(i).map(|(_, t)| t.clone()).unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            };
            let elem = match shapes.as_slice() {
                [] => Ty::Unknown,
                [s] => one(s),
                _ => Ty::Tuple(shapes.iter().map(one).collect()),
            };
            self.push_scope();
            self.bind(&clause.pat, &elem, false, "for");
            let got = self.infer(body);
            if let (Some(i), Some(fields)) = (i, want) {
                let field = &fields[i];
                self.expect_fits(&got, &field.ty, body, &|c, got, want| {
                    cpl_diag::msgs::check::FIELD_BUT.at(&[
                        &field.name,
                        &want.with_article(),
                        &c.quote(body.span),
                        &got.with_article(),
                    ])
                });
            }
            self.pop_scope();
        }
        true
    }

    /// Checks arguments whose parameters are not known.
    pub(super) fn plain_args(&mut self, args: &[Arg]) {
        if self.fields_ctor(None, args) {
            return;
        }
        for arg in args {
            self.infer(&arg.value);
            if arg.mutable {
                self.require_mutable(&arg.value, "passed as `mut`");
            }
        }
    }

    /// `Road(u, v, len)`, `Int(x)`, `List<Int>()`.
    pub(super) fn construct(
        &mut self,
        name: &str,
        type_args: Vec<Ty>,
        callee_text: &str,
        args: &[Arg],
        call: &Expr,
    ) -> Ty {
        if let Some(def) = self.type_def(name) {
            return match &*def {
                Def::Struct {
                    generics,
                    defaults,
                    fields,
                    ctor,
                    methods,
                    receivers,
                    method_rows,
                    generic_bounds,
                    ..
                } => {
                    // Outside the struct's body `S(..)` calls its `new`;
                    // inside, it fills the fields.
                    let inside =
                        matches!(&self.self_ty, Some(Ty::Named { name: own, .. }) if own == name);
                    let (verb, params) = match ctor {
                        Some(params) if !inside => ("takes", params),
                        _ => ("has", fields),
                    };
                    let is_generated_ctor = ctor.is_none() || inside;
                    let own =
                        (generics.is_empty() && is_generated_ctor).then_some((name, &fields[..]));
                    if self.fields_ctor(own, args) {
                        if is_generated_ctor {
                            self.move_ctor_args(args);
                        }
                        return Ty::Named {
                            name: name.to_string(),
                            args: type_args,
                        };
                    }
                    let mut bindings = Bindings::default();
                    if !type_args.is_empty() && type_args.len() <= generics.len() {
                        bindings.extend(generics.iter().cloned().zip(type_args.iter().cloned()));
                    }
                    let before = self.sink.len();
                    // `new<A: Acts<T, F>>(.., act: A)`: `F` is what `A`'s impl gives.
                    let through = (method_rows.iter())
                        .find(|(m, _, _)| m == "new" && !is_generated_ctor)
                        .map(|(_, _, through)| through.clone());
                    let outer = std::mem::replace(&mut self.call_bound_args, through);
                    self.args(callee_text, verb, params, args, call, &mut bindings);
                    let through = std::mem::replace(&mut self.call_bound_args, outer);
                    // `W<T, M: Op<T>>(op: M, xs: List<T>)`: a bare op for `M` is the
                    // monoid, as at a generic fn's bound.
                    let bound_args = through.unwrap_or_else(|| {
                        (generic_bounds.iter())
                            .filter_map(|(p, tr, targs)| {
                                let targs = (targs.iter())
                                    .map(|a| a.map(|j| Ty::Param(generics[j].clone())))
                                    .collect::<Option<Vec<_>>>()?;
                                Some((generics[*p].clone(), tr.clone(), targs))
                            })
                            .collect()
                    });
                    self.bare_ops_take_monoid(&bound_args, params, args, &mut bindings);
                    if !is_generated_ctor && self.sink.len() > before {
                        self.ctor_arg_fix(callee_text, params, (methods, receivers), args, call);
                    }
                    if is_generated_ctor {
                        self.move_ctor_args(args);
                    }
                    for (i, g) in generics.iter().enumerate() {
                        if !bindings.contains_key(g) {
                            if let Some(Some(def_ty)) = defaults.get(i) {
                                bindings.insert(g.clone(), def_ty.subst(&bindings));
                            }
                        }
                    }
                    // Type arguments left out are read off the values.
                    let type_args = if type_args.len() < generics.len()
                        && generics
                            .iter()
                            .enumerate()
                            .all(|(i, g)| i < type_args.len() || bindings.contains_key(g))
                    {
                        generics
                            .iter()
                            .enumerate()
                            .map(|(i, g)| {
                                type_args
                                    .get(i)
                                    .cloned()
                                    .unwrap_or_else(|| Ty::Param(g.clone()).subst(&bindings))
                            })
                            .collect()
                    } else {
                        type_args
                    };
                    Ty::Named {
                        name: name.to_string(),
                        args: type_args,
                    }
                }
                // `MinHeap<(W, V)>()` is the `Heap<(W, V)>` it names.
                Def::Alias { ty, generics } => {
                    self.plain_args(args);
                    ty.subst(&generics.iter().cloned().zip(type_args).collect())
                }
                _ => {
                    self.plain_args(args);
                    Ty::Unknown
                }
            };
        }
        self.plain_args(args);
        match prelude::ty(name) {
            Some(Ty::Named { name, .. }) => Ty::Named {
                name,
                args: type_args,
            },
            Some(ty) => ty,
            None => Ty::Unknown,
        }
    }

    /// Matches `args` to `params`. `verb` is "takes" for a fn and "has" for
    /// a struct's fields.
    ///
    /// A generic callee's type parameters are read off the arguments as
    /// they go by, into `bindings`: after `pair(1, ..)` binds `T = Int`,
    /// the next `T` argument must be an `Int`.
    pub(super) fn args(
        &mut self,
        callee: &str,
        verb: &str,
        params: &[Param],
        args: &[Arg],
        call: &Expr,
        bindings: &mut Bindings,
    ) {
        let positional: Vec<&Param> = params.iter().filter(|p| !p.named_only).collect();
        let variadic = positional.last().is_some_and(|p| p.variadic);
        let labeled = params.iter().all(|p| !p.name.is_empty());
        let noun = if verb == "has" {
            ("field", "fields")
        } else {
            ("argument", "arguments")
        };
        let mut supplied: Vec<&str> = Vec::new();
        let mut next = 0usize;
        let mut extra = 0usize;
        let mut misnamed = false;
        let mut pack: Vec<Ty> = Vec::new();
        let mut named_pack: Vec<(String, Ty)> = Vec::new();
        if !variadic && labeled {
            self.lambda_takes_later_args(params, args, bindings);
        }
        for (i, arg) in args.iter().enumerate() {
            let param = match &arg.name {
                Some(name) if labeled => match params.iter().find(|p| p.name == name.text) {
                    Some(param) => Some(param),
                    // Named arguments to `..xs` make it a named tuple (R06.1.5).
                    None if variadic => {
                        let ty = self.infer(&arg.value);
                        named_pack.push((name.text.clone(), self.settle(&ty).widen()));
                        continue;
                    }
                    None => {
                        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                        let suggestion = closest(&name.text, names.iter().map(String::as_str));
                        let what = if verb == "has" { "field" } else { "parameter" };
                        let message = cpl_diag::msgs::check::HAS_NO_WHAT_NAMED
                            .at(&[&callee, &what, &name.text]);
                        // `act:` is what a lazy structure takes (`SegTree(op, a, act:)`);
                        // another type given one is asked for a specialization it lacks.
                        let lazy_ask = name.text == "act"
                            && self
                                .type_def(callee)
                                .is_some_and(|d| matches!(*d, Def::Struct { .. }));
                        let code = if lazy_ask {
                            codes::LAW_UNSUPPORTED_SPECIALIZATION
                        } else {
                            codes::UNKNOWN_ARG_NAME
                        };
                        if let Some(diag) = self.report_as(code, name.span, message) {
                            let act = code == codes::LAW_UNSUPPORTED_SPECIALIZATION;
                            if let (true, Some(prev)) = (act, i.checked_sub(1)) {
                                let cut = Span::new(args[prev].value.span.end, arg.value.span.end);
                                diag.fix("remove `act:`", cut, "");
                            }
                            match suggestion {
                                Some(s) => diag.suggest(name.span, s),
                                None => {
                                    diag.help =
                                        Some(format!("its {what}s are {}", list_names(&names)));
                                }
                            }
                        }
                        misnamed = true;
                        None
                    }
                },
                Some(_) => None,
                None => {
                    let param = if variadic && next >= positional.len() - 1 {
                        positional.last().copied()
                    } else {
                        positional.get(next).copied()
                    };
                    next += 1;
                    if param.is_none() {
                        extra += 1;
                    }
                    param
                }
            };
            let prev_bounds_range = self.expected_bounds_range;
            let prev_piece = self.expected_piece;
            self.expected_piece = false;
            let mut typed_lambda = false;
            if let Some(p) = param {
                let expected = self
                    .graph_lambda_want(&p.ty, bindings)
                    .unwrap_or_else(|| p.ty.subst(bindings));
                if Self::expects_bounds_range(&expected) {
                    self.expected_bounds_range = true;
                }
                self.expected_piece = Self::expects_piece(&expected);
                typed_lambda = self.lambda_defaults(arg, &expected, callee, params)
                    || self.lambda_takes(arg, &expected);
            }
            let want = param.map(|p| p.ty.subst(bindings));
            let arg_ty = self.infer_arg(&arg.value, want.as_ref());
            // A lambda with unwritten parameters keeps its signature for
            // itself; the generic its result is binds from it.
            let lambda_sig = (typed_lambda && arg_ty == Ty::Unknown)
                .then(|| self.exprs.last_type(arg.value.span).cloned())
                .flatten()
                .filter(|sig| matches!(sig, Ty::Fn { .. }));
            self.expected_bounds_range = prev_bounds_range;
            self.expected_piece = prev_piece;
            let Some(param) = param else {
                if arg.mutable {
                    self.require_mutable(&arg.value, "passed as `mut`");
                }
                continue;
            };
            supplied.push(&param.name);
            if param.variadic && arg.name.is_none() {
                if arg.mutable {
                    self.require_mutable(&arg.value, "passed as `mut`");
                }
                pack.push(self.settle(&arg_ty).widen());
                continue;
            }
            // A `lazy d: () -> T` is given the `T` its lambda gives back.
            let slot = match &param.ty {
                Ty::Fn { params, ret }
                    if param.lazy
                        && params.is_empty()
                        && !matches!(arg.value.kind, ExprKind::Lambda { .. }) =>
                {
                    ret.as_ref()
                }
                ty => ty,
            };
            let arg_ty = self.fn_value_for(&arg.value, arg_ty, &slot.subst(bindings));
            let arg_ty = self.narrowed(&arg_ty, slot);
            slot.unify(lambda_sig.as_ref().unwrap_or(&arg_ty), bindings);
            self.bind_bound_args(bindings);
            let want = slot.subst(bindings);
            if let (
                ExprKind::Lambda {
                    params, param_tys, ..
                },
                Ty::Fn { params: wanted, .. },
            ) = (&arg.value.kind, &want)
            {
                let takes: Vec<Ty> = wanted.iter().map(|p| p.ty.clone()).collect();
                let destructures = matches!(takes.as_slice(), [one] if one.param_fields().is_some_and(|f| f.len() == params.len()));
                if !takes.is_empty()
                    && params.len() != takes.len()
                    && !destructures
                    && param_tys.iter().all(Option::is_none)
                {
                    self.lambda_arity_mismatch(params, &takes);
                    continue;
                }
                self.type_lambda_params(params, param_tys, wanted);
            }
            // `Node([none, none], 0)`: a list written out where an array is
            // wanted is the cells of that array, as in `let a: [0..2]?Int = [..]`.
            if let (ExprKind::List(_), Ty::Array { index, elem }, Ty::Named { name, args }) =
                (&arg.value.kind, &want, &arg_ty)
            {
                if index.len() == 1
                    && name == "List"
                    && args.len() == 1
                    && self.fits(&args[0], elem)
                {
                    continue;
                }
            }
            if callee != "List"
                && callee != "zip"
                && matches!(slot, Ty::Named { name, .. } if name == "List")
                && self.view_kind(&arg.value) == Some("slice")
            {
                let arg_text = self.quote(arg.value.span);
                if let Some(diag) = self.report(
                    arg.value.span,
                    msgs::check::TYPE_MISMATCH_BUT_IS_A_SLICE,
                    &[&callee, &verb, &want.with_article(), &arg_text],
                ) {
                    diag.help = Some(format!(
                        "copy it with `List({arg_text})` or accept `Seq` / `Indexed`"
                    ));
                }
                continue;
            }
            self.expect_fits(&arg_ty, &want, &arg.value, &|c, got, want| {
                let slot = if param.name.is_empty() {
                    String::new()
                } else {
                    format!(" for `{}`", param.name)
                };
                cpl_diag::msgs::check::BUT_2.at(&[
                    &callee,
                    &verb,
                    &want.with_article(),
                    &slot,
                    &c.quote(arg.value.span),
                    &got.with_article(),
                ])
            });
            if param.mutable && !arg.mutable && is_place(&arg.value) {
                let quoted = self.quote(arg.value.span);
                let changed = if param.name == "self" {
                    &quoted
                } else {
                    &param.name
                };
                let at = arg.value.span;
                if let Some(diag) = self.report(
                    at,
                    msgs::check::MISSING_MUT_AT_CALL_CHANGES_SO_THE_ARGUMENT,
                    &[&callee, &changed, &quoted],
                ) {
                    diag.fix("add `mut`", Span::point(at.start), "mut ");
                }
            } else if arg.mutable && !param.mutable && verb == "takes" {
                let amp = Span::new(arg.value.span.start.saturating_sub(1), arg.value.span.start);
                if self.text(amp) == "&" {
                    if let Some(diag) = self.report(
                        arg.value.span,
                        msgs::check::MISSING_MUT_AT_CALL_DOESN_T_CHANGE_SO,
                        &[&callee, &param.name],
                    ) {
                        diag.fix("drop the `&`", amp, String::new());
                    }
                    continue;
                }
                let at = Span::new(arg.value.span.start.saturating_sub(4), arg.value.span.start);
                let written = (self.src.get(at.start as usize..at.end as usize) == Some("mut "))
                    .then_some(at);
                if let Some(diag) = self.report(
                    arg.value.span,
                    msgs::check::MISSING_MUT_AT_CALL_DOESN_T_CHANGE_SO_2,
                    &[&callee, &param.name],
                ) {
                    diag.help = Some(format!(
                        "remove `mut`, or declare the parameter `mut {}`",
                        param.name
                    ));
                    if let Some(at) = written {
                        diag.fix("remove `mut`", at, String::new());
                    }
                }
            }
            if arg.mutable {
                self.require_mutable(&arg.value, "passed as `mut`");
            }
        }
        if let Some(last) = positional.last().filter(|p| p.variadic) {
            if named_pack.is_empty() {
                last.ty.unify(&Ty::Tuple(pack), bindings);
            } else if pack.is_empty() {
                last.ty.unify(&Ty::Record(named_pack), bindings);
            } else {
                self.report(
                    call.span,
                    msgs::check::TYPE_MISMATCH_TAKES_POSITIONAL_OR_NAMED,
                    &[&last.name],
                );
            }
        }
        // A misspelled name already explains a missing argument.
        if misnamed || (!labeled && args.iter().any(|a| a.name.is_some())) {
            return;
        }
        let names: Vec<String> = positional
            .iter()
            .map(|p| p.name.clone())
            .filter(|n| !n.is_empty())
            .collect();
        let listed = if names.is_empty() {
            String::new()
        } else {
            format!(" ({})", names.join(", "))
        };
        let given = args.iter().filter(|a| a.name.is_none()).count();
        if extra > 0 {
            self.report(
                call.span,
                msgs::check::WRONG_ARG_COUNT_BUT_GIVEN,
                &[
                    &callee,
                    &verb,
                    &plural(positional.len(), noun.0, noun.1),
                    &listed,
                    &if given == 1 {
                        "1 was".to_string()
                    } else {
                        format!("{given} were")
                    },
                ],
            );
            return;
        }
        let missing: Vec<String> = params
            .iter()
            .filter(|p| !p.has_default && !p.variadic && !supplied.contains(&p.name.as_str()))
            .map(|p| p.name.clone())
            .collect();
        if !missing.is_empty() {
            let what = if missing.iter().all(|m| !m.is_empty()) {
                format!("{} missing", list_names(&missing))
            } else {
                format!(
                    "{} given",
                    if given == 1 {
                        "only 1 was".to_string()
                    } else {
                        format!("only {given} were")
                    }
                )
            };
            let verb_s = if missing.len() == 1 { "is" } else { "are" };
            let message = if what.ends_with("missing") {
                cpl_diag::msgs::check::ARGS_BUT_MISSING.at(&[
                    &callee,
                    &verb,
                    &plural(positional.len(), noun.0, noun.1),
                    &listed,
                    &list_names(&missing),
                    &verb_s,
                ])
            } else {
                cpl_diag::msgs::check::ARGS_BUT.at(&[
                    &callee,
                    &verb,
                    &plural(positional.len(), noun.0, noun.1),
                    &what,
                ])
            };
            self.report_said(call.span, message);
        }
    }
}

impl Checker<'_> {
    /// A generic fn passed as a value takes its type parameters from the
    /// parameter types it is given: `ap(v, desc)` with `by: V -> K` is
    /// `desc` at `T = V`, so `K` is `Desc<V>`.
    fn fn_value_for(&mut self, value: &Expr, ty: Ty, want: &Ty) -> Ty {
        let ExprKind::Name(name) = &value.kind else {
            return ty;
        };
        if self.local(name).is_some() {
            return ty;
        }
        let Some(Def::Fn { generics, .. }) = self.global(name).as_deref().cloned() else {
            return ty;
        };
        let (Ty::Fn { params, .. }, Ty::Fn { params: wanted, .. }) = (&ty, want) else {
            return ty;
        };
        let mut own = Bindings::default();
        for (param, wanted) in params.iter().zip(wanted) {
            param.ty.unify(&wanted.ty, &mut own);
        }
        own.retain(|g, t| generics.contains(g) && !matches!(t, Ty::Param(_)));
        ty.subst(&own)
    }

    /// `Counter(0)`, `SortedSet(xs)`, `Dsu<V>(n)`: one positional argument
    /// a type's `new` doesn't take. The fix names the option it is, builds
    /// from a list with `of`, or drops a size the type arguments give.
    fn ctor_arg_fix(
        &mut self,
        callee: &str,
        params: &[crate::ty::Param],
        (methods, receivers): (&[(String, Ty)], &[String]),
        args: &[Arg],
        call: &Expr,
    ) {
        let [arg] = args else { return };
        if arg.name.is_some() {
            return;
        }
        let ty = self.quietly(|c| c.infer(&arg.value));
        let text = self.text(arg.value.span).to_string();
        let of = methods.iter().any(|(m, _)| m == "of") && !receivers.iter().any(|r| r == "of");
        let list = matches!(&ty, Ty::Array { .. })
            || matches!(&ty, Ty::Named { name, .. } if name == "List");
        let option = (params.iter())
            .find(|p| p.named_only && (matches!(p.ty, Ty::Param(_)) || self.fits(&ty, &p.ty)));
        let (help, new) = match option {
            Some(p) if p.has_default && text == "0" => (
                format!("`{callee}`'s `{}` is 0 by default", p.name),
                Some(format!("{callee}()")),
            ),
            Some(p) => (
                format!("`{}` is named: `{callee}({}: {text})`", p.name, p.name),
                Some(format!("{callee}({}: {text})", p.name)),
            ),
            None if list && of => (
                format!("`{callee}.of(xs)` builds one from a list"),
                Some(format!("{callee}.of({text})")),
            ),
            None if ty == Ty::Int && callee.contains('<') => (
                format!("`{callee}` takes its size from its type arguments"),
                Some(format!("{callee}()")),
            ),
            None if list => (
                format!("`{callee}()` starts empty: add each element in a `for` loop"),
                None,
            ),
            None => return,
        };
        let ours = |d: &&mut cpl_diag::Diagnostic| {
            matches!(d.code, codes::WRONG_ARG_COUNT | codes::TYPE_MISMATCH) && d.fixes.is_empty()
        };
        if let Some(diag) = self.sink.last_mut().filter(ours) {
            diag.help = Some(help);
            if let Some(new) = new {
                diag.fix(format!("write `{new}`"), call.span, new);
            }
        }
    }

    /// `sum(xs, of: x => x * x)`: a lambda whose parameters it leaves
    /// unwritten takes the ones the fn's earlier arguments fixed, so its
    /// body, and the generic its result binds, are checked with them.
    /// `bfs(p => .., from: [s])`: the arguments after a lambda with unwritten
    /// parameters bind the generics it takes before it is checked.
    fn lambda_takes_later_args(&mut self, params: &[Param], args: &[Arg], bindings: &mut Bindings) {
        let untyped = |a: &Arg| {
            matches!(&a.value.kind, ExprKind::Lambda { param_tys, .. }
                if param_tys.iter().all(Option::is_none))
        };
        let Some(first) = args.iter().position(untyped) else {
            return;
        };
        let landed = crate::check::bound_args::arg_params(params, args);
        for (arg, param) in args.iter().zip(landed).skip(first + 1) {
            let Some(param) = param.filter(|_| !matches!(arg.value.kind, ExprKind::Lambda { .. }))
            else {
                continue;
            };
            let got = self.quietly(|c| c.infer(&arg.value));
            if got.has_vague() {
                continue;
            }
            match &param.ty {
                Ty::Named { name, args: want }
                    if matches!(self.type_def(name).as_deref(), Some(Def::Trait { .. })) =>
                {
                    for (w, g) in want.iter().zip(
                        self.trait_args_of(&got, name, want.len())
                            .unwrap_or_default(),
                    ) {
                        w.unify(&g, bindings);
                    }
                }
                ty => ty.unify(&got, bindings),
            }
        }
    }

    /// `g: G` with `G: Next<V>` given a lambda: any closure from a vertex to
    /// its arcs is a graph, so it takes a `V` (lib/graphs-trees.md).
    fn graph_lambda_want(&self, ty: &Ty, bindings: &Bindings) -> Option<Ty> {
        let Ty::Param(g) = ty else {
            return None;
        };
        let (_, _, args) =
            (self.call_bound_args.as_ref()?.iter()).find(|(p, t, _)| p == g && t == "Next")?;
        let [v] = args.as_slice() else {
            return None;
        };
        let v = v.subst(bindings);
        let param = Param {
            name: String::new(),
            ty: v,
            mutable: false,
            has_default: false,
            named_only: false,
            variadic: false,
            lazy: false,
        };
        Some(Ty::Fn {
            params: vec![param],
            ret: Box::new(Ty::Unknown),
        })
    }

    fn lambda_takes(&mut self, arg: &Arg, expected: &Ty) -> bool {
        let (ExprKind::Lambda { param_tys, .. }, Ty::Fn { params, .. }) =
            (&arg.value.kind, expected)
        else {
            return false;
        };
        let fixed = |t: &Ty| !t.has_vague() && !t.mentions_param();
        if param_tys.iter().any(Option::is_some) || !params.iter().all(|p| fixed(&p.ty)) {
            return false;
        }
        // A catalog row's guess that left a parameter unknown gives way.
        let at = arg.value.span;
        match self.lambda_params.iter().position(|(s, _)| *s == at) {
            Some(i) if self.lambda_params[i].1.iter().all(fixed) => return false,
            Some(i) => {
                self.lambda_params.swap_remove(i);
            }
            None => {}
        }
        let tys = params.iter().map(|p| p.ty.clone()).collect();
        self.lambda_params.push((arg.value.span, tys));
        true
    }
}
