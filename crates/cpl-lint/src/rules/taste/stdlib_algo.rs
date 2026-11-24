//! L0521-L0522: prefix sums and copy-sort-writeback.

use super::*;
use crate::facts::Facts;
use crate::std_fn;
use crate::Calls;
use cpl_check::Ty;
use cpl_diag::msgs::lint_rules;

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

/// Unwraps a 2-D index written either `tbl[a, b]` or `tbl[a][b]` into `(tbl, a, b)`.
pub(super) fn index_2d(e: &Expr) -> Option<(&str, &Expr, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    match indices.as_slice() {
        [a, b] => Some((shapes::name(base)?, a, b)),
        [b] => {
            let (tbl, a) = plain_index(base)?;
            Some((tbl, a, b))
        }
        _ => None,
    }
}

pub(super) enum PrefixLoopKind<'a> {
    Domain1d { var: &'a str, domain: &'a str },
    RangeZero { var: &'a str, hi: &'a Expr },
    RangeOne,
    Grid2d,
}

pub(super) struct PrefixLoop<'a> {
    pub(super) arr: &'a str,
    pub(super) rhs: &'a Expr,
    pub(super) kind: PrefixLoopKind<'a>,
}

pub(super) fn is_field_of(e: &Expr, var: &str, field: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Field { base, name } if shapes::name(base) == Some(var) && name.text == field
    )
}

pub(super) fn split_add_with_prev<'a>(
    value: &'a Expr,
    arr: &str,
    is_prev: impl Fn(&Expr) -> bool,
) -> Option<&'a Expr> {
    let ExprKind::Binary {
        op: BinOp::Add,
        lhs,
        rhs,
        ..
    } = &walk::unparen(value).kind
    else {
        return None;
    };
    if let Some((a, ix)) = plain_index(lhs) {
        if a == arr && is_prev(ix) {
            return Some(rhs);
        }
    }
    if let Some((a, ix)) = plain_index(rhs) {
        if a == arr && is_prev(ix) {
            return Some(lhs);
        }
    }
    None
}

