//! `counter_by_map` (L0579): `cnt[x] = (cnt.get(x) ?? 0) + d`, or a fresh
//! map whose only write is `for x in a: cnt[x] += 1`, counts by hand;
//! `Counter<K>` (`std.hashmap`) reads 0 for a missing key and `a.tally()`
//! is the whole count. The map is
//! labelled so the fact table keeps the finding only on a `Map` or
//! `HashMap` of `Int` (`typed.rs`).

use crate::walk::{unparen, walk_stmt, Visitor};
use crate::{File, Finding, COUNTER_BY_MAP};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{AssignOp, BinOp, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind};
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
            let block: Vec<_> = stmts.iter().map(|s| (self.base, s)).collect();
            self.fill(&block);
        }
    }
    impl Counts<'_> {
        /// `for x in a: m[x] += 1` as the only write to a map made empty
        /// earlier in the block, and no read that depends on key order.
        /// Top-level statements are one item each, so they come with their offsets.
        fn fill(&mut self, stmts: &[(u32, &Stmt)]) {
            for (i, &(at, stmt)) in stmts.iter().enumerate() {
                let StmtKind::For {
                    pat,
                    iter,
                    body,
                    else_body: None,
                    ..
                } = &stmt.kind
                else {
                    continue;
                };
                let (PatternKind::Name(var), [bump]) = (&pat.kind, &body.stmts[..]) else {
                    continue;
                };
                let StmtKind::Assign {
                    target,
                    op: AssignOp::Binary(BinOp::Add),
                    value,
                } = &bump.kind
                else {
                    continue;
                };
                let (Some((map, key)), ExprKind::Index { base, .. }) =
                    (index1(target), &unparen(target).kind)
                else {
                    continue;
                };
                if name(key) != Some(var.as_str()) || int(value) != Some(1) {
                    continue;
                }
                let Some(made) = stmts[..i].iter().rposition(|&(_, s)| empty_map(s, map)) else {
                    continue;
                };
                let mut uses = Uses {
                    map,
                    other: false,
                    names: 0,
                    indexed: 0,
                    fields: 0,
                    walked: 0,
                };
                for (k, &(_, s)) in stmts.iter().enumerate().skip(made + 1) {
                    if k != i {
                        walk_stmt(&mut uses, s);
                    }
                }
                let whole = uses.names > uses.indexed + uses.fields;
                if uses.other
                    || whole
                    || uses.fields > uses.walked
                    || !self.seen.insert(map.to_string())
                {
                    continue;
                }
                let text = |e: &Expr| e.span.shift(at).text(self.src);
                let help = if made + 1 == i {
                    format!(
                        "write `let {map} = {}.tally()` (`std.seq`): a `Counter` of each item, which reads 0 for a key it never saw",
                        text(iter)
                    )
                } else {
                    format!(
                        "declare `var {map} = Counter<K>()` (`std.hashmap`) and write `{map}.push({var})`: hashed, and a key it never saw reads 0"
                    )
                };
                let diag = Diagnostic::warning(
                    COUNTER_BY_MAP.code,
                    stmt.span.shift(at),
                    format!("`{map}` counts by hand: it is a `Counter`"),
                )
                .with_label(base.span.shift(at), "the map")
                .with_help(help);
                self.out.push(Finding {
                    lint: &COUNTER_BY_MAP,
                    diag,
                });
            }
        }

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
                "declare `var {map} = Counter<K>()` (`std.hashmap`) and write `{bump}`: a key it never saw reads 0, so no update repeats `?? 0`"
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
    let top: Vec<_> = file.top_level().collect();
    v.fill(&top);
    v.out
}

/// `var map = Map()` or `var map: Map<K, Int> = Map()` (or `HashMap`).
fn empty_map(stmt: &Stmt, map: &str) -> bool {
    let StmtKind::Let {
        mutable: true,
        pat,
        init: Some(init),
        ..
    } = &stmt.kind
    else {
        return false;
    };
    let ExprKind::Call { callee, args } = &init.kind else {
        return false;
    };
    matches!(&pat.kind, PatternKind::Name(n) if n == map)
        && args.is_empty()
        && matches!(name(callee), Some("Map" | "HashMap"))
}

/// Whether the statements walked use `map` as more than a counter: a
/// method or field other than one a `for` walks (`for (k, c) in m.items`),
/// a write other than the fill, or the map whole (`print(m)` prints a
/// `Counter` in another order). An ordered read such as `.first` is a field.
struct Uses<'a> {
    map: &'a str,
    other: bool,
    names: usize,
    indexed: usize,
    fields: usize,
    walked: usize,
}

impl Uses<'_> {
    fn of(&self, e: &Expr) -> bool {
        matches!(&unparen(e).kind, ExprKind::Field { base, .. } if name(base) == Some(self.map))
    }
}

impl Visitor for Uses<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            let base = index1(target).map(|(m, _)| m).or(name(target));
            self.other |= base == Some(self.map);
        }
    }
    fn for_loop(&mut self, _pat: &Pattern, iter: &Expr) {
        self.walked += usize::from(self.of(iter));
    }
    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Name(n) if n == self.map => self.names += 1,
            ExprKind::Index { base, .. } if name(base) == Some(self.map) => self.indexed += 1,
            ExprKind::Field { base, .. } if name(base) == Some(self.map) => self.fields += 1,
            ExprKind::Comprehension { clauses, .. } => {
                let walked = clauses.iter().filter(|c| self.of(&c.iter)).count();
                self.walked += walked;
            }
            _ => {}
        }
    }
}
