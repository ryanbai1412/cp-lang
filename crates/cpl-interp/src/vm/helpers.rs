use super::ir::*;
use crate::value::{Array, Cells, TupleVal, Value};
use crate::{bug, interp_failed, release, trap, Part, Res};
use cpl_diag::Span;
use cpl_ir::{AssignOp, BinOp, Expr, ExprKind, Iter, Place, PlaceKind, Program, Stmt, Ty};
use std::rc::Rc;

/// The index expressions of `place`, outermost first, in the order the
/// tree-walker evaluates them.
pub(crate) fn place_indices<'p>(place: &'p Place, out: &mut Vec<&'p Expr>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => place_indices(base, out),
        PlaceKind::Index(base, indices) => {
            place_indices(base, out);
            out.extend(indices);
        }
    }
}

pub(super) fn ty_uses_structural_cmp(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Struct(id) => {
            let s = &program.structs[*id];
            s.eq_fn.is_none()
                && s.ord.is_none()
                && s.cached_fields.is_empty()
                && s.fields
                    .iter()
                    .all(|(_, t)| ty_uses_structural_cmp(t, program))
        }
        Ty::Tuple(ts) => ts.iter().all(|t| ty_uses_structural_cmp(t, program)),
        Ty::Record(fs) => fs.iter().all(|(_, t)| ty_uses_structural_cmp(t, program)),
        Ty::List(t) | Ty::Opt(t) => ty_uses_structural_cmp(t, program),
        Ty::Array { elem, .. } => ty_uses_structural_cmp(elem, program),
        Ty::Int
        | Ty::I128
        | Ty::Float
        | Ty::Bool
        | Ty::Char
        | Ty::Str
        | Ty::Unit
        | Ty::Pos(_)
        | Ty::Bound(_) => true,
        _ => false,
    }
}

/// In a traced run a local the VM keeps in a register is left out of the
/// stack, which the tracer shows, until it is declared, as in the tree-walker.
#[inline(always)]
/// The items a `yield from` of `seq` feeds: its elements, in order.
pub(crate) fn yielded_items(seq: Value, span: Span) -> Res<Vec<Value>> {
    match seq.cells() {
        Some(cells) => Ok(cells.to_values()),
        None => bug(span, "a `yields` fn did something else to its list"),
    }
}

pub(crate) fn undeclared(traced: bool, loc: &SlotLoc, held: &Value) -> bool {
    traced && !matches!(loc, SlotLoc::V(_)) && matches!(held, Value::Unit)
}

/// The value of the local at `loc`, read without taking it from its register.
pub(crate) fn read_slot(loc: &SlotLoc, regs_i: &[i64], regs_v: &[Value]) -> Value {
    match *loc {
        SlotLoc::V(r) => regs_v[r as usize].clone(),
        SlotLoc::I(r, kind) => kind.box_val(regs_i[r as usize]),
        SlotLoc::OptI { val, some, kind } => {
            kind.box_opt(regs_i[val as usize], regs_i[some as usize])
        }
        SlotLoc::SmallTuple {
            base: b,
            len,
            kinds,
        } => {
            let fs = crate::value::Shared::collect(len as usize, |k| {
                kinds[k].box_val(regs_i[b as usize + k])
            });
            Value::Tuple(TupleVal::Boxed(fs))
        }
    }
}

pub(super) fn sync_to_stack(
    slot_map: &[SlotLoc],
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        slot_to_stack(slot, loc, regs_i, regs_v, stack, base, traced);
    }
}

#[inline(always)]
fn slot_to_stack(
    slot: usize,
    loc: &SlotLoc,
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    if undeclared(traced, loc, &stack[base + slot]) {
        return;
    }
    // The tree-walker, which a fallback hands the value to, may have
    // stopped with a trap before handing it back.
    if traced && matches!(*loc, SlotLoc::V(r) if matches!(regs_v[r as usize], Value::Unit)) {
        return;
    }
    stack[base + slot] = match *loc {
        SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
        _ => read_slot(loc, regs_i, regs_v),
    };
}

/// Puts on the stack the locals a closure of `chunk` captures by reference,
/// which a call may read or write there.
pub(super) fn sync_refs_to_stack(
    chunk: &Chunk,
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    match &chunk.ref_slots {
        None => sync_to_stack(&chunk.slot_map, regs_i, regs_v, stack, base, chunk.traced),
        Some(slots) => {
            for &slot in slots {
                let slot = slot as usize;
                let loc = &chunk.slot_map[slot];
                slot_to_stack(slot, loc, regs_i, regs_v, stack, base, chunk.traced);
            }
        }
    }
}

/// Takes back what `sync_refs_to_stack` put on the stack.
pub(super) fn sync_refs_from_stack(
    chunk: &Chunk,
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    match &chunk.ref_slots {
        None => sync_from_stack(&chunk.slot_map, regs_i, regs_v, stack, base, chunk.traced),
        Some(slots) => {
            for &slot in slots {
                let slot = slot as usize;
                let loc = &chunk.slot_map[slot];
                slot_from_stack(slot, loc, regs_i, regs_v, stack, base, chunk.traced);
            }
        }
    }
    keep_view_roots(chunk, regs_v, stack, base);
}

/// The slots of `chunk`'s frame a call can reach while the frame lends
/// `lent`: the lent ones and those a closure captures by reference, or
/// `None` for every slot.
fn reachable_slots(chunk: &Chunk, lent: &[(usize, Rc<Part>)], base: usize) -> Option<Vec<usize>> {
    let refs = chunk.ref_slots.as_ref().filter(|_| !chunk.traced)?;
    let mut reach = vec![false; chunk.slot_map.len()];
    for l in lent.iter().rev().take_while(|l| l.0 >= base) {
        reach[l.0 - base] = true;
    }
    for &s in refs {
        reach[usize::from(s)] = true;
    }
    Some(
        (reach.iter().enumerate())
            .filter(|r| *r.1)
            .map(|r| r.0)
            .collect(),
    )
}

/// Puts on the stack the slots a call can reach while the frame lends
/// `lent` (see `reachable_slots`).
pub(super) fn sync_reachable_to_stack(
    chunk: &Chunk,
    lent: &[(usize, Rc<Part>)],
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    match reachable_slots(chunk, lent, base) {
        None => sync_to_stack(&chunk.slot_map, regs_i, regs_v, stack, base, chunk.traced),
        Some(slots) => {
            for slot in slots {
                slot_to_stack(
                    slot,
                    &chunk.slot_map[slot],
                    regs_i,
                    regs_v,
                    stack,
                    base,
                    false,
                );
            }
        }
    }
}

