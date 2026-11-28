//! Find-references and rename.
//!
//! A use is a reference to a declaration when go-to-definition takes it
//! there, so this agrees with the checker on scopes, shadowing, fields
//! and imports by construction. Only names that share the declaration's
//! text are asked, which keeps a request to a handful of lookups.

use cpl_diag::Span;
use cpl_facade::Definition;
use cpl_facade::Session;
use cpl_syntax::ast::StmtKind;

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
    let Some(target) = db.definition(path, offset) else {
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
            db.definition(path, span.start)
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

/// The spans in `path` where a name is also its argument's label,
/// `f(best:)`, each with its colon: renaming one keeps the label,
/// `best: best_rn`.
pub fn shorthand_args(db: &mut Session, path: &str) -> Vec<(Span, Span)> {
    use cpl_syntax::ast::{Expr, ExprKind};
    struct Labels(Vec<Span>);
    impl cpl_syntax::visit::Visitor for Labels {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { args, .. } = &expr.kind {
                let short = args.iter().filter(|a| {
                    matches!(&a.value.kind, ExprKind::Name(_))
                        && a.name.as_ref().is_some_and(|n| n.span == a.value.span)
                });
                self.0.extend(short.map(|a| a.value.span));
            }
        }
    }
    let Some(analysis) = db.analyze(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in &analysis.items {
        let mut labels = Labels(Vec::new());
        for stmt in &item.syntax.parsed.stmts {
            cpl_syntax::visit::walk_stmt(stmt, &mut labels);
        }
        for span in labels.0 {
            let span = span.shift(item.offset);
            let rest = &analysis.text[span.end as usize..];
            let gap = rest.len() - rest.trim_start().len();
            if rest[gap..].starts_with(':') {
                out.push((span, Span::new(span.start, span.end + gap as u32 + 1)));
            }
        }
    }
    out
}

/// Whether `target` is a variant the input spells: a variant without a
/// `tag` of an enum `read` parses (`read<[t]Cmd>()`, through structs and
/// aliases), whose name is the token the input writes for it.
pub fn spelled_by_input(db: &mut Session, target: &Definition, files: &[&str]) -> bool {
    use cpl_syntax::ast::{Expr, ExprKind, StmtKind, Type, TypeKind};
    struct Reads(Vec<Type>);
    impl cpl_syntax::visit::Visitor for Reads {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Generic { base, args } = &expr.kind {
                if matches!(&base.kind, ExprKind::Name(n) if n == "read") {
                    self.0.extend(args.iter().cloned());
                }
            }
        }
    }
    let mut owner = None;
    let mut inside: Vec<(String, Vec<Type>)> = Vec::new();
    let mut reads = Reads(Vec::new());
    let paths = std::iter::once(target.path.as_str()).chain(files.iter().copied());
    for path in paths {
        let Some(analysis) = db.analyze(path) else {
            continue;
        };
        let here = same_file(path, &target.path);
        for item in &analysis.items {
            for stmt in &item.syntax.parsed.stmts {
                cpl_syntax::visit::walk_stmt(stmt, &mut reads);
                let (name, tys): (_, Vec<Type>) = match &stmt.kind {
                    StmtKind::Enum(decl) => {
                        let spelled = decl.variants.iter().any(|v| {
                            v.tag.is_none() && v.name.span.shift(item.offset) == target.span
                        });
                        if here && spelled {
                            owner = Some(decl.name.text.clone());
                        }
                        let fields = decl.variants.iter().flat_map(|v| &v.fields);
                        (&decl.name, fields.map(|(_, t)| t.clone()).collect())
                    }
                    StmtKind::Struct(decl) => (
                        &decl.name,
                        decl.fields.iter().map(|f| f.ty.clone()).collect(),
                    ),
                    StmtKind::TypeAlias { name, ty, .. } => (name, vec![ty.clone()]),
                    _ => continue,
                };
                if !inside.iter().any(|(held, _)| *held == name.text) {
                    inside.push((name.text.clone(), tys));
                }
            }
        }
    }
    let Some(owner) = owner else {
        return false;
    };
    let (mut todo, mut seen) = (reads.0, std::collections::HashSet::new());
    while let Some(ty) = todo.pop() {
        match ty.kind {
            TypeKind::Named { name, args } => {
                if name.text == owner {
                    return true;
                }
                if seen.insert(name.text.clone()) {
                    let held = inside.iter().find(|(held, _)| *held == name.text);
                    todo.extend(held.into_iter().flat_map(|(_, tys)| tys.iter().cloned()));
                }
                todo.extend(args);
            }
            TypeKind::Array { index, elem } => {
                todo.extend(index);
                todo.push(*elem);
            }
            TypeKind::OwnDomain { elem: t, .. } | TypeKind::Optional(t) => todo.push(*t),
            TypeKind::Tuple(tys) => todo.extend(tys),
            TypeKind::Record(fields) => todo.extend(fields.into_iter().map(|(_, t)| t)),
            _ => {}
        }
    }
    false
}

