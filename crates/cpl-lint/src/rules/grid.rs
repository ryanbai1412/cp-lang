//! `nested_list_grid` (L0532): `[[c for _ in 0..m] for _ in 0..n]`, C++'s
//! `vector<vector<T>>` for a fixed rectangle; `[n, m]T = c` is one block
//! whose domain proves each `g[i, j]`.

use crate::walk::Visitor;
use crate::{File, Finding, NESTED_LIST_GRID};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{Clause, Expr, ExprKind, PatternKind, Stmt, StmtKind, TypeKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Grids<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Grids<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            let StmtKind::Let {
                pat,
                ty,
                init: Some(init),
                ..
            } = &stmt.kind
            else {
                return;
            };
            let PatternKind::Name(name) = &pat.kind else {
                return;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src).to_string();
            let ExprKind::Comprehension { body: row, clauses } = &init.kind else {
                return;
            };
            let [outer] = clauses.as_slice() else {
                return;
            };
            let ExprKind::Comprehension {
                body: cell,
                clauses,
            } = &row.kind
            else {
                return;
            };
            let [inner] = clauses.as_slice() else {
                return;
            };
            let bound = |c: &Clause| match &c.pat.kind {
                PatternKind::Name(n) => Some(n.clone()),
                _ => None,
            };
            let (Some(rows), Some(cols)) = (dim(&text(&outer.iter)), dim(&text(&inner.iter)))
            else {
                return;
            };
            let fill = text(cell);
            let uses = |n: &Option<String>| {
                n.as_ref().is_some_and(|n| {
                    cols.contains(n.as_str())
                        || fill
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .any(|w| w == n)
                })
            };
            if !outer.conds.is_empty()
                || !inner.conds.is_empty()
                || uses(&bound(outer))
                || uses(&bound(inner))
            {
                return;
            }
            let elem = match ty.as_ref().map(|t| &t.kind) {
                Some(TypeKind::Named { name, args }) if name.text == "List" => match &args[..] {
                    [inner] => match &inner.kind {
                        TypeKind::Named { name, args }
                            if name.text == "List" && args.len() == 1 =>
                        {
                            args[0].span.shift(self.base).text(self.src).to_string()
                        }
                        _ => return,
                    },
                    _ => return,
                },
                None if matches!(cell.kind, ExprKind::Int(_)) => "Int".into(),
                _ => return,
            };
            let kw = if matches!(stmt.kind, StmtKind::Let { mutable: true, .. }) {
                "var"
            } else {
                "let"
            };
            let span = init.span.shift(self.base);
            let diag = Diagnostic::warning(
                NESTED_LIST_GRID.code,
                span,
                format!("`{name}` is a list of lists for a fixed {rows} by {cols} grid"),
            )
            .with_help(format!(
                "declare `{kw} {name}: [{rows}, {cols}]{elem} = {fill}` and index `{name}[i, j]`: one block instead of a list per row, and its domain proves each index"
            ));
            self.out.push(Finding {
                lint: &NESTED_LIST_GRID,
                diag,
            });
        }
    }
    let mut v = Grids {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// The array dimension a comprehension's range gives: `0..n` is `n`, any
/// other range is itself (`1..=n`).
fn dim(iter: &str) -> Option<String> {
    if !iter.contains("..") || iter.contains(" by ") {
        return None;
    }
    Some(match iter.strip_prefix("0..") {
        Some(hi) if !hi.starts_with('=') => hi.to_string(),
        _ => iter.to_string(),
    })
}
