//! The one door from codegen to what code can do. Every pass that asks
//! whether code can write, trap, call or be skipped asks here, and every
//! answer comes from [`cpl_ir::effects`] or the stores [`Touch::of_block`] finds;
//! `scripts/debt-ratchet.py` (`codegen_effect_guesses`) rejects a purity or
//! effects helper anywhere else in codegen.

use std::collections::BTreeSet;

use cpl_ir::{BinOp, Block, Expr, ExprKind, FnId, PlaceKind, Program, Stmt, Ty};

use crate::devirt::Devirt;
use crate::lend::{whole_reads, Tail};
use crate::walk;

pub(crate) use cpl_ir::effects::{
    in_domain, known_with, pure_path, trap_free_stmts, trap_free_with, Effects,
};

/// Whether every call of `f` sets `root` whole before anything reads it:
/// the global `(true, g)` or the `mut` param `(false, i)`. A call of such a
/// fn ends the old value as a store would, so a name may take it first.
/// `alias` is a global the param is lent from: nothing up to the store may
/// reach it by name.
pub(crate) fn sets_whole_first(
    program: &Program,
    f: FnId,
    root: (bool, usize),
    alias: Option<usize>,
    open: &mut Vec<FnId>,
) -> bool {
    let fun = &program.fns[f];
    if open.contains(&f) || (!root.0 && root.1 >= fun.params.len()) {
        return false;
    }
    if let Some(g) = alias {
        let stored = |s: &Stmt| {
            matches!(s, Stmt::Assign(p, cpl_ir::AssignOp::Set, _)
                if p.kind == PlaceKind::Local(root.1))
        };
        let upto = fun.body.stmts.iter().position(stored).map_or(0, |k| k + 1);
        let before = Block {
            stmts: fun.body.stmts[..upto].to_vec(),
        };
        let mut reached = false;
        walk::deep_exprs(&before, &mut |e| reached |= may_touch(e, (true, g)));
        if reached {
            return false;
        }
    }
    open.push(f);
    let set =
        crate::uses::whole::set_before_read(program, &(root, Vec::new()), &fun.body.stmts, open);
    open.pop();
    set
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
    Touch::of_expr(e).quiet()
}

/// True when a `sort(by:)` key expression is pure, trap-free, and O(1), so comparing
/// `kf(a) < kf(b)` in-place is safe and avoids allocating `keys`/`order`/`sorted` buffers.
/// `traps`: indexing outside a domain and `/ 0` are checked, so only cells of a
/// domain and nonzero literal divisors count;
/// the C++ negates without a check, so `-x` costs nothing. A call counts when its fn
/// only returns such an expression of its parameters (`desc(x)`), and only
/// with `program`: a profile counts a call once per element, as the
/// interpreter makes it.
pub(crate) fn is_cheap_pure_key(program: Option<&Program>, e: &Expr, traps: bool) -> bool {
    cheap_key(program, e, traps, 0)
}

