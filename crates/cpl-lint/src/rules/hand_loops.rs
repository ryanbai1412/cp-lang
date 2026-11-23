//! Loops and guards newcomers write by hand where a std call says what they
//! do: the index of the largest value (`loop_argmax`), the first unset slot
//! (`manual_mex`), a list copied element by element (`copy_comprehension`), a
//! queue walked by a head index (`queue_head_index`), a range check split in
//! two (`split_range_check`) and a map key tested before `+=`
//! (`contains_then_add`).

use crate::walk::{unparen, Visitor};
use crate::{
    File, Finding, Lint, CONTAINS_THEN_ADD, COPY_COMPREHENSION, LOOP_ARGMAX, MANUAL_MEX,
    QUEUE_HEAD_INDEX, SPLIT_RANGE_CHECK,
};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::*;

use super::shapes::{compare, int, is_step, name, single};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandLoops {
        src: file.src,
        base: 0,
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
    /// `and` chains already read whole, so their inner `and`s are skipped.
    chains: Vec<Span>,
    out: Vec<Finding>,
}

/// `if c: body` with no other branch.
fn lone_if(stmt: &Stmt) -> Option<(&Expr, &Block)> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    let ExprKind::If {
        branches,
        otherwise: None,
    } = &e.kind
    else {
        return None;
    };
    match branches.as_slice() {
        [(Cond::Expr(c), body)] => Some((c, body)),
        _ => None,
    }
}

/// `t[x]` with plain names.
fn index1(e: &Expr) -> Option<(&str, &str)> {
    let ExprKind::Index { base, indices } = &unparen(e).kind else {
        return None;
    };
    match indices.as_slice() {
        [x] => Some((name(base)?, name(x)?)),
        _ => None,
    }
}

