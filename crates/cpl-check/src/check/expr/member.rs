//! Member access: fields, methods and members of a type, by name.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// The type of field `name` on a value of type `ty`, if `ty` is a
    /// struct or record that has it.
    pub(crate) fn field_of(&mut self, ty: &Ty, name: &ast::Name) -> Option<Ty> {
        match ty {
            Ty::Record(fields) => fields
                .iter()
                .find(|(n, _)| *n == name.text)
                .map(|(_, t)| t.clone()),
            Ty::Named {
                name: type_name,
                args,
            } => {
                let members = self.members_of(type_name)?;
                let bindings = members.bindings(args);
                members
                    .fields
                    .iter()
                    .find(|f| f.name == name.text)
                    .map(|f| self.settle(&f.ty.subst(&bindings)))
            }
            _ => None,
        }
    }

    /// Everything `value.name` can reach on a struct or enum: its fields,
    /// its methods (a `@get` method reads as its value), what its body
    /// owns, and the same from each impl for it.
    pub(crate) fn members_of(&mut self, type_name: &str) -> Option<super::method_form::Members> {
        let mut out = super::method_form::Members::default();
        let add = |out: &mut super::method_form::Members,
                   methods: &[(String, Ty)],
                   getters: &[String]| {
            for (name, ty) in methods {
                let ty = match ty {
                    Ty::Fn { ret, .. } if getters.contains(name) => (**ret).clone(),
                    other => other.clone(),
                };
                out.members.push((name.clone(), ty));
            }
        };
        match self.type_def(type_name).as_deref() {
            Some(Def::Struct {
                generics,
                fields,
                methods,
                getters,
                mutators,
                owned,
                projections,
                selves,
                ..
            }) => {
                out.mutators.extend(mutators.iter().cloned());
                out.selves = selves.clone();
                out.generics = generics.clone();
                out.fields = fields.clone();
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
            Some(Def::Enum { methods, .. }) => add(&mut out, methods, &[]),
            _ => return None,
        }
        for (trait_name, def) in self.env.impls(type_name) {
            // Every fn of the trait comes with the impl: the ones the
            // trait writes a body for, and the ones the impl must write
            // (one it leaves out is reported once, on the impl).
            if let Some(Def::Trait {
                methods, generics, ..
            }) = trait_name
                .as_deref()
                .and_then(|t| t.split('<').next())
                .and_then(|t| self.type_def(t))
                .as_deref()
            {
                let members = self.trait_members(methods, generics, &def);
                out.members.extend(members);
            }
            if let Def::Impl {
                methods,
                getters,
                mutators,
                owned,
                projections,
                ..
            } = &*def
            {
                out.mutators.extend(mutators.iter().cloned());
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
        }
        Some(out)
    }

    pub(super) fn field(&mut self, expr: &Expr, base: &Expr, name: &ast::Name) -> Ty {
        let ty = self.field_ty(expr, base, name);
        if ty == Ty::Error {
            if let Some(path) = written_path(expr) {
                self.exprs.unresolved.push((expr.span, path));
            }
        }
        ty
    }

    fn field_ty(&mut self, expr: &Expr, base: &Expr, name: &ast::Name) -> Ty {
        if let ExprKind::Name(view) = &base.kind {
            if let Some(domain) = self.value_domain(view, &name.text) {
                return Ty::Domain(domain);
            }
        }
        // `treap.merge` after `use lib.treap`: a public name of a module.
        if let ExprKind::Name(alias) = &base.kind {
            if self.local(alias).is_none() && self.env.is_module(alias) {
                return match self.module_member(alias, name) {
                    Some(def) => def.value_ty(&name.text),
                    None if name.text.is_empty() => Ty::Error,
                    None if self.env.is_private_member(alias, &name.text) => {
                        self.report(
                            name.span,
                            msgs::check::PRIVATE_NAME_IS_PRIVATE_TO_MODULE,
                            &[&name.text, &alias],
                        );
                        Ty::Error
                    }
                    None => {
                        let message =
                            cpl_diag::msgs::check::MODULE_HAS_NO_PUBLIC.at(&[&alias, &name.text]);
                        self.report_said(name.span, message);
                        Ty::Error
                    }
                };
            }
        }
        if matches!(name.text.as_str(), "first" | "last") {
            if let Some(end) = self.range_key_end(base) {
                return end;
            }
        }
        // `D.cuts` is never empty: its first and last cuts are always there.
        if let ExprKind::Field {
            base: inner,
            name: bounds,
        } = &base.kind
        {
            if matches!(bounds.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "first" | "last")
            {
                if let Ty::Domain(d) = self.infer(inner) {
                    return Ty::Pos(format!("{d}.cuts"));
                }
            }
        }
        if let Some(ty) = self.library_domain_member(expr, base, name) {
            return ty;
        }
        let base_ty = self.infer_base(base);
        self.note_method(&base_ty, name);
        if self.member_advice(&base_ty, name) {
            return Ty::Error;
        }
        if self.private_member(&base_ty, name) {
            return Ty::Error;
        }
        let ty = self.member_or_getter(expr, name, base_ty.clone());
        let ty = self.through_value(base, &base_ty, ty);
        let base_ty = base_ty.rev_source().cloned().unwrap_or(base_ty);
        let built_in = matches!(
            &base_ty,
            Ty::Str | Ty::Range { .. } | Ty::Pos(_) | Ty::Domain(_) | Ty::Array { .. }
        ) || matches!(&base_ty, Ty::Named { name, .. } if name == "List" || name == "Range");
        // A generator only has what std declares on sequences: no `next` (R07.1.1).
        let generator = matches!(&base_ty, Ty::Named { name, .. } if name == "Gen");
        if ty == Ty::Unknown
            && (built_in || generator)
            && !(if generator {
                crate::prelude::spec_receiver_method_names().contains(&name.text)
            } else {
                (crate::prelude::spec_member_names().contains(&name.text)
                    && !(matches!(&base_ty, Ty::Named { name, .. } if name == "Range")
                        && crate::prelude::spec_struct_only_member_names().contains(&name.text)))
                    || crate::builtins::BUILTIN_MEMBERS
                        .iter()
                        .any(|m| m.name == name.text)
            })
            && self.global(&name.text).is_none()
            && !self.old_copy_spelling(&base_ty, name)
        {
            let message = Self::no_method(&base_ty, name);
            let suggestion = self.member_typo(&base_ty, &name.text);
            let twin = self.method_twin(base, name, &base_ty);
            let in_place = super::advice::in_place_twin(&name.text, &self.quote(base.span));
            let pair = super::advice::iterator_pair_fix(self.src, base.span, name.span);
            let replaced = pair
                .as_ref()
                .and_then(|f| f.edits.first())
                .map(|(at, _)| *at);
            let listed = matches!(&base_ty, Ty::Str | Ty::Array { .. })
                || matches!(&base_ty, Ty::Named { name, .. } if name == "List");
            let position = listed
                .then(|| super::advice::iterator_position_fix(self.src, base.span, name.span))
                .flatten();
            // `lower_bound(..) - a.begin()`: an earlier fix already rewrites it.
            let diag = match self.rewritten(name.span) {
                true => None,
                false => self.report_said(name.span, message),
            };
            if let Some(diag) = diag {
                if let Some((help, fix)) = twin {
                    diag.help = Some(help);
                    diag.fixes.push(fix);
                } else if let Some(help) = in_place.clone() {
                    diag.help = Some(help);
                } else if let Some(help) = super::advice::foreign_method(&name.text) {
                    diag.help = Some(match (&pair, &position) {
                        (None, Some(_)) => crate::builtins::ITERATOR_POSITION_HELP.to_string(),
                        _ => help.to_string(),
                    });
                    diag.fixes.extend(pair.or(position));
                } else if let Some(s) = suggestion {
                    diag.suggest(name.span, s);
                } else if let Some(help) = habits::help(habits::Kind::Call, &name.text)
                    .or_else(|| super::advice::foreign_method(&name.text))
                {
                    diag.help = Some(help.to_string());
                    diag.fixes.extend(pair);
                } else if generator {
                    diag.help = Some("a generator pushes its values into a loop: `for x in g:`, or `List(g)` to keep them (R07.1.1)".to_string());
                }
            }
            // `a.erase(unique(a.begin(), a.end()), a.end())`: the fix rewrites the
            // whole call, so `erase` and `unique` have no error of their own.
            if let Some(at) =
                replaced.filter(|_| super::advice::foreign_method(&name.text).is_some())
            {
                let own = name.span;
                self.sink.drop_where(|d| {
                    matches!(d.code, codes::UNRESOLVED_NAME | codes::UNKNOWN_FIELD)
                        && at.contains(d.span)
                        && d.span != own
                });
            }
            self.iter_call_help(base, &base_ty, name);
        } else if ty == Ty::Unknown && built_in && self.no_extension_takes(&base_ty, name) {
            return Ty::Error;
        }
        // A built-in member (`xs.len`, `v.next`) gets its hover here.
        let hovered = self.hovers.last().is_some_and(|h| h.span == name.span);
        let owner = match &base_ty {
            Ty::Str => Some("Str".to_string()),
            Ty::Array { .. } => Some("[D]T".to_string()),
            Ty::Named { name, .. } => Some(name.clone()),
            // `Team.Home`: a variant named through its enum.
            Ty::TypeName(name) => Some(name.clone()),
            _ => None,
        };
        if let Some(owner) = owner.as_deref() {
            self.missing_parens(expr, base, name, owner, &ty);
            if ty != Ty::Unknown || matches!(owner, "Map" | "Set") {
                self.old_copy_spelling(&base_ty, name);
            }
        }
        let shown = self.shown_member(&ty, name);
        // A catalog method the call types later (`xs.sorted()`) is still
        // known here, by its catalog signature.
        let known = match shown {
            Ty::Error => false,
            Ty::Unknown => owner
                .as_deref()
                .is_some_and(|o| crate::builtins::builtin_member(o, &name.text).is_some()),
            _ => true,
        };
        if !hovered && !name.text.is_empty() && known {
            self.hovers.push(HoverEntry {
                span: name.span,
                name: name.text.clone(),
                ty: shown,
                keyword: "field",
                decl: None,
                owner,
            });
        }
        let positional = match &base_ty {
            Ty::Domain(_) => ["nth", "at", "wrap"].contains(&name.text.as_str()),
            Ty::Pos(_) => ["offset", "ahead", "back"].contains(&name.text.as_str()),
            _ => false,
        };
        if positional && matches!(ty, Ty::Fn { .. }) {
            self.fixed_io.push(name.span);
        }
        ty
    }

    pub(crate) fn member_ty(&mut self, expr: &Expr, name: &ast::Name, base_ty: Ty) -> Ty {
        if name.text.is_empty() {
            return Ty::Error;
        }
        if let Some(ty) = self.member_through_view(expr, name, &base_ty) {
            return ty;
        }
        if let Ty::Named { name: hash, .. } = &base_ty {
            if let Some(message) = crate::builtins::needs_order(hash, &name.text) {
                self.report_said(name.span, message);
                return Ty::Error;
            }
        }
        if let Ty::Param(p) = &base_ty {
            if let Some(req) = self.row_bounds.get(p).cloned() {
                if let Some((_, fty)) = req.iter().find(|(fnm, _)| fnm == &name.text) {
                    return fty.clone();
                }
                self.report(
                    name.span,
                    msgs::check::UNKNOWN_FIELD_ROW_DOES_NOT_NAME,
                    &[&p, &name.text],
                );
                return Ty::Error;
            }
        }
        // `tree.Pre`: the domain a value owns, named by its type parameter.
        if let Ty::Named {
            name: type_name,
            args,
        } = &base_ty
        {
            if let Some(members) = self.members_of(type_name) {
                if members.generics.contains(&name.text) {
                    return match members.bindings(args).get(&name.text) {
                        Some(Ty::Pos(domain)) => Ty::Domain(domain.clone()),
                        _ => Ty::Unknown,
                    };
                }
            }
        }
        if let Some(elem) = base_ty.seq_elem() {
            match name.text.as_str() {
                "len" => {
                    if !base_ty.is_sized() {
                        self.lazy_len(&base_ty, name);
                        return Ty::Error;
                    }
                    return Ty::Int;
                }
                "is_empty" => return Ty::Bool,
                "last" | "rev" if !base_ty.is_bidir() => {
                    self.report(
                        name.span,
                        msgs::check::MISSING_IMPL_REQUIRES_BIDIR_WHICH_DOES,
                        &[&name.text, &base_ty],
                    );
                    return Ty::Error;
                }
                "items" | "keys" if !base_ty.is_keyed() => {
                    self.report(
                        name.span,
                        msgs::check::MISSING_IMPL_REQUIRES_KEYED_WHICH_DOES,
                        &[&name.text, &base_ty],
                    );
                    return Ty::Error;
                }
                "first" | "last"
                    if base_ty == Ty::Str
                        || matches!(&base_ty, Ty::Named { name, .. } if name == "List") =>
                {
                    return Ty::Optional(Box::new(elem));
                }
                "get" => {
                    return Ty::Fn {
                        params: vec![crate::Param {
                            name: "i".into(),
                            ty: Ty::Int,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Optional(Box::new(elem))),
                    };
                }
                "at" => {
                    let (arg, ret) = match &base_ty {
                        Ty::Domain(d) => match self.product_factors(d) {
                            Some(factors) => (
                                Ty::Tuple(vec![Ty::Unknown; factors.len()]),
                                Ty::Optional(Box::new(elem)),
                            ),
                            None => (Ty::Int, Ty::Optional(Box::new(elem))),
                        },
                        _ => (Ty::Int, elem),
                    };
                    return Ty::Fn {
                        params: vec![crate::Param {
                            name: "i".into(),
                            ty: arg,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(ret),
                    };
                }
                "set_at" if base_ty.is_indexed_mut() => {
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "i".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "x".into(),
                                ty: elem,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Unit),
                    };
                }
                "swap_idx" if base_ty.is_indexed_mut() => {
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "i".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "j".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Unit),
                    };
                }
                "reach" => {
                    let pred_ty = Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "a".into(),
                                ty: elem.clone(),
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "b".into(),
                                ty: elem,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::Bool),
                    };
                    return Ty::Fn {
                        params: vec![
                            crate::Param {
                                name: "while".into(),
                                ty: pred_ty,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            },
                            crate::Param {
                                name: "cyclic".into(),
                                ty: Ty::Bool,
                                mutable: false,
                                has_default: true,
                                named_only: true,
                                variadic: false,
                                lazy: false,
                            },
                        ],
                        ret: Box::new(Ty::list(Ty::Int)),
                    };
                }
                "halves" if matches!(&base_ty, Ty::Named { name, .. } if name == "Range") => {
                    return Ty::Fn {
                        params: vec![],
                        ret: Box::new(Ty::Tuple(vec![base_ty.clone(), base_ty.clone()])),
                    };
                }
                _ => {}
            }
        }
        let own_pow =
            matches!(&base_ty, Ty::Named { .. }) && !self.implements_trait(&base_ty, "Pow");
        if name.text == "pow" && !own_pow {
            let fix = self.pow_fix(expr, name);
            if let Some(diag) = self.report(
                name.span,
                msgs::check::UNKNOWN_FIELD_POWER_USES_NOT_POW,
                &[],
            ) {
                let shown = fix.as_ref().map_or("a ** e", |f| f.edits[0].1.as_str());
                diag.help = Some(format!("write `{shown}`"));
                diag.fixes.extend(fix);
            }
            return Ty::Error;
        }
        if name.text == "enumerate" && base_ty.is_seq() {
            let message = cpl_diag::msgs::check::NO_ENUMERATE_SEQUENCES_CP.at(&[]);
            if let Some(diag) = self.report_said(name.span, message) {
                diag.help = Some("use `.items`: `for (i, x) in xs.items:`".to_string());
            }
            return Ty::Error;
        }
        let renamed = match name.text.as_str() {
            "bsearch_first" => Some((
                "first_true",
                "`r.first_true(ok)` gives the first element where `ok` holds, as `?`",
            )),
            "bsearch_last" => Some((
                "last_true",
                "`r.last_true(ok)` gives the last element where `ok` holds, as `?`",
            )),
            "bounds" if matches!(base_ty, Ty::Domain(_)) => {
                Some(("cuts", "`D.cuts` is the boundaries of `D`"))
            }
            _ => None,
        };
        if let Some((to, help)) = renamed {
            if let Some(diag) = self.report(
                name.span,
                msgs::check::REMOVED_SYNTAX_IS_GONE_IN_0,
                &[&name.text, &to],
            ) {
                diag.help = Some(help.to_string());
                diag.fixes.push(cpl_diag::Fix {
                    title: format!("use `.{to}`"),
                    edits: vec![(name.span, to.to_string())],
                });
            }
            return Ty::Error;
        }

        // `r.lo` / `r.hi` on `Range<T>` were renamed to `.start` / `.end` (`end` exclusive).
        if matches!(name.text.as_str(), "lo" | "hi")
            && matches!(&base_ty, Ty::Named { name: r, .. } if r == "Range")
        {
            let replacement = if name.text == "lo" { "start" } else { "end" };
            if let Some(diag) = self.report(
                name.span,
                msgs::check::REMOVED_SYNTAX_RANGE_USES_NOT,
                &[&replacement, &name.text],
            ) {
                diag.help = Some(format!(
                    "write `.{replacement}` (`.end` is exclusive for `a..b` and `a..=b` alike)"
                ));
                diag.fixes.push(cpl_diag::Fix {
                    title: format!("use `.{replacement}`"),
                    edits: vec![(name.span, replacement.to_string())],
                });
            }
            return Ty::Error;
        }
        // `r.start` / `r.end` on `Range<T>` give the lower / exclusive-upper endpoint.
        if matches!(name.text.as_str(), "start" | "end") {
            if let Ty::Named { name: r, args } = &base_ty {
                if r == "Range" {
                    return match args.first() {
                        Some(Ty::Pos(d)) => {
                            if d.ends_with(".cuts") {
                                Ty::Pos(d.clone())
                            } else {
                                Ty::Pos(format!("{d}.cuts"))
                            }
                        }
                        Some(t @ (Ty::I128 | Ty::Float)) => t.clone(),
                        Some(Ty::Int | Ty::Range { .. }) => Ty::Int,
                        // `Range<D>` over a generic `D`: an endpoint is a `D`.
                        Some(param @ Ty::Param(_)) => param.clone(),
                        _ => Ty::Unknown,
                    };
                }
            }
        }
        // `r.drop(k)`, `r.take(k)`, `r.drop_last(k)`, `r.take_last(k)` on `Range<T>` return `Range<T>`.
        if matches!(
            name.text.as_str(),
            "drop" | "take" | "drop_last" | "take_last"
        ) && matches!(&base_ty, Ty::Named { name: r, .. } if r == "Range")
        {
            return Ty::Fn {
                params: vec![crate::ty::Param {
                    name: "k".into(),
                    ty: Ty::Int,
                    mutable: false,
                    has_default: false,
                    named_only: false,
                    variadic: false,
                    lazy: false,
                }],
                ret: Box::new(base_ty),
            };
        }
        // `s.rev`, `xs.rev`, `a.rev`: O(1) read-only reverse view over a string, list, or 1D domain array.
        if name.text == "rev" {
            if matches!(&base_ty, Ty::Named { name: range, args } if range == "Range" && args.len() == 1)
            {
                return base_ty;
            }
            if base_ty == Ty::Str
                || matches!(&base_ty, Ty::Named { name: list, args } if list == "List" && args.len() == 1)
                || matches!(&base_ty, Ty::Array { index, .. } if index.len() == 1)
            {
                self.note_readonly_view(expr, "reverse view");
                // A reverse view is not `Indexed`: it keeps its source's keys (R07.3.5).
                return Ty::rev_of(base_ty);
            }
        }
        // `xs.items`: each element with its index (an `I` for `[I]T`, else `Int`).
        if name.text == "items" {
            let pair = match &base_ty {
                Ty::Named { name: list, args } if list == "List" && args.len() == 1 => {
                    Some((Ty::Int, args[0].clone()))
                }
                Ty::Str => Some((Ty::Int, Ty::Char)),
                Ty::Named { name: map, args }
                    if matches!(map.as_str(), "Map" | "HashMap") && args.len() == 2 =>
                {
                    Some((args[0].clone(), args[1].clone()))
                }
                Ty::Array { index, elem } => {
                    let mut at: Vec<Ty> = index
                        .iter()
                        .map(|axis| match axis {
                            Ty::Pos(domain) => Ty::Pos(domain.clone()),
                            _ => Ty::Int,
                        })
                        .collect();
                    // `((r, c), x)` for a grid; one index stands alone.
                    let at = if at.len() == 1 {
                        at.remove(0)
                    } else {
                        Ty::Tuple(at)
                    };
                    Some((at, (**elem).clone()))
                }
                _ => None,
            };
            if let Some((at, elem)) = pair {
                return Ty::Named {
                    name: "List".into(),
                    args: vec![Ty::Tuple(vec![at, elem])],
                };
            }
        }
        if let Ty::Named { name: sr, args } = &base_ty {
            if sr == "SortedRange" && args.len() == 1 {
                if name.text == "len" {
                    if let ExprKind::Field { base: recv, .. } = &expr.kind {
                        if let Some(owner) = self.uncounted_sorted_range_owner(recv) {
                            if let Some(diag) = self.report(
                                name.span,
                                msgs::check::TYPE_MISMATCH_RANGE_LEN_IS_NOT,
                                &[&owner],
                            ) {
                                diag.help = Some(format!(
                                    "use `SortedSet` or `SortedBag` for O(log n) `range(r).len`, or count by iterating `{owner}.range(...)`"
                                ));
                            }
                        }
                    }
                }
                return match name.text.as_str() {
                    "first" | "last" => Ty::Optional(Box::new(args[0].clone())),
                    "len" => Ty::Int,
                    "is_empty" => Ty::Bool,
                    "rev" => base_ty.clone(),
                    _ => Ty::Unknown,
                };
            }
        }
        if name.text == "inverse" {
            if let Ty::Array { index, elem } = &base_ty {
                if let Some(ret) = self.array_inverse_ret(index, elem) {
                    return Ty::Fn {
                        params: Vec::new(),
                        ret: Box::new(ret),
                    };
                }
            }
        }
        if matches!(name.text.as_str(), "keys" | "values") {
            if let Ty::Named { name: map, args } = &base_ty {
                if matches!(map.as_str(), "Map" | "HashMap") && args.len() == 2 {
                    let elem = if name.text == "keys" {
                        args[0].clone()
                    } else {
                        args[1].clone()
                    };
                    return Ty::Named {
                        name: "List".into(),
                        args: vec![elem],
                    };
                }
                // A range's keys are its own elements (R04.4.31).
                if name.text == "keys" && matches!(map.as_str(), "List" | "Str" | "Range") {
                    let key = args.first().filter(|_| map == "Range").cloned();
                    return range_ty(key.unwrap_or(Ty::Int));
                }
            }
            if name.text == "keys" {
                // A grid's keys are its cells (R04.2.10).
                if let Ty::Array { index, .. } = &base_ty {
                    let key = match &index[..] {
                        [one] => Some(one.clone()),
                        [] => None,
                        _ => Some(Ty::Tuple(index.clone())),
                    };
                    if let Some(key) = key {
                        return range_ty(key);
                    }
                }
            }
            if let Ty::Array { index, elem } = &base_ty {
                if index.len() == 1 {
                    let item = if name.text == "keys" {
                        index[0].clone()
                    } else {
                        (**elem).clone()
                    };
                    return Ty::Named {
                        name: "List".into(),
                        args: vec![item],
                    };
                }
            }
        }
        let found = match &base_ty {
            Ty::Record(fields) => {
                if !fields.iter().any(|(n, _)| *n == name.text) {
                    if let Some(ty) = self.shape_default_member(&base_ty, &name.text) {
                        return ty;
                    }
                }
                let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                Some((self.field_of(&base_ty, name), names, Vec::new()))
            }
            Ty::Tuple(items) => {
                return match name.text.parse::<usize>() {
                    Ok(i) if i < items.len() => items[i].clone(),
                    _ => match (self.own_method_at(&base_ty, name))
                        .or_else(|| self.shape_default_member(&base_ty, &name.text))
                    {
                        Some(ty) => ty,
                        // A product, `(W, W).all(..)`, or a fn of the name
                        // the call picks once it sees the arguments.
                        None if items
                            .iter()
                            .any(|t| matches!(t, Ty::Domain(_) | Ty::TypeName(_)))
                            || !self.method_candidates(&name.text).is_empty() =>
                        {
                            Ty::Unknown
                        }
                        None => {
                            self.unknown_member(name, &base_ty, &[], &[]);
                            Ty::Error
                        }
                    },
                };
            }
            Ty::Named {
                name: type_name,
                args,
            } => {
                if type_name == "Pair" && name.text == "count" {
                    return Ty::Int;
                }
                if type_name == "Gen" && name.text == "next" {
                    let message = cpl_diag::msgs::check::HAS_NO_METHOD_NEXT.at(&[&base_ty]);
                    self.report_said(name.span, message);
                    return Ty::Error;
                }
                if matches!(type_name.as_str(), "List" | "Str" | "Map" | "Set")
                    && name.text == "buf"
                {
                    self.report(
                        name.span,
                        msgs::check::UNKNOWN_FIELD_HAS_NO_PUBLIC_FIELD,
                        &[&base_ty, &name.text],
                    );
                    return Ty::Error;
                }
                if args.is_empty() && self.finite_index_enum_size(type_name).is_some() {
                    match name.text.as_str() {
                        "ord" | "idx" | "label" => return Ty::Int,
                        "l" | "r" => return Ty::Pos(format!("{type_name}.cuts")),
                        "prev" | "next" => return Ty::Optional(Box::new(base_ty.clone())),
                        "offset" | "ahead" | "back" => {
                            return Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "k".into(),
                                    ty: Ty::Int,
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Optional(Box::new(base_ty.clone()))),
                            };
                        }
                        _ => {}
                    }
                }
                {
                    let has_inherent = match self.type_def(type_name).as_deref() {
                        Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) => {
                            methods.iter().any(|(m, _)| m == &name.text)
                        }
                        _ => false,
                    };
                    if !has_inherent {
                        let mut trait_hits: Vec<String> = Vec::new();
                        for (trait_name, def) in self.env.impls(type_name) {
                            let Some(tn) = trait_name else { continue };
                            let has_in_impl = matches!(
                                &*def,
                                Def::Impl { methods, .. }
                                    if methods.iter().any(|(m, _)| m == &name.text)
                            );
                            let has_in_trait = self
                                .type_def(tn.split('<').next().unwrap_or(&tn))
                                .as_deref()
                                .is_some_and(|d| {
                                    matches!(
                                        d,
                                        Def::Trait { methods, .. }
                                            if methods.iter().any(|m| m.name == name.text)
                                    )
                                });
                            if (has_in_impl || has_in_trait) && !trait_hits.contains(&tn) {
                                trait_hits.push(tn);
                            }
                        }
                        if trait_hits.len() >= 2 {
                            if let Some(diag) = self.report(
                                name.span,
                                msgs::check::AMBIGUOUS_METHOD_IS_AMBIGUOUS_BOTH_AND_2,
                                &[&type_name, &name.text, &trait_hits[0], &trait_hits[1]],
                            ) {
                                diag.help = Some(format!(
                                    "disambiguate with `{}.{}(...)` or `{}.{}(...)`",
                                    trait_hits[0], name.text, trait_hits[1], name.text
                                ));
                            }
                            return Ty::Error;
                        }
                    }
                }
                self.members_of(type_name).map(|members| {
                    let bindings = members.bindings(args);
                    (
                        members
                            .fields
                            .iter()
                            .find(|f| f.name == name.text)
                            .map(|f| f.ty.subst(&bindings)),
                        members.fields.iter().map(|f| f.name.clone()).collect(),
                        members
                            .members
                            .iter()
                            .map(|(m, ty)| {
                                (m.clone(), members.member_ty(m, ty, &base_ty, &bindings))
                            })
                            .collect::<Vec<_>>(),
                    )
                })
            }
            Ty::Param(param) => {
                return match self.bound_method(param, &name.text) {
                    Some(Ty::Fn { params, ret }) if !self.in_call_callee && params.is_empty() => {
                        *ret
                    }
                    other => other.unwrap_or(Ty::Unknown),
                };
            }
            // The members of a domain that give positions or ranges back.
            Ty::Domain(domain) => {
                return match name.text.as_str() {
                    "first" | "last" => Ty::Optional(Box::new(Ty::Pos(domain.clone()))),
                    "all" => Ty::Unknown,
                    "rev" => Ty::Domain(domain.clone()),
                    "mask" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "vs".into(),
                            ty: Ty::Unknown,
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Array {
                            index: vec![Ty::Pos(domain.clone())],
                            elem: Box::new(Ty::Bool),
                        }),
                    },
                    "wrap" | "nth" => int_fn(Ty::Pos(domain.clone())),
                    "bounds" | "cuts" => range_ty(Ty::Pos(format!("{domain}.cuts"))),
                    "Cut" => {
                        self.planned(name.span, &cpl_syntax::planned::CUT_DOMAIN);
                        Ty::Error
                    }
                    "ranges" => Ty::Named {
                        name: "List".into(),
                        args: vec![range_ty(Ty::Pos(domain.clone()))],
                    },
                    "pairs" => Ty::Named {
                        name: "List".into(),
                        args: vec![Ty::Tuple(vec![
                            Ty::Pos(domain.clone()),
                            Ty::Pos(domain.clone()),
                        ])],
                    },
                    "after" | "before" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "pos".into(),
                            ty: Ty::Pos(domain.clone()),
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(range_ty(Ty::Pos(domain.clone()))),
                    },
                    "partition" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "keep".into(),
                            ty: Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "".into(),
                                    ty: Ty::Pos(domain.clone()),
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Bool),
                            },
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::Tuple(vec![
                            Ty::list(Ty::Pos(domain.clone())),
                            Ty::list(Ty::Pos(domain.clone())),
                        ])),
                    },
                    "filter" => Ty::Fn {
                        params: vec![crate::ty::Param {
                            name: "pred".into(),
                            ty: Ty::Fn {
                                params: vec![crate::ty::Param {
                                    name: "".into(),
                                    ty: Ty::Pos(domain.clone()),
                                    mutable: false,
                                    has_default: false,
                                    named_only: false,
                                    variadic: false,
                                    lazy: false,
                                }],
                                ret: Box::new(Ty::Bool),
                            },
                            mutable: false,
                            has_default: false,
                            named_only: false,
                            variadic: false,
                            lazy: false,
                        }],
                        ret: Box::new(Ty::list(Ty::Pos(domain.clone()))),
                    },
                    "sort" => Ty::Unknown,
                    _ => Ty::Unknown,
                };
            }
            Ty::Char => {
                if matches!(name.text.as_str(), "ord" | "idx") {
                    return Ty::Int;
                }
                return Ty::Unknown;
            }
            Ty::Pos(domain) => {
                match name.text.as_str() {
                    "ord" | "idx" | "label" => return Ty::Int,
                    "prev" | "next" => return Ty::Optional(Box::new(Ty::Pos(domain.clone()))),
                    "offset" | "ahead" | "back" => {
                        return Ty::Fn {
                            params: vec![crate::ty::Param {
                                name: "k".into(),
                                ty: Ty::Int,
                                mutable: false,
                                has_default: false,
                                named_only: false,
                                variadic: false,
                                lazy: false,
                            }],
                            ret: Box::new(Ty::Optional(Box::new(Ty::Pos(domain.clone())))),
                        };
                    }
                    // The boundaries on either side of a position.
                    // R04.2.19: the pieces on either side of a position.
                    "before" | "after" if !domain.ends_with(".cuts") => {
                        return range_of(Ty::Pos(domain.clone()));
                    }
                    "l" | "r" | "before" | "after" if !domain.ends_with(".cuts") => {
                        return Ty::Pos(format!("{domain}.cuts"));
                    }
                    _ => {}
                }
                if let Some(factors) = self.product_factors(domain) {
                    if let Some((_, ty)) = factors
                        .iter()
                        .find(|(fname, _)| fname.as_deref() == Some(name.text.as_str()))
                    {
                        return ty.clone();
                    }
                    if let Ok(idx) = name.text.parse::<usize>() {
                        if let Some((_, ty)) = factors.get(idx) {
                            return ty.clone();
                        }
                    }
                }
                return self.domain_impl_member(&base_ty, &name.text);
            }
            // `Dir.L`: a variant named through its enum.
            Ty::TypeName(type_name) => {
                if let Some(Def::Enum { variants, .. }) = self.type_def(type_name).as_deref() {
                    if let Some((_, fields)) = variants.iter().find(|(v, _)| *v == name.text) {
                        return Def::variant_value(type_name, fields);
                    }
                }
                if type_name == "Char" && matches!(name.text.as_str(), "count" | "len") {
                    return Ty::Int;
                }
                if self.finite_index_enum_size(type_name).is_some() {
                    let elem = Ty::Named {
                        name: type_name.clone(),
                        args: Vec::new(),
                    };
                    match name.text.as_str() {
                        "count" | "len" if !self.in_call_callee => return Ty::Int,
                        // The ordinal members a domain has (R04.2).
                        "at" | "try_at" => return int_fn(Ty::Optional(Box::new(elem))),
                        "wrap" | "nth" => return int_fn(elem),
                        "first" | "last" => return Ty::Optional(Box::new(elem)),
                        "all" | "rev" => {
                            return range_ty(elem);
                        }
                        "bounds" | "cuts" => {
                            return range_ty(Ty::Pos(format!("{type_name}.cuts")));
                        }
                        // R04.2.20: a domain is the `Seq` of its positions.
                        _ if matches!(
                            self.type_def(type_name).as_deref(),
                            Some(Def::Enum { .. })
                        ) && !self
                            .members_of(type_name)
                            .is_some_and(|m| m.members.iter().any(|(m, _)| *m == name.text)) =>
                        {
                            return self.member_ty(expr, name, Ty::list(elem));
                        }
                        _ => {}
                    }
                }
                // `Graph.read()`, `Rooted.Pre`: what the type has without a value.
                self.members_of(type_name)
                    .map(|members| (None, Vec::new(), members.members))
            }
            // A fn type's impls answer their own members; any other name is a std fn
            // taking it as `self` (`f.opt()`), resolved once std is loaded.
            Ty::Fn { .. } => {
                let methods = self.fn_impl_members(&base_ty);
                (methods.iter().any(|(m, _)| *m == name.text)).then(|| (None, Vec::new(), methods))
            }
            _ => None,
        };
        let Some((field_ty, field_names, methods)) = found else {
            // Members of library types, arrays and domains (`xs.len`,
            // `V.rev`) come from `std`, which is not loaded yet.
            return if base_ty == Ty::Error {
                Ty::Error
            } else {
                Ty::Unknown
            };
        };
        let ty = match field_ty {
            Some(ty) => self.settle(&ty),
            None => match methods.iter().find(|(m, _)| *m == name.text) {
                Some((_, ty)) => self.settle(ty),
                // `fn area(self: Sq) -> Int` at the top of the file is
                // a method of `Sq`, and of no other struct.
                None if self.own_method(&base_ty, &name.text).is_some() => {
                    match self.own_method_at(&base_ty, name).unwrap_or(Ty::Unknown) {
                        Ty::Fn { params, ret } if !self.in_call_callee && params.is_empty() => *ret,
                        other => other,
                    }
                }
                None => {
                    self.unknown_member(name, &base_ty, &field_names, &methods);
                    return Ty::Error;
                }
            },
        };
        self.hovers.push(HoverEntry {
            span: name.span,
            name: name.text.clone(),
            ty: ty.clone(),
            keyword: "field",
            decl: None,
            owner: match &base_ty {
                Ty::Named { name, .. } | Ty::TypeName(name) => Some(name.clone()),
                _ => None,
            },
        });
        ty
    }
}

/// `t.pre.order` as `["t", "pre", "order"]`: a member path of plain names.
fn written_path(expr: &Expr) -> Option<Vec<String>> {
    match &expr.kind {
        ExprKind::Name(n) => Some(vec![n.clone()]),
        ExprKind::Field { base, name } => {
            let mut path = written_path(base)?;
            path.push(name.text.clone());
            Some(path)
        }
        ExprKind::Paren(inner) => written_path(inner),
        _ => None,
    }
}
