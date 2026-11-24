//! Find-references and rename.
//!
//! A use is a reference to a declaration when go-to-definition takes it
//! there, so this agrees with the checker on scopes, shadowing, fields
//! and imports by construction. Only names that share the declaration's
//! text are asked, which keeps a request to a handful of lookups.

use cpl_check::{Definition, Session};
use cpl_diag::Span;

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The identifier under `offset`, with its span.
pub fn word_at(text: &str, offset: u32) -> Option<(Span, &str)> {
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
    let word = &text[start..end];
    (!word.is_empty() && !word.starts_with(|c: char| c.is_ascii_digit()))
        .then(|| (Span::new(start as u32, end as u32), word))
}

/// The declaration under `offset` and every span in `path` that names it,
/// the declaration first when it is in this file.
pub fn references(db: &mut Session, path: &str, offset: u32) -> Option<(Definition, Vec<Span>)> {
    let analysis = db.analyze(path)?;
    let (_, name) = word_at(&analysis.text, offset)?;
    let Some(target) = db.definition_location(path, offset) else {
        return field_references(&analysis, path, offset, name);
    };
    let spans = uses_in(db, path, &target, name);
    Some((target, spans))
}

/// Every span in `path` that names the declaration at `target`, the
/// declaration first when it is in this file.
pub fn uses_in(db: &mut Session, path: &str, target: &Definition, name: &str) -> Vec<Span> {
    let Some(analysis) = db.analyze(path) else {
        return Vec::new();
    };
    let mut candidates: Vec<Span> = Vec::new();
    let mut spans_of_imports: Vec<Span> = Vec::new();
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
        // `use m.{name}`: the import names the declaration in `m`.
        let imported = analysis.imports().get(name).is_some_and(|import| {
            let module = &analysis.modules()[import.module];
            import.name == name && same_file(&module.path, &target.path)
        });
        for stmt in item.syntax.parsed.stmts.iter().filter(|_| imported) {
            if let cpl_syntax::ast::StmtKind::Use { names, .. } = &stmt.kind {
                let listed = names.iter().filter(|n| n.text == name);
                spans_of_imports.extend(listed.map(|n| at(n.span)));
            }
        }
    }
    let imports = spans_of_imports;
    cpl_diag::sort_by_key(&mut candidates, |s| s.order());
    candidates.dedup();
    let names =
        |span: &Span| analysis.text.get(span.start as usize..span.end as usize) == Some(name);
    let mut spans: Vec<Span> = candidates
        .into_iter()
        .filter(names)
        .filter(|span| {
            db.definition_location(path, span.start)
                .is_some_and(|d| same_place(&d, target))
        })
        .collect();
    spans.extend(imports.into_iter().filter(names));
    let here = same_file(&target.path, path);
    if here && !spans.contains(&target.span) && names(&target.span) {
        spans.push(target.span);
    }
    cpl_diag::sort_by_key(&mut spans, |s| {
        ((!here || s.start != target.span.start) as u64) << 32 | s.start as u64
    });
    spans.dedup();
    spans
}

/// Whether two paths name one file: an import reads a module by its path,
/// the editor opens it by a `file://` URI.
pub fn same_file(a: &str, b: &str) -> bool {
    let file = |p: &str| crate::server::path_of_uri(p).unwrap_or_else(|| p.to_string());
    a == b || file(a) == file(b)
}

fn same_place(a: &Definition, b: &Definition) -> bool {
    a.span == b.span && same_file(&a.path, &b.path)
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

/// The `.cp` files of the project holding `uri` (the folder of the nearest
/// `cp.toml` above it, else its own folder) that import `name`, by text.
pub fn project_files(uri: &str, name: &str) -> Vec<String> {
    let Some(path) = crate::server::path_of_uri(uri) else {
        return Vec::new();
    };
    let Some(dir) = std::path::Path::new(&path).parent() else {
        return Vec::new();
    };
    let root = dir
        .ancestors()
        .find(|d| d.join("cp.toml").is_file())
        .unwrap_or(dir);
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut budget = 5000;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            budget -= 1;
            if budget == 0 {
                return found;
            }
            let path = entry.path();
            let hidden = entry.file_name().to_string_lossy().starts_with('.');
            if hidden
                || ["target", "node_modules", "dist"]
                    .iter()
                    .any(|d| path.ends_with(d))
            {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "cp") {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let imports = text.lines().any(|l| {
                    l.starts_with("use ") && l.split(|c: char| !is_ident(c)).any(|w| w == name)
                });
                if imports {
                    found.push(path.display().to_string());
                }
            }
        }
    }
    found
}

