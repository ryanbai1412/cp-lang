//! L0514 `hand_rolled_range_tree`.

use super::*;

/// Flags hand-rolled Fenwick loops (`p += p & -p` / `p -= p & -p` with table
/// indexing) and hand-rolled iterative or recursive segment trees (`2 * p` /
/// `2 * p + 1` with halving or range-guarded midpoint recursion).
pub(super) fn hand_rolled_range_trees(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint: &HAND_ROLLED_RANGE_TREE,
                diag: Diagnostic::warning(HAND_ROLLED_RANGE_TREE.code, span.shift(self.base), msg)
                    .with_help(format!(
                        "{help}; its index arithmetic is already right and its docs give each cost"
                    )),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { body, .. } => {
                    if let Some(var) = fenwick_step_var(body) {
                        if body_indexes_by(body, &var) {
                            self.warn(
                                stmt.span,
                                "hand-rolled Fenwick loop",
                                "use `Fenwick`, `DualFenwick`, or `RangeFenwick` from `std.fenwick`",
                            );
                        }
                    } else if is_iterative_segtree_loop(body) {
                        self.warn(
                            stmt.span,
                            "hand-rolled segment tree loop",
                            "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                        );
                    }
                }
                StmtKind::For { pat, body, .. } => {
                    if let PatternKind::Name(var) = &pat.kind {
                        if is_segtree_build_pull_loop(var, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled segment tree pull loop",
                                "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                            );
                        }
                    }
                }
                StmtKind::Fn(decl) => {
                    if let FnBody::Block(body) = &decl.body {
                        if is_recursive_segtree_fn(decl, body) {
                            self.warn(
                                decl.name.span,
                                "hand-rolled recursive segment tree",
                                "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut scan = Scan {
        base: 0,
        findings: Vec::new(),
    };
    file.walk(&mut scan, |s, base| s.base = base);
    scan.findings
}

pub(super) fn expr_name(e: &Expr) -> Option<&str> {
    match &walk::unparen(e).kind {
        ExprKind::Name(s) => Some(s.as_str()),
        _ => None,
    }
}

pub(super) fn is_int_lit(e: &Expr, want: i128) -> bool {
    matches!(walk::unparen(e).kind, ExprKind::Int(v) if v == want)
}

/// Matches `v & -v` or `-v & v`.
pub(super) fn is_lowbit(e: &Expr, var: &str) -> bool {
    let ExprKind::Binary {
        op: BinOp::BitAnd,
        lhs,
        rhs,
        ..
    } = &walk::unparen(e).kind
    else {
        return false;
    };
    let is_neg = |x: &Expr| {
        matches!(
            &walk::unparen(x).kind,
            ExprKind::Unary { op: UnOp::Neg, expr } if expr_name(expr) == Some(var)
        )
    };
    (expr_name(lhs) == Some(var) && is_neg(rhs)) || (expr_name(rhs) == Some(var) && is_neg(lhs))
}

/// Matches `v | (v + 1)` or `(v & (v + 1)) - 1` (0-indexed Fenwick step).
pub(super) fn is_zero_indexed_bit_step(e: &Expr, var: &str) -> bool {
    let is_v_plus_1 = |x: &Expr| {
        matches!(
            &walk::unparen(x).kind,
            ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
                if (expr_name(lhs) == Some(var) && is_int_lit(rhs, 1))
                    || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 1))
        )
    };
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::BitOr,
            lhs,
            rhs,
            ..
        } => {
            (expr_name(lhs) == Some(var) && is_v_plus_1(rhs))
                || (expr_name(rhs) == Some(var) && is_v_plus_1(lhs))
        }
        ExprKind::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } if is_int_lit(rhs, 1) => matches!(
            &walk::unparen(lhs).kind,
            ExprKind::Binary { op: BinOp::BitAnd, lhs: a, rhs: b, .. }
                if (expr_name(a) == Some(var) && is_v_plus_1(b))
                    || (expr_name(b) == Some(var) && is_v_plus_1(a))
        ),
        _ => false,
    }
}

