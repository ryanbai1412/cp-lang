//! The one door from codegen to what code can do. Every pass that asks
//! whether code can write, trap, call or be skipped asks here, and every
//! answer comes from [`cpl_ir::effects`] or the stores [`Touch::of_block`] finds;
//! `scripts/debt-ratchet.py` (`codegen_effect_guesses`) rejects a purity or
//! effects helper anywhere else in codegen.

use std::collections::BTreeSet;

use cpl_ir::{Block, Expr, ExprKind, PlaceKind, Program, Stmt};

use crate::devirt::Devirt;
use crate::lend::{whole_reads, Tail};
use crate::walk;

pub(crate) use cpl_ir::effects::{in_domain, known_where, pure_path, trap_free_stmts, Effects};

/// [`cpl_ir::effects::pure_fns`], knowing the closures `devirt` resolves.
pub(crate) fn pure_fns(program: &Program, devirt: &Devirt, traps: bool) -> BTreeSet<usize> {
    cpl_ir::effects::pure_fns(program, traps, &|home, callee| devirt.target(home, callee))
}

pub(crate) fn pure_call(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    home: Option<usize>,
    traps: bool,
    e: &Expr,
) -> bool {
    let target = |h: Option<usize>, callee: &Expr| devirt.target(h, callee);
    cpl_ir::effects::pure_call(program, &target, pure, home, traps, e)
}

/// True when `block` assigns only locals it declares.
pub(crate) fn writes_own(block: &Block) -> bool {
    let mut own = std::collections::BTreeSet::new();
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Let(s, _) | Stmt::For(_, s, ..) = stmt {
            own.insert(*s);
        }
    });
    let mut ok = true;
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(place, ..) = stmt {
            ok &= matches!(place.kind, PlaceKind::Local(s) if own.contains(&s));
        }
    });
    ok
}

/// True when running `block` can neither trap, stop early nor loop forever,
/// so where it runs relative to other effect-free code cannot be seen.
/// Pushes onto `vec` count as nothing; the fused consumer is checked apart.
pub(crate) fn trap_free_block(block: &Block, vec: usize, traps: bool) -> bool {
    trap_free_stmts(&block.stmts, vec, traps, &|_| false)
}

pub(crate) fn writes_nothing(e: &Expr) -> bool {
    Touch::of_expr(e).quiet()
}

/// True when a `sort(by:)` key expression is pure, trap-free, and O(1), so comparing
/// `kf(a) < kf(b)` in-place is safe and avoids allocating `keys`/`order`/`sorted` buffers.
pub(crate) fn is_cheap_pure_key(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Some(inner) | ExprKind::Field(inner, _) => is_cheap_pure_key(inner),
        ExprKind::Unary(cpl_ir::UnOp::Not | cpl_ir::UnOp::BitNot, inner) => {
            is_cheap_pure_key(inner)
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().all(is_cheap_pure_key),
        _ => false,
    }
}

/// A variable, its fields, and cells of its domain-indexed arrays at
/// positions held in variables: naming it runs nothing and cannot trap.
pub(crate) fn safe_place(place: &cpl_ir::Place) -> bool {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => true,
        PlaceKind::Field(base, _) => safe_place(base),
        PlaceKind::Index(base, idxs) => in_domain(&base.ty, idxs) && safe_place(base),
        PlaceKind::Some(_) => false,
    }
}

/// Whether a closure of fn `id` can write its local `root`: the only way
/// code reached through a fn value changes a caller's local.
pub(crate) fn closure_writes(program: &cpl_ir::Program, id: Option<usize>, root: usize) -> bool {
    let Some(id) = id else {
        return true;
    };
    let mut pending = Vec::new();
    walk::deep_exprs(&program.fns[id].body, &mut |e| {
        if let ExprKind::Closure(c) = e.kind {
            pending.push(c);
        }
    });
    pending.into_iter().any(|c| {
        let c = &program.closures[c];
        let mut nested = false;
        walk::deep_exprs(&c.body, &mut |e| {
            nested |= matches!(e.kind, ExprKind::Closure(_))
        });
        let stores = Touch::of_block(&c.body).stored_locals();
        c.captures.iter().any(|cap| {
            !cap.is_global && cap.outer == root && (nested || stores.contains(&cap.inner))
        })
    })
}

/// Whether running `e` may read or write the variable `root`: anything
/// that calls code counts.
pub(crate) fn may_touch(e: &Expr, root: (bool, usize)) -> bool {
    let mut hit = false;
    walk::deep_expr(e, &mut |x| {
        hit |= match &x.kind {
            ExprKind::Local(s) => !root.0 && *s == root.1,
            ExprKind::Global(g) => root.0 && *g == root.1,
            ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, ..) | ExprKind::MutSlice(p, ..) => {
                walk::root(p) == root
            }
            _ => false,
        }
    });
    hit || Touch::of_expr(e).code()
}

impl crate::emit::Emitter<'_> {
    /// What `stmts` may do while a `let` before them lives.
    pub(crate) fn tail(&self, stmts: &[Stmt]) -> Tail {
        let block = Block {
            stmts: stmts.to_vec(),
        };
        let mut whole = whole_reads(&block);
        walk::deep_exprs(&block, &mut |e| {
            if let ExprKind::Closure(id) = e.kind {
                let caps = self.program.closures[id].captures.iter();
                whole.extend(caps.filter(|c| !c.is_global).map(|c| c.outer));
            }
        });
        Tail {
            touch: self.block_touch(&block),
            tested: false,
            lendable: false,
            whole,
        }
    }
}

impl crate::emit::Emitter<'_> {
    /// Whether running `e` leaves the variable `root` as it was: it runs
    /// no fn value or closure, stores nothing into `root`, and calls no fn
    /// that writes it.
    pub(crate) fn leaves(&self, e: &Expr, root: (bool, usize)) -> bool {
        let t = self.uses.table.access(self.program, None, &|_, _| None, e);
        !t.opaque && !t.writes.contains(&root)
    }
}

/// What evaluating an expression touches ([`cpl_ir::effects::Access`]), to
/// tell when the order two pieces of code run in can be seen.
pub(crate) type Touch = cpl_ir::effects::Access;

impl crate::emit::Emitter<'_> {
    /// What evaluating `e` reads, writes and prints.
    pub fn touch(&self, e: &Expr) -> Touch {
        let target = |home, callee: &Expr| self.skippable_target(home, callee);
        let table = &self.uses.table;
        table.access(self.program, self.touch_home(), &target, e)
    }

    /// What running `block` reads, writes and prints.
    pub(crate) fn block_touch(&self, block: &Block) -> Touch {
        let target = |home, callee: &Expr| self.skippable_target(home, callee);
        let table = &self.uses.table;
        table.block_access(self.program, self.touch_home(), &target, block)
    }

    fn touch_home(&self) -> Option<cpl_ir::FnId> {
        self.closure_id.map_or(self.fn_id, |_| None)
    }

    /// The closure a fn value is, when it is one and calling it can be
    /// seen through.
    pub(crate) fn skippable_target(
        &self,
        home: Option<cpl_ir::FnId>,
        callee: &Expr,
    ) -> Option<usize> {
        crate::devirt::skippable(callee)
            .then(|| self.devirt.target(home, callee))
            .flatten()
    }
}
