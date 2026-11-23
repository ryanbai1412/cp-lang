//! Prelude fns the checker knows more about than their signatures say.

use super::*;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl<'a> Checker<'a> {
    /// What the checker knows about a prelude fn it otherwise takes on
    /// trust: `span(a, b)` covers two ends of one domain, and the `max` or
    /// `min` of a slice is optional, since `a[l..l]` is empty.
    pub(super) fn prelude_call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        if self.local(name).is_some() || (self.global(name).is_some() && name != "zip") {
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
            ("print", _) => {
                let allow_none = args
                    .iter()
                    .any(|a| a.name.as_ref().is_some_and(|n| n.text == "none"));
                for (arg, ty) in args.iter().zip(&tys) {
                    if arg.name.is_none() && !self.implements_print(ty, allow_none) {
                        let snippet = self.quote(arg.value.span);
                        let message = if matches!(ty, Ty::Bool) {
                            format!(
                                "`{snippet}` has type `Bool`, which does not implement `Show`; use `print(yes({snippet}))`"
                            )
                        } else {
                            format!("`{snippet}` has type `{ty}`, which does not implement `Show`")
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
            ("max" | "min", [_]) => {
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

    /// `Prefix(a)` on `a: [R, C]T` builds the family's 2-D shape.
    pub(super) fn grid_shape_call(&mut self, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        let shape = prefix_shape_struct(name, 3)?;
        if self.local(name).is_some()
            || self.global(shape).is_none()
            || args.len() != 1
            || args[0].name.is_some()
        {
            return None;
        }
        let Ty::Array { index, elem } = self.quietly(|c| c.infer(&args[0].value)) else {
            return None;
        };
        if index.len() != 2 || !index.iter().all(|d| matches!(d, Ty::Pos(_))) {
            return None;
        }
        self.infer(&args[0].value);
        Some(Ty::Named {
            name: shape.to_string(),
            args: vec![index[0].clone(), index[1].clone(), *elem],
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
                    let Some(Def::Enum {
                        variants,
                        tagged: false,
                        ..
                    }) = self.type_def(name).as_deref().cloned()
                    else {
                        continue;
                    };
                    if variants.iter().any(|(_, fields)| !fields.is_empty()) {
                        continue;
                    }
                    let tags: Vec<String> =
                        variants.iter().map(|(v, _)| format!("{v:?}")).collect();
                    let message = format!(
                        "reading a `{name}` needs to know how its variants are written in the input"
                    );
                    if let Some(diag) = self.error(codes::TYPE_MISMATCH, init.span, message) {
                        diag.help =
                            Some(format!("put `@tags({})` above the enum", tags.join(", ")));
                    }
                }
                _ => {}
            }
        }
    }

    /// `Bool` is not an input token: a judge spells yes and no its own way
    /// (R10.2.3).
    pub(crate) fn not_a_token_type(&mut self, span: Span) {
        let message = "`read()` can't read a `Bool`: it is not a token type".to_string();
        if let Some(diag) = self.error(codes::NOT_A_TOKEN_TYPE, span, message) {
            diag.help = Some("read a `Str` and compare it: `read<Str>() == \"YES\"`".to_string());
        }
    }

    /// `vis.map(yes)`: a fn value given where a fn of `given` parameters
    /// is expected takes only those; every one after them needs a default
    /// (R06.1.8).
    /// Gives what the fn returns.
    pub(crate) fn fn_value_takes(&mut self, method: &str, arg: &Expr, given: usize) -> Ty {
        let Ty::Fn { params, ret } = self.quietly(|c| c.infer(arg)) else {
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
        let message = format!(
            "`{method}` calls `{}` with {given} {noun}, but its parameter `{}` has no default",
            self.quote(arg.span),
            missing.name,
        );
        if let Some(diag) = self.error(codes::TYPE_MISMATCH, arg.span, message) {
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
        let fields = match elem {
            Ty::Tuple(items) => items.len(),
            _ => 1,
        };
        if params != fields {
            let message = format!("lambda takes {params} parameters, but `{elem}` has {fields}");
            self.error(codes::WRONG_ARG_COUNT, span, message);
        }
        Ty::Unknown
    }

    /// `c.bump()` where `bump` takes `mut self` changes `c`, so `c` is a
    /// `var`, a `mut` parameter, or the `mut self` of the calling method.
    /// Without this the call works on a copy and the change is lost.
    /// `p.swap()` on a type with its own non-`mut` `swap`, or `s.add(x, y)`
    /// on an operation-value field `add`, is not the built-in mutator of
    /// that name (R05.4.1).
    pub(crate) fn own_non_mutator(&mut self, callee: &Expr) -> bool {
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let Ty::Named {
            name: type_name, ..
        } = self.quietly(|c| c.infer(base))
        else {
            return false;
        };
        self.members_of(&type_name).is_some_and(|m| {
            m.fields.iter().any(|f| f.name == name.text)
                || m.members.iter().any(|(n, _)| *n == name.text)
                    && !m.mutators.contains(&name.text)
        })
    }
}
