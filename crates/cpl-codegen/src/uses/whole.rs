//! Whether a store sets a variable path whole before anything reads it:
//! `let a = s.ks` may take `s.ks` instead of copying it when the next
//! statements only push onto `s.ks` and then set it, or call a fn that
//! sets it whole first.

use super::effects::{trap_free_with, Touch};
use crate::walk;
use cpl_ir::{Arg, AssignOp, Block, Expr, ExprKind, FnId, Place, PlaceKind, Program, Stmt};

/// A variable and the fields read on it, outermost first: `self.ks`.
pub(crate) fn path_of(e: &Expr) -> Option<((bool, usize), Vec<usize>)> {
    match &e.kind {
        ExprKind::Local(s) => Some(((false, *s), Vec::new())),
        ExprKind::Global(g) => Some(((true, *g), Vec::new())),
        ExprKind::Unwrap(base, _) => path_of(base),
        ExprKind::Field(base, f) => path_of(base).map(|(root, mut fields)| {
            fields.push(*f);
            (root, fields)
        }),
        // A match binding's payload: a step no field number can equal.
        ExprKind::Payload(base, variant, k) => path_of(base).map(|(root, mut fields)| {
            fields.push(PAYLOAD_STEP | variant << 16 | k);
            (root, fields)
        }),
        _ => None,
    }
}

const PAYLOAD_STEP: usize = 1 << 62;

pub(crate) fn place_path(p: &Place) -> Option<((bool, usize), Vec<usize>)> {
    match &p.kind {
        PlaceKind::Local(s) => Some(((false, *s), Vec::new())),
        PlaceKind::Global(g) => Some(((true, *g), Vec::new())),
        PlaceKind::Field(base, f) => place_path(base).map(|(root, mut fields)| {
            fields.push(*f);
            (root, fields)
        }),
        _ => None,
    }
}

/// Whether `stmt` may read or change the variable path `path`, or leave
/// the block early: anything but `let`s and stores that stay off it.
pub(crate) fn touches(stmt: &Stmt, (root, fields): &((bool, usize), Vec<usize>)) -> bool {
    let block = Block {
        stmts: vec![stmt.clone()],
    };
    // Code called by name sees a local only through its arguments, which
    // `uses` counts; a global it may read itself.
    let access = Touch::of_block(&block);
    let opaque = match root.0 {
        true => access.code(),
        false => access.unnamed(),
    };
    let (mut uses, mut elsewhere) = (0usize, 0usize);
    crate::uses::places::each_variable(&block, &mut |v| uses += usize::from(v == *root));
    crate::uses::places::each_field(&block, &mut |base, f| {
        if fields.first().is_some_and(|f0| *f0 != f) {
            elsewhere +=
                usize::from(path_of(base).is_some_and(|(r, fs)| r == *root && fs.is_empty()));
        }
    });
    // A payload of a sibling field reads the root, not the field.
    crate::uses::places::each_payload(&block, &mut |e, base| {
        elsewhere += usize::from(
            path_of(e).is_some_and(|(r, fs)| {
                r == *root && fs.len() == 1 && fields.first().is_some_and(|f0| *f0 != fs[0])
            }) && path_of(base).is_some_and(|(_, fs)| fs.is_empty()),
        );
    });
    let stored = match stmt {
        Stmt::Let(..) | Stmt::Expr(_) => false,
        Stmt::Assign(place, _, _) => place_path(place)
            .map_or(walk::root(place) == *root, |(r, fs)| {
                r == *root && fs.iter().zip(fields).all(|(a, b)| a == b)
            }),
        _ => true,
    };
    opaque || stored || uses != elsewhere
}

/// Whether the variable path `init` names is set whole by one of the
/// statements after its `let` before any of them reads it, changes it or
/// leaves: the `let` may take its value instead of copying it.
pub(crate) fn overwritten_before_read(program: &Program, init: &Expr, rest: &[Stmt]) -> bool {
    path_of(init).is_some_and(|path| set_before_read(program, &path, rest, &mut Vec::new()))
}

