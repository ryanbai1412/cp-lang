//! Hover text, go-to-definition, and the outline.
//!
//! Hover answers "what is this and what does it cost?" (`plan/60-editor.md`):
//!
//! ```text
//! dist: [V]?Int
//!   domain V = 1..=n, n <= 2e5
//!   storage: 8 bytes per element, 1.6 MB max
//! ```

use crate::builtins::builtin_member;
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
    /// The `///` comment above the declaration, wherever it is declared.
    pub doc: Option<String>,
}

/// Where a symbol is declared, possibly in an imported module.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Definition {
    pub path: String,
    pub span: Span,
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
    /// Whether other modules may import it.
    pub public: bool,
}

impl Database {
    /// What to show when the cursor is at `offset`.
    pub fn hover(&mut self, path: &str, offset: u32) -> Option<Hover> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let entry = innermost(&item.check.hovers, at)?;
        let span = entry.span.shift(item.offset);
        let mut lookup = |name: &str| self.lookup(path, &analysis, name);
        let builtin = (entry.keyword == "field")
            .then(|| builtin_member(entry.owner.as_deref()?, &entry.name))
            .flatten();
        let text = match builtin {
            Some(member) if entry.ty == Ty::Unknown => member.signature(),
            _ => render(entry, &mut lookup),
        };
        let doc = self.definition_location(path, offset).and_then(|def| {
            let source = if def.path == path {
                analysis.text.clone()
            } else {
                analysis
                    .modules()
                    .iter()
                    .find(|m| m.path == def.path)?
                    .analysis
                    .text
                    .clone()
            };
            doc_above(&source, def.span.start)
        });
        let doc = doc.or_else(|| builtin.map(|m| m.doc.to_string()));
        let text = match doc.as_deref().and_then(cost_of) {
            Some(cost) if matches!(entry.ty, Ty::Fn { .. }) || entry.keyword == "field" => {
                format!("{text}\ncost: {cost}")
            }
            _ => text,
        };
        Some(Hover { span, text, doc })
    }

    /// Where the name at `offset` is declared, in the same file.
    pub fn definition(&mut self, path: &str, offset: u32) -> Option<Span> {
        let def = self.definition_location(path, offset)?;
        (def.path == path).then_some(def.span)
    }

    /// Where the name at `offset` is declared, following `use` imports into
    /// their defining module.
    pub fn definition_location(&mut self, path: &str, offset: u32) -> Option<Definition> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let entry = innermost(&item.check.hovers, at)?;
        if let (Some(owner), "field") = (&entry.owner, entry.keyword) {
            let (owner_path, owner_analysis, owner_name) = match analysis.imports().get(owner) {
                Some(import) => {
                    let module = &analysis.modules()[import.module];
                    (
                        module.path.clone(),
                        module.analysis.clone(),
                        import.name.clone(),
                    )
                }
                None => (path.to_string(), analysis.clone(), owner.clone()),
            };
            return member_span(&owner_analysis, &owner_name, &entry.name).map(|span| Definition {
                path: owner_path,
                span,
            });
        }
        if let Some(decl) = entry.decl {
            return Some(Definition {
                path: path.to_string(),
                span: decl.shift(item.offset),
            });
        }
        if let Some(span) = analysis.decl_span(&entry.name) {
            return Some(Definition {
                path: path.to_string(),
                span,
            });
        }
        if let Some(import) = analysis.imports().get(&entry.name) {
            let module = &analysis.modules()[import.module];
            if let Some(span) = module.analysis.decl_span(&import.name) {
                return Some(Definition {
                    path: module.path.clone(),
                    span,
                });
            }
        }
        None
    }

    /// The file's top-level declarations, in source order.
    pub fn symbols(&mut self, path: &str) -> Vec<Symbol> {
        let Some(analysis) = self.analyze(path) else {
            return Vec::new();
        };
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
                    public: decl.public,
                });
            }
        }
    }
    out
}

/// Where `owner`'s field or method `member` is declared: in the struct's
/// body, an `impl` of it, or a top-level fn taking it as `self`.
fn member_span(analysis: &Analysis, owner: &str, member: &str) -> Option<Span> {
    use cpl_syntax::ast::{StmtKind, TypeKind};
    let fn_named = |stmts: &[cpl_syntax::ast::Stmt]| {
        stmts.iter().find_map(|stmt| match &stmt.kind {
            StmtKind::Fn(f) if f.name.text == member => Some(f.name.span),
            _ => None,
        })
    };
    for item in &analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            let found = match &stmt.kind {
                StmtKind::Struct(decl) if decl.name.text == owner => decl
                    .fields
                    .iter()
                    .find(|f| f.name.text == member)
                    .map(|f| f.name.span)
                    .or_else(|| fn_named(&decl.methods)),
                StmtKind::Impl { target, body, .. } => match &target.kind {
                    TypeKind::Named { name, .. } if name.text == owner => fn_named(&body.stmts),
                    _ => None,
                },
                _ => None,
            };
            if let Some(span) = found {
                return Some(span.shift(item.offset));
            }
        }
    }
    analysis.decl_span(member)
}

