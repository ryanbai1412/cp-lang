//! `counter_by_map` (L0579): `cnt[x] = (cnt.get(x) ?? 0) + d` counts by
//! hand; `Counter<K>` (`std.hashmap`) reads 0 for a missing key. The map is
//! labelled so the fact table keeps the finding only on a `Map` or
//! `HashMap` of `Int` (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, COUNTER_BY_MAP};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{AssignOp, BinOp, Expr, ExprKind, Stmt, StmtKind};
use std::collections::HashSet;

use super::shapes::{index1, int, name};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Counts<'a> {
        src: &'a str,
        base: u32,
        seen: HashSet<String>,
        out: Vec<Finding>,
    }
    impl Visitor for Counts<'_> {
        fn stmts(&mut self, stmts: &[Stmt]) {
            for s in stmts {
                self.bump(s);
            }
        }
    }
    impl Counts<'_> {
        fn bump(&mut self, stmt: &Stmt) {
            let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            else {
                return;
            };
            let Some((map, key)) = index1(target) else {
                return;
            };
            let ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs: delta,
                ..
            } = &value.kind
            else {
                return;
            };
            let ExprKind::Binary {
                op: BinOp::Coalesce,
                lhs: got,
                rhs: zero,
                ..
            } = &unparen(lhs).kind
            else {
                return;
            };
            let ExprKind::Call { callee, args } = &got.kind else {
                return;
            };
            let ExprKind::Field { base, name: get } = &callee.kind else {
                return;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let same_key =
                matches!(&args[..], [a] if a.name.is_none() && text(&a.value) == text(key));
            if get.text != "get" || name(base) != Some(map) || int(zero) != Some(0) || !same_key {
                return;
            }
            if !self.seen.insert(map.to_string()) {
                return;
            }
            let bump = match int(delta) {
                Some(1) => format!("{map}.push({})", text(key)),
                _ => format!("{map}.add({}, {})", text(key), text(delta)),
            };
            let diag = Diagnostic::warning(
                COUNTER_BY_MAP.code,
                stmt.span.shift(self.base),
                format!("`{map}` counts by hand: it is a `Counter`"),
            )
            .with_label(base.span.shift(self.base), "the map")
            .with_help(format!(
                "declare `var {map} = Counter<K>(0)` (`std.hashmap`) and write `{bump}`: a key it never saw reads 0, so no update repeats `?? 0`"
            ));
            self.out.push(Finding {
                lint: &COUNTER_BY_MAP,
                diag,
            });
        }
    }
    let mut v = Counts {
        src: file.src,
        base: 0,
        seen: HashSet::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
