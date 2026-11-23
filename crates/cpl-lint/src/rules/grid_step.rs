//! A grid's edge tested by hand, C++'s `if (r > 0) ... g[r - 1][c]` and
//! `if (0 <= nx && nx < h && ...)`. With the grid over domains, `std.grid`'s
//! `(r, c).step(d)` and `(r, c).adj4` are `none` or skip a step off the grid.
//! `grid_neighbour_step` (L0534) and `grid_bounds_check` (L0535).

use crate::walk::{unparen, walk_stmt, Visitor};
use crate::{File, Finding, GRID_BOUNDS_CHECK, GRID_NEIGHBOUR_STEP};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{BinOp, CmpOp, Cond, Expr, ExprKind, Stmt};

const STEP_HELP: &str = "declare `domain R = 0..h`, `domain C = 0..w` and the grid over `[R, C]`, then read the neighbour with `if let (pr, pc) = (r, c).step(Dir4.up())` (`std.grid`): a step off the grid is `none`, so there is no edge test to get wrong";
const BOUNDS_HELP: &str = "declare the grid over `[R, C]` (`domain R = 0..h`), then step with `for d in Dir4.all()` and `if let (nx, ny) = (x, y).step(d)` (`std.grid`): a step off the grid is `none`, so there are no bounds to write";

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Edges<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Edges<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::If { branches, .. } = &expr.kind else {
                return;
            };
            for (cond, block) in branches {
                let Cond::Expr(cond) = cond else { continue };
                if let Some((x, y)) = bounds_check(cond) {
                    let both = |ix: &[Expr]| {
                        let named = |n: &str| ix.iter().any(|i| name_of(i) == Some(n));
                        ix.len() >= 2 && named(&x) && named(&y)
                    };
                    if block.stmts.iter().any(|s| indexes(s, &both)) {
                        let diag = Diagnostic::warning(
                            GRID_BOUNDS_CHECK.code,
                            cond.span.shift(self.base),
                            format!("`{x}` and `{y}` are tested against the grid's edges by hand"),
                        )
                        .with_help(BOUNDS_HELP);
                        self.out.push(Finding {
                            lint: &GRID_BOUNDS_CHECK,
                            diag,
                        });
                    }
                    continue;
                }
                let Some(name) = edge_test(cond) else {
                    continue;
                };
                let off = |ix: &[Expr]| {
                    ix.len() >= 2
                        && ix.iter().any(|i| match &unparen(i).kind {
                            ExprKind::Binary {
                                op: BinOp::Sub,
                                lhs,
                                rhs,
                                ..
                            } => name_of(lhs) == Some(name.as_str()) && int_of(rhs) == Some(1),
                            _ => false,
                        })
                };
                if !block.stmts.iter().any(|s| indexes(s, &off)) {
                    continue;
                }
                let diag = Diagnostic::warning(
                    GRID_NEIGHBOUR_STEP.code,
                    cond.span.shift(self.base),
                    format!(
                        "`{}` guards a neighbour read at `{name} - 1` by hand",
                        cond.span.shift(self.base).text(self.src)
                    ),
                )
                .with_help(STEP_HELP);
                self.out.push(Finding {
                    lint: &GRID_NEIGHBOUR_STEP,
                    diag,
                });
            }
        }
    }
    let mut v = Edges {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

fn name_of(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Name(n) => Some(n),
        _ => None,
    }
}

fn int_of(e: &Expr) -> Option<i128> {
    match &unparen(e).kind {
        ExprKind::Int(v) => Some(*v),
        _ => None,
    }
}

/// `r > 0` or `r >= 1`: the name whose low edge is tested.
fn edge_test(cond: &Expr) -> Option<String> {
    let ExprKind::Compare { first, rest } = &unparen(cond).kind else {
        return None;
    };
    let [(op, rhs)] = rest.as_slice() else {
        return None;
    };
    let name = name_of(first)?;
    match (op, int_of(rhs)?) {
        (CmpOp::Gt, 0) | (CmpOp::Ge, 1) => Some(name.to_string()),
        _ => None,
    }
}

/// Whether `stmt` indexes an array with indices that satisfy `pred`.
fn indexes(stmt: &Stmt, pred: &dyn Fn(&[Expr]) -> bool) -> bool {
    struct Find<'a> {
        pred: &'a dyn Fn(&[Expr]) -> bool,
        found: bool,
    }
    impl Visitor for Find<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Index { indices, .. } = &expr.kind {
                self.found |= (self.pred)(indices);
            }
        }
    }
    let mut f = Find { pred, found: false };
    walk_stmt(&mut f, stmt);
    f.found
}

/// `x >= 0 and x < h and y >= 0 and y < w`, in any order and spelling
/// (`0 <= x`, `x <= h - 1`): the two names whose edges are tested.
fn bounds_check(expr: &Expr) -> Option<(String, String)> {
    fn leaves<'a>(e: &'a Expr, out: &mut Vec<&'a Expr>) {
        match &unparen(e).kind {
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => {
                leaves(lhs, out);
                leaves(rhs, out);
            }
            _ => out.push(unparen(e)),
        }
    }
    if !matches!(&expr.kind, ExprKind::Binary { op: BinOp::And, .. }) {
        return None;
    }
    let mut parts = Vec::new();
    leaves(expr, &mut parts);
    // (name, low tested, high tested)
    let mut seen: Vec<(String, bool, bool)> = Vec::new();
    for part in parts {
        let ExprKind::Compare { first, rest } = &part.kind else {
            continue;
        };
        let mut prev = &**first;
        for (op, next) in rest {
            let (name, other, op) = match (name_of(prev), name_of(next)) {
                (Some(n), _) if int_of(next) == Some(0) || int_of(next).is_none() => (n, next, *op),
                (_, Some(n)) => (n, prev, flip(*op)),
                _ => {
                    prev = next;
                    continue;
                }
            };
            let low = int_of(other) == Some(0) && matches!(op, CmpOp::Ge);
            let high = int_of(other).is_none() && matches!(op, CmpOp::Lt | CmpOp::Le);
            if low || high {
                match seen.iter_mut().find(|s| s.0 == name) {
                    Some(s) => {
                        s.1 |= low;
                        s.2 |= high;
                    }
                    None => seen.push((name.to_string(), low, high)),
                }
            }
            prev = next;
        }
    }
    let both: Vec<&String> = seen.iter().filter(|s| s.1 && s.2).map(|s| &s.0).collect();
    match both.as_slice() {
        [a, b, ..] => Some(((*a).clone(), (*b).clone())),
        _ => None,
    }
}

/// `a < b` read from `b`'s side is `b > a`.
fn flip(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        other => other,
    }
}