/// The `///` lines above the line `at` is on, past its decorators.
fn doc_above(text: &str, at: u32) -> Option<String> {
    let head = text.get(..at as usize)?;
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let mut lines = Vec::new();
    for line in head[..line_start].lines().rev() {
        let line = line.trim();
        if let Some(doc) = line.strip_prefix("///") {
            lines.push(doc.strip_prefix(' ').unwrap_or(doc));
        } else if !line.starts_with('@') {
            break;
        }
    }
    lines.reverse();
    let doc = lines.join("\n").trim().to_string();
    (!doc.is_empty()).then_some(doc)
}

/// The smallest entry under the cursor.
/// The first `O(...)` a doc states, with what qualifies it: `O(log n)
/// amortized`.
fn cost_of(doc: &str) -> Option<String> {
    let start = doc.find("O(")?;
    let mut depth = 0;
    for (i, c) in doc[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + i + 1;
                    let amortized = doc[end..].trim_start().starts_with("amortized");
                    let cost = &doc[start..end];
                    return Some(if amortized {
                        format!("{cost} amortized")
                    } else {
                        cost.to_string()
                    });
                }
            }
            _ => {}
        }
    }
    None
}

fn innermost(hovers: &[HoverEntry], at: u32) -> Option<&HoverEntry> {
    hovers
        .iter()
        .filter(|h| h.span.touches(at))
        .min_by_key(|h| h.span.len())
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
        Def::Value { ty, generics, .. } => {
            let titled = with_generics(name, generics);
            render_value(&titled, ty, lookup)
        }
        Def::Fn { ty, generics } => fn_text(&with_generics(name, generics), ty),
        Def::Struct {
            generics,
            fields,
            methods,
            projections,
            ..
        } => {
            let mut out = format!("struct {}", with_generics(name, generics));
            for field in fields {
                out += &format!("\n  {}: {}", field.name, field.ty);
            }
            for (method, ty) in methods {
                let line = match projections.iter().find(|p| p.name == *method) {
                    Some(proj) => {
                        let kw = if proj.mutable { "lends mut" } else { "lends" };
                        let base = fn_text(method, ty);
                        match base.rsplit_once(" -> ") {
                            Some((head, _)) => format!("{head} {kw} {}", proj.lent),
                            None => format!("{base} {kw} {}", proj.lent),
                        }
                    }
                    None => fn_text(method, ty),
                };
                out += &format!("\n  {line}");
            }
            if generics.is_empty() {
                let whole = Ty::Named {
                    name: name.to_string(),
                    args: Vec::new(),
                };
                if let Some(total) = size(&whole, lookup) {
                    out += &format!("\n  storage: {}", human_bytes(total.into()));
                }
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
                        format!(
                            "{v}({})",
                            fields
                                .iter()
                                .map(Ty::to_string)
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                })
                .collect();
            format!("enum {name} = {}", names.join(" | "))
        }
        Def::Variant { of, fields } if fields.is_empty() => format!("{of}.{name}"),
        Def::Variant { of, fields } => {
            format!(
                "{of}.{name}({})",
                fields
                    .iter()
                    .map(Ty::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
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
        Def::Trait { supers, methods } => {
            let mut out = if supers.is_empty() {
                format!("trait {name}")
            } else {
                format!("trait {name}: {}", supers.join(" + "))
            };
            for m in methods {
                out += &format!("\n  {}", fn_text(&m.name, &m.ty));
            }
            out
        }
        Def::Impl { .. } => name.to_string(),
    }
}

fn with_generics(name: &str, generics: &[String]) -> String {
    if generics.is_empty() {
        name.to_string()
    } else {
        format!("{name}<{}>", generics.join(", "))
    }
}

fn fn_text(name: &str, ty: &Ty) -> String {
    match ty.to_string().strip_prefix("fn") {
        Some(rest) => format!("fn {name}{rest}"),
        None => format!("{name}: {ty}"),
    }
}

fn render_value(name: &str, ty: &Ty, lookup: &mut Lookup) -> String {
    let shown = if *ty == Ty::Error {
        "not known until the error in its definition is fixed".to_string()
    } else {
        ty.to_string()
    };
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
    let domain_sizes: Vec<(String, Option<i128>)> = domains
        .into_iter()
        .map(|n| (n.clone(), domain_size(&n, lookup)))
        .collect();
    let mut structs = Vec::new();
    structs_in(ty, &mut structs);
    let struct_sizes: Vec<(String, Option<u64>)> = structs
        .into_iter()
        .map(|name| {
            let size = match lookup(&name).as_deref() {
                Some(Def::Struct { fields, .. }) => {
                    let sizes: Vec<Option<u64>> = fields
                        .iter()
                        .map(|f| size_at_depth(&f.ty, lookup, depth + 1))
                        .collect();
                    struct_size(sizes.into_iter())
                }
                _ => None,
            };
            (name, size)
        })
        .collect();
    size_of(
        ty,
        &|name| {
            domain_sizes
                .iter()
                .find(|(n, _)| n == name)
                .and_then(|(_, s)| *s)
        },
        &|name| {
            struct_sizes
                .iter()
                .find(|(n, _)| n == name)
                .and_then(|(_, s)| *s)
        },
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
        Ty::Range { .. }
        | Ty::Pos(_)
        | Ty::Optional(_)
        | Ty::Tuple(_)
        | Ty::Record(_)
        | Ty::Named { .. } => {
            if let Some(bytes) = size(ty, lookup) {
                out += &format!("\n  storage: {}", human_bytes(bytes.into()));
            }
        }
        _ => {}
    }
    out
}
