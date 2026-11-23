//! Navigation past a name's own declaration: where its type is declared,
//! and every `impl` of a type or trait.

use crate::db::{Analysis, Database};
use crate::def::{impl_parts, DeclKind};
use crate::hover::{innermost, Definition};
use crate::ty::Ty;

impl Database {
    /// Where the type of the name at `offset` is declared: `Road` for `r`
    /// in `let r: Road`, the element type of an array or an optional.
    pub fn type_definition(&mut self, path: &str, offset: u32) -> Option<Definition> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let entry = innermost(&item.check.hovers, at)?;
        declared(&analysis, path, type_name(&entry.ty)?)
    }

    /// Every `impl` of the type or trait named at `offset`, in this file and
    /// the modules it uses.
    pub fn implementations(&mut self, path: &str, offset: u32) -> Vec<Definition> {
        let Some(analysis) = self.analyze(path) else {
            return Vec::new();
        };
        let Some(name) = word_at(&analysis.text, offset) else {
            return Vec::new();
        };
        let mut out = impls_of(&analysis, path, name);
        for module in analysis.modules() {
            out.extend(impls_of(&module.analysis, &module.path, name));
        }
        out
    }
}

fn type_name(ty: &Ty) -> Option<&str> {
    match ty {
        Ty::Named { name, .. } | Ty::TypeName(name) | Ty::Pos(name) | Ty::Domain(name) => {
            Some(name)
        }
        Ty::Array { elem, .. } | Ty::Optional(elem) => type_name(elem),
        _ => None,
    }
}

/// Where the top-level `name` is declared, here or in the module it is
/// imported from.
fn declared(analysis: &Analysis, path: &str, name: &str) -> Option<Definition> {
    if let Some(span) = analysis.decl_span(name) {
        return Some(Definition {
            path: path.to_string(),
            span,
        });
    }
    let import = analysis.imports().get(name)?;
    let module = &analysis.modules()[import.module];
    Some(Definition {
        path: module.path.clone(),
        span: module.analysis.decl_span(&import.name)?,
    })
}

fn impls_of(analysis: &Analysis, path: &str, name: &str) -> Vec<Definition> {
    let head = |s: &str| s.split('<').next().unwrap_or(s).trim().to_string();
    let mut out = Vec::new();
    for item in &analysis.items {
        for decl in &item.syntax.decls {
            let Some((trait_name, target)) = impl_parts(&decl.name) else {
                continue;
            };
            let matches = head(target) == name || trait_name.is_some_and(|t| head(t) == name);
            if decl.kind == DeclKind::Impl && matches {
                out.push(Definition {
                    path: path.to_string(),
                    span: decl.span.shift(item.offset),
                });
            }
        }
    }
    out
}

fn word_at(text: &str, offset: u32) -> Option<&str> {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut at = (offset as usize).min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    let start = text[..at]
        .char_indices()
        .rfind(|&(_, c)| !is_ident(c))
        .map_or(0, |(i, c)| i + c.len_utf8());
    let end = text[at..]
        .find(|c: char| !is_ident(c))
        .map_or(text.len(), |i| at + i);
    (start < end).then(|| &text[start..end])
}