/// Takes back what `sync_reachable_to_stack` put on the stack.
pub(super) fn sync_reachable_from_stack(
    chunk: &Chunk,
    lent: &[(usize, Rc<Part>)],
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    match reachable_slots(chunk, lent, base) {
        None => sync_from_stack(&chunk.slot_map, regs_i, regs_v, stack, base, chunk.traced),
        Some(slots) => {
            for slot in slots {
                slot_from_stack(
                    slot,
                    &chunk.slot_map[slot],
                    regs_i,
                    regs_v,
                    stack,
                    base,
                    false,
                );
            }
        }
    }
    keep_view_roots(chunk, regs_v, stack, base);
}

/// Hands the moved values back to their registers after a trace point,
/// which reads the stack but writes none of the scalars kept in registers.
pub(super) fn take_back_values(
    slot_map: &[SlotLoc],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        if let SlotLoc::V(r) = *loc {
            regs_v[r as usize] = std::mem::replace(&mut stack[base + slot], Value::Unit);
        }
    }
}

/// Takes back every local `sync_to_stack` put on the stack, leaving there
/// the roots of `chunk`'s views.
pub(super) fn sync_back(
    chunk: &Chunk,
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    sync_from_stack(&chunk.slot_map, regs_i, regs_v, stack, base, chunk.traced);
    keep_view_roots(chunk, regs_v, stack, base);
}

/// Leaves on the stack a share of each local a view of `chunk` writes
/// into, where reading through the view looks.
pub(super) fn keep_view_roots(chunk: &Chunk, regs_v: &[Value], stack: &mut [Value], base: usize) {
    for &slot in &chunk.view_roots {
        if let SlotLoc::V(r) = chunk.slot_map[slot as usize] {
            stack[base + slot as usize] = regs_v[r as usize].clone();
        }
    }
}

pub(super) fn sync_from_stack(
    slot_map: &[SlotLoc],
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        slot_from_stack(slot, loc, regs_i, regs_v, stack, base, traced);
    }
}

#[inline(always)]
fn slot_from_stack(
    slot: usize,
    loc: &SlotLoc,
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    if undeclared(traced, loc, &stack[base + slot]) {
        return;
    }
    match *loc {
        SlotLoc::I(r, _) => regs_i[r as usize] = ScalarKind::unbox_val(&stack[base + slot]),
        SlotLoc::OptI { val, some, .. } => {
            let (v, s) = ScalarKind::unbox_opt(&stack[base + slot]);
            regs_i[val as usize] = v;
            regs_i[some as usize] = s;
        }
        SlotLoc::SmallTuple { base: b, len, .. } => {
            if let Value::Tuple(TupleVal::Boxed(fs)) = &stack[base + slot] {
                for (k, f) in fs.iter().take(len as usize).enumerate() {
                    regs_i[b as usize + k] = ScalarKind::unbox_val(f);
                }
            }
        }
        SlotLoc::V(r) => {
            regs_v[r as usize] = std::mem::replace(&mut stack[base + slot], Value::Unit);
        }
    }
}

#[inline(always)]
/// The Int cell `steps` (each an `Index1`) name inside arrays and lists
/// nested from `v`, trapping on an index outside its level.
pub(super) fn nested_read_i(
    v: &Value,
    steps: &[PlaceStep],
    regs_i: &[i64],
    span: Span,
) -> Res<i64> {
    let [PlaceStep::Index1(r), rest @ ..] = steps else {
        return bug(span, "expected an index step");
    };
    let i = regs_i[*r as usize];
    let (len, msg) = match v {
        Value::Array(arr) => (
            arr.dims[0],
            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_LENGTH,
        ),
        Value::List(list) => (
            list.len() as i64,
            cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH,
        ),
        _ => return bug(span, "expected array or list"),
    };
    if i < 0 || i >= len {
        return trap(span, msg.at(&[&i, &len]));
    }
    match (v, rest.is_empty()) {
        (Value::Array(arr), true) => Ok(read_array_i(arr, i as usize)),
        (Value::Array(arr), false) => match &*arr.data {
            Cells::Values(vs) => nested_read_i(&vs[i as usize], rest, regs_i, span),
            _ => bug(span, "expected nested array"),
        },
        (Value::List(list), false) => {
            nested_read_i(&list.get_unchecked_val(i as usize), rest, regs_i, span)
        }
        _ => bug(span, "expected an array of Int"),
    }
}

pub(super) fn read_array_i(arr: &Array, at: usize) -> i64 {
    // Bounds checks here cost the array benches about 2%; the VM checks
    // `at` against the array's domain before it calls this, and debug
    // builds (every test) check it again.
    debug_assert!(
        at < arr.data.len(),
        "cell {at} past the array's {}",
        arr.data.len()
    );
    // SAFETY: in range, as asserted above.
    unsafe {
        match &*arr.data {
            Cells::Int(v) | Cells::OneIntStruct(v) => *v.get_unchecked(at),
            Cells::Bool(v) | Cells::Char(v) => i64::from(*v.get_unchecked(at)),
            Cells::Values(v) => ScalarKind::unbox_val(v.get_unchecked(at)),
            Cells::OptInt(v) => v.get_unchecked(at).unwrap_or(0),
            Cells::Float(_) | Cells::Rows { .. } => 0,
        }
    }
}

#[inline(always)]
pub(super) fn write_array_i(arr: &mut Array, at: usize, val: i64) {
    // Bounds checks here cost the array benches about 2%; the VM checks
    // `at` against the array's domain before it calls this, and debug
    // builds (every test) check it again.
    debug_assert!(
        at < arr.data.len(),
        "cell {at} past the array's {}",
        arr.data.len()
    );
    // SAFETY: in range, as asserted above.
    unsafe {
        match &mut **arr {
            Cells::Int(v) | Cells::OneIntStruct(v) => *v.get_unchecked_mut(at) = val,
            Cells::Bool(v) => *v.get_unchecked_mut(at) = u8::from(val != 0),
            Cells::Char(v) => *v.get_unchecked_mut(at) = val as u8,
            Cells::Values(v) => {
                // Keeps the element's shape: a `[D]Mod<M>` built boxed holds structs.
                match v.get_unchecked_mut(at) {
                    Value::Tuple(TupleVal::OneInt(x)) => *x = val,
                    Value::U64(x) => *x = val as u64,
                    Value::Bool(x) => *x = val != 0,
                    Value::Char(x) => *x = val as u8,
                    slot => *slot = Value::Int(val),
                }
            }
            Cells::OptInt(v) => *v.get_unchecked_mut(at) = Some(val),
            Cells::Float(_) | Cells::Rows { .. } => {}
        }
    }
}

