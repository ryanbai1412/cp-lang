//! Hover text, go-to-definition, and the outline.
//!
//! Hover answers "what is this and what does it cost?" (`plan/60-editor.md`):
//!
//! ```text
//! dist: [V]?Int
//!   domain V = 1..=n, n <= 2e5
//!   storage: 8 bytes per element, 1.6 MB max
//! ```

use crate::check::HoverEntry;
use crate::db::{Analysis, Database};
use crate::def::{DeclKind, Def};
use crate::ty::{human_bytes, human_int, size_of, struct_size, Ty};
use cpl_diag::Span;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hover {
    /// The name or expression hovered, in file positions.
    pub span: Span,
    pub text: String,
}

/// A top-level declaration, for the editor's outline.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: DeclKind,
    /// The whole item.
    pub span: Span,
    /// Just the name.
    pub name_span: Span,
}

impl Database {
    /// What to show when the cursor is at `offset`.
    pub fn hover(&mut self, path: &str, offset: u32) -> Option<Hover> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let entry = innermost(&item.check.hovers, at)?;
        let span = entry.span.shift(item.offset);
        let mut lookup = |name: &str| self.lookup(path, &analysis, name);
        Some(Hover {
            span,
            text: render(entry, &mut lookup),
        })
    }

    /// Where the name at `offset` is declared.
    pub fn definition(&mut self, path: &str, offset: u32) -> Option<Span> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let entry = innermost(&item.check.hovers, at)?;
        match entry.decl {
            Some(decl) => Some(decl.shift(item.offset)),
            None => analysis.decl_span(&entry.name),
        }
    }

    /// The file's top-level declarations, in source order.
    pub fn symbols(&mut self, path: &str) -> Vec<Symbol> {
        let Some(analysis) = self.analyze(path) else { return Vec::new() };
        symbols(&analysis)
    }
}

fn symbols(analysis: &Analysis) -> Vec<Symbol> {
    let mut out = Vec::new();
    for item in &analysis.items {
        for decl in &item.syntax.decls {
            if decl.kind != DeclKind::Variant {
                out.push(Symbol {
                    name: decl.name.clone(),
                    kind: decl.kind,
                    span: item.span,
                    name_span: decl.span.shift(item.offset),
                });
            }
        }
    }
    out
}

/// The smallest entry under the cursor.
fn innermost(hovers: &[HoverEntry], at: u32) -> Option<&HoverEntry> {
    hovers.iter().filter(|h| h.span.touches(at)).min_by_key(|h| h.span.len())
}

type Lookup<'a> = dyn FnMut(&str) -> Option<Arc<Def>> + 'a;

pub(crate) fn render(entry: &HoverEntry, lookup: &mut Lookup) -> String {
    let name = &entry.name;
    match entry.keyword {
        "global" | "type" => match lookup(name).as_deref() {
            Some(def) => render_def(name, def, lookup),
            None => render_value(name, &entry.ty, lookup),
        },
        "fn" => fn_text(name, &entry.ty),
        _ => render_value(name, &entry.ty, lookup),
    }
}

fn render_def(name: &str, def: &Def, lookup: &mut Lookup) -> String {
    match def {
        Def::Value { ty, .. } => render_value(name, ty, lookup),
        Def::Fn { ty, .. } => fn_text(name, ty),
        Def::Struct { fields, methods, .. } => {
            let mut out = format!("struct {name}");
            for field in fields {
                out += &format!("\n  {}: {}", field.name, field.ty);
            }
            for (method, ty) in methods {
                out += &format!("\n  {}", fn_text(method, ty));
            }
            let whole = Ty::Named {
                name: name.to_string(),
                args: Vec::new(),
            };
            if let Some(total) = size(&whole, lookup) {
                out += &format!("\n  storage: {}", human_bytes(total.into()));
            }
            out
        }
        Def::Enum { variants, .. } => {
            let names: Vec<String> = variants
                .iter()
                .map(|(v, fields)| {
                    if fields.is_empty() {
                        v.clone()
                    } else {
                        format!("{v}({})", fields.iter().map(Ty::to_string).collect::<Vec<_>>().join(", "))
                    }
                })
                .collect();
            format!("enum {name} = {}", names.join(" | "))
        }
        Def::Variant { of, fields } if fields.is_empty() => format!("{of}.{name}"),
        Def::Variant { of, fields } => {
            format!("{of}.{name}({})", fields.iter().map(Ty::to_string).collect::<Vec<_>>().join(", "))
        }
        Def::Domain(info) => {
            let mut out = format!("domain {name} = {}", info.text);
            if let Some(max) = info.max_size {
                out += &format!("\n  at most {} positions", human_int(max));
                if let Some(bound) = &info.bound {
                    out += &format!(" ({bound})");
                }
            }
            if let Some(bytes) = size(&Ty::Pos(name.to_string()), lookup) {
                out += &format!("\n  storage: {} per position", human_bytes(bytes.into()));
            }
            out
        }
        Def::Alias { ty } => {
            let mut out = format!("type {name} = {ty}");
            out += &cost_lines(ty, lookup);
            out
        }
        Def::Trait => format!("trait {name}"),
    }
}

