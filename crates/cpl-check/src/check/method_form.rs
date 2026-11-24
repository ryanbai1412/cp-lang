//! `xs.f(args)` on a library `fn f(self: S, ...)` checks as `f(xs, args)`.
//! R11.3.4: every top-level `fn f(self: ..)` in reach, the file's own and
//! each module's, is a candidate, and the call takes the one whose `self`
//! its receiver fits. The choice is the `Resolved` at the method's name,
//! which lowering reads.

use super::*;
use cpl_syntax::ast::{self, Arg, Expr, ExprKind};
use std::sync::Arc;

/// Which candidate a receiver picks.
enum Pick {
    One(String, Arc<Def>),
    Ambiguous(Vec<String>),
    /// The receiver's type is not known well enough to say.
    Unsettled,
    /// The receiver's type is known and no candidate's `self` takes it.
    NoFit,
}

/// The members an optional has of its own, whatever std declares.
pub(super) const OPTIONAL_MEMBERS: &[&str] = &["map", "filter"];

fn takes_self(def: &Def) -> bool {
    matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
        if params.first().is_some_and(|p| p.name == "self" && !p.named_only))
}

impl Checker<'_> {
    /// Every top-level `fn name(self: ..)` in reach, by the name it is
    /// imported under, one per module.
    pub(super) fn method_candidates(&mut self, name: &str) -> Vec<(String, Arc<Def>)> {
        let mut out: Vec<(String, Arc<Def>, Option<String>)> = Vec::new();
        let keys = std::iter::once(name.to_string()).chain(self.env.method_keys(name));
        for key in keys {
            let Some(def) = self.global(&key).filter(|d| takes_self(d)) else {
                continue;
            };
            let origin = if self.local_defs.contains_key(&key) {
                None
            } else {
                self.env.origin(&key)
            };
            if !out.iter().any(|(_, _, o)| *o == origin) {
                out.push((key, def, origin));
            }
        }
        out.into_iter().map(|(k, d, _)| (k, d)).collect()
    }

    /// Whether `base` can be the `self` of the fn `def`: it fits the
    /// parameter's type, and a bare type parameter's bounds.
    fn self_fits(&mut self, base: &Ty, def: &Def) -> bool {
        let Def::Fn {
            ty: Ty::Fn { params, .. },
            bounds,
            ..
        } = def
        else {
            return false;
        };
        let Some(first) = params.first() else {
            return false;
        };
        if let Ty::Param(g) = &first.ty {
            let traits: Vec<String> = (bounds.iter())
                .filter(|(p, _)| p == g)
                .flat_map(|(_, t)| t.iter().cloned())
                .collect();
            return traits.iter().all(|t| self.implements_trait(base, t));
        }
        let same_shape = match (&first.ty, base) {
            (Ty::Record(takes), Ty::Record(on)) => {
                takes.len() == on.len() && takes.iter().zip(on).all(|(a, b)| a.0 == b.0)
            }
            (Ty::Tuple(takes), Ty::Tuple(on)) => takes.len() == on.len(),
            (Ty::Tuple(takes), Ty::Record(on)) => takes.len() == on.len(),
            (Ty::Tuple(takes), Ty::Pos(d)) => {
                (self.product_factors(d)).is_some_and(|parts| parts.len() == takes.len())
            }
            (Ty::Named { name: takes, .. }, Ty::Named { name: on, .. }) => takes == on,
            _ => false,
        };
        // `self: [D]T` takes `List<Int>` with `D` and `T` bound to its parts.
        let mut bindings = crate::ty::Bindings::default();
        first.ty.unify(base, &mut bindings);
        let takes = first.ty.subst(&bindings);
        same_shape || (!matches!(first.ty, Ty::Named { .. }) && self.fits(base, &takes))
    }

    /// The candidate `base` fits; of several, one whose `self` is a type of
    /// its own wins over one whose `self` is any type a bound allows.
    fn pick_method(&mut self, base: &Ty, name: &str) -> Pick {
        self.pick_method_for(base, name, None)
    }

    /// As `pick_method`, also dropping a candidate whose parameters can't
    /// take `args`' shapes: `s.split(sep: c => c == ' ')` on a `Str` is the
    /// predicate split, not `split(self: Str, sep: Char)`.
    fn pick_method_for(&mut self, base: &Ty, name: &str, args: Option<&[Arg]>) -> Pick {
        if base.is_vague() || matches!(base, Ty::Param(_) | Ty::Error) {
            return Pick::Unsettled;
        }
        let mut fit: Vec<(String, Arc<Def>, bool)> = Vec::new();
        for (key, def) in self.method_candidates(name) {
            if self.self_fits(base, &def) && args.is_none_or(|args| takes_shapes(&def, args)) {
                let own_type = !matches!(&*def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
                    if matches!(params[0].ty, Ty::Param(_)));
                fit.push((key, def, own_type));
            }
        }
        if fit.iter().any(|f| f.2) {
            fit.retain(|f| f.2);
        }
        // The file's own fn of the name stands in front of the library's.
        if fit.iter().any(|f| self.env.origin(&f.0).is_none()) {
            fit.retain(|f| self.env.origin(&f.0).is_none());
        }
        match fit.len() {
            0 => Pick::NoFit,
            1 => {
                let (key, def, _) = fit.pop().unwrap();
                Pick::One(key, def)
            }
            _ => Pick::Ambiguous(fit.into_iter().map(|f| f.0).collect()),
        }
    }

    /// The fn `base.name` calls: the candidate it fits, else the only one.
    pub(super) fn method_def(&mut self, base: &Ty, name: &str) -> Option<Arc<Def>> {
        match self.pick_method(base, name) {
            Pick::One(_, def) => Some(def),
            _ => self.method_candidates(name).into_iter().next().map(|c| c.1),
        }
    }

    /// The fn a call at `name` was resolved to, else the only candidate.
    pub(super) fn chosen_method(&mut self, name: &ast::Name) -> Option<Arc<Def>> {
        let at = (self.resolved.iter().rev())
            .chain(&self.chosen_methods)
            .find(|r| r.0 == name.span);
        let key = at.map(|(_, r)| match r.module.as_str() {
            "" => name.text.clone(),
            module => crate::prelude::method_key(&name.text, module),
        });
        match key
            .and_then(|k: String| self.global(&k))
            .filter(|d| takes_self(d))
        {
            Some(def) => Some(def),
            None => self
                .method_candidates(&name.text)
                .into_iter()
                .next()
                .map(|c| c.1),
        }
    }

    /// The top-level fn `name` whose `self` the struct, record or tuple
    /// `base` fits, as the method it is: the fn without that parameter.
    pub(super) fn own_method(&mut self, base: &Ty, name: &str) -> Option<Ty> {
        self.own_method_key(base, name).map(|(_, ty)| ty)
    }

    /// `own_method`, recording the choice at `name`.
    pub(super) fn own_method_at(&mut self, base: &Ty, name: &ast::Name) -> Option<Ty> {
        let (key, ty) = self.own_method_key(base, &name.text)?;
        self.record_method(name, &key);
        Some(ty)
    }

    fn own_method_key(&mut self, base: &Ty, name: &str) -> Option<(String, Ty)> {
        if !matches!(base, Ty::Named { .. } | Ty::Record(_) | Ty::Tuple(_)) {
            return None;
        }
        let Pick::One(key, def) = self.pick_method(base, name) else {
            return None;
        };
        let Def::Fn {
            ty: Ty::Fn { params, ret },
            ..
        } = &*def
        else {
            return None;
        };
        let ty = Ty::Fn {
            params: params[1..].to_vec(),
            ret: ret.clone(),
        };
        Some((key, ty))
    }

    /// Records which candidate `base.name` calls, when its receiver says.
    pub(super) fn note_method(&mut self, base: &Ty, name: &ast::Name) {
        // Beside a built-in member of the name, the call's arguments pick
        // (`method_form_call`): `xs.fold(+)` is the built-in's.
        let builtin = super::call::catalog_owner(base)
            .is_some_and(|owner| crate::builtins::builtin_member(owner, &name.text).is_some());
        if builtin {
            return;
        }
        if let Pick::One(key, _) = self.pick_method(base, &name.text) {
            self.record_method(name, &key);
        }
    }

    /// The `Resolved` at a method's name: the module of the fn it calls.
    fn record_method(&mut self, name: &ast::Name, key: &str) {
        let module = if self.local_defs.contains_key(key) {
            None
        } else {
            self.env.origin(key)
        };
        let resolved = Resolved {
            module: module.unwrap_or_default(),
            name: name.text.clone(),
            owner: None,
        };
        self.chosen_methods.retain(|r| r.0 != name.span);
        self.chosen_methods.push((name.span, resolved));
    }

    /// The `Resolved` facts with each method choice not already among them.
    pub(super) fn with_chosen_methods(&mut self) -> Vec<(Span, Resolved)> {
        let mut resolved = std::mem::take(&mut self.resolved);
        for (span, chosen) in std::mem::take(&mut self.chosen_methods) {
            if !resolved.iter().any(|r| r.0 == span) {
                resolved.push((span, chosen));
            }
        }
        resolved
    }
}

