//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `@allow` afterwards.

mod bounds;
mod fields;
mod floats;
mod imports;
mod lists;
mod memo;
mod mod_div;
mod scopes;
mod taste;
mod types;

use crate::{Config, File, Finding};

pub(crate) fn run(file: &File, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(scopes::check(file));
    findings.extend(types::check(file));
    findings.extend(floats::check(file));
    findings.extend(imports::check(file));
    findings.extend(fields::check(file));
    findings.extend(memo::check(file, config.memory_mb));
    findings.extend(lists::check(file));
    findings.extend(mod_div::check(file));
    findings.extend(taste::check(file));
    findings
}

/// Each immutable `let` without a written type, and each loop variable over
/// a range, whose value the declared bounds prove, as the name's offset and
/// the inclusive range. Inside a fn, a typed range parameter is a bound too.
pub(crate) fn proven_ranges(file: &File) -> Vec<(u32, i128, i128)> {
    proven(file).names
}

/// Each `+`, `-`, `*`, `/`, `%` and `**` whose operands the declared bounds prove, with the
/// range of its value, under the same scoping as [`proven_ranges`].
pub(crate) fn proven_expr_ranges(file: &File) -> Vec<crate::ProvenExpr> {
    proven(file).exprs
}

pub(crate) fn proven_loops(file: &File) -> Vec<crate::ProvenLoop> {
    proven(file).loops
}

struct Proven {
    names: Vec<(u32, i128, i128)>,
    exprs: Vec<crate::ProvenExpr>,
    loops: Vec<crate::ProvenLoop>,
    /// How many times the innermost enclosing loops run in all.
    around: Option<u128>,
}

impl Default for Proven {
    fn default() -> Proven {
        Proven {
            names: Vec::new(),
            exprs: Vec::new(),
            loops: Vec::new(),
            around: Some(1),
        }
    }
}

fn proven(file: &File) -> Proven {
    let bounds = bounds::Bounds::of(file);
    let mut out = Proven::default();
    // A fn declared above a top-level `var` can assign it too.
    let top: Vec<&cpl_syntax::ast::Stmt> = file.top_level().map(|(_, s)| s).collect();
    // Top-level statements see the ranges of the names bound above them; a
    // fn body may run after any of them, so it sees only the declared ones.
    let mut above = bounds.clone();
    for (base, stmt) in file.top_level() {
        let scope = match stmt.kind {
            cpl_syntax::ast::StmtKind::Fn(_)
            | cpl_syntax::ast::StmtKind::Impl { .. }
            | cpl_syntax::ast::StmtKind::Struct(_) => &mut bounds.clone(),
            _ => &mut above,
        };
        ranges_in(base, std::slice::from_ref(stmt), &top, scope, &mut out);
    }
    out.names.sort();
    out.names.dedup();
    out.exprs.sort_by_key(|e| e.op);
    out.exprs.dedup_by_key(|e| e.op);
    out.loops.sort_by_key(|l| l.head);
    out.loops.dedup_by_key(|l| l.head);
    out
}

/// The arithmetic in `expr` the bounds prove, without entering anything
/// that binds names of its own (lambdas, comprehensions, match arms, blocks).
fn arith(base: u32, expr: &cpl_syntax::ast::Expr, bounds: &bounds::Bounds, out: &mut Vec<crate::ProvenExpr>) {
    use cpl_syntax::ast::{BinOp, ExprKind};
    match &expr.kind {
        ExprKind::Binary { op, op_span, lhs, rhs } => {
            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow
            ) {
                // A constant says nothing, unless a relation made it one (`lo - hi` after a search).
                let constant = |e| bounds.expr(e).is_some_and(|v: bounds::Interval| v.lo == v.hi);
                let shown = |v: &bounds::Interval| v.lo != v.hi || !(constant(lhs) && constant(rhs));
                if let Some(v) = bounds.expr(expr).filter(shown) {
                    let int = i64::MAX as i128;
                    let leaves = v.lo < -int - 1 || v.hi > int;
                    out.push(crate::ProvenExpr {
                        op: (base + op_span.start, base + op_span.end),
                        expr: (base + expr.span.start, base + expr.span.end),
                        lo: v.lo,
                        hi: v.hi,
                        overflow: leaves.then(|| bounds.overflow(expr)).flatten(),
                    });
                }
            }
            arith(base, lhs, bounds, out);
            arith(base, rhs, bounds, out);
        }
        ExprKind::Unary { expr: inner, .. }
        | ExprKind::Paren(inner)
        | ExprKind::Try(inner)
        | ExprKind::Unwrap(inner)
        | ExprKind::Field { base: inner, .. } => arith(base, inner, bounds, out),
        ExprKind::Compare { first, rest } => {
            arith(base, first, bounds, out);
            rest.iter().for_each(|(_, e)| arith(base, e, bounds, out));
        }
        ExprKind::Range { lo, hi, step, .. } => {
            for e in [lo, hi, step].into_iter().flatten() {
                arith(base, e, bounds, out);
            }
        }
        ExprKind::Call { callee, args } => {
            arith(base, callee, bounds, out);
            args.iter().for_each(|a| arith(base, &a.value, bounds, out));
        }
        ExprKind::Index { base: seq, indices } => {
            arith(base, seq, bounds, out);
            indices.iter().for_each(|e| arith(base, e, bounds, out));
        }
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            items.iter().for_each(|e| arith(base, e, bounds, out));
        }
        _ => {}
    }
}