#[inline(always)]
pub(super) fn read_array_opt_i(arr: &Array, at: usize) -> (i64, i64) {
    // Bounds checks here cost the array benches about 2%; the VM checks
    // `at` against the array's domain before it calls this, and debug
    // builds (every test) check it again.
    debug_assert!(
        at < arr.data.len(),
        "cell {at} past the array's {}",
        arr.data.len()
    );
    // SAFETY: in range, as asserted above.
    unsafe {
        match &*arr.data {
            Cells::OptInt(v) => match *v.get_unchecked(at) {
                Some(x) => (x, 1),
                None => (0, 0),
            },
            Cells::Values(v) => ScalarKind::unbox_opt(v.get_unchecked(at)),
            _ => (0, 0),
        }
    }
}

#[inline(always)]
pub(super) fn write_array_opt_i(arr: &mut Array, at: usize, val: i64, some: i64, kind: ScalarKind) {
    // Bounds checks here cost the array benches about 2%; the VM checks
    // `at` against the array's domain before it calls this, and debug
    // builds (every test) check it again.
    debug_assert!(
        at < arr.data.len(),
        "cell {at} past the array's {}",
        arr.data.len()
    );
    match &mut **arr {
        // SAFETY: in range, as asserted above.
        Cells::OptInt(v) if kind == ScalarKind::Int => unsafe {
            *v.get_unchecked_mut(at) = if some != 0 { Some(val) } else { None };
        },
        // SAFETY: in range, as asserted above.
        Cells::Values(v) => unsafe {
            *v.get_unchecked_mut(at) = kind.box_opt(val, some);
        },
        _ => {
            arr.as_mut_values()[at] = kind.box_opt(val, some);
        }
    }
}

/// Stores `raw` boxed as `kind` in `slot`, in place when it already holds an `Int`.
#[inline(always)]
pub(super) fn set_scalar(slot: &mut Value, kind: ScalarKind, raw: i64) {
    match (kind, &mut *slot) {
        (ScalarKind::Int, Value::Int(held)) => *held = raw,
        _ => *slot = kind.box_val(raw),
    }
}

pub(super) fn quiet(e: &Expr) -> bool {
    let mut quiet = true;
    cpl_ir::walk::deep_expr(e, &mut |e| quiet &= quiet_node(e));
    quiet
}

/// Whether `e` itself, not counting its operands, writes or calls nothing.
pub(super) fn quiet_node(e: &Expr) -> bool {
    !matches!(
        e.kind,
        ExprKind::Call(..)
            | ExprKind::CallExpr(..)
            | ExprKind::Closure(..)
            | ExprKind::Do(_)
            | ExprKind::MutSlice(..)
            | ExprKind::MutLine(..)
            | ExprKind::Mutate(..)
            | ExprKind::Sort(..)
            | ExprKind::Print(..)
            | ExprKind::Debug(..)
    )
}

/// Whether the indices of `place` are [`quiet`] and read nothing of its root
/// the write can change, so that evaluating them after the write gives the
/// cell that was written.
pub(super) fn quiet_place(place: &Place, root: (bool, usize)) -> bool {
    quiet_place_under(place, root, &written_fields(place).0)
}

fn quiet_place_under(place: &Place, root: (bool, usize), written: &[usize]) -> bool {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => true,
        PlaceKind::Index(base, indices) => {
            quiet_place_under(base, root, written)
                && (indices.iter())
                    .all(|index| quiet(index) && !reads_written(index, root, written))
        }
        PlaceKind::Field(base, _) => quiet_place_under(base, root, written),
        PlaceKind::Some(_) => false,
    }
}

/// The fields from its root down to the first index or `?` of `place`: a
/// write to `place` changes nothing of the root outside them.
fn written_fields(place: &Place) -> (Vec<usize>, bool) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => (Vec::new(), true),
        PlaceKind::Field(base, field) => {
            let (mut fields, open) = written_fields(base);
            if open {
                fields.push(*field);
            }
            (fields, open)
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => (written_fields(base).0, false),
    }
}

/// Whether `e` reads a part of `root` that overlaps its `written` fields.
fn reads_written(e: &Expr, root: (bool, usize), written: &[usize]) -> bool {
    if let Some(read) = root_fields(e, root) {
        return read.iter().zip(written).all(|(r, w)| r == w);
    }
    let mut reads = false;
    cpl_ir::walk::children(e, &mut |child| reads |= reads_written(child, root, written));
    reads
}

/// The fields `e` reads of `root`, when `e` is `root` or a field path of it.
fn root_fields(e: &Expr, root: (bool, usize)) -> Option<Vec<usize>> {
    match e.kind {
        ExprKind::Global(id) if root == (true, id) => Some(Vec::new()),
        ExprKind::Local(slot) if root == (false, slot) => Some(Vec::new()),
        ExprKind::Field(ref base, field) => root_fields(base, root).map(|mut fields| {
            fields.push(field);
            fields
        }),
        _ => None,
    }
}

#[inline(always)]
pub(super) fn place_root_mut<'a>(
    root: PlaceRoot,
    regs_v: &'a mut [Value],
    globals: &'a mut [Value],
) -> &'a mut Value {
    match root {
        PlaceRoot::LocalV(v) => &mut regs_v[v as usize],
        PlaceRoot::Global(gid) => &mut globals[gid as usize],
    }
}

/// `U64` arithmetic: the wrapping ops and in-range shifts inline into the
/// dispatch loop, the rest (division, powers, traps) stay out of line.
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub(super) fn exec_bin_u64(a_raw: i64, b_raw: i64, op: BinOp, span: Span) -> Res<i64> {
    let (av, bv) = (a_raw as u64, b_raw as u64);
    let out = match op {
        BinOp::Add => av.wrapping_add(bv),
        BinOp::Sub => av.wrapping_sub(bv),
        BinOp::Mul => av.wrapping_mul(bv),
        BinOp::Shl if bv < 64 => av << bv,
        BinOp::Shr if bv < 64 => av >> bv,
        _ => return exec_bin_u64_rest(a_raw, b_raw, op, span),
    };
    Ok(out as i64)
}

