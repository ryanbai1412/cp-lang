//! Calls: fns, constructors, and conversions.

use super::expr::{list_names, plural};
use super::*;
use crate::ty::{Bindings, Param};
use cpl_diag::habits;
use cpl_diag::msgs;
use cpl_syntax::ast::{Arg, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind, Type};

mod construct;
mod declared_ret;

impl<'a> Checker<'a> {
    /// Whether `e` writes or reads in O(1): a number, a character, a tuple of
    /// them, or a string of fixed text around them.
    pub(crate) fn fixed_size(&mut self, e: &Expr) -> bool {
        if let ExprKind::Str(pieces) = &e.kind {
            return pieces.iter().all(|p| match p {
                cpl_syntax::ast::StrPiece::Text(_) => true,
                cpl_syntax::ast::StrPiece::Expr(e) => self.fixed_size(e),
                cpl_syntax::ast::StrPiece::Spec(e, _) => self.fixed_size(e),
            });
        }
        if let ExprKind::If {
            branches,
            otherwise: Some(otherwise),
        } = &e.kind
        {
            return branches.iter().map(|(_, b)| b).chain([otherwise]).all(|b| {
                match b.stmts.last() {
                    Some(Stmt {
                        kind: StmtKind::Expr(e),
                        ..
                    }) => self.fixed_size(e),
                    _ => false,
                }
            });
        }
        let ty = self.infer_quietly(e);
        self.io_fixed(&ty)
    }

