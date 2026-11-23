//! `let (a, b) = (x, y)` binds each part straight from its own expression:
//! the tuple the pattern takes apart is never built, so a part that is a
//! closure keeps its own type instead of a `std::function` slot of a
//! `std::tuple`. It majorizes: the same expressions run in the same order.

use crate::walk;
use cpl_ir::{Block, Expr, ExprKind, Local, Program, Stmt, Ty};

pub(crate) fn unpack(program: &mut Program) {
    let Program {
        setup,
        main,
        main_locals,
        fns,
        closures,
        ..
    } = program;
    let captured: Vec<(bool, usize)> = closures
        .iter()
        .flat_map(|c| c.captures.iter().map(|c| (c.is_global, c.outer)))
        .collect();
    let top_captured = |slot| captured.contains(&(false, slot));
    for block in [setup, main] {
        home(block, main_locals, &top_captured);
    }
    for f in fns.iter_mut() {
        home(&mut f.body, &f.locals, &top_captured);
    }
    for c in closures.iter_mut() {
        home(&mut c.body, &c.locals, &top_captured);
    }
}

fn home(body: &mut Block, locals: &[Local], captured: &dyn Fn(usize) -> bool) {
    let mut reads = vec![0usize; locals.len()];
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            if let Some(n) = reads.get_mut(s) {
                *n += 1;
            }
        }
    });
    walk::blocks_mut(body, &mut |block| {
        let mut k = 0;
        while k < block.stmts.len() {
            if let Some(n) = parts(&block.stmts, k, locals, &reads, captured) {
                let Stmt::Let(
                    _,
                    Some(Expr {
                        kind: ExprKind::Tuple(items),
                        ..
                    }),
                ) = block.stmts.remove(k)
                else {
                    unreachable!("parts checked the shape");
                };
                for (i, item) in items.into_iter().enumerate() {
                    if let Stmt::Let(_, init) = &mut block.stmts[k + i] {
                        *init = Some(item);
                    }
                }
                k += n;
            } else {
                k += 1;
            }
        }
    });
}

/// How many parts `stmts[k]`, a `let` of a tuple temporary, has when the
/// statements right after it bind each part in order and nothing else
/// reads the temporary.
fn parts(
    stmts: &[Stmt],
    k: usize,
    locals: &[Local],
    reads: &[usize],
    captured: &dyn Fn(usize) -> bool,
) -> Option<usize> {
    let Stmt::Let(
        t,
        Some(Expr {
            kind: ExprKind::Tuple(items),
            ..
        }),
    ) = &stmts[k]
    else {
        return None;
    };
    // Only a tuple with a closure in it is taken apart. A part read from a
    // variable stays in the tuple: bound alone it would be a copy the C++
    // spells out, where the tuple's is the same copy.
    if !items.iter().any(|item| matches!(item.ty, Ty::Fn(..)))
        || items
            .iter()
            .any(|item| !item.ty.is_scalar() && crate::stmt::place_root(item).is_some())
    {
        return None;
    }
    let n = items.len();
    let binds = |(i, stmt): (usize, &Stmt)| {
        matches!(stmt, Stmt::Let(_, Some(Expr { kind: ExprKind::Field(base, j), .. }))
            if *j == i && matches!(base.kind, ExprKind::Local(s) if s == *t))
    };
    let named = !locals[*t].name.is_empty();
    let rest = stmts.get(k + 1..k + 1 + n)?;
    (n > 0 && !named && !captured(*t) && reads[*t] == n && rest.iter().enumerate().all(binds))
        .then_some(n)
}