/// `stmts` is a block, or one top-level statement whose `var`s are in
/// scope across all of `top`.
fn ranges_in(
    base: u32,
    stmts: &[cpl_syntax::ast::Stmt],
    top: &[&cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
    proven: &mut Proven,
) {
    use cpl_syntax::ast::{ExprKind, StmtKind};
    for (i, stmt) in stmts.iter().enumerate() {
        let compound = match &stmt.kind {
            StmtKind::While { .. } => true,
            StmtKind::Expr(e) => matches!(e.kind, ExprKind::If { .. }),
            _ => false,
        };
        let entry = compound.then(|| bounds.clone());
        let carry = carried(stmt, bounds);
        // Before the statement, so what a loop's exit proves survives it.
        for name in bounds.related() {
            if assigned_in(&name, std::slice::from_ref(stmt)) {
                bounds.unrelate(&name);
            }
        }
        ranges_of_stmt(base, stmts, i, top, bounds, entry.as_ref(), proven);
        if let Some((name, relations, now)) = carry {
            for (other, d) in relations {
                bounds.relate(&name, &other, d);
            }
            if let Some(v) = now {
                bounds.set_now(&name, v);
            }
        }
    }
}

/// What assigning a plain name keeps true: after `x = y + c`, `x - z` is
/// `y - z` shifted by `c` for each relation of `y` (`y` may be `x`), and
/// `x - y` is `c`. After `x = e`, `x` holds a value of `e` until it is next
/// assigned.
type Carried = (String, Vec<(String, bounds::Interval)>, Option<bounds::Interval>);

fn carried(stmt: &cpl_syntax::ast::Stmt, bounds: &bounds::Bounds) -> Option<Carried> {
    use cpl_syntax::ast::{AssignOp, BinOp, ExprKind, StmtKind};
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return None;
    };
    let ExprKind::Name(x) = &target.kind else {
        return None;
    };
    if bounds.written_by_fn(x) {
        return None;
    }
    let int = |e: &cpl_syntax::ast::Expr| match e.kind {
        ExprKind::Int(c) => Some(c),
        _ => None,
    };
    let shift = match (op, &value.kind) {
        (AssignOp::Set, ExprKind::Name(y)) => Some((y.as_str(), 0)),
        (AssignOp::Set, ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }) => match (&lhs.kind, &rhs.kind) {
            (ExprKind::Name(y), _) => int(rhs).map(|c| (y.as_str(), c)),
            (_, ExprKind::Name(y)) => int(lhs).map(|c| (y.as_str(), c)),
            _ => None,
        },
        (AssignOp::Set, ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }) => match &lhs.kind {
            ExprKind::Name(y) => int(rhs).and_then(|c| Some((y.as_str(), c.checked_neg()?))),
            _ => None,
        },
        (AssignOp::Binary(BinOp::Add), _) => int(value).map(|c| (x.as_str(), c)),
        (AssignOp::Binary(BinOp::Sub), _) => int(value).and_then(|c| Some((x.as_str(), c.checked_neg()?))),
        _ => None,
    };
    let mut relations = Vec::new();
    if let Some((y, c)) = shift {
        let moved = |d: bounds::Interval| {
            Some(bounds::Interval { lo: d.lo.checked_add(c)?, hi: d.hi.checked_add(c)? })
        };
        if y != x && !bounds.written_by_fn(y) {
            relations.push((y.to_string(), bounds::Interval { lo: c, hi: c }));
        }
        for (a, b, _) in bounds.relations() {
            let z = if a == y { b } else if b == y { a } else { continue };
            if z == *x && y != x {
                continue;
            }
            if let Some(d) = bounds.name_diff(y, &z).and_then(moved) {
                relations.push((z, d));
            }
        }
    }
    let now = match op {
        AssignOp::Set => bounds.expr(value),
        _ => None,
    };
    Some((x.clone(), relations, now))
}

