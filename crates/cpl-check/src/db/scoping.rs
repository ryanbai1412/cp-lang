//! Where a definition is found and when it runs: an inline module names its
//! siblings (`use a.f` inside `mod b:`, R11.1.3, R11.2.2), and a `domain` or
//! `const` whose value names a run-time value runs where it is written, like
//! a `let` (R11.1.1).

use super::*;

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

/// True when item `k` declares a domain, type or const whose definition names
/// a top-level `let` or `var`.
pub(super) fn in_place(items: &[Arc<ItemSyntax>], scope: &Scope, k: usize) -> bool {
    items[k].parsed.stmts.iter().any(|stmt| {
        let def = match &stmt.kind {
            StmtKind::Domain { def, .. } => def,
            StmtKind::Const {
                init: Some(init), ..
            } => init,
            _ => return false,
        };
        crate::check::lends::names_in(def).iter().any(|name| {
            scope.names.get(name).is_some_and(|&at| {
                at != k
                    && items[at]
                        .decls
                        .iter()
                        .any(|d| d.name == *name && d.kind == DeclKind::Value)
            })
        })
    })
}
