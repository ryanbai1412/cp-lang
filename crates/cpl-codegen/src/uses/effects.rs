//! The one door from codegen to what code can do. Every pass that asks
//! whether code can write, trap, call or be skipped asks here, and every
//! answer comes from [`cpl_ir::effects`] or the stores [`Stores`] finds;
//! `scripts/debt-ratchet.py` (`codegen_effect_guesses`) rejects a purity or
//! effects helper anywhere else in codegen.

use std::collections::BTreeSet;

use cpl_ir::{Block, Expr, ExprKind, PlaceKind, Program, Stmt};

use crate::devirt::Devirt;
use crate::lend::{whole_reads, Tail};
use crate::walk;

pub(crate) use cpl_ir::effects::{in_domain, known_where, pure_path, trap_free_stmts, Effects};
/// The locals and globals code assigns and the fns it calls by name.
pub(crate) use cpl_ir::walk::Effects as Stores;

/// The kinds of code an expression or block runs, read from its syntax:
/// what a pass needs to know before it reorders, skips or lends across it.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Runs {
    /// Calls a fn by name.
    pub(crate) fns: bool,
    /// Calls a fn value.
    pub(crate) values: bool,
    /// Makes a closure, which may later run with its captures.
    pub(crate) closures: bool,
    /// Changes a container in place.
    pub(crate) mutates: bool,
    /// Prints.
    pub(crate) prints: bool,
}

impl Runs {
    fn add(&mut self, e: &Expr) {
        match e.kind {
            ExprKind::Call(..) => self.fns = true,
            ExprKind::CallExpr(..) => self.values = true,
            ExprKind::Closure(_) => self.closures = true,
            ExprKind::Mutate(..) => self.mutates = true,
            ExprKind::Print(..) => self.prints = true,
            _ => {}
        }
    }

    pub(crate) fn of_expr(e: &Expr) -> Runs {
        let mut runs = Runs::default();
        walk::deep_expr(e, &mut |x| runs.add(x));
        runs
    }

    pub(crate) fn of_block(b: &Block) -> Runs {
        let mut runs = Runs::default();
        walk::deep_exprs(b, &mut |x| runs.add(x));
        runs
    }

    /// Calls a fn, by name or as a value.
    pub(crate) fn calls(&self) -> bool {
        self.fns || self.values
    }

    /// Runs code this pass cannot see from here: a call or a closure.
    pub(crate) fn code(&self) -> bool {
        self.calls() || self.closures
    }

    /// Runs code whose target is not named: a fn value or a closure.
    pub(crate) fn opaque(&self) -> bool {
        self.values || self.closures
    }

    /// Runs no code and changes no container.
    pub(crate) fn quiet(&self) -> bool {
        !self.code() && !self.mutates
    }
}

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
    Runs::of_expr(e).quiet()
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

/// Adds what running closure `c` can change to `effects`, with its
/// by-reference captures as the caller's variables. False when that is not
/// known: it calls a fn value, makes a closure, or takes `mut` parameters.
pub(crate) fn closure_effects(
    c: &cpl_ir::ClosureDef,
    effects: &mut Stores,
    calls: &mut bool,
) -> bool {
    let known = c.muts.is_empty() && !Runs::of_block(&c.body).opaque();
    let inner = Stores::of_block(&c.body);
    for cap in c
        .captures
        .iter()
        .filter(|cap| inner.locals.contains(&cap.inner))
    {
        match cap.is_global {
            true => effects.globals.insert(cap.outer),
            false => effects.locals.insert(cap.outer),
        };
    }
    *calls |= !inner.calls.is_empty();
    effects.globals.extend(inner.globals);
    effects.calls.extend(inner.calls);
    known
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
        let stores = Stores::of_block(&c.body).locals;
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
    hit || Runs::of_expr(e).code()
}

impl Tail {
    pub(crate) fn of(program: &cpl_ir::Program, stmts: &[Stmt]) -> Tail {
        let block = Block {
            stmts: stmts.to_vec(),
        };
        let (mut calls, mut opaque, mut closures) = (false, false, Vec::new());
        let mut whole = whole_reads(&block);
        let mut effects = Stores::of_block(&block);
        // A local the block binds to a closure and never sets again calls
        // that closure, whose effects are counted below.
        let mut bound = std::collections::BTreeSet::new();
        walk::deep_stmts(&block, &mut |stmt| {
            if let Stmt::Let(
                s,
                Some(Expr {
                    kind: ExprKind::Closure(_),
                    ..
                }),
            ) = stmt
            {
                bound.insert(*s);
            }
        });
        bound.retain(|s| !effects.locals.contains(s));
        walk::deep_exprs(&block, &mut |e| match &e.kind {
            ExprKind::Call(..) => calls = true,
            ExprKind::CallExpr(callee, _) => {
                calls = true;
                opaque |= !matches!(callee.kind, ExprKind::Local(s) if bound.contains(&s));
            }
            ExprKind::Closure(id) => closures.push(*id),
            _ => {}
        });
        for &id in &closures {
            let caps = program.closures[id].captures.iter();
            whole.extend(caps.filter(|c| !c.is_global).map(|c| c.outer));
        }
        for id in closures {
            opaque |= !closure_effects(&program.closures[id], &mut effects, &mut calls);
        }
        Tail {
            effects,
            calls,
            opaque,
            tested: false,
            lendable: false,
            whole,
        }
    }
}

impl super::Uses {
    /// Whether running `e` leaves the variable `root` as it was: it runs
    /// no fn value or closure, stores nothing into `root`, and calls no fn
    /// that writes it.
    pub(crate) fn leaves(&self, e: &Expr, root: (bool, usize)) -> bool {
        let stores = Stores::of_expr(e);
        let writes = |f: &usize| root.0 && self.effects(*f).writes.contains(&root.1);
        let stored = match root.0 {
            true => &stores.globals,
            false => &stores.locals,
        };
        !Runs::of_expr(e).opaque() && !stores.calls.iter().any(writes) && !stored.contains(&root.1)
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
    fn skippable_target(&self, home: Option<cpl_ir::FnId>, callee: &Expr) -> Option<usize> {
        crate::devirt::skippable(callee)
            .then(|| self.devirt.target(home, callee))
            .flatten()
    }
}
