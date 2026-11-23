use super::compile::Compiler;
use super::ir::*;
use crate::value::{Array, ArrayData, TupleVal, Value};
use crate::{trap, Res, LENT};
use cpl_diag::Span;
use cpl_ir::{AssignOp, BinOp, Block, Expr, ExprKind, Iter, Place, PlaceKind, Program, Stmt, Ty};
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

pub(super) fn block_has_by_ref_closure(block: &Block, _program: &Program) -> bool {
    Compiler::stmts_have_closure(&block.stmts)
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
            let fs = (0..len as usize)
                .map(|k| kinds[k].box_val(regs_i[b as usize + k]))
                .collect();
            Value::Tuple(TupleVal::Boxed(Rc::new(fs)))
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
        if undeclared(traced, loc, &stack[base + slot]) {
            continue;
        }
        // The tree-walker, which a fallback hands the value to, may have
        // stopped with a trap before handing it back.
        if traced && matches!(*loc, SlotLoc::V(r) if matches!(regs_v[r as usize], Value::Unit)) {
            continue;
        }
        stack[base + slot] = match *loc {
            SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
            _ => read_slot(loc, regs_i, regs_v),
        };
    }
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

pub(super) fn sync_from_stack(
    slot_map: &[SlotLoc],
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        if undeclared(traced, loc, &stack[base + slot]) {
            continue;
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
}

#[inline(always)]
pub(super) fn read_array_i(arr: &Array, at: usize) -> i64 {
    unsafe {
        match &arr.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => *v.get_unchecked(at),
            ArrayData::Bool(v) | ArrayData::Char(v) => i64::from(*v.get_unchecked(at)),
            ArrayData::Values(v) => ScalarKind::unbox_val(v.get_unchecked(at)),
            ArrayData::OptInt(v) => v.get_unchecked(at).unwrap_or(0),
            ArrayData::Float(_) | ArrayData::SmallIntTuple { .. } => 0,
        }
    }
}

#[inline(always)]
pub(super) fn write_array_i(arr: &mut Array, at: usize, val: i64) {
    unsafe {
        match &mut arr.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => *v.get_unchecked_mut(at) = val,
            ArrayData::Bool(v) => *v.get_unchecked_mut(at) = u8::from(val != 0),
            ArrayData::Char(v) => *v.get_unchecked_mut(at) = val as u8,
            ArrayData::Values(v) => {
                // Keeps the element's shape: a `[D]Mod<M>` built boxed holds structs.
                match v.get_unchecked_mut(at) {
                    Value::Tuple(TupleVal::OneInt(x)) => *x = val,
                    Value::U64(x) => *x = val as u64,
                    Value::Bool(x) => *x = val != 0,
                    Value::Char(x) => *x = val as u8,
                    slot => *slot = Value::Int(val),
                }
            }
            ArrayData::OptInt(v) => *v.get_unchecked_mut(at) = Some(val),
            ArrayData::Float(_) | ArrayData::SmallIntTuple { .. } => {}
        }
    }
}

#[inline(always)]
pub(super) fn read_array_opt_i(arr: &Array, at: usize) -> (i64, i64) {
    unsafe {
        match &arr.data {
            ArrayData::OptInt(v) => match *v.get_unchecked(at) {
                Some(x) => (x, 1),
                None => (0, 0),
            },
            ArrayData::Values(v) => ScalarKind::unbox_opt(v.get_unchecked(at)),
            _ => (0, 0),
        }
    }
}

