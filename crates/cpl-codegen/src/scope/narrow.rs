//! Element and field narrowing: which ranged ints, arrays, lists and tuples store as 32-bit across every flow.

use super::*;

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
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _) => {
                widen(place, &mut narrow)
            }
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

pub(super) fn is_narrowable_seq(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Array { .. } | Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
    )
}

pub(super) fn peel_layout(
    layout: Option<&cpl_ir::NarrowLayout>,
    depth: usize,
) -> Option<&cpl_ir::NarrowLayout> {
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
                .is_some_and(|(_, t)| matches!(t, Ty::Array { elem, .. } | Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) if matches!(elem.as_ref(), Ty::Int)))
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
    // An unannotated `let` of a narrowed value starts narrow; any flow below
    // that needs it wide undoes that. A local stored into element-wise, mutated
    // or given a fresh value keeps its declared width, since those stores are
    // not range-checked against the source's range.
    let scopes: Vec<(ScopeId, &cpl_ir::Block)> = [
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
    let mut stored: BTreeSet<ElemLoc> = BTreeSet::new();
    for &(scope, body) in &scopes {
        crate::walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::Assign(place, op, val) = stmt {
                let root = match &place.kind {
                    PlaceKind::Index(_, _) if matches!(val.kind, ExprKind::None) => None,
                    PlaceKind::Index(base, _) => place_loc(scope, base),
                    _ if *op != cpl_ir::AssignOp::Set
                        || is_fresh_adaptable(val)
                        || expr_loc(scope, val).is_none() =>
                    {
                        place_loc(scope, place)
                    }
                    _ => None,
                };
                stored.extend(root);
            }
        });
        crate::walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, ..) => {
                stored.extend(place_loc(scope, place))
            }
            _ => {}
        });
    }
    loop {
        let mut grew = false;
        for &(scope, body) in &scopes {
            crate::walk::deep_stmts(body, &mut |stmt| {
                let (dst, val) = match stmt {
                    Stmt::Let(slot, Some(val)) => (ElemLoc::Local(scope, *slot), val),
                    Stmt::Assign(place, cpl_ir::AssignOp::Set, val)
                        if matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) =>
                    {
                        match place_loc(scope, place) {
                            Some(dst) => (dst, val),
                            None => return,
                        }
                    }
                    _ => return,
                };
                {
                    if is_narrowable_seq(&val.ty)
                        && !stored.contains(&dst)
                        && !narrow.contains_key(&dst)
                    {
                        if let Some(w) = expr_loc(scope, val).and_then(|l| narrow.get(&l)).cloned()
                        {
                            narrow.insert(dst, w);
                            grew = true;
                        }
                    }
                }
            });
        }
        if !grew {
            break;
        }
    }
    // Propagate outer widths into closure captures so closures capturing a narrowed
    // array can stay narrow if all uses inside the closure are compatible.
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
            ExprKind::Builtin(
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::ToList
                | Builtin::Copy,
                args,
            ) => args.first().and_then(|a| expr_loc(scope, a)),
            ExprKind::Do(blk) => match blk.stmts.last() {
                Some(Stmt::Give(inner)) => expr_loc(scope, inner),
                _ => None,
            },
            _ => None,
        }
    }

    // Propagate outer layouts into closure captures so closures capturing a narrowed
    // container can stay narrow if all uses inside the closure are compatible.
    let mut cap_links: Vec<(ElemLoc, ElemLoc)> = Vec::new();
    let collect_caps =
        |scope: ScopeId, body: &cpl_ir::Block, links: &mut Vec<(ElemLoc, ElemLoc)>| {
            crate::walk::deep_exprs(body, &mut |e| {
                if let ExprKind::Closure(cid) = &e.kind {
                    let c = &program.closures[*cid];
                    for cap in &c.captures {
                        if c.locals
                            .get(cap.inner)
                            .is_some_and(|l| is_narrowable_seq(&l.ty))
                        {
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
            ExprKind::Builtin(
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::ToList
                | Builtin::Copy,
                args,
            ) => args.first().and_then(|a| expr_root(scope, a)),
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
    let collect_fors =
        |scope: ScopeId, body: &cpl_ir::Block, links: &mut Vec<(ElemLoc, usize, ElemLoc)>| {
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
            // These read the sequence's shape or one element, as an index does.
            ExprKind::Builtin(
                Builtin::Len | Builtin::PosKey | Builtin::KeyPos | Builtin::First | Builtin::Last,
                args,
            ) => {
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
            ExprKind::Builtin(
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::ToList
                | Builtin::Copy,
                args,
            ) => {
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
                        | Mutation::Reserve
                        | Mutation::Swap
                        | Mutation::Remove
                );
                if safe_mut && is_narrowable_seq(&place.ty) {
                    if matches!(m, Mutation::Pop) {
                        if let Some((loc, depth)) = place_root(scope, place) {
                            if matches!(
                                peel_layout(narrow.get(&loc), depth),
                                Some(NarrowLayout::Seq(_))
                            ) {
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
                } else if matches!(m, Mutation::Extend)
                    && is_narrowable_seq(&place.ty)
                    && args.len() == 1
                {
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
                                    if peel_layout(narrow.get(&src_loc), src_depth)
                                        != narrow.get(&p_loc)
                                    {
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
            ExprKind::Builtin(
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::ToList
                | Builtin::Copy,
                args,
            ) => {
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
                        Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
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
                        Iter::Seq(e, _) | Iter::RangeVal(e, _) => {
                            check_val(program, scope, e, narrow, changed);
                        }
                        Iter::TreeRange {
                            container, lo, hi, ..
                        } => {
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
        check_block(
            program,
            ScopeId::Main,
            &program.setup,
            &mut narrow,
            &mut changed,
            false,
        );
        check_block(
            program,
            ScopeId::Main,
            &program.main,
            &mut narrow,
            &mut changed,
            false,
        );
        for (fid, f) in program.fns.iter().enumerate() {
            check_block(
                program,
                ScopeId::Fn(fid),
                &f.body,
                &mut narrow,
                &mut changed,
                false,
            );
        }
        for (cid, c) in program.closures.iter().enumerate() {
            check_block(
                program,
                ScopeId::Closure(cid),
                &c.body,
                &mut narrow,
                &mut changed,
                false,
            );
        }
        if !changed {
            break;
        }
    }
    narrow
}
