//! Loops and guards newcomers write by hand where a std call says what they
//! do: a list copied element by element (`copy_comprehension`), a
//! queue walked by a head index (`queue_head_index`), a range check split in
//! two (`split_range_check`) and a map key tested before `+=`
//! (`contains_then_add`).

use crate::facts::Facts;
use crate::is_std;
use crate::std_fn;
use crate::walk::{unparen, walk_stmt, Visitor};
use crate::{
    File, Finding, CONTAINS_THEN_ADD, COPY_COMPREHENSION, QUEUE_HEAD_INDEX, SPLIT_RANGE_CHECK,
};
use cpl_diag::{Fix, Span};
use cpl_syntax::ast::*;

use super::shapes::{compare, int, is_step, name, single};
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandLoops {
        src: file.src,
        base: 0,
        facts: file.facts,
        chains: Vec::new(),
        out: Vec::new(),
    };
    let top: Vec<_> = file.top_level().collect();
    v.queue_head(&top);
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandLoops<'a> {
    src: &'a str,
    base: u32,
    facts: Option<&'a Facts>,
    /// `and` chains already read whole, so their inner `and`s are skipped.
    chains: Vec<Span>,
    out: Vec<Finding>,
}

impl HandLoops<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    fn copy(&mut self, expr: &Expr) {
        let ExprKind::Comprehension { body, clauses } = &expr.kind else {
            return;
        };
        let [Clause { pat, iter, conds }] = clauses.as_slice() else {
            return;
        };
        let PatternKind::Name(x) = &pat.kind else {
            return;
        };
        if !conds.is_empty() || name(body) != Some(x) || !self.text(expr.span).starts_with('[') {
            return;
        }
        let it = self.text(iter.span);
        let it = match iter.kind {
            ExprKind::Name(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Call { .. }
            | ExprKind::Paren(_)
            | ExprKind::List(_) => it.to_string(),
            _ => format!("({it})"),
        };
        let mut diag = lint_rules::COPY_COMPREHENSION
            .warning(expr.span.shift(self.base), &[&self.text(expr.span)])
            .with_help(format!(
                "write `{it}.collect()`: it copies any sequence into a new `List`"
            ));
        // `collect` takes a `Seq`; `typed.rs` drops the finding where the
        // checker types the iterable as something else. A range always is one.
        if !matches!(iter.kind, ExprKind::Range { .. }) {
            diag = diag.with_label(iter.span.shift(self.base), "the iterable");
        }
        // `typed.rs` keeps the fix only where both spellings are a `List` of
        // the same elements.
        diag.fixes.push(Fix {
            title: format!("write `{it}.collect()`"),
            edits: vec![(expr.span.shift(self.base), format!("{it}.collect()"))],
        });
        self.out.push(Finding {
            lint: &COPY_COMPREHENSION,
            diag,
        });
    }

    /// Each statement with its offset: top-level ones come from different items.
    fn queue_head(&mut self, stmts: &[(u32, &Stmt)]) {
        for (k, &(decl_base, decl)) in stmts.iter().enumerate() {
            let StmtKind::Let {
                mutable: true,
                pat,
                init: Some(init),
                ..
            } = &decl.kind
            else {
                continue;
            };
            let (PatternKind::Name(h), Some(0)) = (&pat.kind, int(init)) else {
                continue;
            };
            let Some(&(w_base, w)) = stmts[k + 1..]
                .iter()
                .find(|(_, s)| matches!(s.kind, StmtKind::While { .. }))
            else {
                continue;
            };
            let StmtKind::While {
                cond: Cond::Expr(c),
                body,
                ..
            } = &w.kind
            else {
                continue;
            };
            let Some((lhs, CmpOp::Lt, rhs)) = compare(c) else {
                continue;
            };
            let len = unparen(rhs);
            let ExprKind::Field { base, .. } = &len.kind else {
                continue;
            };
            let is_len = is_std(self.facts, len.span.shift(w_base), std_fn::LEN);
            let (Some(q), true) = (name(base), name(lhs) == Some(h) && is_len) else {
                continue;
            };
            let read = format!("{q}[{h}]");
            if !body.stmts.iter().any(|s| is_step(s, h))
                || reads_head_otherwise(&body.stmts, h)
                || !body
                    .stmts
                    .iter()
                    .any(|s| s.span.shift(w_base).text(self.src).contains(&read))
            {
                continue;
            }
            let span = Span::new(decl.span.start + decl_base, c.span.end + w_base);
            let diag = lint_rules::QUEUE_HEAD_INDEX.warning(span, &[&q, &h])
            .with_help("`bfs(next, from: [s])` from `std.graph` runs the queue and returns the distances and the visit order; a queue run by hand is a `Deque`, which pops from the front");
            self.out.push(Finding {
                lint: &QUEUE_HEAD_INDEX,
                diag,
            });
        }
    }

