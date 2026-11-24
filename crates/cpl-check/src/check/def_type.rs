//! The type a declared name stands for when written in a type: a struct with its
//! type arguments checked and defaults filled, an enum, an alias, a domain.

use super::*;

impl Checker<'_> {
    /// The type a definition gives its name when the name is used as a type.
    pub(super) fn def_type(&mut self, def: &Def, name: &str, span: Span, args: Vec<Ty>) -> Ty {
        match def {
            Def::Domain(_) => Ty::Pos(name.to_string()),
            // `let (C, comp) = scc(..)`: a domain a fn made and returned.
            Def::Value {
                ty: Ty::Domain(domain),
                ..
            } => Ty::Pos(domain.clone()),
            Def::Alias { ty, generics } => ty.subst(&generics.iter().cloned().zip(args).collect()),
            Def::Struct {
                generics, defaults, ..
            } => {
                // `Pt<1..=100>` on a struct with no generics, or `Q<Int, Int>`
                // on `Q<T>`: arguments past the declared ones (R08.2.5).
                if args.len() > generics.len() {
                    let message = format!(
                        "`{name}` takes {}, but {} given",
                        match generics.len() {
                            0 => "no type arguments".to_string(),
                            n => super::expr::plural(n, "type argument", "type arguments"),
                        },
                        if args.len() == 1 {
                            "1 was".to_string()
                        } else {
                            format!("{} were", args.len())
                        }
                    );
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return Ty::Error;
                }
                let mut args = args;
                if args.len() < defaults.len() {
                    let mut bindings: HashMap<String, Ty> = generics
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
                Ty::Named {
                    name: name.to_string(),
                    args,
                }
            }
            Def::Enum { .. } => Ty::Named {
                name: name.to_string(),
                args,
            },
            Def::Trait { .. } => Ty::Unknown,
            // Declared by a broken statement: its error is already reported.
            Def::Value { ty: Ty::Error, .. } | Def::Fn { ty: Ty::Error, .. } => Ty::Error,
            Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. } | Def::Impl { .. } => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    span,
                    format!("`{name}` is a value, not a type"),
                );
                Ty::Error
            }
        }
    }
}
