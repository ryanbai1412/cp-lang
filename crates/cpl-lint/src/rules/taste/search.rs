//! Binary-search loop shapes behind L0520 `hand_rolled_answer_search`.

use super::*;

/// `var x = e` with a plain name.
/// A `while lo < hi:` loop whose body is `let mid = ..` and one
/// `if p: .. else: ..` that moves one bound to `mid`.
pub(super) struct Halving<'a> {
    pub(super) lo: &'a str,
    pub(super) hi: &'a str,
    pub(super) mid: &'a str,
    pub(super) pred: &'a Expr,
    /// `lo = mid` / `hi = mid - 1` on the rounded-up midpoint.
    pub(super) up: bool,
    /// The `if` branch is the one that keeps `pred`'s side (`hi = mid` for
    /// a first search, `lo = mid` for a last one).
    pub(super) keep: bool,
}

pub(super) fn halving_loop(stmt: &Stmt) -> Option<Halving<'_>> {
    let StmtKind::While {
        cond: Cond::Expr(cond),
        body,
        else_body: None,
    } = &stmt.kind
    else {
        return None;
    };
    let ExprKind::Compare { first, rest } = &cond.kind else {
        return None;
    };
    let [(CmpOp::Lt, hi)] = rest.as_slice() else {
        return None;
    };
    let (lo, hi) = (expr_name(first)?, expr_name(hi)?);
    let [let_mid, branch] = body.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Let {
        mutable: false,
        pat: Pattern {
            kind: PatternKind::Name(mid),
            ..
        },
        init: Some(m),
        otherwise: None,
        guard: None,
        ..
    } = &let_mid.kind
    else {
        return None;
    };
    let up = midpoint(m, lo, hi)?;
    let StmtKind::Expr(Expr {
        kind: ExprKind::If {
            branches,
            otherwise: Some(other),
        },
        ..
    }) = &branch.kind
    else {
        return None;
    };
    let [(Cond::Expr(pred), then)] = branches.as_slice() else {
        return None;
    };
    let (t, e) = (bound_move(then, mid)?, bound_move(other, mid)?);
    // A first search sets `hi = mid` and `lo = mid + 1`; a last one sets
    // `lo = mid` and `hi = mid - 1`.
    let (keep_name, keep_step, skip_name, skip_step) =
        if up { (lo, 0, hi, -1) } else { (hi, 0, lo, 1) };
    let keep = if t == (keep_name, keep_step) && e == (skip_name, skip_step) {
        true
    } else if e == (keep_name, keep_step) && t == (skip_name, skip_step) {
        false
    } else {
        return None;
    };
    Some(Halving {
        lo,
        hi,
        mid,
        pred,
        up,
        keep,
    })
}

/// `(lo + hi) / 2` or `lo + (hi - lo) / 2` gives `Some(false)`; with a
/// `+ 1` that rounds up, `Some(true)`.
pub(super) fn midpoint(m: &Expr, lo: &str, hi: &str) -> Option<bool> {
    fn bin(e: &Expr) -> Option<(BinOp, &Expr, &Expr)> {
        match &walk::unparen(e).kind {
            ExprKind::Binary { op, lhs, rhs, .. } => Some((*op, &**lhs, &**rhs)),
            _ => None,
        }
    }
    let is = |e: &Expr, n: &str| expr_name(e) == Some(n);
    // `x + 1` gives `(x, true)`, anything else `(x, false)`.
    let plus_one = |e: &Expr| match bin(e) {
        Some((BinOp::Add, x, one)) if is_int_lit(one, 1) => (x.clone(), true),
        _ => (e.clone(), false),
    };
    let halved = |e: &Expr| match bin(e)? {
        (BinOp::Div, x, two) if is_int_lit(two, 2) => Some(x.clone()),
        (BinOp::Shr, x, one) if is_int_lit(one, 1) => Some(x.clone()),
        _ => None,
    };
    if let Some(sum) = halved(m) {
        let (sum, up) = plus_one(&sum);
        let (op, a, b) = bin(&sum)?;
        return (op == BinOp::Add && ((is(a, lo) && is(b, hi)) || (is(a, hi) && is(b, lo))))
            .then_some(up);
    }
    let (BinOp::Add, base, half) = bin(m)? else {
        return None;
    };
    if !is(base, lo) {
        return None;
    }
    let (diff, up) = plus_one(&halved(half)?);
    let (BinOp::Sub, a, b) = bin(&diff)? else {
        return None;
    };
    (is(a, hi) && is(b, lo)).then_some(up)
}

