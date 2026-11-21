//! Which locals can be declared where they are first set.
//!
//! C++ ends a variable at the `}` of the block that declares it. A local
//! of the lowered program has no such end, so each use is checked: if
//! every use is in sight of the one statement that declares it, the
//! declaration stays there; otherwise it moves to the top of the fn.

use crate::walk;
use cpl_ir::{Arg, Block, Builtin, Expr, ExprKind, Iter, Place, PlaceKind, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Scopes {
    visible: Vec<BTreeSet<usize>>,
    declared: BTreeMap<usize, u32>,
    hoisted: BTreeSet<usize>,
}

/// The locals of a fn that must be declared at its top. The first
/// `params` slots are parameters and always in sight.
pub(crate) fn hoisted(body: &Block, params: usize) -> BTreeSet<usize> {
    let mut scopes = Scopes::default();
    scopes.visible.push((0..params).collect());
    scopes.block(body);
    scopes.hoisted
}

/// The parameters, of the first `params` slots, that the body names.
pub(crate) fn named_params(body: &Block, params: usize) -> BTreeSet<usize> {
    // With no parameter in sight, each one the body names is "hoisted".
    hoisted(body, 0).into_iter().filter(|slot| *slot < params).collect()
}

impl Scopes {
    fn declare(&mut self, slot: usize) {
        let count = self.declared.entry(slot).or_default();
        *count += 1;
        if *count > 1 {
            self.hoisted.insert(slot);
        }
        if let Some(top) = self.visible.last_mut() {
            top.insert(slot);
        }
    }

    fn uses(&mut self, slot: usize) {
        if !self.visible.iter().any(|scope| scope.contains(&slot)) {
            self.hoisted.insert(slot);
        }
    }

    fn block(&mut self, block: &Block) {
        self.visible.push(BTreeSet::new());
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
        self.visible.pop();
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(slot, init) => {
                if let Some(init) = init {
                    self.expr(init);
                }
                self.declare(*slot);
            }
            Stmt::Assign(place, _, value) => {
                self.expr(value);
                self.place(place);
            }
            Stmt::Expr(e) | Stmt::Give(e) => self.expr(e),
            Stmt::Return(value) => value.iter().for_each(|e| self.expr(e)),
            Stmt::If(cond, then, otherwise) => {
                self.expr(cond);
                self.block(then);
                self.block(otherwise);
            }
            Stmt::While(_, cond, body) => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For(_, slot, iter, body) => {
                match iter {
                    Iter::Range { lo, hi, step, .. } => {
                        self.expr(lo);
                        self.expr(hi);
                        step.iter().for_each(|e| self.expr(e));
                    }
                    Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => self.expr(e),
                }
                self.visible.push(BTreeSet::new());
                self.declare(*slot);
                self.block(body);
                self.visible.pop();
            }
            Stmt::Assert { cond, message, .. } => {
                self.expr(cond);
                message.iter().for_each(|e| self.expr(e));
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.expr(lo);
                self.expr(len);
            }
            Stmt::DefDomainFromVal { val, .. } => self.expr(val),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
        }
    }

    fn place(&mut self, place: &Place) {
        match &place.kind {
            PlaceKind::Local(slot) => self.uses(*slot),
            PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(base),
            PlaceKind::Index(base, indices) => {
                self.place(base);
                indices.iter().for_each(|e| self.expr(e));
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Local(slot) => self.uses(*slot),
            ExprKind::Do(block) => self.block(block),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                if let ExprKind::CallExpr(callee, _) = &e.kind {
                    self.expr(callee);
                }
                for arg in args {
                    match arg {
                        Arg::Value(value) => self.expr(value),
                        Arg::Mut(place) => self.place(place),
                    }
                }
            }
            ExprKind::Mutate(_, place, args) => {
                self.place(place);
                args.iter().for_each(|e| self.expr(e));
            }
            ExprKind::Sort(place, sort_by) => {
                self.place(place);
                match sort_by {
                    cpl_ir::SortBy::Natural => {}
                    cpl_ir::SortBy::Key(slot, key) => {
                        self.visible.push(BTreeSet::new());
                        self.declare(*slot);
                        self.expr(key);
                        self.visible.pop();
                    }
                    cpl_ir::SortBy::Fn { func, .. } => self.expr(func),
                }
            }
            _ => walk::children(e, &mut |child| self.expr(child)),
        }
    }
}