#[inline(always)]
pub(super) fn write_array_opt_i(arr: &mut Array, at: usize, val: i64, some: i64, kind: ScalarKind) {
    match &mut arr.data {
        ArrayData::OptInt(v) if kind == ScalarKind::Int => unsafe {
            *v.get_unchecked_mut(at) = if some != 0 { Some(val) } else { None };
        },
        ArrayData::Values(v) => unsafe {
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

/// Where a traced `for` reports its variable, when its iterable is [`quiet`].
pub(super) fn quiet_iter(iter: &Iter) -> Option<Span> {
    match iter {
        Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
        Iter::Range { lo, hi, step, .. } => {
            (quiet(lo) && quiet(hi) && step.as_ref().is_none_or(quiet)).then_some(lo.span)
        }
        Iter::RangeVal(e, _) | Iter::Seq(e, _) => quiet(e).then_some(e.span),
        Iter::TreeRange { .. } => None,
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

#[cfg_attr(target_arch = "wasm32", inline(never))]
pub(super) fn exec_bin_u64(a_raw: i64, b_raw: i64, op: BinOp, span: Span) -> Res<i64> {
    let av = a_raw as u64;
    let bv = b_raw as u64;
    let out = match op {
        BinOp::Add => av.wrapping_add(bv),
        BinOp::Sub => av.wrapping_sub(bv),
        BinOp::Mul => av.wrapping_mul(bv),
        BinOp::Div => {
            if bv == 0 {
                return trap(span, format!("division by zero: `{av} / 0`"));
            }
            av / bv
        }
        BinOp::Rem => {
            if bv == 0 {
                return trap(span, format!("division by zero: `{av} % 0`"));
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
                return trap(span, format!("a shift by {b_raw} is outside 0..64"));
            }
            av << b_raw
        }
        BinOp::Shr => {
            if !(0..64).contains(&b_raw) {
                return trap(span, format!("a shift by {b_raw} is outside 0..64"));
            }
            av >> b_raw
        }
        _ => return trap(span, "internal: unsupported U64 op"),
    };
    Ok(out as i64)
}

#[inline(never)]
/// The elements of a list or array in the stable order of `keys`, one
/// per element; `desc` puts larger keys first and keeps ties in order.
pub(super) fn sort_by_keys(seq: Value, keys: &Value, desc: bool, span: Span) -> Res<Value> {
    let Value::List(keys) = keys else {
        return trap(span, "internal: expected the sort keys");
    };
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by(|&a, &b| {
        let o = keys[a].cmp(&keys[b]);
        if desc {
            o.reverse()
        } else {
            o
        }
    });
    match seq {
        Value::List(items) => Ok(Value::List(Rc::new(
            order.iter().map(|&k| items[k].clone()).collect(),
        ))),
        Value::Array(arr) => {
            let values = order.iter().map(|&k| arr.get_unchecked_val(k)).collect();
            Ok(Value::Array(Rc::new(Array::from_values(
                arr.dims.clone(),
                values,
            ))))
        }
        _ => trap(span, LENT),
    }
}

pub(super) fn exec_sort_place(target: &mut Value, desc: bool, span: Span) -> Res<()> {
    fn sort_ord<T: Ord>(v: &mut [T], desc: bool) {
        v.sort_unstable();
        if desc {
            v.reverse();
        }
    }
    match target {
        Value::List(items) => sort_ord(Rc::make_mut(items).as_mut_slice(), desc),
        Value::Str(bytes) => sort_ord(Rc::make_mut(bytes).as_mut_slice(), desc),
        Value::Array(arr) => {
            let a = Rc::make_mut(arr);
            match &mut a.data {
                ArrayData::Int(v) | ArrayData::OneIntStruct(v) => sort_ord(v, desc),
                ArrayData::OptInt(v) => sort_ord(v, desc),
                ArrayData::Bool(v) => sort_ord(v, desc),
                ArrayData::Char(v) => sort_ord(v, desc),
                ArrayData::Values(v) => sort_ord(v, desc),
                ArrayData::Float(v) => {
                    v.sort_unstable_by(f64::total_cmp);
                    if desc {
                        v.reverse();
                    }
                }
                ArrayData::SmallIntTuple { .. } => {
                    let mut boxed: Vec<Value> =
                        (0..a.len()).map(|i| a.get_unchecked_val(i)).collect();
                    sort_ord(&mut boxed, desc);
                    for (i, val) in boxed.into_iter().enumerate() {
                        a.set(i, val);
                    }
                }
            }
        }
        _ => return trap(span, "internal: cannot sort this value"),
    }
    Ok(())
}

pub(super) enum ExecExit {
    Return(Value),
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
}

pub(super) fn store_fed_val(ctx: &mut FedCtx<'_, '_>, vm_regs_i: &mut [i64], val: Value) {
    let regs_i = &mut vm_regs_i[ctx.caller_base_i..];
    write_slot(&ctx.var_loc, regs_i, ctx.caller_regs_v, val);
}

/// Puts `val` in the local at `loc`: [`read_slot`]'s inverse.
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
        return trap(span, LENT);
    }
    let empty = match &*cell {
        Value::List(_) => Value::List(Rc::default()),
        Value::Str(_) => Value::Str(Rc::default()),
        Value::Map(_) => Value::Map(Rc::default()),
        Value::Set(_) => Value::Set(Rc::default()),
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
        PlaceStep::Key(_) => {
            unreachable!("a path ends at a map entry only for an op that walks it whole")
        }
        PlaceStep::Index1(r_i) => {
            let idx = regs_i[r_i as usize];
            let len = arr.dims[0];
            if idx < 0 || idx >= len {
                return trap(
                    span,
                    format!("index {idx} is outside an array dimension of length {len}"),
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
                    format!("index {i} is outside an array dimension of length {d0}"),
                );
            }
            if j < 0 || j >= d1 {
                return trap(
                    span,
                    format!("index {j} is outside an array dimension of length {d1}"),
                );
            }
            Ok((i * d1 + j) as usize)
        }
        PlaceStep::IndexN(idx, n) => {
            nd_flat_index(&arr.dims, &regs_i[idx as usize..(idx + n) as usize], span)
        }
        PlaceStep::Field(_) => trap(span, "internal: expected index step"),
    }
}

/// The row-major cell of `indices` in an array of `dims`, each checked.
pub(super) fn nd_flat_index(dims: &[i64], indices: &[i64], span: Span) -> Res<usize> {
    let mut flat = 0;
    for (&i, &d) in indices.iter().zip(dims) {
        if i < 0 || i >= d {
            return trap(
                span,
                format!("index {i} is outside an array dimension of length {d}"),
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
            if !matches!(arr.data, ArrayData::Values(_)) {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                let mut cell = a.get(flat).expect("index checked");
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
                Value::Map(entries) => match Rc::make_mut(entries).get_mut(&keys[n as usize]) {
                    Some(value) => cur = value,
                    None => {
                        return trap(
                            span,
                            "this key is not in the map; store with `m[k] = v` first",
                        );
                    }
                },
                _ => return trap(span, "internal: expected map"),
            },
            PlaceStep::Field(f) => match cur {
                Value::Tuple(TupleVal::Boxed(fs)) => {
                    cur = &mut Rc::make_mut(fs)[f as usize];
                }
                _ => return trap(span, "internal: expected tuple"),
            },
            PlaceStep::Index1(r_i) => match cur {
                Value::List(items) => {
                    let idx = regs_i[r_i as usize];
                    let len = items.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            span,
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    cur = &mut Rc::make_mut(items)[idx as usize];
                }
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => cur = &mut vs[flat],
                        _ => return trap(span, "internal: expected boxed array element"),
                    }
                }
                _ => return trap(span, "internal: expected array or list"),
            },
            PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match cur {
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => cur = &mut vs[flat],
                        _ => return trap(span, "internal: expected boxed array element"),
                    }
                }
                _ => return trap(span, "internal: expected array"),
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
        PlaceStep::Key(_) => {
            unreachable!("a path ends at a map entry only for an op that walks it whole")
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = kind.box_val(raw);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = raw;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v[flat] = raw,
                    ArrayData::Bool(v) => v[flat] = u8::from(raw != 0),
                    ArrayData::Char(v) => v[flat] = raw as u8,
                    ArrayData::Values(v) => v[flat] = kind.box_val(raw),
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
                        format!("index {idx} is outside a list of length {len}"),
                    );
                }
                Rc::make_mut(items)[idx as usize] = kind.box_val(raw);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v[flat] = raw,
                    ArrayData::Bool(v) => v[flat] = u8::from(raw != 0),
                    ArrayData::Char(v) => v[flat] = raw as u8,
                    ArrayData::Values(v) => v[flat] = kind.box_val(raw),
                    _ => a.set(flat, kind.box_val(raw)),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
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
        PlaceStep::Key(_) => {
            unreachable!("a path ends at a map entry only for an op that walks it whole")
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = kind.box_opt(raw, some);
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    ArrayData::Values(v) => v[flat] = kind.box_opt(raw, some),
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
                        format!("index {idx} is outside a list of length {len}"),
                    );
                }
                Rc::make_mut(items)[idx as usize] = kind.box_opt(raw, some);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    ArrayData::Values(v) => v[flat] = kind.box_opt(raw, some),
                    _ => a.set(flat, kind.box_opt(raw, some)),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
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
        PlaceStep::Key(_) => {
            unreachable!("a path ends at a map entry only for an op that walks it whole")
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = val;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
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
                        format!("index {idx} is outside a list of length {len}"),
                    );
                }
                Rc::make_mut(items)[idx as usize] = val;
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                Rc::make_mut(arr).set(flat, val);
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
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
                    return trap(span, format!("division by zero: `{a} / 0`"));
                }
                a / b
            }
            AssignOp::Binary(BinOp::Rem) => {
                if b == 0 {
                    return trap(span, format!("division by zero: `{a} % 0`"));
                }
                a % b
            }
            AssignOp::Binary(BinOp::BitAnd) => a & b,
            AssignOp::Binary(BinOp::BitOr) => a | b,
            AssignOp::Binary(BinOp::BitXor) => a ^ b,
            AssignOp::Binary(BinOp::Shl) => {
                if !(0..64).contains(&rhs) {
                    return trap(span, format!("a shift by {rhs} is outside 0..64"));
                }
                a << rhs
            }
            AssignOp::Binary(BinOp::Shr) => {
                if !(0..64).contains(&rhs) {
                    return trap(span, format!("a shift by {rhs} is outside 0..64"));
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
            None => {
                return trap(
                    span,
                    format!("integer overflow: `{cur} + {rhs}` does not fit in an `Int` (64 bits)"),
                )
            }
        },
        AssignOp::Binary(BinOp::Sub) => match cur.checked_sub(rhs) {
            Some(v) => v,
            None => {
                return trap(
                    span,
                    format!("integer overflow: `{cur} - {rhs}` does not fit in an `Int` (64 bits)"),
                )
            }
        },
        AssignOp::Binary(BinOp::Mul) => match cur.checked_mul(rhs) {
            Some(v) => v,
            None => {
                return trap(
                    span,
                    format!("integer overflow: `{cur} * {rhs}` does not fit in an `Int` (64 bits)"),
                )
            }
        },
        AssignOp::Binary(BinOp::Div) => {
            if rhs == 0 {
                return trap(span, format!("division by zero: `{cur} / 0`"));
            }
            match cur.checked_div_euclid(rhs) {
                Some(v) => v,
                None => {
                    return trap(
                        span,
                        format!(
                            "integer overflow: `{cur} / {rhs}` does not fit in an `Int` (64 bits)"
                        ),
                    )
                }
            }
        }
        AssignOp::Binary(BinOp::Rem) => {
            if rhs == 0 {
                return trap(span, format!("division by zero: `{cur} % 0`"));
            }
            match cur.checked_rem_euclid(rhs) {
                Some(v) => v,
                None => {
                    return trap(
                        span,
                        format!(
                            "integer overflow: `{cur} % {rhs}` does not fit in an `Int` (64 bits)"
                        ),
                    )
                }
            }
        }
        AssignOp::Binary(BinOp::BitAnd) => cur & rhs,
        AssignOp::Binary(BinOp::BitOr) => cur | rhs,
        AssignOp::Binary(BinOp::BitXor) => cur ^ rhs,
        AssignOp::Binary(BinOp::Shl) => {
            if !(0..64).contains(&rhs) {
                return trap(span, format!("a shift by {rhs} is outside 0..64"));
            }
            cur.wrapping_shl(rhs as u32)
        }
        AssignOp::Binary(BinOp::Shr) => {
            if !(0..64).contains(&rhs) {
                return trap(span, format!("a shift by {rhs} is outside 0..64"));
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
        PlaceStep::Key(_) => {
            unreachable!("a path ends at a map entry only for an op that walks it whole")
        }
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                let slice = Rc::make_mut(fs);
                let cur = ScalarKind::unbox_val(&slice[f as usize]);
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice[f as usize] = kind.box_val(next);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = apply_scalar_assign_op(*x, rhs, op, kind, span)?;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    ArrayData::Values(v) => {
                        let cur = ScalarKind::unbox_val(&v[flat]);
                        let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                        v[flat] = kind.box_val(next);
                    }
                    _ => return trap(span, "internal: unsupported array assign"),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(
                        span,
                        format!("index {idx} is outside a list of length {len}"),
                    );
                }
                let slice = Rc::make_mut(items);
                let cur = ScalarKind::unbox_val(&slice[idx as usize]);
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice[idx as usize] = kind.box_val(next);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) | PlaceStep::IndexN(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    ArrayData::Values(v) => {
                        let cur = ScalarKind::unbox_val(&v[flat]);
                        let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                        v[flat] = kind.box_val(next);
                    }
                    _ => return trap(span, "internal: unsupported array assign"),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
        },
    }
}

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

