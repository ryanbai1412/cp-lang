//! L0516 `hand_rolled_string_algo`: KMP, Z-function, polynomial hash, trie.

use super::*;

/// Flags hand-rolled KMP prefix-function, Z-function, polynomial rolling-hash,
/// and trie insertion loops that duplicate `std.strings`.
pub(super) fn hand_rolled_string_algos(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint: &HAND_ROLLED_STRING_ALGO,
                diag: Diagnostic::warning(HAND_ROLLED_STRING_ALGO.code, span.shift(self.base), msg)
                    .with_help(help),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::For { pat, body, .. } => {
                    let var = match &pat.kind {
                        PatternKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    };
                    if let Some(v) = var {
                        if is_kmp_prefix_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled prefix-function (KMP) loop",
                                "use `prefix_function` or `find_all` from `std.strings`",
                            );
                            return;
                        }
                        if is_z_function_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled Z-function loop",
                                "use `z_function` from `std.strings`",
                            );
                            return;
                        }
                        if is_poly_hash_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled polynomial rolling hash",
                                "use `RollingHash` or `Hash2D` from `std.strings`",
                            );
                            return;
                        }
                    }
                    if is_trie_insert_loop(body) {
                        self.warn(
                            stmt.span,
                            "hand-rolled trie loop",
                            "use `Trie` or `AhoCorasick` from `std.strings`",
                        );
                    }
                }
                StmtKind::While { body, .. } if is_trie_insert_loop(body) => {
                    self.warn(
                        stmt.span,
                        "hand-rolled trie loop",
                        "use `Trie` or `AhoCorasick` from `std.strings`",
                    );
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

/// Extracts `(base_name, index_expr)` from `arr[idx]` or `arr.at(idx)`.
pub(super) fn seq_index_parts(e: &Expr) -> Option<(&str, &Expr)> {
    match &walk::unparen(e).kind {
        ExprKind::Index { base, indices } if indices.len() == 1 => {
            Some((expr_name(base)?, &indices[0]))
        }
        ExprKind::Call { callee, args } if args.len() == 1 => {
            if let ExprKind::Field { base, name } = &walk::unparen(callee).kind {
                if name.text == "at" {
                    return Some((expr_name(base)?, &args[0].value));
                }
            }
            None
        }
        _ => None,
    }
}

/// Extracts `(lhs, op, rhs)` from a single binary comparison `lhs op rhs`.
pub(super) fn single_cmp(e: &Expr) -> Option<(&Expr, CmpOp, &Expr)> {
    if let ExprKind::Compare { first, rest } = &walk::unparen(e).kind {
        if rest.len() == 1 {
            return Some((first, rest[0].0, &rest[0].1));
        }
    }
    None
}

/// Matches `var - 1`.
pub(super) fn is_var_minus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
            if expr_name(lhs) == Some(var) && is_int_lit(rhs, 1)
    )
}

/// Matches `var + 1` or `1 + var`.
pub(super) fn is_var_plus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
            if (expr_name(lhs) == Some(var) && is_int_lit(rhs, 1))
                || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 1))
    )
}

/// Collects all top-level `and`-conjuncts of a condition.
pub(super) fn collect_and<'a>(e: &'a Expr, out: &mut Vec<&'a Expr>) {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            collect_and(lhs, out);
            collect_and(rhs, out);
        }
        _ => out.push(walk::unparen(e)),
    }
}

pub(super) fn cond_expr(cond: &Cond) -> Option<&Expr> {
    match cond {
        Cond::Expr(e) => Some(e),
        _ => None,
    }
}

