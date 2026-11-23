//! L0517-L0522: DSU, sieve, binary lifting, prefix sums, copy-sort-writeback.

use super::*;

/// Flags hand-rolled union-find (L0517), prime/SPF sieve (L0518), and
/// binary-lifting tables (L0519) that duplicate `std.dsu`, `std.math`, and
/// `std.tree`.
pub(super) fn hand_rolled_stdlib_algos(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, lint: &'static crate::Lint, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint,
                diag: Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { cond, body, .. } if is_dsu_path_halving_loop(cond, body) => {
                    self.warn(
                        &HAND_ROLLED_DSU,
                        stmt.span,
                        "hand-rolled union-find (DSU) path compression",
                        "use `Dsu`, `DsuOf`, `ParityDsu`, or `PotentialDsu` from `std.dsu`",
                    );
                }
                StmtKind::Fn(decl) => {
                    if let FnBody::Block(body) = &decl.body {
                        if is_dsu_recursive_or_two_pass_fn(decl, body) {
                            self.warn(
                                &HAND_ROLLED_DSU,
                                stmt.span,
                                "hand-rolled union-find (DSU) `find` function",
                                "use `Dsu`, `DsuOf`, `ParityDsu`, or `PotentialDsu` from `std.dsu`",
                            );
                        }
                    }
                }
                StmtKind::For { pat, body, .. } => {
                    if let PatternKind::Name(var) = &pat.kind {
                        if is_sieve_loop(var, body) {
                            self.warn(
                                &HAND_ROLLED_SIEVE,
                                stmt.span,
                                "hand-rolled prime/SPF sieve loop",
                                "use `Sieve` or `sieve_multiplicative` from `std.math`",
                            );
                        }
                        if is_binary_lifting_loop(var, body) {
                            self.warn(
                                &HAND_ROLLED_BINARY_LIFTING,
                                stmt.span,
                                "hand-rolled binary lifting table",
                                "use `Rooted.lca` / `Rooted.ancestor` or `JumpTree` from `std.tree`",
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

/// Unwraps a 1-D index `arr[ix]` and returns `(arr_name, ix)`.
pub(super) fn plain_index(e: &Expr) -> Option<(&str, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    let [ix] = indices.as_slice() else {
        return None;
    };
    Some((expr_name(base)?, ix))
}

/// Matches `arr[x] != x` (or `x != arr[x]`), returning `(arr, x)`.
pub(super) fn dsu_not_root_cond(cond: &Cond) -> Option<(&str, &str)> {
    let e = cond_expr(cond)?;
    let (lhs, CmpOp::Ne, rhs) = single_cmp(e)? else {
        return None;
    };
    if let (Some((arr, ix)), Some(v)) = (plain_index(lhs), expr_name(rhs)) {
        if expr_name(ix) == Some(v) {
            return Some((arr, v));
        }
    }
    if let (Some(v), Some((arr, ix))) = (expr_name(lhs), plain_index(rhs)) {
        if expr_name(ix) == Some(v) {
            return Some((arr, v));
        }
    }
    None
}

/// `while up[x] != x:` containing `up[x] = up[up[x]]` (path halving).
pub(super) fn is_dsu_path_halving_loop(cond: &Cond, body: &Block) -> bool {
    let Some((arr, var)) = dsu_not_root_cond(cond) else {
        return false;
    };
    struct Check<'a> {
        arr: &'a str,
        var: &'a str,
        halved: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            {
                if let Some((a1, ix1)) = plain_index(target) {
                    if a1 == self.arr && expr_name(ix1) == Some(self.var) {
                        if let Some((a2, ix2)) = plain_index(value) {
                            if a2 == self.arr {
                                if let Some((a3, ix3)) = plain_index(ix2) {
                                    if a3 == self.arr && expr_name(ix3) == Some(self.var) {
                                        self.halved = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        arr,
        var,
        halved: false,
    };
    walk_block(&mut c, body);
    c.halved
}

/// Recursive path-compressing DSU `find(x)` (`if up[x] == x: ...; up[x] = find(up[x])`)
/// or two-pass iterative DSU `find(v)` (chase `up[r]` to root, then compress `up[x] = r`).
pub(super) fn is_dsu_recursive_or_two_pass_fn(decl: &FnDecl, body: &Block) -> bool {
    if decl.params.len() != 1 {
        return false;
    }
    let fn_name = decl.name.text.as_str();
    let param = decl.params[0].name.text.as_str();

    // Case 1: recursive `fn find(x)`: calls `find(up[x])` and writes `up[x] = ...`.
    struct Rec<'a> {
        fn_name: &'a str,
        param: &'a str,
        rec_arr: Option<String>,
        writes_arr: Vec<String>,
    }
    impl Visitor for Rec<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { callee, args } = &expr.kind {
                if expr_name(callee) == Some(self.fn_name) && args.len() == 1 {
                    if let Some((arr, ix)) = plain_index(&args[0].value) {
                        if expr_name(ix) == Some(self.param) {
                            self.rec_arr = Some(arr.to_string());
                        }
                    }
                }
            }
        }
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                ..
            } = &stmt.kind
            {
                if let Some((arr, ix)) = plain_index(target) {
                    if expr_name(ix) == Some(self.param) {
                        self.writes_arr.push(arr.to_string());
                    }
                }
            }
        }
    }
    let mut r = Rec {
        fn_name,
        param,
        rec_arr: None,
        writes_arr: Vec::new(),
    };
    walk_block(&mut r, body);
    if let Some(arr) = r.rec_arr {
        if r.writes_arr.iter().any(|w| w == &arr) {
            return true;
        }
    }

    // Case 2: two-pass iterative `find(v)`: two `while` loops over the same parent array
    // where the second writes `up[x] = r`.
    let mut chase_arrs: Vec<String> = Vec::new();
    let mut compress_arrs: Vec<String> = Vec::new();
    for stmt in &body.stmts {
        if let StmtKind::While { cond, body: wb, .. } = &stmt.kind {
            let cond_arr = if let Some((arr, _)) = dsu_not_root_cond(cond) {
                Some(arr.to_string())
            } else if let Cond::Let { value, .. } = cond {
                plain_index(value).map(|(arr, _)| arr.to_string())
            } else {
                None
            };
            if let Some(arr) = cond_arr {
                let writes_arr = wb.stmts.iter().any(|s| {
                    matches!(
                        &s.kind,
                        StmtKind::Assign { target, op: AssignOp::Set, .. }
                            if plain_index(target).map(|(a, _)| a) == Some(arr.as_str())
                    )
                });
                if writes_arr {
                    compress_arrs.push(arr);
                } else {
                    chase_arrs.push(arr);
                }
            }
        }
    }
    compress_arrs.iter().any(|a| chase_arrs.contains(a))
}

/// Matches a prime/SPF sieve outer loop `for i in ...:`:
/// - Eratosthenes: initializes `j = i * i` (or `2 * i`), increments `j += i`,
///   and writes `arr[j] = false` / `arr[D(j)] = false` / `arr[j] = i` / `arr[j] = 0`.
/// - Linear (Euler) sieve: inner `for p in primes:` writing `arr[i * p] = p` (or `false`).
pub(super) fn is_sieve_loop(outer_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        outer: &'a str,
        multiple_vars: Vec<String>,
        stepped_vars: Vec<String>,
        marked_vars: Vec<String>,
        linear_sieve: bool,
    }
    fn is_mul_of_var(e: &Expr, var: &str) -> bool {
        let ExprKind::Binary {
            op: BinOp::Mul,
            lhs,
            rhs,
            ..
        } = &walk::unparen(e).kind
        else {
            return false;
        };
        (expr_name(lhs) == Some(var) && (expr_name(rhs) == Some(var) || is_int_lit(rhs, 2)))
            || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 2))
    }
    fn unwrap_domain_wrap(e: &Expr) -> &Expr {
        let u = walk::unparen(e);
        if let ExprKind::Call { args, .. } = &u.kind {
            if args.len() == 1 {
                return walk::unparen(&args[0].value);
            }
        }
        u
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Let {
                    pat:
                        Pattern {
                            kind: PatternKind::Name(v),
                            ..
                        },
                    init: Some(init),
                    ..
                } if is_mul_of_var(init, self.outer) => {
                    self.multiple_vars.push(v.clone());
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Binary(BinOp::Add),
                    value,
                } => {
                    if let Some(v) = expr_name(target) {
                        if expr_name(value) == Some(self.outer) {
                            self.stepped_vars.push(v.to_string());
                        }
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    if let Some((_, ix)) = plain_index(target) {
                        let inner = unwrap_domain_wrap(ix);
                        if let Some(v) = expr_name(inner) {
                            let marks = matches!(
                                &walk::unparen(value).kind,
                                ExprKind::Bool(_) | ExprKind::Int(0)
                            ) || expr_name(value) == Some(self.outer);
                            if marks {
                                self.marked_vars.push(v.to_string());
                            }
                        }
                    }
                }
                StmtKind::For {
                    pat:
                        Pattern {
                            kind: PatternKind::Name(p),
                            ..
                        },
                    body: inner_body,
                    ..
                } => {
                    for s in &inner_body.stmts {
                        if let StmtKind::Assign {
                            target,
                            op: AssignOp::Set,
                            value,
                        } = &s.kind
                        {
                            if let Some((_, ix)) = plain_index(target) {
                                let inner = unwrap_domain_wrap(ix);
                                if let ExprKind::Binary {
                                    op: BinOp::Mul,
                                    lhs,
                                    rhs,
                                    ..
                                } = &inner.kind
                                {
                                    let is_ip = (expr_name(lhs) == Some(self.outer)
                                        && expr_name(rhs) == Some(p.as_str()))
                                        || (expr_name(rhs) == Some(self.outer)
                                            && expr_name(lhs) == Some(p.as_str()));
                                    let writes_p = expr_name(value) == Some(p.as_str())
                                        || matches!(
                                            walk::unparen(value).kind,
                                            ExprKind::Bool(false)
                                        );
                                    if is_ip && writes_p {
                                        self.linear_sieve = true;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        outer: outer_var,
        multiple_vars: Vec::new(),
        stepped_vars: Vec::new(),
        marked_vars: Vec::new(),
        linear_sieve: false,
    };
    walk_block(&mut c, body);
    c.linear_sieve
        || c.multiple_vars
            .iter()
            .any(|v| c.stepped_vars.contains(v) && c.marked_vars.contains(v))
}

/// Unwraps a 2-D index written either `tbl[a, b]` or `tbl[a][b]` into `(tbl, a, b)`.
pub(super) fn index_2d(e: &Expr) -> Option<(&str, &Expr, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    match indices.as_slice() {
        [a, b] => Some((expr_name(base)?, a, b)),
        [b] => {
            let (tbl, a) = plain_index(base)?;
            Some((tbl, a, b))
        }
        _ => None,
    }
}

/// Matches a binary lifting recurrence inside `for k in 1..lg:`:
/// `up[k, v] = up[k - 1, up[k - 1, v]]` or `up[k][v] = up[k - 1][up[k - 1][v]]`
/// (in either `(k, v)` or `(v, k)` index order).
pub(super) fn is_binary_lifting_loop(level_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        k: &'a str,
        found: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            else {
                return;
            };
            let Some((tbl, la, lb)) = index_2d(target) else {
                return;
            };
            let Some((rtbl, ra, rb)) = index_2d(value) else {
                return;
            };
            if tbl != rtbl {
                return;
            }
            // Orientation 1: `up[k, v] = up[k - 1, up[k - 1, v]]`
            if expr_name(la) == Some(self.k) && is_var_minus_one(ra, self.k) {
                if let Some((itbl, ia, ib)) = index_2d(rb) {
                    if itbl == tbl && is_var_minus_one(ia, self.k) && expr_name(ib) == expr_name(lb)
                    {
                        self.found = true;
                        return;
                    }
                }
            }
            // Orientation 2: `up[v, k] = up[up[v, k - 1], k - 1]`
            if expr_name(lb) == Some(self.k) && is_var_minus_one(rb, self.k) {
                if let Some((itbl, ia, ib)) = index_2d(ra) {
                    if itbl == tbl && is_var_minus_one(ib, self.k) && expr_name(ia) == expr_name(la)
                    {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut c = Check {
        k: level_var,
        found: false,
    };
    walk_block(&mut c, body);
    c.found
}

/// Bottom-up segment-tree build/pull loop: `for i in (1..n).rev:` writing
/// `t[i] = ... t[2 * i] ... t[2 * i + 1]` on the same array `t`.
pub(super) fn is_segtree_build_pull_loop(var: &str, body: &Block) -> bool {
    struct Check<'a> {
        var: &'a str,
        found: bool,
    }
    fn rhs_reads_both_children(e: &Expr, arr: &str, var: &str) -> bool {
        struct Reads<'a> {
            arr: &'a str,
            var: &'a str,
            left: bool,
            right: bool,
        }
        impl Visitor for Reads<'_> {
            fn expr(&mut self, expr: &Expr) {
                if let Some((a, ix)) = plain_index(expr) {
                    if a == self.arr {
                        if is_double(ix, self.var) {
                            self.left = true;
                        }
                        if is_double_plus_one(ix, self.var) {
                            self.right = true;
                        }
                    }
                }
            }
        }
        let mut r = Reads {
            arr,
            var,
            left: false,
            right: false,
        };
        walk::walk_expr(&mut r, e);
        r.left && r.right
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            {
                if let Some((arr, ix)) = plain_index(target) {
                    if expr_name(ix) == Some(self.var)
                        && rhs_reads_both_children(value, arr, self.var)
                    {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut c = Check { var, found: false };
    walk_block(&mut c, body);
    c.found
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
        ExprKind::Field { base, name } if expr_name(base) == Some(var) && name.text == field
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
        if expr_name(iter).is_none()
            && !matches!(
                &walk::unparen(iter).kind,
                ExprKind::Range { lo: Some(s), .. } if is_int_lit(s, 0)
            )
        {
            return None;
        }
        if expr_name(inner_iter).is_none()
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
                        expr_name(ir) == Some(var)
                    };
                    let c_prev = if domain_2d {
                        is_field_of(ic, cvar, "l")
                    } else {
                        expr_name(ic) == Some(cvar)
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
    if let Some(domain) = expr_name(iter) {
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
            let rhs = split_add_with_prev(value, arr, |ix| expr_name(ix) == Some(var))?;
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
        if is_int_lit(lo, 1) && expr_name(t_ix) == Some(var) {
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
                && expr_name(callee).is_some()
        }
        _ => false,
    }
}

pub(super) fn prefix_zero_decl<'a>(
    src: &str,
    stmts: &[(u32, &'a Stmt)],
    loop_idx: usize,
    arr: &str,
    kind: &PrefixLoopKind<'_>,
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
    match kind {
        PrefixLoopKind::Domain1d { domain, .. } => {
            let Some(ty) = ty else {
                return None;
            };
            let TypeKind::Array { index, .. } = &ty.kind else {
                return None;
            };
            if index.len() != 1 {
                return None;
            }
            let dim_src =
                &src[(base + index[0].span.start) as usize..(base + index[0].span.end) as usize];
            if dim_src == format!("{domain}.cuts") && is_zero_init(val) {
                Some((idx, base, stmt))
            } else {
                None
            }
        }
        PrefixLoopKind::RangeZero { .. } => {
            if let Some(ty) = ty {
                let TypeKind::Array { index, .. } = &ty.kind else {
                    return None;
                };
                if index.len() != 1 {
                    return None;
                }
                let is_zero_range = matches!(
                    &index[0].kind,
                    TypeKind::Range { lo: Some(lo), hi: Some(_), .. } if is_int_lit(lo, 0)
                );
                if is_zero_range && is_zero_init(val) {
                    return Some((idx, base, stmt));
                }
                return None;
            }
            let ExprKind::Call { callee, args } = &walk::unparen(val).kind else {
                return None;
            };
            let ExprKind::Field {
                base: recv,
                name: mname,
            } = &walk::unparen(callee).kind
            else {
                return None;
            };
            if expr_name(recv) == Some("List")
                && mname.text == "filled"
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
                if expr_name(target) == Some(self.arr)
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
                ExprKind::Index { base, indices } if expr_name(base) == Some(self.arr) => {
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
    pub(super) method: &'static str,
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
) -> Option<(&'static str, String)> {
    let (base, stmt) = *stmt_entry;
    let StmtKind::Expr(expr) = &stmt.kind else {
        return None;
    };
    let ExprKind::Call { callee, args, .. } = &expr.kind else {
        return None;
    };
    let ExprKind::Field {
        base: recv,
        name: method,
    } = &callee.kind
    else {
        return None;
    };
    if !matches!(&recv.kind, ExprKind::Name(n) if n == sub_name) {
        return None;
    }
    let m = match method.text.as_str() {
        "sort" => "sort",
        "reverse" => "reverse",
        "rotate_left" => "rotate_left",
        "rotate_right" => "rotate_right",
        "scan_in_place" => "scan_in_place",
        "suffix_scan_in_place" => "suffix_scan_in_place",
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
pub(super) fn running_total_loop(stmt: &Stmt) -> Option<(&str, &str, &Expr)> {
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
            let (table, value) = push_call(e)?;
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
            let acc = expr_name(target)?;
            let table = match &store.kind {
                StmtKind::Expr(e) => {
                    let (table, v) = push_call(e)?;
                    (expr_name(v) == Some(acc)).then_some(table)?
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    let (table, ix) = plain_index(target)?;
                    (expr_name(ix) == Some(var) && expr_name(value) == Some(acc))
                        .then_some(table)?
                }
                _ => return None,
            };
            (table != acc).then_some((table, var, added))
        }
        _ => None,
    }
}

fn push_call(e: &Expr) -> Option<(&str, &Expr)> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name } = &callee.kind else {
        return None;
    };
    match args.as_slice() {
        [arg] if name.text == "push" && arg.name.is_none() => Some((expr_name(base)?, &arg.value)),
        _ => None,
    }
}

/// `e` is `table[table.len - 1]` or `table.last!`.
fn is_last_of(e: &Expr, table: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Unwrap(inner) => is_field_of(inner, table, "last"),
        ExprKind::Index { base, indices } => {
            expr_name(base) == Some(table)
                && matches!(indices.as_slice(), [i] if matches!(
                    &walk::unparen(i).kind,
                    ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
                        if is_field_of(lhs, table, "len") && is_int_lit(rhs, 1)
                ))
        }
        _ => false,
    }
}

impl Taste<'_> {
    /// A running total stored each step (L0521, help only): the loop also does what the
    /// table's reads need, so `cpl fix` leaves the rewrite to the contestant.
    pub(super) fn running_total(&mut self, base: u32, stmt: &Stmt) {
        let Some((table, var, added)) = running_total_loop(stmt) else {
            return;
        };
        let added = &self.src[(base + added.span.start) as usize..(base + added.span.end) as usize];
        if !mentions(added, var) || has_try(stmt) {
            return;
        }
        // A `Prefix` answers element and range reads but can't be walked,
        // printed or passed on as a list, so a table used whole keeps its loop.
        if !only_indexed_after(self.src, (base + stmt.span.end) as usize, table) {
            return;
        }
        let diag = Diagnostic::warning(
            HAND_ROLLED_PREFIX_SUM.code,
            stmt.span,
            format!("`{table}` is a running total built by hand: use `Prefix(...)`"),
        )
        .with_help(
            "`Prefix(a)` (or `a.scan(+)`) builds every prefix sum in O(n); `.prefix(cut)` reads one and `.fold(range)` sums a range in O(1)",
        );
        self.push(&HAND_ROLLED_PREFIX_SUM, diag);
    }
}
