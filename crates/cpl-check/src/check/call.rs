//! Calls: fns, constructors, and conversions.

use super::expr::{list_names, plural};
use super::*;
use crate::ty::{Bindings, Param};
use cpl_syntax::ast::{Arg, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind, Type};

mod construct;

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
        let ty = self.quietly(|c| c.infer(e));
        self.io_fixed(&ty)
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
            let ty = self.quietly(|c| c.infer(&arg.value));
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

    pub(crate) fn call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Ty {
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
                        self.error(
                            codes::USED_BEFORE_SET,
                            callee.span,
                            format!("`{fn_name}` captures `{cap}`, which is not initialized yet"),
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
                let over = self.quietly(|c| c.infer(&arg.value));
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
        // `gen<Rooted<V>>(shape: path)`: the shape is a name of `gen`'s own.
        if let ExprKind::Generic { base, .. } = &callee.kind {
            if matches!(&base.kind, ExprKind::Name(n) if n == "gen" && self.local(n).is_none() && self.global(n).is_none())
            {
                for arg in args {
                    let shape = arg.name.as_ref().is_some_and(|n| n.text == "shape");
                    if !(shape && matches!(arg.value.kind, ExprKind::Name(_))) {
                        self.infer(&arg.value);
                    }
                }
                return self.read_ty(callee, args).unwrap_or(Ty::Unknown);
            }
        }
        if let Some(on_self) = self.implicit_self_method(callee) {
            return self.call(call, &on_self, args);
        }
        if let Some(ty) = self.builtin_getter_call(call, callee, args) {
            return ty;
        }
        if let Some(ty) = self.getter_call(callee, args) {
            return ty;
        }
        if let Some(ty) = self.filled(callee, args) {
            return ty;
        }
        if let Some(ty) = self.grid_shape_call(callee, args) {
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
            if matches!(arg.value.kind, ExprKind::Lambda { .. }) {
                let params = self.catalog_lambda_params(callee, args, arg);
                if !params.is_empty() {
                    self.lambda_params.push((arg.value.span, params));
                }
            }
        }
        let past_row = self.past_builtin_row(callee, args);
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
            if let Some(read) = self.read_ty(callee, args) {
                ty = read;
            }
        }
        let converts_one = match &callee.kind {
            ExprKind::Name(n) if n == "print" => {
                self.record_io(callee.span, n, args);
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

    /// `read<T>()` (optionally with `base:` / `none:` / `gen:`) is a `T`.
    fn read_ty(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Generic { base, args: tys } = &callee.kind else {
            return None;
        };
        let (ExprKind::Name(name), [t]) = (&base.kind, tys.as_slice()) else {
            return None;
        };
        if name == "gen" && self.local(name).is_none() && self.global(name).is_none() {
            let ty = self.quietly(|c| c.type_arg(t));
            return (!matches!(ty, Ty::Unknown | Ty::Error)).then_some(ty);
        }
        if !args.iter().all(|a| {
            a.name
                .as_ref()
                .is_some_and(|n| matches!(n.text.as_str(), "base" | "none" | "gen"))
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
        let prelude = matches!(&base.kind, ExprKind::Name(list)
            if list == "List" && self.local(list).is_none() && self.global(list).is_none());
        let [count, value] = args else {
            return None;
        };
        if !prelude || name.text != "filled" {
            return None;
        }
        self.infer(&count.value);
        let elem = self.infer(&value.value).widen();
        Some(Ty::Named {
            name: "List".into(),
            args: vec![elem],
        })
    }

    /// `s.range(r)` on `SortedBag<K>`, `SortedSet<K>`, `Set<K>`, or `Map<K, V>`:
    /// returns an O(1) `SortedRange<E>` view over entries matching `r`
    /// (`a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..`, or a `Range<K>` value).
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
        let (k_ty, elem_ty) = match (cname.as_str(), targs.as_slice()) {
            ("SortedBag" | "SortedSet" | "Set", [k]) => (k.clone(), k.clone()),
            ("Map", [k, v]) => (k.clone(), Ty::Tuple(vec![k.clone(), v.clone()])),
            _ => return None,
        };
        self.note_projection(call, cname, "range");
        if args.len() != 1 || args[0].name.is_some() {
            for a in args {
                self.infer(&a.value);
            }
            self.error(
                codes::WRONG_ARG_COUNT,
                call.span,
                format!(
                    "`{cname}.range` takes 1 range argument, found {}",
                    args.len()
                ),
            );
            return Some(Ty::Named {
                name: "SortedRange".into(),
                args: vec![elem_ty],
            });
        }
        let arg = &args[0].value;
        match &arg.kind {
            ExprKind::Range { lo, hi, step, .. } => {
                if let Some(st) = step {
                    self.infer(st);
                    self.error(
                        codes::TYPE_MISMATCH,
                        st.span,
                        format!("`{cname}.range` does not take a stepped range"),
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
                        this.expect_fits(&ety, &k_ty, e, |c, got, want| {
                            format!(
                                "range bound `{}` is {}, expected {}",
                                c.quote(e.span),
                                got.with_article(),
                                want.with_article()
                            )
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
                    self.expect_fits(&got, &want, arg, |c, g, w| {
                        format!(
                            "`{}` is {}, expected {}",
                            c.quote(arg.span),
                            g.with_article(),
                            w.with_article()
                        )
                    });
                }
            }
        }
        Some(Ty::Named {
            name: "SortedRange".into(),
            args: vec![elem_ty],
        })
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
        if !matches!(cname.as_str(), "SortedBag" | "SortedSet") || targs.len() != 1 {
            return None;
        }
        let k_ty = targs[0].clone();
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
        let msg = format!("`{cname}.{}` was replaced by `{replacement}`", name.text);
        if let Some(diag) = self.error(codes::UNKNOWN_FIELD, call.span, msg) {
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
        let base_ty = self.quietly(|c| c.infer(base));
        if matches!(name.text.as_str(), "set_at" | "swap_idx") && base_ty.is_indexed_mut() {
            self.require_mutable(base, &format!("changed by `{}`", name.text));
            return;
        }
        let members = match &base_ty {
            Ty::Named { name, .. } => self.members_of(name),
            _ => None,
        };
        let mutates = match &members {
            Some(m) if m.members.iter().any(|(n, _)| n == &name.text) => {
                m.mutators.contains(&name.text)
            }
            _ => self.takes_mut_self(&base_ty, &name.text),
        };
        if mutates {
            self.require_mutable(base, &format!("changed by `{}`", name.text));
        }
    }

    /// Built-in `MemberKind::Get` properties (`len`, `is_empty`, `first`,
    /// `last`, `rev`, `items`) are read without parentheses.
    fn builtin_getter_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        if !args.is_empty() {
            return None;
        }
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let base_ty = self.quietly(|c| c.infer(base));
        let is_builtin_get = match &base_ty {
            Ty::Named { name: n, args } if n == "List" && args.len() == 1 => {
                crate::builtins::builtin_members(crate::builtins::Receiver::List)
                    .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name.text)
            }
            Ty::Str => crate::builtins::builtin_members(crate::builtins::Receiver::Str)
                .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name.text),
            Ty::Named { name: n, args }
                if matches!(n.as_str(), "Map" | "HashMap") && args.len() == 2 =>
            {
                crate::builtins::builtin_members(crate::builtins::Receiver::Map)
                    .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name.text)
            }
            Ty::Named { name: n, args }
                if matches!(n.as_str(), "Set" | "HashSet") && args.len() == 1 =>
            {
                crate::builtins::builtin_members(crate::builtins::Receiver::Set)
                    .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name.text)
            }
            Ty::Array { .. } => crate::builtins::builtin_members(crate::builtins::Receiver::Array)
                .any(|m| m.kind == crate::builtins::MemberKind::Get && m.name == name.text),
            Ty::Named { name: n, .. } if n == "Range" => {
                matches!(
                    name.text.as_str(),
                    "len" | "is_empty" | "first" | "last" | "rev" | "start" | "end"
                )
            }
            Ty::Named { name: n, .. } if n == "SortedRange" => {
                matches!(
                    name.text.as_str(),
                    "len" | "is_empty" | "first" | "last" | "rev"
                )
            }
            Ty::Domain(_) => {
                matches!(
                    name.text.as_str(),
                    "len" | "first" | "last" | "rev" | "bounds" | "cuts"
                )
            }
            t if t.is_int() => matches!(name.text.as_str(), "abs" | "ord" | "sqrt"),
            Ty::Float => matches!(
                name.text.as_str(),
                "abs" | "ceil" | "floor" | "round" | "sqrt" | "trunc"
            ),
            _ => false,
        };
        if !is_builtin_get {
            return None;
        }
        let callee_text = self.quote(callee.span);
        let message = format!(
            "`{}` is read without parentheses: `{callee_text}`",
            name.text
        );
        if let Some(diag) = self.error(codes::NOT_CALLABLE, call.span, message) {
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
        } = self.quietly(|c| c.infer(base))
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
        let given = self.quietly(|c| c.infer(&only.value));
        let array = matches!(given, Ty::Array { .. });
        let suffix = match given.clone() {
            Ty::Named { name: list, args } if list == "List" => match args.as_slice() {
                [Ty::Tuple(fields)] => fields.len(),
                [Ty::Record(fields)] => fields.len(),
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
        let message = format!("`{name}<T>()` takes no arguments");
        self.error(codes::WRONG_ARG_COUNT, call.span, message);
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
        let own = match self.type_def(owner).as_deref() {
            Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) => {
                methods.iter().any(|(m, _)| *m == name.text)
            }
            _ => false,
        };
        if own {
            return false;
        }
        let mut traits = Vec::new();
        for (trait_name, def) in self.env.impls(owner) {
            match (trait_name, &*def) {
                (None, Def::Impl { methods, .. })
                    if methods.iter().any(|(m, _)| *m == name.text) =>
                {
                    return false;
                }
                (Some(t), Def::Impl { methods, .. })
                    if methods.iter().any(|(m, _)| *m == name.text) =>
                {
                    traits.push(t)
                }
                _ => {}
            }
        }
        let [first, second, ..] = traits.as_slice() else {
            return false;
        };
        let shown = self.quote(base.span);
        let method = &name.text;
        let message = format!(
            "`{shown}.{method}` is ambiguous: both `{first}` and `{second}` give `{owner}` a `{method}`"
        );
        if let Some(diag) = self.error(codes::AMBIGUOUS_METHOD, name.span, message) {
            diag.help = Some(format!("name the trait: `{first}.{method}({shown})`"));
        }
        true
    }

    fn receiver(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let base_ty = self.quietly(|c| c.infer(base));
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
                self.expect_fits(&got, &Ty::Int, &arg.value, |c, got, _| {
                    format!(
                        "`{}` takes an `Int`, but `{}` is {}",
                        c.quote(callee.span),
                        c.quote(arg.value.span),
                        got.with_article()
                    )
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
                        } if !params.is_empty() => self.quietly(|c| {
                            c.push_scope();
                            let pty = param_tys
                                .first()
                                .filter(|_| params.len() == 1)
                                .and_then(|t| t.as_ref())
                                .map(|t| c.resolve_type(t))
                                .unwrap_or_else(|| (**elem).clone());
                            c.bind(
                                &Pattern::single_param(params).expect("a parameter"),
                                &pty,
                                false,
                                "param",
                            );
                            let ret = c.infer(body).widen();
                            c.pop_scope();
                            ret
                        }),
                        _ => match self.quietly(|c| c.infer(&arg.value)) {
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
        let seq_elem: Option<Ty> = base_ty.seq_elem().or_else(|| match &base_ty {
            Ty::Named { name: n, args }
                if matches!(n.as_str(), "Bitset" | "Set" | "HashSet") && args.len() == 1 =>
            {
                Some(args[0].clone())
            }
            _ => None,
        });
        if let Some(elem) = seq_elem {
            match name.text.as_str() {
                "map" => {
                    let out = if let [arg] = args {
                        match &arg.value.kind {
                            ExprKind::Lambda {
                                params,
                                param_tys,
                                body,
                                ..
                            } if params.len() == 1
                                || elem.param_fields().is_some_and(|f| f.len() == params.len()) =>
                            {
                                self.quietly(|c| {
                                    c.push_scope();
                                    let pty = param_tys
                                        .first()
                                        .filter(|_| params.len() == 1)
                                        .and_then(|t| t.as_ref())
                                        .map(|t| c.resolve_type(t))
                                        .unwrap_or_else(|| elem.clone());
                                    c.bind(
                                        &Pattern::single_param(params).expect("a parameter"),
                                        &pty,
                                        false,
                                        "param",
                                    );
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
                                || matches!(self.quietly(|c| c.infer(&arg.value)), Ty::Fn { .. }))
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
                "argmax" | "argmin" if !base_ty.is_keyed() => {
                    let message = format!(
                        "`{}` requires `Keyed`, but `{}` has type `{base_ty}`",
                        name.text,
                        self.quote(base.span)
                    );
                    self.error(codes::MISSING_IMPL, name.span, message);
                    return Some(Ty::Error);
                }
                "sort" | "reverse" | "shuffle" if !matches!(base_ty, Ty::Domain(_)) => {
                    // A read-only view (`xs.rev`) is reported once, as not writable.
                    if !base_ty.is_indexed_mut() && !self.is_readonly_view(base) {
                        let message = format!(
                            "`{}` requires `IndexedMut`, but `{}` has type `{base_ty}`",
                            name.text,
                            self.quote(base.span)
                        );
                        self.error(codes::MISSING_IMPL, name.span, message);
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
                if args.len() == 1
                    && args[0].name.is_none()
                    && matches!(&args[0].value.kind, ExprKind::Lambda { params, .. } if params.len() == 2)
                    && self.text(args[0].value.span).contains("=>")
                {
                    let elem = self.element_ty(&base_ty);
                    if !matches!(elem, Ty::Unknown | Ty::Error) {
                        self.infer(&args[0].value);
                        return Some(Ty::Optional(Box::new(elem)));
                    }
                }
            }
        }
        match base_ty {
            Ty::Optional(ref inner)
                if super::method_form::OPTIONAL_MEMBERS.contains(&name.text.as_str()) =>
            {
                if let ExprKind::Call { args, .. } = &call.kind {
                    if args.len() == 1 && args[0].name.is_none() {
                        let f = &args[0].value;
                        let fn_ret = match &f.kind {
                            ExprKind::Lambda {
                                params,
                                param_tys,
                                body,
                                ..
                            } if !params.is_empty() => {
                                self.push_scope();
                                let pty = param_tys
                                    .first()
                                    .filter(|_| params.len() == 1)
                                    .and_then(|o| o.as_ref())
                                    .map(|t| self.resolve_type(t))
                                    .unwrap_or_else(|| (**inner).clone());
                                self.bind(
                                    &Pattern::single_param(params).expect("a parameter"),
                                    &pty,
                                    false,
                                    "param",
                                );
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
                    let message = format!(
                        "`inverse()` requires elements of an index domain or bounded range, not `{elem}`"
                    );
                    self.error(codes::TYPE_MISMATCH, name.span, message);
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
            // `adj.out(u)` is `adj[u]` for a table of neighbour lists only.
            Ty::Array { index, elem }
                if name.text == "out"
                    && !elem.is_vague()
                    && !matches!(&*elem, Ty::Named { name, .. } if name == "List") =>
            {
                let shown = Ty::Array { index, elem }.to_string();
                let message =
                    format!("`.out(v)` reads a table of neighbour lists, not a `{shown}`");
                if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                    diag.help = Some("index it: `a[v]`".to_string());
                }
                Some(Ty::Error)
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

    fn call_kind(&mut self, call: &Expr, callee: &Expr, callee_ty: Ty, args: &[Arg]) -> Ty {
        let callee_text = self.quote(callee.span);
        match callee_ty {
            Ty::Fn { params, ret } => {
                let mut bindings = Bindings::default();
                self.args(&callee_text, "takes", &params, args, call, &mut bindings);
                let callee_name = match &callee.kind {
                    ExprKind::Name(n) => Some(n.as_str()),
                    ExprKind::Generic { base, .. } => match &base.kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(fn_name) = callee_name {
                    if let Some(Def::Fn { bounds, .. }) = self.global(fn_name).as_deref().cloned() {
                        for (param, traits) in bounds {
                            if let Some(bound_ty) = bindings.get(&param) {
                                let bound_ty = self.settle(bound_ty);
                                for trait_name in traits {
                                    if !self.implements_trait(&bound_ty, &trait_name) {
                                        let message = format!(
                                            "`{bound_ty}` does not implement `{trait_name}` required by `{fn_name}`"
                                        );
                                        let help = self.unmet_bound_help(&bound_ty);
                                        if let Some(diag) =
                                            self.error(codes::MISSING_IMPL, call.span, message)
                                        {
                                            diag.help = help;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let ExprKind::Name(fname) = &callee.kind {
                    if let Some(Def::Fn {
                        row_bounds,
                        trait_bounds,
                        ..
                    }) = self.global(fname).as_deref().cloned()
                    {
                        for (param, tname) in trait_bounds {
                            let Some(actual) = bindings.get(&param) else {
                                continue;
                            };
                            let Some(Def::Trait { fields, .. }) =
                                self.global(&tname).as_deref().cloned()
                            else {
                                continue;
                            };
                            if fields.is_empty() {
                                continue;
                            }
                            match actual {
                                Ty::Record(actual_fs) => {
                                    for (req_name, req_ty) in &fields {
                                        let found = actual_fs.iter().find(|(n, _)| n == req_name);
                                        if !matches!(found, Some((_, got_ty)) if self.fits(got_ty, req_ty))
                                        {
                                            self.error(
                                                codes::MISSING_IMPL,
                                                call.span,
                                                format!(
                                                    "record `{actual}` does not satisfy field trait `{tname}` (missing `{req_name}: {req_ty}`)"
                                                ),
                                            );
                                        }
                                    }
                                }
                                Ty::Named { name: sname, .. } => {
                                    let has_impl = self.env.impls(sname).iter().any(|(tn, _)| {
                                        tn.as_deref().and_then(|t| t.split('<').next())
                                            == Some(tname.as_str())
                                    });
                                    if !has_impl {
                                        self.error(
                                            codes::MISSING_IMPL,
                                            call.span,
                                            format!(
                                                "`{sname}` does not implement `{tname}`; structs must opt in with `impl {tname} for {sname}`"
                                            ),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
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
                                                let diag = self.error(
                                                    codes::ROW_MISSING_FIELD,
                                                    call.span,
                                                    format!(
                                                        "`{callee_text}` needs a record with `{req_name}: {req_ty}`, but `{actual}` {}",
                                                        found.map_or(format!("has no `{req_name}`"), |(_, t)| format!("has `{req_name}: {t}`"))
                                                    ),
                                                );
                                                suggest_rename(diag, args, actual_fs, &req_fields);
                                            }
                                        }
                                    }
                                }
                                Ty::Error | Ty::Unknown | Ty::Param(_) => {}
                                other => {
                                    self.error(
                                        codes::ROW_MISSING_FIELD,
                                        call.span,
                                        format!(
                                            "`{callee_text}` needs a record for `{param}`, but this is `{other}`"
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
                self.settle(&ret.subst(&bindings))
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
            } => self.construct(&name, type_args, &callee_text, args, call),
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
                            self.expect_fits(&arg_ty, want, &arg.value, |c, got, want| {
                                format!(
                                    "`{name}` expects {want}, but `{}` is {}",
                                    c.quote(arg.value.span),
                                    got.with_article()
                                )
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
                            self.expect_fits(&arg_ty, &want, &args[0].value, |c, got, want| {
                                format!(
                                    "`{name}` expects {want}, but `{}` is {}",
                                    c.quote(args[0].value.span),
                                    got.with_article()
                                )
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
                let message = format!("`{callee_text}` is {}, not a fn", ty.widen().with_article());
                let product =
                    (ty.is_numeric() && args.len() == 1).then(|| self.quote(args[0].value.span));
                if let Some(diag) = self.error(codes::NOT_CALLABLE, callee.span, message) {
                    if let Some(arg) = product {
                        diag.help = Some(format!("to multiply, write `{callee_text} * ({arg})`"));
                    }
                }
                Ty::Error
            }
            _ => {
                self.plain_args(args);
                Ty::Unknown
            }
        }
    }
}

impl Checker<'_> {
    /// The parameter types of `arg`, a lambda passed to a built-in method, as
    /// its catalog row writes them: `x` in `xs.map(x => ...)` is an element,
    /// `(a, b)` in `xs.reach(while: (a, b) => ...)` two.
    fn catalog_lambda_params(&mut self, callee: &Expr, args: &[Arg], arg: &Arg) -> Vec<Ty> {
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
            .map(|t| catalog_ty(t, &elem).unwrap_or(Ty::Unknown))
            .collect()
    }

    /// The catalog row for `base.name` on a built-in type, and the
    /// element type its `T` stands for.
    pub(super) fn catalog_row(
        &mut self,
        base: &Expr,
        name: &str,
    ) -> Option<(&'static crate::builtins::BuiltinMember, Option<Ty>)> {
        let base_ty = self.quietly(|c| c.infer(base));
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
        // `xs.find(x => x > 1)` is the element, not the row's index.
        if args
            .iter()
            .any(|a| matches!(a.value.kind, ExprKind::Lambda { .. }))
        {
            return None;
        }
        let ret = match (name.text.as_str(), elem) {
            ("min" | "max", Some(elem)) if member.ret == "?T" => Ty::min_max_of(elem),
            (_, elem) => catalog_ty(member.ret, &elem)?,
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
fn catalog_ty(text: &str, elem: &Option<Ty>) -> Option<Ty> {
    if let Some(inner) = text.trim().strip_prefix('?') {
        return Some(Ty::Optional(Box::new(catalog_ty(inner, elem)?)));
    }
    match text.trim() {
        "T" => elem.clone(),
        "Char" => Some(Ty::Char),
        "Int" => Some(Ty::Int),
        "Bool" => Some(Ty::Bool),
        "Str" => Some(Ty::Str),
        _ => None,
    }
}

/// `a: Int, f: (T, T) -> Bool` split at its top-level commas.
fn split_top(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' if !text[..i].ends_with('-') => depth -= 1,
            ',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !text[start..].trim().is_empty() {
        out.push(text[start..].trim());
    }
    out
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
