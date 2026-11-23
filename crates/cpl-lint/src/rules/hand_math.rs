//! Number theory, trees, geometry and counting that newcomers write by hand
//! where std names it: trial division (`trial_division`), a memo map
//! (`memo_by_hand`), climbing to the LCA (`lca_by_hand`), a cross product of
//! parallel coordinate lists (`cross_by_hand`), `n choose k` from factorial
//! tables (`binom_by_hand`), a run counter (`runs_by_hand`), a difference
//! array (`diff_by_hand`) and counting components by DFS
//! (`components_by_dfs`).

use crate::walk::{unparen, Visitor};
use crate::{
    File, Finding, Lint, BINOM_BY_HAND, COMPONENTS_BY_DFS, CROSS_BY_HAND, DIFF_BY_HAND,
    LCA_BY_HAND, MEMO_BY_HAND, RUNS_BY_HAND, TRIAL_DIVISION,
};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::*;

use super::shapes::{compare, index1, int, is_step, name, single};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandMath {
        src: file.src,
        base: 0,
        products: Vec::new(),
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
            v.warn(
                &MEMO_BY_HAND,
                stmt.span,
                format!("`{m}` is a memo table kept by hand for `{f}`"),
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
    /// Products already read whole, so their inner parts are skipped.
    products: Vec<Span>,
    /// (map, fn, lookup) for each `if k in map: return map[k]` at the top of
    /// a fn.
    memos: Vec<(String, String, Span)>,
    out: Vec<Finding>,
}

fn binary(e: &Expr, want: BinOp) -> Option<(&Expr, &Expr)> {
    match &unparen(e).kind {
        ExprKind::Binary { op, lhs, rhs, .. } if *op == want => Some((lhs, rhs)),
        _ => None,
    }
}

fn is_reset(stmt: &Stmt, var: &str) -> bool {
    matches!(&stmt.kind, StmtKind::Assign { target, op: AssignOp::Set, value }
        if name(target) == Some(var) && int(value) == Some(0))
}

/// Every one-index `t[i]` in a product, through calls and arithmetic.
fn indexed<'e>(e: &'e Expr, out: &mut Vec<(&'e str, &'e Expr)>) {
    let e = unparen(e);
    if let Some(hit) = index1(e) {
        out.push(hit);
        return;
    }
    match &e.kind {
        ExprKind::Binary { lhs, rhs, .. } => {
            indexed(lhs, out);
            indexed(rhs, out);
        }
        ExprKind::Call { args, .. } => {
            for a in args {
                indexed(&a.value, out);
            }
        }
        _ => {}
    }
}

