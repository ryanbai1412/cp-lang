//! The type a declared name stands for when written in a type: a struct with its
//! type arguments checked and defaults filled, an enum, an alias, a domain.

use super::*;
use cpl_diag::msgs;

impl Checker<'_> {
    /// A struct's type arguments meet its parameters' bounds, reported at the type the
    /// user wrote: `Graph<V, E>` with `E` an edge domain is not an arc type of `V`
    /// (`A: Target<V>`), caught here rather than inside a std body (R08.2.5).
    fn struct_args_meet_bounds(
        &mut self,
        name: &str,
        span: Span,
        generics: &[String],
        bounds: &[(usize, String, Vec<Option<usize>>)],
        args: &[Ty],
    ) {
        for (i, trait_name, trait_args) in bounds {
            let Some(ty) = args.get(*i) else {
                continue;
            };
            if ty.mentions_param() || matches!(ty, Ty::Unknown | Ty::Error) {
                continue;
            }
            let given: Option<Vec<Ty>> = trait_args
                .iter()
                .map(|a| a.and_then(|a| args.get(a).cloned()))
                .collect();
            let (meets, wants) = match (trait_name.as_str(), given.as_deref()) {
                ("Target", Some([v])) => {
                    if v.mentions_param() || matches!(v, Ty::Unknown | Ty::Error) {
                        continue;
                    }
                    let meets = match ty {
                        Ty::Record(fields) => fields.iter().any(|(n, t)| n == "to" && t == v),
                        other => other == v,
                    };
                    (
                        meets,
                        format!("an arc type of `{v}`: `{v}` itself or a record with `to: {v}`"),
                    )
                }
                // Built-in types meet std traits through their impls in std, which the
                // checker's `implements_trait` does not read yet; only a struct is asked.
                _ if !matches!(ty, Ty::Named { .. }) => continue,
                _ => (
                    self.implements_trait(ty, trait_name),
                    format!("`{trait_name}`"),
                ),
            };
            if meets {
                continue;
            }
            let param = generics.get(*i).map_or("", String::as_str);
            if let Some(diag) = self.report(
                span,
                msgs::check::MISSING_IMPL_IS_NOT_WHICH_NEEDS,
                &[&ty, &wants, &name, &param],
            ) {
                diag.help = Some(format!(
                    "drop the type arguments and let `{name}` infer them, or pass {wants} for `{param}`"
                ));
            }
        }
    }

    /// The type a definition gives its name when the name is used as a type.
    pub(super) fn def_type(&mut self, def: &Def, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        match def {
            Def::Domain(_) => Ty::Pos(name.to_string()),
            // `let (C, comp) = scc(..)`: a domain a fn made and returned.
            Def::Value {
                ty: Ty::Domain(domain),
                ..
            } => Ty::Pos(domain.clone()),
            Def::Alias { ty, generics } => {
                match ty.subst(&generics.iter().cloned().zip(args).collect()) {
                    // A range keeps how it is written, which here is the alias.
                    Ty::Range { lo, hi, .. } => Ty::Range {
                        lo,
                        hi,
                        text: name.to_string(),
                    },
                    ty => ty,
                }
            }
            Def::Struct {
                generics,
                defaults,
                generic_bounds,
                ..
            } => {
                // `Pt<1..=100>` on a struct with no generics, or `Q<Int, Int>`
                // on `Q<T>`: arguments past the declared ones (R08.2.5).
                if args.len() > generics.len() {
                    self.report(
                        span,
                        msgs::check::WRONG_ARG_COUNT_TAKES_BUT_GIVEN,
                        &[
                            &name,
                            &match generics.len() {
                                0 => "no type arguments".to_string(),
                                n => super::expr::plural(n, "type argument", "type arguments"),
                            },
                            &if args.len() == 1 {
                                "1 was".to_string()
                            } else {
                                format!("{} were", args.len())
                            },
                        ],
                    );
                    return Ty::Error;
                }
                let mut args = args;
                if args.len() < defaults.len() {
                    let mut bindings: crate::ty::Bindings = generics
                        .iter()
                        .zip(&args)
                        .map(|(g, a)| (g.clone(), a.clone()))
                        .collect();
                    for (i, default_ty) in defaults.iter().enumerate().skip(args.len()) {
                        if let Some(default_ty) = default_ty {
                            let filled = default_ty.subst(&bindings);
                            if let Some(g) = generics.get(i) {
                                bindings.insert(g.clone(), filled.clone());
                            }
                            args.push(filled);
                        } else {
                            break;
                        }
                    }
                }
                self.struct_args_meet_bounds(name, span, generics, generic_bounds, &args);
                Ty::Named {
                    name: name.to_string(),
                    args,
                }
            }
            Def::Enum { .. } => Ty::Named {
                name: name.to_string(),
                args,
            },
            Def::Trait { .. } => Ty::Named {
                name: name.to_string(),
                args,
            },
            // Declared by a broken statement: its error is already reported.
            Def::Value { ty: Ty::Error, .. } | Def::Fn { ty: Ty::Error, .. } => Ty::Error,
            Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. } | Def::Impl { .. } => {
                self.report(span, msgs::check::UNKNOWN_TYPE_IS_A_VALUE_NOT, &[&name]);
                Ty::Error
            }
        }
    }
}
