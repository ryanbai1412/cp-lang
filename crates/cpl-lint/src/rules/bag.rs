//! `map_as_multiset` (L0563): C++'s `map<int, int> cnt; cnt[x]++; if
//! (cnt[x] > 0) cnt[x]--;`, a count per key kept by hand. `SortedBag`
//! (`std.bag`) is that multiset, with `remove` that is false when absent and
//! `first`/`last` for the smallest and largest. A map counts as a bag when its
//! entries step by one both ways and a decrement is guarded by the count. The map is labelled so the fact table keeps the
//! finding only on a `Map<K, Int>` (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, MAP_AS_MULTISET};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{AssignOp, BinOp, CmpOp, Expr, ExprKind, Stmt, StmtKind};
use std::collections::HashMap;

use super::shapes::{compare, int, name};

#[derive(Default)]
struct Uses {
    /// The first `m[k] += 1` and its `m`.
    up: Option<(Span, Span)>,
    down: bool,
    /// `m[k] > 0`: the counts are read as a bag's.
    bag: bool,
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Counts {
        base: u32,
        maps: HashMap<String, Uses>,
        order: Vec<String>,
    }
    impl Counts {
        fn uses(&mut self, m: &str) -> &mut Uses {
            if !self.maps.contains_key(m) {
                self.order.push(m.to_string());
            }
            self.maps.entry(m.to_string()).or_default()
        }
    }
    impl Visitor for Counts {
        fn stmt(&mut self, stmt: &Stmt) {
            let StmtKind::Assign {
                target,
                op: AssignOp::Binary(op @ (BinOp::Add | BinOp::Sub)),
                value,
            } = &stmt.kind
            else {
                return;
            };
            let Some(m) = entry_of(target) else { return };
            if int(value) != Some(1) {
                return;
            }
            let (span, base) = (stmt.span.shift(self.base), m.span.shift(self.base));
            let uses = self.uses(name(m).unwrap_or_default());
            match op {
                BinOp::Add => {
                    uses.up.get_or_insert((span, base));
                }
                _ => uses.down = true,
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let Some((lhs, op, rhs)) = compare(expr) {
                let positive = matches!(
                    (op, int(rhs)),
                    (CmpOp::Gt | CmpOp::Ne, Some(0)) | (CmpOp::Ge, Some(1))
                );
                if let (true, Some(m)) = (positive, entry_of(lhs)) {
                    self.uses(name(m).unwrap_or_default()).bag = true;
                }
            }
        }
    }
    let mut v = Counts {
        base: 0,
        maps: HashMap::new(),
        order: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let mut out = Vec::new();
    for m in &v.order {
        let uses = &v.maps[m];
        let (Some((span, label)), true, true) = (uses.up, uses.down, uses.bag) else {
            continue;
        };
        let diag = Diagnostic::warning(
            MAP_AS_MULTISET.code,
            span,
            format!("`{m}` counts the copies of each key by hand: it is a multiset"),
        )
        .with_label(label, "the map")
        .with_help(format!(
            "declare `var {m} = SortedBag<K>()` (`std.bag`, in the prelude) and write `{m}.add(x)` / `{m}.remove(x)`: a bag keeps the counts, `remove` is false when `x` is absent, and `first`/`last` read the smallest and largest"
        ));
        out.push(Finding {
            lint: &MAP_AS_MULTISET,
            diag,
        });
    }
    out
}

/// `m` in `m[k]`, when `m` is a name and there is one index.
fn entry_of(e: &Expr) -> Option<&Expr> {
    let ExprKind::Index { base, indices } = &unparen(e).kind else {
        return None;
    };
    (indices.len() == 1 && name(base).is_some()).then_some(&**base)
}
