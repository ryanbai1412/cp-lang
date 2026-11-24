//! L0522: copy-sort-writeback.

use super::*;
use crate::facts::Facts;
use crate::std_fn;
use crate::Calls;

/// Unwraps a 1-D index `arr[ix]` and returns `(arr_name, ix)`.
pub(super) fn plain_index(e: &Expr) -> Option<(&str, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    let [ix] = indices.as_slice() else {
        return None;
    };
    Some((shapes::name(base)?, ix))
}

pub(super) struct CopyMutate {
    pub(super) target: String,
    pub(super) range: String,
    pub(super) lo: Option<String>,
    pub(super) method: String,
    pub(super) args: String,
    pub(super) wb_idx: usize,
}

pub(super) fn extract_contiguous_slice(
    src: &str,
    base: u32,
    expr: &Expr,
) -> Option<(String, String, Option<String>)> {
    let at = |span: Span| src[(base + span.start) as usize..(base + span.end) as usize].trim();
    let ExprKind::Index {
        base: target,
        indices,
    } = &expr.kind
    else {
        return None;
    };
    if indices.len() != 1 {
        return None;
    }
    let ExprKind::Range {
        lo,
        hi: _,
        inclusive: _,
        step: None,
    } = &indices[0].kind
    else {
        return None;
    };
    let target_text = at(target.span).to_string();
    let range_text = at(indices[0].span).to_string();
    let lo_text = lo.as_ref().map(|e| at(e.span).to_string());
    Some((target_text, range_text, lo_text))
}

pub(super) fn call_args_text(
    src: &str,
    base: u32,
    call_span: Span,
    callee_span: Span,
    args: &[Arg],
) -> String {
    if args.is_empty() {
        return String::new();
    }
    let tail = &src[(base + callee_span.end) as usize..(base + call_span.end) as usize];
    if let (Some(open), Some(close)) = (tail.find('('), tail.rfind(')')) {
        if open < close {
            return tail[open + 1..close].trim().to_string();
        }
    }
    String::new()
}

pub(super) fn match_in_place_mut_stmt(
    src: &str,
    stmt_entry: &(u32, &Stmt),
    sub_name: &str,
    facts: Option<&Facts>,
) -> Option<(&'static str, String)> {
    let (base, stmt) = *stmt_entry;
    let StmtKind::Expr(expr) = &stmt.kind else {
        return None;
    };
    let ExprKind::Call { callee, args, .. } = &expr.kind else {
        return None;
    };
    let ExprKind::Field { base: recv, .. } = &callee.kind else {
        return None;
    };
    if !matches!(&recv.kind, ExprKind::Name(n) if n == sub_name) {
        return None;
    }
    let m = match Calls::new(facts, base).name(expr) {
        Some(std_fn::SORT) => std_fn::SORT,
        Some(std_fn::REVERSE) => std_fn::REVERSE,
        Some(std_fn::ROTATE_LEFT) => std_fn::ROTATE_LEFT,
        Some(std_fn::ROTATE_RIGHT) => std_fn::ROTATE_RIGHT,
        Some("scan_in_place") => "scan_in_place",
        Some("suffix_scan_in_place") => "suffix_scan_in_place",
        _ => return None,
    };
    Some((m, call_args_text(src, base, expr.span, callee.span, args)))
}

pub(super) fn is_zero_lo(lo_text: Option<&str>) -> bool {
    matches!(lo_text, None | Some("0"))
}

pub(super) fn is_sum_with(
    src: &str,
    base: u32,
    expr: &Expr,
    var: &str,
    lo_text: Option<&str>,
) -> bool {
    if is_zero_lo(lo_text) {
        return matches!(&expr.kind, ExprKind::Name(n) if n == var);
    }
    let Some(lo) = lo_text else {
        return false;
    };
    let at = |span: Span| src[(base + span.start) as usize..(base + span.end) as usize].trim();
    match &expr.kind {
        ExprKind::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        } => {
            (matches!(&lhs.kind, ExprKind::Name(n) if n == var) && at(rhs.span) == lo)
                || (matches!(&rhs.kind, ExprKind::Name(n) if n == var) && at(lhs.span) == lo)
        }
        _ => false,
    }
}