/// Matches a KMP prefix-function loop: contains a `while j > 0 and s[...] != s[...]`
/// whose body steps `j = pi[j - 1]`, and writes `pi[i] = ...` in the loop body.
pub(super) fn is_kmp_prefix_loop(for_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        for_var: &'a str,
        kmp_tables: Vec<String>,
        written_tables: Vec<String>,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { cond, body, .. } => {
                    let Some(cond_e) = cond_expr(cond) else {
                        return;
                    };
                    let mut fallback: Option<(String, String)> = None;
                    for s in &body.stmts {
                        if let StmtKind::Assign {
                            target,
                            op: AssignOp::Set,
                            value,
                        } = &s.kind
                        {
                            if let Some(j) = expr_name(target) {
                                if let Some((pi, idx)) = seq_index_parts(value) {
                                    if is_var_minus_one(idx, j) {
                                        fallback = Some((j.to_string(), pi.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    if let Some((j, pi)) = fallback {
                        let mut parts = Vec::new();
                        collect_and(cond_e, &mut parts);
                        let has_gt0 = parts.iter().any(|p| {
                            matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Gt, rhs))
                                    if expr_name(lhs) == Some(j.as_str()) && is_int_lit(rhs, 0)
                            ) || matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Lt, rhs))
                                    if is_int_lit(lhs, 0) && expr_name(rhs) == Some(j.as_str())
                            )
                        });
                        let has_seq_ne = parts.iter().any(|p| {
                            matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Ne, rhs))
                                    if matches!(
                                        (seq_index_parts(lhs), seq_index_parts(rhs)),
                                        (Some((s1, _)), Some((s2, _))) if s1 == s2
                                    )
                            )
                        });
                        if has_gt0 && has_seq_ne {
                            self.kmp_tables.push(pi);
                        }
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    ..
                } => {
                    if let Some((arr, idx)) = seq_index_parts(target) {
                        if expr_name(idx) == Some(self.for_var) {
                            self.written_tables.push(arr.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        for_var,
        kmp_tables: Vec::new(),
        written_tables: Vec::new(),
    };
    walk_block(&mut c, body);
    c.kmp_tables.iter().any(|t| c.written_tables.contains(t))
}

/// Checks if two expressions have the same simple shape (`var` or `arr[var]`).
pub(super) fn same_simple_expr(a: &Expr, b: &Expr) -> bool {
    let a = walk::unparen(a);
    let b = walk::unparen(b);
    match (&a.kind, &b.kind) {
        (ExprKind::Name(x), ExprKind::Name(y)) => x == y,
        (
            ExprKind::Index {
                base: t1,
                indices: i1,
            },
            ExprKind::Index {
                base: t2,
                indices: i2,
            },
        ) if i1.len() == 1 && i2.len() == 1 => {
            expr_name(t1) == expr_name(t2)
                && expr_name(t1).is_some()
                && same_simple_expr(&i1[0], &i2[0])
        }
        _ => false,
    }
}

/// Matches `for_var + k` or `k + for_var`.
pub(super) fn is_for_plus_expr(e: &Expr, for_var: &str, k: &Expr) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
            if (expr_name(lhs) == Some(for_var) && same_simple_expr(rhs, k))
                || (expr_name(rhs) == Some(for_var) && same_simple_expr(lhs, k))
    )
}