    /// A bare `read()` or `[]`, or an `ask(..)`, takes its type
    /// from where its value goes (R10.2.1): record that type as its own, so
    /// its facts say what it is.
    pub(crate) fn note_read(&mut self, value: &Expr, want: &Ty) {
        if matches!(&value.kind, ExprKind::List(items) if items.is_empty())
            && !want.is_vague()
            && *want != Ty::Error
        {
            self.exprs.types.push((value.span, want.clone()));
            return;
        }
        let ExprKind::Call { callee, args } = &value.kind else {
            return;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        let reads = (matches!(name.as_str(), "read" | "gen") && args.is_empty())
            || name == "ask"
            || self.returns_only_its_own_param(name);
        if reads && self.local(name).is_none() && !want.is_vague() && *want != Ty::Error {
            if name == "gen" {
                self.gens_given_a_type.insert(value.span);
            }
            self.exprs.types.push((value.span, want.clone()));
        }
    }

    /// Whether `name` is a generic fn whose result is a type parameter none of
    /// its parameters names, `fn mk<T: Default>() -> T`: only the type its
    /// value goes to can bind it.
    fn returns_only_its_own_param(&mut self, name: &str) -> bool {
        let Some(Def::Fn {
            ty: Ty::Fn { params, ret },
            generics,
            ..
        }) = self.global(name).as_deref().cloned()
        else {
            return false;
        };
        let Ty::Param(p) = &*ret else {
            return false;
        };
        let names_p = |t: &Ty| {
            let mut hit = false;
            t.map(&mut |x| {
                hit |= matches!(x, Ty::Param(q) if q == p);
                None
            });
            hit
        };
        generics.contains(p) && !params.iter().any(|q| names_p(&q.ty))
    }

    /// [`Checker::narrowed`], after [`Checker::note_read`] on `at`.
    pub(crate) fn read_narrowed(&mut self, got: &Ty, want: &Ty, at: &Expr) -> Ty {
        self.note_read(at, want);
        self.narrowed(got, want)
    }

    /// Whether a value of `ty` is read or written in a bounded number of
    /// characters: a fixed-size value, a position in a domain of integers,
    /// a variant of an enum without fields, or a tuple of them.
    pub(crate) fn io_fixed(&mut self, ty: &Ty) -> bool {
        match ty {
            Ty::Tuple(items) => items.iter().all(|t| self.io_fixed(t)),
            Ty::Pos(d) => match self.global(d).as_deref() {
                Some(Def::Domain(info)) if info.factors.is_empty() => {
                    info.text.contains("..") && !info.text.contains(['[', '"', '\''])
                }
                Some(Def::Domain(info)) => {
                    let factors: Vec<Ty> = info.factors.iter().map(|(_, t)| t.clone()).collect();
                    factors.iter().all(|t| self.io_fixed(t))
                }
                _ => false,
            },
            Ty::Named { name, args } if args.is_empty() => matches!(
                self.type_def(name).as_deref(),
                Some(Def::Enum { variants, .. }) if variants.iter().all(|(_, fields)| fields.is_empty())
            ),
            _ => ty.is_fixed_size(),
        }
    }

    /// What writing `args` costs: `Some([])` when each has a fixed size,
    /// a term such as `|V| * |W|` for each array over domains of
    /// fixed-size values and `xs.len` for each named list of them, `None`
    /// when some size is unbounded.
    pub(crate) fn io_terms(&mut self, args: &[Arg]) -> Option<Vec<String>> {
        let mut terms = Vec::new();
        for arg in args {
            if self.fixed_size(&arg.value) {
                continue;
            }
            let ty = self.infer_quietly(&arg.value);
            if let (ExprKind::Name(name), Ty::Named { name: list, args }) = (&arg.value.kind, &ty) {
                if list == "List" && args.len() == 1 && self.io_fixed(&args[0]) {
                    terms.push(format!("{name}.len"));
                    continue;
                }
            }
            let Ty::Array { index, elem } = ty else {
                return None;
            };
            let domains: Option<Vec<String>> = index
                .iter()
                .map(|t| match t {
                    Ty::Pos(d) | Ty::Domain(d) => Some(format!("|{d}|")),
                    _ => None,
                })
                .collect();
            if !self.io_fixed(&elem) {
                return None;
            }
            terms.push(domains?.join(" * "));
        }
        Some(terms)
    }

    /// Records the cost of `print` or `ask` of `args` at `callee`, when
    /// every argument's size is bounded.
    pub(crate) fn record_io(&mut self, callee: Span, name: &str, args: &[Arg]) {
        match self.io_terms(args) {
            Some(terms) if terms.is_empty() => self.fixed_io.push(callee),
            Some(terms) => self.io_costs.push((
                callee,
                name.to_string(),
                format!("O({})", terms.join(" + ")),
            )),
            None => {}
        }
    }

    pub(super) fn call_unrecorded(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Ty {
        if let Some(picked) = self.overload_for(callee, args) {
            return self.call_unrecorded(call, &picked, args);
        }
        if let ExprKind::Name(fn_name) = &callee.kind {
            if let Some(caps) = self
                .local_fn_captures
                .iter()
                .rev()
                .find(|(n, _)| n == fn_name)
                .map(|(_, c)| c.clone())
            {
                for cap in caps {
                    if self.local(&cap).is_none() {
                        self.report(
                            callee.span,
                            msgs::check::USED_BEFORE_SET_CAPTURES_WHICH_IS_NOT,
                            &[&fn_name, &cap],
                        );
                        break;
                    }
                }
            }
        }
        if let (ExprKind::Name(n), [arg]) = (&callee.kind, args) {
            if n == "count"
                && arg.name.is_none()
                && !Self::generator_test(callee, args)
                && self.local(n).is_none()
            {
                let over = self.infer_quietly(&arg.value);
                self.bool_count(call, &over);
            }
        }
        if Self::binary_min_max(callee, args) || Self::generator_test(callee, args) {
            for arg in args {
                self.infer(&arg.value);
            }
            if let Some(ty) = self.prelude_call(call, callee, args) {
                return ty;
            }
        }
        if let ExprKind::Generic { base, args: tys } = &callee.kind {
            if matches!(&base.kind, ExprKind::Name(n) if n == "gen" && self.local(n).is_none() && self.global(n).is_none())
            {
                let ty = match tys.as_slice() {
                    [t] => {
                        let ty = self.quietly(|c| c.type_arg(t));
                        // `gen<T>(args)` on a type that draws itself is `T.gen(args)`;
                        // a domain draws a position.
                        if !matches!(ty, Ty::Pos(_) | Ty::Param(_)) {
                            if let Some(own) = t.gen_call(args, call.span) {
                                return self.infer(&own);
                            }
                        }
                        ty
                    }
                    _ => Ty::Unknown,
                };
                for arg in args {
                    self.infer(&arg.value);
                }
                return if matches!(ty, Ty::Error) {
                    Ty::Unknown
                } else {
                    ty
                };
            }
        }
        if let Some(on_self) = self.implicit_self_method(callee) {
            return self.call(call, &on_self, args);
        }
        if let Some(ty) = self.builtin_getter_call(call, callee, args) {
            self.record_getter(call, callee, true);
            return ty;
        }
        if let Some(ty) = self.getter_call(callee, args) {
            self.record_getter(call, callee, false);
            return ty;
        }
        if let Some(ty) = self.filled(callee, args) {
            return ty;
        }
        if let Some(ty) = self.grid_shape_call(callee, args) {
            return ty;
        }
        if let Some(ty) = self.list_shape_call(callee, args) {
            return ty;
        }
        if let Some(ty) = self.sorted_range_call(call, callee, args) {
            return ty;
        }
        if let Some(ty) = self.sorted_legacy_bound_call(call, callee, args) {
            return ty;
        }
        let callee_ty = match self.list_form(callee, args) {
            Some(ty) => ty,
            None => {
                let prev = std::mem::replace(&mut self.in_call_callee, true);
                let ty = self.infer(callee);
                self.in_call_callee = prev;
                ty
            }
        };
        let receiver = self.receiver(call, callee, args);
        self.free_call_mut_args(callee, args);
        let given = self.lambda_params.len();
        for arg in args {
            let params = self.catalog_arg(callee, args, arg);
            if !params.is_empty() {
                self.lambda_params.push((arg.value.span, params));
            }
        }
        self.sort_key_by_position(callee, args);
        let past_row = self.past_builtin_row(callee, args);
        if !past_row && self.member_hides_extension(callee, args) {
            for arg in args {
                self.infer(&arg.value);
            }
            return Ty::Error;
        }
        let library = past_row
            .then(|| self.method_form_call(call, callee, args))
            .flatten();
        let mut ty = if let Some(ty) = library {
            ty
        } else if receiver.is_some()
            && !matches!(
                callee_ty,
                Ty::Fn { .. } | Ty::Unknown | Ty::Error | Ty::TypeName(_)
            )
        {
            for arg in args {
                self.infer(&arg.value);
            }
            Ty::Unknown
        } else {
            self.call_kind(call, callee, callee_ty.clone(), args)
        };
        if receiver.is_none() && matches!(ty, Ty::Unknown) {
            if let Some(known) = self.catalog_ret(callee, args) {
                ty = known;
            }
        }
        if let Some(known) = self.prelude_call(call, callee, args) {
            ty = known;
        }
        if matches!(ty, Ty::Unknown) {
            if let Some(known) = self.method_form_call(call, callee, args) {
                ty = known;
            }
        }
        if matches!(ty, Ty::Unknown) {
            if let Some(known) = self.free_as_method(call, callee, args) {
                ty = known;
            }
        }
        if matches!(ty, Ty::Unknown) {
            if let Some(read) = self.read_ty(callee, args) {
                ty = read;
            }
        }
        let converts_one = match &callee.kind {
            ExprKind::Name(n) if n == "print" => {
                self.record_io(callee.span, n, args);
                self.exprs.types.push((call.span, Ty::Unit));
                false
            }
            ExprKind::Name(n) => {
                ["Int", "I128", "U64", "F64", "Char", "Bool"].contains(&n.as_str())
                    && args.len() == 1
            }
            _ => false,
        };
        if converts_one && self.fixed_size(&args[0].value) {
            self.fixed_io.push(callee.span);
        }
        let converts = match &callee_ty {
            Ty::Domain(_) => true,
            Ty::TypeName(name) | Ty::Named { name, .. } => match self.type_def(name).as_deref() {
                Some(Def::Alias {
                    ty: Ty::Named { name: inner, .. },
                    ..
                }) => {
                    let inner = inner.clone();
                    self.is_number(&inner)
                }
                _ => self.is_number(name),
            },
            _ => false,
        };
        if converts && !args.is_empty() && args.iter().all(|a| self.fixed_size(&a.value)) {
            self.fixed_io.push(callee.span);
        }
        if let ExprKind::Generic { base, args: tys } = &callee.kind {
            if matches!(&base.kind, ExprKind::Name(n) if n == "read") && args.is_empty() {
                if let [t] = tys.as_slice() {
                    let t = self.resolve_type(t);
                    if t == Ty::Bool {
                        self.not_a_token_type(callee.span);
                    }
                    if self.io_fixed(&t) {
                        self.fixed_io.push(base.span);
                    }
                }
            }
        }
        self.lambda_params.truncate(given);
        self.empty_collection(call, callee, args);
        self.lent_together(args);
        // `f(mut a)` may change `a` as an assignment to it would.
        for arg in args.iter().filter(|a| a.mutable) {
            self.check_access(&arg.value, true);
        }
        self.call_effects(call, callee, args);
        self.sort_key_uses(callee, args);
        self.debug_changes(callee, args);
        self.no_escape_args(callee, &callee_ty, args);
        self.check_store_domain_escape(callee, args);
        if !self.own_non_mutator(callee) {
            self.mutates_view(call, callee);
        }
        self.mutates_self(callee);
        receiver.unwrap_or(ty)
    }

    /// R06.1.6: the callee naming the overload of `name(..)` or
    /// `module.name(..)` whose labels the call gives, when the name has
    /// several and that one is not the first.
    fn overload_for(&mut self, callee: &Expr, args: &[Arg]) -> Option<Expr> {
        let (alias, name) = match &callee.kind {
            ExprKind::Name(name) if self.local(name).is_none() => (None, name.clone()),
            ExprKind::Field { base, name } => match &base.kind {
                ExprKind::Name(m) if self.local(m).is_none() && self.env.is_module(m) => {
                    (Some(m.clone()), name.text.clone())
                }
                _ => return None,
            },
            _ => return None,
        };
        if crate::prelude::overload_part(&name).is_some() {
            return None;
        }
        let path = |key: &str| match &alias {
            Some(m) => format!("{m}.{key}"),
            None => key.to_string(),
        };
        let mut keys: Vec<String> = vec![name.clone()];
        match &alias {
            None => keys.extend(
                (self.env.method_keys(&name).into_iter())
                    .filter(|k| crate::prelude::overload_part(k).is_some()),
            ),
            Some(_) => keys.extend(
                (2..)
                    .map(|k| crate::prelude::overload_key(&name, k))
                    .take_while(|k| self.global(&path(k)).is_some()),
            ),
        }
        if keys.len() < 2 {
            return None;
        }
        let fits: Vec<String> = (keys.into_iter())
            .filter(|k| {
                self.global(&path(k))
                    .is_some_and(|d| super::method_form::def_labels_fit(&d, args))
            })
            .collect();
        let [one] = fits.as_slice() else {
            return None;
        };
        if *one == name {
            return None;
        }
        let kind = match &callee.kind {
            ExprKind::Field { base, name } => ExprKind::Field {
                base: base.clone(),
                name: cpl_syntax::ast::Name {
                    text: one.clone(),
                    span: name.span,
                },
            },
            _ => ExprKind::Name(one.clone()),
        };
        Some(Expr {
            kind,
            span: callee.span,
        })
    }

    /// `read<T>()` (optionally with `base:` / `none:` / `gen:`) is a `T`.
    fn read_ty(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Generic { base, args: tys } = &callee.kind else {
            return None;
        };
        let (ExprKind::Name(name), [t]) = (&base.kind, tys.as_slice()) else {
            return None;
        };
        if !args.iter().all(|a| {
            a.name
                .as_ref()
                .is_some_and(|n| matches!(n.text.as_str(), "base" | "none" | "gen" | "from"))
        }) {
            return None;
        }
        if !matches!(name.as_str(), "read" | "read_output")
            || self.local(name).is_some()
            || self.global(name).is_some()
        {
            return None;
        }
        let ty = self.quietly(|c| c.type_arg(t));
        let from = args
            .iter()
            .any(|a| a.name.as_ref().is_some_and(|n| n.text == "from"));
        let ty = if from { Ty::Optional(Box::new(ty)) } else { ty };
        (!matches!(ty, Ty::Unknown | Ty::Error)).then_some(ty)
    }

    /// `List.filled(n, x)` is a list of what `x` is, widened as a list literal's
    /// element is (`let inf = 1e9` fills a `List<Int>`). A whole number
    /// stands for any number type, so it also fills a `List<Z>`.
    fn filled(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        if let (ExprKind::List(index), [value]) = (&base.kind, args) {
            if name.text != "filled" || index.is_empty() {
                return None;
            }
            let index = index
                .iter()
                .map(|e| self.resolve_type(&Type::of_index(e)))
                .collect();
            let elem = self.infer(&value.value).widen();
            return Some(Ty::Array {
                index,
                elem: Box::new(elem),
            });
        }
        let value = self.list_fill(callee, args)?;
        self.infer(&args[0].value);
        let elem = self.infer(&value.value).widen();
        Some(Ty::list(elem))
    }

    /// The `x` of a prelude `List.filled(n, x)`: each of its cells is a copy.
    pub(crate) fn list_fill<'e>(&mut self, callee: &Expr, args: &'e [Arg]) -> Option<&'e Arg> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let prelude = matches!(&base.kind, ExprKind::Name(list)
            if list == "List" && self.local(list).is_none() && self.global(list).is_none());
        match args {
            [_, value] if prelude && name.text == "filled" => Some(value),
            _ => None,
        }
    }

    /// Records `call` as a call of `type_name`'s projection `name`, if it has one.
    fn note_projection(&mut self, call: &Expr, type_name: &str, name: &str) -> Option<()> {
        let members = self.members_of(type_name)?;
        let projection = members.projections.into_iter().find(|p| p.name == name)?;
        self.note_proj_call(
            call,
            super::lends::ProjCall {
                who: format!("{type_name}.{}", projection.name),
                name: projection.name,
                mutable: projection.mutable,
                lent: projection.lent,
                law: projection.law,
            },
        );
        Some(())
    }

    /// `s.range(r)` on `SortedBag<K>`, `SortedSet<K>`, `Set<K>`, or `Map<K, V>`:
    /// checks `r`'s bounds against the key; the call returns what `range` is declared to return
    /// (`a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..`, or a `Range<K>` value).
    fn sorted_range_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        if name.text != "range" {
            return None;
        }
        let base_ty = self.infer(base);
        let Ty::Named {
            name: cname,
            args: targs,
        } = &base_ty
        else {
            return None;
        };
        let k_ty = match (cname.as_str(), targs.as_slice()) {
            ("Set", [k]) | ("Map", [k, _]) => k.clone(),
            _ => self.sorted_view_key(cname, targs)?.clone(),
        };
        let ret = self
            .declared_ret(base, &base_ty, "range", call.span)
            .unwrap_or(Ty::Unknown);
        self.note_projection(call, cname, "range");
        if args.len() != 1 || args[0].name.is_some() {
            for a in args {
                self.infer(&a.value);
            }
            self.report(
                call.span,
                msgs::check::WRONG_ARG_COUNT_RANGE_TAKES_1_RANGE,
                &[&cname, &args.len()],
            );
            return Some(ret);
        }
        let arg = &args[0].value;
        match &arg.kind {
            ExprKind::Range { lo, hi, step, .. } => {
                if let Some(st) = step {
                    self.infer(st);
                    self.report(
                        st.span,
                        msgs::check::TYPE_MISMATCH_RANGE_DOES_NOT_TAKE,
                        &[&cname],
                    );
                }
                let check_end = |this: &mut Self, e: &Expr| {
                    let ety = this.infer(e);
                    if matches!(ety, Ty::Error | Ty::Unknown) {
                        return;
                    }
                    let fits_bound = match (&k_ty, &ety) {
                        (Ty::Pos(d), Ty::Pos(bd)) => bd.strip_suffix(".cuts") == Some(d.as_str()),
                        (Ty::Pos(d), Ty::Named { name, args }) if name == "Cut" => {
                            matches!(args.as_slice(), [Ty::Pos(bd)] if bd == d)
                        }
                        _ => false,
                    };
                    if !fits_bound {
                        this.expect_fits(&ety, &k_ty, e, &|c, got, want| {
                            cpl_diag::msgs::check::RANGE_BOUND_EXPECTED.at(&[
                                &c.quote(e.span),
                                &got.with_article(),
                                &want.with_article(),
                            ])
                        });
                    }
                };
                if let Some(l) = lo {
                    check_end(self, l);
                }
                if let Some(h) = hi {
                    check_end(self, h);
                }
            }
            _ => {
                let want = Ty::Named {
                    name: "Range".into(),
                    args: vec![k_ty.clone()],
                };
                let got = self.infer(arg);
                if !matches!(got, Ty::Error | Ty::Unknown) {
                    self.expect_fits(&got, &want, arg, &|c, g, w| {
                        cpl_diag::msgs::check::EXPECTED.at(&[
                            &c.quote(arg.span),
                            &g.with_article(),
                            &w.with_article(),
                        ])
                    });
                }
            }
        }
        Some(ret)
    }

    /// Migration codemod (`cpl fix`) for removed `SortedBag`/`SortedSet`
    /// bound-query methods (`first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range`).
    fn sorted_legacy_bound_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        if !matches!(
            name.text.as_str(),
            "first_ge" | "first_gt" | "last_le" | "last_lt" | "count_range"
        ) {
            return None;
        }
        let base_ty = self.infer(base);
        let Ty::Named {
            name: cname,
            args: targs,
        } = &base_ty
        else {
            return None;
        };
        if self.struct_method(cname, &name.text).is_some() {
            return None;
        }
        let k_ty = self.sorted_view_key(cname, targs)?;
        for a in args {
            self.infer(&a.value);
        }
        let (replacement, ret_ty) = match (name.text.as_str(), args) {
            ("first_ge", [a]) => (
                format!("range({}..).first", self.quote(a.value.span)),
                Ty::Optional(Box::new(k_ty)),
            ),
            ("last_le", [a]) => (
                format!("range(..={}).last", self.quote(a.value.span)),
                Ty::Optional(Box::new(k_ty)),
            ),
            ("last_lt", [a]) => (
                format!("range(..{}).last", self.quote(a.value.span)),
                Ty::Optional(Box::new(k_ty)),
            ),
            ("first_gt", [a]) => {
                let q = self.quote(a.value.span);
                let repl = if matches!(k_ty, Ty::Int | Ty::I128 | Ty::U64) {
                    format!("range({q} + 1..).first")
                } else {
                    format!("first_true(x => x > {q})")
                };
                (repl, Ty::Optional(Box::new(k_ty)))
            }
            ("count_range", [lo, hi]) => (
                format!(
                    "range({}..{}).len",
                    self.quote(lo.value.span),
                    self.quote(hi.value.span)
                ),
                Ty::Int,
            ),
            _ => return None,
        };
        let replace_span = Span::new(name.span.start, call.span.end);
        if let Some(diag) = self.report(
            call.span,
            msgs::check::UNKNOWN_FIELD_WAS_REPLACED_BY,
            &[&cname, &name.text, &replacement],
        ) {
            diag.help = Some(format!(
                "write `{replacement}` (or run `cpl fix` to migrate)"
            ));
            diag.fix(
                format!("replace `{}` with `{replacement}`", name.text),
                replace_span,
                replacement,
            );
        }
        Some(ret_ty)
    }

    fn mutates_self(&mut self, callee: &Expr) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        let base_ty = self.base_ty(base);
        if self.declares_field(&base_ty, &name.text) {
            return;
        }
        if matches!(name.text.as_str(), "set_at" | "swap_idx") && base_ty.is_indexed_mut() {
            self.require_mutable(base, &format!("changed by `{}`", name.text));
            return;
        }
        if self.changes_self(&base_ty, &name.text) {
            self.require_mutable(base, &format!("changed by `{}`", name.text));
        }
    }

    /// Built-in `MemberKind::Get` properties (`len`, `is_empty`, `first`,
    /// `last`, `rev`, `items`) are read without parentheses.
    /// `xs.len`, `x.sqrt`: a getter the compiler gives `base_ty`, with no
    /// declaration of its own.
    pub(super) fn builtin_get(&self, base_ty: &Ty, name: &str) -> bool {
        let is_builtin_get = match base_ty {
            Ty::Named { name: n, args } if n == "List" && args.len() == 1 => {
                crate::builtins::builtin_members(crate::builtins::Receiver::List)
                    .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name)
            }
            Ty::Str => crate::builtins::builtin_members(crate::builtins::Receiver::Str)
                .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name),
            Ty::Named { name: n, args }
                if matches!(n.as_str(), "Map" | "HashMap") && args.len() == 2
                    || matches!(n.as_str(), "Set" | "HashSet") && args.len() == 1 =>
            {
                let receiver = crate::builtins::Receiver::ALL
                    .into_iter()
                    .find(|r| r.name() == n);
                receiver.is_some_and(|r| {
                    crate::builtins::builtin_members(r)
                        .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name)
                })
            }
            Ty::Array { .. } => crate::builtins::builtin_members(crate::builtins::Receiver::Array)
                .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name),
            Ty::Named { name: n, .. } if n == "Range" => {
                matches!(
                    name,
                    "len" | "is_empty" | "first" | "last" | "rev" | "start" | "end"
                )
            }
            Ty::Domain(_) => {
                matches!(name, "len" | "first" | "last" | "rev" | "bounds" | "cuts")
            }
            t if t.is_int() => matches!(name, "abs" | "ord" | "sqrt"),
            Ty::Float => matches!(name, "abs" | "ceil" | "floor" | "round" | "sqrt" | "trunc"),
            _ => false,
        };
        // `xs.rev`: every `Bidir` sequence's reverse view, a std `@get`.
        let is_builtin_get = is_builtin_get || (name == "rev" && base_ty.is_bidir());
        // `g.rows()`: a std method on arrays that is only ever `@get`; a trait
        // method of the same name (`5.rev()`, `Reversible`) is not one.
        let is_builtin_get = is_builtin_get
            || (matches!(base_ty, Ty::Array { .. })
                && crate::prelude::getter_methods().iter().any(|g| g == name));
        is_builtin_get
    }

    fn builtin_getter_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        if !args.is_empty() {
            return None;
        }
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let base_ty = self.infer_quietly(base);
        if !self.builtin_get(&base_ty, &name.text) {
            return None;
        }
        let callee_text = self.quote(callee.span);
        if let Some(diag) = self.report(
            call.span,
            msgs::check::NOT_CALLABLE_IS_READ_WITHOUT_PARENTHESES,
            &[&name.text, &callee_text],
        ) {
            diag.help = Some(format!(
                "write `{callee_text}` (or run `cpl fix` to drop `()`)"
            ));
            let parens = Span::new(callee.span.end, call.span.end);
            diag.fix(format!("drop `()` to read `{callee_text}`"), parens, "");
        }
        Some(Ty::Error)
    }

    /// `x.inv()`: a `@get` method of a struct reads the same with empty
    /// parentheses, unless what it gives is itself a fn to call.
    fn getter_call(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let Ty::Named {
            name: type_name, ..
        } = self.infer_quietly(base)
        else {
            return None;
        };
        let is_getter = |getters: &[String]| getters.contains(&name.text);
        let found = match self.type_def(&type_name).as_deref() {
            Some(Def::Struct { getters, .. }) => is_getter(getters),
            _ => false,
        } || self
            .env
            .impls(&type_name)
            .into_iter()
            .any(|(_, def)| matches!(&*def, Def::Impl { getters, .. } if is_getter(getters)));
        if !found {
            return None;
        }
        let prev = std::mem::replace(&mut self.in_call_callee, true);
        let inferred_callee = self.infer(callee);
        self.in_call_callee = prev;
        match inferred_callee {
            Ty::Fn { .. } => None,
            // `p.shifted(by: 3)`: a `@get` method given its defaulted arguments.
            ty => {
                self.plain_args(args);
                Some(ty)
            }
        }
    }

    /// `Graph.undirected(pairs)`: a static fn given a plain list of pairs
    /// or triples is the fn its module declares for that, named
    /// `<fn>_of_pairs` or `<fn>_of_triples`.
    fn list_form(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        // `Graph<V>.undirected(pairs)` is the same fn as `Graph.undirected(pairs)`:
        // type arguments on the receiver fill the fn's like-named generics,
        // they never pick a different fn.
        let (type_name, type_args) = match &base.kind {
            ExprKind::Name(type_name) => (type_name, &[][..]),
            ExprKind::Generic { base, args } => match &base.kind {
                ExprKind::Name(type_name) => (type_name, args.as_slice()),
                _ => return None,
            },
            _ => return None,
        };
        let [only] = args else {
            return None;
        };
        if self.local(type_name).is_some() {
            return None;
        }
        let Some(Def::Struct {
            generics: struct_generics,
            ..
        }) = self.type_def(type_name).as_deref().cloned()
        else {
            return None;
        };
        let given = self.infer_quietly(&only.value);
        let array = matches!(given, Ty::Array { .. });
        let suffix = match given.clone() {
            Ty::Named { name: list, args } if list == "List" => match args.as_slice() {
                [Ty::Tuple(fields)] => fields.len(),
                _ => return None,
            },
            // An array over a plain range names no edge domain either,
            // and the cells of one of plain tuples have no `u` and `v`.
            Ty::Array { index, elem }
                if matches!(index.as_slice(), [Ty::Int])
                    || (index.len() == 1 && matches!(*elem, Ty::Tuple(_))) =>
            {
                match *elem {
                    Ty::Tuple(fields) => fields.len(),
                    Ty::Record(fields) => fields.len(),
                    _ => return None,
                }
            }
            _ => return None,
        };
        let suffix = match suffix {
            2 => "pairs",
            3 => "triples",
            _ => return None,
        };
        let Some(Def::Fn { ty, generics, .. }) = self
            .type_def(&format!("{}_of_{suffix}", name.text))
            .as_deref()
            .cloned()
        else {
            return None;
        };
        // `Graph<V, E>.directed(pairs)` meets `Graph`'s bounds as `Graph<V, E>.directed(edges)`
        // does (R08.2.5): the receiver is the same type either way.
        if !type_args.is_empty() {
            self.infer(base);
        }
        let mut bindings = Bindings::default();
        for (generic, arg) in struct_generics.iter().zip(type_args) {
            if generics.contains(generic) {
                let arg = self.type_arg(arg);
                bindings.insert(generic.clone(), arg);
            }
        }
        let ty = ty.subst(&bindings);
        match ty {
            // The array is read as the list of its cells.
            Ty::Fn { mut params, ret } if array && bindings.is_empty() => {
                if let Some(first) = params.first_mut() {
                    first.ty = given;
                }
                Some(Ty::Fn { params, ret })
            }
            Ty::Fn { .. } => Some(ty),
            _ => None,
        }
    }

    /// `List<T>()` and `MinHeap<T>()` start empty: they take nothing.
    fn empty_collection(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) {
        let ExprKind::Generic { base, .. } = &callee.kind else {
            return;
        };
        let ExprKind::Name(name) = &base.kind else {
            return;
        };
        let prelude = name == "List" && self.local(name).is_none() && self.global(name).is_none();
        if !prelude || args.is_empty() {
            return;
        }
        self.report(
            call.span,
            msgs::check::WRONG_ARG_COUNT_T_TAKES_NO_ARGUMENTS,
            &[&name],
        );
    }

    /// What a method call means beyond its signature: `g.row(r)` lends a
    /// row, `v.l(k)` is a boundary, `b.at(i)` is a projection of `b`.
    /// Returns the call's type when the receiver decides it.
    /// R08.1.4: `s.m()` where two traits `S` implements both define `m`
    /// and `S` has no `m` of its own.
    fn ambiguous_method(
        &mut self,
        base_ty: &Ty,
        base: &Expr,
        name: &cpl_syntax::ast::Name,
    ) -> bool {
        let Ty::Named { name: owner, .. } = base_ty else {
            return false;
        };
        let Some(traits) = self.member_traits(owner, &name.text) else {
            return false;
        };
        let [first, second, ..] = traits.as_slice() else {
            return false;
        };
        let shown = self.quote(base.span);
        let method = &name.text;
        if let Some(diag) = self.report(
            name.span,
            msgs::check::AMBIGUOUS_METHOD_IS_AMBIGUOUS_BOTH_AND,
            &[&shown, &method, &first, &second, &owner, &method],
        ) {
            diag.help = Some(format!("name the trait: `{first}.{method}({shown})`"));
        }
        true
    }

    fn receiver(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let base_ty = self.base_ty(base);
        if self.ambiguous_method(&base_ty, base, name) {
            return Some(Ty::Error);
        }
        if name.text == "count" && args.is_empty() {
            self.bool_count(call, &base_ty);
        }
        // R04.2.17: `V.cuts(k)` is the cut numbered `k`, not the range of cuts.
        if let (Ty::Domain(d), "cuts" | "bounds", [arg]) = (&base_ty, name.text.as_str(), args) {
            if arg.name.is_none() {
                let got = self.infer(&arg.value);
                self.expect_fits(&got, &Ty::Int, &arg.value, &|c, got, _| {
                    cpl_diag::msgs::check::TAKES_INT_BUT.at(&[
                        &c.quote(callee.span),
                        &c.quote(arg.value.span),
                        &got.with_article(),
                    ])
                });
                return Some(Ty::Pos(format!("{d}.cuts")));
            }
        }
        // Fixed array `.map(f)` keeps the shape: `[D, E]T -> [D, E]U`.
        if name.text == "map" {
            if let Ty::Array {
                ref index,
                ref elem,
            } = base_ty
            {
                let out = if let [arg] = args {
                    match &arg.value.kind {
                        ExprKind::Lambda {
                            params,
                            param_tys,
                            body,
                            ..
                        } if let Some(param) = Pattern::single_param(params) => self.quietly(|c| {
                            c.push_scope();
                            let pty = param_tys
                                .first()
                                .filter(|_| params.len() == 1)
                                .and_then(|t| t.as_ref())
                                .map(|t| c.resolve_type(t))
                                .unwrap_or_else(|| (**elem).clone());
                            c.bind(&param, &pty, false, "param");
                            let ret = c.infer(body).widen();
                            c.pop_scope();
                            ret
                        }),
                        _ => match self.infer_quietly(&arg.value) {
                            Ty::Fn { ret, .. } => (*ret).clone(),
                            _ => Ty::Unknown,
                        },
                    }
                } else {
                    Ty::Unknown
                };
                return Some(Ty::Array {
                    index: index.clone(),
                    elem: Box::new(out),
                });
            }
        }
        // Sequence higher-order operations: `map`, `filter`, `partition`,
        // `group`, `first(pred)`, `sorted`, through one unified `seq_elem()` path.
        // A struct's own method of the name wins: `LinkedList.reverse` is not `seq.reverse`.
        let seq_elem: Option<Ty> = base_ty.seq_elem().or_else(|| match &base_ty {
            Ty::Named { .. } if self.declares_member(&base_ty, &name.text) => None,
            Ty::Named { .. } => match self.element_ty(&base_ty) {
                Ty::Unknown | Ty::Error => None,
                elem => Some(elem),
            },
            _ => None,
        });
        // `a.reverse()` on a `LinkedList`: a type's own fn, not the built-in row of the name.
        let seq_elem = seq_elem.filter(|_| !self.declares_member(&base_ty, &name.text));
        if let Some(elem) = seq_elem {
            self.removed_seq_spelling(name, args, &elem);
            match name.text.as_str() {
                "map" => {
                    let out = if let [arg] = args {
                        match &arg.value.kind {
                            ExprKind::Lambda {
                                params,
                                param_tys,
                                body,
                                ..
                            } if let Some(param) = Pattern::single_param(params).filter(|_| {
                                params.len() == 1
                                    || elem.param_fields().is_some_and(|f| f.len() == params.len())
                            }) =>
                            {
                                self.quietly(|c| {
                                    c.push_scope();
                                    let pty = param_tys
                                        .first()
                                        .filter(|_| params.len() == 1)
                                        .and_then(|t| t.as_ref())
                                        .map(|t| c.resolve_type(t))
                                        .unwrap_or_else(|| elem.clone());
                                    c.bind(&param, &pty, false, "param");
                                    let ret = c.infer(body).widen();
                                    c.pop_scope();
                                    ret
                                })
                            }
                            ExprKind::Lambda { params, .. } if params.len() > 1 => {
                                self.lambda_takes_elem(params.len(), &elem, arg.value.span)
                            }
                            _ => self.fn_value_takes("map", &arg.value, 1),
                        }
                    } else {
                        Ty::Unknown
                    };
                    return Some(Ty::list(out));
                }
                "partition" => {
                    return Some(Ty::Tuple(vec![Ty::list(elem.clone()), Ty::list(elem)]));
                }
                "first" => {
                    return Some(Ty::Optional(Box::new(elem)));
                }
                "last"
                    if matches!(
                        args,
                        [arg] if arg.name.is_none()
                            && (matches!(arg.value.kind, ExprKind::Lambda { .. })
                                || matches!(self.infer_quietly(&arg.value), Ty::Fn { .. }))
                    ) =>
                {
                    return Some(Ty::Optional(Box::new(elem)));
                }
                "filter" => {
                    if let [arg] = args {
                        self.fn_value_takes("filter", &arg.value, 1);
                    }
                    return Some(Ty::Named {
                        name: "Filter".into(),
                        args: vec![elem],
                    });
                }
                "sorted" => {
                    return Some(Ty::list(elem));
                }
                "argmax" | "argmin" if !self.implements_trait(&base_ty, "Keyed") => {
                    self.report(
                        name.span,
                        msgs::check::MISSING_IMPL_REQUIRES_KEYED_BUT_HAS,
                        &[&name.text, &self.quote(base.span), &base_ty],
                    );
                    return Some(Ty::Error);
                }
                "sort" | "reverse" if !matches!(base_ty, Ty::Domain(_)) => {
                    // A read-only view (`xs.rev`) is reported once, as not writable.
                    if !self.implements_trait(&base_ty, "IndexedMut")
                        && !self.is_readonly_view(base)
                    {
                        self.report(
                            name.span,
                            msgs::check::MISSING_IMPL_REQUIRES_INDEXEDMUT_BUT_HAS,
                            &[&name.text, &self.quote(base.span), &base_ty],
                        );
                        return Some(Ty::Error);
                    }
                    self.require_mutable(base, "changed in place (needs `mut`)");
                    self.reverse_view_help(&name.text, base);
                }
                "sort" if matches!(base_ty, Ty::Domain(_)) => {
                    return Some(Ty::list(elem));
                }
                _ => {}
            }
        }
        if name.text == "fold" {
            if let ExprKind::Call { args, .. } = &call.kind {
                if let [arg] = args.as_slice() {
                    if self.type_name_as_op(&arg.value) {
                        return Some(Ty::Error);
                    }
                }
                if let Some(acc) = self.seeded_fold(&base_ty, name, args) {
                    return Some(acc);
                }
                if let [arg] = args.as_slice() {
                    if arg.name.is_none() && self.fold_without_identity(&arg.value) {
                        let elem = self.element_ty(&base_ty);
                        if !matches!(elem, Ty::Unknown | Ty::Error) {
                            self.infer(&arg.value);
                            self.fold_is_reduce(name);
                            return Some(Ty::Optional(Box::new(elem)));
                        }
                    }
                }
            }
        }
        match base_ty {
            Ty::Optional(ref inner) if crate::builtins::optional_member(&name.text) => {
                if let ExprKind::Call { args, .. } = &call.kind {
                    if args.len() == 1 && args[0].name.is_none() {
                        let f = &args[0].value;
                        let fn_ret = match &f.kind {
                            ExprKind::Lambda {
                                params,
                                param_tys,
                                body,
                                ..
                            } if let Some(param) = Pattern::single_param(params) => {
                                self.push_scope();
                                let pty = param_tys
                                    .first()
                                    .filter(|_| params.len() == 1)
                                    .and_then(|o| o.as_ref())
                                    .map(|t| self.resolve_type(t))
                                    .unwrap_or_else(|| (**inner).clone());
                                self.bind(&param, &pty, false, "param");
                                let r = self.infer(body);
                                self.pop_scope();
                                r
                            }
                            _ => {
                                let fty = self.infer(f);
                                match fty {
                                    Ty::Fn { ret, .. } => *ret,
                                    Ty::Array { elem, .. } => *elem,
                                    _ => Ty::Unknown,
                                }
                            }
                        };
                        return Some(if name.text == "map" {
                            Ty::Optional(Box::new(fn_ret))
                        } else {
                            Ty::Optional(inner.clone())
                        });
                    }
                }
                None
            }
            Ty::Array {
                ref index,
                ref elem,
            } if name.text == "inverse" => {
                if let Some(ret) = self.array_inverse_ret(index, elem) {
                    Some(ret)
                } else if !elem.is_vague() {
                    self.report(
                        name.span,
                        msgs::check::TYPE_MISMATCH_INVERSE_REQUIRES_ELEMENTS_OF,
                        &[&elem],
                    );
                    Some(Ty::Error)
                } else {
                    None
                }
            }
            Ty::Array { index, elem } if name.text == "row" && index.len() > 1 => {
                self.note_view(call, "row");
                Some(Ty::Array {
                    index: index[1..].to_vec(),
                    elem,
                })
            }
            // `v.l(k)`, `v.r(k)`: the far end of a window of `k` at `v`.
            Ty::Pos(domain)
                if matches!(name.text.as_str(), "l" | "r") && !domain.ends_with(".cuts") =>
            {
                Some(Ty::Pos(format!("{domain}.cuts")))
            }
            Ty::Named {
                name: type_name, ..
            } => {
                self.note_projection(call, &type_name, &name.text);
                None
            }
            _ => None,
        }
    }

    /// Each `R: (u: V, ..)` bound holds of what `R` bound to, or the call is E0704.
    fn rows_hold(
        &mut self,
        call: &Expr,
        callee_text: &str,
        args: &[Arg],
        row_bounds: crate::def::RowBounds,
        bindings: &Bindings,
    ) {
        for (param, req_fields) in row_bounds {
            let Some(actual) = bindings.get(&param) else {
                continue;
            };
            match actual {
                Ty::Record(actual_fs) => {
                    for (req_name, req_ty) in &req_fields {
                        let found = actual_fs.iter().find(|(n, _)| n == req_name);
                        match found {
                            Some((_, got_ty)) if self.fits(got_ty, req_ty) => {}
                            _ => {
                                let help = super::expr::rename_fields(
                                    actual,
                                    &Ty::Record(req_fields.clone()),
                                );
                                let diag = self.report(
                                    call.span,
                                    msgs::check::ROW_MISSING_FIELD_NEEDS_A_RECORD_WITH,
                                    &[
                                        &callee_text,
                                        &req_name,
                                        &req_ty,
                                        &actual,
                                        &found.map_or(format!("has no `{req_name}`"), |(_, t)| {
                                            format!("has `{req_name}: {t}`")
                                        }),
                                    ],
                                );
                                if let Some(diag) = diag {
                                    if help.is_some() {
                                        diag.help = help;
                                    }
                                    suggest_rename(Some(diag), args, actual_fs, &req_fields);
                                }
                                break;
                            }
                        }
                    }
                }
                Ty::Error | Ty::Unknown | Ty::Param(_) => {}
                other => {
                    self.report(
                        call.span,
                        msgs::check::ROW_MISSING_FIELD_NEEDS_A_RECORD_FOR,
                        &[&callee_text, &param, &other],
                    );
                }
            }
        }
    }

    /// `Graph.directed(flights)`, `s.apply(f)`: a method's bounds, read off its declared
    /// parameters and the type it is called on, as a free fn's are.
    fn method_rows_hold(
        &mut self,
        call: &Expr,
        callee_text: &str,
        base: &Expr,
        method: &str,
        args: &[Arg],
        bindings: &Bindings,
    ) {
        let recv = self.quietly(|c| c.infer(base));
        let ty_name = match &recv {
            Ty::TypeName(n) | Ty::Named { name: n, .. } => n.clone(),
            _ => return,
        };
        let impls = self.env.impls(&ty_name).into_iter().map(|(_, d)| d);
        let mut found = None;
        for def in self.type_def(&ty_name).into_iter().chain(impls) {
            let (methods, rows, target) = match &*def {
                Def::Struct {
                    generics,
                    methods,
                    method_rows,
                    ..
                } => (
                    methods,
                    method_rows,
                    Ty::Named {
                        name: ty_name.clone(),
                        args: generics.iter().cloned().map(Ty::Param).collect(),
                    },
                ),
                Def::Impl {
                    methods,
                    method_rows,
                    target,
                    ..
                } => (methods, method_rows, target.clone()),
                _ => continue,
            };
            if let (Some((_, Ty::Fn { params, .. })), Some(row)) = (
                methods.iter().find(|m| m.0 == method),
                rows.iter().find(|r| r.0 == method),
            ) {
                found = Some((params.clone(), row.clone(), target));
                break;
            }
        }
        let Some((params, (_, rows, through, traits), target)) = found else {
            return;
        };
        let mut bindings = bindings.clone();
        target.unify(&recv, &mut bindings);
        let mut positional = params.iter().filter(|p| !p.named_only);
        for arg in args {
            let param = match &arg.name {
                Some(n) => params.iter().find(|p| p.name == n.text),
                None => positional.next(),
            };
            if let Some(param) = param {
                let got = self.infer_quietly(&arg.value);
                param.ty.unify(&got, &mut bindings);
            }
        }
        // `S: Indexed<E, R>`: `R` is what the `S` given holds.
        for (param, trait_name, trait_args) in &through {
            let held = self
                .seq_elem_role(trait_name)
                .and_then(|i| trait_args.get(i));
            let elem = bindings.get(param).and_then(|t| t.seq_elem());
            if let (Some(held), Some(elem)) = (held, elem) {
                held.unify(&elem, &mut bindings);
            }
        }
        self.method_bounds_hold(method, (&traits, &params), &bindings, (args, call));
        if !matches!(recv, Ty::TypeName(_)) {
            return;
        }
        // A tuple argument (`Graph.directed(pairs)`) is read by another path; only a record
        // can miss a field.
        bindings.retain(|_, t| matches!(t, Ty::Record(_)));
        self.rows_hold(call, callee_text, args, rows, &bindings);
    }

    fn call_kind(&mut self, call: &Expr, callee: &Expr, callee_ty: Ty, args: &[Arg]) -> Ty {
        let callee_text = self.quote(callee.span);
        match callee_ty {
            Ty::Fn { params, ret } => {
                let mut bindings = Bindings::default();
                let callee_name = match &callee.kind {
                    ExprKind::Name(n) => Some(n.as_str()),
                    ExprKind::Generic { base, .. } => match &base.kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    _ => None,
                };
                let callee_def = callee_name.and_then(|n| self.global(n));
                let through = match callee_def.as_deref() {
                    Some(Def::Fn { bound_args, .. }) => Some(bound_args.clone()),
                    _ => None,
                };
                let outer = std::mem::replace(&mut self.call_bound_args, through);
                self.args(&callee_text, "takes", &params, args, call, &mut bindings);
                self.call_bound_args = outer;
                if let Some(Def::Fn { default_binds, .. }) = callee_def.as_deref() {
                    for (generic, ty) in default_binds {
                        if !bindings.contains_key(generic) {
                            let ty = ty.subst(&bindings);
                            bindings.insert(generic.clone(), ty);
                        }
                    }
                }
                if let Some(def) = callee_def.as_deref() {
                    self.fn_items_take_their_type(def, &params, args, &mut bindings);
                    if let Def::Fn { bound_args, .. } = def {
                        self.bare_ops_take_monoid(bound_args, &params, args, &mut bindings);
                    }
                }
                if let (
                    Some(fn_name),
                    Some(Def::Fn {
                        bound_args, bounds, ..
                    }),
                ) = (callee_name, callee_def.as_deref())
                {
                    self.shared_bound_args_agree(fn_name, bound_args, &bindings, call.span);
                    let bounds = (bounds.iter())
                        .flat_map(|(p, ts)| ts.iter().map(move |t| (p.clone(), t.clone())))
                        .collect::<Vec<_>>();
                    self.method_bounds_hold(fn_name, (&bounds, &params), &bindings, (args, call));
                }
                if let (
                    ExprKind::Name(_),
                    Some(Def::Fn {
                        row_bounds,
                        trait_bounds,
                        ..
                    }),
                ) = (&callee.kind, callee_def.as_deref())
                {
                    for (param, tname) in trait_bounds {
                        let Some(actual) = bindings.get(param) else {
                            continue;
                        };
                        let Some(Def::Trait { fields, .. }) =
                            self.global(tname).as_deref().cloned()
                        else {
                            continue;
                        };
                        if fields.is_empty() {
                            continue;
                        }
                        if let Ty::Record(actual_fs) = actual {
                            for (req_name, req_ty) in &fields {
                                let found = actual_fs.iter().find(|(n, _)| n == req_name);
                                if !matches!(found, Some((_, got_ty)) if self.fits(got_ty, req_ty))
                                {
                                    self.report(
                                        call.span,
                                        msgs::check::MISSING_IMPL_RECORD_DOES_NOT_SATISFY,
                                        &[&actual, &tname, &req_name, &req_ty],
                                    );
                                }
                            }
                        }
                    }
                    self.rows_hold(call, &callee_text, args, row_bounds.clone(), &bindings);
                }
                if let ExprKind::Field { base, name } = &callee.kind {
                    self.method_rows_hold(call, &callee_text, base, &name.text, args, &bindings);
                }
                self.record_instantiation(call, &bindings);
                self.settle(&ret.fresh_apart(&bindings).subst(&bindings))
            }
            Ty::Named {
                name,
                args: type_args,
            } if name == "Range"
                && matches!(type_args.as_slice(), [Ty::Pos(d)] if d.ends_with(".cuts")) =>
            {
                for arg in args {
                    self.infer(&arg.value);
                }
                type_args[0].clone()
            }
            Ty::Named {
                name,
                args: type_args,
            } => {
                // A `type_of(f)` value calls as `f` (R08.1.10).
                if let Some(f) = name
                    .strip_prefix("type_of(")
                    .and_then(|f| f.strip_suffix(')'))
                {
                    let ty = self.name_ty(f, callee.span);
                    return self.call_kind(call, callee, ty, args);
                }
                self.construct(&name, type_args, &callee_text, args, call)
            }
            Ty::TypeName(name) => self.construct(&name, Vec::new(), &callee_text, args, call),
            // `V(i)`: a number checked into the domain, or `State(v, k)` for a product domain.
            Ty::Domain(name) => {
                if let Some(factors) = self.product_factors(&name) {
                    if args.len() == factors.len() {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_ty = self.infer(&arg.value);
                            let want = arg
                                .name
                                .as_ref()
                                .and_then(|n| {
                                    factors
                                        .iter()
                                        .find(|(fnm, _)| fnm.as_deref() == Some(n.text.as_str()))
                                        .map(|(_, t)| t)
                                })
                                .unwrap_or(&factors[i].1);
                            self.expect_fits(&arg_ty, want, &arg.value, &|c, got, want| {
                                cpl_diag::msgs::check::EXPECTS_BUT.at(&[
                                    &name,
                                    &want,
                                    &c.quote(arg.value.span),
                                    &got.with_article(),
                                ])
                            });
                        }
                        return Ty::Pos(name);
                    }
                    if args.len() == 1 && args[0].name.is_none() {
                        let arg_ty = self.infer(&args[0].value);
                        if let Ty::Tuple(elems) = &arg_ty {
                            if elems.len() == factors.len()
                                && elems
                                    .iter()
                                    .zip(&factors)
                                    .all(|(e, (_, f))| e.is_int() || self.fits(e, f))
                            {
                                return Ty::Pos(name);
                            }
                        }

                        if matches!(arg_ty, Ty::Tuple(_) | Ty::Record(_)) {
                            let want = Ty::Pos(name.clone());
                            self.expect_fits(&arg_ty, &want, &args[0].value, &|c, got, want| {
                                cpl_diag::msgs::check::EXPECTS_BUT.at(&[
                                    &name,
                                    &want,
                                    &c.quote(args[0].value.span),
                                    &got.with_article(),
                                ])
                            });
                        }
                        return Ty::Pos(name);
                    }
                }
                self.plain_args(args);
                Ty::Pos(name)
            }
            Ty::Error => {
                self.plain_args(args);
                Ty::Error
            }
            // `v.l(k)`: a boundary reached from a position.
            Ty::Pos(_) if matches!(&callee.kind, ExprKind::Field { name, .. } if matches!(name.text.as_str(), "l" | "r")) =>
            {
                self.plain_args(args);
                callee_ty
            }
            // `xs.len()`, `xs.is_empty()`: zero-arg getter calls on built-in sequences.
            ty @ (Ty::Int | Ty::Bool)
                if args.is_empty()
                    && matches!(&callee.kind, ExprKind::Field { name, .. } if matches!(name.text.as_str(), "len" | "is_empty")) =>
            {
                ty
            }
            ty if ty.is_numeric() || matches!(ty, Ty::Bool | Ty::Str | Ty::Char | Ty::Pos(_)) => {
                self.plain_args(args);
                let product =
                    (ty.is_numeric() && args.len() == 1).then(|| self.quote(args[0].value.span));
                if let Some(diag) = self.report(
                    callee.span,
                    msgs::check::NOT_CALLABLE_IS_NOT_A_FN,
                    &[&callee_text, &ty.widen().with_article()],
                ) {
                    if let Some(arg) = product {
                        diag.help = Some(format!("to multiply, write `{callee_text} * ({arg})`"));
                    }
                }
                Ty::Error
            }
            // `T(read() for _ in fields(T))` in a fn generic over `T` makes a `T`.
            _ if let ExprKind::Name(n) = &callee.kind
                && self.local(n).is_none()
                && self.generics.iter().any(|g| g == n) =>
            {
                self.plain_args(args);
                Ty::Param(n.clone())
            }
            _ => {
                self.plain_args(args);
                Ty::Unknown
            }
        }
    }
}

