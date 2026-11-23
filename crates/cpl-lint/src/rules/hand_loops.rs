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
        starts: Vec::new(),
        out: Vec::new(),
    };
    let top: Vec<_> = file.top_level().collect();
    v.queue_head(&top);
    v.starts(&top);
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandLoops<'a> {
    src: &'a str,
    base: u32,
    /// `and` chains already read whole, so their inner `and`s are skipped.
    chains: Vec<Span>,
    /// `var j = init` right before a statement: (that statement, `j`, the
    /// `var`'s span, `init`), all in file offsets.
    starts: Vec<(Span, String, Span, String)>,
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
fn index_names(e: &Expr) -> Option<(&str, &str)> {
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

    fn starts(&mut self, stmts: &[(u32, &Stmt)]) {
        for pair in stmts.windows(2) {
            let [(base, decl), (next_base, next)] = pair else {
                continue;
            };
            let StmtKind::Let {
                mutable: true,
                pat,
                ty: None,
                init: Some(init),
                otherwise: None,
                guard: None,
            } = &decl.kind
            else {
                continue;
            };
            let PatternKind::Name(j) = &pat.kind else {
                continue;
            };
            let init = init.span.shift(*base).text(self.src).to_string();
            self.starts.push((
                next.span.shift(*next_base),
                j.to_string(),
                decl.span.shift(*base),
                init,
            ));
        }
    }

    /// The range a loop over `iter` covers once `j` starts at `init`:
    /// `lo..hi` from `init` or from `init + 1`.
    fn range_from(&self, iter: &Expr, init: &str) -> Option<String> {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &unparen(iter).kind
        else {
            return None;
        };
        let next = init.parse::<i128>().ok().map(|v| v + 1);
        let from_next = next.is_some() && int(lo) == next;
        let lo = self.text(lo.span);
        if lo != init && lo != format!("{init} + 1") && !from_next {
            return None;
        }
        let dots = if *inclusive { "..=" } else { ".." };
        Some(format!("({init}{dots}{})", self.text(hi.span)))
    }

    fn argmax(&mut self, stmt: &Stmt) {
        let StmtKind::For {
            pat,
            iter,
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
        let (Some((t, x)), Some((t2, y))) = (index_names(lhs), index_names(rhs)) else {
            return;
        };
        if v != i || t != t2 || x != i || y != j {
            return;
        }
        let largest = op == CmpOp::Gt;
        let ext = if largest { "largest" } else { "smallest" };
        let whole = if largest {
            format!("{t}.argmax()!")
        } else {
            format!("{t}.argmin()!")
        };
        let at = stmt.span.shift(self.base);
        let start = self
            .starts
            .iter()
            .find(|(s, var, ..)| *s == at && var == j)
            .map(|(_, _, decl, init)| (*decl, init.clone()));
        let pick = |over: &str| {
            if largest {
                format!("{over}.max(by: {i} => {t}[{i}])!")
            } else {
                format!("{over}.min(by: {i} => {t}[{i}])!")
            }
        };
        let walked = self.text(iter.span).to_string();
        let mut fix = None;
        let help = match &start {
            // `D.first!` traps on an empty `D` just as the call does.
            Some((decl, init)) if name(iter).is_some() && *init == format!("{walked}.first!") => {
                let call = pick(&walked);
                let between = Span {
                    start: decl.end,
                    end: at.start,
                };
                let looped = at.text(self.src);
                // A statement's span can run on over the newline and indent after it.
                let tail = &looped[looped.trim_end().len()..];
                if between.text(self.src).trim().is_empty() {
                    fix = Some(Fix {
                        title: format!("write `var {j} = {call}`"),
                        edits: vec![(
                            Span {
                                start: decl.start,
                                end: at.end,
                            },
                            format!("var {j} = {call}{tail}"),
                        )],
                    });
                }
                format!("write `var {j} = {call}`: the first {ext} on ties, as the strict loop keeps")
            }
            Some((_, init)) if self.range_from(iter, init).is_some() => {
                let call = pick(&self.range_from(iter, init).unwrap_or_default());
                format!(
                    "write `var {j} = {call}`: the first {ext} on ties, as the strict loop keeps; `{whole}` when that range is all of `{t}`"
                )
            }
            _ => format!(
                "write `{whole}` when `{j}` starts at the first index and the loop covers the rest; over part of `{t}`, `{}` keeps the first {ext} on ties",
                pick("(lo..hi)")
            ),
        };
        let mut diag = Diagnostic::warning(
            LOOP_ARGMAX.code,
            at,
            format!("this loop keeps in `{j}` the index of the first {ext} `{t}[{i}]`"),
        )
        .with_help(help);
        diag.fixes.extend(fix);
        self.out.push(Finding {
            lint: &LOOP_ARGMAX,
            diag,
        });
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
        let Some((t, m)) = index_names(c) else {
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
        let mut diag = Diagnostic::warning(
            SPLIT_RANGE_CHECK.code,
            expr.span.shift(self.base),
            format!("a range check written as {count} comparisons"),
        )
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
        self.starts(&stmts);
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
