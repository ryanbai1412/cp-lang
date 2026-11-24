//! Constructor calls and argument lists: matching args to params and fields.

use super::*;

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
                            format!("`fields({n})` here names the type it builds, `{name}`");
                        self.error(codes::TYPE_MISMATCH, v.value.span, message);
                    }
                    continue;
                }
            }
            let ty = self.infer(&v.value);
            let same_arity = |r: &Result<Option<Vec<(String, Ty)>>, ()>| matches!((own, r), (Some((_, f)), Ok(Some(g))) if f.len() == g.len());
            let reflected = self.reflect(&ty);
            if let (Some((name, _)), Ty::Named { name: got, .. }) = (own, &ty) {
                if got != name && !same_arity(&reflected) {
                    let message = format!(
                        "`fields` here walks values of the type it builds, `{name}`; this is a `{ty}`"
                    );
                    self.error(codes::TYPE_MISMATCH, v.value.span, message);
                }
            }
            shapes.push(reflected.ok().flatten());
        }
        if !clause.conds.is_empty() {
            let message =
                "a `fields` constructor gives every field a value, so it has no `if`".to_string();
            self.error(codes::UNSUPPORTED, clause.iter.span, message);
        }
        if shapes.is_empty() && !matches!(clause.pat.kind, PatternKind::Wildcard) {
            if let Some(d) = self.error(
                codes::TYPE_MISMATCH,
                clause.pat.span,
                "`fields` of a type has no values to bind".to_string(),
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
                self.expect_fits(&got, &field.ty, body, |c, got, want| {
                    format!(
                        "the field `{}` is {}, but `{}` is {}",
                        field.name,
                        want.with_article(),
                        c.quote(body.span),
                        got.with_article()
                    )
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
                    self.args(callee_text, verb, params, args, call, &mut bindings);
                    if !is_generated_ctor && self.sink.len() > before {
                        self.ctor_arg_fix(callee_text, params, (methods, receivers), args, call);
                    }
                    if is_generated_ctor {
                        self.move_ctor_args(args);
                    }
                    if generics.iter().any(|g| g == "F") && !bindings.contains_key("F") {
                        if let Some(act_arg) = args
                            .iter()
                            .find(|a| a.name.as_ref().is_some_and(|n| n.text == "act"))
                        {
                            let act_ty = self.quietly(|c| c.infer(&act_arg.value));
                            let elem_ty = bindings.get("T").cloned().unwrap_or(Ty::Int);
                            let inferred_f = match &act_ty {
                                Ty::Named { name: n, args: aa }
                                    if n == "Action" && aa.len() == 2 =>
                                {
                                    Some(aa[1].clone())
                                }
                                Ty::Named { name: n, .. } if n == "AddAction" => Some(elem_ty),
                                Ty::Named { name: n, .. } if n == "AssignAction" => {
                                    Some(Ty::Optional(Box::new(elem_ty)))
                                }
                                Ty::Named { name: n, .. } if n == "AffineAction" => {
                                    Some(Ty::Tuple(vec![elem_ty.clone(), elem_ty]))
                                }
                                Ty::Named { name: n, .. } if n == "FlipAction" => Some(Ty::Bool),
                                _ => None,
                            };
                            if let Some(f_ty) = inferred_f {
                                bindings.insert("F".to_string(), f_ty);
                            }
                        }
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
                                    .unwrap_or_else(|| bindings[g].clone())
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
                Def::Alias { ty, .. } => {
                    self.plain_args(args);
                    ty.clone()
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
                        let message = format!("`{callee}` has no {what} named `{}`", name.text);
                        let code = if callee == "Fenwick" && name.text == "act" {
                            codes::LAW_UNSUPPORTED_SPECIALIZATION
                        } else {
                            codes::UNKNOWN_ARG_NAME
                        };
                        if let Some(diag) = self.error(code, name.span, message) {
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
            if let Some(p) = param {
                let expected = p.ty.subst(bindings);
                if Self::expects_bounds_range(&expected) {
                    self.expected_bounds_range = true;
                }
                self.expected_piece = Self::expects_piece(&expected);
            }
            let arg_ty = self.infer(&arg.value);
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
            slot.unify(&arg_ty, bindings);
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
                let message = format!(
                    "`{callee}` {verb} {}, but `{arg_text}` is a slice view, not a `List`",
                    want.with_article(),
                );
                if let Some(diag) = self.error(codes::TYPE_MISMATCH, arg.value.span, message) {
                    diag.help = Some(format!(
                        "copy it with `List({arg_text})` or accept `Seq` / `Indexed`"
                    ));
                }
                continue;
            }
            self.expect_fits(&arg_ty, &want, &arg.value, |c, got, want| {
                let slot = if param.name.is_empty() {
                    String::new()
                } else {
                    format!(" for `{}`", param.name)
                };
                format!(
                    "`{callee}` {verb} {}{slot}, but `{}` is {}",
                    want.with_article(),
                    c.quote(arg.value.span),
                    got.with_article()
                )
            });
            if param.mutable && !arg.mutable && is_place(&arg.value) {
                let quoted = self.quote(arg.value.span);
                let changed = if param.name == "self" {
                    &quoted
                } else {
                    &param.name
                };
                let message = format!(
                    "`{callee}` changes `{changed}`, so the argument must be passed as `mut {quoted}`"
                );
                let at = arg.value.span;
                if let Some(diag) = self.error(codes::MISSING_MUT_AT_CALL, at, message) {
                    diag.fix("add `mut`", Span::point(at.start), "mut ");
                }
            } else if arg.mutable && !param.mutable && verb == "takes" {
                let amp = Span::new(arg.value.span.start.saturating_sub(1), arg.value.span.start);
                if self.text(amp) == "&" {
                    let message = format!(
                        "`{callee}` doesn't change `{}`, so the argument is passed without `&`",
                        param.name
                    );
                    if let Some(diag) =
                        self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message)
                    {
                        diag.fix("drop the `&`", amp, String::new());
                    }
                    continue;
                }
                let message = format!(
                    "`{callee}` doesn't change `{}`, so `mut` doesn't belong here",
                    param.name
                );
                let at = Span::new(arg.value.span.start.saturating_sub(4), arg.value.span.start);
                let written = (self.src.get(at.start as usize..at.end as usize) == Some("mut "))
                    .then_some(at);
                if let Some(diag) = self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message)
                {
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
                let message = format!(
                    "`..{}` takes positional or named arguments in one call, not both",
                    last.name
                );
                self.error(codes::TYPE_MISMATCH, call.span, message);
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
            let message = format!(
                "`{callee}` {verb} {}{listed}, but {} given",
                plural(positional.len(), noun.0, noun.1),
                if given == 1 {
                    "1 was".to_string()
                } else {
                    format!("{given} were")
                }
            );
            self.error(codes::WRONG_ARG_COUNT, call.span, message);
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
                format!(
                    "`{callee}` {verb} {}{listed}, but {} {verb_s} missing",
                    plural(positional.len(), noun.0, noun.1),
                    list_names(&missing)
                )
            } else {
                format!(
                    "`{callee}` {verb} {}, but {what}",
                    plural(positional.len(), noun.0, noun.1)
                )
            };
            self.error(codes::WRONG_ARG_COUNT, call.span, message);
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
}
