//! Structs of one name and different arities (R02.11.4): the file's first
//! keeps the name, each other is keyed `Prefix'3`, and so is an impl of the
//! file's for it.

use super::{decls_of, ItemSyntax};
use cpl_syntax::ast::{StmtKind, TypeKind};
use std::sync::Arc;

pub(super) fn key_overloads(items: Vec<Arc<ItemSyntax>>) -> Vec<Arc<ItemSyntax>> {
    let mut first: crate::names::NameMap<usize> = Default::default();
    let mut keyed: Vec<(String, usize)> = Vec::new();
    for item in &items {
        for stmt in &item.parsed.stmts {
            if let StmtKind::Struct(s) = &stmt.kind {
                let n = s.generics.len();
                match first.get(s.name.text.as_str()) {
                    None => {
                        first.insert(s.name.text.clone(), n);
                    }
                    Some(&m) if m != n && !keyed.contains(&(s.name.text.clone(), n)) => {
                        keyed.push((s.name.text.clone(), n));
                    }
                    Some(_) => {}
                }
            }
        }
    }
    if keyed.is_empty() {
        return items;
    }
    let is_keyed = |name: &str, n: usize| keyed.iter().any(|(k, m)| k == name && *m == n);
    let mut seen: Vec<(String, usize)> = Vec::new();
    items
        .into_iter()
        .map(|item| {
            let mut parsed = item.parsed.clone();
            let mut changed = false;
            for stmt in &mut parsed.stmts {
                match &mut stmt.kind {
                    StmtKind::Struct(s) => {
                        let n = s.generics.len();
                        let once = (s.name.text.clone(), n);
                        if is_keyed(&s.name.text, n) && !seen.contains(&once) {
                            s.name.text = crate::prelude::arity_key(&s.name.text, n);
                            changed = true;
                        }
                        seen.push(once);
                    }
                    StmtKind::Impl { target, .. } => {
                        if let TypeKind::Named { name, args } = &mut target.kind {
                            if is_keyed(&name.text, args.len()) {
                                name.text = crate::prelude::arity_key(&name.text, args.len());
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !changed {
                return item;
            }
            Arc::new(ItemSyntax {
                text: item.text.clone(),
                col: item.col,
                tail: item.tail,
                hash: item.hash ^ 0x2545_f491_4f6c_dd1d,
                decls: decls_of(&parsed.stmts),
                flow: item.flow.clone(),
                has_use: item.has_use,
                parsed,
            })
        })
        .collect()
}
