//! What a binding pattern binds: each name, its type and its span, in one
//! order shared by `pattern_types` and `name_spans`.

use super::*;
use cpl_syntax::ast::{self, Pattern, PatternKind};

pub(crate) fn pattern_types(pat: &ast::Pattern, ty: &Ty, f: &mut dyn FnMut(&str, Ty)) {
    use ast::PatternKind as P;
    match &pat.kind {
        P::Name(name) => f(name, ty.clone()),
        P::Domain(name) => f(&name.text, Ty::Domain(name.text.clone())),
        P::Var(inner) => pattern_types(inner, ty, f),
        P::Tuple(items) => {
            let mut renames: Vec<(String, String)> = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let mut item_ty = match ty {
                    Ty::Tuple(tys) if tys.len() == items.len() => tys[i].clone(),
                    Ty::Record(fields) if fields.len() == items.len() => fields[i].1.clone(),
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                for (from, to) in &renames {
                    item_ty = item_ty.rename_domain(from, to);
                }
                match domain_binding(item) {
                    // `let (domain P, part) = f(..)` or `let (C, comp) = scc(..)`:
                    // bind the fresh domain under the caller's name and rename it
                    // across the sibling tuple items.
                    Some(name) => {
                        if let Ty::Domain(orig) = &item_ty {
                            if orig != name {
                                renames.push((orig.clone(), name.to_string()));
                            }
                        }
                        f(name, Ty::Domain(name.to_string()));
                    }
                    None => pattern_types(item, &item_ty, f),
                }
            }
        }
        P::Record { fields, .. } => {
            for (name, item) in fields {
                let item_ty = match ty {
                    Ty::Record(have) => have
                        .iter()
                        .find(|(n, _)| *n == name.text)
                        .map_or(Ty::Unknown, |(_, t)| t.clone()),
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                pattern_types(item, &item_ty, f);
            }
        }
        _ => {}
    }
}

/// `domain P` or a capitalized name alone inside a tuple pattern binds a domain:
/// `let (domain P, part) = f(..)` or `let (C, comp) = scc(..)`.
pub(crate) fn domain_binding(pat: &ast::Pattern) -> Option<&str> {
    match &pat.kind {
        ast::PatternKind::Domain(name) => Some(&name.text),
        ast::PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
            Some(&path[0].text)
        }
        _ => None,
    }
}

/// The span of each name a pattern binds, in the order `pattern_types`
/// visits them.
pub(crate) fn name_spans(pat: &Pattern, is_var: bool, out: &mut Vec<(Span, bool)>) {
    match &pat.kind {
        PatternKind::Name(_) => out.push((pat.span, is_var)),
        PatternKind::Domain(name) => out.push((name.span, is_var)),
        PatternKind::Var(inner) => name_spans(inner, true, out),
        PatternKind::Tuple(items) => {
            for item in items {
                match &item.kind {
                    PatternKind::Domain(name) => out.push((name.span, is_var)),
                    _ => match domain_binding(item) {
                        Some(_) => out.push((item.span, is_var)),
                        None => name_spans(item, is_var, out),
                    },
                }
            }
        }
        PatternKind::Record { fields, .. } => {
            for (_, item) in fields {
                name_spans(item, is_var, out);
            }
        }
        _ => {}
    }
}
