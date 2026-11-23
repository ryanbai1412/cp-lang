//! Data-structure algorithms written out by hand: merging the smaller set
//! into the larger (`small_to_large_by_hand`), counting inversions with a
//! Fenwick tree (`inversions_by_fenwick`) and an interval DP stepped by
//! length (`interval_dp_by_length`).

use crate::walk::{unparen, Visitor};
use crate::{
    File, Finding, Lint, INTERVAL_DP_BY_LENGTH, INVERSIONS_BY_FENWICK, SMALL_TO_LARGE_BY_HAND,
};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::*;

use super::shapes::{compare, int, name, single};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandDs {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandDs<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// `recv.method(args)`, the receiver as written.
fn call<'e>(e: &'e Expr, method: &str) -> Option<(&'e Expr, &'e [Arg])> {
    let ExprKind::Call { callee, args } = &unparen(e).kind else {
        return None;
    };
    match &callee.kind {
        ExprKind::Field { base, name: m } if m.text == method => Some((base, args)),
        _ => None,
    }
}

fn call_stmt<'e>(stmt: &'e Stmt, methods: &[&str]) -> Option<(&'e Expr, &'e [Arg])> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    methods.iter().find_map(|m| call(e, m))
}

fn loop_var(pat: &Pattern) -> Option<&str> {
    match &pat.kind {
        PatternKind::Name(n) => Some(n.as_str()),
        _ => None,
    }
}

impl HandDs<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    fn warn(&mut self, lint: &'static Lint, span: Span, msg: String, help: String) {
        let diag = Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help);
        self.out.push(Finding { lint, diag });
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
            let is_len = |e: &Expr| matches!(&unparen(e).kind, ExprKind::Field { name, .. } if name.text == "len");
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
            let Some((recv, _)) = single(body).and_then(|s| call_stmt(s, &["add", "insert"]))
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
            self.warn(
                &SMALL_TO_LARGE_BY_HAND,
                pair[0].span.to(pair[1].span),
                format!("`{from}` is merged into `{into}` smaller-into-larger by hand"),
                format!("`{into}.absorb(mut {from})` swaps so the smaller side moves, then empties `{from}` (`SortedSet`, `SortedBag`, `SortedMap`, `FlatMap`, `Counter`, `MeldHeap`)"),
            );
        }
    }

    /// `for i ..: inv += i - bit.prefix(x); bit.add(x, 1)`.
    fn inversions(&mut self, stmt: &Stmt) {
        let StmtKind::For { body, .. } = &stmt.kind else {
            return;
        };
        let counted = body.stmts.iter().find_map(|s| {
            let StmtKind::Assign {
                op: AssignOp::Binary(BinOp::Add),
                value,
                ..
            } = &s.kind
            else {
                return None;
            };
            let ExprKind::Binary {
                op: BinOp::Sub,
                rhs,
                ..
            } = &unparen(value).kind
            else {
                return None;
            };
            name(call(rhs, "prefix")?.0)
        });
        let Some(bit) = counted else {
            return;
        };
        let adds_one = body.stmts.iter().any(|s| {
            call_stmt(s, &["add"]).is_some_and(|(r, args)| {
                name(r) == Some(bit)
                    && match args {
                        [_] => true,
                        [_, one] => int(&one.value) == Some(1),
                        _ => false,
                    }
            })
        });
        if !adds_one {
            return;
        }
        self.warn(
            &INVERSIONS_BY_FENWICK,
            stmt.span,
            format!("`{bit}` counts, for each item, the earlier ones above it: an inversion count by hand"),
            "`inversions(xs)` is the number of pairs `i < j` with `xs[i] > xs[j]`, in \\(O(n \\log n)\\), with no compression or Fenwick tree".to_string(),
        );
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
            body.stmts.first().is_some_and(|s| match &s.kind {
                StmtKind::Let {
                    init: Some(init), ..
                } => squash(self.text(init.span)) == format!("{l}+{len}-1"),
                _ => false,
            })
        });
        if !hit {
            return;
        }
        self.warn(
            &INTERVAL_DP_BY_LENGTH,
            stmt.span,
            format!("an interval DP stepped by length `{len}` with its end computed by hand"),
            "`var dp: [I.ranges]T` keys the table by range, and `for r in I.ranges` visits shorter ranges first; `r.first!`, `r.last!`, `r.drop(1)` and `r.drop_last(1)` name the ends and the smaller ranges".to_string(),
        );
    }
}

impl Visitor for HandDs<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.inversions(stmt);
        self.interval_by_length(stmt);
    }

    fn stmts(&mut self, stmts: &[Stmt]) {
        self.small_to_large(stmts);
    }
}
