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
                    Iter::TreeRange { container, lo, hi, .. } => {
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

/// The struct fields that hold a row of a fixed small count of cells and
/// can be kept inside the struct: `(struct, field)` to the count. A field
/// that is somewhere sorted, grown, filled, updated as a whole or handed
/// to a call as `mut` stays a `std::vector`, which those work on.
pub(crate) fn rows(program: &Program, facts: &crate::ranges::Facts<'_>) -> BTreeMap<(usize, usize), u32> {
    let mut rows = BTreeMap::new();
    for (id, def) in program.structs.iter().enumerate() {
        for (field, cells) in &def.fixed {
            if matches!(&def.fields[*field].1, Ty::Array { dims, .. } if dims.len() == 1) {
                rows.insert((id, *field), *cells);
            }
        }
        // A Bitset<D> over a domain with a small static upper bound (<= 4096 bits)
        // keeps its U64 words inline in a `cpl::Row<U64, W>` instead of heap-allocating.
        if def.name.starts_with("Bitset")
            && def.fields.len() == 1
            && def.fields[0].0 == "w"
            && matches!(&def.fields[0].1, Ty::List(elem) if elem.as_ref() == &Ty::U64)
        {
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
                        rows.insert((id, 0), words);
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
                let Stmt::Return(Some(value)) = stmt else { return };
                let ExprKind::Tuple(items) = &value.kind else { return };
                if matches!(value.ty, Ty::Unit | Ty::Error | Ty::Map(..) | Ty::HashMap(..)) {
                    return;
                }
                for (idx, item) in items.iter().enumerate() {
                    let ExprKind::Local(slot) = item.kind else { continue };
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
            sunk
        })
        .collect()
}

/// Ranged `Int` struct fields that can be stored as 32-bit integers (`uint32_t`/`int32_t`).
/// Fields passed by `mut` reference or updated with a compound operator stay `ll`.
pub(crate) fn narrow_ints(program: &Program) -> BTreeMap<(usize, usize), cpl_ir::IntWidth> {
    let mut narrow = BTreeMap::new();
    for (id, def) in program.structs.iter().enumerate() {
        for (field, width) in &def.narrow_ints {
            if matches!(def.fields.get(*field).map(|(_, t)| t), Some(Ty::Int)) {
                narrow.insert((id, *field), *width);
            }
        }
    }
    if narrow.is_empty() {
        return narrow;
    }
    fn widen(place: &Place, narrow: &mut BTreeMap<(usize, usize), cpl_ir::IntWidth>) {
        if let PlaceKind::Field(base, field) = &place.kind {
            if let Ty::Struct(id) = base.ty {
                narrow.remove(&(id, *field));
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
                    widen(place, &mut narrow);
                }
            }
        });
        crate::walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _) => widen(place, &mut narrow),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                for arg in args {
                    if let cpl_ir::Arg::Mut(place) = arg {
                        widen(place, &mut narrow);
                    }
                }
            }
            _ => {}
        });
    }
    narrow
}

fn is_narrowable_seq(ty: &Ty) -> bool {
    matches!(ty, Ty::Array { .. } | Ty::List(_) | Ty::ListView(_) | Ty::MutView(_))
}

fn peel_layout(layout: Option<&cpl_ir::NarrowLayout>, depth: usize) -> Option<&cpl_ir::NarrowLayout> {
    let mut cur = layout?;
    for _ in 0..depth {
        match cur {
            cpl_ir::NarrowLayout::Seq(inner) => cur = inner,
            _ => return None,
        }
    }
    Some(cur)
}

