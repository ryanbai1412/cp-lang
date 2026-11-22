//! `redundant_prelude_import`: a `use` of a name the prelude already gives
//! every program, such as `use std.tree.{Rooted}`.
//!
//! The checker brings in a prelude name wherever a file uses it and neither
//! declares nor imports its own, so dropping the `use` changes nothing. A
//! library (a file with `pub` items) is skipped: a name only its `@test`
//! items use is not brought in when another file imports it.

use crate::{File, Finding, REDUNDANT_PRELUDE_IMPORT};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::StmtKind;
use std::collections::HashSet;
use std::sync::OnceLock;

/// `(module, name)` for each name `spec/std/prelude.cp` lists, the module
/// written `std.tree`.
fn prelude() -> &'static HashSet<(String, String)> {
    static NAMES: OnceLock<HashSet<(String, String)>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let text = include_str!("../../../../spec/std/prelude.cp");
        let mut found = HashSet::new();
        for (_, item) in cpl_syntax::parse_file(text).items {
            for stmt in item.stmts {
                if let StmtKind::Use { path, names, .. } = stmt.kind {
                    let module = dotted(&path);
                    found.extend(names.iter().map(|n| (module.clone(), n.text.clone())));
                }
            }
        }
        found
    })
}

fn dotted(path: &[cpl_syntax::ast::Name]) -> String {
    path.iter()
        .map(|n| n.text.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    if file.top_level().any(|(_, stmt)| stmt.public) {
        return Vec::new();
    }
    let declared: HashSet<&str> = file
        .top_level()
        .filter_map(|(_, stmt)| match &stmt.kind {
            StmtKind::Fn(decl) => Some(decl.name.text.as_str()),
            StmtKind::Struct(decl) => Some(decl.name.text.as_str()),
            StmtKind::Enum(decl) => Some(decl.name.text.as_str()),
            StmtKind::Domain { name, .. }
            | StmtKind::TypeAlias { name, .. }
            | StmtKind::Trait { name, .. }
            | StmtKind::Const { name, .. } => Some(name.text.as_str()),
            _ => None,
        })
        .collect();
    let redundant = |module: &str, name: &str| {
        !declared.contains(name) && prelude().contains(&(module.to_string(), name.to_string()))
    };
    let mut out = Vec::new();
    let mut run_start: Option<(u32, u32)> = None;
    for (base, stmt) in file.top_level() {
        let StmtKind::Use {
            path,
            names,
            glob: false,
            alias: None,
        } = &stmt.kind
        else {
            continue;
        };
        if !stmt.decorators.is_empty() {
            continue;
        }
        let (module, listed): (String, Vec<&str>) = if names.is_empty() {
            let Some((last, module)) = path.split_last() else {
                continue;
            };
            (dotted(module), vec![last.text.as_str()])
        } else {
            (
                dotted(path),
                names.iter().map(|n| n.text.as_str()).collect(),
            )
        };
        let (dropped, kept): (Vec<&str>, Vec<&str>) =
            listed.iter().partition(|name| redundant(&module, name));
        if dropped.is_empty() {
            continue;
        }
        let span = stmt.span.shift(base);
        let (edit, replacement) = if kept.is_empty() {
            let text = &file.src[span.start as usize..span.end as usize];
            let rest = &file.src[span.end as usize..];
            let end = match text.find("//") {
                Some(comment) => span.start + comment as u32,
                None => {
                    let start = match run_start {
                        Some((start, end)) if end + 1 == span.start => start,
                        _ => span.start,
                    };
                    run_start = Some((start, span.end));
                    let before = &file.src[..start as usize];
                    let blank_before = before.is_empty() || before.ends_with("\n\n");
                    let line = u32::from(rest.starts_with('\n'));
                    let blank_after = rest[line as usize..].starts_with('\n');
                    span.end + line + u32::from(line == 1 && blank_before && blank_after)
                }
            };
            (Span::new(span.start, end), String::new())
        } else {
            let (first, last) = (&names[0].span, &names[names.len() - 1].span);
            (
                Span::new(first.start + base, last.end + base),
                kept.join(", "),
            )
        };
        let quoted: Vec<String> = dropped.iter().map(|n| format!("`{n}`")).collect();
        let mut diag = Diagnostic::warning(
            REDUNDANT_PRELUDE_IMPORT.code,
            span,
            format!(
                "{} is in the prelude: this `use` is not needed",
                quoted.join(", ")
            ),
        )
        .with_help("every program has the prelude's names without a `use`");
        diag.fixes.push(Fix {
            title: "drop the prelude's names from the `use`".into(),
            edits: vec![(edit, replacement)],
        });
        out.push(Finding {
            lint: &REDUNDANT_PRELUDE_IMPORT,
            diag,
        });
    }
    out
}