#[inline(never)]
fn exec_bin_u64_rest(a_raw: i64, b_raw: i64, op: BinOp, span: Span) -> Res<i64> {
    let av = a_raw as u64;
    let bv = b_raw as u64;
    let out = match op {
        BinOp::Div => {
            if bv == 0 {
                return release::div_zero(
                    span,
                    cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&av, &"/"]),
                );
            }
            av / bv
        }
        BinOp::Rem => {
            if bv == 0 {
                return release::div_zero(
                    span,
                    cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&av, &"%"]),
                );
            }
            av % bv
        }
        BinOp::Pow => {
            let mut base = av;
            let mut exp = bv;
            let mut result = 1u64;
            while exp > 0 {
                if exp & 1 == 1 {
                    result = result.wrapping_mul(base);
                }
                base = base.wrapping_mul(base);
                exp >>= 1;
            }
            result
        }
        BinOp::Shl => {
            if !(0..64).contains(&b_raw) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&b_raw]),
                );
            }
            av << b_raw
        }
        BinOp::Shr => {
            if !(0..64).contains(&b_raw) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&b_raw]),
                );
            }
            av >> b_raw
        }
        _ => return bug(span, "unsupported U64 op"),
    };
    Ok(out as i64)
}

#[inline(never)]
/// The elements of a list or array in the stable order of `keys`, one
/// per element; `desc` puts larger keys first and keeps ties in order.
pub(super) fn sort_by_keys(seq: Value, keys: &Value, desc: bool, span: Span) -> Res<Value> {
    let Value::List(keys) = keys else {
        return bug(span, "expected the sort keys");
    };
    let order = crate::sort::stable_order(keys.len(), &mut |a, b| {
        let (a, b) = (keys.get_unchecked_val(a), keys.get_unchecked_val(b));
        Ok(if desc { a > b } else { a < b })
    })?;
    match seq {
        Value::List(items) => Ok(Value::List(Rc::new(
            order.iter().map(|&k| items.get_unchecked_val(k)).collect(),
        ))),
        Value::Array(arr) => {
            let values = order.iter().map(|&k| arr.get_unchecked_val(k)).collect();
            Ok(Value::Array(Rc::new(Array::from_values(
                arr.dims.clone(),
                values,
            ))))
        }
        _ => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
    }
}

/// The elements of a list or an array, with the array's dims.
pub(crate) fn seq_items(seq: Value, span: Span) -> Res<(Vec<Value>, Option<Vec<i64>>)> {
    match seq {
        Value::List(items) => Ok((crate::value::Cells::unwrap_values(items), None)),
        Value::Array(arr) => {
            let arr = Rc::unwrap_or_clone(arr);
            Ok((crate::value::Cells::unwrap_values(arr.data), Some(arr.dims)))
        }
        _ => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
    }
}

/// The inverse of `seq_items`.
pub(crate) fn seq_of_items(items: Vec<Value>, dims: Option<Vec<i64>>) -> Value {
    match dims {
        None => Value::list(items),
        Some(dims) => Value::Array(Rc::new(Array::from_values(dims, items))),
    }
}

pub(super) fn exec_sort_place(target: &mut Value, desc: bool, span: Span) -> Res<()> {
    match target {
        Value::List(items) => {
            let items = Rc::make_mut(items);
            items.sort_unstable();
            if desc {
                items.reverse();
            }
        }
        Value::Str(bytes) => {
            let bytes = Rc::make_mut(bytes);
            bytes.sort_unstable();
            if desc {
                bytes.reverse();
            }
        }
        Value::Array(arr) => {
            let a = Rc::make_mut(arr);
            (**a).sort_unstable();
            if desc {
                (**a).reverse();
            }
        }
        _ => return bug(span, "cannot sort this value"),
    }
    Ok(())
}

pub(super) enum ExecExit {
    Return(Value),
    /// A `RetSmallTuple`: the fields are in `Interp::ret_tuple`.
    ReturnTuple,
    FeedNext,
    FeedBreak,
}

pub(super) struct FedCtx<'a, 'p> {
    pub(super) caller_chunk: &'a Chunk<'p>,
    pub(super) caller_base_i: usize,
    pub(super) caller_regs_v: &'a mut [Value],
    pub(super) caller_base: usize,
    pub(super) body_pc: usize,
    pub(super) var_loc: SlotLoc,
    /// The call feeding the frame this one's loop body runs in, when that
    /// frame is itself a `yields` fn fed by a loop: its yields go on out.
    pub(super) parent: Option<&'a mut (dyn FedParent<'p> + 'a)>,
}

/// A [`FedCtx`] whatever its borrows, for a fed call inside a fed call.
pub(super) trait FedParent<'p> {
    fn with(
        &mut self,
        f: &mut dyn FnMut(&mut FedCtx<'_, 'p>) -> crate::Res<ExecExit>,
    ) -> crate::Res<ExecExit>;
}

impl<'p> FedParent<'p> for FedCtx<'_, 'p> {
    fn with(
        &mut self,
        f: &mut dyn FnMut(&mut FedCtx<'_, 'p>) -> crate::Res<ExecExit>,
    ) -> crate::Res<ExecExit> {
        f(self)
    }
}

pub(super) fn store_fed_val(ctx: &mut FedCtx<'_, '_>, vm_regs_i: &mut [i64], val: Value) {
    let regs_i = &mut vm_regs_i[ctx.caller_base_i..];
    write_slot(&ctx.var_loc, regs_i, ctx.caller_regs_v, val);
}

/// Puts `val` in the local at `loc`: [`read_slot`]'s inverse.
/// The value a slot holds, boxed from its registers; a `V` register is left
/// `Unit`. `None` for a small tuple, which no lent place holds.
pub(super) fn take_slot(loc: &SlotLoc, regs_i: &[i64], regs_v: &mut [Value]) -> Option<Value> {
    Some(match *loc {
        SlotLoc::I(r, kind) => kind.box_val(regs_i[r as usize]),
        SlotLoc::OptI { val, some, kind } => {
            kind.box_opt(regs_i[val as usize], regs_i[some as usize])
        }
        SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
        SlotLoc::SmallTuple { .. } => return None,
    })
}

pub(super) fn write_slot(loc: &SlotLoc, regs_i: &mut [i64], regs_v: &mut [Value], val: Value) {
    match *loc {
        SlotLoc::I(r, _) => regs_i[r as usize] = ScalarKind::unbox_val(&val),
        SlotLoc::OptI {
            val: r_v,
            some: r_s,
            ..
        } => {
            let (v, s) = ScalarKind::unbox_opt(&val);
            regs_i[r_v as usize] = v;
            regs_i[r_s as usize] = s;
        }
        SlotLoc::SmallTuple { base, len, .. } => match &val {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                for k in 0..len as usize {
                    regs_i[base as usize + k] = ScalarKind::unbox_val(&fs[k]);
                }
            }
            Value::Tuple(TupleVal::OneInt(v)) => regs_i[base as usize] = *v,
            _ => {}
        },
        SlotLoc::V(r) => regs_v[r as usize] = val,
    }
}

