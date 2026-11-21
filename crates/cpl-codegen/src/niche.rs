//! Which optionals fit in one scalar.
//!
//! `?V` is a `std::optional<V>`: the value and a flag beside it, 16 bytes
//! for an `Int`. Where some value of the scalar is never a `V`, that
//! value can say `none` instead, and the optional is the size of what it
//! holds. Three cases are decided by the type alone: a position is an
//! ordinal from 0, so -1 is free; a `Bool` is 0 or 1 in a byte; a `Char`
//! is a byte in sixteen bits.
//!
//! `Int` stops at `-(2^63 - 1)`, so a `?Int` is one `Int` everywhere,
//! with the raw `INT64_MIN` as `none`.

use crate::names::Niches;
use crate::walk;
use cpl_ir::{Arg, Block, ExprKind, Place, PlaceKind, Program, Stmt, Ty};

/// True when the inside of a `?Bool` or `?Char` is taken as a place
/// somewhere in `block`: `(o ??= x) = y`.
fn small_place(block: &Block, bools: &mut bool, chars: &mut bool) {
    let mut place = |mut place: &Place| loop {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => return,
            PlaceKind::Some(base) => {
                *bools &= base.ty != Ty::opt(Ty::Bool);
                *chars &= base.ty != Ty::opt(Ty::Char);
                place = base;
            }
            PlaceKind::Index(base, _) | PlaceKind::Field(base, _) => place = base,
        }
    };
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(target, ..) = stmt {
            place(target);
        }
    });
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    place(target);
                }
            }
        }
        ExprKind::Mutate(_, target, _) | ExprKind::Sort(target, _, _) => place(target),
        _ => {}
    });
}

/// Whether `?Bool` and `?Char` are packed in this program.
pub(crate) fn small(program: &Program) -> Niches {
    let (mut bools, mut chars) = (true, true);
    let bodies = [&program.setup, &program.main]
        .into_iter()
        .chain(program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body));
    for body in bodies {
        small_place(body, &mut bools, &mut chars);
    }
    Niches { bools, chars }
}