/// Whether `rest` sets the local `slot` whole before anything reads it.
pub(crate) fn local_set_before_read(program: &Program, slot: usize, rest: &[Stmt]) -> bool {
    set_before_read(program, &((false, slot), Vec::new()), rest, &mut Vec::new())
}

/// Whether `rest` sets `path` (or a place holding it) whole before anything
/// reads it, a call counting as a store when its fn does that first to the
/// global or `mut` param it reaches the path through. `open` holds the fns
/// being asked about, so a recursive one answers no.
pub(crate) fn set_before_read(
    program: &Program,
    path: &((bool, usize), Vec<usize>),
    rest: &[Stmt],
    open: &mut Vec<FnId>,
) -> bool {
    for stmt in rest {
        if call_sets_whole(program, stmt, path, open) {
            return true;
        }
        if let Stmt::Assign(place, AssignOp::Set, value) = stmt {
            // Setting the path or a place holding it replaces the whole of it.
            if place_path(place)
                .is_some_and(|(root, fields)| root == path.0 && path.1.starts_with(&fields))
            {
                let value = Stmt::Expr(value.clone());
                return !touches(&value, path);
            }
        }
        if touches(stmt, path) && !only_grows(stmt, path) {
            return false;
        }
    }
    false
}

/// Whether `stmt` calls a fn that sets the root of `path` whole before it
/// reads it, the root reached as that global or as the call's one `mut`
/// argument, with no other argument reading the path.
fn call_sets_whole(
    program: &Program,
    stmt: &Stmt,
    path: &((bool, usize), Vec<usize>),
    open: &mut Vec<FnId>,
) -> bool {
    let Stmt::Expr(Expr {
        kind: ExprKind::Call(f, args),
        ..
    }) = stmt
    else {
        return false;
    };
    let root = path.0;
    let mut through = None;
    for (i, arg) in args.iter().enumerate() {
        match arg {
            Arg::Value(e) if touches(&Stmt::Expr(e.clone()), path) => return false,
            Arg::Value(_) => {}
            Arg::Mut(p) if walk::root(p) != root => {}
            Arg::Mut(p)
                if matches!(p.kind, PlaceKind::Local(_) | PlaceKind::Global(_))
                    && through.is_none() =>
            {
                through = Some(i)
            }
            Arg::Mut(_) => return false,
        }
    }
    let effects = crate::uses::effects::sets_whole_first;
    match (root, through) {
        ((true, g), None) => effects(program, *f, (true, g), None, open),
        // A global the call lends as a `mut` param: the fn must not reach it
        // by its name too before it stores the param.
        ((true, g), Some(i)) => effects(program, *f, (false, i), Some(g), open),
        ((false, _), Some(i)) => effects(program, *f, (false, i), None, open),
        _ => false,
    }
}

/// Whether `stmt` only pushes onto the variable path `path`, from values
/// that neither read it nor can trap (`for i in 0..2: s.ks.push(i)`): what
/// it builds there is never seen when a store sets the path whole before
/// anything reads it, so the old value may be taken first.
fn only_grows(stmt: &Stmt, path: &((bool, usize), Vec<usize>)) -> bool {
    match stmt {
        Stmt::Expr(Expr {
            kind: ExprKind::Mutate(cpl_ir::Mutation::Push, place, args),
            ..
        }) => {
            place_path(place).as_ref() == Some(path)
                && args.iter().all(|arg| {
                    !touches(&Stmt::Expr(arg.clone()), path)
                        && trap_free_with(arg, true, &|_| false)
                })
        }
        Stmt::For(id, slot, iter, body) => {
            let head = Stmt::For(*id, *slot, iter.clone(), Block { stmts: Vec::new() });
            let mut free = true;
            walk::stmt_exprs(&head, &mut |e| {
                free &= !touches(&Stmt::Expr(e.clone()), path)
            });
            free && body.stmts.iter().all(|s| only_grows(s, path))
        }
        _ => false,
    }
}

/// Whether nothing in `rest`, nor code it calls, reads or sets the
/// variable path `e` names.
pub(crate) fn untouched_in(e: &Expr, rest: &[Stmt]) -> bool {
    path_of(e).is_some_and(|path| !rest.iter().any(|stmt| touches(stmt, &path)))
}
