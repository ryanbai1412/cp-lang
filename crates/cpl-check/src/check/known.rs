//! Prelude fns the checker knows more about than their signatures say.

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{Arg, Expr, ExprKind, UnOp};

impl<'a> Checker<'a> {
    /// What the checker knows about a prelude fn it otherwise takes on
    /// trust: `span(a, b)` covers two ends of one domain, and the `max` or
    /// `min` of a slice is optional, since `a[l..l]` is empty.
    /// `min(a, b)`: two or more plain values call the core binary `min`
    /// or `max`, even beside `std.seq`'s fold of that name (R11.3.3).
    pub(super) fn binary_min_max(callee: &Expr, args: &[Arg]) -> bool {
        matches!(&callee.kind, ExprKind::Name(n) if matches!(n.as_str(), "min" | "max"))
            && args.len() >= 2
            && args.iter().all(|a| a.name.is_none() && !a.mutable)
    }

    /// `count(x > 1 for x in xs)`: a free `count`, `all` or `any` of one
    /// generator tests each value it makes, even beside `std.seq`'s fold
    /// of that name (decision 53).
    pub(super) fn generator_test(callee: &Expr, args: &[Arg]) -> bool {
        matches!(&callee.kind, ExprKind::Name(n) if matches!(n.as_str(), "count" | "all" | "any"))
            && matches!(args, [arg] if arg.name.is_none()
                && !arg.mutable
                && matches!(arg.value.kind, ExprKind::Comprehension { .. }))
    }

    /// `argmax(f(v) for v in V)`: the language's position form over one
    /// walk, even beside `std.seq`'s `argmax` of that name.
    pub(super) fn position_walk(callee: &Expr, args: &[Arg]) -> bool {
        matches!(&callee.kind, ExprKind::Name(n) if crate::prelude::SEQUENCE_FORMS.contains(&n.as_str()))
            && matches!(args, [arg] if arg.name.is_none()
                && !arg.mutable
                && matches!(arg.value.kind, ExprKind::Comprehension { .. }))
    }

    /// What `argmax(f(v) for v in V)` names: the value its `for` binds, or
    /// over several `for`s or a pattern the tuple of the names they bind.
    fn walk_position(&mut self, clauses: &[cpl_syntax::ast::Clause]) -> Ty {
        self.push_scope();
        self.branch += 1;
        let mut names = Vec::new();
        for clause in clauses {
            let iter_ty = self.infer(&clause.iter);
            let elem = self.walk_elem_ty(&clause.iter, &iter_ty);
            self.bind_whole(&clause.pat, &elem, false, "for");
            for cond in &clause.conds {
                self.cond(cond);
            }
            super::pattern::name_spans(&clause.pat, false, &mut names);
        }
        let mut tys: Vec<Ty> = names
            .iter()
            .map(|(span, _)| {
                let name = self.text(*span).to_string();
                self.local(&name).map_or(Ty::Unknown, |l| l.ty.clone())
            })
            .collect();
        self.branch -= 1;
        self.pop_scope();
        match tys.len() {
            1 => tys.remove(0),
            _ => Ty::Tuple(tys),
        }
    }