impl Checker<'_> {
    /// `arg` against its built-in method's catalog row. A lambda gets the
    /// parameter types the row writes: `x` in `xs.map(x => ...)` is an
    /// element, `(a, b)` in `xs.reach(while: (a, b) => ...)` two. A list
    /// fills cells, so `ys` in `xs.extend(ys)` takes none wider than `xs`'s
    /// range (R04.4.24).
    fn catalog_arg(&mut self, callee: &Expr, args: &[Arg], arg: &Arg) -> Vec<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return Vec::new();
        };
        let Some((member, elem)) = self.catalog_row(base, &name.text) else {
            return Vec::new();
        };
        let params = split_top(member.params);
        let slot = match &arg.name {
            Some(given) => params.iter().find(|p| {
                p.split_once(':')
                    .is_some_and(|(n, _)| n.trim() == given.text)
            }),
            None => args
                .iter()
                .filter(|a| a.name.is_none())
                .position(|a| std::ptr::eq(a, arg))
                .and_then(|i| params.get(i)),
        };
        let Some((_, ty)) = slot.and_then(|p| p.split_once(':')) else {
            return Vec::new();
        };
        // `cmp: ?((T, T) -> Bool) = None` takes a `(T, T) -> Bool`.
        let ty = ty.split_once(" = ").map_or(ty, |(t, _)| t).trim();
        let ty = match ty.strip_prefix("?(") {
            Some(t) => t.strip_suffix(')').unwrap_or(t),
            None => ty,
        };
        if let Some(cell) = (ty.trim().strip_prefix("List<"))
            .and_then(|t| catalog_ty(t.strip_suffix('>')?, &elem, &[]))
        {
            let want = Ty::list(cell);
            let got = self.quietly(|c| c.infer(&arg.value));
            if super::expr::narrows_cells(&got, &want, false) {
                self.expect_fits(&got, &want, &arg.value, &|c, got, want| {
                    cpl_diag::msgs::check::EXPECTS_BUT.at(&[
                        &name.text,
                        &want,
                        &c.quote(arg.value.span),
                        &got.with_article(),
                    ])
                });
            }
        }
        if !matches!(arg.value.kind, ExprKind::Lambda { .. }) {
            return Vec::new();
        }
        let Some((takes, _)) = ty.split_once("->") else {
            return Vec::new();
        };
        let takes = takes.trim();
        let takes = takes
            .strip_prefix('(')
            .and_then(|t| t.strip_suffix(')'))
            .map_or_else(|| vec![takes], split_top);
        takes
            .iter()
            .map(|t| catalog_ty(t, &elem, &[]).unwrap_or(Ty::Unknown))
            .collect()
    }

    /// The catalog row for `base.name` on a built-in type, and the
    /// element type its `T` stands for.
    pub(super) fn catalog_row(
        &mut self,
        base: &Expr,
        name: &str,
    ) -> Option<(&'static crate::builtins::BuiltinMember, Option<Ty>)> {
        let base_ty = self.base_ty(base);
        let owner = catalog_owner(&base_ty)?;
        let elem = match &base_ty {
            Ty::Str => Some(Ty::Char),
            Ty::Named { args, .. } => {
                (base_ty.seq_elem()).or_else(|| (args.len() == 1).then(|| args[0].clone()))
            }
            _ => base_ty.seq_elem(),
        };
        Some((crate::builtins::builtin_member(owner, name)?, elem))
    }

    /// What a built-in method returns when nothing more specific knows:
    /// its catalog row's type, e.g. the element for `a.sum()` on a `[V]Int`.
    /// `?T` for `xs.pop()`, so printing or adding it says `None` must be handled.
    fn catalog_ret(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let (member, elem) = self.catalog_row(base, &name.text)?;
        // `m.get(k, 0)` is not the row `get(k)`: its arity error says so.
        let arity = split_top(member.params)
            .into_iter()
            .filter(|p| !p.trim().is_empty());
        if args.len() > arity.count() {
            return None;
        }
        // `xs.find(x => x > 1)` is the element, not the row's index.
        if args
            .iter()
            .any(|a| matches!(a.value.kind, ExprKind::Lambda { .. }))
        {
            return None;
        }
        // `m.get(k)` on a `Map<K, V>` is a `?V`.
        let key_value = match self.base_ty(base) {
            Ty::Named { name, args } if matches!(name.as_str(), "Map" | "HashMap") => args,
            _ => Vec::new(),
        };
        let ret = match (name.text.as_str(), elem) {
            ("min" | "max", Some(elem)) if member.ret == "?T" => Ty::min_max_of(elem),
            (_, elem) => catalog_ty(member.ret, &elem, &key_value)?,
        };
        // A sum of `1..=1e9`s leaves that range, and may not hold its `0`.
        if name.text == "sum" && matches!(ret, Ty::Range { .. }) {
            return Some(ret.widen());
        }
        Some(ret)
    }

    /// `(i, j) => ...` passed where `(I, I) -> Int` is wanted: its unwritten
    /// parameters are an `I` each, for hover and inlay hints.
    fn type_lambda_params(
        &mut self,
        params: &[Pattern],
        written: &[Option<Type>],
        wanted: &[Param],
    ) {
        for (i, (param, want)) in params.iter().zip(wanted).enumerate() {
            let ty = &want.ty;
            if written.get(i).is_some_and(Option::is_some)
                || !matches!(param.kind, PatternKind::Name(_))
                || ty.has_vague()
            {
                continue;
            }
            for entry in &mut self.hovers {
                if entry.decl == Some(param.span) && matches!(entry.ty, Ty::Unknown) {
                    entry.ty = ty.clone();
                }
            }
        }
    }
}