pub(super) fn prefix_sum_loop(stmt: &Stmt) -> Option<PrefixLoop<'_>> {
    let StmtKind::For {
        pat, iter, body, ..
    } = &stmt.kind
    else {
        return None;
    };
    let PatternKind::Name(var) = &pat.kind else {
        return None;
    };
    let var = var.as_str();
    if body.stmts.len() != 1 {
        return None;
    }
    // Nested 2D prefix loop: `for r in R: for c in C: p[r.r, c.r] = ...`
    if let StmtKind::For {
        pat: inner_pat,
        iter: inner_iter,
        body: inner_body,
        ..
    } = &body.stmts[0].kind
    {
        let PatternKind::Name(cvar) = &inner_pat.kind else {
            return None;
        };
        let cvar = cvar.as_str();
        if inner_body.stmts.len() != 1 {
            return None;
        }
        let StmtKind::Assign {
            target,
            op: AssignOp::Set,
            value,
        } = &inner_body.stmts[0].kind
        else {
            return None;
        };
        let (arr, tr, tc) = index_2d(target)?;
        let domain_2d = is_field_of(tr, var, "r") && is_field_of(tc, cvar, "r");
        let int_2d = is_var_plus_one(tr, var) && is_var_plus_one(tc, cvar);
        if !domain_2d && !int_2d {
            return None;
        }
        if shapes::name(iter).is_none()
            && !matches!(
                &walk::unparen(iter).kind,
                ExprKind::Range { lo: Some(s), .. } if is_int_lit(s, 0)
            )
        {
            return None;
        }
        if shapes::name(inner_iter).is_none()
            && !matches!(
                &walk::unparen(inner_iter).kind,
                ExprKind::Range { lo: Some(s), .. } if is_int_lit(s, 0)
            )
        {
            return None;
        }
        let mut has_up = false;
        let mut has_left = false;
        let mut has_diag = false;
        let mut extra: Option<&Expr> = None;
        let mut terms: Vec<(bool, &Expr)> = Vec::new();
        fn collect_add_sub<'a>(e: &'a Expr, sign: bool, out: &mut Vec<(bool, &'a Expr)>) {
            match &walk::unparen(e).kind {
                ExprKind::Binary {
                    op: BinOp::Add,
                    lhs,
                    rhs,
                    ..
                } => {
                    collect_add_sub(lhs, sign, out);
                    collect_add_sub(rhs, sign, out);
                }
                ExprKind::Binary {
                    op: BinOp::Sub,
                    lhs,
                    rhs,
                    ..
                } => {
                    collect_add_sub(lhs, sign, out);
                    collect_add_sub(rhs, !sign, out);
                }
                _ => out.push((sign, e)),
            }
        }
        collect_add_sub(value, true, &mut terms);
        for (pos, term) in terms {
            if let Some((a, ir, ic)) = index_2d(term) {
                if a == arr {
                    let r_prev = if domain_2d {
                        is_field_of(ir, var, "l")
                    } else {
                        shapes::name(ir) == Some(var)
                    };
                    let c_prev = if domain_2d {
                        is_field_of(ic, cvar, "l")
                    } else {
                        shapes::name(ic) == Some(cvar)
                    };
                    let r_next = if domain_2d {
                        is_field_of(ir, var, "r")
                    } else {
                        is_var_plus_one(ir, var)
                    };
                    let c_next = if domain_2d {
                        is_field_of(ic, cvar, "r")
                    } else {
                        is_var_plus_one(ic, cvar)
                    };
                    if pos && r_prev && c_next {
                        has_up = true;
                        continue;
                    }
                    if pos && r_next && c_prev {
                        has_left = true;
                        continue;
                    }
                    if !pos && r_prev && c_prev {
                        has_diag = true;
                        continue;
                    }
                }
            }
            if pos && extra.is_none() {
                extra = Some(term);
            } else {
                return None;
            }
        }
        if has_up && has_left && has_diag {
            return Some(PrefixLoop {
                arr,
                rhs: extra?,
                kind: PrefixLoopKind::Grid2d,
            });
        }
        return None;
    }

    let StmtKind::Assign {
        target,
        op: AssignOp::Set,
        value,
    } = &body.stmts[0].kind
    else {
        return None;
    };
    let (arr, t_ix) = plain_index(target)?;

    // 1. `for i in D: p[i.r] = p[i.l] + rhs`
    if let Some(domain) = shapes::name(iter) {
        if is_field_of(t_ix, var, "r") {
            let rhs = split_add_with_prev(value, arr, |ix| is_field_of(ix, var, "l"))?;
            return Some(PrefixLoop {
                arr,
                rhs,
                kind: PrefixLoopKind::Domain1d { var, domain },
            });
        }
    }

    // 2. `for i in 0..n: p[i + 1] = p[i] + rhs`
    if let ExprKind::Range {
        lo: Some(lo),
        hi: Some(hi),
        inclusive: false,
        step: None,
    } = &walk::unparen(iter).kind
    {
        if is_int_lit(lo, 0) && is_var_plus_one(t_ix, var) {
            let rhs = split_add_with_prev(value, arr, |ix| shapes::name(ix) == Some(var))?;
            return Some(PrefixLoop {
                arr,
                rhs,
                kind: PrefixLoopKind::RangeZero { var, hi },
            });
        }
    }

    // 3. `for i in 1..=n: p[i] = p[i - 1] + rhs`
    if let ExprKind::Range {
        lo: Some(lo),
        hi: Some(_),
        step: None,
        ..
    } = &walk::unparen(iter).kind
    {
        if is_int_lit(lo, 1) && shapes::name(t_ix) == Some(var) {
            let rhs = split_add_with_prev(value, arr, |ix| is_var_minus_one(ix, var))?;
            return Some(PrefixLoop {
                arr,
                rhs,
                kind: PrefixLoopKind::RangeOne,
            });
        }
    }
    None
}

pub(super) fn is_zero_init(e: &Expr) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Int(0) => true,
        ExprKind::Field { name, .. } if name.text == "zero" => true,
        ExprKind::Call { callee, args } if args.len() == 1 => {
            matches!(&walk::unparen(&args[0].value).kind, ExprKind::Int(0))
                && shapes::name(callee).is_some()
        }
        _ => false,
    }
}