fn ranges_of_stmt(
    base: u32,
    stmts: &[cpl_syntax::ast::Stmt],
    i: usize,
    top: &[&cpl_syntax::ast::Stmt],
    bounds: &mut bounds::Bounds,
    entry: Option<&bounds::Bounds>,
    proven: &mut Proven,
) {
    use cpl_syntax::ast::{Cond, ExprKind, FnBody, PatternKind, StmtKind};
    let stmt = &stmts[i];
    {
        match &stmt.kind {
            StmtKind::Let { init: Some(e), .. }
            | StmtKind::Return(Some(e))
            | StmtKind::Yield { value: e, .. }
            | StmtKind::Assert { cond: e, .. }
            | StmtKind::For { iter: e, .. } => arith(base, e, bounds, &mut proven.exprs),
            StmtKind::Assign { target, value, .. } => {
                arith(base, target, bounds, &mut proven.exprs);
                arith(base, value, bounds, &mut proven.exprs);
            }
            StmtKind::While { cond: Cond::Expr(e), .. } => arith(base, e, bounds, &mut proven.exprs),
            StmtKind::Expr(e) => match &e.kind {
                ExprKind::If { branches, .. } => {
                    for (cond, _) in branches {
                        if let Cond::Expr(c) = cond {
                            arith(base, c, bounds, &mut proven.exprs);
                        }
                    }
                }
                _ => arith(base, e, bounds, &mut proven.exprs),
            },
            _ => {}
        }
        let out = &mut proven.names;
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
                ..
            } => {
                let value = match (&pat.kind, ty) {
                    (PatternKind::Name(_), Some(ty)) => bounds.range(ty),
                    (PatternKind::Name(_), None) if !mutable => {
                        init.as_ref().and_then(|e| bounds.expr(e))
                    }
                    (PatternKind::Name(name), None) => {
                        let scope: Vec<&cpl_syntax::ast::Stmt> = if top.is_empty() {
                            stmts[i + 1..].iter().collect()
                        } else {
                            let at = top.iter().position(|s| std::ptr::eq(*s, stmt)).unwrap_or(0);
                            let earlier = top[..at].iter().filter(|s| {
                                matches!(s.kind, StmtKind::Fn(_) | StmtKind::Struct(_) | StmtKind::Impl { .. })
                            });
                            earlier.chain(&top[at + 1..]).copied().collect()
                        };
                        init.as_ref()
                            .and_then(|e| bounds.expr(e))
                            .and_then(|start| var_range(name, start, &scope, bounds))
                    }
                    _ => None,
                };
                if let (Some(v), None) = (value, ty) {
                    out.push((base + pat.span.start, v.lo, v.hi));
                }
                forget(pat, bounds);
                if let PatternKind::Name(name) = &pat.kind {
                    match (value, ty, init) {
                        (Some(v), Some(_), _) => bounds.input(name, v),
                        (_, None, Some(init)) if !mutable => {
                            bounds.bind(name, value);
                            bounds.define(name, init);
                            relate_midpoint(name, init, bounds);
                        }
                        _ => {
                            bounds.bind(name, value);
                            if let Some(v) = init.as_ref().filter(|_| *mutable).and_then(|e| bounds.expr(e)) {
                                bounds.set_now(name, v);
                            }
                        }
                    }
                }
            }
            StmtKind::Const { name, init, .. } => {
                let value = init.as_ref().and_then(|e| bounds.expr(e));
                bounds.bind(&name.text, value);
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                let mut inner = bounds.clone();
                let times = match &iter.kind {
                    ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    } => bounds.span(lo, hi, *inclusive).map(bounds::Interval::count),
                    ExprKind::Name(name) => bounds.domain_size(name),
                    _ => None,
                };
                let around = proven.around;
                let total = around.zip(times).and_then(|(a, t)| a.checked_mul(t));
                if let Some(times) = times {
                    proven.loops.push(crate::ProvenLoop {
                        head: (base + stmt.span.start, base + pat.span.start),
                        times,
                        total,
                    });
                }
                forget(pat, &mut inner);
                if let (
                    PatternKind::Name(name),
                    ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    },
                ) = (&pat.kind, &iter.kind)
                {
                    let value = inner.span(lo, hi, *inclusive).filter(|v| v.lo <= v.hi);
                    if let Some(v) = value {
                        out.push((base + pat.span.start, v.lo, v.hi));
                    }
                    inner.bind(name, value);
                }
                proven.around = total;
                ranges_in(base, &body.stmts, &[], &mut inner, proven);
                proven.around = around;
            }
            StmtKind::While {
                cond,
                body,
                else_body,
            } => {
                let invariant = match (cond, entry) {
                    (Cond::Expr(test), Some(entry)) => loop_invariant(base, test, &body.stmts, entry, bounds),
                    _ => Vec::new(),
                };
                let mut inner = bounds.clone();
                for (a, b, d) in &invariant {
                    inner.relate(a, b, *d);
                }
                if let Cond::Expr(test) = cond {
                    narrow(test, true, &|n| assigned_in(n, &body.stmts), &mut inner);
                }
                let around = proven.around.take();
                ranges_in(base, &body.stmts, &[], &mut inner, proven);
                proven.around = around;
                // A loop that only ends by its condition failing leaves it false.
                if let (Cond::Expr(test), None) = (cond, else_body) {
                    if !breaks(&body.stmts) {
                        for (a, b, d) in &invariant {
                            bounds.relate(a, b, *d);
                        }
                        let rest: Vec<&cpl_syntax::ast::Stmt> = if top.is_empty() {
                            stmts[i + 1..].iter().collect()
                        } else {
                            let at = top.iter().position(|s| std::ptr::eq(*s, stmt)).unwrap_or(0);
                            top[at + 1..].to_vec()
                        };
                        let later = |n: &str| rest.iter().any(|s| assigned_in(n, std::slice::from_ref(*s)));
                        narrow(test, false, &later, bounds);
                    }
                }
            }
            StmtKind::Unchecked(body) => {
                ranges_in(base, &body.stmts, &[], &mut bounds.clone(), proven);
            }
            StmtKind::Expr(expr) => {
                if let ExprKind::If {
                    branches,
                    otherwise,
                } = &expr.kind
                {
                    // Each branch starts from what held before the `if`; where
                    // they meet, what every way through it proves holds.
                    let start = entry.unwrap_or(bounds);
                    let mut exits = Vec::new();
                    let mut declared = std::collections::HashSet::new();
                    let mut joins = true;
                    // An `if let` binds names the branch would see through.
                    let mut failed = Vec::new();
                    for (cond, block) in branches {
                        let Cond::Expr(test) = cond else {
                            joins = false;
                            continue;
                        };
                        let mut inner = start.clone();
                        for &earlier in &failed {
                            narrow(earlier, false, &|n| assigned_in(n, &block.stmts), &mut inner);
                        }
                        narrow(test, true, &|n| assigned_in(n, &block.stmts), &mut inner);
                        ranges_in(base, &block.stmts, &[], &mut inner, proven);
                        declared.extend(declared_in(&block.stmts));
                        exits.push(inner);
                        failed.push(test);
                    }
                    let mut inner = start.clone();
                    let none: &[cpl_syntax::ast::Stmt] = &[];
                    let rest = otherwise.as_ref().map_or(none, |b| &b.stmts[..]);
                    for &earlier in &failed {
                        narrow(earlier, false, &|n| assigned_in(n, rest), &mut inner);
                    }
                    if let Some(block) = otherwise {
                        ranges_in(base, &block.stmts, &[], &mut inner, proven);
                        declared.extend(declared_in(&block.stmts));
                    }
                    exits.push(inner);
                    if joins {
                        join(&exits, &declared, bounds);
                    }
                }
            }
            StmtKind::Fn(decl) => {
                bounds.declare_fn(&decl.name.text);
                let mut inner = bounds.clone();
                for param in &decl.params {
                    match param.ty.as_ref().and_then(|t| inner.range(t)) {
                        Some(range) => inner.input(&param.name.text, range),
                        None => inner.bind(&param.name.text, None),
                    }
                }
                if let FnBody::Block(block) = &decl.body {
                    inner.declare_fn(&decl.name.text);
                    let around = proven.around.replace(1);
                    ranges_in(base, &block.stmts, &[], &mut inner, proven);
                    proven.around = around;
                }
            }
            _ => {}
        }
    }
}