/// A catalog type string the checker can read without the row's generics.
/// A catalog row's type `text`, with `T` the element and, for a map's rows,
/// `K` and `V` its `key_value` arguments.
fn catalog_ty(text: &str, elem: &Option<Ty>, key_value: &[Ty]) -> Option<Ty> {
    if let Some(inner) = text.trim().strip_prefix('?') {
        return Some(Ty::Optional(Box::new(catalog_ty(inner, elem, key_value)?)));
    }
    match (text.trim(), key_value) {
        ("K", [k, _]) => Some(k.clone()),
        ("V", [_, v]) => Some(v.clone()),
        _ => None,
    }
    .or_else(|| match text.trim() {
        "T" => elem.clone(),
        "Char" => Some(Ty::Char),
        "Int" => Some(Ty::Int),
        "Bool" => Some(Ty::Bool),
        "Str" => Some(Ty::Str),
        "" => Some(Ty::Unit),
        _ => None,
    })
}

pub(super) use crate::builtins::split_params as split_top;

impl Checker<'_> {
    /// `xs.count(v)` and `xs.first(p)`/`xs.last(p)`, which std writes
    /// `xs.count(x => x == v)` and `xs.find(p)`/`xs.rfind(p)`: the error
    /// carries the rewrite, so `cpl fix` migrates the call.
    fn removed_seq_spelling(&mut self, name: &cpl_syntax::ast::Name, args: &[Arg], elem: &Ty) {
        let [arg] = args else { return };
        if arg.name.is_some() {
            return;
        }
        let got = match arg.value.kind {
            ExprKind::Lambda { .. } => None,
            _ => Some(self.quietly(|c| c.infer(&arg.value))),
        };
        let is_fn = got.as_ref().is_none_or(|t| matches!(t, Ty::Fn { .. }));
        let (old, new, span, with) = match name.text.as_str() {
            // `s.count("a")` on a `Str` is a type error, not the old spelling.
            "count" if got.as_ref().is_some_and(|t| !is_fn && t.fits(elem)) => {
                let v = self.text(arg.value.span);
                let x = super::removed::fresh(v, "x", 0);
                let with = format!("{x} => {x} == {v}");
                let new = format!("count({with})");
                (format!("count({v})"), new, arg.value.span, with)
            }
            "first" | "last" if is_fn => {
                let to = if name.text == "first" {
                    "find"
                } else {
                    "rfind"
                };
                (name.text.clone(), to.to_string(), name.span, to.to_string())
            }
            _ => return,
        };
        if let Some(diag) = self.report(
            span,
            msgs::check::REMOVED_SYNTAX_IS_SPELLED_SINCE_0,
            &[&old, &new],
        ) {
            diag.fix(habits::WRITE.say(&[&new]), span, with);
        }
    }
}

