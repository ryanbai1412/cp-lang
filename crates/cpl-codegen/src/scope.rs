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

fn is_1d_int_seq(ty: &Ty) -> bool {
    match ty {
        Ty::Array { dims, elem } => dims.len() == 1 && matches!(elem.as_ref(), Ty::Int),
        Ty::List(elem) | Ty::ListView(elem) => matches!(elem.as_ref(), Ty::Int),
        _ => false,
    }
}

/// 1D ranged-`Int` arrays, lists, and list views whose elements can be stored as
/// 32-bit integers (`uint32_t` / `int32_t`) consistently across every value flow.
pub(crate) fn narrow_elems(
    program: &Program,
    rows: &BTreeMap<(usize, usize), u32>,
) -> BTreeMap<cpl_ir::ElemLoc, cpl_ir::IntWidth> {
    use cpl_ir::{Arg, Builtin, ElemLoc, IntWidth, Iter, Mutation, ScopeId};

    let mut narrow = program.narrow_elems.clone();
    for (id, def) in program.structs.iter().enumerate() {
        for (&field, &width) in &def.narrow_ints {
            if rows.contains_key(&(id, field)) {
                continue;
            }
            if def.fields.get(field).is_some_and(|(_, t)| is_1d_int_seq(t)) {
                narrow.insert(ElemLoc::Field(id, field), width);
            }
        }
    }
    // Propagate outer widths into closure captures so closures capturing a narrowed
    // array can stay narrow if all uses inside the closure are compatible.
    let mut cap_links: Vec<(ElemLoc, ElemLoc)> = Vec::new();
    let collect_caps = |scope: ScopeId, body: &cpl_ir::Block, links: &mut Vec<(ElemLoc, ElemLoc)>| {
        crate::walk::deep_exprs(body, &mut |e| {
            if let ExprKind::Closure(cid) = &e.kind {
                let c = &program.closures[*cid];
                for cap in &c.captures {
                    if c.locals.get(cap.inner).is_some_and(|l| is_1d_int_seq(&l.ty)) {
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
        if let Some(&w) = narrow.get(&outer) {
            narrow.entry(inner).or_insert(w);
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

    fn place_loc(scope: ScopeId, place: &Place) -> Option<ElemLoc> {
        match &place.kind {
            PlaceKind::Global(g) => Some(ElemLoc::Global(*g)),
            PlaceKind::Local(s) => Some(ElemLoc::Local(scope, *s)),
            PlaceKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => Some(ElemLoc::Field(id, *f)),
                _ => None,
            },
            _ => None,
        }
    }

    fn expr_loc(scope: ScopeId, e: &Expr) -> Option<ElemLoc> {
        match &e.kind {
            ExprKind::Global(g) => Some(ElemLoc::Global(*g)),
            ExprKind::Local(s) => Some(ElemLoc::Local(scope, *s)),
            ExprKind::Field(base, f) => match base.ty {
                Ty::Struct(id) => Some(ElemLoc::Field(id, *f)),
                _ => None,
            },
            ExprKind::Call(fid, _) => Some(ElemLoc::FnRet(*fid)),
            ExprKind::Builtin(Builtin::ListSlice | Builtin::ToList | Builtin::Copy, args) => {
                args.first().and_then(|a| expr_loc(scope, a))
            }
            ExprKind::Do(blk) => match blk.stmts.last() {
                Some(Stmt::Give(inner)) => expr_loc(scope, inner),
                _ => None,
            },
            _ => None,
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

    fn widen_loc(loc: ElemLoc, narrow: &mut BTreeMap<ElemLoc, IntWidth>, changed: &mut bool) {
        if narrow.remove(&loc).is_some() {
            *changed = true;
        }
    }

    fn widen_expr(
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        if let Some(loc) = expr_loc(scope, e) {
            widen_loc(loc, narrow, changed);
        }
    }

    fn check_flow(
        scope: ScopeId,
        src: &Expr,
        dst: ElemLoc,
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        if is_fresh_adaptable(src) {
            return;
        }
        if let Some(src_loc) = expr_loc(scope, src) {
            if narrow.get(&src_loc) != narrow.get(&dst) {
                widen_loc(src_loc, narrow, changed);
                widen_loc(dst, narrow, changed);
            }
        } else {
            widen_loc(dst, narrow, changed);
        }
    }

    fn check_expr(
        program: &Program,
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        match &e.kind {
            ExprKind::Index(base, indices) if indices.len() == 1 && is_1d_int_seq(&base.ty) => {
                // Element read on a narrowed 1D sequence is safe (widened to `ll` on read).
                check_nested_base(program, scope, base, narrow, changed);
                for idx in indices {
                    check_val(program, scope, idx, narrow, changed);
                }
            }
            ExprKind::Builtin(Builtin::Len, args) => {
                for a in args {
                    if is_1d_int_seq(&a.ty) {
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
                if safe_mut && is_1d_int_seq(&place.ty) {
                    check_place_nested(program, scope, place, narrow, changed);
                    for a in args {
                        check_val(program, scope, a, narrow, changed);
                    }
                } else if matches!(m, Mutation::Extend) && is_1d_int_seq(&place.ty) && args.len() == 1 {
                    check_place_nested(program, scope, place, narrow, changed);
                    if let Some(dst) = place_loc(scope, place) {
                        check_flow(scope, &args[0], dst, narrow, changed);
                        check_nested_base(program, scope, &args[0], narrow, changed);
                    } else {
                        check_val(program, scope, &args[0], narrow, changed);
                    }
                } else {
                    if is_1d_int_seq(&place.ty) {
                        if let Some(loc) = place_loc(scope, place) {
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
                if matches!(by, cpl_ir::SortBy::Natural) && is_1d_int_seq(&place.ty) {
                    check_place_nested(program, scope, place, narrow, changed);
                } else {
                    if is_1d_int_seq(&place.ty) {
                        if let Some(loc) = place_loc(scope, place) {
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
                        if is_1d_int_seq(&val.ty) {
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
                            if p_ty.is_some_and(is_1d_int_seq) {
                                check_flow(scope, v, p_loc, narrow, changed);
                                check_nested_base(program, scope, v, narrow, changed);
                            } else {
                                check_val(program, scope, v, narrow, changed);
                            }
                        }
                        Arg::Mut(p) => {
                            if p_ty.is_some_and(is_1d_int_seq) {
                                if let Some(src_loc) = place_loc(scope, p) {
                                    if narrow.get(&src_loc) != narrow.get(&p_loc) {
                                        widen_loc(src_loc, narrow, changed);
                                        widen_loc(p_loc, narrow, changed);
                                    }
                                } else {
                                    widen_loc(p_loc, narrow, changed);
                                }
                                check_place_nested(program, scope, p, narrow, changed);
                            } else {
                                if is_1d_int_seq(&p.ty) {
                                    if let Some(loc) = place_loc(scope, p) {
                                        widen_loc(loc, narrow, changed);
                                    }
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
                            if is_1d_int_seq(&p.ty) {
                                if let Some(loc) = place_loc(scope, p) {
                                    widen_loc(loc, narrow, changed);
                                }
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
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        if is_1d_int_seq(&e.ty) {
            widen_expr(scope, e, narrow, changed);
        }
        check_expr(program, scope, e, narrow, changed);
    }

    fn check_nested_base(
        program: &Program,
        scope: ScopeId,
        e: &Expr,
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        match &e.kind {
            ExprKind::Global(_) | ExprKind::Local(_) => {}
            ExprKind::Field(base, _) => check_val(program, scope, base, narrow, changed),
            ExprKind::Builtin(Builtin::ListSlice | Builtin::ToList | Builtin::Copy, args) => {
                if let Some((first, rest)) = args.split_first() {
                    check_nested_base(program, scope, first, narrow, changed);
                    for a in rest {
                        check_val(program, scope, a, narrow, changed);
                    }
                }
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
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
    ) {
        match &p.kind {
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                check_place_nested(program, scope, base, narrow, changed)
            }
            PlaceKind::Index(base, indices) => {
                if indices.len() == 1 && is_1d_int_seq(&base.ty) {
                    check_place_nested(program, scope, base, narrow, changed);
                } else {
                    if is_1d_int_seq(&base.ty) {
                        if let Some(loc) = place_loc(scope, base) {
                            widen_loc(loc, narrow, changed);
                        }
                    }
                    check_place_nested(program, scope, base, narrow, changed);
                }
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
        narrow: &mut BTreeMap<ElemLoc, IntWidth>,
        changed: &mut bool,
        allow_narrow_give: bool,
    ) {
        let len = blk.stmts.len();
        for (idx, stmt) in blk.stmts.iter().enumerate() {
            let is_last = idx + 1 == len;
            match stmt {
                Stmt::Let(slot, Some(val)) => {
                    if is_1d_int_seq(&val.ty) {
                        check_flow(scope, val, ElemLoc::Local(scope, *slot), narrow, changed);
                        check_nested_base(program, scope, val, narrow, changed);
                    } else {
                        check_val(program, scope, val, narrow, changed);
                    }
                }
                Stmt::Let(_, None) => {}
                Stmt::Assign(place, op, val) => {
                    if is_1d_int_seq(&place.ty) {
                        if *op != cpl_ir::AssignOp::Set {
                            if let Some(loc) = place_loc(scope, place) {
                                widen_loc(loc, narrow, changed);
                            }
                            check_place_nested(program, scope, place, narrow, changed);
                            check_val(program, scope, val, narrow, changed);
                        } else if let Some(dst) = place_loc(scope, place) {
                            check_flow(scope, val, dst, narrow, changed);
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
                    if is_1d_int_seq(&val.ty) {
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
                    if allow_narrow_give && is_last && is_1d_int_seq(&val.ty) {
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
                Stmt::For(_, _, iter, body) => {
                    match iter {
                        Iter::Seq(e, _) if is_1d_int_seq(&e.ty) => {
                            check_nested_base(program, scope, e, narrow, changed);
                        }
                        Iter::Seq(e, _)
                        | Iter::RangeVal(e, _)
                        | Iter::Subsets(e)
                        | Iter::Members(e) => {
                            check_val(program, scope, e, narrow, changed);
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
