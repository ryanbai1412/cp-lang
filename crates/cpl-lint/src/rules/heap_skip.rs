//! `stale_entry_skip` (L0553): C++'s lazy-deletion Dijkstra, `auto [d, u] =
//! pq.top(); pq.pop(); if (d > dist[u]) continue;`. `dijkstra` (`std.graph`)
//! is the whole loop, settling each vertex once. The popped receiver is
//! labelled so the fact table keeps the finding only on a heap (`typed.rs`).

use crate::walk::Visitor;
use crate::{File, Finding, STALE_ENTRY_SKIP};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{CmpOp, Cond, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind};

const HELP: &str = "build the graph with `Graph.directed(es)` and call `dijkstra(g, from: [(s, 0)])` (`std.graph`): it is this heap loop, settles each vertex once and returns `Paths`, so there is no stale entry to skip";

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Skips {
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Skips {
        fn stmt(&mut self, stmt: &Stmt) {
            let StmtKind::While { cond, body, .. } = &stmt.kind else {
                return;
            };
            let mut popped = match cond {
                Cond::Let { pat, value, .. } => popped_from(value).zip(names(pat)),
                _ => None,
            };
            for s in &body.stmts {
                if let StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } = &s.kind
                {
                    let init = match &init.kind {
                        ExprKind::Unwrap(e) => e,
                        _ => init,
                    };
                    if let Some(p) = popped_from(init).zip(names(pat)) {
                        popped = Some(p);
                    }
                }
                let Some((heap, (d, u))) = &popped else {
                    continue;
                };
                if stale_skip(s, d, u) {
                    let diag = Diagnostic::warning(
                        STALE_ENTRY_SKIP.code,
                        s.span.shift(self.base),
                        format!("`{d}` is checked against the best distance of `{u}` to skip a stale heap entry"),
                    )
                    .with_label(heap.span.shift(self.base), "the heap")
                    .with_help(HELP);
                    self.out.push(Finding {
                        lint: &STALE_ENTRY_SKIP,
                        diag,
                    });
                }
            }
        }
    }
    let mut v = Skips {
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// `h` in `h.m()`: a pop when the facts say `h` is a heap.
fn popped_from(e: &Expr) -> Option<&Expr> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, .. } = &callee.kind else {
        return None;
    };
    (args.is_empty() && matches!(base.kind, ExprKind::Name(_))).then_some(&**base)
}

/// `(d, u)`.
fn names(pat: &Pattern) -> Option<(String, String)> {
    let PatternKind::Tuple(parts) = &pat.kind else {
        return None;
    };
    match parts.as_slice() {
        [a, b] => match (&a.kind, &b.kind) {
            (PatternKind::Name(d), PatternKind::Name(u)) => Some((d.clone(), u.clone())),
            _ => None,
        },
        _ => None,
    }
}

/// `if d > dist[u]: continue` (or `!=`, or `dist[u]!`).
fn stale_skip(stmt: &Stmt, d: &str, u: &str) -> bool {
    let StmtKind::Expr(e) = &stmt.kind else {
        return false;
    };
    let ExprKind::If {
        branches,
        otherwise: None,
    } = &e.kind
    else {
        return false;
    };
    let [(Cond::Expr(c), then)] = branches.as_slice() else {
        return false;
    };
    let continues = matches!(then.stmts.as_slice(), [s]
        if matches!(&s.kind, StmtKind::Continue(None)));
    let ExprKind::Compare { first, rest } = &c.kind else {
        return false;
    };
    let [(CmpOp::Gt | CmpOp::Ne, best)] = rest.as_slice() else {
        return false;
    };
    let best = match &best.kind {
        ExprKind::Unwrap(e) => e,
        _ => best,
    };
    let ExprKind::Index { indices, .. } = &best.kind else {
        return false;
    };
    continues
        && matches!(&first.kind, ExprKind::Name(n) if n == d)
        && matches!(indices.as_slice(), [i] if matches!(&i.kind, ExprKind::Name(n) if n == u))
}
