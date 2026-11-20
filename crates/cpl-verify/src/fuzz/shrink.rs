//! Cuts a failing program down while it keeps failing.
//!
//! Greedy: try every one-step simplification, keep the first that is
//! still a valid, defined program and still fails, and start again.

use super::ast::*;
use super::check::check;

/// Returns the smallest program found for which `still_fails` holds.
/// `still_fails` is only called on programs that pass [`check`], and at
/// most `budget` times, since each call usually costs a C++ compile.
pub fn shrink(
    start: &Program,
    budget: usize,
    mut still_fails: impl FnMut(&Program) -> bool,
) -> Program {
    let mut best = start.clone();
    let mut calls = 0;
    'outer: loop {
        for cand in candidates(&best) {
            if check(&cand).is_err() {
                continue;
            }
            if calls == budget {
                break 'outer;
            }
            calls += 1;
            if still_fails(&cand) {
                best = cand;
                continue 'outer;
            }
        }
        break;
    }
    best
}

/// Number of statements and expression nodes; what shrinking reduces.
pub fn size(p: &Program) -> usize {
    let n = std::cell::Cell::new(p.funcs.len());
    let mut p = p.clone();
    let bump = || n.set(n.get() + 1);
    for f in &mut p.funcs {
        walk_block(&mut f.body, &mut |_| bump(), &mut |_| bump());
    }
    walk_block(&mut p.body, &mut |_| bump(), &mut |_| bump());
    n.get()
}

/// Every program one simplification away from `p`, biggest cuts first.
fn candidates(p: &Program) -> Vec<Program> {
    let mut out = vec![];
    for i in 0..p.funcs.len() {
        let mut c = p.clone();
        c.funcs.remove(i);
        out.push(c);
    }
    // Statement edits: block `b` (in a fixed walk order), statement `i`.
    let blocks = count_blocks(p);
    for b in 0..blocks {
        let len = with_block(&mut p.clone(), b, |stmts| stmts.len());
        for i in 0..len {
            let mut c = p.clone();
            with_block(&mut c, b, |stmts| {
                stmts.remove(i);
            });
            out.push(c);
        }
        for i in 0..len {
            let fewer = with_block(&mut p.clone(), b, |stmts| fewer_print_args(&stmts[i]));
            for stmt in fewer {
                let mut c = p.clone();
                with_block(&mut c, b, |stmts| stmts[i] = stmt.clone());
                out.push(c);
            }
        }
        for i in 0..len {
            // Replace a compound statement with one of its bodies.
            let bodies = with_block(&mut p.clone(), b, |stmts| inner_bodies(&stmts[i]));
            for body in bodies {
                let mut c = p.clone();
                with_block(&mut c, b, |stmts| {
                    stmts.splice(i..=i, body.clone());
                });
                out.push(c);
            }
        }
    }
    // Expression edits: expression `e` (in a fixed walk order).
    let exprs = count_exprs(p);
    for e in 0..exprs {
        let simpler = with_expr(&mut p.clone(), e, |x| simpler_exprs(x));
        for s in simpler {
            let mut c = p.clone();
            with_expr(&mut c, e, |x| *x = s.clone());
            out.push(c);
        }
    }
    out
}

fn fewer_print_args(s: &Stmt) -> Vec<Stmt> {
    match s {
        Stmt::Print(args) if args.len() > 1 => (0..args.len())
            .map(|i| {
                let mut fewer = args.clone();
                fewer.remove(i);
                Stmt::Print(fewer)
            })
            .collect(),
        _ => vec![],
    }
}

fn inner_bodies(s: &Stmt) -> Vec<Vec<Stmt>> {
    match s {
        Stmt::If { arms, els } => arms
            .iter()
            .map(|(_, b)| b.clone())
            .chain(els.iter().cloned())
            .collect(),
        Stmt::ForRange { body, .. } | Stmt::ForIn { body, .. } | Stmt::While { body, .. } => {
            vec![body.clone()]
        }
        _ => vec![],
    }
}

/// Smaller expressions that may have the same type; `check` decides.
fn simpler_exprs(e: &Expr) -> Vec<Expr> {
    let mut out = match e {
        Expr::Int(0) | Expr::Bool(false) => vec![],
        Expr::Int(1) => vec![Expr::Int(0)],
        Expr::Bool(true) => vec![Expr::Bool(false)],
        Expr::ListLit(items) if items.len() > 1 => (0..items.len())
            .map(|i| {
                let mut fewer = items.clone();
                fewer.remove(i);
                Expr::ListLit(fewer)
            })
            .collect(),
        Expr::ListLit(_) => vec![],
        _ => vec![
            Expr::Int(0),
            Expr::Int(1),
            Expr::Bool(false),
            Expr::Bool(true),
        ],
    };
    match e {
        Expr::Neg(a) | Expr::Not(a) => out.push((**a).clone()),
        Expr::Bin(_, a, b) => out.extend([(**a).clone(), (**b).clone()]),
        Expr::If(_, a, b) => out.extend([(**a).clone(), (**b).clone()]),
        Expr::Call(_, args) | Expr::Builtin(_, args) => out.extend(args.iter().cloned()),
        _ => {}
    }
    out
}

fn count_blocks(p: &Program) -> usize {
    let mut n = 0;
    each_block(&mut p.clone(), &mut |_| n += 1);
    n
}