/// Range-typed arrays, grids, nested arrays, lists, and all-ranged tuple elements
/// whose elements can be stored as 32-bit integers consistently across every flow.
pub(crate) fn narrow_elems(
    program: &Program,
    rows: &BTreeMap<(usize, usize), u32>,
) -> BTreeMap<cpl_ir::ElemLoc, cpl_ir::NarrowLayout> {
    use cpl_ir::{Arg, Builtin, ElemLoc, Iter, Mutation, NarrowLayout, ScopeId};

    let mut narrow = program.narrow_elems.clone();
    for (id, def) in program.structs.iter().enumerate() {
        for (&field, &width) in &def.narrow_ints {
            if rows.contains_key(&(id, field)) {
                continue;
            }
            if def
                .fields
                .get(field)
                .is_some_and(|(_, t)| matches!(t, Ty::Array { elem, .. } | Ty::List(elem) | Ty::ListView(elem) if matches!(elem.as_ref(), Ty::Int)))
            {
                narrow.entry(ElemLoc::Field(id, field)).or_insert(NarrowLayout::Int(width));
            }
        }
        for &(sid, field) in rows.keys() {
            if sid == id {
                narrow.remove(&ElemLoc::Field(id, field));
            }
        }
    }
    // Propagate outer layouts into closure captures so closures capturing a narrowed
    // container can stay narrow if all uses inside the closure are compatible.
    let mut cap_links: Vec<(ElemLoc, ElemLoc)> = Vec::new();
    let collect_caps = |scope: ScopeId, body: &cpl_ir::Block, links: &mut Vec<(ElemLoc, ElemLoc)>| {
        crate::walk::deep_exprs(body, &mut |e| {
            if let ExprKind::Closure(cid) = &e.kind {
                let c = &program.closures[*cid];
                for cap in &c.captures {
                    if c.locals.get(cap.inner).is_some_and(|l| is_narrowable_seq(&l.ty)) {
                        let outer = if cap.is_global {
                            ElemLoc::Global(cap.outer)
                        } else {
                            ElemLoc::Local(scope, cap.outer)
                        };
                        let inner = ElemLoc::Local(ScopeId::Closure(*cid), cap.inner);
                        links.push((outer, inner));
                    }
                }
            }
        });
    };
    collect_caps(ScopeId::Main, &program.setup, &mut cap_links);
    collect_caps(ScopeId::Main, &program.main, &mut cap_links);
    for (fid, f) in program.fns.iter().enumerate() {
        collect_caps(ScopeId::Fn(fid), &f.body, &mut cap_links);
    }
    for (cid, c) in program.closures.iter().enumerate() {
        collect_caps(ScopeId::Closure(cid), &c.body, &mut cap_links);
    }
    for &(outer, inner) in &cap_links {
        if let Some(l) = narrow.get(&outer).cloned() {
            narrow.entry(inner).or_insert(l);
        }
    }
    // Grader/judge functions keep wide `ll` signatures to match `cpl::Judge<T>`.
    for &fid in program.judge_fns.iter().chain(program.entry_fns.iter()) {
        narrow.remove(&ElemLoc::FnRet(fid));
        for i in 0..program.fns[fid].params.len() {
            narrow.remove(&ElemLoc::Local(ScopeId::Fn(fid), i));
        }
    }
    if narrow.is_empty() {
        return narrow;
    }

    fn place_any_seq_root(scope: ScopeId, place: &Place) -> Option<ElemLoc> {
        match &place.kind {
            PlaceKind::Global(g) if is_narrowable_seq(&place.ty) => Some(ElemLoc::Global(*g)),
            PlaceKind::Local(s) if is_narrowable_seq(&place.ty) => Some(ElemLoc::Local(scope, *s)),
            PlaceKind::Field(base, f) => {
                if is_narrowable_seq(&place.ty) {
                    if let Ty::Struct(id) = base.ty {
                        return Some(ElemLoc::Field(id, *f));
                    }
                }
                place_any_seq_root(scope, base)
            }
            PlaceKind::Index(base, _) | PlaceKind::Some(base) => place_any_seq_root(scope, base),
            _ => None,
        }
    }

    fn place_root(scope: ScopeId, place: &Place) -> Option<(ElemLoc, usize)> {
        match &place.kind {
            PlaceKind::Global(g) => Some((ElemLoc::Global(*g), 0)),
            PlaceKind::Local(s) => Some((ElemLoc::Local(scope, *s), 0)),
            PlaceKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => Some((ElemLoc::Field(id, *f), 0)),
                _ => None,
            },
            PlaceKind::Index(base, _) => {
                let (loc, d) = place_root(scope, base)?;
                Some((loc, d + 1))
            }
            _ => None,
        }
    }

    fn expr_root(scope: ScopeId, e: &Expr) -> Option<(ElemLoc, usize)> {
        match &e.kind {
            ExprKind::Global(g) => Some((ElemLoc::Global(*g), 0)),
            ExprKind::Local(s) => Some((ElemLoc::Local(scope, *s), 0)),
            ExprKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => Some((ElemLoc::Field(id, *f), 0)),
                _ => None,
            },
            ExprKind::Call(fid, _) => Some((ElemLoc::FnRet(*fid), 0)),
            ExprKind::Index(base, _) => {
                let (loc, d) = expr_root(scope, base)?;
                Some((loc, d + 1))
            }
            ExprKind::Builtin(Builtin::ListSlice | Builtin::ToList | Builtin::Copy, args) => {
                args.first().and_then(|a| expr_root(scope, a))
            }
            ExprKind::MutSlice(place, _, _) => place_root(scope, place),
            ExprKind::Do(blk) => match blk.stmts.last() {
                Some(Stmt::Give(inner)) => expr_root(scope, inner),
                _ => None,
            },
            _ => None,
        }
    }

    // Seed `for row in nested_arr` loop variables from their sequence's inner layout.
    let mut for_links: Vec<(ElemLoc, usize, ElemLoc)> = Vec::new();
    let collect_fors = |scope: ScopeId, body: &cpl_ir::Block, links: &mut Vec<(ElemLoc, usize, ElemLoc)>| {
        crate::walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::For(_, slot, Iter::Seq(e, _), _) = stmt {
                if is_narrowable_seq(&e.ty) {
                    if let Some((src_loc, depth)) = expr_root(scope, e) {
                        links.push((src_loc, depth + 1, ElemLoc::Local(scope, *slot)));
                    }
                }
            }
        });
    };
    collect_fors(ScopeId::Main, &program.setup, &mut for_links);
    collect_fors(ScopeId::Main, &program.main, &mut for_links);
    for (fid, f) in program.fns.iter().enumerate() {
        collect_fors(ScopeId::Fn(fid), &f.body, &mut for_links);
    }
    for (cid, c) in program.closures.iter().enumerate() {
        collect_fors(ScopeId::Closure(cid), &c.body, &mut for_links);
    }
    for &(src_loc, depth, dst_loc) in &for_links {
        if let Some(inner) = peel_layout(narrow.get(&src_loc), depth).cloned() {
            narrow.entry(dst_loc).or_insert(inner);
        }
    }

    fn is_fresh_adaptable(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Fill(..) => true,
            ExprKind::Tuple(_) if matches!(e.ty, Ty::List(_)) => true,
            ExprKind::Builtin(Builtin::ListFilled, _) => true,
            ExprKind::Read(spec) if matches!(spec.as_ref(), cpl_ir::Spec::Array { .. }) => true,
            _ => false,
        }
    }

    fn widen_loc(loc: ElemLoc, narrow: &mut BTreeMap<ElemLoc, NarrowLayout>, changed: &mut bool) {
        if narrow.remove(&loc).is_some() {
            *changed = true;
        }
    }

    fn widen_expr(
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        if let Some((loc, _)) = expr_root(scope, e) {
            widen_loc(loc, narrow, changed);
        }
    }

    fn check_flow_at(
        scope: ScopeId,
        src: &Expr,
        dst: ElemLoc,
        dst_depth: usize,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        if is_fresh_adaptable(src) {
            if let ExprKind::Fill(_, Some(inner)) = &src.kind {
                if is_narrowable_seq(&inner.ty) {
                    check_flow_at(scope, inner, dst, dst_depth + 1, narrow, changed);
                }
            }
            if let ExprKind::Tuple(items) = &src.kind {
                if matches!(src.ty, Ty::List(_)) {
                    for item in items {
                        if is_narrowable_seq(&item.ty) {
                            check_flow_at(scope, item, dst, dst_depth + 1, narrow, changed);
                        }
                    }
                }
            }
            return;
        }
        if let Some((src_loc, src_depth)) = expr_root(scope, src) {
            let src_l = peel_layout(narrow.get(&src_loc), src_depth);
            let dst_l = peel_layout(narrow.get(&dst), dst_depth);
            if src_l != dst_l {
                widen_loc(src_loc, narrow, changed);
                widen_loc(dst, narrow, changed);
            }
        } else {
            widen_loc(dst, narrow, changed);
        }
    }

    fn check_flow(
        scope: ScopeId,
        src: &Expr,
        dst: ElemLoc,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        check_flow_at(scope, src, dst, 0, narrow, changed);
    }

    fn check_expr(
        program: &Program,
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        match &e.kind {
            ExprKind::Index(base, indices) if is_narrowable_seq(&base.ty) => {
                // Element read on a narrowed sequence is safe (leaf Int/Tuple widens on read;
                // nested sequence stays narrow only if its consumer is also compatible).
                check_nested_base(program, scope, base, narrow, changed);
                for idx in indices {
                    check_val(program, scope, idx, narrow, changed);
                }
            }
            ExprKind::Builtin(Builtin::Len, args) => {
                for a in args {
                    if is_narrowable_seq(&a.ty) {
                        check_nested_base(program, scope, a, narrow, changed);
                    } else {
                        check_val(program, scope, a, narrow, changed);
                    }
                }
            }
            ExprKind::Print(parts, end) => {
                for p in parts {
                    check_val(program, scope, &p.value, narrow, changed);
                }
                if let Some(e) = end {
                    check_val(program, scope, e, narrow, changed);
                }
            }
            ExprKind::Debug(_, parts) => {
                for p in parts {
                    check_val(program, scope, &p.value, narrow, changed);
                }
            }
            ExprKind::Builtin(Builtin::ListSlice | Builtin::ToList | Builtin::Copy, args) => {
                // When reached via check_val (not as a safe consumer), widen the base.
                for a in args {
                    check_val(program, scope, a, narrow, changed);
                }
            }
            ExprKind::MutSlice(place, lo, hi) => {
                if let Some((loc, _)) = place_root(scope, place) {
                    if narrow.remove(&loc).is_some() {
                        *changed = true;
                    }
                }
                check_val(program, scope, lo, narrow, changed);
                check_val(program, scope, hi, narrow, changed);
            }
            ExprKind::Mutate(m, place, args) => {
                let safe_mut = matches!(
                    m,
                    Mutation::Push
                        | Mutation::Pop
                        | Mutation::Clear
                        | Mutation::Fill
                        | Mutation::Resize
                        | Mutation::Reverse
                        | Mutation::Truncate
                        | Mutation::Swap
                        | Mutation::Remove
                );
                if safe_mut && is_narrowable_seq(&place.ty) {
                    if matches!(m, Mutation::Pop) {
                        if let Some((loc, depth)) = place_root(scope, place) {
                            if matches!(peel_layout(narrow.get(&loc), depth), Some(NarrowLayout::Seq(_))) {
                                widen_loc(loc, narrow, changed);
                            }
                        }
                    }
                    check_place_nested(program, scope, place, narrow, changed);
                    for a in args {
                        if matches!(m, Mutation::Push | Mutation::Fill | Mutation::Resize)
                            && is_narrowable_seq(&a.ty)
                        {
                            if let Some((loc, depth)) = place_root(scope, place) {
                                check_flow_at(scope, a, loc, depth + 1, narrow, changed);
                                check_nested_base(program, scope, a, narrow, changed);
                            } else {
                                check_val(program, scope, a, narrow, changed);
                            }
                        } else {
                            check_val(program, scope, a, narrow, changed);
                        }
                    }
                } else if matches!(m, Mutation::Extend) && is_narrowable_seq(&place.ty) && args.len() == 1 {
                    check_place_nested(program, scope, place, narrow, changed);
                    if let Some((dst, depth)) = place_root(scope, place) {
                        check_flow_at(scope, &args[0], dst, depth, narrow, changed);
                        check_nested_base(program, scope, &args[0], narrow, changed);
                    } else {
                        check_val(program, scope, &args[0], narrow, changed);
                    }
                } else {
                    if is_narrowable_seq(&place.ty) {
                        if let Some((loc, _)) = place_root(scope, place) {
                            widen_loc(loc, narrow, changed);
                        }
                    }
                    check_place_nested(program, scope, place, narrow, changed);
                    for a in args {
                        check_val(program, scope, a, narrow, changed);
                    }
                }
            }
            ExprKind::Sort(place, by, _) => {
                if matches!(by, cpl_ir::SortBy::Natural) && is_narrowable_seq(&place.ty) {
                    check_place_nested(program, scope, place, narrow, changed);
                } else {
                    if is_narrowable_seq(&place.ty) {
                        if let Some((loc, _)) = place_root(scope, place) {
                            widen_loc(loc, narrow, changed);
                        }
                    }
                    check_place_nested(program, scope, place, narrow, changed);
                    match by {
                        cpl_ir::SortBy::Natural => {}
                        cpl_ir::SortBy::Key(_, k) => check_val(program, scope, k, narrow, changed),
                        cpl_ir::SortBy::Fn { func, .. } => {
                            check_val(program, scope, func, narrow, changed)
                        }
                    }
                }
            }
            ExprKind::Tuple(values) => {
                if let Ty::Struct(sid) = e.ty {
                    for (idx, val) in values.iter().enumerate() {
                        if is_narrowable_seq(&val.ty) {
                            check_flow(scope, val, ElemLoc::Field(sid, idx), narrow, changed);
                            check_nested_base(program, scope, val, narrow, changed);
                        } else {
                            check_val(program, scope, val, narrow, changed);
                        }
                    }
                } else {
                    for val in values {
                        check_val(program, scope, val, narrow, changed);
                    }
                }
            }
            ExprKind::Call(fid, args) => {
                let f = &program.fns[*fid];
                for (i, arg) in args.iter().enumerate() {
                    let p_ty = f.locals.get(i).map(|l| &l.ty);
                    let p_loc = ElemLoc::Local(ScopeId::Fn(*fid), i);
                    match arg {
                        Arg::Value(v) => {
                            if p_ty.is_some_and(is_narrowable_seq) {
                                check_flow(scope, v, p_loc, narrow, changed);
                                check_nested_base(program, scope, v, narrow, changed);
                            } else {
                                check_val(program, scope, v, narrow, changed);
                            }
                        }
                        Arg::Mut(p) => {
                            if p_ty.is_some_and(is_narrowable_seq) {
                                if let Some((src_loc, src_depth)) = place_root(scope, p) {
                                    if peel_layout(narrow.get(&src_loc), src_depth) != narrow.get(&p_loc) {
                                        widen_loc(src_loc, narrow, changed);
                                        widen_loc(p_loc, narrow, changed);
                                    }
                                } else {
                                    widen_loc(p_loc, narrow, changed);
                                }
                                check_place_nested(program, scope, p, narrow, changed);
                            } else {
                                if let Some(loc) = place_any_seq_root(scope, p) {
                                    widen_loc(loc, narrow, changed);
                                }
                                check_place_nested(program, scope, p, narrow, changed);
                            }
                        }
                    }
                }
            }
            ExprKind::CallExpr(callee, args) => {
                check_val(program, scope, callee, narrow, changed);
                for arg in args {
                    match arg {
                        Arg::Value(v) => check_val(program, scope, v, narrow, changed),
                        Arg::Mut(p) => {
                            if let Some(loc) = place_any_seq_root(scope, p) {
                                widen_loc(loc, narrow, changed);
                            }
                            check_place_nested(program, scope, p, narrow, changed);
                        }
                    }
                }
            }
            ExprKind::Do(blk) => {
                check_block(program, scope, blk, narrow, changed, false);
            }
            ExprKind::If(c, t, f) => {
                check_val(program, scope, c, narrow, changed);
                check_val(program, scope, t, narrow, changed);
                check_val(program, scope, f, narrow, changed);
            }
            ExprKind::Unary(_, a)
            | ExprKind::Cast(_, a)
            | ExprKind::Some(a)
            | ExprKind::IsSome(a)
            | ExprKind::Unwrap(a, _)
            | ExprKind::RangeCheck { value: a, .. } => {
                check_val(program, scope, a, narrow, changed);
                if let ExprKind::RangeCheck { lo, hi, .. } = &e.kind {
                    if let Some(l) = lo {
                        check_val(program, scope, l, narrow, changed);
                    }
                    if let Some(h) = hi {
                        check_val(program, scope, h, narrow, changed);
                    }
                }
            }
            ExprKind::Binary(_, a, b) | ExprKind::Cmp(_, a, b) => {
                check_val(program, scope, a, narrow, changed);
                check_val(program, scope, b, narrow, changed);
            }
            ExprKind::Field(base, _) => {
                check_val(program, scope, base, narrow, changed);
            }
            ExprKind::Index(base, indices) => {
                check_val(program, scope, base, narrow, changed);
                for idx in indices {
                    check_val(program, scope, idx, narrow, changed);
                }
            }
            ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
                for item in items {
                    check_val(program, scope, item, narrow, changed);
                }
            }
            ExprKind::Fill(dims, val) => {
                for d in dims {
                    if let cpl_ir::DimSpec::Int(n) | cpl_ir::DimSpec::IntFrom(_, n) = d {
                        check_val(program, scope, n, narrow, changed);
                    }
                }
                if let Some(val) = val {
                    check_val(program, scope, val, narrow, changed);
                }
            }
            _ => {}
        }
    }

    fn check_val(
        program: &Program,
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        if is_narrowable_seq(&e.ty) {
            widen_expr(scope, e, narrow, changed);
        }
        check_expr(program, scope, e, narrow, changed);
    }

    fn check_nested_base(
        program: &Program,
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        match &e.kind {
            ExprKind::Global(_) | ExprKind::Local(_) => {}
            ExprKind::Field(base, _) => check_val(program, scope, base, narrow, changed),
            ExprKind::Index(base, indices) if is_narrowable_seq(&base.ty) => {
                check_nested_base(program, scope, base, narrow, changed);
                for idx in indices {
                    check_val(program, scope, idx, narrow, changed);
                }
            }
            ExprKind::Fill(dims, val) => {
                for d in dims {
                    if let cpl_ir::DimSpec::Int(n) | cpl_ir::DimSpec::IntFrom(_, n) = d {
                        check_val(program, scope, n, narrow, changed);
                    }
                }
                if let Some(v) = val {
                    if is_narrowable_seq(&v.ty) {
                        check_nested_base(program, scope, v, narrow, changed);
                    } else {
                        check_val(program, scope, v, narrow, changed);
                    }
                }
            }
            ExprKind::Tuple(items) if matches!(e.ty, Ty::List(_)) => {
                for item in items {
                    if is_narrowable_seq(&item.ty) {
                        check_nested_base(program, scope, item, narrow, changed);
                    } else {
                        check_val(program, scope, item, narrow, changed);
                    }
                }
            }
            ExprKind::Builtin(Builtin::ListSlice | Builtin::ToList | Builtin::Copy, args) => {
                if let Some((first, rest)) = args.split_first() {
                    check_nested_base(program, scope, first, narrow, changed);
                    for a in rest {
                        check_val(program, scope, a, narrow, changed);
                    }
                }
            }
            ExprKind::MutSlice(place, lo, hi) => {
                if let Some((loc, _)) = place_root(scope, place) {
                    if narrow.remove(&loc).is_some() {
                        *changed = true;
                    }
                }
                check_val(program, scope, lo, narrow, changed);
                check_val(program, scope, hi, narrow, changed);
            }
            ExprKind::Do(blk) => {
                check_block(program, scope, blk, narrow, changed, true);
            }
            _ => check_expr(program, scope, e, narrow, changed),
        }
    }

    fn check_place_nested(
        program: &Program,
        scope: ScopeId,
        p: &Place,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
    ) {
        match &p.kind {
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                check_place_nested(program, scope, base, narrow, changed)
            }
            PlaceKind::Index(base, indices) => {
                check_place_nested(program, scope, base, narrow, changed);
                for idx in indices {
                    check_val(program, scope, idx, narrow, changed);
                }
            }
            PlaceKind::Global(_) | PlaceKind::Local(_) => {}
        }
    }

    fn check_block(
        program: &Program,
        scope: ScopeId,
        blk: &cpl_ir::Block,
        narrow: &mut BTreeMap<ElemLoc, NarrowLayout>,
        changed: &mut bool,
        allow_narrow_give: bool,
    ) {
        let len = blk.stmts.len();
        for (idx, stmt) in blk.stmts.iter().enumerate() {
            let is_last = idx + 1 == len;
            match stmt {
                Stmt::Let(slot, Some(val)) => {
                    if is_narrowable_seq(&val.ty) {
                        check_flow(scope, val, ElemLoc::Local(scope, *slot), narrow, changed);
                        check_nested_base(program, scope, val, narrow, changed);
                    } else {
                        check_val(program, scope, val, narrow, changed);
                    }
                }
                Stmt::Let(_, None) => {}
                Stmt::Assign(place, op, val) => {
                    if is_narrowable_seq(&place.ty) {
                        if *op != cpl_ir::AssignOp::Set {
                            if let Some((loc, _)) = place_root(scope, place) {
                                widen_loc(loc, narrow, changed);
                            }
                            check_place_nested(program, scope, place, narrow, changed);
                            check_val(program, scope, val, narrow, changed);
                        } else if let Some((dst, depth)) = place_root(scope, place) {
                            check_flow_at(scope, val, dst, depth, narrow, changed);
                            check_place_nested(program, scope, place, narrow, changed);
                            check_nested_base(program, scope, val, narrow, changed);
                        } else {
                            check_place_nested(program, scope, place, narrow, changed);
                            check_val(program, scope, val, narrow, changed);
                        }
                    } else {
                        check_place_nested(program, scope, place, narrow, changed);
                        check_val(program, scope, val, narrow, changed);
                    }
                }
                Stmt::Return(Some(val)) => {
                    if is_narrowable_seq(&val.ty) {
                        if let ScopeId::Fn(fid) = scope {
                            check_flow(scope, val, ElemLoc::FnRet(fid), narrow, changed);
                            check_nested_base(program, scope, val, narrow, changed);
                        } else {
                            check_val(program, scope, val, narrow, changed);
                        }
                    } else {
                        check_val(program, scope, val, narrow, changed);
                    }
                }
                Stmt::Give(val) => {
                    if allow_narrow_give && is_last && is_narrowable_seq(&val.ty) {
                        check_nested_base(program, scope, val, narrow, changed);
                    } else {
                        check_val(program, scope, val, narrow, changed);
                    }
                }
                Stmt::Expr(e) => check_val(program, scope, e, narrow, changed),
                Stmt::Assert { cond, message, .. } => {
                    check_val(program, scope, cond, narrow, changed);
                    if let Some(m) = message {
                        check_val(program, scope, m, narrow, changed);
                    }
                }
                Stmt::If(cond, then, otherwise) => {
                    check_val(program, scope, cond, narrow, changed);
                    check_block(program, scope, then, narrow, changed, false);
                    check_block(program, scope, otherwise, narrow, changed, false);
                }
                Stmt::While(_, cond, body) => {
                    check_val(program, scope, cond, narrow, changed);
                    check_block(program, scope, body, narrow, changed, false);
                }
                Stmt::For(_, slot, iter, body) => {
                    match iter {
                        Iter::Seq(e, _) if is_narrowable_seq(&e.ty) => {
                            if let Some((src_loc, depth)) = expr_root(scope, e) {
                                let inner = peel_layout(narrow.get(&src_loc), depth + 1);
                                let dst_loc = ElemLoc::Local(scope, *slot);
                                if (inner.is_some() || narrow.contains_key(&dst_loc))
                                    && inner != narrow.get(&dst_loc)
                                {
                                    widen_loc(src_loc, narrow, changed);
                                    widen_loc(dst_loc, narrow, changed);
                                }
                            }
                            check_nested_base(program, scope, e, narrow, changed);
                        }
                        Iter::Seq(e, _)
                        | Iter::RangeVal(e, _)
                        | Iter::Subsets(e)
                        | Iter::Members(e) => {
                            check_val(program, scope, e, narrow, changed);
                        }
                        Iter::TreeRange { container, lo, hi, .. } => {
                            check_val(program, scope, container, narrow, changed);
                            if let Some((e, _)) = lo {
                                check_val(program, scope, e, narrow, changed);
                            }
                            if let Some((e, _)) = hi {
                                check_val(program, scope, e, narrow, changed);
                            }
                        }
                        Iter::Range { lo, hi, step, .. } => {
                            check_val(program, scope, lo, narrow, changed);
                            check_val(program, scope, hi, narrow, changed);
                            if let Some(s) = step {
                                check_val(program, scope, s, narrow, changed);
                            }
                        }
                    }
                    check_block(program, scope, body, narrow, changed, false);
                }
                Stmt::DefDomain { lo, len, .. } => {
                    check_val(program, scope, lo, narrow, changed);
                    check_val(program, scope, len, narrow, changed);
                }
                Stmt::DefDomainFromVal { val, .. } => {
                    check_val(program, scope, val, narrow, changed);
                }
                Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
            }
        }
    }

    loop {
        let mut changed = false;
        for &(outer, inner) in &cap_links {
            if narrow.get(&outer) != narrow.get(&inner) {
                widen_loc(outer, &mut narrow, &mut changed);
                widen_loc(inner, &mut narrow, &mut changed);
            }
        }
        check_block(program, ScopeId::Main, &program.setup, &mut narrow, &mut changed, false);
        check_block(program, ScopeId::Main, &program.main, &mut narrow, &mut changed, false);
        for (fid, f) in program.fns.iter().enumerate() {
            check_block(program, ScopeId::Fn(fid), &f.body, &mut narrow, &mut changed, false);
        }
        for (cid, c) in program.closures.iter().enumerate() {
            check_block(program, ScopeId::Closure(cid), &c.body, &mut narrow, &mut changed, false);
        }
        if !changed {
            break;
        }
    }
    narrow
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
        v.iter().position(|(x, _)| *x == l).map(|i| v.swap_remove(i)).is_some()
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
    .chain(program.fns.iter().enumerate().map(|(i, f)| (ScopeId::Fn(i), &f.body)))
    .chain(program.closures.iter().enumerate().map(|(i, c)| (ScopeId::Closure(i), &c.body)))
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
                let ExprKind::Call(fid, args) = &e.kind else { return };
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
                if is_1d(&seq.ty) && matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_)) =>
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