/// Matches a Z-function extension loop: `while ... and s[k] == s[i + k]: k += 1`.
pub(super) fn is_z_function_loop(for_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        for_var: &'a str,
        found: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            if let StmtKind::While { cond, body, .. } = &stmt.kind {
                let Some(cond_e) = cond_expr(cond) else {
                    return;
                };
                let mut parts = Vec::new();
                collect_and(cond_e, &mut parts);
                for p in parts {
                    if let Some((lhs, CmpOp::Eq, rhs)) = single_cmp(p) {
                        if let (Some((s1, i1)), Some((s2, i2))) =
                            (seq_index_parts(lhs), seq_index_parts(rhs))
                        {
                            if s1 != s2 {
                                continue;
                            }
                            let k = if is_for_plus_expr(i2, self.for_var, i1) {
                                Some(i1)
                            } else if is_for_plus_expr(i1, self.for_var, i2) {
                                Some(i2)
                            } else {
                                None
                            };
                            if let Some(k) = k {
                                let inc = body.stmts.iter().any(|s| match &s.kind {
                                    StmtKind::Assign {
                                        target,
                                        op: AssignOp::Binary(BinOp::Add),
                                        value,
                                    } => same_simple_expr(target, k) && is_int_lit(value, 1),
                                    StmtKind::Assign {
                                        target,
                                        op: AssignOp::Set,
                                        value,
                                    } => {
                                        same_simple_expr(target, k)
                                            && matches!(
                                                &walk::unparen(value).kind,
                                                ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
                                                    if (same_simple_expr(lhs, k) && is_int_lit(rhs, 1))
                                                        || (same_simple_expr(rhs, k) && is_int_lit(lhs, 1))
                                            )
                                    }
                                    _ => false,
                                });
                                if inc {
                                    self.found = true;
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        for_var,
        found: false,
    };
    walk_block(&mut c, body);
    c.found
}

/// Unwraps an outer `% mod` if present, returning `(inner, has_mod)`.
pub(super) fn strip_mod(e: &Expr) -> (&Expr, bool) {
    let u = walk::unparen(e);
    if let ExprKind::Binary {
        op: BinOp::Rem,
        lhs,
        ..
    } = &u.kind
    {
        (walk::unparen(lhs), true)
    } else {
        (u, false)
    }
}

pub(super) fn is_small_radix(e: &Expr) -> bool {
    matches!(
        walk::unparen(e).kind,
        ExprKind::Int(2 | 8 | 10 | 16 | 26 | 100)
    )
}

/// Checks if an expression looks like a character/byte code (`Int(s[i])`, `ch.ord`, `'a'` offset).
pub(super) fn expr_looks_like_char_code(e: &Expr) -> bool {
    struct Check {
        found: bool,
    }
    impl Visitor for Check {
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Char(_) => self.found = true,
                ExprKind::Field { name, .. } if name.text == "ord" => self.found = true,
                ExprKind::Call { callee, args }
                    if expr_name(callee) == Some("Int")
                        && args.len() == 1
                        && matches!(
                            walk::unparen(&args[0].value).kind,
                            ExprKind::Index { .. } | ExprKind::Name(_)
                        ) =>
                {
                    self.found = true;
                }
                _ => {}
            }
        }
    }
    let mut c = Check { found: false };
    walk::walk_expr(&mut c, e);
    c.found
}

/// Matches a polynomial rolling-hash recurrence inside a `for` loop:
/// `h[i + 1] = (h[i] * B + term) % M` (or `h = (h * B + term) % M`) with either
/// a matching power recurrence `pw[i + 1] = (pw[i] * B) % M` in the same loop or
/// a char-code term with a non-decimal base.
pub(super) fn is_poly_hash_loop(for_var: &str, body: &Block) -> bool {
    let mut hash_bases: Vec<(String, bool, bool)> = Vec::new();
    let mut pow_bases: Vec<String> = Vec::new();

    let base_key = |e: &Expr| -> Option<String> {
        let u = walk::unparen(e);
        if is_small_radix(u) {
            return None;
        }
        match &u.kind {
            ExprKind::Name(n) => Some(n.clone()),
            ExprKind::Int(v) if *v >= 27 => Some(v.to_string()),
            _ => None,
        }
    };

    for s in &body.stmts {
        let StmtKind::Assign {
            target,
            op: AssignOp::Set,
            value,
        } = &s.kind
        else {
            continue;
        };
        let (inner, has_mod) = strip_mod(value);
        let prev_matches = |cand: &Expr| -> bool {
            if let Some((arr, idx)) = seq_index_parts(target) {
                if let Some((arr2, idx2)) = seq_index_parts(cand) {
                    if arr != arr2 {
                        return false;
                    }
                    return (is_var_plus_one(idx, for_var) && expr_name(idx2) == Some(for_var))
                        || (expr_name(idx) == Some(for_var) && is_var_minus_one(idx2, for_var));
                }
                false
            } else if let Some(h) = expr_name(target) {
                expr_name(cand) == Some(h)
            } else {
                false
            }
        };

        match &inner.kind {
            ExprKind::Binary {
                op: BinOp::Mul,
                lhs,
                rhs,
                ..
            } => {
                if prev_matches(lhs) {
                    if let Some(b) = base_key(rhs) {
                        pow_bases.push(b);
                    }
                } else if prev_matches(rhs) {
                    if let Some(b) = base_key(lhs) {
                        pow_bases.push(b);
                    }
                }
            }
            ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
                ..
            } => {
                let mut check_pair = |mul_side: &Expr, term_side: &Expr| {
                    if let ExprKind::Binary {
                        op: BinOp::Mul,
                        lhs: ml,
                        rhs: mr,
                        ..
                    } = &walk::unparen(mul_side).kind
                    {
                        let base = if prev_matches(ml) {
                            base_key(mr)
                        } else if prev_matches(mr) {
                            base_key(ml)
                        } else {
                            None
                        };
                        if let Some(b) = base {
                            hash_bases.push((b, has_mod, expr_looks_like_char_code(term_side)));
                        }
                    }
                };
                check_pair(lhs, rhs);
                check_pair(rhs, lhs);
            }
            _ => {}
        }
    }

    hash_bases
        .iter()
        .any(|(b, has_mod, char_term)| pow_bases.contains(b) || (*has_mod && *char_term))
}

/// Checks if an expression looks like a trie child-table lookup on `cur`:
/// `ch[cur][c]`, `ch[cur, c]`, or `nodes[cur].next[c]`.
/// Returns `(table_key, cursor_var)`.
pub(super) fn trie_child_slot(e: &Expr) -> Option<(String, String)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    if indices.len() == 2 {
        let tbl = expr_name(base)?;
        let cur = expr_name(&indices[0])?;
        return Some((tbl.to_string(), cur.to_string()));
    }
    if indices.len() != 1 {
        return None;
    }
    match &walk::unparen(base).kind {
        ExprKind::Index {
            base: outer,
            indices: outer_idx,
        } if outer_idx.len() == 1 => {
            let tbl = expr_name(outer)?;
            let cur = expr_name(&outer_idx[0])?;
            Some((tbl.to_string(), cur.to_string()))
        }
        ExprKind::Field {
            base: node_expr,
            name,
        } => {
            let (arr, idx) = seq_index_parts(node_expr)?;
            let cur = expr_name(idx)?;
            Some((format!("{arr}.{}", name.text), cur.to_string()))
        }
        _ => None,
    }
}

/// Matches a hand-rolled trie insertion loop: allocates a missing child slot
/// (`nodes[cur].kid[c] ??= ...` or `if ch[cur][c] == -1 / 0 / none: ch[cur][c] = ...`)
/// and advances `cur = ...` from that same child table.
pub(super) fn is_trie_insert_loop(body: &Block) -> bool {
    struct Check {
        alloc_slots: Vec<(String, String)>,
        advance_vars: Vec<String>,
    }
    impl Visitor for Check {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign {
                    target,
                    op: AssignOp::Binary(BinOp::Coalesce),
                    ..
                } => {
                    if let Some(slot) = trie_child_slot(target) {
                        self.alloc_slots.push(slot);
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    if let Some(cur) = expr_name(target) {
                        let inner = match &walk::unparen(value).kind {
                            ExprKind::Unwrap(e) => e,
                            _ => value,
                        };
                        if let Some((_, c2)) = trie_child_slot(inner) {
                            if c2 == cur {
                                self.advance_vars.push(cur.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::If { branches, .. } = &expr.kind {
                for (cond, then_b) in branches {
                    let Some(cond_e) = cond_expr(cond) else {
                        continue;
                    };
                    let slot = match single_cmp(cond_e) {
                        Some((lhs, CmpOp::Eq, rhs))
                            if matches!(
                                walk::unparen(rhs).kind,
                                ExprKind::Int(0)
                                    | ExprKind::None
                                    | ExprKind::Unary { op: UnOp::Neg, .. }
                            ) =>
                        {
                            trie_child_slot(lhs)
                        }
                        _ => None,
                    };
                    if let Some(slot) = slot {
                        let writes_slot = then_b.stmts.iter().any(|s| {
                            matches!(
                                &s.kind,
                                StmtKind::Assign { target, op: AssignOp::Set, .. }
                                    if trie_child_slot(target) == Some(slot.clone())
                            )
                        });
                        if writes_slot {
                            self.alloc_slots.push(slot);
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        alloc_slots: Vec::new(),
        advance_vars: Vec::new(),
    };
    walk_block(&mut c, body);
    c.alloc_slots
        .iter()
        .any(|(_, cur)| c.advance_vars.contains(cur))
}