impl<'a> Checker<'a> {
    /// The type of `base.name(args)` when `name` is a top-level fn taking
    /// `self`, found by checking the free form `name(base, args)`, when that
    /// checks cleanly to a plain value type. Its resolution, and its hover
    /// where the member has none, are the free form's in any case.
    pub(super) fn method_form_call(
        &mut self,
        call: &Expr,
        callee: &Expr,
        args: &[Arg],
    ) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let module = matches!(&base.kind, ExprKind::Name(m) if self.local(m).is_none() && self.env.is_module(m));
        if module || self.local(&name.text).is_some() {
            return None;
        }
        // `Sub<V>.find(p)`: a type with `impl Index` is the `Seq` of its values (R04.2.20).
        if self.quietly(|c| c.library_domain_walk(base)).is_some() {
            return None;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        if self.declares_member(&base_ty, &name.text) {
            return None;
        }
        let pick = match self.pick_method_for(&base_ty, &name.text, Some(args)) {
            Pick::NoFit => self.pick_method(&base_ty, &name.text),
            pick => pick,
        };
        let key = match &pick {
            Pick::One(key, _) => key.clone(),
            Pick::Ambiguous(keys) => {
                self.ambiguous_candidates(base, name, keys);
                return Some(Ty::Error);
            }
            Pick::Unsettled | Pick::NoFit => {
                self.method_candidates(&name.text).into_iter().next()?.0
            }
        };
        let own_member = match &base_ty {
            Ty::Optional(_) => OPTIONAL_MEMBERS.contains(&name.text.as_str()),
            Ty::Domain(_) => crate::builtins::DOMAIN_MEMBERS.contains(&name.text.as_str()),
            Ty::Pos(_) => crate::builtins::POS_MEMBERS.contains(&name.text.as_str()),
            _ => self.catalog_row(base, &name.text).is_some(),
        };
        // A view has its source's members, `xs.rev.nth(1)` (R07.4.5).
        let vague = base_ty.has_vague()
            || matches!(base_ty, Ty::Fn { .. })
            || base_ty.view_source().is_some();
        let fits = own_member || vague || !matches!(pick, Pick::NoFit);
        let mut free_args = vec![Arg {
            name: None,
            mutable: false,
            value: (**base).clone(),
        }];
        free_args.extend(args.iter().cloned());
        let free = Expr {
            kind: ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Name(key.clone()),
                    span: name.span,
                }),
                args: free_args,
            },
            span: call.span,
        };
        let (ty, hover, resolved, errors) = self.quietly(|c| {
            let ty = c.infer(&free);
            let errors: Vec<_> = (c.sink.diagnostics().iter())
                .filter(|d| d.severity == cpl_diag::Severity::Error)
                .cloned()
                .collect();
            let ty = if errors.is_empty() { ty } else { Ty::Unknown };
            let hover = c.hovers.iter().rev().find(|h| h.span == name.span).cloned();
            let resolved = c.resolved.iter().rev().find(|r| r.0 == name.span).cloned();
            (ty, hover, resolved, errors)
        });
        // No candidate takes the receiver for want of a bound: the free
        // form's E0606 is the method's, `?List<Int>` lacking the `Seq` that
        // `sum` requires. A receiver of another shape has a member of its
        // own, `(0..10).step(3)`, that the free form says nothing of.
        let free_failed = !errors.is_empty();
        let missing: Vec<_> = (errors.into_iter())
            .filter(|d| d.code == codes::MISSING_IMPL)
            .collect();
        // Only where the checker knows every trait the receiver has: an
        // optional, or a type the program or library declares. A built-in
        // container's members and traits are lowering's.
        let known = match &base_ty {
            Ty::Optional(_) => true,
            Ty::Named { name, .. } => (self.type_def(name))
                .is_some_and(|d| matches!(&*d, Def::Struct { .. } | Def::Enum { .. })),
            _ => false,
        };
        if !fits && known && !missing.is_empty() {
            for mut diag in missing {
                diag.message = diag
                    .message
                    .replace(&format!("`{key}`"), &format!("`{}`", name.text));
                self.sink.push(diag);
            }
            return Some(Ty::Error);
        }
        // A receiver no candidate takes calls none of them: an own member
        // such as `?T.map`, or a fn lowering finds by other means.
        if matches!(pick, Pick::NoFit) && free_failed {
            // Lowering may still find the fn, `(V, V).all(..)` over a
            // product: the name keeps the hover of the fn it would call.
            if let Some(mut hover) =
                hover.filter(|_| !own_member && !self.hovers.iter().any(|h| h.span == name.span))
            {
                hover.name = name.text.clone();
                self.hovers.push(hover);
            }
            return None;
        }
        // Arguments the fn doesn't take are the receiver's own member's,
        // `xs.fold(+)` beside `xs.fold(f, init: 0)`.
        if own_member && free_failed {
            self.chosen_methods.retain(|r| r.0 != name.span);
            return None;
        }
        if let Some(mut hover) = hover.filter(|_| !self.hovers.iter().any(|h| h.span == name.span))
        {
            hover.name = name.text.clone();
            self.hovers.push(hover);
        }
        if let Some((span, mut resolved)) = resolved {
            resolved.name = name.text.clone();
            self.resolved.retain(|r| r.0 != span);
            self.resolved.push((span, resolved.clone()));
            // Kept past a probe that runs this call quietly, for lowering.
            self.chosen_methods.retain(|r| r.0 != span);
            self.chosen_methods.push((span, resolved));
        }
        if plain_value(&ty) {
            // The probe's facts went with it, and the caller checks nothing
            // more: check the call for real so its arguments keep their
            // hovers, refs and warnings (a lambda's body).
            let (hovers, refs) = (self.hovers.len(), self.resolved.len());
            self.infer(&free);
            let mut at = 0;
            self.hovers.retain(|h| {
                at += 1;
                at <= hovers || h.span != name.span
            });
            let mut at = 0;
            self.resolved.retain(|r| {
                at += 1;
                at <= refs || r.0 != name.span
            });
            return Some(ty);
        }
        self.shapes.push((call.span, ty));
        None
    }
}

