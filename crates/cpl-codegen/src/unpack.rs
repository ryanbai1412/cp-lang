//! `let (a, b) = (x, y)`, and a `for (a, b) in` over a fused or spliced
//! producer's `(x, y)`, binds each part straight from its own expression:
//! the tuple the pattern takes apart is never built, so its parts are
//! plain locals and a part that is a closure keeps its own type instead of
//! a `std::function` slot of a `std::tuple`, and a part read from a place
//! is a `let` of that place: it lends the place while nothing changes it,
//! moves it at its last read, and copies it only otherwise. It majorizes:
//! the same expressions run in the same order.

use crate::uses::read_counts;
use crate::walk;
use cpl_ir::{AssignOp, Block, Expr, ExprKind, Local, PlaceKind, Program, Stmt};

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
    let reads = read_counts(body, locals.len());
    walk::blocks_mut(body, &mut |block| {
        let mut k = 0;
        while k < block.stmts.len() {
            let n = parts(&block.stmts, k, locals, &reads, captured);
            if let (
                Some(n),
                Stmt::Let(
                    _,
                    Some(Expr {
                        kind: ExprKind::Tuple(items),
                        ..
                    }),
                ),
            ) = (n, &mut block.stmts[k])
            {
                let items = std::mem::take(items);
                block.stmts.remove(k);
                for (i, item) in items.into_iter().enumerate() {
                    match &mut block.stmts[k + i] {
                        Stmt::Let(_, init) => *init = Some(item),
                        Stmt::Assign(_, _, value) => *value = item,
                        _ => {}
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
    let n = items.len();
    let part = |e: &Expr, i: usize| {
        matches!(&e.kind, ExprKind::Field(base, j)
            if *j == i && matches!(base.kind, ExprKind::Local(s) if s == *t))
    };
    // A top-level `var (a, b)` stores each part into its global.
    let binds = |(i, stmt): (usize, &Stmt)| match stmt {
        Stmt::Let(_, Some(e)) => part(e, i),
        Stmt::Assign(place, AssignOp::Set, e) => {
            matches!(place.kind, PlaceKind::Global(_)) && part(e, i)
        }
        _ => false,
    };
    let named = !locals[*t].name.is_empty();
    let rest = stmts.get(k + 1..k + 1 + n)?;
    (n > 0 && !named && !captured(*t) && reads[*t] == n && rest.iter().enumerate().all(binds))
        .then_some(n)
}