    /// `xs.sort(x => -x)` on a built-in row: the key goes by name (decision 290).
    pub(super) fn sort_key_by_position(&mut self, callee: &Expr, args: &[Arg]) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        let [arg] = args else {
            return;
        };
        if arg.name.is_some()
            || !matches!(arg.value.kind, ExprKind::Lambda { .. })
            || !matches!(name.text.as_str(), "sort" | "sorted")
            || self.catalog_row(base, &name.text).is_none()
        {
            return;
        }
        let at = arg.value.span;
        let title = format!("write `by: {}`", self.quote(at));
        if let Some(diag) = self.report(at, msgs::check::WRONG_ARG_COUNT_A_SORT_TAKES_ITS, &[]) {
            diag.fix(title, Span::new(at.start, at.start), "by: ");
        }
    }

    /// `b.count()` or `count(b)` on bools: `count()` counts every element, the falses too, so it
    /// is an error naming the two questions it could mean (decision 378, R07.9.4).
    pub(super) fn bool_count(&mut self, call: &Expr, over: &Ty) {
        // A `count` of the type's own, not the `Seq` one every sequence has, settles it.
        let own = self.method_def(over, "count").is_some_and(|def| {
            !matches!(&*def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
                if matches!(params.first().map(|p| &p.ty), Some(Ty::Param(_))))
        });
        if self.element_ty(over) != Ty::Bool || own {
            return;
        }
        if let Some(diag) = self.report(
            call.span,
            msgs::check::WRONG_ARG_COUNT_COUNT_WITH_NO_TEST,
            &[],
        ) {
            diag.help = Some("the trues are `count(x => x)`; every element is `len`".into());
        }
    }

    /// The type of a call the checker knows as the prelude's, recorded as
    /// resolving there.
    pub(super) fn prelude_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ty = self.known_prelude_call(call, callee, args)?;
        if let ExprKind::Name(name) = &callee.kind {
            let resolved = Resolved {
                module: "prelude".into(),
                name: name.clone(),
                owner: None,
            };
            self.resolved.push((callee.span, resolved));
        }
        Some(ty)
    }

    fn known_prelude_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        let generator = Self::generator_test(callee, args);
        // The core fn wins over a fold of its name, a module's or the
        // file's own `fn max(self: ..)`, never over a file's `fn max(a, b)`
        // or `let max` (R11.2.3).
        let position = Self::position_walk(callee, args);
        let core = (Self::binary_min_max(callee, args) || generator || position)
            && (self.env.origin(name).is_some()
                || self
                    .free_global(name)
                    .is_some_and(|d| super::method_form::takes_self(&d)));
        let shadowed = name != "zip" && !core;
        if self.local(name).is_some() || (self.free_global(name).is_some() && shadowed) {
            return None;
        }
        if name == "gen" && args.is_empty() && matches!(callee.kind, ExprKind::Name(_)) {
            self.bare_gens.push(call.span);
        }
        let tys: Vec<Ty> = args
            .iter()
            .map(|arg| self.infer_quietly(&arg.value))
            .collect();
        match (name.as_str(), tys.as_slice()) {
            ("gen", [_]) if matches!(args[0].value.kind, ExprKind::Range { .. }) => Some(Ty::Int),
            ("List", []) => Some(Ty::Named {
                name: "List".into(),
                args: vec![],
            }),
            ("List", [only]) => {
                let elem = match only {
                    Ty::Named { name, args }
                        if matches!(
                            name.as_str(),
                            "Gen" | "Iter" | "Filter" | "Zip" | "Range" | "Map"
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
                    // `List(w)` copies what a `for` over `w` walks: a
                    // `Str`'s `Char`s, a list's items (R04b.3.2).
                    other => match self.element_ty(other) {
                        Ty::Unknown => other.clone(),
                        elem => elem,
                    },
                };
                Some(Ty::list(elem))
            }
            ("List", [first, ..]) => Some(Ty::list(first.clone())),
            ("laws_hold", [_, _]) => Some(Ty::Bool),
            ("read_line", _) => Some(Ty::Str),
            ("print", _) => {
                let allow_none = args
                    .iter()
                    .any(|a| a.name.as_ref().is_some_and(|n| n.text == "none"));
                for (arg, ty) in args.iter().zip(&tys) {
                    if arg.name.is_none() {
                        self.require_show(&arg.value, ty, allow_none, false);
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
                                        self.report(
                                            arg.value.span,
                                            msgs::check::WRONG_DOMAIN_ZIP_EXPECTS_MATCHING_DOMAINS,
                                            &[&self.quote(arg.value.span), &d, &prev],
                                        );
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
            ("count", [_]) if generator => Some(Ty::Int),
            ("all" | "any", [_]) if generator => Some(Ty::Bool),
            ("abs", [Ty::Int | Ty::Range { .. }]) => Some(Ty::Int),
            ("abs", [t @ (Ty::I128 | Ty::Float)]) => Some(t.clone()),
            ("copy", [t]) if args[0].name.is_none() => Some(t.clone()),
            // `first(xs)`, `last(xs, x => x > 3)`: the free form of the members, an optional.
            ("first" | "last", [over, ..])
                if args.len() <= 2
                    && args[0].name.is_none()
                    && !matches!(over, Ty::Named { name, .. } if name.contains("Map")) =>
            {
                match self.element_ty(over) {
                    Ty::Unknown => None,
                    elem => Some(Ty::Optional(Box::new(elem))),
                }
            }
            // `position` holds only for the `SEQUENCE_FORMS`.
            _ if position => {
                let ExprKind::Comprehension { clauses, .. } = &args[0].value.kind else {
                    return None;
                };
                let at = self.quietly(|c| c.walk_position(clauses));
                Some(Ty::Optional(Box::new(at)))
            }
            // `none` is neither skipped nor passed on by `sum` (v2.2).
            ("sum", [over]) => {
                let elem = self.element_ty(over);
                if matches!(elem, Ty::Optional(_)) {
                    if let Some(diag) = self.report(
                        args[0].value.span,
                        msgs::check::BAD_OPERANDS_SUM_ADDS_NUMBERS_NOT,
                        &[&elem.with_article()],
                    ) {
                        diag.help = Some(
                            "`sum` does not skip or absorb `None`: add `x ?? 0`, or filter with `if let`".into(),
                        );
                    }
                }
                None
            }
            // `max(e for x in s)` is the free form of `s.map(x => e).max()` (R01.10.1).
            ("max" | "min", [over])
                if args[0].value.is_generator() && args[0].name.is_none() && !args[0].mutable =>
            {
                Some(Ty::min_max_of(self.element_ty(over)))
            }
            ("max" | "min", [_]) => {
                let xs = self.text(args[0].value.span);
                let xs = if xs.contains(' ') && !xs.starts_with('[') {
                    format!("({xs})")
                } else {
                    xs.to_string()
                };
                let ours = format!("{xs}.{name}()!");
                if let Some(diag) = self.report(
                    call.span,
                    msgs::check::WRONG_ARG_COUNT_TAKES_2_ARGUMENTS_BUT,
                    &[&name, &name],
                ) {
                    diag.fix(format!("write `{ours}`"), call.span, ours);
                }
                Some(Ty::Error)
            }
            ("max" | "min", [first, rest @ ..]) if !rest.is_empty() => {
                if tys.iter().all(|t| t == first) && !matches!(first, Ty::Unknown | Ty::Error) {
                    Some(first.clone())
                } else if tys.iter().all(Ty::is_int) {
                    Some(Ty::Int)
                } else if tys.contains(&Ty::Unknown) {
                    // Two plain values are the core fn while a type is still open, never a fold.
                    Some(Ty::Unknown)
                } else {
                    None
                }
            }
            // `prefix(a)` is `Prefix(a)`: the word std's `@word(prefix)` built on the type.
            (word, [_]) if let Some(on) = crate::prelude::word_type(word) => {
                let ctor = Expr {
                    kind: ExprKind::Name(on.to_string()),
                    span: callee.span,
                };
                let kind = ExprKind::Call {
                    callee: Box::new(ctor),
                    args: args.to_vec(),
                };
                Some(self.infer(&Expr {
                    kind,
                    span: call.span,
                }))
            }
            _ => None,
        }
    }

    /// The key of the struct `name` with `arity` type arguments, when it shares
    /// its name with one of another arity (R02.11.4), else `name`.
    pub(super) fn arity_name(&mut self, name: &str, arity: usize) -> String {
        let key = crate::prelude::arity_key(name, arity);
        match self.global(&key).as_deref() {
            Some(Def::Struct { .. }) => key,
            _ => name.to_string(),
        }
    }

    /// `Prefix(a)` on `a: [R, C]T` builds the family's 2-D shape.
    pub(super) fn grid_shape_call(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        let shape = crate::prelude::arity_key(name, 3);
        if self.local(name).is_some()
            || self.global(&shape).is_none()
            || args.len() != 1
            || args[0].name.is_some()
        {
            return None;
        }
        let Ty::Array { index, elem } = self.infer_quietly(&args[0].value) else {
            return None;
        };
        if index.len() != 2 || !index.iter().all(|d| matches!(d, Ty::Pos(_))) {
            return None;
        }
        self.infer(&args[0].value);
        let has_new = !matches!(
            self.global(&shape).as_deref(),
            Some(crate::def::Def::Struct { ctor: None, .. })
        );
        if !has_new {
            let at = self.report(
                callee.span,
                cpl_diag::msgs::check::NO_METHOD_ON,
                &[&"struct", &name, &"new"],
            );
            if let Some(diag) = at {
                diag.help = Some(format!(
                    "`{name}(a)` on a 2-D array calls `new(a)` of the `{name}` with three type parameters"
                ));
            }
            return Some(Ty::Error);
        }
        Some(Ty::Named {
            name: shape.to_string(),
            args: vec![index[0].clone(), index[1].clone(), *elem],
        })
    }

    /// `Prefix(a)` on a `List<T>` or a `[0..n]T` builds the family's shape
    /// whose `new` takes a `List<T>`, as `new(a: List<T>)` states.
    pub(super) fn list_shape_call(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        let shape = crate::prelude::arity_key(name, 1);
        let takes_list = self.global(&shape).is_some_and(|d| {
            matches!(&*d, Def::Struct { ctor: Some(ps), .. }
                if matches!(ps.first().map(|p| &p.ty), Some(Ty::Named { name, .. }) if name == "List"))
        });
        if self.local(name).is_some() || !takes_list || args.len() != 1 || args[0].name.is_some() {
            return None;
        }
        let elem = match self.infer_quietly(&args[0].value) {
            Ty::Array { index, elem } if index.len() == 1 && index[0].is_int() => *elem,
            Ty::Named { name, mut args } if name == "List" && args.len() == 1 => args.remove(0),
            _ => return None,
        };
        self.infer(&args[0].value);
        Some(Ty::Named {
            name: shape,
            args: vec![elem],
        })
    }

    /// `let g: [Row, Col]Tile = read()` reads each `Tile` as text, so the
    /// enum says how its variants are written: nothing is read as a
    /// number or by guessing.
    pub(crate) fn readable(&mut self, declared: &Ty, init: &Expr) {
        let reads = matches!(
            &init.kind,
            ExprKind::Call { callee, args }
                if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read")
        );
        if !reads {
            return;
        }
        let mut todo = vec![declared];
        while let Some(ty) = todo.pop() {
            match ty {
                Ty::Array { elem, .. } | Ty::Optional(elem) => todo.push(elem),
                Ty::Tuple(items) => todo.extend(items),
                Ty::Bool => {
                    self.not_a_token_type(init.span);
                    return;
                }
                Ty::Named { name, args } => {
                    todo.extend(args);
                    let found = self.type_def(name);
                    let Some(Def::Enum {
                        variants,
                        tagged: false,
                        ..
                    }) = found.as_deref()
                    else {
                        continue;
                    };
                    if variants.iter().any(|(_, fields)| !fields.is_empty()) {
                        continue;
                    }
                    let tags: Vec<String> = variants
                        .iter()
                        .map(|(v, _)| format!("@tag({v:?}) {v}"))
                        .collect();
                    if let Some(diag) = self.report(
                        init.span,
                        msgs::check::TYPE_MISMATCH_READING_A_NEEDS_TO,
                        &[&name],
                    ) {
                        diag.help = Some(format!(
                            "write each variant's token: `{}`",
                            tags.join(" | ")
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    /// `Bool` is not an input token: a judge spells yes and no its own way
    /// (R10.2.3).
    pub(crate) fn not_a_token_type(&mut self, span: Span) {
        if let Some(diag) = self.report(span, msgs::check::NOT_A_TOKEN_TYPE_READ_CAN_T_READ, &[]) {
            diag.help = Some("read a `Str` and compare it: `read<Str>() == \"YES\"`".to_string());
        }
    }

    /// `vis.map(yes)`: a fn value given where a fn of `given` parameters
    /// is expected takes only those; every one after them needs a default
    /// (R06.1.8).
    /// Gives what the fn returns.
    pub(crate) fn fn_value_takes(&mut self, method: &str, arg: &Expr, given: usize) -> Ty {
        let Ty::Fn { params, ret } = self.infer_quietly(arg) else {
            return Ty::Unknown;
        };
        let Some(missing) = params
            .iter()
            .skip(given)
            .find(|p| !p.has_default && !p.variadic)
        else {
            return *ret;
        };
        let noun = if given == 1 { "argument" } else { "arguments" };
        if let Some(diag) = self.report(
            arg.span,
            msgs::check::TYPE_MISMATCH_CALLS_WITH_BUT_ITS,
            &[&method, &self.quote(arg.span), &given, &noun, &missing.name],
        ) {
            diag.help = Some(format!(
                "give `{}` a default, or pass a lambda that supplies it",
                missing.name
            ));
        }
        *ret
    }

    /// `pairs.map((a, b) => ..)`: a lambda of several parameters takes the
    /// fields of a tuple element, so it has one per field.
    pub(crate) fn lambda_takes_elem(&mut self, params: usize, elem: &Ty, span: Span) -> Ty {
        let fields = elem.param_fields().map_or(1, |f| f.len());
        if params != fields {
            self.report(
                span,
                msgs::check::WRONG_ARG_COUNT_LAMBDA_TAKES_PARAMETERS_BUT,
                &[&params, &elem, &fields],
            );
        }
        Ty::Unknown
    }

    /// `rows.sort(by: (a, b) => ..)`: the call passes the lambda one
    /// element, so a key of two parameters is a comparator it can't take.
    pub(crate) fn lambda_arity_mismatch(
        &mut self,
        params: &[cpl_syntax::ast::Pattern],
        given: &[Ty],
    ) {
        let (Some(first), Some(last)) = (params.first(), params.last()) else {
            return;
        };
        let passed = given.len();
        let noun = if passed == 1 { "argument" } else { "arguments" };
        let what = given
            .iter()
            .map(|t| format!("`{t}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let help = if passed == 1 {
            "take one parameter".to_string()
        } else {
            format!("take {passed} parameters")
        };
        if let Some(diag) = self.report(
            first.span.to(last.span),
            msgs::check::WRONG_ARG_COUNT_THIS_LAMBDA_TAKES_PARAMETERS,
            &[&params.len(), &passed, &noun, &what],
        ) {
            diag.help = Some(help);
        }
    }

    /// `c.bump()` where `bump` takes `mut self` changes `c`, so `c` is a
    /// `var`, a `mut` parameter, or the `mut self` of the calling method.
    /// Without this the call works on a copy and the change is lost.
    /// `p.swap()` on a type with its own non-`mut` `swap`, or `s.add(x, y)`
    /// on an operation-value field `add`, is not the built-in mutator of
    /// that name (R05.4.2).
    pub(crate) fn own_non_mutator(&mut self, callee: &Expr) -> bool {
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let Ty::Named {
            name: type_name, ..
        } = self.base_ty(base)
        else {
            return false;
        };
        self.members_of(&type_name).is_some_and(|m| {
            m.fields.iter().any(|f| f.name == name.text)
                || m.members.iter().any(|(n, _)| *n == name.text)
                    && !m.mutators.contains(&name.text)
        })
    }

    /// `base.name(args)` changes its receiver: a built-in row does, or a fn
    /// that takes `mut self`, std's or a program's.
    pub(crate) fn call_mutator(&mut self, base: &Ty, name: &str, args: &[Arg]) -> bool {
        !base.is_vague()
            && !self.declares_field(base, name)
            && (mutating_call(name, args) || self.changes_self(base, name))
    }

    /// `base`'s type declares a field `name`: `base.name(..)` calls it.
    pub(crate) fn declares_field(&mut self, base: &Ty, name: &str) -> bool {
        match base {
            Ty::Record(fields) => fields.iter().any(|(n, _)| n == name),
            Ty::Named { name: owner, .. } => {
                (self.members_of(owner)).is_some_and(|m| m.fields.iter().any(|f| f.name == name))
            }
            _ => false,
        }
    }
}

/// Methods that change their receiver.
const MUTATORS: &[&str] = &[
    "push", "pop", "clear", "insert", "remove", "take", "swap", "swap_idx", "set_at", "add",
    "resize", "truncate", "reserve", "fill",
];
/// `List` methods that change length and therefore cannot run on a fixed-length slice view.
pub(super) const LENGTH_CHANGING_MUTATORS: &[&str] = &[
    "push", "pop", "clear", "insert", "remove", "take", "resize", "truncate", "reserve", "extend",
];
/// `xs.take()` empties `xs`; `xs.take(k)` is a view of its first `k`.
pub(super) fn mutating_call(name: &str, args: &[Arg]) -> bool {
    MUTATORS.contains(&name) && !(name == "take" && !args.is_empty())
}

/// An optional has no text for `None`, so it prints once that is said
/// (R10.3.7): unwrapped, given a default, or, in a sequence, `none:`.
fn optional_print_help(ty: &Ty, snippet: &str) -> Option<String> {
    let operand = if snippet.contains(' ') {
        format!("({snippet})")
    } else {
        snippet.to_string()
    };
    match ty {
        Ty::Optional(_) => Some(format!(
            "`None` has no text: write `{operand}!` to unwrap it, or `{operand} ?? d` with a default `d`"
        )),
        // R10.3.8: a hash table has no order to print in.
        Ty::Named { name, .. } if matches!(name.as_str(), "HashMap" | "HashSet") => {
            let keys = if name == "HashMap" { ".keys" } else { "" };
            Some(format!(
                "a `{name}` has no defined order: print the order you want, `print({operand}{keys}.sorted())`"
            ))
        }
        Ty::Named { name, args } if name == "List" && matches!(args.as_slice(), [Ty::Optional(_)]) => {
            Some(format!(
                "`None` has no text: write `print({snippet}, none: d)` to print each `None` as `d`"
            ))
        }
        _ => None,
    }
}

impl Checker<'_> {
    pub(crate) fn implements_print(&mut self, ty: &Ty, allow_none: bool) -> bool {
        let mut seen = Vec::new();
        self.derives_print(ty, allow_none, &mut seen)
    }

    /// R10.3.2: what `print` shows, and each piece of an interpolation, is
    /// `Show`; a `Bool`, an optional or a hash table is not.
    pub(crate) fn require_show(&mut self, expr: &Expr, ty: &Ty, allow_none: bool, piece: bool) {
        // A piece of an unbounded `T` is shown per instance, as before.
        if self.implements_print(ty, allow_none) || (piece && matches!(ty, Ty::Param(_))) {
            return;
        }
        let snippet = self.quote(expr.span);
        let message = match ty {
            Ty::Bool if piece => {
                cpl_diag::msgs::check::HAS_TYPE_BOOL_WHICH.at(&[&snippet, &snippet])
            }
            Ty::Bool => cpl_diag::msgs::check::HAS_TYPE_BOOL_WHICH_2.at(&[&snippet, &snippet]),
            _ => cpl_diag::msgs::check::HAS_TYPE_WHICH_DOES.at(&[&snippet, &ty]),
        };
        let help = optional_print_help(ty, &snippet);
        let span = expr.span;
        let text = self.src[span.start as usize..span.end as usize].to_string();
        if let Some(diag) = self.report_said(span, message) {
            diag.help = help;
            if matches!(ty, Ty::Bool) {
                diag.fix(format!("write `yes({text})`"), span, format!("yes({text})"));
            }
            // `print(xs.pop())`: unwrap it, as the help says first.
            if matches!(ty, Ty::Optional(_)) {
                let (open, close) = if snippet.contains(' ') {
                    ("(", ")!")
                } else {
                    ("", "!")
                };
                diag.fixes.push(cpl_diag::Fix {
                    title: "unwrap with `!`".into(),
                    edits: vec![
                        (Span::point(span.start), open.into()),
                        (Span::point(span.end), close.into()),
                    ],
                });
            }
        }
    }

    /// R01.4.2: an integer literal takes `F64` where one is wanted, when the
    /// float holds it exactly. `None` leaves an `Int` literal to the domain rules.
    pub(super) fn literal_float(&mut self, at: &Expr, want: &Ty) -> Option<bool> {
        let float = match want {
            Ty::Optional(inner) => matches!(**inner, Ty::Float),
            want => matches!(want, Ty::Float),
        };
        let v = match &at.kind {
            ExprKind::Int(_) if !float => return None,
            ExprKind::Int(v) => *v,
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } if float => match expr.kind {
                ExprKind::Int(v) => -v,
                _ => return Some(false),
            },
            _ => return Some(false),
        };
        if v.unsigned_abs() > 1 << 53 && (v as f64) as i128 != v {
            let text = self.quote(at.span);
            let message = cpl_diag::msgs::check::NOT_EXACTLY_F64_WRITE.at(&[&text, &text]);
            if let Some(diag) = self.report_said(at.span, message) {
                diag.fix(
                    format!("write `F64({text})`"),
                    at.span,
                    format!("F64({text})"),
                );
            }
        }
        Some(true)
    }
}