impl Checker<'_> {
    /// Whether the type `base` declares a fn `name` of its own, in its body,
    /// an impl or a trait it implements: that member, not a top-level fn
    /// of the name, is what `base.name` calls.
    fn declares_member(&mut self, base: &Ty, name: &str) -> bool {
        let Ty::Named { name: owner, .. } = base else {
            return false;
        };
        if let Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) =
            self.type_def(owner).as_deref()
        {
            if methods.iter().any(|(m, _)| m == name) {
                return true;
            }
        }
        for (trait_name, def) in self.env.impls(owner) {
            if matches!(&*def, Def::Impl { methods, .. } if methods.iter().any(|(m, _)| m == name))
            {
                return true;
            }
            let trait_def = (trait_name.as_deref())
                .and_then(|t| t.split('<').next())
                .and_then(|t| self.type_def(t));
            if matches!(trait_def.as_deref(), Some(Def::Trait { methods, .. })
                if methods.iter().any(|m| m.name == name))
            {
                return true;
            }
        }
        false
    }

    /// Whether `base.name(args)` gives arguments its built-in member's row
    /// doesn't take while a library fn fits the receiver: then the call is
    /// the library's, `xs.fold(f, init: 0)` beside the built-in `xs.fold(+)`.
    pub(super) fn past_builtin_row(&mut self, callee: &Expr, args: &[Arg]) -> bool {
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let Some((row, _)) = self.catalog_row(base, &name.text) else {
            return false;
        };
        let base_ty = self.quietly(|c| c.infer(base));
        if !matches!(self.pick_method(&base_ty, &name.text), Pick::One(..)) {
            return false;
        }
        let params = row_params(row.params);
        let takes_fn = |ty: &str| ty.contains("->") || (ty.len() == 1 && ty != "T");
        let positional = args.iter().filter(|a| a.name.is_none()).count();
        positional > params.len()
            || (args.iter()).any(|a| {
                (a.name.as_ref()).is_some_and(|n| !params.iter().any(|(p, _)| *p == n.text))
            })
            || (args.iter().zip(&params)).any(|(a, (_, ty))| {
                matches!(a.value.kind, ExprKind::Lambda { .. }) && !takes_fn(ty)
            })
    }

    /// A named type a `for` walks is a `Seq`: `SortedRange<K>`, or a
    /// struct whose `each` yields its elements.
    pub(super) fn walks_by_each(&mut self, ty: &Ty) -> bool {
        matches!(ty, Ty::Named { .. }) && !matches!(self.element_ty(ty), Ty::Unknown | Ty::Error)
    }

    /// The type of a call, index or other compound expression, for tools.
    /// An untyped call keeps the library type `method_form_call` gave it.
    pub(super) fn record_shape(&mut self, expr: &Expr, ty: &Ty) {
        let compound = matches!(
            expr.kind,
            ExprKind::Call { .. }
                | ExprKind::Index { .. }
                | ExprKind::Paren(_)
                | ExprKind::Range { .. }
                | ExprKind::Unwrap(_)
                | ExprKind::Try(_)
        );
        let kept = *ty == Ty::Unknown && self.shapes.last().is_some_and(|s| s.0 == expr.span);
        if compound && !kept {
            self.shapes.push((expr.span, ty.clone()));
        }
    }
}

