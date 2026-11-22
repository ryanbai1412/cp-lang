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
use crate::ty::{has_niche, human_bytes, human_int, size_of, struct_layout, struct_size, Ty};
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
    /// The most bytes the file's top-level values take at max input, and
    /// the names of those whose size has no bound, with the size of each
    /// element for a list of fixed-size values.
    #[allow(clippy::type_complexity)]
    pub fn storage(&mut self, path: &str) -> Option<(u128, Vec<(String, Option<u64>)>)> {
        let analysis = self.analyze(path)?;
        let mut names: Vec<String> = analysis.scope.names.keys().cloned().collect();
        names.sort();
        let (mut total, mut unbounded) = (0u128, Vec::new());
        for name in names {
            let Some(def) = self.lookup(path, &analysis, &name) else {
                continue;
            };
            let Def::Value { ty, .. } = &*def else {
                continue;
            };
            let mut lookup = |n: &str| self.lookup(path, &analysis, n);
            match (value_bytes(ty, &mut lookup), ty) {
                (Some(bytes), _) => total += bytes,
                (None, Ty::Named { name: list, args }) if list == "List" && args.len() == 1 => {
                    let per = size(&args[0], &mut lookup);
                    unbounded.push((name, per));
                }
                (None, _) => unbounded.push((name, None)),
            }
        }
        Some((total, unbounded))
    }

    pub fn hover(&mut self, path: &str, offset: u32) -> Option<Hover> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let Some(entry) = innermost(&item.check.hovers, at) else {
            return prelude_value_hover(item, at);
        };
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
        let receiver = item.check.hovers.iter().find(|e| e.span.end + 1 == entry.span.start);
        let generics = match (entry.keyword, entry.owner.as_deref(), receiver.map(|r| &r.ty)) {
            ("field", Some(owner), Some(Ty::Named { name, args })) if name == owner => {
                match self.lookup(path, &analysis, owner).as_deref() {
                    Some(Def::Struct { generics, .. }) => {
                        generics.iter().cloned().zip(args.iter().cloned()).collect()
                    }
                    _ => Vec::new(),
                }
            }
            _ => Vec::new(),
        };
        let cost = doc.as_deref().and_then(cost_of).map(|c| in_receiver_domains(&c, &generics));
        let text = match cost {
            Some(cost) if matches!(entry.ty, Ty::Fn { .. }) || entry.keyword == "field" => {
                format!("{text}\ncost: {cost}")
            }
            _ if item.check.fixed_io.contains(&entry.span) => format!("{text}\ncost: O(1)"),
            _ => text,
        };
        Some(Hover { span, text, doc })
    }

    /// Each call in `start..end` whose doc states a cost above `O(1)`:
    /// where its name is, the name as hovered (`.sorted`, `bfs`) and the
    /// cost (`O(n log n)`), in file positions.
    pub fn costs_in(&mut self, path: &str, start: u32, end: u32) -> Vec<(Span, String, String)> {
        let Some(analysis) = self.analyze(path) else {
            return Vec::new();
        };
        let Some((item, _)) = analysis.item_at(start) else {
            return Vec::new();
        };
        let mut named: Vec<(Span, String)> = item
            .check
            .hovers
            .iter()
            .filter(|e| e.keyword == "field" || matches!(e.ty, Ty::Fn { .. }))
            .filter(|e| e.decl != Some(e.span))
            .map(|e| {
                let name = if e.keyword == "field" { format!(".{}", e.name) } else { e.name.clone() };
                (e.span.shift(item.offset), name)
            })
            .filter(|(span, _)| start <= span.start && span.end <= end)
            .collect();
        named.sort_by_key(|(span, _)| (span.start, span.end));
        named.dedup_by_key(|(span, _)| *span);
        let io = item.check.io_costs.iter().map(|(s, n, c)| (s.shift(item.offset), n.clone(), c.clone()));
        let io: Vec<_> = io.filter(|(s, ..)| start <= s.start && s.end <= end).collect();
        named
            .into_iter()
            .filter_map(|(span, name)| {
                let hover = self.hover(path, span.start)?;
                let cost = hover.text.lines().find_map(|l| l.strip_prefix("cost: "))?;
                (hover.span == span && cost.starts_with("O(") && !cost.starts_with("O(1)")).then(|| (span, name, cost.to_string()))
            })
            .chain(io)
            .collect()
    }

    /// The innermost index at `offset` that its types prove in bounds, in
    /// file positions; with `on_index`, only when `offset` is on its indices.
    pub fn typed_index(&mut self, path: &str, offset: u32, on_index: bool) -> Option<crate::TypedIndex> {
        let analysis = self.analyze(path)?;
        let (item, at) = analysis.item_at(offset)?;
        let inside = |s: Span| s.start <= at && at < s.end;
        let found = item
            .check
            .typed_indexes
            .iter()
            .filter(|t| inside(t.access) && !inside(t.seq) && inside(t.index) == on_index)
            .min_by_key(|t| t.access.len())?;
        Some(crate::TypedIndex {
            access: found.access.shift(item.offset),
            seq: found.seq.shift(item.offset),
            index: found.index.shift(item.offset),
            proof: found.proof.clone(),
        })
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

/// A method's documented cost with its struct's domain parameters written as
/// the receiver's: `O(log |D|)` on a `Fenwick<V, Int>` is `O(log |V|)`, and so
/// is `O(log D.K)`.
fn in_receiver_domains(cost: &str, generics: &[(String, Ty)]) -> String {
    let mut out = cost.to_string();
    for (g, arg) in generics {
        if let Ty::Pos(d) = arg {
            out = out.replace(&format!("|{g}|"), &format!("|{d}|"));
            out = out.replace(&format!("{g}.K"), &format!("|{d}|"));
        }
    }
    out
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

/// A name of the compiler's own prelude at `at`, which the checker records
/// no hover for, shown with its doc for the shape of its call.
fn prelude_value_hover(item: &crate::db::AnalyzedItem, at: u32) -> Option<Hover> {
    use cpl_syntax::ast::{Expr, ExprKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    struct Find {
        at: u32,
        name: Option<(Span, String)>,
        calls: Vec<(Span, usize)>,
    }
    impl Visitor for Find {
        fn expr(&mut self, expr: &Expr) {
            match &expr.kind {
                ExprKind::Name(n) if expr.span.touches(self.at) => self.name = Some((expr.span, n.clone())),
                ExprKind::Call { callee, args } => self.calls.push((callee.span, args.len())),
                _ => {}
            }
        }
    }
    let mut find = Find { at, name: None, calls: Vec::new() };
    for stmt in &item.syntax.parsed.stmts {
        walk_stmt(stmt, &mut find);
    }
    let (span, name) = find.name?;
    let args = find.calls.iter().find(|(callee, _)| *callee == span).map(|&(_, n)| n);
    let fixed = item.check.fixed_io.contains(&span);
    let doc = crate::prelude::value_doc(&name, args, fixed)?;
    let io = item.check.io_costs.iter().find(|(s, ..)| *s == span).map(|(.., c)| c.clone());
    let text = match io.or_else(|| cost_of(doc)) {
        Some(cost) => format!("{name}: built-in\ncost: {cost}"),
        None => format!("{name}: built-in"),
    };
    Some(Hover { span: span.shift(item.offset), text, doc: Some(doc.to_string()) })
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
        Def::Fn { ty, generics, .. } => fn_text(&with_generics(name, generics), ty),
        Def::Struct {
            generics,
            fields,
            methods,
            projections,
            ..
        } => {
            let mut out = format!("struct {}", with_generics(name, generics));
            let sizes: Option<Vec<u64>> = if generics.is_empty() {
                fields.iter().map(|f| size(&f.ty, lookup)).collect()
            } else {
                None
            };
            match &sizes {
                Some(sizes) if !sizes.is_empty() => {
                    let layout = struct_layout(sizes);
                    for (i, field) in fields.iter().enumerate() {
                        let pad = layout.padding_before(i, sizes);
                        if pad > 0 {
                            out += &format!("\n  // {} padding", human_bytes(pad.into()));
                        }
                        out += &format!(
                            "\n  {}: {}  // at {}, {}",
                            field.name,
                            field.ty,
                            layout.offsets[i],
                            human_bytes(sizes[i].into())
                        );
                    }
                    let pad = layout.padding_before(fields.len(), sizes);
                    if pad > 0 {
                        out += &format!("\n  // {} padding", human_bytes(pad.into()));
                    }
                }
                _ => {
                    for field in fields {
                        out += &format!("\n  {}: {}", field.name, field.ty);
                    }
                }
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
                    let align = sizes.as_deref().map_or(1, |s| struct_layout(s).align);
                    out += &format!("\n  storage: {}, aligned to {align}", human_bytes(total.into()));
                } else {
                    let named: Vec<(String, &Ty)> = fields.iter().map(|f| (f.name.to_string(), &f.ty)).collect();
                    if let Some(note) = fields_note(&named, lookup) {
                        out += &format!("\n  storage: {note}");
                    }
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
        Def::Trait {
            supers, methods, ..
        } => {
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

/// The most bytes a value of `ty` takes, an array's at its indexes' most
/// values.
fn value_bytes(ty: &Ty, lookup: &mut Lookup) -> Option<u128> {
    match ty {
        Ty::Array { index, elem } => {
            let per = u128::from(size(elem, lookup)?);
            index.iter().try_fold(per, |acc, i| {
                acc.checked_mul(u128::try_from(index_size(i, lookup)?).ok()?)
            })
        }
        _ => size(ty, lookup).map(u128::from),
    }
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

/// How many values an array's index type has at most: a domain's
/// positions or cuts, or a range's numbers.
fn index_size(index: &Ty, lookup: &mut Lookup) -> Option<i128> {
    match index {
        Ty::Pos(name) => match name.strip_suffix(".cuts") {
            Some(domain) => domain_size(domain, lookup).map(|n| n + 1),
            None => domain_size(name, lookup),
        },
        Ty::Range {
            lo: Some(lo),
            hi: Some(hi),
            ..
        } => Some((hi - lo + 1).max(0)),
        _ => None,
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
                let count = index
                    .iter()
                    .try_fold(1i128, |acc, i| acc.checked_mul(index_size(i, lookup)?));
                if let Some(count) = count {
                    out += &format!(", {} max", human_bytes(count as u128 * u128::from(per)));
                }
                out += &layout_note(elem, lookup);
            } else if let Some(note) = element_note(elem, lookup) {
                out += &format!("\n  storage: {note}");
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
                out += &layout_note(ty, lookup);
            } else {
                out += &held_note(ty, lookup);
            }
        }
        _ => {}
    }
    out
}

/// How a value of `ty` is laid out, after its size: where `none` goes for
/// an optional, and each field's offset and the padding for a tuple, record
/// or struct.
fn layout_note(ty: &Ty, lookup: &mut Lookup) -> String {
    let sizes: Option<Vec<u64>> = match ty {
        Ty::Optional(inner) => {
            let Some(inner_size) = size(inner, lookup) else {
                return String::new();
            };
            return if has_niche(inner, inner_size) {
                " (none is a spare value: no flag)".to_string()
            } else {
                format!(
                    " ({} + {} flag for none)",
                    human_bytes(inner_size.into()),
                    human_bytes(inner_size.clamp(1, 8).into())
                )
            };
        }
        Ty::Tuple(items) => items.iter().map(|t| size(t, lookup)).collect(),
        Ty::Record(fields) => fields.iter().map(|(_, t)| size(t, lookup)).collect(),
        Ty::Named { name, args } if args.is_empty() => match lookup(name).as_deref() {
            Some(Def::Struct { fields, generics, .. }) if generics.is_empty() => {
                fields.iter().map(|f| size(&f.ty, lookup)).collect()
            }
            _ => None,
        },
        _ => None,
    };
    let Some(sizes) = sizes.filter(|s| s.len() > 1) else {
        return String::new();
    };
    let layout = struct_layout(&sizes);
    let at: Vec<String> = layout.offsets.iter().map(u64::to_string).collect();
    let padding: u64 = (0..=sizes.len()).map(|i| layout.padding_before(i, &sizes)).sum();
    let mut note = format!(" (fields at {}", at.join(", "));
    if padding > 0 {
        note += &format!("; {} padding", human_bytes(padding.into()));
    }
    note + ")"
}

/// For a type with no fixed size of its own, such as `List<V>` or a
/// record with a `Str` field: what its fixed-size parts take.
fn held_note(ty: &Ty, lookup: &mut Lookup) -> String {
    unsized_note(ty, lookup).map_or_else(String::new, |note| format!("\n  storage: {note}"))
}

fn unsized_note(ty: &Ty, lookup: &mut Lookup) -> Option<String> {
    match ty {
        Ty::Str => Some("grows on the heap".into()),
        Ty::Named { args, .. } => {
            let held: Vec<String> = args
                .iter()
                .filter_map(|a| size(a, lookup).map(|s| format!("{a}: {}", human_bytes(s.into()))))
                .collect();
            (!held.is_empty()).then(|| format!("grows on the heap; {} each", held.join(", ")))
        }
        Ty::Optional(inner) => unsized_note(inner, lookup),
        Ty::Array { elem, .. } => match size(elem, lookup) {
            Some(per) => Some(format!("{} per element", human_bytes(per.into()))),
            None => element_note(elem, lookup),
        },
        Ty::Tuple(items) => {
            let named: Vec<(String, &Ty)> = items.iter().enumerate().map(|(i, t)| (i.to_string(), t)).collect();
            fields_note(&named, lookup)
        }
        Ty::Record(fields) => {
            let named: Vec<(String, &Ty)> = fields.iter().map(|(n, t)| (n.to_string(), t)).collect();
            fields_note(&named, lookup)
        }
        _ => None,
    }
}

fn element_note(elem: &Ty, lookup: &mut Lookup) -> Option<String> {
    let note = unsized_note(elem, lookup)?;
    Some(match elem {
        Ty::Named { .. } => note,
        Ty::Array { .. } => format!("each row {note}"),
        _ => format!("per element, {note}"),
    })
}

fn fields_note(fields: &[(String, &Ty)], lookup: &mut Lookup) -> Option<String> {
    let parts: Vec<String> = fields
        .iter()
        .map(|(name, ty)| match size(ty, lookup) {
            Some(s) => format!("{name}: {}", human_bytes(s.into())),
            None => match unsized_note(ty, lookup) {
                Some(note) => format!("{name}: {note}"),
                None => format!("{name}: not fixed"),
            },
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}