impl HandMath<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    fn warn(&mut self, lint: &'static Lint, span: Span, msg: String, help: String) {
        let diag = Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help);
        self.out.push(Finding { lint, diag });
    }

    fn trial_division(&mut self, stmt: &Stmt) {
        let StmtKind::While {
            cond: Cond::Expr(c),
            body,
            ..
        } = &stmt.kind
        else {
            return;
        };
        let Some((lhs, CmpOp::Le, rhs)) = compare(c) else {
            return;
        };
        // `d * d <= x` or `d <= x / d`.
        let (d, x) = match (binary(lhs, BinOp::Mul), binary(rhs, BinOp::Div)) {
            (Some((a, b)), _) if name(a).is_some() && name(a) == name(b) => (name(a), name(rhs)),
            (_, Some((x, b))) if name(lhs).is_some() && name(lhs) == name(b) => {
                (name(lhs), name(x))
            }
            _ => return,
        };
        let (Some(d), Some(x)) = (d, x) else {
            return;
        };
        let test = format!("{x} % {d} == 0");
        if !self.text(body.span).contains(&test) {
            return;
        }
        // Dividing `x` by `d` anywhere in the loop, even under an `if`, factors it.
        let factors = body
            .stmts
            .iter()
            .any(|s| matches!(&s.kind, StmtKind::While { .. }))
            || self.text(body.span).contains(&format!("{x} /= {d}"));
        let (msg, help) = if factors {
            (
                format!("trial division by `{d}` factors `{x}` by hand"),
                format!("`factor({x})` from `std.math` returns its `(prime, exponent)` pairs"),
            )
        } else {
            (
                format!("trial division by `{d}` walks the divisors of `{x}` by hand"),
                format!("`divisors({x})` from `std.math` lists them; `.len` counts them"),
            )
        };
        self.warn(&TRIAL_DIVISION, stmt.span, msg, help);
    }

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

    fn lca(&mut self, stmt: &Stmt) {
        let StmtKind::While {
            cond: Cond::Expr(c),
            body,
            ..
        } = &stmt.kind
        else {
            return;
        };
        let Some((lhs, CmpOp::Gt, rhs)) = compare(c) else {
            return;
        };
        let (Some((d, u)), Some((d2, v))) = (index1(lhs), index1(rhs)) else {
            return;
        };
        let (Some(u), Some(v)) = (name(u), name(v)) else {
            return;
        };
        let Some(StmtKind::Assign {
            target,
            op: AssignOp::Set,
            value,
        }) = single(body).map(|s| &s.kind)
        else {
            return;
        };
        let Some((p, up)) = index1(value) else {
            return;
        };
        if d != d2 || name(target) != Some(u) || name(up) != Some(u) || u == v {
            return;
        }
        self.warn(
            &LCA_BY_HAND,
            stmt.span,
            format!("climbing `{u}` by `{p}` until `{d}` matches is the LCA, by hand"),
            format!(
                "build the tree once as a `Rooted` (`std.tree`, e.g. `Rooted.from_parents(par)`): `t.lca({u}, {v})` and `t.dist({u}, {v})` answer each query in O(log n)"
            ),
        );
    }

    fn cross(&mut self, expr: &Expr) {
        let Some((l, r)) = binary(expr, BinOp::Sub) else {
            return;
        };
        let (Some((a, b)), Some((c, d))) = (binary(l, BinOp::Mul), binary(r, BinOp::Mul)) else {
            return;
        };
        let (Some((xs, i)), Some((ys, j)), Some((xs2, j2)), Some((ys2, i2))) =
            (index1(a), index1(b), index1(c), index1(d))
        else {
            return;
        };
        let t = |e: &Expr| self.text(e.span);
        if xs != xs2 || ys != ys2 || xs == ys || t(i) != t(i2) || t(j) != t(j2) || t(i) == t(j) {
            return;
        }
        self.warn(
            &CROSS_BY_HAND,
            expr.span,
            format!("`{}` is a cross product of points kept in two lists", self.text(expr.span)),
            "store the points as `Pt` (`std.geometry`): `p.cross(q)` is this product, and `area2(poly)` sums it around a polygon".to_string(),
        );
    }

    fn binom(&mut self, expr: &Expr) {
        // A binomial multiplies and divides; `ps[x] - (ps[n] - ps[n - x])` is a prefix sum.
        let ExprKind::Binary {
            op: BinOp::Mul | BinOp::Div | BinOp::Rem,
            ..
        } = &expr.kind
        else {
            return;
        };
        if self
            .products
            .iter()
            .any(|p| p.start <= expr.span.start && expr.span.end <= p.end)
        {
            return;
        }
        let mut hits = Vec::new();
        indexed(expr, &mut hits);
        for &(f, a) in &hits {
            for &(g, b) in &hits {
                let want = format!("{} - {}", self.text(a.span), self.text(b.span));
                let found = hits
                    .iter()
                    .any(|&(h, c)| h == f && self.text(c.span) == want);
                if g == f && !std::ptr::eq(a, b) && found {
                    self.products.push(expr.span);
                    let (a, b) = (self.text(a.span).to_string(), self.text(b.span).to_string());
                    self.warn(
                        &BINOM_BY_HAND,
                        expr.span,
                        format!("`{f}[{a}]` over `{f}[{b}]` and `{f}[{a} - {b}]` is `{a}` choose `{b}`, by hand"),
                        format!(
                            "`Comb<Z>(limit)` (`std.mod`, with `type Z = Mod<P>`) keeps the factorial tables: `c.choose({a}, {b})` is the binomial, `0` when `{b} > {a}`"
                        ),
                    );
                    return;
                }
            }
        }
        self.binom_inverse_table(expr, &hits);
    }

    /// `fact[a + b] * inv_fact[a] * inv_fact[b]`: the factorials and their
    /// inverses kept in two tables, the inverse one named after its job.
    fn binom_inverse_table(&mut self, expr: &Expr, hits: &[(&str, &Expr)]) {
        let key = |e: &Expr| -> String {
            match &unparen(e).kind {
                ExprKind::Call { args, .. } if args.len() == 1 => self.text(args[0].value.span),
                _ => self.text(e.span),
            }
            .to_string()
        };
        for &(f, a) in hits {
            for (i, &(g, b)) in hits.iter().enumerate() {
                let inverse = g.contains("inv") || g == format!("i{f}");
                if g == f || !inverse {
                    continue;
                }
                for &(g2, c) in &hits[i + 1..] {
                    let (ka, kb, kc) = (key(a), key(b), key(c));
                    let split = ka == format!("{kb} + {kc}")
                        || ka == format!("{kc} + {kb}")
                        || kc == format!("{ka} - {kb}")
                        || kb == format!("{ka} - {kc}");
                    if g2 != g || !split {
                        continue;
                    }
                    self.products.push(expr.span);
                    let k = if kb == format!("{ka} - {kc}") { kc } else { kb };
                    self.warn(
                        &BINOM_BY_HAND,
                        expr.span,
                        format!("`{f}[..]` over two `{g}[..]` entries is `{ka}` choose `{k}`, by hand"),
                        format!(
                            "`Comb<Z>(limit)` (`std.mod`, with `type Z = Mod<P>`) keeps the factorial tables: `c.choose({ka}, {k})` is the binomial, `0` when `{k} > {ka}`"
                        ),
                    );
                    return;
                }
            }
        }
    }

    fn runs(&mut self, stmt: &Stmt) {
        let StmtKind::For { iter, body, .. } = &stmt.kind else {
            return;
        };
        let Some(StmtKind::Expr(e)) = single(body).map(|s| &s.kind) else {
            return;
        };
        let ExprKind::If {
            branches,
            otherwise: Some(els),
        } = &e.kind
        else {
            return;
        };
        let [(_, then)] = branches.as_slice() else {
            return;
        };
        let run = |step: &Block, reset: &Block| -> Option<String> {
            let StmtKind::Assign { target, .. } = &single(step)?.kind else {
                return None;
            };
            let r = name(target)?;
            (is_step(single(step)?, r) && reset.stmts.iter().any(|s| is_reset(s, r)))
                .then(|| r.to_string())
        };
        let Some(r) = run(then, els).or_else(|| run(els, then)) else {
            return;
        };
        let it = self.text(iter.span);
        self.warn(
            &RUNS_BY_HAND,
            stmt.span,
            format!("`{r}` counts runs of equal values by hand"),
            format!(
                "`{it}.runs()` (`std.seq`) yields each run as `(val, len)`: work on whole runs instead of resetting a counter"
            ),
        );
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
            let diag = Diagnostic::warning(
                DIFF_BY_HAND.code,
                Span::new(s1.span.start + b1, s2.span.end + b2),
                format!("`{d}` is a difference array kept by hand"),
            )
            .with_help(format!(
                "`Diff<D, T>()` (`std.prefix`) is one: `{d}.add(l..r, x)` for each update, then `{d}.finish()` is the table"
            ));
            self.out.push(Finding {
                lint: &DIFF_BY_HAND,
                diag,
            });
        }
    }

    fn components(&mut self, stmt: &Stmt) {
        let StmtKind::For { pat, body, .. } = &stmt.kind else {
            return;
        };
        let PatternKind::Name(u) = &pat.kind else {
            return;
        };
        let Some(StmtKind::Expr(e)) = single(body).map(|s| &s.kind) else {
            return;
        };
        let ExprKind::If {
            branches,
            otherwise: None,
        } = &e.kind
        else {
            return;
        };
        let [(Cond::Expr(c), then)] = branches.as_slice() else {
            return;
        };
        let ExprKind::Unary {
            op: UnOp::Not,
            expr: seen,
        } = &unparen(c).kind
        else {
            return;
        };
        let Some((seen, at)) = index1(seen) else {
            return;
        };
        let counts = then.stmts.iter().any(|s| match &s.kind {
            StmtKind::Assign { target, .. } => name(target).is_some_and(|t| is_step(s, t)),
            _ => false,
        });
        let visits = then.stmts.iter().any(|s| match &s.kind {
            StmtKind::Expr(e) => matches!(&e.kind, ExprKind::Call { args, .. }
                if args.iter().any(|a| name(&a.value) == Some(u))),
            _ => false,
        });
        if name(at) != Some(u) || !counts || !visits {
            return;
        }
        self.warn(
            &COMPONENTS_BY_DFS,
            stmt.span,
            format!("a search from each unvisited `{u}` counts components by hand, with `{seen}`"),
            "`let (domain C, comp) = components(adj)` (`std.graph`) labels each vertex with its component; `C.count` is how many there are".to_string(),
        );
    }
}

impl Visitor for HandMath<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let stmts: Vec<_> = stmts.iter().map(|s| (base, s)).collect();
        self.diff(&stmts);
    }

    fn stmt(&mut self, stmt: &Stmt) {
        self.trial_division(stmt);
        self.memo(stmt);
        self.lca(stmt);
        self.runs(stmt);
        self.components(stmt);
    }

    fn expr(&mut self, expr: &Expr) {
        self.cross(expr);
        self.binom(expr);
    }
}
