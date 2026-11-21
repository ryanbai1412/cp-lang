//! Find-references and rename.
//!
//! A use is a reference to a declaration when go-to-definition takes it
//! there, so this agrees with the checker on scopes, shadowing, fields
//! and imports by construction. Only names that share the declaration's
//! text are asked, which keeps a request to a handful of lookups.

use cpl_check::{Database, Definition};
use cpl_diag::Span;

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The identifier under `offset`, with its span.
pub fn word_at(text: &str, offset: u32) -> Option<(Span, &str)> {
    let at = (offset as usize).min(text.len());
    let start = text[..at]
        .rfind(|c: char| !is_ident(c))
        .map_or(0, |i| i + 1);
    let end = text[at..]
        .find(|c: char| !is_ident(c))
        .map_or(text.len(), |i| at + i);
    let word = &text[start..end];
    (!word.is_empty() && !word.starts_with(|c: char| c.is_ascii_digit()))
        .then(|| (Span::new(start as u32, end as u32), word))
}

/// The declaration under `offset` and every span in `path` that names it,
/// the declaration first when it is in this file.
pub fn references(db: &mut Database, path: &str, offset: u32) -> Option<(Definition, Vec<Span>)> {
    let analysis = db.analyze(path)?;
    let (_, name) = word_at(&analysis.text, offset)?;
    let Some(target) = db.definition_location(path, offset) else {
        return field_references(&analysis, path, offset, name);
    };
    let mut candidates: Vec<Span> = Vec::new();
    for item in &analysis.items {
        let at = |span: Span| Span::new(item.offset + span.start, item.offset + span.end);
        let hovers = item
            .check
            .hovers
            .iter()
            .filter(|e| e.name == name)
            .map(|e| at(e.span));
        let refs = item
            .check
            .refs
            .iter()
            .filter(|r| r.name == name)
            .map(|r| at(r.span));
        let decls = item
            .syntax
            .decls
            .iter()
            .filter(|d| d.name == name)
            .map(|d| at(d.span));
        candidates.extend(hovers.chain(refs).chain(decls));
    }
    candidates.sort_by_key(|s| (s.start, s.end));
    candidates.dedup();
    let mut spans: Vec<Span> = candidates
        .into_iter()
        .filter(|span| analysis.text.get(span.start as usize..span.end as usize) == Some(name))
        .filter(|span| db.definition_location(path, span.start).as_ref() == Some(&target))
        .collect();
    if target.path == path && !spans.contains(&target.span) {
        spans.push(target.span);
    }
    spans.sort_by_key(|s| (s.start != target.span.start || target.path != path, s.start));
    Some((target, spans))
}

/// Why `name` cannot be the new name, if it cannot.
pub fn invalid_name(name: &str) -> Option<String> {
    if name.is_empty()
        || !name.chars().all(is_ident)
        || name.starts_with(|c: char| c.is_ascii_digit())
    {
        return Some(format!("`{name}` is not a name"));
    }
    cpl_syntax::KEYWORDS
        .contains(&name)
        .then(|| format!("`{name}` is a keyword"))
}

/// `r.w`: go-to-definition does not follow fields, so a field's uses are
/// the accesses of that name on a value of the same type, and its
/// declaration is the struct's field of that name.
fn field_references(
    analysis: &cpl_check::Analysis,
    path: &str,
    offset: u32,
    name: &str,
) -> Option<(Definition, Vec<Span>)> {
    let cursor = |offset| crate::complete::Cursor {
        uri: path,
        analysis,
        offset,
    };
    let type_name = |dot: u32| match crate::complete::receiver_type(&cursor(dot), dot)? {
        cpl_check::Ty::Named { name, .. } => Some(name),
        _ => None,
    };
    let (span, _) = word_at(&analysis.text, offset)?;
    let owner = analysis.text[..span.start as usize]
        .ends_with('.')
        .then(|| type_name(span.start - 1))
        .flatten();
    // The field's declaration, and the struct it belongs to.
    let mut found = None;
    for item in &analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            let cpl_syntax::ast::StmtKind::Struct(s) = &stmt.kind else {
                continue;
            };
            let Some(field) = s.fields.iter().find(|f| f.name.text == name) else {
                continue;
            };
            let at = Span::new(
                item.offset + field.name.span.start,
                item.offset + field.name.span.end,
            );
            let here = match &owner {
                Some(owner) => *owner == s.name.text,
                None => at == span,
            };
            if here {
                found = Some((s.name.text.clone(), at));
            }
        }
    }
    let (owner, decl) = found?;
    let mut spans = vec![decl];
    for item in &analysis.items {
        for entry in item
            .check
            .hovers
            .iter()
            .filter(|e| e.keyword == "field" && e.name == name)
        {
            let start = item.offset + entry.span.start;
            let on_value = analysis.text[..start as usize].ends_with('.');
            if on_value && type_name(start - 1).as_deref() == Some(owner.as_str()) {
                spans.push(Span::new(start, item.offset + entry.span.end));
            }
        }
    }
    spans.dedup();
    let target = Definition {
        path: path.to_string(),
        span: decl,
    };
    Some((target, spans))
}