fn with_block<R>(p: &mut Program, index: usize, f: impl FnOnce(&mut Vec<Stmt>) -> R) -> R {
    let mut f = Some(f);
    let mut result = None;
    let mut n = 0;
    each_block(p, &mut |stmts| {
        if n == index {
            result = f.take().map(|f| f(stmts));
        }
        n += 1;
    });
    result.expect("block index in range")
}

/// Calls `f` on every statement list, outermost first.
fn each_block(p: &mut Program, f: &mut dyn FnMut(&mut Vec<Stmt>)) {
    fn go(stmts: &mut Vec<Stmt>, f: &mut dyn FnMut(&mut Vec<Stmt>)) {
        f(stmts);
        for s in stmts {
            match s {
                Stmt::If { arms, els } => {
                    for (_, b) in arms {
                        go(b, f);
                    }
                    if let Some(b) = els {
                        go(b, f);
                    }
                }
                Stmt::ForRange { body, .. }
                | Stmt::ForIn { body, .. }
                | Stmt::While { body, .. } => go(body, f),
                _ => {}
            }
        }
    }
    go(&mut p.body, f);
    for func in &mut p.funcs {
        go(&mut func.body, f);
    }
}

fn count_exprs(p: &Program) -> usize {
    let mut n = 0;
    each_expr(&mut p.clone(), &mut |_| n += 1);
    n
}

fn with_expr<R>(p: &mut Program, index: usize, f: impl FnOnce(&mut Expr) -> R) -> R {
    let mut f = Some(f);
    let mut result = None;
    let mut n = 0;
    each_expr(p, &mut |e| {
        if n == index {
            result = f.take().map(|f| f(e));
        }
        n += 1;
    });
    result.expect("expression index in range")
}

/// Calls `f` on every expression, parents before children.
fn each_expr(p: &mut Program, f: &mut dyn FnMut(&mut Expr)) {
    walk_block(&mut p.body, &mut |_| {}, f);
    for func in &mut p.funcs {
        walk_block(&mut func.body, &mut |_| {}, f);
    }
}

fn walk_block(
    stmts: &mut [Stmt],
    on_stmt: &mut dyn FnMut(&mut Stmt),
    on_expr: &mut dyn FnMut(&mut Expr),
) {
    for s in stmts {
        on_stmt(s);
        match s {
            Stmt::Let { init: e, .. }
            | Stmt::Assign { value: e, .. }
            | Stmt::Push { value: e, .. }
            | Stmt::Return(e)
            | Stmt::Tail(e) => walk_expr(e, on_expr),
            Stmt::IndexAssign { index, value, .. } => {
                walk_expr(index, on_expr);
                walk_expr(value, on_expr);
            }
            Stmt::If { arms, els } => {
                for (cond, body) in arms {
                    walk_expr(cond, on_expr);
                    walk_block(body, on_stmt, on_expr);
                }
                if let Some(body) = els {
                    walk_block(body, on_stmt, on_expr);
                }
            }
            Stmt::ForRange { lo, hi, body, .. } => {
                walk_expr(lo, on_expr);
                walk_expr(hi, on_expr);
                walk_block(body, on_stmt, on_expr);
            }
            Stmt::ForIn { body, .. } => walk_block(body, on_stmt, on_expr),
            Stmt::While { cond, body } => {
                walk_expr(cond, on_expr);
                walk_block(body, on_stmt, on_expr);
            }
            Stmt::Print(args) => {
                for a in args {
                    walk_expr(a, on_expr);
                }
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
}

fn walk_expr(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    f(e);
    match e {
        Expr::Neg(a) | Expr::Not(a) | Expr::Index(_, a) => walk_expr(a, f),
        Expr::Bin(_, a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expr::If(c, a, b) => {
            walk_expr(c, f);
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expr::Call(_, args) | Expr::Builtin(_, args) | Expr::ListLit(args) => {
            for a in args {
                walk_expr(a, f);
            }
        }
        Expr::Int(_) | Expr::Bool(_) | Expr::Var(_) | Expr::Len(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzz::{eval, gen, print};

    /// A stand-in for a backend bug: any program that prints a line with
    /// a multiple of 7 above 20 "fails".
    fn prints_bad_number(p: &Program) -> bool {
        eval::run(p).is_ok_and(|out| {
            out.split_whitespace()
                .filter_map(|w| w.parse::<i64>().ok())
                .any(|v| v > 20 && v % 7 == 0)
        })
    }

    #[test]
    fn shrinks_a_failing_program_to_a_few_lines() {
        let cfg = gen::Config::default();
        let case = (0..)
            .map(|seed| gen::generate(seed, &cfg))
            .find(|c| prints_bad_number(&c.program) && size(&c.program) > 60)
            .unwrap();
        let small = shrink(&case.program, 100_000, prints_bad_number);
        assert!(prints_bad_number(&small));
        check(&small).unwrap();
        let source = print::to_source(&small);
        assert!(
            size(&small) * 4 < size(&case.program),
            "barely shrunk:\n{source}"
        );
        assert!(source.lines().count() <= 8, "not small:\n{source}");
    }

    #[test]
    fn stops_at_the_budget() {
        let case = gen::generate(3, &gen::Config::default());
        let mut calls = 0;
        shrink(&case.program, 5, |_| {
            calls += 1;
            false
        });
        assert_eq!(calls, 5);
    }
}