fn fn_text(name: &str, ty: &Ty) -> String {
    match ty.to_string().strip_prefix("fn") {
        Some(rest) => format!("fn {name}{rest}"),
        None => format!("{name}: {ty}"),
    }
}

fn render_value(name: &str, ty: &Ty, lookup: &mut Lookup) -> String {
    let shown = if *ty == Ty::Error { "unknown, because of the error above".to_string() } else { ty.to_string() };
    format!("{name}: {shown}{}", cost_lines(ty, lookup))
}

fn size(ty: &Ty, lookup: &mut Lookup) -> Option<u64> {
    size_at_depth(ty, lookup, 0)
}

fn size_at_depth(ty: &Ty, lookup: &mut Lookup, depth: u32) -> Option<u64> {
    // `size_of` takes shared closures, so what it will ask for is fetched
    // first. The depth limit stops a struct that contains itself.
    if depth > 8 {
        return None;
    }
    let mut domains = Vec::new();
    domains_in(ty, &mut domains);
    let domain_sizes: Vec<(String, Option<i128>)> = domains.into_iter().map(|n| (n.clone(), domain_size(&n, lookup))).collect();
    let mut structs = Vec::new();
    structs_in(ty, &mut structs);
    let struct_sizes: Vec<(String, Option<u64>)> = structs
        .into_iter()
        .map(|name| {
            let size = match lookup(&name).as_deref() {
                Some(Def::Struct { fields, .. }) => {
                    let sizes: Vec<Option<u64>> = fields.iter().map(|f| size_at_depth(&f.ty, lookup, depth + 1)).collect();
                    struct_size(sizes.into_iter())
                }
                _ => None,
            };
            (name, size)
        })
        .collect();
    size_of(
        ty,
        &|name| domain_sizes.iter().find(|(n, _)| n == name).and_then(|(_, s)| *s),
        &|name| struct_sizes.iter().find(|(n, _)| n == name).and_then(|(_, s)| *s),
    )
}

/// The struct names a type's size depends on.
fn structs_in(ty: &Ty, out: &mut Vec<String>) {
    match ty {
        Ty::Named { name, args } if args.is_empty() => {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        Ty::Optional(inner) => structs_in(inner, out),
        Ty::Tuple(items) => items.iter().for_each(|i| structs_in(i, out)),
        Ty::Record(fields) => fields.iter().for_each(|(_, t)| structs_in(t, out)),
        _ => {}
    }
}

fn domain_size(name: &str, lookup: &mut Lookup) -> Option<i128> {
    match lookup(name).as_deref() {
        Some(Def::Domain(info)) => info.max_size,
        _ => None,
    }
}

/// The domains a type mentions, in the order they appear.
fn domains_in(ty: &Ty, out: &mut Vec<String>) {
    match ty {
        Ty::Pos(name) | Ty::Domain(name) => {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        Ty::Array { index, elem } => {
            index.iter().for_each(|i| domains_in(i, out));
            domains_in(elem, out);
        }
        Ty::Optional(inner) => domains_in(inner, out),
        Ty::Tuple(items) => items.iter().for_each(|i| domains_in(i, out)),
        Ty::Record(fields) => fields.iter().for_each(|(_, t)| domains_in(t, out)),
        Ty::Named { args, .. } => args.iter().for_each(|a| domains_in(a, out)),
        _ => {}
    }
}

/// The lines under a value's type: its domains and its storage.
fn cost_lines(ty: &Ty, lookup: &mut Lookup) -> String {
    let mut out = String::new();
    let mut names = Vec::new();
    domains_in(ty, &mut names);
    for name in &names {
        if let Some(Def::Domain(info)) = lookup(name).as_deref() {
            out += &format!("\n  domain {name} = {}", info.text);
            if let Some(bound) = &info.bound {
                out += &format!(", {bound}");
            } else if let Some(max) = info.max_size {
                out += &format!(", at most {} positions", human_int(max));
            }
        }
    }
    match ty {
        Ty::Array { index, elem } => {
            if let Some(per) = size(elem, lookup) {
                out += &format!("\n  storage: {} per element", human_bytes(per.into()));
                let count = index.iter().try_fold(1i128, |acc, i| match i {
                    Ty::Pos(name) => domain_size(name, lookup).and_then(|n| acc.checked_mul(n)),
                    _ => None,
                });
                if let Some(count) = count {
                    out += &format!(", {} max", human_bytes(count as u128 * u128::from(per)));
                }
            }
        }
        Ty::Range { .. } | Ty::Pos(_) | Ty::Optional(_) | Ty::Tuple(_) | Ty::Record(_) | Ty::Named { .. } => {
            if let Some(bytes) = size(ty, lookup) {
                out += &format!("\n  storage: {}", human_bytes(bytes.into()));
            }
        }
        _ => {}
    }
    out
}