    fn range_check(&mut self, expr: &Expr) {
        let ExprKind::Binary { op: BinOp::And, .. } = &expr.kind else {
            return;
        };
        // Spans are item-relative; the chains of every item are kept here.
        let at = expr.span.shift(self.base);
        if self
            .chains
            .iter()
            .any(|c| c.start <= at.start && at.end <= c.end)
        {
            return;
        }
        self.chains.push(at);
        let mut terms = Vec::new();
        flatten(expr, &mut terms);
        let asc: Vec<_> = terms.iter().map(|e| ascending(e)).collect();
        let t = |e: &Expr| self.text(e.span).to_string();
        // Neighbouring comparisons through a shared name: `lo <= x` then
        // `x < hi` (or written the other way round) is `lo <= x < hi`.
        let links = |j: usize, forward: bool| match (&asc[j], asc.get(j + 1)) {
            (Some((lo, _, hi)), Some(Some((lo2, _, hi2)))) => {
                let (a, b) = if forward { (hi, lo2) } else { (lo, hi2) };
                name(a).is_some() && name(a) == name(b)
            }
            _ => false,
        };
        let mut runs = Vec::new();
        let mut k = 0;
        while k < terms.len() {
            let Some(forward) = [true, false]
                .into_iter()
                .find(|&f| k + 1 < terms.len() && links(k, f))
            else {
                k += 1;
                continue;
            };
            let mut e = k + 1;
            while e + 1 < terms.len() && links(e, forward) {
                e += 1;
            }
            let mut order: Vec<_> = (k..=e).filter_map(|j| asc[j]).collect();
            if !forward {
                order.reverse();
            }
            let mut chain = t(order[0].0);
            for (_, op, hi) in &order {
                chain = format!("{chain} {op} {}", t(hi));
            }
            let span = Span::new(terms[k].span.start, terms[e].span.end);
            // A chain written the other way round evaluates its bounds in
            // the other order, so they must not call anything.
            let pure = forward || !self.text(span).contains('(');
            runs.push((span, e - k + 1, chain, pure));
            k = e + 1;
        }
        // Two coordinates checked together are `grid_bounds_check` (L0535).
        let [(span, len, chain, pure)] = runs.as_slice() else {
            return;
        };
        let count = match len {
            2 => "two".to_string(),
            3 => "three".to_string(),
            n => n.to_string(),
        };
        let mut diag = lint_rules::SPLIT_RANGE_CHECK
            .warning(expr.span.shift(self.base), &[&count])
            .with_help(format!(
                "write `{chain}`: one chained comparison reads as the range it checks"
            ));
        if *pure {
            diag.fixes.push(Fix {
                title: format!("write `{chain}`"),
                edits: vec![(span.shift(self.base), chain.clone())],
            });
        }
        self.out.push(Finding {
            lint: &SPLIT_RANGE_CHECK,
            diag,
        });
    }