impl HandLoops<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    fn warn(&mut self, lint: &'static Lint, span: Span, msg: String, help: String) {
        let diag = Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help);
        self.out.push(Finding { lint, diag });
    }

    fn argmax(&mut self, stmt: &Stmt) {
        let StmtKind::For {
            pat,
            body,
            else_body: None,
            ..
        } = &stmt.kind
        else {
            return;
        };
        let PatternKind::Name(i) = &pat.kind else {
            return;
        };
        let Some((cond, then)) = single(body).and_then(lone_if) else {
            return;
        };
        let Some(StmtKind::Assign {
            target,
            op: AssignOp::Set,
            value,
        }) = single(then).map(|s| &s.kind)
        else {
            return;
        };
        let (Some(j), Some(v)) = (name(target), name(value)) else {
            return;
        };
        let Some((lhs, op @ (CmpOp::Gt | CmpOp::Lt), rhs)) = compare(cond) else {
            return;
        };
        let (Some((t, x)), Some((t2, y))) = (index1(lhs), index1(rhs)) else {
            return;
        };
        if v != i || t != t2 || x != i || y != j {
            return;
        }
        let help = if op == CmpOp::Gt {
            format!(
                "write `{t}.argmax()!` when `{j}` starts at the first index and the loop covers the rest; over part of `{t}`, `(lo..hi).max(by: {i} => {t}[{i}])!` keeps the first largest on ties"
            )
        } else {
            format!(
                "write `{t}.argmin()!` when `{j}` starts at the first index and the loop covers the rest; over part of `{t}`, `(lo..hi).min(by: {i} => {t}[{i}])!` keeps the first smallest on ties"
            )
        };
        let ext = if op == CmpOp::Gt {
            "largest"
        } else {
            "smallest"
        };
        self.warn(
            &LOOP_ARGMAX,
            stmt.span,
            format!("this loop keeps in `{j}` the index of the first {ext} `{t}[{i}]`"),
            help,
        );
    }

    fn mex(&mut self, stmt: &Stmt) {
        let StmtKind::While {
            cond: Cond::Expr(c),
            body,
            else_body: None,
        } = &stmt.kind
        else {
            return;
        };
        let Some((t, m)) = index1(c) else {
            return;
        };
        if !single(body).is_some_and(|s| is_step(s, m)) {
            return;
        }
        self.warn(
            &MANUAL_MEX,
            stmt.span,
            format!("`while {t}[{m}]: {m} += 1` searches for the first unmarked value by hand"),
            format!(
                "write `(lo..=hi).find({m} => not {t}[{m}])` over the values `{m}` can take: it names the search, the mex when `{t}` marks what is present"
            ),
        );
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
            ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => it.to_string(),
            _ => format!("({it})"),
        };
        let mut diag = Diagnostic::warning(
            COPY_COMPREHENSION.code,
            expr.span.shift(self.base),
            format!(
                "`{}` copies a sequence one element at a time",
                self.text(expr.span)
            ),
        )
        .with_help(format!(
            "write `{it}.collect()`: it copies any sequence into a new `List`"
        ));
        // `collect` takes a `Seq`; `typed.rs` drops the finding where the
        // checker types the iterable as something else. A range always is one.
        if !matches!(iter.kind, ExprKind::Range { .. }) {
            diag = diag.with_label(iter.span.shift(self.base), "the iterable");
        }
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
            let ExprKind::Field { base, name: len } = &unparen(rhs).kind else {
                continue;
            };
            let (Some(q), true) = (name(base), name(lhs) == Some(h) && len.text == "len") else {
                continue;
            };
            let read = format!("{q}[{h}]");
            if !body.stmts.iter().any(|s| is_step(s, h))
                || !body
                    .stmts
                    .iter()
                    .any(|s| s.span.shift(w_base).text(self.src).contains(&read))
            {
                continue;
            }
            let span = Span::new(decl.span.start + decl_base, c.span.end + w_base);
            let diag = Diagnostic::warning(
                QUEUE_HEAD_INDEX.code,
                span,
                format!("`{q}` is walked as a queue by the head index `{h}`"),
            )
            .with_help("`bfs(next, from: [s])` from `std.graph` runs the queue and returns the distances and the visit order; a queue run by hand is a `Deque`, which pops from the front");
            self.out.push(Finding {
                lint: &QUEUE_HEAD_INDEX,
                diag,
            });
        }
    }

    /// A comparison bounding a name: the name, whether it is the lower bound,
    /// and the bound's side of the chained comparison.
    fn bound<'e>(&self, e: &'e Expr) -> Option<(&'e str, bool, String)> {
        let (l, op, r) = compare(e)?;
        let t = |e: &Expr| self.text(e.span).to_string();
        Some(match (name(l), name(r), op) {
            (Some(x), _, CmpOp::Ge) => (x, true, format!("{} <=", t(r))),
            (Some(x), _, CmpOp::Gt) => (x, true, format!("{} <", t(r))),
            (Some(x), _, CmpOp::Lt) => (x, false, format!("< {}", t(r))),
            (Some(x), _, CmpOp::Le) => (x, false, format!("<= {}", t(r))),
            (_, Some(x), CmpOp::Le) => (x, true, format!("{} <=", t(l))),
            (_, Some(x), CmpOp::Lt) => (x, true, format!("{} <", t(l))),
            (_, Some(x), CmpOp::Gt) => (x, false, format!("< {}", t(l))),
            (_, Some(x), CmpOp::Ge) => (x, false, format!("<= {}", t(l))),
            _ => return None,
        })
    }

    fn range_check(&mut self, expr: &Expr) {
        let ExprKind::Binary { op: BinOp::And, .. } = &expr.kind else {
            return;
        };
        if self
            .chains
            .iter()
            .any(|c| c.start <= expr.span.start && expr.span.end <= c.end)
        {
            return;
        }
        self.chains.push(expr.span);
        let mut terms = Vec::new();
        flatten(expr, &mut terms);
        let bounds: Vec<_> = terms.iter().map(|e| self.bound(e)).collect();
        let mut chained = Vec::new();
        for (k, b) in bounds.iter().enumerate() {
            let Some((x, true, lo)) = b else { continue };
            // The other bound is next to it, so the chain keeps the order.
            let near = [k.wrapping_sub(1), k + 1];
            let upper = near.iter().find_map(|&m| match bounds.get(m) {
                Some(Some((y, false, hi))) if y == x => Some((m, hi)),
                _ => None,
            });
            if let Some((m, hi)) = upper {
                // Bounds that swap places must not call anything: the chain
                // evaluates the lower one first.
                let pure = m > k || !format!("{lo}{hi}").contains('(');
                chained.push((format!("{lo} {x} {hi}"), pure));
            }
        }
        // Two coordinates checked together are `grid_bounds_check` (L0535).
        let [(first, pure)] = chained.as_slice() else {
            return;
        };
        let mut diag = Diagnostic::warning(
            SPLIT_RANGE_CHECK.code,
            expr.span.shift(self.base),
            "a range check written as two comparisons",
        )
        .with_help(format!(
            "write `{first}`: one chained comparison reads as the range it checks"
        ));
        if terms.len() == 2 && *pure {
            diag.fixes.push(Fix {
                title: format!("write `{first}`"),
                edits: vec![(expr.span.shift(self.base), first.clone())],
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
        let mut diag = Diagnostic::warning(
            CONTAINS_THEN_ADD.code,
            stmt.span.shift(self.base),
            format!("`{place}` is tested for a key before `+=`"),
        )
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
        self.argmax(stmt);
        self.mex(stmt);
        self.contains_add(stmt);
    }

    fn expr(&mut self, expr: &Expr) {
        self.copy(expr);
        self.range_check(expr);
    }
}