/// The tuple temporaries of a fn that exist only to be taken apart:
/// `let (u, arcs, i) = path[k]` is lowered as a copy of `path[k]`
/// followed by one `let` per field, and nothing else reads the copy. Its
/// fields can be moved out of it rather than copied a second time.
///
/// A slot qualifies when its `let` is directly followed by the `let`s
/// of its fields, each field at most once, and those are its only uses.
pub(crate) fn spent(program: &cpl_ir::Program, body: &Block) -> BTreeSet<usize> {
    let mut reads: BTreeMap<usize, usize> = BTreeMap::new();
    let mut barred = walk::Effects::of_block(body).locals;
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Local(slot) => *reads.entry(*slot).or_default() += 1,
        ExprKind::Closure(id) => {
            barred.extend(program.closures[*id].captures.iter().map(|c| c.outer));
        }
        _ => {}
    });
    // The run of field `let`s being followed: the slot and the fields
    // taken so far.
    let mut run: Option<(usize, BTreeSet<usize>)> = None;
    let mut found = BTreeSet::new();
    let mut close = |run: &mut Option<(usize, BTreeSet<usize>)>| {
        if let Some((slot, fields)) = run.take() {
            if reads.get(&slot) == Some(&fields.len()) && !barred.contains(&slot) {
                found.insert(slot);
            }
        }
    };
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Let(_, Some(init)) = stmt {
            if let ExprKind::Field(base, index) = &init.kind {
                if let (ExprKind::Local(slot), Some((of, fields))) = (&base.kind, &mut run) {
                    if slot == of && fields.insert(*index) {
                        return;
                    }
                    // A second read of one field: the slot is not spent.
                    if slot == of {
                        run = None;
                        return;
                    }
                }
            }
        }
        close(&mut run);
        if let Stmt::Let(slot, Some(init)) = stmt {
            if matches!(init.ty, cpl_ir::Ty::Tuple(_)) {
                run = Some((*slot, BTreeSet::new()));
            }
        }
    });
    close(&mut run);
    found
}

/// The local lists of fn `id` that can live in one buffer for the whole
/// run: each starts empty (`var xs: List<T> = []`) or as one filled row
/// (`var seen: [0..n]Bool = false`), is only indexed,
/// measured, walked and changed in place, and the fn can't be running
/// twice at once (neither it nor anything it calls reaches it again or
/// calls a fn value). The top-level code (`id` is `None`) is never
/// running twice, so a scratch list made anew on every turn of a loop
/// there keeps its buffer too. The C++ then clears the buffer instead of
/// allocating a new one on every call.
pub(crate) fn reused(program: &Program, id: Option<usize>, body: &Block) -> BTreeSet<usize> {
    let mut reach: BTreeSet<usize> = BTreeSet::new();
    let mut todo = vec![id];
    while let Some(f) = todo.pop() {
        let body = f.map_or(body, |f| &program.fns[f].body);
        let mut opaque = false;
        walk::deep_exprs(body, &mut |e| {
            opaque |= matches!(e.kind, ExprKind::CallExpr(..) | ExprKind::Closure(_));
        });
        if opaque {
            return BTreeSet::new();
        }
        // Nothing calls the top-level code.
        if id.is_none() {
            break;
        }
        for callee in walk::Effects::of_block(body).calls {
            if Some(callee) == id {
                return BTreeSet::new();
            }
            if reach.insert(callee) {
                todo.push(Some(callee));
            }
        }
    }
    let mut starts: Vec<usize> = Vec::new();
    let mut twice: BTreeSet<usize> = BTreeSet::new();
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Let(slot, Some(init)) = stmt {
            let empty = matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty()) && matches!(init.ty, Ty::List(_))
                || matches!(&init.kind, ExprKind::Fill(dims, _) if dims.len() == 1);
            if !empty || starts.contains(slot) {
                twice.insert(*slot);
            }
            starts.push(*slot);
        }
    });
    // Every place the whole list is read as a value, against the ones
    // that only index, measure or walk it.
    let mut seen: BTreeMap<usize, isize> = BTreeMap::new();
    let mut plain = |e: &Expr, by: isize| {
        if let ExprKind::Local(slot) = e.kind {
            *seen.entry(slot).or_default() += by;
        }
    };
    walk::deep_exprs(body, &mut |e| {
        plain(e, 1);
        match &e.kind {
            ExprKind::Index(base, _) => plain(base, -1),
            ExprKind::Builtin(Builtin::Len, args) => args.iter().for_each(|a| plain(a, -1)),
            _ => {}
        }
    });
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::For(_, _, Iter::Seq(seq, _), _) = stmt {
            plain(seq, -1);
        }
    });
    starts
        .into_iter()
        .filter(|slot| !twice.contains(slot) && seen.get(slot).copied().unwrap_or(0) == 0)
        .filter(|slot| *slot >= id.map_or(0, |id| program.fns[id].params.len()))
        .collect()
}