/// What std type calls the method `target`, `name`, on the user's type
/// (`LiChao` calls a line's `at`): renaming it would break that call, as
/// lowering the program renamed tells.
pub fn std_calls(db: &mut Session, path: &str, target: &Definition, name: &str) -> Option<String> {
    let text = db.file_text(path)?;
    let analysis = db.analyze(path)?;
    let is_method = analysis.items.iter().any(|item| {
        let methods = item
            .syntax
            .parsed
            .stmts
            .iter()
            .flat_map(|stmt| match &stmt.kind {
                StmtKind::Struct(decl) => decl.methods.as_slice(),
                StmtKind::Enum(decl) => decl.methods.as_slice(),
                _ => &[],
            });
        let mut fns = methods.filter_map(|m| match &m.kind {
            StmtKind::Fn(f) => Some(f.name.span.shift(item.offset)),
            _ => None,
        });
        fns.any(|span| span == target.span)
    });
    if !same_file(&target.path, path) || !is_method {
        return None;
    }
    let gap = |d: &cpl_diag::Diagnostic| d.notes.iter().any(|n| n.contains("the gap is in `std."));
    let lowered = |db: &mut Session| {
        let diagnostics = db.diagnostics(path, cpl_facade::Layer::Program);
        let gaps = diagnostics.into_iter().filter(gap);
        gaps.map(|d| d.message).collect::<Vec<_>>()
    };
    let before = lowered(db);
    let trial = format!("{name}_");
    let spans = uses_in(db, path, target, name);
    let edited = cpl_diag::splice(&text, spans.iter().map(|&s| (s, trial.clone())).collect())?;
    let after = lowered(&mut db.with_file(path, edited));
    let new = after.into_iter().find(|m| !before.contains(m))?;
    Some(
        new.split(" does not work")
            .next()
            .unwrap_or("std")
            .to_string(),
    )
}

/// The edit renaming `old` to `new` at `span`.
pub fn edit_at(span: Span, labels: &[(Span, Span)], old: &str, new: &str) -> (Span, String) {
    match labels.iter().find(|(name, _)| *name == span) {
        Some((_, with_colon)) => (*with_colon, format!("{old}: {new}")),
        None => (span, new.to_string()),
    }
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
    analysis: &cpl_facade::today::Analysis,
    path: &str,
    offset: u32,
    name: &str,
) -> Option<(Definition, Vec<Span>)> {
    let cursor = |offset| crate::complete::Cursor {
        uri: path,
        analysis,
        offset,
        fit: None,
    };
    let type_name = |dot: u32| match crate::complete::receiver_type(&cursor(dot), dot)? {
        cpl_facade::today::Ty::Named { name, .. } => Some(name),
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
#[cfg(not(target_arch = "wasm32"))]
pub fn project_files(uri: &str, name: &str) -> Vec<String> {
    let Some(path) = crate::server::path_of_uri(uri) else {
        return Vec::new();
    };
    let Some(dir) = std::path::Path::new(&path).parent() else {
        return Vec::new();
    };
    let root = cpl_config::project_dir(dir).unwrap_or_else(|| dir.to_path_buf());
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

/// A browser has no project on disk: only its open files.
#[cfg(target_arch = "wasm32")]
pub fn project_files(_: &str, _: &str) -> Vec<String> {
    Vec::new()
}

/// Why replacing `spans` of `old` with `new` in each file would make some
/// name resolve to a different declaration than it does now: two locals
/// merged, a use captured by a nearer binding, a declaration shadowed. The
/// files are re-checked with the edits applied and restored afterwards.
pub fn changed_meaning(
    db: &mut Session,
    edits: &[(String, Vec<(Span, String)>)],
    old: &str,
    new: &str,
) -> Option<String> {
    // Where the name at `at` sits once the edits are made: each edit
    // before it shifts it, and an edit at it ends in the new name.
    let moved = |file: &str, at: u32| -> u32 {
        let spans = edits
            .iter()
            .find(|(f, _)| same_file(f, file))
            .map_or(&[][..], |(_, s)| s.as_slice());
        let mut to = at as i64;
        for (span, text) in spans.iter().filter(|(s, _)| s.start <= at) {
            if span.start == at {
                to += text.len() as i64 - new.len() as i64;
            } else {
                to += text.len() as i64 - (span.end - span.start) as i64;
            }
        }
        to as u32
    };
    let mut expected = Vec::new();
    for (file, _) in edits {
        let Some(text) = db.file_text(file) else {
            continue;
        };
        for at in words(&text, old).chain(words(&text, new)) {
            if let Some(def) = db.definition(file, at) {
                let def_at = moved(&def.path, def.span.start);
                expected.push((file.clone(), at, moved(file, at), def.path, def_at));
            }
        }
    }
    // An open library is also in the database under the path its importers
    // read it by.
    let mut trial = db.clone();
    for (file, spans) in edits {
        let alias = crate::server::path_of_uri(file).filter(|p| p != file);
        for path in std::iter::once(file.clone()).chain(alias) {
            let Some(text) = db.file_text(&path) else {
                continue;
            };
            let words = spans.to_vec();
            let Some(edited) = cpl_diag::splice(&text, words) else {
                continue;
            };
            trial = trial.with_file(&path, edited);
        }
    }
    let broken = expected.into_iter().find(|(file, _, at, path, def_at)| {
        trial
            .definition(file, *at)
            .is_none_or(|d| !same_file(&d.path, path) || d.span.start != *def_at)
    });
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