impl VmStats {
    pub(crate) fn from_env() -> Option<Self> {
        let val = std::env::var_os("CPL_VM_STATS")?;
        if val.is_empty() || val == "0" {
            return None;
        }
        Some(Self::default())
    }

    pub(crate) fn record(&mut self, tag: String) {
        self.record_with_loop(tag, false);
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
        let mut entries: Vec<(&str, u64)> =
            self.counts.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        let mut loop_entries: Vec<(&str, u64)> = self
            .loop_counts
            .iter()
            .map(|(k, &v)| (k.as_str(), v))
            .collect();
        loop_entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        if std::env::var("CPL_VM_STATS").ok().as_deref() == Some("json") {
            let mut s = format!(
                "{{\"total_fallbacks\":{},\"loop_fallbacks\":{},\"hot_loop_fallbacks\":{},\"walked\":{},\"top\":[",
                self.total_fallbacks, self.loop_fallbacks, self.hot_loop_fallbacks, self.walked
            );
            for (i, (k, v)) in entries.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                let escaped = cpl_json::escape_string(k);
                s.push_str(&format!("[{escaped},{v}]"));
            }
            s.push_str("],\"loop_top\":[");
            for (i, (k, v)) in loop_entries.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                let escaped = cpl_json::escape_string(k);
                s.push_str(&format!("[{escaped},{v}]"));
            }
            let mut walked: Vec<_> = self.walked_counts.iter().collect();
            walked.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            s.push_str("],\"walked_top\":[");
            for (i, (k, v)) in walked.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("[{},{v}]", cpl_json::escape_string(k)));
            }
            s.push_str("],\"kinds\":[");
            for (i, (k, _)) in entries.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&cpl_json::escape_string(k));
            }
            s.push_str("]}");
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
        ExprKind::Parse(..) => "Parse",
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
        _ => "Other",
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
        _ => {
            let s = expr_tag_short(expr);
            if s == "Other" {
                let dbg = format!("{:?}", expr.kind);
                let head = dbg.split(['(', '{', ' ']).next().unwrap_or("Other");
                format!("expr:{head}")
            } else {
                format!("expr:{s}")
            }
        }
    }
}
