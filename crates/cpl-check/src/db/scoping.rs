//! Where a definition is found and when it runs: an inline module names its
//! siblings (`use a.f` inside `mod b:`, R11.1.3, R11.2.2), and a `domain`
//! without compile-time bounds runs where it is written, like a `let`
//! (R11.1.1).

use super::*;
use cpl_syntax::const_int;

impl Database {
    /// The path and text of `parts` seen from `from`: a sibling inline
    /// module first, then whatever the loader finds.
    pub(super) fn find_module(&self, from: &str, parts: &[String]) -> Option<(String, String)> {
        if let ([name], Some((file, _))) = (parts, from.rsplit_once("::mod::")) {
            let path = format!("{file}::mod::{name}");
            if let Some(state) = self.files.get(&path) {
                return Some((path, state.text.to_string()));
            }
        }
        self.loader.load(from, parts)
    }
}

/// True when item `k` declares a domain whose bounds are not compile-time
/// integers, so it runs where it is written; a constant one is hoisted.
pub(super) fn in_place(items: &[Arc<ItemSyntax>], scope: &Scope, k: usize) -> bool {
    items[k].parsed.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Domain { def, .. } => {
            !const_int::constant_domain(def, &|n| const_value(items, scope, n, 0))
        }
        _ => false,
    })
}

/// The value of the top-level integer `const` called `name`.
fn const_value(items: &[Arc<ItemSyntax>], scope: &Scope, name: &str, depth: usize) -> Option<i64> {
    let &at = scope.names.get(name).filter(|_| depth < 64)?;
    items[at]
        .parsed
        .stmts
        .iter()
        .find_map(|stmt| match &stmt.kind {
            StmtKind::Const {
                name: n,
                generics,
                init: Some(init),
                ..
            } if n.text == name && generics.is_empty() => {
                const_int::eval(init, &|m| const_value(items, scope, m, depth + 1))
            }
            _ => None,
        })
}

/// R11.3.4: each `pub fn f(self: ..)` of a library module is a method its
/// receivers find, under its `method_key`, whatever else the file calls `f`.
pub(super) fn add_methods(scope: &mut Scope, index: usize) {
    let module = &scope.modules[index];
    if module.name.starts_with("std.") {
        return;
    }
    let mut found = Vec::new();
    for item in &module.analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            let StmtKind::Fn(f) = &stmt.kind else {
                continue;
            };
            let public = item
                .syntax
                .decls
                .iter()
                .any(|d| d.public && d.name == f.name.text);
            if public && f.params.first().is_some_and(|p| p.name.text == "self") {
                let key = crate::prelude::method_key(&f.name.text, &module.name);
                found.push((key, f.name.text.clone()));
            }
        }
    }
    for (key, name) in found {
        scope.imports.entry(key).or_insert(Import {
            module: index,
            name,
        });
    }
}
