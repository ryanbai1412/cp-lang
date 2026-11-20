//! Calls: fns, constructors, and conversions.

use super::expr::{list_names, plural};
use super::*;
use crate::ty::{Bindings, Param};
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl<'a> Checker<'a> {
    pub(crate) fn call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Ty {
        let callee_ty = self.infer(callee);
        let receiver = self.receiver(call, callee);
        let mut ty = self.call_kind(call, callee, callee_ty.clone(), args);
        if let Some(known) = self.prelude_call(call, callee, args) {
            ty = known;
        }
        self.lent_together(args);
        self.call_effects(call, callee, args);
        self.no_escape_args(callee, &callee_ty, args);
        self.mutates_view(call, callee);
        receiver.unwrap_or(ty)
    }

    /// What the checker knows about a prelude fn it otherwise takes on
    /// trust: `span(a, b)` covers two ends of one domain, and the `max` or
    /// `min` of a slice is optional, since `a[l..l]` is empty.
    fn prelude_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        if self.local(name).is_some() || self.env.global(name).is_some() {
            return None;
        }
        let tys: Vec<Ty> = args
            .iter()
            .map(|arg| self.quietly(|c| c.infer(&arg.value)))
            .collect();
        match (name.as_str(), tys.as_slice()) {
            ("span", [Ty::Pos(a), Ty::Pos(b)]) => {
                let (a, b) = (
                    a.strip_suffix(".bounds").unwrap_or(a),
                    b.strip_suffix(".bounds").unwrap_or(b),
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
            ("max" | "min", [Ty::Array { elem, .. }]) if is_slice(&args[0].value) => {
                Some(Ty::Optional(Box::new(elem.widen())))
            }
            _ => None,
        }
    }

    /// What a method call means beyond its signature: `g.row(r)` lends a
    /// row, `v.l(k)` is a boundary, `b.at(i)` is a projection of `b`.
    /// Returns the call's type when the receiver decides it.
    fn receiver(&mut self, call: &Expr, callee: &Expr) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        match self.quietly(|c| c.infer(base)) {
            Ty::Array { index, elem } if name.text == "row" && index.len() > 1 => {
                self.note_view(call, "row");
                Some(Ty::Array {
                    index: index[1..].to_vec(),
                    elem,
                })
            }
            // `v.l(k)`, `v.r(k)`: the far end of a window of `k` at `v`.
            Ty::Pos(domain)
                if matches!(name.text.as_str(), "l" | "r") && !domain.ends_with(".bounds") =>
            {
                Some(Ty::Pos(format!("{domain}.bounds")))
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
                self.settle(&ret.subst(&bindings))
            }
            Ty::Named {
                name,
                args: type_args,
            } => self.construct(&name, type_args, &callee_text, args, call),
            Ty::TypeName(name) => self.construct(&name, Vec::new(), &callee_text, args, call),
            // `V(i)`: a number checked into the domain.
            Ty::Domain(name) => {
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

    /// Checks arguments whose parameters are not known.
    fn plain_args(&mut self, args: &[Arg]) {
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
        if let Some(def) = self.env.type_def(name) {
            return match &*def {
                Def::Struct {
                    generics,
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
                    let mut bindings = Bindings::new();
                    if type_args.len() == generics.len() {
                        bindings.extend(generics.iter().cloned().zip(type_args.iter().cloned()));
                    }
                    self.args(callee_text, verb, params, args, call, &mut bindings);
                    // Type arguments left out are read off the values.
                    let type_args = if type_args.is_empty()
                        && generics.iter().all(|g| bindings.contains_key(g))
                    {
                        generics.iter().map(|g| bindings[g].clone()).collect()
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
        for arg in args {
            let arg_ty = self.infer(&arg.value);
            let param = match &arg.name {
                Some(name) if labeled => match params.iter().find(|p| p.name == name.text) {
                    Some(param) => Some(param),
                    None => {
                        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                        let suggestion = closest(&name.text, names.iter().map(String::as_str));
                        let what = if verb == "has" { "field" } else { "parameter" };
                        let message = format!("`{callee}` has no {what} named `{}`", name.text);
                        if let Some(diag) = self.error(codes::UNKNOWN_ARG_NAME, name.span, message)
                        {
                            diag.help = Some(match suggestion {
                                Some(s) => format!("did you mean `{s}`?"),
                                None => format!("its {what}s are {}", list_names(&names)),
                            });
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
            let Some(param) = param else {
                if arg.mutable {
                    self.require_mutable(&arg.value, "passed as `mut`");
                }
                continue;
            };
            supplied.push(&param.name);
            let arg_ty = self.narrowed(&arg_ty, &param.ty);
            param.ty.unify(&arg_ty, bindings);
            let want = param.ty.subst(bindings);
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
            if param.mutable && !arg.mutable {
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