pub(super) fn take_value(cell: &mut Value, span: Span) -> Res<Value> {
    if matches!(cell, Value::Moved) {
        return trap(span, cpl_diag::msgs::interp::LENT.at(&[]));
    }
    let empty = match &*cell {
        Value::List(_) => Value::List(Rc::default()),
        Value::Str(_) => Value::Str(Rc::default()),
        Value::Map(m) => Value::Map(Rc::new(m.empty_like())),
        Value::Set(s) => Value::Set(Rc::new(s.empty_like())),
        Value::Tuple(TupleVal::Boxed(_)) => Value::Moved,
        other => other.clone(),
    };
    Ok(std::mem::replace(cell, empty))
}

#[inline(never)]
pub(super) fn array_flat_index(
    arr: &Array,
    step: PlaceStep,
    regs_i: &[i64],
    span: Span,
) -> Res<usize> {
    match step {
        PlaceStep::Key(_) | PlaceStep::Some => Err(interp_failed(span)),
        PlaceStep::Index1(r_i) => {
            let idx = regs_i[r_i as usize];
            let len = arr.dims[0];
            if idx < 0 || idx >= len {
                return trap(
                    span,
                    cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH.at(&[&idx, &len]),
                );
            }
            Ok(idx as usize)
        }
        PlaceStep::Index2(r_i, r_j) => {
            let i = regs_i[r_i as usize];
            let j = regs_i[r_j as usize];
            let d0 = arr.dims[0];
            let d1 = arr.dims[1];
            if i < 0 || i >= d0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH.at(&[&i, &d0]),
                );
            }
            if j < 0 || j >= d1 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH.at(&[&j, &d1]),
                );
            }
            Ok((i * d1 + j) as usize)
        }
        PlaceStep::IndexN(idx, n) => {
            nd_flat_index(&arr.dims, &regs_i[idx as usize..(idx + n) as usize], span)
        }
        PlaceStep::Field(_) => bug(span, "expected index step"),
    }
}

/// The row-major cell of `indices` in an array of `dims`, each checked.
/// The first value of `lo..hi by step` (`..=hi` when `closed`), or of it
/// walked backward when `rev`; `None` when no value is in range.
pub(super) fn step_first(
    lo: i64,
    hi: i64,
    step: i64,
    closed: bool,
    rev: bool,
    span: Span,
) -> Res<Option<i64>> {
    if step <= 0 {
        return trap(
            span,
            cpl_diag::msgs::interp::RANGE_STEPS_FORWARD_BY_NOT.at(&[&step]),
        );
    }
    // One past the last value, which an `Int` may not hold.
    let (lo, end, step) = (
        i128::from(lo),
        i128::from(hi) + i128::from(closed),
        i128::from(step),
    );
    if lo >= end {
        return Ok(None);
    }
    let count = (end - lo + step - 1) / step;
    Ok(Some(if rev { lo + (count - 1) * step } else { lo } as i64))
}

/// The value after `i` in a stepped loop that ends at `end` (see
/// [`Op::ForStepNext`]), or `None` past it.
pub(super) fn step_next(i: i64, step: i64, end: i64, closed: bool, rev: bool) -> Option<i64> {
    let (i, step, end) = (i128::from(i), i128::from(step), i128::from(end));
    let next = if rev { i - step } else { i + step };
    let in_range = if rev {
        next >= end
    } else {
        next < end + i128::from(closed)
    };
    in_range.then_some(next as i64)
}

pub(super) fn nd_flat_index(dims: &[i64], indices: &[i64], span: Span) -> Res<usize> {
    let mut flat = 0;
    for (&i, &d) in indices.iter().zip(dims) {
        if i < 0 || i >= d {
            return trap(
                span,
                cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH.at(&[&i, &d]),
            );
        }
        flat = flat * d + i;
    }
    Ok(flat as usize)
}

#[inline(never)]
/// The keys of `pp`'s `Key` steps, taken before its root is borrowed.
pub(super) fn place_keys(pp: &PlacePath, regs_v: &[Value]) -> Vec<Value> {
    pp.keys
        .iter()
        .map(|&r| regs_v[r as usize].clone())
        .collect()
}

/// Runs `f` on the cell at `steps`. An element of a packed array (`[V]Int`)
/// has no `Value` cell, so `f` changes a copy that is stored back.
/// `cell op= val`, moving the old value into the combination so a list or
/// string grows where it is.
pub(super) fn combine_in_place(cell: &mut Value, op: AssignOp, val: Value, span: Span) -> Res<()> {
    let old = std::mem::take(cell);
    if matches!(old, Value::Moved) {
        *cell = old;
        return trap(span, cpl_diag::msgs::interp::LENT.at(&[]));
    }
    *cell = crate::combine(op, old, val, span)?;
    Ok(())
}

pub(super) fn with_place_cell<R>(
    root: &mut Value,
    steps: &[PlaceStep],
    regs_i: &[i64],
    keys: &[Value],
    span: Span,
    f: impl FnOnce(&mut Value) -> Res<R>,
) -> Res<R> {
    if let Some((
        &last @ (PlaceStep::Index1(_) | PlaceStep::Index2(..) | PlaceStep::IndexN(..)),
        prefix,
    )) = steps.split_last()
    {
        let container = walk_place_mut(root, prefix, regs_i, keys, span)?;
        if let Value::Array(arr) = container {
            if !matches!(*arr.data, Cells::Values(_)) {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                let Some(mut cell) = a.get(flat) else {
                    return Err(interp_failed(span));
                };
                let out = f(&mut cell)?;
                a.set(flat, cell);
                return Ok(out);
            }
        }
    }
    f(walk_place_mut(root, steps, regs_i, keys, span)?)
}

