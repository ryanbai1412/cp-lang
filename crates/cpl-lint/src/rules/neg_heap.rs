//! `negated_heap` (L0571): C++'s `priority_queue` habit turned around,
//! `h.push(-x)` into a min-heap to pop the largest. `MaxHeap<T>`
//! (`std.heap`) pops the largest with the same costs. The heap is labelled
//! so the fact table keeps the finding only on a min-heap (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, NEGATED_HEAP};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{Expr, ExprKind, UnOp};
use std::collections::HashSet;

use super::shapes::name;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Pushes {
        base: u32,
        seen: HashSet<String>,
        out: Vec<Finding>,
    }
    impl Visitor for Pushes {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Call { callee, args } = &expr.kind else {
                return;
            };
            let ExprKind::Field { base, name: method } = &callee.kind else {
                return;
            };
            let (Some(h), [arg]) = (name(base), args.as_slice()) else {
                return;
            };
            let negated = matches!(
                &unparen(&arg.value).kind,
                ExprKind::Unary { op: UnOp::Neg, .. }
            );
            if method.text != "push" || !negated || !self.seen.insert(h.to_string()) {
                return;
            }
            let diag = Diagnostic::warning(
                NEGATED_HEAP.code,
                expr.span.shift(self.base),
                format!("`{h}` holds negated values so it pops the largest: it is a `MaxHeap`"),
            )
            .with_label(base.span.shift(self.base), "the heap")
            .with_help(format!(
                "declare `var {h} = MaxHeap<T>()` (`std.heap`, in the prelude) and push and pop plain values: it pops the largest at the same cost, with no `-` to pair on every pop"
            ));
            self.out.push(Finding {
                lint: &NEGATED_HEAP,
                diag,
            });
        }
    }
    let mut v = Pushes {
        base: 0,
        seen: HashSet::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