/// Whether `e` names storage the caller keeps; a temporary may be passed to
/// a `mut` parameter without `mut`, since nothing the caller holds changes.
pub(super) fn is_place(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Name(_)
        | ExprKind::SelfValue
        | ExprKind::Field { .. }
        | ExprKind::Index { .. } => true,
        ExprKind::Paren(inner) => is_place(inner),
        _ => false,
    }
}

impl Checker<'_> {
    /// `add(c, k)` inside a method of a struct with an `add` method is
    /// `self.add(c, k)`, as its fields are named without `self.`.
    fn implicit_self_method(&mut self, callee: &Expr) -> Option<Expr> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        if self.local(name).is_some() || self.local("self").is_none() {
            return None;
        }
        let Some(Ty::Named { name: owner, .. }) = self.self_ty.clone() else {
            return None;
        };
        let has = match self.global(&owner).as_deref() {
            Some(Def::Struct {
                methods, getters, ..
            }) => methods.iter().any(|(m, _)| m == name) && !getters.iter().any(|g| g == name),
            _ => false,
        };
        has.then(|| Expr {
            kind: ExprKind::Field {
                base: Box::new(Expr {
                    kind: ExprKind::SelfValue,
                    span: callee.span,
                }),
                name: cpl_syntax::ast::Name {
                    text: name.clone(),
                    span: callee.span,
                },
            },
            span: callee.span,
        })
    }
}

/// The catalog's name for the built-in type whose members `ty` has.
pub(super) fn catalog_owner(ty: &Ty) -> Option<&str> {
    Some(match ty {
        Ty::Str => "Str",
        Ty::Array { .. } => "[D]T",
        Ty::Named { name, .. } => match name.as_str() {
            "HashSet" => "Set",
            "HashMap" => "Map",
            other => other,
        },
        _ => return None,
    })
}