fn cheap_key(program: Option<&Program>, e: &Expr, traps: bool, depth: usize) -> bool {
    let cheap = |e: &Expr| cheap_key(program, e, traps, depth);
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Some(inner) | ExprKind::Field(inner, _) => cheap(inner),
        ExprKind::Unary(cpl_ir::UnOp::Not | cpl_ir::UnOp::BitNot | cpl_ir::UnOp::Neg, inner) => {
            cheap(inner)
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().all(cheap),
        ExprKind::Binary(op, l, r) => {
            let safe = match op {
                BinOp::Div | BinOp::Rem => !traps || matches!(r.kind, ExprKind::Int(k) if k != 0),
                BinOp::Pow | BinOp::Shl | BinOp::Shr => false,
                _ => true,
            };
            safe && l.ty != Ty::Str && cheap(l) && cheap(r)
        }
        ExprKind::Cmp(_, l, r) => cheap(l) && cheap(r),
        ExprKind::Index(base, idxs) => {
            (!traps || cpl_ir::effects::in_domain(&base.ty, idxs))
                && cheap(base)
                && idxs.iter().all(cheap)
        }
        ExprKind::Call(fid, args) if depth < 4 => {
            let Some(f) = program.map(|p| &p.fns[*fid]) else {
                return false;
            };
            let [Stmt::Return(Some(made))] = &f.body.stmts[..] else {
                return false;
            };
            !f.params.iter().any(|p| p.mutable)
                && args
                    .iter()
                    .all(|a| matches!(a, cpl_ir::Arg::Value(v) if cheap(v)))
                && cheap_key(program, made, traps, depth + 1)
        }
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
    /// Whether `e` is plain arithmetic that can neither trap nor overflow
    /// (a product key's flat index): unread, it is not written at all.
    pub(crate) fn trap_free(&self, e: &Expr) -> bool {
        match &e.kind {
            cpl_ir::ExprKind::Local(_)
            | cpl_ir::ExprKind::Global(_)
            | cpl_ir::ExprKind::Int(_)
            | cpl_ir::ExprKind::DomainLen(_)
            | cpl_ir::ExprKind::DomainLo(_) => true,
            cpl_ir::ExprKind::Cast(
                cpl_ir::Cast::Ord | cpl_ir::Cast::Trust | cpl_ir::Cast::EnumOrd(_),
                x,
            ) => self.trap_free(x),
            cpl_ir::ExprKind::Binary(
                op @ (cpl_ir::BinOp::Add | cpl_ir::BinOp::Sub | cpl_ir::BinOp::Mul),
                l,
                r,
            ) => {
                l.ty == cpl_ir::Ty::Int
                    && self.facts.fits(self.facts_home(), *op, l, r)
                    && self.trap_free(l)
                    && self.trap_free(r)
            }
            _ => false,
        }
    }

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
            read_after: BTreeSet::new(),
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

/// A closure that captures nothing and whose body calls nothing and cannot trap,
/// even unchecked (a division by zero still faults):
/// running it interleaved with a producer cannot be told from running it after.
pub(crate) fn pure_closure(program: &Program, cid: usize) -> bool {
    let c = &program.closures[cid];
    c.muts.is_empty()
        && c.captures.is_empty()
        && cpl_ir::effects::pure_body(&c.body, true, &|_| false)
}

/// A pure closure that may read locals it copied when made (`let`s, never `var`s
/// or globals) and call pure fns, so nothing the producer writes can reach it.
pub(crate) fn value_closure(
    program: &Program,
    devirt: &Devirt,
    pure: &BTreeSet<usize>,
    cid: usize,
) -> bool {
    let c = &program.closures[cid];
    let call_ok = |e: &Expr| pure_call(program, devirt, pure, None, true, e);
    pure_closure(program, cid)
        || c.muts.is_empty()
            && c.captures.iter().all(|cap| !cap.by_ref && !cap.is_global)
            && cpl_ir::effects::pure_body(&c.body, true, &call_ok)
}

/// `stmt` may call back into the recursion: a call into `reach`, a call of
/// a fn value, or a closure that makes either.
pub(crate) fn calls_back(program: &Program, reach: &BTreeSet<FnId>, stmt: &Stmt) -> bool {
    let b = Block {
        stmts: vec![stmt.clone()],
    };
    let mut hot = false;
    let mut built = Vec::new();
    walk::deep_exprs(&b, &mut |e| match &e.kind {
        ExprKind::Call(callee, _) => hot |= reach.contains(callee),
        ExprKind::CallExpr(..) => hot = true,
        ExprKind::Closure(c) => built.push(*c),
        _ => {}
    });
    hot || built.into_iter().any(|c| {
        let mut inner = false;
        walk::deep_exprs(&program.closures[c].body, &mut |e| {
            inner |= matches!(e.kind, ExprKind::CallExpr(..) | ExprKind::Closure(_))
                || matches!(e.kind, ExprKind::Call(callee, _) if reach.contains(&callee));
        });
        inner
    })
}

/// The fns from which a call can come back to `target`, `target` among them.
pub(crate) fn reaching(program: &Program, target: FnId) -> BTreeSet<FnId> {
    let calls: Vec<BTreeSet<FnId>> = program
        .fns
        .iter()
        .map(|f| calls_in(program, &f.body))
        .collect();
    let mut reach = BTreeSet::from([target]);
    loop {
        let before = reach.len();
        for (id, out) in calls.iter().enumerate() {
            if !out.is_disjoint(&reach) {
                reach.insert(id);
            }
        }
        if reach.len() == before {
            return reach;
        }
    }
}

/// The fns `block` calls, inside the closures it builds too.
fn calls_in(program: &Program, block: &Block) -> BTreeSet<FnId> {
    let (mut out, mut seen, mut todo) = (BTreeSet::new(), BTreeSet::new(), vec![block]);
    while let Some(b) = todo.pop() {
        let mut built = Vec::new();
        walk::deep_exprs(b, &mut |e| match &e.kind {
            ExprKind::Call(callee, _) => {
                out.insert(*callee);
            }
            ExprKind::Closure(c) => built.push(*c),
            _ => {}
        });
        for c in built {
            if seen.insert(c) {
                todo.push(&program.closures[c].body);
            }
        }
    }
    out
}