pub(super) fn walk_place_mut<'a>(
    mut cur: &'a mut Value,
    steps: &[PlaceStep],
    regs_i: &[i64],
    keys: &[Value],
    span: Span,
) -> Res<&'a mut Value> {
    for &step in steps {
        match step {
            PlaceStep::Key(n) => match cur {
                Value::Map(entries) => match Rc::make_mut(entries)
                    .get_mut(&keys[n as usize], &mut crate::value::keys::tree_only(span))?
                {
                    Some(value) => cur = value,
                    None => {
                        return trap(span, cpl_diag::msgs::interp::THIS_KEY_NOT_IN_MAP.at(&[]));
                    }
                },
                _ => return bug(span, "expected map"),
            },
            PlaceStep::Some => cur = crate::assign::inside_mut(cur, span)?,
            PlaceStep::Field(f) => match cur {
                Value::Tuple(TupleVal::Boxed(fs)) => {
                    cur = &mut crate::value::Shared::make_mut(fs)[f as usize];
                }
                _ => return bug(span, "expected tuple"),
            },
            PlaceStep::Index1(r_i) => match cur {
                Value::List(items) => {
                    let idx = regs_i[r_i as usize];
                    let len = items.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            span,
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                        );
                    }
                    cur = &mut Rc::make_mut(items).as_mut_values()[idx as usize];
                }
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut **a {
                        Cells::Values(vs) => cur = &mut vs[flat],
                        _ => return bug(span, "expected boxed array element"),
                    }
                }
                _ => return bug(span, "expected array or list"),
            },
            PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match cur {
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut **a {
                        Cells::Values(vs) => cur = &mut vs[flat],
                        _ => return bug(span, "expected boxed array element"),
                    }
                }
                _ => return bug(span, "expected array"),
            },
        }
    }
    Ok(cur)
}

#[inline(never)]
pub(super) fn store_place_leaf_scalar(
    container: &mut Value,
    last: PlaceStep,
    raw: i64,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Key(_) => Err(interp_failed(span)),
        PlaceStep::Some => {
            *crate::assign::inside_mut(container, span)? = kind.box_val(raw);
            Ok(())
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                crate::value::Shared::make_mut(fs)[f as usize] = kind.box_val(raw);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = raw;
                Ok(())
            }
            _ => bug(span, "expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::Int(v) | Cells::OneIntStruct(v) => v[flat] = raw,
                    Cells::Bool(v) => v[flat] = u8::from(raw != 0),
                    Cells::Char(v) => v[flat] = raw as u8,
                    Cells::Values(v) => v[flat] = kind.box_val(raw),
                    _ => a.set(flat, kind.box_val(raw)),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                    );
                }
                Rc::make_mut(items).set(idx as usize, kind.box_val(raw));
                Ok(())
            }
            Value::Str(bytes) => {
                let (idx, len) = (regs_i[r_i as usize], bytes.len());
                let Some(i) = usize::try_from(idx).ok().filter(|i| *i < len) else {
                    let msg = cpl_diag::msgs::interp::INDEX_OUTSIDE_STRING_LENGTH;
                    return trap(span, msg.at(&[&idx, &len]));
                };
                Rc::make_mut(bytes)[i] = raw as u8;
                Ok(())
            }
            _ => bug(span, "expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::Int(v) | Cells::OneIntStruct(v) => v[flat] = raw,
                    Cells::Bool(v) => v[flat] = u8::from(raw != 0),
                    Cells::Char(v) => v[flat] = raw as u8,
                    Cells::Values(v) => v[flat] = kind.box_val(raw),
                    _ => a.set(flat, kind.box_val(raw)),
                }
                Ok(())
            }
            _ => bug(span, "expected array"),
        },
    }
}

#[inline(never)]
pub(super) fn store_place_leaf_opt_scalar(
    container: &mut Value,
    last: PlaceStep,
    raw: i64,
    some: i64,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Key(_) => Err(interp_failed(span)),
        PlaceStep::Some => {
            *crate::assign::inside_mut(container, span)? = kind.box_opt(raw, some);
            Ok(())
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                crate::value::Shared::make_mut(fs)[f as usize] = kind.box_opt(raw, some);
                Ok(())
            }
            _ => bug(span, "expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    Cells::Values(v) => v[flat] = kind.box_opt(raw, some),
                    _ => a.set(flat, kind.box_opt(raw, some)),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                    );
                }
                Rc::make_mut(items).set(idx as usize, kind.box_opt(raw, some));
                Ok(())
            }
            _ => bug(span, "expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    Cells::Values(v) => v[flat] = kind.box_opt(raw, some),
                    _ => a.set(flat, kind.box_opt(raw, some)),
                }
                Ok(())
            }
            _ => bug(span, "expected array"),
        },
    }
}

#[inline(never)]
pub(super) fn store_place_leaf_val(
    container: &mut Value,
    last: PlaceStep,
    val: Value,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Key(_) => Err(interp_failed(span)),
        PlaceStep::Some => {
            *crate::assign::inside_mut(container, span)? = val;
            Ok(())
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                crate::value::Shared::make_mut(fs)[f as usize] = val;
                Ok(())
            }
            _ => bug(span, "expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                Rc::make_mut(arr).set(flat, val);
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                    );
                }
                Rc::make_mut(items).set(idx as usize, val);
                Ok(())
            }
            _ => bug(span, "expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                Rc::make_mut(arr).set(flat, val);
                Ok(())
            }
            _ => bug(span, "expected array"),
        },
    }
}

