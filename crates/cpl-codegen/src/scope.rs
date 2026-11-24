//! Which locals can be declared where they are first set.
//!
//! C++ ends a variable at the `}` of the block that declares it. A local
//! of the lowered program has no such end, so each use is checked: if
//! every use is in sight of the one statement that declares it, the
//! declaration stays there; otherwise it moves to the top of the fn.

use crate::uses::effects::Touch;
use crate::walk;
use cpl_ir::{
    Arg, Block, Builtin, CmpOp, Expr, ExprKind, Iter, Place, PlaceKind, Program, Stmt, Ty,
};
use std::collections::{BTreeMap, BTreeSet};

mod narrow;

pub(crate) use narrow::{narrow_elems, narrow_ints};

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
    hoisted(body, 0)
        .into_iter()
        .filter(|slot| *slot < params)
        .collect()
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
                    Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
                    Iter::Range { lo, hi, step, .. } => {
                        self.expr(lo);
                        self.expr(hi);
                        step.iter().for_each(|e| self.expr(e));
                    }
                    Iter::RangeVal(e, _) | Iter::Seq(e, _) => self.expr(e),
                    Iter::TreeRange {
                        container, lo, hi, ..
                    } => {
                        self.expr(container);
                        if let Some((e, _)) = lo {
                            self.expr(e);
                        }
                        if let Some((e, _)) = hi {
                            self.expr(e);
                        }
                    }
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
            ExprKind::Sort(place, sort_by, desc) => {
                self.place(place);
                if let Some(desc) = desc {
                    self.expr(desc);
                }
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
    let mut barred = Touch::of_block(body).stored_locals();
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

/// `for v in r: if v >= e: ..` where the body never changes the local
/// `e`: the loop can start at `e` instead of testing it every turn. Gives
/// `e` and the `if`'s body.
pub(crate) fn start_guard(slot: usize, body: &Block) -> Option<(&Expr, &Block)> {
    let [Stmt::If(cond, then, otherwise)] = body.stmts.as_slice() else {
        return None;
    };
    let (ExprKind::Cmp(CmpOp::Ge, v, e) | ExprKind::Cmp(CmpOp::Le, e, v)) = &cond.kind else {
        return None;
    };
    let (ExprKind::Local(v), ExprKind::Local(bound)) = (&v.kind, &e.kind) else {
        return None;
    };
    let opaque = Touch::of_block(body).unnamed();
    let set = Touch::of_block(body).stored_locals();
    (*v == slot && *bound != slot && otherwise.stmts.is_empty() && !opaque && !set.contains(bound))
        .then_some((e, then))
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
        if Touch::of_block(body).unnamed() {
            return BTreeSet::new();
        }
        // Nothing calls the top-level code.
        if id.is_none() {
            break;
        }
        for callee in Touch::of_block(body).calls {
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
    // `let xs = [.. for ..]`: `xs` names the list its block built and gave.
    let mut built: Vec<(usize, usize)> = Vec::new();
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Let(slot, Some(init)) = stmt {
            if let ExprKind::Do(block) = &init.kind {
                if let (Some(Stmt::Let(list, _)), Some(Stmt::Give(gave))) =
                    (block.stmts.first(), block.stmts.last())
                {
                    if matches!(gave.kind, ExprKind::Local(l) if l == *list) {
                        built.push((*slot, *list));
                    }
                }
            }
            let empty = matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty())
                && matches!(init.ty, Ty::List(_))
                || matches!(&init.kind, ExprKind::Fill(dims, _) if dims.len() == 1)
                || built.last().is_some_and(|(s, _)| s == slot);
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
    for (_, list) in &built {
        *seen.entry(*list).or_default() -= 1;
    }
    let mut kept: BTreeSet<usize> = starts
        .into_iter()
        .filter(|slot| !twice.contains(slot) && seen.get(slot).copied().unwrap_or(0) == 0)
        .filter(|slot| *slot >= id.map_or(0, |id| program.fns[id].params.len()))
        .collect();
    // The name and its list share one buffer, so both are kept or neither.
    for (name, list) in built {
        if !(kept.contains(&name) && kept.contains(&list)) {
            kept.remove(&name);
            kept.remove(&list);
        }
    }
    kept
}

/// The struct fields that hold a row of a fixed small count of cells and
/// can be kept inside the struct: `(struct, field)` to the count. A field
/// that is somewhere sorted, grown, filled, updated as a whole or handed
/// to a call as `mut` stays a `std::vector`, which those work on.
pub(crate) fn rows(
    program: &Program,
    facts: &crate::ranges::Facts<'_>,
) -> BTreeMap<(usize, usize), u32> {
    let mut rows = BTreeMap::new();
    for (id, def) in program.structs.iter().enumerate() {
        for (field, cells) in &def.fixed {
            if matches!(&def.fields[*field].1, Ty::Array { dims, .. } if dims.len() == 1) {
                rows.insert((id, *field), *cells);
            }
        }
        // A Bitset<D> over a domain with a small static upper bound (<= 4096 bits)
        // keeps its U64 words inline in a `cpl::Row<U64, W>` instead of heap-allocating.
        let words = def.fields.iter().position(|(name, ty)| {
            name == "w" && matches!(ty, Ty::List(elem) if elem.as_ref() == &Ty::U64)
        });
        if let (true, Some(words_field)) = (def.name.starts_with("Bitset"), words) {
            let prefix = format!("{}_", crate::names::fn_name(&def.name));
            let mut domain_id = None;
            for f in &program.fns {
                if !crate::names::fn_name(&f.name).starts_with(&prefix) {
                    continue;
                }
                for local in f.locals.iter().take(f.params.len()) {
                    if let Ty::Pos(d) = local.ty {
                        domain_id = Some(d);
                        break;
                    }
                }
                if domain_id.is_none() {
                    crate::walk::deep_exprs(&f.body, &mut |e| {
                        if domain_id.is_none() {
                            if let ExprKind::DomainLen(d) = e.kind {
                                domain_id = Some(d);
                            }
                        }
                    });
                }
                if domain_id.is_some() {
                    break;
                }
            }
            if let Some(d) = domain_id {
                if let Some(max_k) = facts.domain_max_len(d) {
                    if (0..=4096).contains(&max_k) {
                        let words = ((max_k + 63) / 64).max(1) as u32;
                        rows.insert((id, words_field), words);
                    }
                }
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
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _) => whole(place, &mut rows),
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

/// Value parameters each fn returns as a field of the struct or tuple it
/// builds, with nothing else in that return naming them. They are taken by
/// value so the return moves them, and a caller at its last use moves in.
pub(crate) fn sunk_params(program: &Program) -> Vec<BTreeSet<usize>> {
    program
        .fns
        .iter()
        .map(|f| {
            let mut sunk = BTreeSet::new();
            walk::deep_stmts(&f.body, &mut |stmt| {
                let Stmt::Return(Some(value)) = stmt else {
                    return;
                };
                let ExprKind::Tuple(items) = &value.kind else {
                    return;
                };
                if matches!(
                    value.ty,
                    Ty::Unit | Ty::Error | Ty::Map(..) | Ty::HashMap(..)
                ) {
                    return;
                }
                for (idx, item) in items.iter().enumerate() {
                    let ExprKind::Local(slot) = item.kind else {
                        continue;
                    };
                    if slot >= f.params.len()
                        || f.params[slot].mutable
                        || item.ty.is_scalar()
                        || matches!(item.ty, Ty::Fn(..))
                    {
                        continue;
                    }
                    let elsewhere = items.iter().enumerate().any(|(j, other)| {
                        j != idx && {
                            let mut used = false;
                            walk::deep_expr(other, &mut |e| {
                                used |= matches!(e.kind, ExprKind::Local(s) if s == slot);
                            });
                            used
                        }
                    });
                    if !elsewhere {
                        sunk.insert(slot);
                    }
                }
            });
            sunk.extend(crate::uses::copied_once(f));
            sunk
        })
        .collect()
}

/// 1D fixed-size arrays in locals and globals whose length is a compile-time
/// constant in `1..=64` and that never escape to a `std::vector`-requiring context.
#[inline(never)]
pub(crate) fn fixed_arrays(
    program: &Program,
    narrow_elems: &BTreeMap<cpl_ir::ElemLoc, cpl_ir::NarrowLayout>,
) -> Vec<(cpl_ir::ElemLoc, u8)> {
    use cpl_ir::{Builtin, DimSpec, ElemLoc, Iter, Mutation, ScopeId};

    let is_1d = |ty: &Ty| matches!(ty, Ty::Array { dims, .. } if dims.len() == 1);
    let get = |v: &[(ElemLoc, u8)], l: ElemLoc| v.iter().find(|(x, _)| *x == l).map(|(_, k)| *k);
    let del = |v: &mut Vec<(ElemLoc, u8)>, l: ElemLoc| -> bool {
        v.iter()
            .position(|(x, _)| *x == l)
            .map(|i| v.swap_remove(i))
            .is_some()
    };
    let fill_k = |dims: &[DimSpec]| -> Option<u8> {
        if dims.len() != 1 {
            return None;
        }
        let n = match &dims[0] {
            DimSpec::Int(e) | DimSpec::IntFrom(_, e) => match e.kind {
                ExprKind::Int(k) => k,
                _ => return None,
            },
            DimSpec::Enum(eid) => program.enums[*eid].variants.len() as i64,
            _ => return None,
        };
        (1..=64).contains(&n).then_some(n as u8)
    };
    let p_loc = |s: ScopeId, p: &Place| match &p.kind {
        PlaceKind::Global(g) => Some(ElemLoc::Global(*g)),
        PlaceKind::Local(l) => Some(ElemLoc::Local(s, *l)),
        _ => None,
    };
    fn e_loc(s: ScopeId, e: &Expr) -> Option<ElemLoc> {
        match &e.kind {
            ExprKind::Global(g) => Some(ElemLoc::Global(*g)),
            ExprKind::Local(l) => Some(ElemLoc::Local(s, *l)),
            ExprKind::Do(b) => match b.stmts.last() {
                Some(Stmt::Give(x)) => e_loc(s, x),
                _ => None,
            },
            _ => None,
        }
    }

    let mut fixed: Vec<(ElemLoc, u8)> = Vec::new();
    let mut banned: Vec<ElemLoc> = narrow_elems.keys().copied().collect();
    let mut ban = |l: ElemLoc| {
        if !banned.contains(&l) {
            banned.push(l);
        }
    };
    for &fid in program.judge_fns.iter().chain(program.entry_fns.iter()) {
        for slot in 0..program.fns[fid].params.len() {
            ban(ElemLoc::Local(ScopeId::Fn(fid), slot));
        }
    }
    for (cid, c) in program.closures.iter().enumerate() {
        for slot in 0..c.params.len() {
            ban(ElemLoc::Local(ScopeId::Closure(cid), slot));
        }
        for cap in &c.captures {
            ban(ElemLoc::Local(ScopeId::Closure(cid), cap.inner));
            if cap.is_global {
                ban(ElemLoc::Global(cap.outer));
            }
        }
    }
    let bodies: Vec<(ScopeId, &cpl_ir::Block)> = [
        (ScopeId::Main, &program.setup),
        (ScopeId::Main, &program.main),
    ]
    .into_iter()
    .chain(
        program
            .fns
            .iter()
            .enumerate()
            .map(|(i, f)| (ScopeId::Fn(i), &f.body)),
    )
    .chain(
        program
            .closures
            .iter()
            .enumerate()
            .map(|(i, c)| (ScopeId::Closure(i), &c.body)),
    )
    .collect();
    for &(scope, body) in &bodies {
        crate::walk::deep_exprs(body, &mut |e| {
            if let ExprKind::Closure(cid) = &e.kind {
                for cap in &program.closures[*cid].captures {
                    if !cap.is_global {
                        ban(ElemLoc::Local(scope, cap.outer));
                    }
                }
            }
        });
    }

    // Pass 1: seed constant lengths from Fill and propagate across Let/Assign(Set).
    loop {
        let mut changed = false;
        for &(scope, body) in &bodies {
            crate::walk::deep_stmts(body, &mut |stmt| {
                let (loc, src) = match stmt {
                    Stmt::Let(slot, Some(init)) if is_1d(&init.ty) => {
                        (ElemLoc::Local(scope, *slot), Some(init))
                    }
                    Stmt::Let(slot, None) => (ElemLoc::Local(scope, *slot), None),
                    Stmt::Assign(p, cpl_ir::AssignOp::Set, v) if is_1d(&p.ty) => {
                        let Some(l) = p_loc(scope, p) else { return };
                        (l, Some(v))
                    }
                    _ => return,
                };
                if banned.contains(&loc) {
                    return;
                }
                let Some(val) = src else {
                    changed |= del(&mut fixed, loc);
                    banned.push(loc);
                    return;
                };
                let mut cur = val;
                while let ExprKind::Do(b) = &cur.kind {
                    if let Some(Stmt::Give(x)) = b.stmts.last() {
                        cur = x;
                    } else {
                        break;
                    }
                }
                let k = match &cur.kind {
                    ExprKind::Fill(dims, _) => fill_k(dims),
                    _ => e_loc(scope, cur).and_then(|l| get(&fixed, l)),
                };
                if let Some(k) = k {
                    match get(&fixed, loc) {
                        Some(p) if p == k => {}
                        Some(_) => {
                            del(&mut fixed, loc);
                            banned.push(loc);
                            changed = true;
                        }
                        None => {
                            fixed.push((loc, k));
                            changed = true;
                        }
                    }
                } else if e_loc(scope, val).is_none() {
                    del(&mut fixed, loc);
                    banned.push(loc);
                    changed = true;
                }
            });
            crate::walk::deep_exprs(body, &mut |e| {
                let ExprKind::Call(fid, args) = &e.kind else {
                    return;
                };
                let f = &program.fns[*fid];
                for (slot, arg) in args.iter().enumerate() {
                    if slot >= f.params.len() || !is_1d(&f.locals[slot].ty) {
                        continue;
                    }
                    let loc = ElemLoc::Local(ScopeId::Fn(*fid), slot);
                    if banned.contains(&loc) {
                        continue;
                    }
                    let src_loc = match arg {
                        Arg::Value(v) => e_loc(scope, v),
                        Arg::Mut(p) => p_loc(scope, p),
                    };
                    if let Some(k) = src_loc.and_then(|l| get(&fixed, l)) {
                        match get(&fixed, loc) {
                            Some(p) if p == k => {}
                            Some(_) => {
                                del(&mut fixed, loc);
                                banned.push(loc);
                                changed = true;
                            }
                            None => {
                                fixed.push((loc, k));
                                changed = true;
                            }
                        }
                    } else if src_loc.is_none() {
                        del(&mut fixed, loc);
                        banned.push(loc);
                        changed = true;
                    }
                }
            });
        }
        if !changed {
            break;
        }
    }
    if fixed.is_empty() {
        return fixed;
    }

    // Pass 2: disqualify any candidate that escapes to an unsupported context.
    // Every `Expr`/`Place` pointer that is an *allowed* direct use of a fixed array
    // is skipped; any other occurrence of a fixed-array `Local`/`Global` bans it.
    let mut allowed_exprs: Vec<*const Expr> = Vec::new();
    let mut allowed_places: Vec<*const Place> = Vec::new();
    let mut flow_edges: Vec<(ElemLoc, ElemLoc)> = Vec::new();
    let mut non_flow_dst: Vec<ElemLoc> = Vec::new();

    for &(scope, body) in &bodies {
        crate::walk::deep_stmts(body, &mut |stmt| match stmt {
            Stmt::Let(slot, Some(init)) if is_1d(&init.ty) => {
                let dst = ElemLoc::Local(scope, *slot);
                let mut cur = init;
                while let ExprKind::Do(b) = &cur.kind {
                    allowed_exprs.push(cur as *const Expr);
                    if let Some(Stmt::Give(x)) = b.stmts.last() {
                        cur = x;
                    } else {
                        break;
                    }
                }
                if let ExprKind::Fill(dims, _) = &cur.kind {
                    if fill_k(dims).is_some() {
                        allowed_exprs.push(cur as *const Expr);
                    } else {
                        non_flow_dst.push(dst);
                    }
                } else if let Some(src) = e_loc(scope, cur) {
                    allowed_exprs.push(cur as *const Expr);
                    flow_edges.push((dst, src));
                } else {
                    non_flow_dst.push(dst);
                }
            }
            Stmt::Assign(pl, cpl_ir::AssignOp::Set, val) if is_1d(&pl.ty) => {
                if let Some(dst) = p_loc(scope, pl) {
                    allowed_places.push(pl as *const Place);
                    let mut cur = val;
                    while let ExprKind::Do(b) = &cur.kind {
                        allowed_exprs.push(cur as *const Expr);
                        if let Some(Stmt::Give(x)) = b.stmts.last() {
                            cur = x;
                        } else {
                            break;
                        }
                    }
                    if let ExprKind::Fill(dims, _) = &cur.kind {
                        if fill_k(dims).is_some() {
                            allowed_exprs.push(cur as *const Expr);
                        } else {
                            non_flow_dst.push(dst);
                        }
                    } else if let Some(src) = e_loc(scope, cur) {
                        allowed_exprs.push(cur as *const Expr);
                        flow_edges.push((dst, src));
                    } else {
                        non_flow_dst.push(dst);
                    }
                }
            }
            Stmt::For(_, _, Iter::Seq(seq, _), _)
                if is_1d(&seq.ty)
                    && matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_)) =>
            {
                allowed_exprs.push(seq as *const Expr);
            }
            _ => {}
        });
        crate::walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Index(base, idx)
                if idx.len() == 1
                    && is_1d(&base.ty)
                    && matches!(base.kind, ExprKind::Local(_) | ExprKind::Global(_)) =>
            {
                allowed_exprs.push(&**base as *const Expr);
            }
            ExprKind::Builtin(Builtin::Len | Builtin::First | Builtin::Last, args) => {
                for a in args {
                    if is_1d(&a.ty) && matches!(a.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        allowed_exprs.push(a as *const Expr);
                    }
                }
            }
            ExprKind::Cmp(_, l, r) if is_1d(&l.ty) && is_1d(&r.ty) => {
                if let (Some(a), Some(b)) = (e_loc(scope, l), e_loc(scope, r)) {
                    allowed_exprs.push(&**l as *const Expr);
                    allowed_exprs.push(&**r as *const Expr);
                    flow_edges.push((a, b));
                }
            }
            ExprKind::Mutate(m, pl, _)
                if matches!(m, Mutation::Fill | Mutation::Swap | Mutation::Reverse)
                    && is_1d(&pl.ty) =>
            {
                allowed_places.push(&**pl as *const Place);
            }
            ExprKind::Call(fid, args) => {
                let f = &program.fns[*fid];
                for (slot, arg) in args.iter().enumerate() {
                    if slot >= f.params.len() || !is_1d(&f.locals[slot].ty) {
                        continue;
                    }
                    let dst = ElemLoc::Local(ScopeId::Fn(*fid), slot);
                    match arg {
                        Arg::Value(v) => {
                            let mut cur = v;
                            while let ExprKind::Do(b) = &cur.kind {
                                allowed_exprs.push(cur as *const Expr);
                                if let Some(Stmt::Give(x)) = b.stmts.last() {
                                    cur = x;
                                } else {
                                    break;
                                }
                            }
                            if let Some(src) = e_loc(scope, cur) {
                                allowed_exprs.push(cur as *const Expr);
                                flow_edges.push((dst, src));
                            } else {
                                non_flow_dst.push(dst);
                            }
                        }
                        Arg::Mut(pl) => {
                            if let Some(src) = p_loc(scope, pl) {
                                allowed_places.push(pl as *const Place);
                                flow_edges.push((dst, src));
                            } else {
                                non_flow_dst.push(dst);
                            }
                        }
                    }
                }
            }
            _ => {}
        });
    }

    for dst in non_flow_dst {
        del(&mut fixed, dst);
    }
    // Any unallowed Local/Global expression or place occurrence bans its ElemLoc.
    let check_place_root = |scope: ScopeId, pl: &Place, fixed: &mut Vec<(ElemLoc, u8)>| {
        let mut cur = pl;
        loop {
            match &cur.kind {
                PlaceKind::Index(base, idx) if idx.len() == 1 && is_1d(&base.ty) => {
                    if p_loc(scope, base).is_some() {
                        return;
                    }
                    cur = base;
                }
                PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    cur = base;
                }
                PlaceKind::Local(_) | PlaceKind::Global(_) => {
                    if !allowed_places.contains(&(cur as *const Place)) {
                        if let Some(l) = p_loc(scope, cur) {
                            del(fixed, l);
                        }
                    }
                    return;
                }
            }
        }
    };
    for &(scope, body) in &bodies {
        crate::walk::deep_exprs(body, &mut |e| {
            if matches!(e.kind, ExprKind::Local(_) | ExprKind::Global(_))
                && !allowed_exprs.contains(&(e as *const Expr))
            {
                if let Some(l) = e_loc(scope, e) {
                    del(&mut fixed, l);
                }
            }
            match &e.kind {
                ExprKind::Mutate(_, pl, _) | ExprKind::Sort(pl, _, _) => {
                    check_place_root(scope, pl, &mut fixed);
                }
                ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                    for a in args {
                        if let cpl_ir::Arg::Mut(pl) = a {
                            check_place_root(scope, pl, &mut fixed);
                        }
                    }
                }
                _ => {}
            }
        });
        crate::walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::Assign(pl, _, _) = stmt {
                check_place_root(scope, pl, &mut fixed);
            }
        });
    }
    // Propagate bans across flow_edges (Let / Assign / Cmp pairs).
    loop {
        let mut changed = false;
        for &(a, b) in &flow_edges {
            if get(&fixed, a) != get(&fixed, b) {
                changed |= del(&mut fixed, a);
                changed |= del(&mut fixed, b);
            }
        }
        if !changed {
            break;
        }
    }
    fixed
}