/// The hull of every value the `var` `name` takes in `scope`, which starts
/// at `start`: the least range every assignment in `scope` keeps it in.
/// Other `var`s and immutable `let`s in `scope` bound nowhere else in the
/// file are solved with it, so a binary search's `lo`, `hi` and `mid` get
/// ranges from each other.
fn var_range(
    name: &str,
    start: bounds::Interval,
    scope: &[&cpl_syntax::ast::Stmt],
    bounds: &bounds::Bounds,
) -> Option<bounds::Interval> {
    use cpl_syntax::ast::{AssignOp, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    struct Scan {
        writes: HashMap<String, Vec<(AssignOp, Expr)>>,
        opaque: HashSet<String>,
        bound: Vec<String>,
        /// Each `let` or `var` with a plain name and an initializer, in
        /// source order: (name, init, is a var).
        defs: Vec<(String, Expr, bool)>,
    }
    fn names(pat: &Pattern, out: &mut Vec<String>) {
        match &pat.kind {
            PatternKind::Name(n) => out.push(n.clone()),
            PatternKind::Domain(n) => out.push(n.text.clone()),
            PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
                parts.iter().for_each(|p| names(p, out))
            }
            PatternKind::Variant { args, .. } => args.iter().for_each(|p| names(p, out)),
            PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
        }
    }
    fn mentioned(e: &Expr, out: &mut std::collections::HashSet<String>) {
        match &e.kind {
            ExprKind::Name(n) => {
                out.insert(n.clone());
            }
            ExprKind::Paren(inner) => mentioned(inner, out),
            ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().for_each(|i| mentioned(i, out)),
            _ => {}
        }
    }
    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign { target, op, value } => {
                    if let ExprKind::Name(n) = &target.kind {
                        self.writes.entry(n.clone()).or_default().push((*op, value.clone()));
                    } else {
                        mentioned(target, &mut self.opaque);
                    }
                }
                StmtKind::Lend { place, .. } => mentioned(place, &mut self.opaque),
                StmtKind::Fn(decl) => {
                    self.bound.extend(decl.params.iter().map(|p| p.name.text.clone()))
                }
                StmtKind::Let {
                    mutable,
                    pat,
                    ty: None,
                    init: Some(init),
                    ..
                } => {
                    if let PatternKind::Name(n) = &pat.kind {
                        self.defs.push((n.clone(), init.clone(), *mutable));
                    }
                }
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { args, .. } = &expr.kind {
                for a in args.iter().filter(|a| a.mutable) {
                    mentioned(&a.value, &mut self.opaque);
                }
            }
        }
        fn pattern(&mut self, pat: &Pattern) {
            names(pat, &mut self.bound);
        }
    }

    let mut scan = Scan::default();
    scope.iter().for_each(|s| walk_stmt(s, &mut scan));
    if scan.opaque.contains(name) || scan.bound.iter().any(|b| b == name) {
        return None;
    }
    let solvable = |n: &str| bounds.bound_once(n) && !scan.opaque.contains(n);
    let mut vars: HashSet<&str> = scan
        .defs
        .iter()
        .filter(|(n, _, var)| *var && solvable(n))
        .map(|(n, ..)| n.as_str())
        .collect();
    let mut guards = HashMap::new();
    for s in scope {
        guarded(s, &mut guards);
    }
    // One round maps hulls to the values they can produce: each var's
    // initializer and every assignment, read with the vars in `hulls`. `Err`
    // names a var that can't be bounded.
    let round = |hulls: &HashMap<&str, bounds::Interval>,
                 vars: &HashSet<&str>|
     -> Result<HashMap<&str, bounds::Interval>, String> {
        let mut known = bounds.clone();
        for b in &scan.bound {
            known.bind(b, None);
        }
        for (v, h) in hulls {
            known.bind(v, Some(*h));
        }
        let mut next = HashMap::from([(name, start)]);
        for (n, init, var) in &scan.defs {
            let value = known.expr(init);
            if !var {
                known.bind(n, value.filter(|_| solvable(n)));
            } else if vars.contains(n.as_str()) {
                let v = value.ok_or_else(|| n.clone())?;
                next.insert(n.as_str(), v);
                known.bind(n, Some(hulls.get(n.as_str()).map_or(v, |h| h.hull(v))));
            }
        }
        for (&v, out) in next.iter_mut() {
            let old = hulls.get(v).copied().unwrap_or(*out);
            for (op, value) in scan.writes.get(v).into_iter().flatten() {
                let (w, old) = match guards.get(&(value.span.start, value.span.end)) {
                    Some((test, before)) => {
                        let mut inner = known.clone();
                        narrow(test, true, &|n| assigned_in(n, before), &mut inner);
                        // The guard can't hold, so this never runs.
                        let Some(old) = inner.expr_name(v) else { continue };
                        (inner.expr(value), old)
                    }
                    None => (known.expr(value), old),
                };
                let w = w.ok_or_else(|| v.to_string())?;
                let after = match op {
                    AssignOp::Set => Some(w),
                    AssignOp::Binary(op) => bounds::Bounds::combine(*op, old, w),
                    AssignOp::Max => Some(bounds::Interval {
                        lo: old.lo.max(w.lo),
                        hi: old.hi.max(w.hi),
                    }),
                    AssignOp::Min => Some(bounds::Interval {
                        lo: old.lo.min(w.lo),
                        hi: old.hi.min(w.hi),
                    }),
                };
                *out = out.hull(after.ok_or_else(|| v.to_string())?);
            }
        }
        Ok(next)
    };
    let within = |inner: &HashMap<&str, bounds::Interval>, outer: &HashMap<&str, bounds::Interval>| {
        inner.iter().all(|(v, i)| outer.get(v).is_some_and(|o| o.lo <= i.lo && i.hi <= o.hi))
    };
    // A var that can't be bounded is dropped and the rest solved again.
    'solve: loop {
        let mut hulls: HashMap<&str, bounds::Interval> = HashMap::from([(name, start)]);
        let mut settled = false;
        // Grow the hulls until a round stays inside them; a bound still
        // growing after a few rounds jumps to the edge of `Int`.
        for step in 0..80 {
            let next = match round(&hulls, &vars) {
                Ok(next) => next,
                Err(v) if v != name => {
                    vars.remove(v.as_str());
                    continue 'solve;
                }
                Err(_) => return None,
            };
            if within(&next, &hulls) {
                settled = true;
                break;
            }
            for (v, n) in next {
                let grown = match hulls.get(v) {
                    Some(h) => {
                        let mut g = h.hull(n);
                        if step >= 64 {
                            let int = i64::MAX as i128;
                            if g.lo < h.lo && g.lo >= -int - 1 {
                                g.lo = -int - 1;
                            }
                            if g.hi > h.hi && g.hi <= int {
                                g.hi = int;
                            }
                        }
                        g
                    }
                    None => n,
                };
                hulls.insert(v, grown);
            }
        }
        if !settled {
            if vars.is_empty() {
                return None;
            }
            // Some hull never settled: solve `name` alone.
            vars.clear();
            continue;
        }
        // Each round of hulls a round stays inside also stays inside the
        // hulls, so it tightens them and keeps every value they held.
        for _ in 0..72 {
            let Ok(next) = round(&hulls, &vars) else { break };
            if next == hulls || !within(&next, &hulls) {
                break;
            }
            hulls = next;
        }
        return hulls.get(name).copied();
    }
}