/// A type the checker models in full: no library type (`Slice<S>`, a
/// generator) whose members and traits only lowering knows, nothing vague.
fn plain_value(ty: &Ty) -> bool {
    match ty {
        Ty::Optional(inner) => plain_value(inner),
        Ty::Tuple(parts) => parts.iter().all(plain_value),
        Ty::Record(fields) => fields.iter().all(|(_, t)| plain_value(t)),
        Ty::Array { index, elem } => index.iter().all(plain_value) && plain_value(elem),
        Ty::Named { .. }
        | Ty::TypeName(_)
        | Ty::Fn { .. }
        | Ty::Param(_)
        | Ty::RowWithout(..)
        | Ty::Unknown
        | Ty::Error => false,
        _ => true,
    }
}

impl Checker<'_> {
    /// E0701: two candidates fit the receiver equally well.
    fn ambiguous_candidates(&mut self, base: &Expr, name: &ast::Name, keys: &[String]) {
        let mut modules: Vec<String> = Vec::new();
        for key in keys {
            let origin = self.env.origin(key).unwrap_or_else(|| "this file".into());
            modules.push(format!("`{origin}`"));
        }
        let shown = self.quote(base.span);
        let method = &name.text;
        let message = format!(
            "`{shown}.{method}` is ambiguous: {} each give its type a `{method}`",
            modules.join(" and ")
        );
        if let Some(diag) = self.error(codes::AMBIGUOUS_METHOD, name.span, message) {
            diag.help = Some(format!(
                "import the one you mean and call it as `{method}({shown}, ..)`"
            ));
        }
    }
}