#[inline(never)]
pub(super) fn apply_scalar_assign_op(
    cur: i64,
    rhs: i64,
    op: AssignOp,
    kind: ScalarKind,
    span: Span,
) -> Res<i64> {
    if kind == ScalarKind::U64 {
        let a = cur as u64;
        let b = rhs as u64;
        let out = match op {
            AssignOp::Binary(BinOp::Add) => a.wrapping_add(b),
            AssignOp::Binary(BinOp::Sub) => a.wrapping_sub(b),
            AssignOp::Binary(BinOp::Mul) => a.wrapping_mul(b),
            AssignOp::Binary(BinOp::Div) => {
                if b == 0 {
                    return release::div_zero(
                        span,
                        cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&a, &"/"]),
                    );
                }
                a / b
            }
            AssignOp::Binary(BinOp::Rem) => {
                if b == 0 {
                    return release::div_zero(
                        span,
                        cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&a, &"%"]),
                    );
                }
                a % b
            }
            AssignOp::Binary(BinOp::BitAnd) => a & b,
            AssignOp::Binary(BinOp::BitOr) => a | b,
            AssignOp::Binary(BinOp::BitXor) => a ^ b,
            AssignOp::Binary(BinOp::Shl) => {
                if !(0..64).contains(&rhs) {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&rhs]),
                    );
                }
                a << rhs
            }
            AssignOp::Binary(BinOp::Shr) => {
                if !(0..64).contains(&rhs) {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&rhs]),
                    );
                }
                a >> rhs
            }
            AssignOp::Min => a.min(b),
            AssignOp::Max => a.max(b),
            _ => b,
        };
        return Ok(out as i64);
    }
    let out = match op {
        AssignOp::Binary(BinOp::Add) => match cur.checked_add(rhs) {
            Some(v) => v,
            None => release::overflow(
                span,
                cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&cur, &"+", &rhs]),
                cur.wrapping_add(rhs),
            )?,
        },
        AssignOp::Binary(BinOp::Sub) => match cur.checked_sub(rhs) {
            Some(v) => v,
            None => release::overflow(
                span,
                cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&cur, &"-", &rhs]),
                cur.wrapping_sub(rhs),
            )?,
        },
        AssignOp::Binary(BinOp::Mul) => match cur.checked_mul(rhs) {
            Some(v) => v,
            None => release::overflow(
                span,
                cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&cur, &"*", &rhs]),
                cur.wrapping_mul(rhs),
            )?,
        },
        AssignOp::Binary(BinOp::Div) => {
            if rhs == 0 {
                return release::div_zero(
                    span,
                    cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&cur, &"/"]),
                );
            }
            match cur.checked_div_euclid(rhs) {
                Some(v) => v,
                None => release::overflow(
                    span,
                    cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&cur, &"/", &rhs]),
                    cur.wrapping_div_euclid(rhs),
                )?,
            }
        }
        AssignOp::Binary(BinOp::Rem) => {
            if rhs == 0 {
                return release::div_zero(
                    span,
                    cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&cur, &"%"]),
                );
            }
            match cur.checked_rem_euclid(rhs) {
                Some(v) => v,
                None => release::overflow(
                    span,
                    cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&cur, &"%", &rhs]),
                    cur.wrapping_rem_euclid(rhs),
                )?,
            }
        }
        AssignOp::Binary(BinOp::BitAnd) => cur & rhs,
        AssignOp::Binary(BinOp::BitOr) => cur | rhs,
        AssignOp::Binary(BinOp::BitXor) => cur ^ rhs,
        AssignOp::Binary(BinOp::Shl) => {
            if !(0..64).contains(&rhs) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&rhs]),
                );
            }
            cur.wrapping_shl(rhs as u32)
        }
        AssignOp::Binary(BinOp::Shr) => {
            if !(0..64).contains(&rhs) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&rhs]),
                );
            }
            cur.wrapping_shr(rhs as u32)
        }
        AssignOp::Min => cur.min(rhs),
        AssignOp::Max => cur.max(rhs),
        _ => rhs,
    };
    Ok(out)
}

#[inline(never)]
pub(super) fn assign_place_leaf_scalar(
    container: &mut Value,
    last: PlaceStep,
    rhs: i64,
    op: AssignOp,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Key(_) => Err(interp_failed(span)),
        PlaceStep::Some => {
            let cell = crate::assign::inside_mut(container, span)?;
            let next = apply_scalar_assign_op(ScalarKind::unbox_val(cell), rhs, op, kind, span)?;
            *cell = kind.box_val(next);
            Ok(())
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                let slice = crate::value::Shared::make_mut(fs);
                let cur = ScalarKind::unbox_val(&slice[f as usize]);
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice[f as usize] = kind.box_val(next);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = apply_scalar_assign_op(*x, rhs, op, kind, span)?;
                Ok(())
            }
            _ => bug(span, "expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::Int(v) | Cells::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    cells => {
                        let Some(cur) = cells.get(flat) else {
                            return bug(span, "unsupported array assign");
                        };
                        let next = apply_scalar_assign_op(
                            ScalarKind::unbox_val(&cur),
                            rhs,
                            op,
                            kind,
                            span,
                        )?;
                        cells.set(flat, kind.box_val(next));
                    }
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                    );
                }
                let slice = Rc::make_mut(items);
                let cur = ScalarKind::unbox_val(&slice.get_unchecked_val(idx as usize));
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice.set(idx as usize, kind.box_val(next));
                Ok(())
            }
            _ => bug(span, "expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut **a {
                    Cells::Int(v) | Cells::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    cells => {
                        let Some(cur) = cells.get(flat) else {
                            return bug(span, "unsupported array assign");
                        };
                        let next = apply_scalar_assign_op(
                            ScalarKind::unbox_val(&cur),
                            rhs,
                            op,
                            kind,
                            span,
                        )?;
                        cells.set(flat, kind.box_val(next));
                    }
                }
                Ok(())
            }
            _ => bug(span, "expected array"),
        },
    }
}

/// `CPL_VM_STATS` is native only: `cpl.wasm` has no counting and no report.
#[cfg(not(target_family = "wasm"))]
#[derive(Default)]
pub(crate) struct VmStats {
    pub(crate) total_fallbacks: u64,
    pub(crate) loop_fallbacks: u64,
    pub(crate) hot_loop_fallbacks: u64,
    /// Statements the tree-walker ran, for any reason: what deleting it
    /// still has to move.
    pub(crate) walked: u64,
    pub(crate) walked_counts: std::collections::BTreeMap<String, u64>,
    pub(crate) counts: std::collections::BTreeMap<String, u64>,
    pub(crate) loop_counts: std::collections::BTreeMap<String, u64>,
}

#[cfg(not(target_family = "wasm"))]
impl VmStats {
    pub(crate) fn from_env() -> Option<Self> {
        let val = cpl_env::VM_STATS.raw()?;
        if val.is_empty() || val == "0" {
            return None;
        }
        Some(Self::default())
    }