/// Each assignment directly in a `while` body, keyed by the span of the
/// value it assigns, with the loop's condition and the statements before it
/// in that body: those that run between the condition holding and it.
fn guarded<'a>(
    stmt: &'a cpl_syntax::ast::Stmt,
    out: &mut std::collections::HashMap<(u32, u32), (&'a cpl_syntax::ast::Expr, &'a [cpl_syntax::ast::Stmt])>,
) {
    use cpl_syntax::ast::{Cond, ExprKind, FnBody, StmtKind};
    let blocks: Vec<&[cpl_syntax::ast::Stmt]> = match &stmt.kind {
        StmtKind::While { cond, body, else_body } => {
            if let Cond::Expr(test) = cond {
                for (k, s) in body.stmts.iter().enumerate() {
                    if let StmtKind::Assign { value, .. } = &s.kind {
                        out.insert((value.span.start, value.span.end), (test, &body.stmts[..k]));
                    }
                }
            }
            std::iter::once(&body.stmts[..]).chain(else_body.iter().map(|b| &b.stmts[..])).collect()
        }
        StmtKind::For { body, .. } | StmtKind::Unchecked(body) => vec![&body.stmts[..]],
        StmtKind::Fn(decl) => match &decl.body {
            FnBody::Block(b) => vec![&b.stmts[..]],
            _ => Vec::new(),
        },
        StmtKind::Expr(e) => match &e.kind {
            ExprKind::If { branches, otherwise } => branches
                .iter()
                .map(|(_, b)| &b.stmts[..])
                .chain(otherwise.iter().map(|b| &b.stmts[..]))
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    for block in blocks {
        for s in block {
            guarded(s, out);
        }
    }
}

/// Inside a block that runs only when `test` is `holds`, each bounded name
/// a comparison in `test` limits is in the tighter range, unless the block
/// assigns it.
/// Gives `bounds` what every one of `exits` proves, about names declared
/// outside the blocks they come from.
fn join(exits: &[bounds::Bounds], declared: &std::collections::HashSet<String>, bounds: &mut bounds::Bounds) {
    let Some((first, others)) = exits.split_first() else {
        return;
    };
    let outer = |n: &str| !declared.contains(n);
    for (a, b, d) in first.relations() {
        if !outer(&a) || !outer(&b) {
            continue;
        }
        let all = others.iter().try_fold(d, |d, e| e.name_diff(&a, &b).map(|o| d.hull(o)));
        if let Some(d) = all {
            bounds.relate(&a, &b, d);
        }
    }
    for (n, v) in first.nows() {
        if !outer(&n) || bounds.now_of(&n).is_some() {
            continue;
        }
        if let Some(v) = others.iter().try_fold(v, |v, e| e.now_of(&n).map(|o| v.hull(o))) {
            bounds.set_now(&n, v);
        }
    }
}

/// The names `stmts` binds at their own level.
fn declared_in(stmts: &[cpl_syntax::ast::Stmt]) -> std::collections::HashSet<String> {
    use cpl_syntax::ast::{Pattern, PatternKind, StmtKind};
    fn names(pat: &Pattern, out: &mut std::collections::HashSet<String>) {
        match &pat.kind {
            PatternKind::Name(n) => {
                out.insert(n.clone());
            }
            PatternKind::Tuple(parts) | PatternKind::Or(parts) => parts.iter().for_each(|p| names(p, out)),
            PatternKind::Variant { args, .. } => args.iter().for_each(|p| names(p, out)),
            _ => {}
        }
    }
    let mut out = std::collections::HashSet::new();
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Let { pat, .. } => names(pat, &mut out),
            StmtKind::Const { name, .. } => {
                out.insert(name.text.clone());
            }
            _ => {}
        }
    }
    out
}