/// Returns the stepped variable if `body` updates a variable with a BIT step.
pub(super) fn fenwick_step_var(body: &Block) -> Option<String> {
    for stmt in &body.stmts {
        if let StmtKind::Assign { target, op, value } = &stmt.kind {
            let Some(var) = expr_name(target) else {
                continue;
            };
            match op {
                AssignOp::Binary(BinOp::Add | BinOp::Sub) if is_lowbit(value, var) => {
                    return Some(var.to_string());
                }
                AssignOp::Set => {
                    if let ExprKind::Binary {
                        op: BinOp::Add | BinOp::Sub,
                        lhs,
                        rhs,
                        ..
                    } = &walk::unparen(value).kind
                    {
                        if expr_name(lhs) == Some(var) && is_lowbit(rhs, var) {
                            return Some(var.to_string());
                        }
                    }
                    if is_zero_indexed_bit_step(value, var) {
                        return Some(var.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

pub(super) fn walk_block(v: &mut dyn Visitor, block: &Block) {
    for s in &block.stmts {
        walk::walk_stmt(v, s);
    }
}

pub(super) fn expr_mentions(e: &Expr, var: &str) -> bool {
    struct Find<'a>(&'a str, bool);
    impl Visitor for Find<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Name(s) = &expr.kind {
                if s == self.0 {
                    self.1 = true;
                }
            }
        }
    }
    let mut f = Find(var, false);
    walk::walk_expr(&mut f, e);
    f.1
}

pub(super) fn body_indexes_by(body: &Block, var: &str) -> bool {
    struct Find<'a>(&'a str, bool);
    impl Visitor for Find<'_> {
        fn expr(&mut self, expr: &Expr) {
            if self.1 {
                return;
            }
            if let ExprKind::Index { indices, .. } = &expr.kind {
                if indices.iter().any(|ix| expr_mentions(ix, self.0)) {
                    self.1 = true;
                }
            }
        }
    }
    let mut f = Find(var, false);
    walk_block(&mut f, body);
    f.1
}

/// Matches `2 * v`, `v * 2`, or `v << 1`.
pub(super) fn is_double(e: &Expr, var: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Mul,
            lhs,
            rhs,
            ..
        } => {
            (expr_name(lhs) == Some(var) && is_int_lit(rhs, 2))
                || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 2))
        }
        ExprKind::Binary {
            op: BinOp::Shl,
            lhs,
            rhs,
            ..
        } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 1),
        _ => false,
    }
}

/// Matches `2 * v + 1`, `v * 2 + 1`, `(v << 1) + 1`, or `(v << 1) | 1`.
pub(super) fn is_double_plus_one(e: &Expr, var: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Add | BinOp::BitOr,
            lhs,
            rhs,
            ..
        } => {
            (is_double(lhs, var) && is_int_lit(rhs, 1))
                || (is_double(rhs, var) && is_int_lit(lhs, 1))
        }
        _ => false,
    }
}

pub(super) fn is_halve_assign(stmt: &Stmt) -> Option<String> {
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return None;
    };
    let var = expr_name(target)?;
    let halved = match op {
        AssignOp::Binary(BinOp::Shr) => is_int_lit(value, 1),
        AssignOp::Binary(BinOp::Div) => is_int_lit(value, 2),
        AssignOp::Set => match &walk::unparen(value).kind {
            ExprKind::Binary {
                op: BinOp::Shr,
                lhs,
                rhs,
                ..
            } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 1),
            ExprKind::Binary {
                op: BinOp::Div,
                lhs,
                rhs,
                ..
            } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 2),
            _ => false,
        },
        _ => false,
    };
    halved.then(|| var.to_string())
}