pub(super) fn is_diff_with(
    src: &str,
    base: u32,
    expr: &Expr,
    var: &str,
    lo_text: Option<&str>,
) -> bool {
    if is_zero_lo(lo_text) {
        return matches!(&expr.kind, ExprKind::Name(n) if n == var);
    }
    let Some(lo) = lo_text else {
        return false;
    };
    let at = |span: Span| src[(base + span.start) as usize..(base + span.end) as usize].trim();
    matches!(
        &expr.kind,
        ExprKind::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } if matches!(&lhs.kind, ExprKind::Name(n) if n == var) && at(rhs.span) == lo
    )
}

pub(super) fn is_slice_writeback_loop(
    src: &str,
    base: u32,
    stmt: &Stmt,
    target_text: &str,
    sub_name: &str,
    range_text: &str,
    lo_text: Option<&str>,
) -> bool {
    let at = |span: Span| src[(base + span.start) as usize..(base + span.end) as usize].trim();
    let StmtKind::For {
        pat, iter, body, ..
    } = &stmt.kind
    else {
        return false;
    };
    if body.stmts.len() != 1 {
        return false;
    }
    let StmtKind::Assign {
        target,
        op: AssignOp::Set,
        value,
    } = &body.stmts[0].kind
    else {
        return false;
    };
    let ExprKind::Index {
        base: dst_base,
        indices: dst_indices,
    } = &target.kind
    else {
        return false;
    };
    if dst_indices.len() != 1 || at(dst_base.span) != target_text {
        return false;
    }
    let dst_idx = &dst_indices[0];

    // Pattern A: `for (i, v) in sub.items: xs[l + i] = v`
    if let PatternKind::Tuple(parts) = &pat.kind {
        if parts.len() == 2 {
            if let (PatternKind::Name(idx_var), PatternKind::Name(val_var)) =
                (&parts[0].kind, &parts[1].kind)
            {
                let is_items = matches!(
                    &iter.kind,
                    ExprKind::Field { base: ib, name }
                        if matches!(&ib.kind, ExprKind::Name(n) if n == sub_name)
                            && name.text == "items"
                );
                if is_items
                    && is_sum_with(src, base, dst_idx, idx_var, lo_text)
                    && matches!(&value.kind, ExprKind::Name(n) if n == val_var)
                {
                    return true;
                }
            }
        }
        return false;
    }

    let PatternKind::Name(idx_var) = &pat.kind else {
        return false;
    };
    let ExprKind::Index {
        base: rhs_base,
        indices: rhs_indices,
    } = &value.kind
    else {
        return false;
    };
    if rhs_indices.len() != 1 || !matches!(&rhs_base.kind, ExprKind::Name(n) if n == sub_name) {
        return false;
    }
    let rhs_idx = &rhs_indices[0];

    // Pattern B: `for i in l..r: xs[i] = sub[i - l]`
    if at(iter.span) == range_text
        && matches!(&dst_idx.kind, ExprKind::Name(n) if n == idx_var)
        && is_diff_with(src, base, rhs_idx, idx_var, lo_text)
    {
        return true;
    }

    // Pattern C: `for i in 0..sub.len: xs[l + i] = sub[i]` (or `0..r - l`)
    if let ExprKind::Range {
        lo: Some(rlo),
        hi: Some(rhi),
        inclusive: false,
        step: None,
    } = &iter.kind
    {
        if at(rlo.span) == "0"
            && matches!(&rhs_idx.kind, ExprKind::Name(n) if n == idx_var)
            && is_sum_with(src, base, dst_idx, idx_var, lo_text)
        {
            let hi_txt = at(rhi.span);
            if hi_txt == format!("{sub_name}.len")
                || lo_text.is_some_and(|lo| {
                    matches!(
                        &rhi.kind,
                        ExprKind::Binary { op: BinOp::Sub, rhs, .. } if at(rhs.span) == lo
                    )
                })
            {
                return true;
            }
        }
    }

    false
}