pub(super) fn prefix_zero_decl<'a>(
    stmts: &[(u32, &'a Stmt)],
    loop_idx: usize,
    arr: &str,
    kind: &PrefixLoopKind<'_>,
    facts: Option<&Facts>,
) -> Option<(usize, u32, &'a Stmt)> {
    if loop_idx == 0 {
        return None;
    }
    let idx = loop_idx - 1;
    let (base, stmt) = stmts[idx];
    let StmtKind::Let {
        mutable: true,
        pat,
        ty,
        init: Some(val),
        otherwise: None,
        guard: None,
    } = &stmt.kind
    else {
        return None;
    };
    let PatternKind::Name(name) = &pat.kind else {
        return None;
    };
    if name != arr {
        return None;
    }
    let declared = || ty.as_ref().and(facts)?.place_type(pat.span.shift(base));
    match kind {
        PrefixLoopKind::Domain1d { domain, .. } => {
            let Some(Ty::Array { index, .. }) = declared() else {
                return None;
            };
            let [Ty::Pos(d)] = index.as_slice() else {
                return None;
            };
            (*d == format!("{domain}.cuts") && is_zero_init(val)).then_some((idx, base, stmt))
        }
        PrefixLoopKind::RangeZero { .. } => {
            if ty.is_some() {
                let Some(Ty::Array { index, .. }) = declared() else {
                    return None;
                };
                let is_zero_range = matches!(index.as_slice(), [Ty::Range { lo: Some(0), .. }]);
                if is_zero_range && is_zero_init(val) {
                    return Some((idx, base, stmt));
                }
                return None;
            }
            let call = walk::unparen(val);
            let ExprKind::Call { callee, args } = &call.kind else {
                return None;
            };
            let ExprKind::Field { base: recv, .. } = &walk::unparen(callee).kind else {
                return None;
            };
            if shapes::name(recv) == Some("List")
                && Calls::new(facts, base).is(call, std_fn::FILLED)
                && args.len() == 2
                && is_zero_init(&args[1].value)
            {
                Some((idx, base, stmt))
            } else {
                None
            }
        }
        PrefixLoopKind::RangeOne | PrefixLoopKind::Grid2d => None,
    }
}

/// Checks that every occurrence of `arr` in `later` is a read-only `arr[ix]`
/// expression (no mutation, no bare pass, no method call, no shadowing).
/// Returns `(stmt_base, index_expr_span, ix_span)` for every `arr[ix]`.
pub(super) fn collect_pure_index_reads(
    later: &[(u32, &Stmt)],
    arr: &str,
) -> Option<Vec<(u32, Span, Span)>> {
    struct Collector<'a> {
        arr: &'a str,
        base: u32,
        ok: bool,
        allowed_bases: HashSet<(u32, u32)>,
        reads: Vec<(u32, Span, Span)>,
    }
    impl Visitor for Collector<'_> {
        fn bind(&mut self, name: &str, _span: Span, _how: Bind) {
            if name == self.arr {
                self.ok = false;
            }
        }
        fn stmt(&mut self, stmt: &Stmt) {
            if !self.ok {
                return;
            }
            if let StmtKind::Assign { target, .. } = &stmt.kind {
                if shapes::name(target) == Some(self.arr)
                    || plain_index(target).is_some_and(|(a, _)| a == self.arr)
                {
                    self.ok = false;
                }
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if !self.ok {
                return;
            }
            match &expr.kind {
                ExprKind::Index { base, indices } if shapes::name(base) == Some(self.arr) => {
                    if indices.len() != 1 {
                        self.ok = false;
                        return;
                    }
                    let unp = walk::unparen(base);
                    self.allowed_bases
                        .insert((self.base + unp.span.start, self.base + unp.span.end));
                    self.reads.push((self.base, expr.span, indices[0].span));
                }
                ExprKind::Name(id) if id == self.arr => {
                    let key = (self.base + expr.span.start, self.base + expr.span.end);
                    if !self.allowed_bases.contains(&key) {
                        self.ok = false;
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Collector {
        arr,
        base: 0,
        ok: true,
        allowed_bases: HashSet::new(),
        reads: Vec::new(),
    };
    for &(base, stmt) in later {
        c.base = base;
        walk::walk_stmt(&mut c, stmt);
        if !c.ok {
            return None;
        }
    }
    Some(c.reads)
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

/// A running total stored at each step of a loop: `for x in a: run += x; out.push(run)`,
/// `for i in I: run += a[i]; s[i] = run`, or `for x in a: ps.push(ps[ps.len - 1] + x)` (also
/// `ps.last! + x`). Returns the table, the loop variable and the term added each step.
pub(super) fn running_total_loop<'s>(
    stmt: &'s Stmt,
    calls: Calls,
) -> Option<(&'s str, &'s str, &'s Expr)> {
    let StmtKind::For { pat, body, .. } = &stmt.kind else {
        return None;
    };
    let PatternKind::Name(var) = &pat.kind else {
        return None;
    };
    let var = var.as_str();
    match body.stmts.as_slice() {
        [one] => {
            let StmtKind::Expr(e) = &one.kind else {
                return None;
            };
            let (table, value) = push_call(e, calls)?;
            let ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
                ..
            } = &walk::unparen(value).kind
            else {
                return None;
            };
            if is_last_of(lhs, table) {
                Some((table, var, rhs))
            } else if is_last_of(rhs, table) {
                Some((table, var, lhs))
            } else {
                None
            }
        }
        [step, store] => {
            let StmtKind::Assign {
                target,
                op: AssignOp::Binary(BinOp::Add),
                value: added,
            } = &step.kind
            else {
                return None;
            };
            let acc = shapes::name(target)?;
            let table = match &store.kind {
                StmtKind::Expr(e) => {
                    let (table, v) = push_call(e, calls)?;
                    (shapes::name(v) == Some(acc)).then_some(table)?
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    let (table, ix) = plain_index(target)?;
                    (shapes::name(ix) == Some(var) && shapes::name(value) == Some(acc))
                        .then_some(table)?
                }
                _ => return None,
            };
            (table != acc).then_some((table, var, added))
        }
        _ => None,
    }
}

fn push_call<'e>(e: &'e Expr, calls: Calls) -> Option<(&'e str, &'e Expr)> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, .. } = &callee.kind else {
        return None;
    };
    match args.as_slice() {
        [arg] if arg.name.is_none() && calls.is(e, std_fn::PUSH) => {
            Some((shapes::name(base)?, &arg.value))
        }
        _ => None,
    }
}

/// `e` is `table[table.len - 1]` or `table.last!`.
fn is_last_of(e: &Expr, table: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Unwrap(inner) => is_field_of(inner, table, std_fn::LAST),
        ExprKind::Index { base, indices } => {
            shapes::name(base) == Some(table)
                && matches!(indices.as_slice(), [i] if matches!(
                    &walk::unparen(i).kind,
                    ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
                        if is_field_of(lhs, table, std_fn::LEN) && is_int_lit(rhs, 1)
                ))
        }
        _ => false,
    }
}

impl Taste<'_> {
    /// A running total stored each step (L0521, help only): the loop also does what the
    /// table's reads need, so `cpl fix` leaves the rewrite to the contestant.
    pub(super) fn running_total(&mut self, base: u32, stmt: &Stmt) {
        let Some((table, var, added)) = running_total_loop(stmt, Calls::new(self.facts, base))
        else {
            return;
        };
        let added = &self.src[(base + added.span.start) as usize..(base + added.span.end) as usize];
        if !mentions(added, var) || has_try(stmt) {
            return;
        }
        let diag = lint_rules::HAND_ROLLED_PREFIX_SUM.warning(stmt.span, &[&table])
        .with_help(
            "`a.scan(+)` builds every running total in O(n) and walks like the list (`.min()`, `.any(..)`, `inversions`); `.prefix(cut)` reads one and `.fold(range)` sums a range in O(1)",
        );
        self.push(&HAND_ROLLED_PREFIX_SUM, diag);
    }
}