/// What a `while` keeps true about two names its condition compares: start
/// from their difference on entry, run the body until that stops widening
/// (at most three times), and keep each pair that settles. The body must not
/// `continue`, which would come back to the condition from elsewhere.
fn loop_invariant(
    base: u32,
    test: &cpl_syntax::ast::Expr,
    body: &[cpl_syntax::ast::Stmt],
    entry: &bounds::Bounds,
    bounds: &bounds::Bounds,
) -> Vec<(String, String, bounds::Interval)> {
    let declared = declared_in(body);
    let mut pairs = Vec::new();
    compared_names(test, &mut pairs);
    let mut cands: Vec<(String, String, bounds::Interval)> = pairs
        .into_iter()
        .filter(|(a, b)| a != b && !declared.contains(a) && !declared.contains(b))
        .filter(|(a, b)| assigned_in(a, body) || assigned_in(b, body))
        .filter_map(|(a, b)| entry.current_diff(&a, &b).map(|d| (a, b, d)))
        .collect();
    if cands.is_empty() || continues(body) {
        return Vec::new();
    }
    for _ in 0..3 {
        let mut inner = bounds.clone();
        for (a, b, d) in &cands {
            inner.relate(a, b, *d);
        }
        narrow(test, true, &|n| assigned_in(n, body), &mut inner);
        ranges_in(base, body, &[], &mut inner, &mut Proven::default());
        let mut settled = true;
        cands.retain_mut(|(a, b, d)| match inner.name_diff(a, b) {
            Some(out) => {
                let wider = d.hull(out);
                settled &= wider == *d;
                *d = wider;
                true
            }
            None => false,
        });
        if settled {
            return cands;
        }
    }
    Vec::new()
}