/// A block that is only `x = mid`, `x = mid + 1` or `x = mid - 1`.
pub(super) fn bound_move<'a>(block: &'a Block, mid: &str) -> Option<(&'a str, i128)> {
    let [Stmt {
        kind:
            StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            },
        ..
    }] = block.stmts.as_slice()
    else {
        return None;
    };
    let name = expr_name(target)?;
    if expr_name(value) == Some(mid) {
        return Some((name, 0));
    }
    let ExprKind::Binary { op, lhs, rhs, .. } = &walk::unparen(value).kind else {
        return None;
    };
    if expr_name(lhs) != Some(mid) || !is_int_lit(rhs, 1) {
        return None;
    }
    match op {
        BinOp::Add => Some((name, 1)),
        BinOp::Sub => Some((name, -1)),
        _ => None,
    }
}

/// The statements right before `stmts[i]` that declare `lo` and `hi`:
/// `var (lo, hi) = (a, b)`, or `var lo = a` then `var hi = b`. Gives the
/// index of the first of them and `a`, `b`.
pub(super) fn bounds_decl<'a>(
    stmts: &[(u32, &'a Stmt)],
    i: usize,
    lo: &str,
    hi: &str,
) -> Option<(usize, &'a Expr, &'a Expr, usize)> {
    let prev = stmts.get(i.checked_sub(1)?)?.1;
    if let StmtKind::Let {
        mutable: true,
        pat: Pattern {
            kind: PatternKind::Tuple(names),
            ..
        },
        ty: None,
        init: Some(init),
        otherwise: None,
        guard: None,
        ..
    } = &prev.kind
    {
        let (ExprKind::Tuple(values), [n0, n1]) = (&init.kind, names.as_slice()) else {
            return None;
        };
        let [a, b] = values.as_slice() else {
            return None;
        };
        fn name(p: &Pattern) -> Option<&str> {
            match &p.kind {
                PatternKind::Name(n) => Some(n.as_str()),
                _ => None,
            }
        }
        return (name(n0) == Some(lo) && name(n1) == Some(hi)).then_some((i - 1, a, b, i - 1));
    }
    let (n1, b) = var_decl(prev)?;
    let (n0, a) = var_decl(stmts.get(i.checked_sub(2)?)?.1)?;
    (n0 == lo && n1 == hi).then_some((i - 2, a, b, i - 1))
}

/// A first search over `0..xs.len` on `xs[mid] >= t` (or `> t`, or a field
/// of `xs[mid]`) is the sorted lookup `xs.first_ge(t)` (or `first_gt`).
/// Gives the method's name and the call.
pub(super) fn sorted_lookup<'e>(
    pred: &'e Expr,
    mid: &str,
    keep: bool,
    end: &str,
    text: impl Fn(&'e Expr) -> &'e str,
) -> Option<(String, String)> {
    let ExprKind::Compare { first, rest } = &walk::unparen(pred).kind else {
        return None;
    };
    let [(op, target)] = rest.as_slice() else {
        return None;
    };
    let method = match (op, keep) {
        (CmpOp::Ge, true) | (CmpOp::Lt, false) => "first_ge",
        (CmpOp::Gt, true) | (CmpOp::Le, false) => "first_gt",
        _ => return None,
    };
    let (elem, field) = match &walk::unparen(first).kind {
        ExprKind::Field { base, name } => (&**base, Some(name.text.as_str())),
        _ => (&**first, None),
    };
    let (list, ix) = plain_index(elem)?;
    if expr_name(ix) != Some(mid) || end != format!("{list}.len") || mentions(text(target), mid) {
        return None;
    }
    let by = field
        .map(|f| format!(", by: v => v.{f}"))
        .unwrap_or_default();
    Some((
        method.to_string(),
        format!("{list}.{method}({}{by})", text(target)),
    ))
}