/// Bottom-up segment-tree loop: halves a node index (`p >>= 1` / `p /= 2`)
/// while indexing children (`2 * p` and `2 * p + 1`), OR halves two endpoints
/// (`l >>= 1`, `r >>= 1`) while testing parity (`& 1`) and indexing by them.
pub(super) fn is_iterative_segtree_loop(body: &Block) -> bool {
    let mut halved = Vec::new();
    for stmt in &body.stmts {
        if let Some(v) = is_halve_assign(stmt) {
            if !halved.contains(&v) {
                halved.push(v);
            }
        }
    }
    if halved.is_empty() {
        return false;
    }
    struct Checks<'a> {
        vars: &'a [String],
        has_left: bool,
        has_right: bool,
        has_parity: bool,
        has_index: bool,
    }
    impl Visitor for Checks<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Index { indices, .. } = &expr.kind {
                for ix in indices {
                    for v in self.vars {
                        if is_double(ix, v) {
                            self.has_left = true;
                        }
                        if is_double_plus_one(ix, v) {
                            self.has_right = true;
                        }
                        if expr_mentions(ix, v) {
                            self.has_index = true;
                        }
                    }
                }
            }
            if let ExprKind::Binary {
                op: BinOp::BitAnd,
                lhs,
                rhs,
                ..
            } = &expr.kind
            {
                for v in self.vars {
                    if (expr_name(lhs) == Some(v.as_str()) && is_int_lit(rhs, 1))
                        || (expr_name(rhs) == Some(v.as_str()) && is_int_lit(lhs, 1))
                    {
                        self.has_parity = true;
                    }
                }
            }
        }
    }
    let mut c = Checks {
        vars: &halved,
        has_left: false,
        has_right: false,
        has_parity: false,
        has_index: false,
    };
    walk_block(&mut c, body);
    (c.has_left && c.has_right) || (halved.len() >= 2 && c.has_parity && c.has_index)
}

/// Top-down recursive segment-tree function: computes a midpoint `(lo + hi) / 2`,
/// recurses on `2 * node` and `2 * node + 1`, and guards a query/update range
/// or branches on a point index (`pos < mid` / `qr <= lo or hi <= ql`).
pub(super) fn is_recursive_segtree_fn(decl: &FnDecl, body: &Block) -> bool {
    if decl.params.len() < 3 {
        return false;
    }
    let param_names: Vec<&str> = decl.params.iter().map(|p| p.name.text.as_str()).collect();
    struct Check<'a> {
        fn_name: &'a str,
        params: &'a [&'a str],
        has_mid: bool,
        has_left_rec: bool,
        has_right_rec: bool,
        has_range_or_point_guard: bool,
    }
    impl Visitor for Check<'_> {
        fn expr(&mut self, expr: &Expr) {
            match &expr.kind {
                ExprKind::If { branches, .. } => {
                    for (cond, _) in branches {
                        if let Cond::Expr(e) = cond {
                            match &walk::unparen(e).kind {
                                ExprKind::Binary {
                                    op: BinOp::Or | BinOp::And,
                                    lhs,
                                    rhs,
                                    ..
                                } if matches!(
                                    walk::unparen(lhs).kind,
                                    ExprKind::Compare { .. }
                                ) && matches!(
                                    walk::unparen(rhs).kind,
                                    ExprKind::Compare { .. }
                                ) =>
                                {
                                    self.has_range_or_point_guard = true;
                                }
                                ExprKind::Compare { first, rest } if rest.len() == 1 => {
                                    let (op, rhs) = &rest[0];
                                    if matches!(op, CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge) {
                                        let l = expr_name(first);
                                        let r = expr_name(rhs);
                                        if matches!(l, Some("mid") | Some("m"))
                                            || matches!(r, Some("mid") | Some("m"))
                                        {
                                            self.has_range_or_point_guard = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ExprKind::Binary {
                    op: BinOp::Div,
                    lhs,
                    rhs,
                    ..
                } if is_int_lit(rhs, 2)
                    && matches!(
                        walk::unparen(lhs).kind,
                        ExprKind::Binary { op: BinOp::Add, .. }
                    ) =>
                {
                    self.has_mid = true;
                }
                ExprKind::Binary {
                    op: BinOp::Shr,
                    lhs,
                    rhs,
                    ..
                } if is_int_lit(rhs, 1)
                    && matches!(
                        walk::unparen(lhs).kind,
                        ExprKind::Binary { op: BinOp::Add, .. }
                    ) =>
                {
                    self.has_mid = true;
                }
                ExprKind::Call { callee, args } if expr_name(callee) == Some(self.fn_name) => {
                    for arg in args {
                        for p in self.params {
                            if is_double(&arg.value, p) {
                                self.has_left_rec = true;
                            }
                            if is_double_plus_one(&arg.value, p) {
                                self.has_right_rec = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        fn_name: decl.name.text.as_str(),
        params: &param_names,
        has_mid: false,
        has_left_rec: false,
        has_right_rec: false,
        has_range_or_point_guard: false,
    };
    walk_block(&mut c, body);
    c.has_mid && c.has_left_rec && c.has_right_rec && c.has_range_or_point_guard
}
