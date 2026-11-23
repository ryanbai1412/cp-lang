//! A top-level `let x = x!` rebinds `x` from there on (R02.4.8): the unwrap
//! that ends a fill phase. Each rebinding is a name of its own, `x` and a
//! version, and a read of `x` takes the latest one above it.

use super::*;

const VERSION: char = '\u{1}';

/// Each top-level name's item, rebindings as versions; the rest that are
/// declared twice.
pub(super) fn top_level_names(
    items: &[Arc<ItemSyntax>],
    names: &mut HashMap<String, usize>,
) -> Vec<(usize, Decl)> {
    let mut duplicates: Vec<(usize, Decl)> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        for decl in &item.decls {
            match names.get(&decl.name) {
                None => {
                    names.insert(decl.name.clone(), index);
                }
                Some(&first_idx)
                    if decl.kind == DeclKind::Variant
                        && items[first_idx]
                            .decls
                            .iter()
                            .any(|d| d.name == decl.name && d.kind == DeclKind::Variant) => {}
                Some(_) if rebind::rebinds_itself(item, &decl.name) => {
                    add(names, &decl.name, index)
                }
                Some(_) => duplicates.push((index, decl.clone())),
            }
        }
    }
    duplicates
}

/// True if `item` is `let name = name!`.
pub(super) fn rebinds_itself(item: &ItemSyntax, name: &str) -> bool {
    let is_name = |e: &ast::Expr| matches!(&e.kind, ast::ExprKind::Name(n) if n == name);
    item.parsed.stmts.iter().any(|stmt| match &stmt.kind {
        ast::StmtKind::Let {
            mutable: false,
            pat,
            init: Some(init),
            ..
        } => {
            matches!(&pat.kind, ast::PatternKind::Name(n) if n == name)
                && matches!(&init.kind, ast::ExprKind::Unwrap(inner) if is_name(inner))
        }
        _ => false,
    })
}

/// Records the item at `index` as the next rebinding of `name`.
pub(super) fn add(names: &mut HashMap<String, usize>, name: &str, index: usize) {
    let next = (1..).find(|&k| !names.contains_key(&versioned(name, k)));
    names.insert(versioned(name, next.unwrap_or(1)), index);
}

fn versioned(name: &str, k: usize) -> String {
    format!("{name}{VERSION}{k}")
}

/// The name a key defines: `x` for every rebinding of `x`.
pub(super) fn base(key: &str) -> &str {
    key.split(VERSION).next().unwrap_or(key)
}

pub(super) fn is_version(key: &str) -> bool {
    key.contains(VERSION)
}

impl FileCtx<'_> {
    /// The key `name` means where it is read, in the item being defined or
    /// checked: its latest rebinding above that item, else `name` itself.
    pub(super) fn rebound(&self, name: &str) -> String {
        let reader = self
            .in_progress
            .last()
            .and_then(|n| self.scope.names.get(n));
        let Some(&at) = reader.or(self.current.as_ref()) else {
            return name.to_string();
        };
        let mut found = name.to_string();
        for k in 1.. {
            match self.scope.names.get(&versioned(name, k)) {
                Some(&index) if index < at => found = versioned(name, k),
                _ => break,
            }
        }
        found
    }
}