    fn contains_add(&mut self, stmt: &Stmt) {
        let StmtKind::Expr(e) = &stmt.kind else {
            return;
        };
        let ExprKind::If {
            branches,
            otherwise: Some(els),
        } = &e.kind
        else {
            return;
        };
        let [(Cond::Expr(c), then)] = branches.as_slice() else {
            return;
        };
        let Some((k, op @ (CmpOp::In | CmpOp::NotIn), m)) = compare(c) else {
            return;
        };
        let (add, set) = if op == CmpOp::In {
            (then, els)
        } else {
            (els, then)
        };
        let (
            Some(StmtKind::Assign {
                target: t1,
                op: AssignOp::Binary(BinOp::Add),
                value: v1,
            }),
            Some(StmtKind::Assign {
                target: t2,
                op: AssignOp::Set,
                value: v2,
            }),
        ) = (single(add).map(|s| &s.kind), single(set).map(|s| &s.kind))
        else {
            return;
        };
        let place = format!("{}[{}]", self.text(m.span), self.text(k.span));
        // The `if` ends with its later branch's value, before the newline.
        let whole = Span::new(stmt.span.start, v1.span.end.max(v2.span.end)).shift(self.base);
        let (v1, v2) = (self.text(v1.span), self.text(v2.span));
        if self.text(t1.span) != place || self.text(t2.span) != place || v1 != v2 {
            return;
        }
        // The fix holds where `+=` starts a missing key at 0; the checker's
        // type of the map (the label) decides that in `typed.rs`.
        let bump = format!("{place} += {v1}");
        let mut diag = lint_rules::CONTAINS_THEN_ADD.warning(stmt.span.shift(self.base), &[&place])
        .with_label(m.span.shift(self.base), "the map")
        .with_help(format!(
            "write `{bump}`: on a `Map`, `+=` starts a missing key at `0`, so the test and the `else` are the same step"
        ));
        diag.fixes.push(Fix {
            title: format!("write `{bump}`"),
            edits: vec![(whole, bump)],
        });
        self.out.push(Finding {
            lint: &CONTAINS_THEN_ADD,
            diag,
        });
    }
}

/// `e` as `lo op hi` with `op` `<` or `<=`: `x >= 0` is `0 <= x`.
fn ascending(e: &Expr) -> Option<(&Expr, &'static str, &Expr)> {
    let (l, op, r) = compare(e)?;
    Some(match op {
        CmpOp::Lt => (l, "<", r),
        CmpOp::Le => (l, "<=", r),
        CmpOp::Gt => (r, "<", l),
        CmpOp::Ge => (r, "<=", l),
        _ => return None,
    })
}

fn flatten<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            flatten(lhs, out);
            flatten(rhs, out);
        }
        _ => out.push(e),
    }
}

impl Visitor for HandLoops<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let stmts: Vec<_> = stmts.iter().map(|s| (base, s)).collect();
        self.queue_head(&stmts);
    }

    fn stmt(&mut self, stmt: &Stmt) {
        self.contains_add(stmt);
    }

    fn expr(&mut self, expr: &Expr) {
        self.copy(expr);
        self.range_check(expr);
    }
}

/// Whether `body` reads the head index `h` other than as an index (`q[h]`,
/// `qy[h]`) and its step: then it is a result too (a rank, a count), which a
/// queue drops.
fn reads_head_otherwise(body: &[Stmt], h: &str) -> bool {
    struct Reads<'a> {
        h: &'a str,
        names: usize,
        heads: usize,
    }
    impl Visitor for Reads<'_> {
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Name(n) if n == self.h => self.names += 1,
                ExprKind::Index { indices, .. } if matches!(indices.as_slice(), [i] if name(i) == Some(self.h)) => {
                    self.heads += 1
                }
                _ => {}
            }
        }
    }
    let mut reads = Reads {
        h,
        names: 0,
        heads: 0,
    };
    for s in body.iter().filter(|s| !is_step(s, h)) {
        walk_stmt(&mut reads, s);
    }
    reads.names > reads.heads
}