/// The pairs of plain names `test` compares where it holds.
fn compared_names(test: &cpl_syntax::ast::Expr, out: &mut Vec<(String, String)>) {
    use cpl_syntax::ast::{BinOp, ExprKind};
    match &test.kind {
        ExprKind::Paren(inner) => compared_names(inner, out),
        ExprKind::Binary { op: BinOp::And, lhs, rhs, .. } => {
            compared_names(lhs, out);
            compared_names(rhs, out);
        }
        ExprKind::Compare { first, rest } => {
            let mut left = first.as_ref();
            for (_, right) in rest {
                if let (ExprKind::Name(a), ExprKind::Name(b)) = (&left.kind, &right.kind) {
                    out.push((a.clone(), b.clone()));
                }
                left = right;
            }
        }
        _ => {}
    }
}

fn narrow(
    test: &cpl_syntax::ast::Expr,
    holds: bool,
    block: &dyn Fn(&str) -> bool,
    bounds: &mut bounds::Bounds,
) {
    use cpl_syntax::ast::{BinOp, CmpOp, ExprKind, UnOp};
    match &test.kind {
        ExprKind::Paren(inner) => narrow(inner, holds, block, bounds),
        ExprKind::Unary { op: UnOp::Not, expr } => narrow(expr, !holds, block, bounds),
        ExprKind::Binary { op, lhs, rhs, .. }
            if (*op == BinOp::And && holds) || (*op == BinOp::Or && !holds) =>
        {
            narrow(lhs, holds, block, bounds);
            narrow(rhs, holds, block, bounds);
        }
        ExprKind::Compare { first, rest } if holds || rest.len() == 1 => {
            let mut left = first.as_ref();
            for (op, right) in rest {
                let op = &if holds {
                    *op
                } else {
                    match op {
                        CmpOp::Lt => CmpOp::Ge,
                        CmpOp::Le => CmpOp::Gt,
                        CmpOp::Gt => CmpOp::Le,
                        CmpOp::Ge => CmpOp::Lt,
                        CmpOp::Ne => CmpOp::Eq,
                        _ => CmpOp::Ne,
                    }
                };
                let flipped = match op {
                    CmpOp::Lt => Some(CmpOp::Gt),
                    CmpOp::Le => Some(CmpOp::Ge),
                    CmpOp::Gt => Some(CmpOp::Lt),
                    CmpOp::Ge => Some(CmpOp::Le),
                    CmpOp::Eq => Some(CmpOp::Eq),
                    _ => None,
                };
                if let Some(flipped) = flipped {
                    for (name, op, other) in [(left, *op, right), (right, flipped, left)] {
                        let ExprKind::Name(name) = &name.kind else {
                            continue;
                        };
                        let (Some(have), Some(other)) = (bounds.expr_name(name), bounds.expr(other)) else {
                            continue;
                        };
                        if block(name) || bounds.written_by_fn(name) {
                            continue;
                        }
                        let (lo, hi) = match op {
                            CmpOp::Lt => (have.lo, have.hi.min(other.hi - 1)),
                            CmpOp::Le => (have.lo, have.hi.min(other.hi)),
                            CmpOp::Gt => (have.lo.max(other.lo + 1), have.hi),
                            CmpOp::Ge => (have.lo.max(other.lo), have.hi),
                            _ => (have.lo.max(other.lo), have.hi.min(other.hi)),
                        };
                        // A branch the bounds prove dead claims nothing.
                        match (lo <= hi).then_some(bounds::Interval { lo, hi }) {
                            Some(v) => bounds.restrict(name, v),
                            None => bounds.bind(name, None),
                        }
                    }
                    relate(left, *op, right, bounds);
                }
                left = right;
            }
        }
        _ => {}
    }
}