    pub(crate) fn record_with_loop(&mut self, tag: String, in_loop: bool) {
        self.total_fallbacks += 1;
        if in_loop {
            self.loop_fallbacks += 1;
            let is_io = matches!(tag.as_str(), "expr:Read" | "stmt:Expr(Print)");
            if !is_io {
                self.hot_loop_fallbacks += 1;
                *self.loop_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        *self.counts.entry(tag).or_insert(0) += 1;
    }

    pub(crate) fn emit_stderr(&self) {
        let entries = by_count(&self.counts);
        let loop_entries = by_count(&self.loop_counts);
        if cpl_env::VM_STATS.is("json") {
            let walked = by_count(&self.walked_counts);
            let s = cpl_json::write_with(|w| {
                w.object().key("total_fallbacks").int(self.total_fallbacks);
                w.key("loop_fallbacks").int(self.loop_fallbacks);
                w.key("hot_loop_fallbacks").int(self.hot_loop_fallbacks);
                w.key("walked").int(self.walked);
                for (key, top) in [
                    ("top", &entries),
                    ("loop_top", &loop_entries),
                    ("walked_top", &walked),
                ] {
                    w.key(key).array();
                    for (k, v) in top.iter().take(20) {
                        w.array().str(k).int(*v).end();
                    }
                    w.end();
                }
                w.key("kinds").array();
                for (k, _) in &entries {
                    w.str(k);
                }
                w.end().end();
            });
            eprintln!("{s}");
        } else {
            eprintln!(
                "vm-stats: total_fallbacks={} loop_fallbacks={} hot_loop_fallbacks={} walked={}",
                self.total_fallbacks, self.loop_fallbacks, self.hot_loop_fallbacks, self.walked
            );
            for (k, v) in entries.iter().take(15) {
                eprintln!("  {v:>10}  {k}");
            }
        }
    }
}

/// `map`'s entries by count, largest first, then by name: `map` iterates by
/// name and the shared sort is stable.
#[cfg(not(target_family = "wasm"))]
fn by_count(map: &std::collections::BTreeMap<String, u64>) -> Vec<(&str, u64)> {
    let mut v: Vec<(&str, u64)> = map.iter().map(|(k, &n)| (k.as_str(), n)).collect();
    cpl_diag::sort_by_key(&mut v, |e| u64::MAX - e.1);
    v
}

pub(super) fn place_tag(place: &Place) -> &'static str {
    match &place.kind {
        PlaceKind::Local(_) => "Local",
        PlaceKind::Global(_) => "Global",
        PlaceKind::Index(base, idx) => match (&base.kind, idx.len()) {
            (PlaceKind::Index(..), _) => "Index2D",
            (_, 2) => "Index2D",
            (_, 1) => "Index1D",
            _ => "IndexND",
        },
        PlaceKind::Field(base, _) => match &base.kind {
            PlaceKind::Index(..) => "Index.Field",
            PlaceKind::Field(..) => "Field.Field",
            _ => "Field",
        },
        PlaceKind::Some(_) => "Some",
    }
}

pub(crate) fn stmt_tag(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Let(_, e) => format!("stmt:Let({})", e.as_ref().map_or("Uninit", expr_tag_short)),
        Stmt::Assign(place, op, _) => format!("stmt:Assign({},{:?})", place_tag(place), op),
        Stmt::Expr(e) => format!("stmt:Expr({})", expr_tag_short(e)),
        Stmt::If(..) => "stmt:If".into(),
        Stmt::While(..) => "stmt:While".into(),
        Stmt::For(_, _, iter, _) => match iter {
            Iter::Range { .. } => "stmt:For(Range)".into(),
            Iter::Seq(_, rev) => {
                if *rev {
                    "stmt:For(SeqRev)".into()
                } else {
                    "stmt:For(Seq)".into()
                }
            }
            _ => "stmt:For(Other)".into(),
        },
        Stmt::Return(_) => "stmt:Return".into(),
        Stmt::Break(_) => "stmt:Break".into(),
        Stmt::Continue(_) => "stmt:Continue".into(),
        Stmt::Give(_) => "stmt:Give".into(),
        Stmt::Assert { .. } => "stmt:Assert".into(),
        Stmt::Trap(..) => "stmt:Trap".into(),
        Stmt::DefDomain { .. } => "stmt:DefDomain".into(),
        Stmt::DefDomainFromVal { .. } => "stmt:DefDomainFromVal".into(),
    }
}

pub(super) fn expr_tag_short(expr: &Expr) -> &'static str {
    match &expr.kind {
        ExprKind::Call(..) => "Call",
        ExprKind::CallExpr(..) => "CallExpr",
        ExprKind::Builtin(..) => "Builtin",
        ExprKind::Mutate(..) => "Mutate",
        ExprKind::Index(..) => "Index",
        ExprKind::Field(..) => "Field",
        ExprKind::Tuple(_) => "Tuple",
        ExprKind::Fill(..) => "Fill",
        ExprKind::Cast(..) => "Cast",
        ExprKind::Unary(..) => "Unary",
        ExprKind::Binary(..) => "Binary",
        ExprKind::Closure { .. } => "Closure",
        ExprKind::Variant(..) => "Variant",
        ExprKind::IsVariant(..) => "IsVariant",
        ExprKind::Payload(..) => "Payload",
        ExprKind::Read(_) => "Read",
        ExprKind::ReadFrom(..) => "ReadFrom",
        ExprKind::Print { .. } => "Print",
        ExprKind::Debug { .. } => "Debug",
        ExprKind::Format(_) => "Format",
        ExprKind::Some(_) => "Some",
        ExprKind::None => "None",
        ExprKind::IsSome(_) => "IsSome",
        ExprKind::Unwrap(..) => "Unwrap",
        ExprKind::If(..) => "If",
        ExprKind::Do(_) => "Do",
        ExprKind::Range { .. } => "Range",
        ExprKind::Cmp(..) => "Cmp",
        ExprKind::Sort(..) => "Sort",
        ExprKind::DomainVal(_) => "DomainVal",
        ExprKind::DomainLen(_) => "DomainLen",
        ExprKind::DomainLo(_) => "DomainLo",
        ExprKind::Str(_) => "Str",
        ExprKind::Float(_) => "Float",
        ExprKind::Int(_) => "Int",
        ExprKind::Bool(_) => "Bool",
        ExprKind::Char(_) => "Char",
        ExprKind::Local(_) => "Local",
        ExprKind::Global(_) => "Global",
        ExprKind::Unit => "Unit",
        ExprKind::RangeCheck { .. } => "RangeCheck",
        ExprKind::Intrinsic(..) => "Intrinsic",
        ExprKind::MutSlice(..) => "MutSlice",
        ExprKind::MutLine(..) => "MutLine",
        ExprKind::Error => "Error",
    }
}

pub(super) fn expr_tag(expr: &Expr, program: &Program) -> String {
    match &expr.kind {
        ExprKind::Call(id, _) => {
            let name = program.fns.get(*id).map(|f| f.name.as_str()).unwrap_or("?");
            format!("expr:Call({name})")
        }
        ExprKind::Builtin(b, _) => format!("expr:Builtin({b:?})"),
        ExprKind::Mutate(m, _, _) => format!("expr:Mutate({m:?})"),
        ExprKind::Cast(c, _) => format!("expr:Cast({c:?})"),
        ExprKind::Binary(op, _, _) => format!("expr:Binary({op:?})"),
        ExprKind::Unary(op, _) => format!("expr:Unary({op:?})"),
        _ => format!("expr:{}", expr_tag_short(expr)),
    }
}
