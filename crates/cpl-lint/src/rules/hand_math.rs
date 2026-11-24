//! Bookkeeping newcomers write by hand where std names it: a memo map
//! (`memo_by_hand`) and a difference array (`diff_by_hand`). The idioms one
//! library call replaces are `@lint` declarations in `std/` (R06.4.10).

use crate::walk::Visitor;
use crate::{File, Finding, DIFF_BY_HAND, MEMO_BY_HAND};
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::{compare, index1, name, single};
use super::Warn;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandMath {
        src: file.src,
        base: 0,
        memos: Vec::new(),
        out: Vec::new(),
    };
    let top: Vec<_> = file.top_level().collect();
    v.diff(&top);
    file.walk(&mut v, |v, base| v.base = base);
    // A memo map is reported at its declaration.
    let memos = std::mem::take(&mut v.memos);
    for (base, stmt) in top {
        let StmtKind::Let { pat, .. } = &stmt.kind else {
            continue;
        };
        let PatternKind::Name(m) = &pat.kind else {
            continue;
        };
        if let Some((_, f, lookup)) = memos.iter().find(|(name, ..)| name == m) {
            v.base = base;
            v.warn_at(
                &MEMO_BY_HAND,
                stmt.span,
&lint_rules::MEMO_BY_HAND, &[&m, &f],
                format!(
                    "put `@memo(map)` (or `@memo(array)` over bounded parameters) on `fn {f}` and drop `{m}`: the lookup and the store become one attribute"
                ),
            );
            if let Some(last) = v.out.last_mut() {
                last.diag = last
                    .diag
                    .clone()
                    .with_label(*lookup, "the lookup `@memo` replaces");
            }
        }
    }
    v.out
}

struct HandMath<'a> {
    src: &'a str,
    base: u32,
    /// (map, fn, lookup) for each `if k in map: return map[k]` at the top of
    /// a fn.
    memos: Vec<(String, String, Span)>,
    out: Vec<Finding>,
}

impl super::Warn for HandMath<'_> {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32) {
        (&mut self.out, self.base)
    }
}

impl HandMath<'_> {
    fn memo(&mut self, stmt: &Stmt) {
        let StmtKind::Fn(decl) = &stmt.kind else {
            return;
        };
        let FnBody::Block(body) = &decl.body else {
            return;
        };
        for s in &body.stmts {
            let StmtKind::Expr(e) = &s.kind else {
                continue;
            };
            let ExprKind::If {
                branches,
                otherwise: None,
            } = &e.kind
            else {
                continue;
            };
            let [(Cond::Expr(c), then)] = branches.as_slice() else {
                continue;
            };
            let Some((_, CmpOp::In, m)) = compare(c) else {
                continue;
            };
            let Some(StmtKind::Return(Some(r))) = single(then).map(|s| &s.kind) else {
                continue;
            };
            if let (Some(m), Some((t, _))) = (name(m), index1(r)) {
                if m == t {
                    let lookup = s.span.shift(self.base);
                    self.memos
                        .push((m.to_string(), decl.name.text.clone(), lookup));
                }
            }
        }
    }

    /// `d[a] += x` then `d[b] -= x`; each statement with its offset.
    fn diff(&mut self, stmts: &[(u32, &Stmt)]) {
        for pair in stmts.windows(2) {
            let [(b1, s1), (b2, s2)] = pair else {
                continue;
            };
            let (
                StmtKind::Assign {
                    target: t1,
                    op: AssignOp::Binary(BinOp::Add),
                    value: v1,
                },
                StmtKind::Assign {
                    target: t2,
                    op: AssignOp::Binary(BinOp::Sub),
                    value: v2,
                },
            ) = (&s1.kind, &s2.kind)
            else {
                continue;
            };
            let (Some((d, _)), Some((d2, _))) = (index1(t1), index1(t2)) else {
                continue;
            };
            let text = |b: u32, e: &Expr| e.span.shift(b).text(self.src);
            // The one-off shift (`d[r + 1]` or `d[l - 1]`) tells a range
            // update from moving an amount between two cells.
            let shifted = text(*b2, t2).ends_with("+ 1]") || text(*b1, t1).ends_with("- 1]");
            if d != d2 || text(*b1, v1) != text(*b2, v2) || !shifted {
                continue;
            }
            let diag = lint_rules::DIFF_BY_HAND.warning(Span::new(s1.span.start + b1, s2.span.end + b2), &[&d])
            .with_help(format!(
                "`Diff<D, T>()` (`std.prefix`) is one: `{d}.add(l..r, x)` for each update, then `{d}.finish()` is the table"
            ));
            self.out.push(Finding {
                lint: &DIFF_BY_HAND,
                diag,
            });
        }
    }
}

impl Visitor for HandMath<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let stmts: Vec<_> = stmts.iter().map(|s| (base, s)).collect();
        self.diff(&stmts);
    }

    fn stmt(&mut self, stmt: &Stmt) {
        self.memo(stmt);
    }
}