/// The slot of the list a `yields` fn builds, when the fn can also hand
/// each value to a callback as it is made: `for x in f(..)` then runs
/// its body between the yields with no list. That shows only if the fn
/// does something a loop body could see, so it must not read input,
/// print, store to a global, change an argument or call a fn value.
pub(crate) fn pusher(f: &cpl_ir::Function, summary: &crate::emit::FnSummary) -> Option<usize> {
    let acc = f.locals.iter().position(|local| local.name == "__yields")?;
    if acc < f.params.len()
        || f.params.iter().any(|p| p.mutable)
        || summary.io
        || summary.opaque
        || !summary.writes.is_empty()
    {
        return None;
    }
    let is_acc = |e: &Expr| matches!(e.kind, ExprKind::Local(slot) if slot == acc);
    let fills = |e: &Expr| {
        matches!(&e.kind, ExprKind::Mutate(cpl_ir::Mutation::Push | cpl_ir::Mutation::Extend, place, _)
            if matches!(place.kind, cpl_ir::PlaceKind::Local(slot) if slot == acc))
    };
    // The list is only made, pushed to and returned.
    let (mut lets, mut pushes, mut returns, mut odd) = (0, 0, 0, false);
    walk::deep_stmts(&f.body, &mut |stmt| match stmt {
        Stmt::Let(slot, _) if *slot == acc => lets += 1,
        Stmt::Expr(e) if fills(e) => pushes += 1,
        Stmt::Return(Some(e)) if is_acc(e) => returns += 1,
        Stmt::Return(_) => odd = true,
        _ => {}
    });
    let (mut named, mut filled) = (0, 0);
    walk::deep_exprs(&f.body, &mut |e| {
        named += usize::from(is_acc(e));
        filled += usize::from(fills(e));
    });
    (!odd && lets == 1 && named == returns && filled == pushes).then_some(acc)
}

/// The struct fields that hold a row of a fixed small count of cells and
/// can be kept inside the struct: `(struct, field)` to the count. A field
/// that is somewhere sorted, grown, filled, updated as a whole or handed
/// to a call as `mut` stays a `std::vector`, which those work on.
pub(crate) fn rows(program: &Program) -> BTreeMap<(usize, usize), u32> {
    let mut rows = BTreeMap::new();
    for (id, def) in program.structs.iter().enumerate() {
        for (field, cells) in &def.fixed {
            if matches!(&def.fields[*field].1, Ty::Array { dims, .. } if dims.len() == 1) {
                rows.insert((id, *field), *cells);
            }
        }
    }
    if rows.is_empty() {
        return rows;
    }
    fn whole(place: &Place, rows: &mut BTreeMap<(usize, usize), u32>) {
        if let PlaceKind::Field(base, field) = &place.kind {
            if let Ty::Struct(id) = base.ty {
                rows.remove(&(id, *field));
            }
        }
    }
    let bodies = [&program.setup, &program.main]
        .into_iter()
        .chain(program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body));
    for body in bodies {
        crate::walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::Assign(place, op, _) = stmt {
                if *op != cpl_ir::AssignOp::Set {
                    whole(place, &mut rows);
                }
            }
        });
        crate::walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _) => whole(place, &mut rows),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                for arg in args {
                    if let cpl_ir::Arg::Mut(place) = arg {
                        whole(place, &mut rows);
                    }
                }
            }
            _ => {}
        });
    }
    rows
}