/// A catalog row's parameters, `r: Range<T>, by: T -> K`, as names and types.
fn row_params(params: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in params.char_indices().chain([(params.len(), ',')]) {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                if let Some((n, t)) = params[start..i].split_once(':') {
                    out.push((n.trim(), t.trim()));
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// Whether each argument has the shape its parameter takes: a lambda goes
/// where a fn is taken, and a literal where a value is.
fn takes_shapes(def: &Def, args: &[Arg]) -> bool {
    let Def::Fn {
        ty: Ty::Fn { params, .. },
        ..
    } = def
    else {
        return true;
    };
    let after_self = params.get(1..).unwrap_or_default();
    let mut positional = after_self.iter().filter(|p| !p.named_only);
    args.iter().all(|arg| {
        let param = match &arg.name {
            Some(n) => after_self.iter().find(|p| p.name == n.text),
            None => positional.next(),
        };
        let Some(param) = param else {
            return true;
        };
        let lambda = matches!(arg.value.kind, ExprKind::Lambda { .. });
        let literal = matches!(
            arg.value.kind,
            ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Str(_)
                | ExprKind::Char(_)
                | ExprKind::Bool(_)
        );
        match &param.ty {
            Ty::Fn { .. } => !literal,
            Ty::Param(_) | Ty::Unknown => true,
            _ => !lambda,
        }
    })
}
