//! What a binding pattern binds: each name, its type and its span, in one
//! order shared by `pattern_types` and `name_spans`.

use super::*;
use cpl_syntax::ast::{self, Pattern, PatternKind};

pub(crate) fn pattern_types(pat: &ast::Pattern, ty: &Ty, f: &mut dyn FnMut(&str, Ty)) {
    typed_pattern(pat, ty, &mut |_, _| Vec::new(), f)
}

/// [`pattern_types`], with `payload` giving the types a variant pattern
/// (`Some(y)`, `Node(l, r)`) matched against a value of type `ty` holds.
pub(crate) fn typed_pattern(
    pat: &ast::Pattern,
    ty: &Ty,
    payload: &mut dyn FnMut(&[ast::Name], &Ty) -> Vec<Ty>,
    f: &mut dyn FnMut(&str, Ty),
) {
    use ast::PatternKind as P;
    match &pat.kind {
        P::Name(name) => f(name, ty.clone()),
        P::Domain(name) => f(&name.text, Ty::Domain(name.text.clone())),
        P::Var(inner) => typed_pattern(inner, ty, payload, f),
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
                    None => typed_pattern(item, &item_ty, payload, f),
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
                typed_pattern(item, &item_ty, payload, f);
            }
        }
        P::Variant { path, args } => {
            let fields = payload(path, ty);
            for (i, arg) in args.iter().enumerate() {
                let ty = fields.get(i).cloned().unwrap_or(Ty::Unknown);
                typed_pattern(arg, &ty, payload, f);
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

impl Checker<'_> {
    /// Binds the names a refutable pattern takes apart: `let ... else`, or an
    /// arm like `(Some(lo), Some(hi)) if lo < hi`.
    /// The types the variant `path` holds when matched against a `ty`.
    pub(crate) fn variant_payload(&mut self, path: &[ast::Name], ty: &Ty) -> Vec<Ty> {
        if self.some_pattern(path) {
            return vec![match ty {
                Ty::Optional(inner) => (**inner).clone(),
                _ => Ty::Unknown,
            }];
        }
        match path.last().and_then(|n| self.global(&n.text)) {
            Some(def) => def.payload_at(ty),
            None => Vec::new(),
        }
    }

    pub(crate) fn bind_refutable(
        &mut self,
        pat: &Pattern,
        ty: &Ty,
        mutable: bool,
        keyword: &'static str,
    ) {
        let PatternKind::Tuple(items) = &pat.kind else {
            return self.bind(pat, ty, mutable, keyword);
        };
        for (i, item) in items.iter().enumerate() {
            let item_ty = match ty {
                Ty::Tuple(tys) if tys.len() == items.len() => tys[i].clone(),
                Ty::Record(fields) if fields.len() == items.len() => fields[i].1.clone(),
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            };
            self.bind_refutable(item, &item_ty, mutable, keyword);
        }
    }
}