/// Why replacing `spans` of `old` with `new` in each file would make some
/// name resolve to a different declaration than it does now: two locals
/// merged, a use captured by a nearer binding, a declaration shadowed. The
/// files are re-checked with the edits applied and restored afterwards.
pub fn changed_meaning(
    db: &mut Session,
    edits: &[(String, Vec<Span>)],
    old: &str,
    new: &str,
) -> Option<String> {
    let grow = new.len() as i64 - old.len() as i64;
    let moved = |file: &str, at: u32| -> u32 {
        let spans = edits
            .iter()
            .find(|(f, _)| same_file(f, file))
            .map_or(&[][..], |(_, s)| s.as_slice());
        let before = spans.iter().filter(|s| s.start < at).count() as i64;
        (at as i64 + before * grow) as u32
    };
    let mut expected = Vec::new();
    for (file, _) in edits {
        let Some(text) = db.file_text(file) else {
            continue;
        };
        for at in words(&text, old).chain(words(&text, new)) {
            if let Some(def) = db.definition_location(file, at) {
                let def_at = moved(&def.path, def.span.start);
                expected.push((file.clone(), at, moved(file, at), def.path, def_at));
            }
        }
    }
    // An open library is also in the database under the path its importers
    // read it by.
    let mut originals = Vec::new();
    for (file, spans) in edits {
        let alias = crate::server::path_of_uri(file).filter(|p| p != file);
        for path in std::iter::once(file.clone()).chain(alias) {
            let Some(text) = db.file_text(&path) else {
                continue;
            };
            let mut edited = text.to_string();
            for span in spans.iter().rev() {
                edited.replace_range(span.start as usize..span.end as usize, new);
            }
            originals.push((path.clone(), text, db.file_loaded(&path)));
            db.set_file(&path, edited);
        }
    }
    let broken = expected.into_iter().find(|(file, _, at, path, def_at)| {
        db.definition_location(file, *at)
            .is_none_or(|d| !same_file(&d.path, path) || d.span.start != *def_at)
    });
    for (path, text, loaded) in originals {
        db.restore_file(&path, text, loaded);
    }
    let (file, at, ..) = broken?;
    let text = db.file_text(&file)?;
    let line = text[..at as usize].matches('\n').count() + 1;
    let (_, word) = word_at(&text, at)?;
    Some(format!(
        "renaming `{old}` to `{new}` would make `{word}` on line {line} refer to another declaration"
    ))
}

/// The start of each whole-word occurrence of `name` in `text`.
fn words<'t>(text: &'t str, name: &'t str) -> impl Iterator<Item = u32> + 't {
    text.match_indices(name).filter_map(move |(i, _)| {
        let end = i + name.len();
        let open = !text[..i].ends_with(is_ident);
        let close = !text[end..].starts_with(is_ident);
        (open && close).then_some(i as u32)
    })
}

#[cfg(test)]
mod tests {
    use super::word_at;

    #[test]
    fn the_word_after_a_wide_character_is_cut_at_a_character_boundary() {
        let text = "let sΣx = Σ(xs)\n";
        let (span, word) = word_at(text, text.find("(xs").unwrap() as u32 + 2).unwrap();
        assert_eq!(word, "xs");
        assert_eq!(&text[span.start as usize..span.end as usize], "xs");
        assert_eq!(word_at(text, text.rfind('Σ').unwrap() as u32 + 1), None);
    }
}