/// `let m = (x + y) / 2` is `x + (y - x) / 2` rounded down, so a relation
/// between `x` and `y` bounds `m - x` and `y - m`.
fn relate_midpoint(m: &str, init: &cpl_syntax::ast::Expr, bounds: &mut bounds::Bounds) {
    use cpl_syntax::ast::{BinOp, ExprKind};
    fn strip(e: &cpl_syntax::ast::Expr) -> &cpl_syntax::ast::Expr {
        match &e.kind {
            ExprKind::Paren(inner) => strip(inner),
            _ => e,
        }
    }
    let ExprKind::Binary { op: BinOp::Div, lhs, rhs, .. } = &strip(init).kind else {
        return;
    };
    if !matches!(strip(rhs).kind, ExprKind::Int(2)) {
        return;
    }
    let ExprKind::Binary { op: BinOp::Add, lhs: x, rhs: y, .. } = &strip(lhs).kind else {
        return;
    };
    let (ExprKind::Name(xn), ExprKind::Name(yn)) = (&strip(x).kind, &strip(y).kind) else {
        return;
    };
    if xn == m || yn == m || bounds.written_by_fn(m) {
        return;
    }
    let Some(d) = bounds.diff(y, x) else {
        return;
    };
    // `y - x` is in `d`; `m - x` is its half rounded down, `y - m` rounded up.
    let (down, up) = (|v: i128| v.div_euclid(2), |v: i128| -(-v).div_euclid(2));
    bounds.relate(m, xn, bounds::Interval { lo: down(d.lo), hi: down(d.hi) });
    bounds.relate(yn, m, bounds::Interval { lo: up(d.lo), hi: up(d.hi) });
}

/// Records what `a op b` proves about `a - b` when both are bounded names
/// no fn assigns. It holds where the condition was tested; `ranges_in`
/// drops it before the first statement that assigns either name.
fn relate(
    a: &cpl_syntax::ast::Expr,
    op: cpl_syntax::ast::CmpOp,
    b: &cpl_syntax::ast::Expr,
    bounds: &mut bounds::Bounds,
) {
    use cpl_syntax::ast::{BinOp, CmpOp, ExprKind};
    let (ExprKind::Name(x), ExprKind::Name(y)) = (&a.kind, &b.kind) else {
        return;
    };
    if x == y || [x, y].iter().any(|n| bounds.written_by_fn(n)) {
        return;
    }
    let (Some(vx), Some(vy)) = (bounds.expr_name(x), bounds.expr_name(y)) else {
        return;
    };
    let Some(d) = bounds::Bounds::combine(BinOp::Sub, vx, vy) else {
        return;
    };
    let (lo, hi) = match op {
        CmpOp::Lt => (d.lo, d.hi.min(-1)),
        CmpOp::Le => (d.lo, d.hi.min(0)),
        CmpOp::Gt => (d.lo.max(1), d.hi),
        CmpOp::Ge => (d.lo.max(0), d.hi),
        CmpOp::Eq => (d.lo.max(0), d.hi.min(0)),
        _ => return,
    };
    if lo <= hi {
        bounds.relate(x, y, bounds::Interval { lo, hi });
    }
}

/// Whether a `continue` in `stmts` can go back to a loop's condition.
fn continues(stmts: &[cpl_syntax::ast::Stmt]) -> bool {
    use cpl_syntax::ast::{Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    struct Continues(bool);
    impl Visitor for Continues {
        fn stmt(&mut self, stmt: &Stmt) {
            self.0 |= matches!(stmt.kind, StmtKind::Continue(_));
        }
    }
    let mut v = Continues(false);
    stmts.iter().for_each(|s| walk_stmt(s, &mut v));
    v.0
}

/// Whether a `break` in `stmts` can leave a loop early.
fn breaks(stmts: &[cpl_syntax::ast::Stmt]) -> bool {
    use cpl_syntax::ast::{Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    struct Breaks(bool);
    impl Visitor for Breaks {
        fn stmt(&mut self, stmt: &Stmt) {
            self.0 |= matches!(stmt.kind, StmtKind::Break(_));
        }
    }
    let mut v = Breaks(false);
    stmts.iter().for_each(|s| walk_stmt(s, &mut v));
    v.0
}

fn assigned_in(name: &str, stmts: &[cpl_syntax::ast::Stmt]) -> bool {
    use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
    use cpl_syntax::visit::{walk_stmt, Visitor};
    struct Writes<'a>(&'a str, bool);
    impl Writes<'_> {
        fn mentions(&self, e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Name(n) => n == self.0,
                ExprKind::Paren(inner) => self.mentions(inner),
                ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().any(|i| self.mentions(i)),
                _ => false,
            }
        }
    }
    impl Visitor for Writes<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign { target, .. } => self.1 |= self.mentions(target),
                StmtKind::Lend { place, .. } => self.1 |= self.mentions(place),
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { args, .. } = &expr.kind {
                self.1 |= args.iter().any(|a| a.mutable && self.mentions(&a.value));
            }
        }
    }
    let mut writes = Writes(name, false);
    stmts.iter().for_each(|s| walk_stmt(s, &mut writes));
    writes.1
}

/// Every name `pat` binds stops being bounded.
fn forget(pat: &cpl_syntax::ast::Pattern, bounds: &mut bounds::Bounds) {
    use cpl_syntax::ast::PatternKind;
    match &pat.kind {
        PatternKind::Name(name) => bounds.bind(name, None),
        PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
            parts.iter().for_each(|p| forget(p, bounds))
        }
        PatternKind::Variant { args, .. } => args.iter().for_each(|p| forget(p, bounds)),
        _ => {}
    }
}
