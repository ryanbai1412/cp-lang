//! Data-structure algorithms written out by hand: merging the smaller set
//! into the larger (`small_to_large_by_hand`) and an interval DP stepped by
//! length (`interval_dp_by_length`).

use crate::facts::Facts;
use crate::std_fn;
use crate::walk::{unparen, Visitor};
use crate::Calls;
use crate::{File, Finding, INTERVAL_DP_BY_LENGTH, SMALL_TO_LARGE_BY_HAND};
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::{compare, name, single, squash};
use super::Warn;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandDs {
        src: file.src,
        base: 0,
        facts: file.facts,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandDs<'a> {
    src: &'a str,
    base: u32,
    facts: Option<&'a Facts>,
    out: Vec<Finding>,
}

/// `recv.method(args)`, the receiver as written.
fn call<'e>(e: &'e Expr, method: &str, calls: Calls) -> Option<(&'e Expr, &'e [Arg])> {
    let e = unparen(e);
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    match &callee.kind {
        ExprKind::Field { base, .. } if calls.is(e, method) => Some((base, args)),
        _ => None,
    }
}

fn call_stmt<'e>(stmt: &'e Stmt, methods: &[&str], calls: Calls) -> Option<(&'e Expr, &'e [Arg])> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    methods.iter().find_map(|m| call(e, m, calls))
}

fn loop_var(pat: &Pattern) -> Option<&str> {
    match &pat.kind {
        PatternKind::Name(n) => Some(n.as_str()),
        _ => None,
    }
}

impl super::Warn for HandDs<'_> {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32) {
        (&mut self.out, self.base)
    }
}

impl HandDs<'_> {
    fn calls(&self) -> Calls<'_> {
        Calls::new(self.facts, self.base)
    }

    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    /// `if a.len < b.len: (a, b) = (b, a)` then `for x in b: a.add(x)`.
    fn small_to_large(&mut self, stmts: &[Stmt]) {
        for pair in stmts.windows(2) {
            let StmtKind::Expr(e) = &pair[0].kind else {
                continue;
            };
            let ExprKind::If {
                branches,
                otherwise: None,
            } = &e.kind
            else {
                continue;
            };
            let [(Cond::Expr(c), swap)] = branches.as_slice() else {
                continue;
            };
            let Some((l, CmpOp::Lt | CmpOp::Gt | CmpOp::Le | CmpOp::Ge, r)) = compare(c) else {
                continue;
            };
            let is_len = |e: &Expr| {
                let e = unparen(e);
                matches!(e.kind, ExprKind::Field { .. }) && self.calls().is(e, std_fn::LEN)
            };
            if !is_len(l) || !is_len(r) {
                continue;
            }
            let Some(Stmt {
                kind:
                    StmtKind::Assign {
                        target,
                        op: AssignOp::Set,
                        value,
                    },
                ..
            }) = single(swap)
            else {
                continue;
            };
            let (ExprKind::Tuple(t), ExprKind::Tuple(v)) = (&target.kind, &value.kind) else {
                continue;
            };
            let ([a, b], [b2, a2]) = (t.as_slice(), v.as_slice()) else {
                continue;
            };
            let (a, b) = (name(a), name(b));
            if a.is_none() || a != name(a2) || b != name(b2) {
                continue;
            }
            let (Some(big), Some(small)) = (a, b) else {
                continue;
            };
            let StmtKind::For { iter, body, .. } = &pair[1].kind else {
                continue;
            };
            let Some((recv, _)) = single(body)
                .and_then(|s| call_stmt(s, &[std_fn::ADD, std_fn::INSERT], self.calls()))
            else {
                continue;
            };
            let mentions = |text: &str, w: &str| {
                text.match_indices(w).any(|(i, _)| {
                    let before = text[..i].chars().next_back();
                    let after = text[i + w.len()..].chars().next();
                    !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                        && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
                })
            };
            let (into, from) = (self.text(recv.span), self.text(iter.span));
            if !mentions(into, big) || !mentions(from, small) {
                continue;
            }
            let (into, from) = (into.to_string(), from.to_string());
            self.warn_at(
                &SMALL_TO_LARGE_BY_HAND,
                pair[0].span.to(pair[1].span),
&lint_rules::SMALL_TO_LARGE_BY_HAND, &[&from, &into],
                format!("`{into}.absorb(mut {from})` swaps so the smaller side moves, then empties `{from}` (`SortedSet`, `SortedBag`, `SortedMap`, `FlatMap`, `Counter`, `MeldHeap`)"),
            );
        }
    }

    /// `for len in ..: for l in ..: let r = l + len - 1`.
    fn interval_by_length(&mut self, stmt: &Stmt) {
        let StmtKind::For { pat, body, .. } = &stmt.kind else {
            return;
        };
        let Some(len) = loop_var(pat) else {
            return;
        };
        let hit = body.stmts.iter().any(|s| {
            let StmtKind::For { pat, body, .. } = &s.kind else {
                return false;
            };
            let Some(l) = loop_var(pat) else {
                return false;
            };
            let Some(Stmt {
                kind:
                    StmtKind::Let {
                        pat,
                        init: Some(init),
                        ..
                    },
                ..
            }) = body.stmts.first()
            else {
                return false;
            };
            let Some(r) = loop_var(pat) else {
                return false;
            };
            let table = squash(self.text(body.span));
            squash(self.text(init.span)) == format!("{l}+{len}-1")
                && (table.contains(&format!("[{l},{r}]")) || table.contains(&format!("[{l}][{r}]")))
        });
        if !hit {
            return;
        }
        self.warn_at(
            &INTERVAL_DP_BY_LENGTH,
            stmt.span,
&lint_rules::INTERVAL_DP_BY_LENGTH, &[&len],
            "`var dp: [I.ranges]T` keys the table by range, and `for r in I.ranges` visits shorter ranges first; `r.first!`, `r.last!`, `r.drop(1)` and `r.drop_last(1)` name the ends and the smaller ranges".to_string(),
        );
    }
}

impl Visitor for HandDs<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.interval_by_length(stmt);
    }

    fn stmts(&mut self, stmts: &[Stmt]) {
        self.small_to_large(stmts);
    }
}
