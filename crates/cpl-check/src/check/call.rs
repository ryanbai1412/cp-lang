//! Calls: fns, constructors, and conversions.

use super::expr::{list_names, plural};
use super::*;
use crate::ty::{Bindings, Param};
use cpl_syntax::ast::{Arg, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind, Type};

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
        let given = self.lambda_params.len();
        for arg in args {
            if matches!(arg.value.kind, ExprKind::Lambda { .. }) {
                let params = self.catalog_lambda_params(callee, args, arg);
                if !params.is_empty() {
                    self.lambda_params.push((arg.value.span, params));
                }
            }
        }
        let mut ty = if receiver.is_some()
            && !matches!(
                callee_ty,
                Ty::Fn { .. } | Ty::Unknown | Ty::Error | Ty::TypeName(_)
            ) {
            for arg in args {
                self.infer(&arg.value);
            }
            Ty::Unknown
        } else {
            self.call_kind(call, callee, callee_ty.clone(), args)
        };
        if receiver.is_none() && matches!(ty, Ty::Unknown) {
            if let Some(known) = self.catalog_ret(callee) {
                ty = known;
            }
        }
        if let Some(known) = self.prelude_call(call, callee, args) {
            ty = known;
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
        self.mutates_view(call, callee);
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
        if !args.iter().all(|a| {
            a.name
                .as_ref()
                .is_some_and(|n| matches!(n.text.as_str(), "base" | "none" | "gen"))
        }) {
            return None;
        }
        if name != "read" || self.local(name).is_some() || self.global(name).is_some() {
            return None;
        }
        let ty = self.quietly(|c| c.type_arg(t));
        (!matches!(ty, Ty::Unknown | Ty::Error)).then_some(ty)
    }

    /// `List.filled(n, x)` is a list of what `x` is. A whole number
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
                .map(|e| {
                    let kind = match &e.kind {
                        ExprKind::Name(n) => cpl_syntax::ast::TypeKind::Named {
                            name: cpl_syntax::ast::Name {
                                text: n.clone(),
                                span: e.span,
                            },
                            args: Vec::new(),
                        },
                        _ => cpl_syntax::ast::TypeKind::Value(Box::new(e.clone())),
                    };
                    self.resolve_type(&Type { kind, span: e.span })
                })
                .collect();
            let elem = self.infer(&value.value);
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
        let elem = self.infer(&value.value);
        Some(Ty::Named {
            name: "List".into(),
            args: vec![elem],
        })
    }

    /// `s.range(r)` on `SortedBag<K>`, `SortedSet<K>`, `Set<K>`, or `Map<K, V>`:
    /// returns an O(1) `SortedRange<E>` view over entries matching `r`
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
        let (k_ty, elem_ty) = match (cname.as_str(), targs.as_slice()) {
            ("SortedBag" | "SortedSet" | "Set", [k]) => (k.clone(), k.clone()),
            ("Map", [k, v]) => (k.clone(), Ty::Tuple(vec![k.clone(), v.clone()])),
            _ => return None,
        };
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
            diag.fixes.push(cpl_diag::Fix {
                title: format!("replace `{}` with `{replacement}`", name.text),
                edits: vec![(replace_span, replacement)],
            });
        }
        Some(ret_ty)
    }

    /// `c.bump()` where `bump` takes `mut self` changes `c`, so `c` is a
    /// `var`, a `mut` parameter, or the `mut self` of the calling method.
    /// Without this the call works on a copy and the change is lost.
    fn mutates_self(&mut self, callee: &Expr) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        let base_ty = self.quietly(|c| c.infer(base));
        if matches!(name.text.as_str(), "set_at" | "swap_idx") && base_ty.is_indexed_mut() {
            self.require_mutable(base, &format!("changed by `{}`", name.text));
            return;
        }
        let Ty::Named {
            name: type_name, ..
        } = base_ty
        else {
            return;
        };
        if self
            .members_of(&type_name)
            .is_some_and(|m| m.mutators.contains(&name.text))
        {
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
            diag.fixes.push(cpl_diag::Fix {
                title: format!("drop `()` to read `{callee_text}`"),
                edits: vec![(parens, String::new())],
            });
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
        let mut bindings = Bindings::new();
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

    /// What the checker knows about a prelude fn it otherwise takes on
    /// trust: `span(a, b)` covers two ends of one domain, and the `max` or
    /// `min` of a slice is optional, since `a[l..l]` is empty.
    fn prelude_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        if self.local(name).is_some()
            || (self.global(name).is_some() && !(name == "zip" && crate::edition::is_v03()))
        {
            return None;
        }
        let tys: Vec<Ty> = args
            .iter()
            .map(|arg| self.quietly(|c| c.infer(&arg.value)))
            .collect();
        match (name.as_str(), tys.as_slice()) {
            ("List", []) => Some(Ty::Named {
                name: "List".into(),
                args: vec![],
            }),
            ("List", [only]) => {
                let elem = match only {
                    Ty::Named { name, args }
                        if matches!(
                            name.as_str(),
                            "Gen"
                                | "Iter"
                                | "Filter"
                                | "Zip"
                                | "Range"
                                | "Slice"
                                | "Rev"
                                | "Map"
                                | "Rebased"
                        ) && args.len() == 1 =>
                    {
                        args[0].clone()
                    }
                    Ty::Named {
                        name,
                        args: list_args,
                    } if name == "List"
                        && list_args.len() == 1
                        && self.view_kind(&args[0].value) == Some("slice") =>
                    {
                        list_args[0].clone()
                    }
                    Ty::Array { elem, .. } => (**elem).clone(),
                    other => other.clone(),
                };
                Some(Ty::Named {
                    name: "List".into(),
                    args: vec![elem],
                })
            }
            ("List", [first, ..]) => Some(Ty::Named {
                name: "List".into(),
                args: vec![first.clone()],
            }),
            ("read_line", _) => Some(Ty::Str),
            ("print", _) if crate::edition::is_v03() => {
                let allow_none = args
                    .iter()
                    .any(|a| a.name.as_ref().is_some_and(|n| n.text == "none"));
                for (arg, ty) in args.iter().zip(&tys) {
                    if arg.name.is_none() && !self.implements_print(ty, allow_none) {
                        let snippet = self.quote(arg.value.span);
                        let message = if matches!(ty, Ty::Bool) {
                            format!(
                                "`{snippet}` has type `Bool`, which does not implement `Print`; use `print(yes({snippet}))`"
                            )
                        } else {
                            format!("`{snippet}` has type `{ty}`, which does not implement `Print`")
                        };
                        self.error(codes::MISSING_IMPL, arg.value.span, message);
                    }
                }
                None
            }
            ("zip", _) => {
                if tys.len() >= 2 {
                    let mut first_dom: Option<&str> = None;
                    for (arg, ty) in args.iter().zip(&tys) {
                        if let Ty::Array { index, .. } = ty {
                            if let [Ty::Pos(d)] = index.as_slice() {
                                if let Some(prev) = first_dom {
                                    if prev != d {
                                        let message = format!(
                                            "`zip` expects matching domains, but `{}` is indexed by `{d}` instead of `{prev}`",
                                            self.quote(arg.value.span)
                                        );
                                        self.error(codes::WRONG_DOMAIN, arg.value.span, message);
                                    }
                                } else {
                                    first_dom = Some(d.as_str());
                                }
                            }
                        }
                    }
                    let elems: Vec<Ty> = tys.iter().map(|t| self.element_ty(t)).collect();
                    return Some(Ty::Named {
                        name: "Zip".into(),
                        args: vec![Ty::Tuple(elems)],
                    });
                }
                None
            }
            ("abs", [Ty::Int | Ty::Range { .. }]) => Some(Ty::Int),
            ("abs", [t @ (Ty::I128 | Ty::Float)]) => Some(t.clone()),
            // `none` is neither skipped nor passed on by `sum` (v2.2).
            ("sum", [over]) => {
                let elem = self.element_ty(over);
                if matches!(elem, Ty::Optional(_)) {
                    let message = format!("`sum` adds numbers, not {}", elem.with_article());
                    if let Some(diag) = self.error(codes::BAD_OPERANDS, args[0].value.span, message)
                    {
                        diag.help = Some(
                            "`sum` does not skip or absorb `none`: add `x ?? 0`, or filter with `if let`".into(),
                        );
                    }
                }
                None
            }
            ("span", [Ty::Pos(a), Ty::Pos(b)]) => {
                let (a, b) = (
                    a.strip_suffix(".cuts").unwrap_or(a),
                    b.strip_suffix(".cuts").unwrap_or(b),
                );
                if a != b {
                    let message = format!(
                        "span of {} and {}",
                        Ty::Pos(a.into()).with_article(),
                        Ty::Pos(b.into()).with_article()
                    );
                    self.error(codes::WRONG_DOMAIN, call.span, message);
                    return Some(Ty::Error);
                }
                Some(Ty::Named {
                    name: "Range".into(),
                    args: vec![Ty::Pos(a.into())],
                })
            }
            ("max" | "min", [Ty::Array { elem, .. }])
                if !crate::edition::is_v03() && is_slice(&args[0].value) =>
            {
                Some(Ty::Optional(Box::new(elem.widen())))
            }
            ("max" | "min", [_]) if crate::edition::is_v03() => {
                let message = format!(
                    "`{name}` takes 2 arguments, but 1 was given; use `.{name}()` on a sequence"
                );
                self.error(codes::WRONG_ARG_COUNT, call.span, message);
                Some(Ty::Error)
            }
            ("max" | "min", [first, rest @ ..]) if !rest.is_empty() => {
                if tys.iter().all(|t| t == first) && !matches!(first, Ty::Unknown | Ty::Error) {
                    Some(first.clone())
                } else if tys.iter().all(Ty::is_int) {
                    Some(Ty::Int)
                } else {
                    None
                }
            }
            ("minmax", [a, b]) => {
                let elem = if *a == Ty::Unknown {
                    b.clone()
                } else {
                    a.clone()
                };
                Some(Ty::Tuple(vec![elem.clone(), elem]))
            }
            ("prefix", [arg]) => match arg {
                Ty::Array { index, elem } if index.len() == 1 && matches!(index[0], Ty::Pos(_)) => {
                    Some(Ty::Named {
                        name: "Prefix".into(),
                        args: vec![index[0].clone(), (**elem).clone()],
                    })
                }
                Ty::Array { index, elem }
                    if index.len() == 2
                        && matches!((&index[0], &index[1]), (Ty::Pos(_), Ty::Pos(_))) =>
                {
                    Some(Ty::Named {
                        name: "_Prefix2".into(),
                        args: vec![index[0].clone(), index[1].clone(), (**elem).clone()],
                    })
                }
                Ty::Array { index, elem } if index.len() == 1 && index[0].is_int() => {
                    Some(Ty::Named {
                        name: "_PrefixList".into(),
                        args: vec![(**elem).clone()],
                    })
                }
                Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(Ty::Named {
                    name: "_PrefixList".into(),
                    args: vec![args[0].clone()],
                }),
                _ => None,
            },
            ("choose2", [_]) => Some(Ty::Int),
            ("adjacency", [edges, ..]) => {
                let elem = self.element_ty(edges);
                let v_ty = match &elem {
                    Ty::Record(fields) => fields
                        .iter()
                        .find(|(n, _)| n == "u")
                        .map(|(_, t)| t.clone()),
                    Ty::Tuple(parts) if parts.len() >= 2 => Some(parts[0].clone()),
                    _ => None,
                };
                if let Some(Ty::Pos(dom)) = v_ty {
                    let into_bitset = args.iter().any(|a| {
                        a.name.as_ref().map(|n| n.text.as_str()) == Some("into")
                            && matches!(&a.value.kind, ExprKind::Name(n) if n == "Bitset")
                    });
                    let cell = if into_bitset {
                        Ty::Named {
                            name: "Bitset".into(),
                            args: vec![Ty::Pos(dom.clone())],
                        }
                    } else {
                        Ty::Named {
                            name: "List".into(),
                            args: vec![Ty::Pos(dom.clone())],
                        }
                    };
                    return Some(Ty::Array {
                        index: vec![Ty::Pos(dom)],
                        elem: Box::new(cell),
                    });
                }
                None
            }
            _ => None,
        }
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
        if crate::ty::spec_edition() && self.ambiguous_method(&base_ty, base, name) {
            return Some(Ty::Error);
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
        // Fixed array `.map(f)`: shape-preserving element-wise map `[D...]T -> [D...]U`
        // (including 1D `[D]T` and multi-dimensional `[D, E]T`).
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
                                || matches!(&elem, Ty::Tuple(items) if items.len() == params.len()) =>
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
                            ExprKind::Lambda { params, .. }
                                if crate::edition::is_v03() && params.len() > 1 =>
                            {
                                let expected_arity = match &elem {
                                    Ty::Tuple(items) => items.len(),
                                    _ => 1,
                                };
                                if params.len() != expected_arity {
                                    let message = format!(
                                        "lambda takes {} parameters, but `{elem}` has {expected_arity}",
                                        params.len()
                                    );
                                    self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                                }
                                Ty::Unknown
                            }
                            _ => match self.quietly(|c| c.infer(&arg.value)) {
                                Ty::Fn { ret, .. } => (*ret).clone(),
                                _ => Ty::Unknown,
                            },
                        }
                    } else {
                        Ty::Unknown
                    };
                    return Some(Ty::list(out));
                }
                "partition" => {
                    return Some(Ty::Tuple(vec![Ty::list(elem.clone()), Ty::list(elem)]));
                }
                "group" if !crate::edition::is_v03() => {
                    return Some(Ty::list(Ty::list(elem)));
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
                "filter" if crate::edition::is_v03() => {
                    return Some(Ty::Named {
                        name: "Filter".into(),
                        args: vec![elem],
                    });
                }
                "filter" | "sorted" => {
                    if name.text == "filter" {
                        self.note_walk_only(call);
                    }
                    return Some(Ty::list(elem));
                }
                "argmax" | "argmin" if crate::edition::is_v03() && !base_ty.is_keyed() => {
                    let message = format!(
                        "`{}` requires `Keyed`, but `{}` has type `{base_ty}`",
                        name.text,
                        self.quote(base.span)
                    );
                    self.error(codes::MISSING_IMPL, name.span, message);
                    return Some(Ty::Error);
                }
                "sort" | "reverse" | "shuffle"
                    if crate::edition::is_v03() && !matches!(base_ty, Ty::Domain(_)) =>
                {
                    if !base_ty.is_indexed_mut() {
                        let message = format!(
                            "`{}` requires `IndexedMut`, but `{}` has type `{base_ty}`",
                            name.text,
                            self.quote(base.span)
                        );
                        self.error(codes::MISSING_IMPL, name.span, message);
                        return Some(Ty::Error);
                    }
                    self.require_mutable(base, "changed in place (needs `mut`)");
                }
                "sort" if matches!(base_ty, Ty::Domain(_)) => {
                    return Some(Ty::list(elem));
                }
                _ => {}
            }
        }
        if crate::edition::is_v03() && name.text == "fold" {
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
            Ty::Optional(ref inner) if matches!(name.text.as_str(), "map" | "filter") => {
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
            // `adj.out(u)` is `adj[u]` for a table of neighbour lists, and
            // for no other table.
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
                let members = self.members_of(&type_name)?;
                let projection = members
                    .projections
                    .iter()
                    .find(|p| p.name == name.text)?
                    .clone();
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
                None
            }
            _ => None,
        }
    }

    fn call_kind(&mut self, call: &Expr, callee: &Expr, callee_ty: Ty, args: &[Arg]) -> Ty {
        let callee_text = self.quote(callee.span);
        match callee_ty {
            Ty::Fn { params, ret } => {
                let mut bindings = Bindings::new();
                self.args(&callee_text, "takes", &params, args, call, &mut bindings);
                let callee_name = match &callee.kind {
                    ExprKind::Name(n) => Some(n.as_str()),
                    ExprKind::Generic { base, .. } => match &base.kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    _ => None,
                };
                if crate::edition::is_v03() {
                    if let Some(fn_name) = callee_name {
                        if let Some(Def::Fn { bounds, .. }) =
                            self.global(fn_name).as_deref().cloned()
                        {
                            for (param, traits) in bounds {
                                if let Some(bound_ty) = bindings.get(&param) {
                                    let bound_ty = self.settle(bound_ty);
                                    for trait_name in traits {
                                        if !self.implements_trait(&bound_ty, &trait_name) {
                                            let message = format!(
                                                "`{bound_ty}` does not implement `{trait_name}` required by `{fn_name}`"
                                            );
                                            self.error(codes::MISSING_IMPL, call.span, message);
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
                                                self.error(
                                                    cpl_diag::Code("E0704"),
                                                    call.span,
                                                    format!(
                                                        "record `{actual}` does not satisfy row bound for `{param}`: missing or incompatible field `{req_name}: {req_ty}`"
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                                Ty::Error | Ty::Unknown | Ty::Param(_) => {}
                                other => {
                                    self.error(
                                        cpl_diag::Code("E0704"),
                                        call.span,
                                        format!(
                                            "row bound on `{param}` requires a record, found `{other}`"
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
                        if crate::edition::is_v03() {
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
                let message = format!("`{callee_text}` is {}, not a fn", ty.with_article());
                self.error(codes::NOT_CALLABLE, callee.span, message);
                Ty::Error
            }
            _ => {
                self.plain_args(args);
                Ty::Unknown
            }
        }
    }

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
            if let (Some((name, _)), Ty::Named { name: got, .. }) = (own, &ty) {
                if got != name {
                    let message =
                        format!("`fields` here walks values of the type it builds, `{name}`; this is a `{ty}`");
                    self.error(codes::TYPE_MISMATCH, v.value.span, message);
                }
            }
            shapes.push(self.reflect(&ty).ok().flatten());
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
    fn plain_args(&mut self, args: &[Arg]) {
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
    fn construct(
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
                    let mut bindings = Bindings::new();
                    if !type_args.is_empty() && type_args.len() <= generics.len() {
                        bindings.extend(generics.iter().cloned().zip(type_args.iter().cloned()));
                    }
                    self.args(callee_text, verb, params, args, call, &mut bindings);
                    if is_generated_ctor {
                        self.move_ctor_args(args);
                    }
                    if generics.iter().any(|g| g == "F") && !bindings.contains_key("F") {
                        if let Some(act_arg) =
                            args.iter().find(|a| a.name.as_ref().is_some_and(|n| n.text == "act"))
                        {
                            let act_ty = self.quietly(|c| c.infer(&act_arg.value));
                            let elem_ty = bindings.get("T").cloned().unwrap_or(Ty::Int);
                            let inferred_f = match &act_ty {
                                Ty::Named { name: n, args: aa } if n == "Action" && aa.len() == 2 => {
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
                Def::Alias { ty } => {
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
    fn args(
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
        for arg in args {
            let param = match &arg.name {
                Some(name) if labeled => match params.iter().find(|p| p.name == name.text) {
                    Some(param) => Some(param),
                    None => {
                        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                        let suggestion = closest(&name.text, names.iter().map(String::as_str));
                        let what = if verb == "has" { "field" } else { "parameter" };
                        let message = format!("`{callee}` has no {what} named `{}`", name.text);
                        let code = if crate::edition::is_v03()
                            && callee == "Fenwick"
                            && name.text == "act"
                        {
                            cpl_diag::Code("E0705")
                        } else {
                            codes::UNKNOWN_ARG_NAME
                        };
                        if let Some(diag) = self.error(code, name.span, message) {
                            match suggestion {
                                Some(s) => {
                                    diag.help = Some(format!("did you mean `{s}`?"));
                                    diag.fixes.push(cpl_diag::Fix {
                                        title: format!("change to `{s}`"),
                                        edits: vec![(name.span, s.to_string())],
                                    });
                                }
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
            if crate::edition::is_v03()
                && callee != "List"
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
                let message = format!(
                    "`{callee}` changes `{}`, so the argument must be passed as `mut {quoted}`",
                    param.name
                );
                let at = arg.value.span;
                if let Some(diag) = self.error(codes::MISSING_MUT_AT_CALL, at, message) {
                    diag.fixes.push(cpl_diag::Fix {
                        title: "add `mut`".into(),
                        edits: vec![(Span::point(at.start), "mut ".into())],
                    });
                }
            } else if arg.mutable && !param.mutable && verb == "takes" {
                let message = format!(
                    "`{callee}` doesn't change `{}`, so `mut` doesn't belong here",
                    param.name
                );
                if let Some(diag) = self.error(codes::MISSING_MUT_AT_CALL, arg.value.span, message)
                {
                    diag.help = Some(format!(
                        "remove `mut`, or declare the parameter `mut {}`",
                        param.name
                    ));
                }
            }
            if arg.mutable {
                self.require_mutable(&arg.value, "passed as `mut`");
            }
        }
        if let Some(last) = positional.last().filter(|p| p.variadic) {
            last.ty.unify(&Ty::Tuple(pack), bindings);
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

/// `a[l..r]`: an array seen through a range.
fn is_slice(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Paren(inner) => is_slice(inner),
        ExprKind::Index { indices, .. } => indices
            .iter()
            .any(|i| matches!(i.kind, ExprKind::Range { .. })),
        _ => false,
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
    fn catalog_row(
        &mut self,
        base: &Expr,
        name: &str,
    ) -> Option<(&'static crate::builtins::BuiltinMember, Option<Ty>)> {
        let base_ty = self.quietly(|c| c.infer(base));
        let (owner, elem) = match &base_ty {
            Ty::Str => ("Str", Some(Ty::Char)),
            Ty::Array { .. } => ("[D]T", base_ty.seq_elem()),
            Ty::Named { name, args } => (
                match name.as_str() {
                    "HashSet" => "Set",
                    "HashMap" => "Map",
                    other => other,
                },
                base_ty
                    .seq_elem()
                    .or_else(|| (args.len() == 1).then(|| args[0].clone())),
            ),
            _ => return None,
        };
        Some((crate::builtins::builtin_member(owner, name)?, elem))
    }

    /// What a built-in method returns when nothing more specific knows:
    /// its catalog row's type, e.g. the element for `a.sum()` on a `[V]Int`.
    /// Optional returns (`?T` for `pop`) stay unknown: the checker still
    /// takes them on trust as the element.
    fn catalog_ret(&mut self, callee: &Expr) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let (member, elem) = self.catalog_row(base, &name.text)?;
        catalog_ty(member.ret, &elem)
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
fn is_place(e: &Expr) -> bool {
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
