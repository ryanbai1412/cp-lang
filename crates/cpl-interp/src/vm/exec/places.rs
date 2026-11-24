use crate::checkpoint::Body;
use crate::observe::Observer;
use crate::value::{ArrayData, CaptureVal, OptVal, TupleVal, Value};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{trap, Flow, Interp, Res};
use cpl_ir::{AssignOp, BinOp, CmpOp, LoopId};
use std::rc::Rc;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(super) fn exec_places_and_fallback(
        &mut self,
        op: Op,
        cur_pc: usize,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
        escaped: &mut Option<(bool, Option<LoopId>)>,
        fed: &mut Option<&mut FedCtx<'_, 'p>>,
    ) -> Res<Option<ExecExit>> {
        let spans = chunk.spans.as_slice();
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        match op {
            Op::LStrLen { dst, str_v } => {
                regs_i[dst as usize] = match &regs_v[str_v as usize] {
                    Value::Str(bytes) => bytes.len() as i64,
                    Value::StrView(view) => view.len as i64,
                    _ => 0,
                };
            }
            Op::BinU64 { dst, a, b, op } => {
                regs_i[dst as usize] =
                    exec_bin_u64(regs_i[a as usize], regs_i[b as usize], op, spans[cur_pc])?;
            }
            Op::NegU64 { dst, a } => {
                regs_i[dst as usize] = (regs_i[a as usize] as u64).wrapping_neg() as i64;
            }
            Op::BitAndU64 { dst, a, b } => {
                regs_i[dst as usize] = regs_i[a as usize] & regs_i[b as usize];
            }
            Op::BitOrU64 { dst, a, b } => {
                regs_i[dst as usize] = regs_i[a as usize] | regs_i[b as usize];
            }
            Op::BitXorU64 { dst, a, b } => {
                regs_i[dst as usize] = regs_i[a as usize] ^ regs_i[b as usize];
            }
            Op::BitNotI { dst, a } => {
                regs_i[dst as usize] = !regs_i[a as usize];
            }
            Op::CmpU64 { dst, a, b, op } => {
                let av = regs_i[a as usize] as u64;
                let bv = regs_i[b as usize] as u64;
                let res = match op {
                    CmpOp::Eq => av == bv,
                    CmpOp::Ne => av != bv,
                    CmpOp::Lt => av < bv,
                    CmpOp::Le => av <= bv,
                    CmpOp::Gt => av > bv,
                    CmpOp::Ge => av >= bv,
                    _ => false,
                };
                regs_i[dst as usize] = i64::from(res);
            }
            Op::U64ToInt { dst, src } => {
                let v = regs_i[src as usize] as u64;
                match i64::try_from(v) {
                    Ok(iv) => regs_i[dst as usize] = iv,
                    Err(_) => {
                        return trap(spans[cur_pc], format!("{v} does not fit in an `Int`"));
                    }
                }
            }
            Op::StorePlaceI { path, r_val, kind } => {
                let pp = &chunk.places[path as usize];
                let raw = regs_i[r_val as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                let container = walk_place_mut(root, prefix, regs_i, &keys, spans[cur_pc])?;
                store_place_leaf_scalar(container, last, raw, kind, regs_i, spans[cur_pc])?;
            }
            Op::StorePlaceOptI {
                path,
                r_val,
                r_some,
                kind,
            } => {
                let pp = &chunk.places[path as usize];
                let raw = regs_i[r_val as usize];
                let some = regs_i[r_some as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                let container = walk_place_mut(root, prefix, regs_i, &keys, spans[cur_pc])?;
                store_place_leaf_opt_scalar(
                    container,
                    last,
                    raw,
                    some,
                    kind,
                    regs_i,
                    spans[cur_pc],
                )?;
            }
            Op::StorePlaceV { path, r_val } => {
                let pp = &chunk.places[path as usize];
                let val = regs_v[r_val as usize].clone();
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                let container = walk_place_mut(root, prefix, regs_i, &keys, spans[cur_pc])?;
                store_place_leaf_val(container, last, val, regs_i, spans[cur_pc])?;
            }
            Op::AssignPlaceI {
                path,
                r_rhs,
                op,
                kind,
            } => {
                let pp = &chunk.places[path as usize];
                let rhs = regs_i[r_rhs as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                let container = walk_place_mut(root, prefix, regs_i, &keys, spans[cur_pc])?;
                assign_place_leaf_scalar(container, last, rhs, op, kind, regs_i, spans[cur_pc])?;
            }
            Op::Fail { idx, msg_v } => {
                let message = (msg_v != u16::MAX)
                    .then(|| std::mem::replace(&mut regs_v[msg_v as usize], Value::Unit));
                return Err(crate::stmt_failure(chunk.fails[idx as usize], message));
            }
            Op::AssignPlaceV { path, r_val, op } => {
                let pp = &chunk.places[path as usize];
                let val = regs_v[r_val as usize].clone();
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let span = spans[cur_pc];
                with_place_cell(root, &pp.steps, regs_i, &keys, span, |cell| {
                    combine_in_place(cell, op, val, span)
                })?;
            }
            Op::ListPopDiscardPlace { path } => {
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                match target {
                    Value::List(items) => {
                        Rc::make_mut(items).pop();
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                }
            }
            Op::ListPushPlaceI { path, val_i, kind } => {
                let pp = &chunk.places[path as usize];
                let val = kind.box_val(regs_i[val_i as usize]);
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                match target {
                    Value::List(items) => {
                        Rc::make_mut(items).push(val);
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                }
            }
            Op::ListPushPlaceV { path, val_v } => {
                let pp = &chunk.places[path as usize];
                let val = regs_v[val_v as usize].clone();
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                match target {
                    Value::List(items) => {
                        Rc::make_mut(items).push(val);
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                }
            }
            Op::SortByKeysPlace {
                path,
                seq_v,
                keys_v,
                desc_r,
            } => {
                let desc = desc_r != u16::MAX && regs_i[desc_r as usize] != 0;
                let seq = std::mem::replace(&mut regs_v[seq_v as usize], Value::Unit);
                let by = std::mem::replace(&mut regs_v[keys_v as usize], Value::Unit);
                let sorted = sort_by_keys(seq, &by, desc, spans[cur_pc])?;
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                *walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])? = sorted;
            }
            Op::SortPlace { path, desc_r } => {
                let pp = &chunk.places[path as usize];
                let desc = desc_r != u16::MAX && regs_i[desc_r as usize] != 0;
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                exec_sort_place(target, desc, spans[cur_pc])?;
            }
            Op::RangeCheckI {
                dst,
                val,
                lo,
                hi,
                text_idx,
            } => {
                let v = regs_i[val as usize];
                let lo_v = regs_i[lo as usize];
                let hi_v = regs_i[hi as usize];
                if v < lo_v || v > hi_v {
                    let text = chunk.range_texts[text_idx as usize];
                    return trap(spans[cur_pc], format!("{v} is outside `{text}`"));
                }
                regs_i[dst as usize] = v;
            }
            Op::ConstUnit { dst } => {
                regs_v[dst as usize] = Value::Unit;
            }
            Op::SetMinMaxV { dst, src_v, is_max } => {
                regs_v[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Set(items) => {
                        let item = if is_max { items.last() } else { items.first() };
                        Value::Opt(OptVal::from_option(item.cloned()))
                    }
                    Value::Map(entries) => {
                        let entry = if is_max {
                            entries.iter().next_back()
                        } else {
                            entries.iter().next()
                        };
                        Value::Opt(OptVal::from_option(
                            entry.map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                        ))
                    }
                    _ => Value::Opt(OptVal::None),
                };
            }
            Op::ReversePlace { path } => {
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                match target {
                    Value::List(items) => Rc::make_mut(items).reverse(),
                    Value::Array(arr) => Rc::make_mut(arr).reverse(),
                    Value::Str(bytes) => Rc::make_mut(bytes).reverse(),
                    _ => return trap(spans[cur_pc], "internal: cannot reverse this value"),
                }
            }
            Op::BitWidthI { dst, src } => {
                regs_i[dst as usize] =
                    i64::from(64 - (regs_i[src as usize] as u64).leading_zeros());
            }
            Op::IntToFloat { dst, src } => {
                regs_v[dst as usize] = Value::Float(regs_i[src as usize] as f64);
            }
            Op::FloatSqrt { dst, src } => {
                let x = regs_v[src as usize].float();
                if x < 0.0 {
                    return trap(spans[cur_pc], format!("sqrt({x}) of a negative number"));
                }
                regs_v[dst as usize] = Value::Float(x.sqrt());
            }
            Op::ContainsV { dst, src_v, key_v } => {
                let needle = &regs_v[key_v as usize];
                let found = match &regs_v[src_v as usize] {
                    Value::Set(items) => items.contains(needle),
                    Value::Map(entries) => entries.contains_key(needle),
                    Value::List(items) => items.contains(needle),
                    Value::ListView(view) => view.contains(needle),
                    Value::Array(arr) => arr.contains(needle),
                    _ => false,
                };
                regs_i[dst as usize] = i64::from(found);
            }
            Op::MapGetV { dst, map_v, key_v } => {
                let found = match &regs_v[map_v as usize] {
                    Value::Map(entries) => entries.get(&regs_v[key_v as usize]).cloned(),
                    _ => None,
                };
                regs_v[dst as usize] = Value::Opt(OptVal::from_option(found));
            }
            Op::MapIndexV {
                dst,
                map_v,
                key_v,
                key_ty,
            } => {
                let found = match &regs_v[map_v as usize] {
                    Value::Map(entries) => entries.get(&regs_v[key_v as usize]).cloned(),
                    _ => None,
                };
                if let Some(v) = found {
                    regs_v[dst as usize] = v;
                } else {
                    let mut key = Vec::new();
                    crate::show(
                        &mut key,
                        &regs_v[key_v as usize],
                        chunk.fill_tys[key_ty as usize],
                        6,
                        self.program,
                        &self.domains,
                    );
                    let key: String = key.iter().map(|b| *b as char).collect();
                    return trap(
                        spans[cur_pc],
                        format!("the key `{key}` is not in the map; `m.get(k)` gives an optional"),
                    );
                }
            }
            Op::SetMapMutPlace {
                dst,
                path,
                val_v,
                is_remove,
            } => {
                let arg = regs_v[val_v as usize].clone();
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                let ok = match (is_remove, target) {
                    (false, Value::Set(items)) => Rc::make_mut(items).insert(arg),
                    (true, Value::Set(items)) => Rc::make_mut(items).remove(&arg),
                    (true, Value::Map(entries)) => Rc::make_mut(entries).remove(&arg).is_some(),
                    _ => {
                        return trap(
                            spans[cur_pc],
                            "internal: this value can't be changed that way",
                        );
                    }
                };
                regs_i[dst as usize] = i64::from(ok);
            }
            Op::MapPlaceAssign {
                path,
                key_v,
                val_v,
                op,
                val_ty,
            } => {
                let key = regs_v[key_v as usize].clone();
                let value = regs_v[val_v as usize].clone();
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                if let Value::Map(entries) = target {
                    let map = Rc::make_mut(entries);
                    if op == AssignOp::Set {
                        map.insert(key, value);
                    } else {
                        let next = if let Some(old) = map.get(&key).cloned() {
                            crate::combine(op, old, value, spans[cur_pc])?
                        } else {
                            match op {
                                AssignOp::Max
                                | AssignOp::Min
                                | AssignOp::Binary(
                                    BinOp::Add
                                    | BinOp::Mul
                                    | BinOp::BitOr
                                    | BinOp::BitXor
                                    | BinOp::BitAnd,
                                ) => value,
                                AssignOp::Binary(BinOp::Sub) => crate::combine(
                                    op,
                                    crate::zero(chunk.fill_tys[val_ty as usize], self.program),
                                    value,
                                    spans[cur_pc],
                                )?,
                                _ => {
                                    return trap(
                                        spans[cur_pc],
                                        "this key is not in the map; store with `m[k] = v` first",
                                    );
                                }
                            }
                        };
                        map.insert(key, next);
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected map");
                }
            }
            Op::SetMapBoundV {
                dst,
                src_v,
                key_v,
                mode,
            } => {
                use std::ops::Bound::{Excluded, Included, Unbounded};
                let key = regs_v[key_v as usize].clone();
                let range = match mode {
                    0 => (Included(key), Unbounded),
                    1 => (Excluded(key), Unbounded),
                    2 => (Unbounded, Included(key)),
                    _ => (Unbounded, Excluded(key)),
                };
                let first_in = mode <= 1;
                regs_v[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Set(items) => {
                        let mut found = items.range(range);
                        let item = if first_in {
                            found.next()
                        } else {
                            found.next_back()
                        };
                        Value::Opt(OptVal::from_option(item.cloned()))
                    }
                    Value::Map(entries) => {
                        let mut found = entries.range(range);
                        let entry = if first_in {
                            found.next()
                        } else {
                            found.next_back()
                        };
                        Value::Opt(OptVal::from_option(
                            entry.map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                        ))
                    }
                    _ => Value::Opt(OptVal::None),
                };
            }
            Op::SetMapPopEndPlace { dst, path, is_last } => {
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let target = walk_place_mut(root, &pp.steps, regs_i, &keys, spans[cur_pc])?;
                regs_v[dst as usize] = match target {
                    Value::Set(items) => {
                        let items = Rc::make_mut(items);
                        let v = if is_last {
                            items.pop_last()
                        } else {
                            items.pop_first()
                        };
                        Value::Opt(OptVal::from_option(v))
                    }
                    Value::Map(entries) => {
                        let entries = Rc::make_mut(entries);
                        let v = if is_last {
                            entries.pop_last()
                        } else {
                            entries.pop_first()
                        };
                        Value::Opt(OptVal::from_option(
                            v.map(|(k, v)| Value::tuple(vec![k, v])),
                        ))
                    }
                    _ => {
                        return trap(
                            spans[cur_pc],
                            "internal: this value can't be changed that way",
                        );
                    }
                };
            }
            Op::LoadList1SmallTuple {
                dst_base,
                len,
                root,
                is_global,
                r_i,
            } => {
                let idx = regs_i[r_i as usize];
                let list_val = if is_global {
                    &self.globals[root as usize]
                } else {
                    &regs_v[root as usize]
                };
                let elem = match list_val {
                    Value::List(list) => {
                        let n_len = list.len() as i64;
                        if idx < 0 || idx >= n_len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {n_len}"),
                            );
                        }
                        &list[idx as usize]
                    }
                    Value::ListView(view) => {
                        let n_len = view.len as i64;
                        if idx < 0 || idx >= n_len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {n_len}"),
                            );
                        }
                        view.get(idx as usize).unwrap()
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                let b = dst_base as usize;
                let n = len as usize;
                if let Value::Tuple(TupleVal::Boxed(fs)) = elem {
                    for (k, f) in fs.iter().take(n).enumerate() {
                        regs_i[b + k] = ScalarKind::unbox_val(f);
                    }
                }
            }
            Op::MinMaxArrOptI {
                root,
                is_global,
                is_2d,
                is_max,
                r_i,
                r_j,
                r_val,
            } => {
                let i0 = regs_i[r_i as usize];
                let val = regs_i[r_val as usize];
                let target = if is_global {
                    &mut self.globals[root as usize]
                } else {
                    &mut regs_v[root as usize]
                };
                if let Value::Array(arr) = target {
                    if is_2d && arr.dims.len() == 1 {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array of length {l0}"),
                            );
                        }
                        let i1 = regs_i[r_j as usize];
                        let arr_mut = Rc::make_mut(arr);
                        let ArrayData::Values(rows) = &mut arr_mut.data else {
                            return trap(spans[cur_pc], "internal: expected nested array");
                        };
                        let Value::Array(row_arr) = &mut rows[i0 as usize] else {
                            return trap(spans[cur_pc], "internal: expected inner array");
                        };
                        let l1 = row_arr.dims[0];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array of length {l1}"),
                            );
                        }
                        let row_mut = Rc::make_mut(row_arr);
                        match &mut row_mut.data {
                            ArrayData::OptInt(v) => {
                                let slot = unsafe { v.get_unchecked_mut(i1 as usize) };
                                let replace = match *slot {
                                    None => true,
                                    Some(cur) => {
                                        if is_max {
                                            val > cur
                                        } else {
                                            val < cur
                                        }
                                    }
                                };
                                if replace {
                                    *slot = Some(val);
                                }
                            }
                            _ => {
                                let (cur, some) = read_array_opt_i(row_mut, i1 as usize);
                                if some == 0 || (if is_max { val > cur } else { val < cur }) {
                                    write_array_opt_i(
                                        row_mut,
                                        i1 as usize,
                                        val,
                                        1,
                                        ScalarKind::Int,
                                    );
                                }
                            }
                        }
                        return Ok(None);
                    }
                    let flat = if is_2d {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let i1 = regs_i[r_j as usize];
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        (i0 * l1 + i1) as usize
                    } else {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array of length {l0}"),
                            );
                        }
                        i0 as usize
                    };
                    let arr_mut = Rc::make_mut(arr);
                    match &mut arr_mut.data {
                        ArrayData::OptInt(v) => {
                            let slot = unsafe { v.get_unchecked_mut(flat) };
                            let replace = match *slot {
                                None => true,
                                Some(cur) => {
                                    if is_max {
                                        val > cur
                                    } else {
                                        val < cur
                                    }
                                }
                            };
                            if replace {
                                *slot = Some(val);
                            }
                        }
                        _ => {
                            let (cur, some) = read_array_opt_i(arr_mut, flat);
                            if some == 0 || (if is_max { val > cur } else { val < cur }) {
                                write_array_opt_i(arr_mut, flat, val, 1, ScalarKind::Int);
                            }
                        }
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::LoadIdxFieldIdxOptI {
                dst_val,
                dst_some,
                root,
                is_global,
                r_i,
                field,
                r_j,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let seq_val = if is_global {
                    &self.globals[root as usize]
                } else {
                    &regs_v[root as usize]
                };
                let elem = match seq_val {
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside a list of length {len}"),
                            );
                        }
                        &list[i as usize]
                    }
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {len}"),
                            );
                        }
                        match &arr.data {
                            ArrayData::Values(v) => &v[i as usize],
                            _ => return trap(spans[cur_pc], "internal: expected struct element"),
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list or array"),
                };
                let field_val = match elem {
                    Value::Tuple(TupleVal::Boxed(fs)) => &fs[field as usize],
                    _ => return trap(spans[cur_pc], "internal: expected struct/tuple"),
                };
                let (v, s) = match field_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if j < 0 || j >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array of length {len}"),
                            );
                        }
                        read_array_opt_i(arr, j as usize)
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if j < 0 || j >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside a list of length {len}"),
                            );
                        }
                        match &list[j as usize] {
                            Value::Opt(OptVal::Int(n)) => (*n, 1),
                            Value::Opt(OptVal::None) => (0, 0),
                            Value::Opt(OptVal::Boxed(b)) => (ScalarKind::unbox_val(b), 1),
                            _ => (0, 0),
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list field"),
                };
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
            }
            Op::LoadNestedArrOptI {
                dst_val,
                dst_some,
                root,
                is_global,
                r_i,
                r_j,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let outer_val = if is_global {
                    &self.globals[root as usize]
                } else {
                    &regs_v[root as usize]
                };
                let inner_val = match outer_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {len}"),
                            );
                        }
                        match &arr.data {
                            ArrayData::Values(v) => &v[i as usize],
                            _ => return trap(spans[cur_pc], "internal: expected nested array"),
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside a list of length {len}"),
                            );
                        }
                        &list[i as usize]
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                };
                let (v, s) = match inner_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if j < 0 || j >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array of length {len}"),
                            );
                        }
                        read_array_opt_i(arr, j as usize)
                    }
                    _ => return trap(spans[cur_pc], "internal: expected inner array"),
                };
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
            }
            Op::LoadNestedArrI {
                dst,
                root,
                is_global,
                r_i,
                r_j,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let outer_val = if is_global {
                    &self.globals[root as usize]
                } else {
                    &regs_v[root as usize]
                };
                let inner_val = match outer_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {len}"),
                            );
                        }
                        match &arr.data {
                            ArrayData::Values(v) => &v[i as usize],
                            _ => return trap(spans[cur_pc], "internal: expected nested array"),
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside a list of length {len}"),
                            );
                        }
                        &list[i as usize]
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                };
                regs_i[dst as usize] = match inner_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if j < 0 || j >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array of length {len}"),
                            );
                        }
                        read_array_i(arr, j as usize)
                    }
                    _ => return trap(spans[cur_pc], "internal: expected inner array"),
                };
            }
            Op::SeqIdxEndOptI {
                dst_val,
                dst_some,
                root,
                is_global,
                r_i,
                is_last,
            } => {
                let i = regs_i[r_i as usize];
                let outer_val = if is_global {
                    &self.globals[root as usize]
                } else {
                    &regs_v[root as usize]
                };
                let inner_val = match outer_val {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {len}"),
                            );
                        }
                        match &arr.data {
                            ArrayData::Values(v) => &v[i as usize],
                            _ => return trap(spans[cur_pc], "internal: expected sequence element"),
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if i < 0 || i >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside a list of length {len}"),
                            );
                        }
                        &list[i as usize]
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                };
                let (v, s) = match self.seq_end(inner_val, is_last, spans[cur_pc])? {
                    Some(x) => (ScalarKind::unbox_val(&x), 1),
                    None => (0, 0),
                };
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
            }
            Op::MakeClosure { dst, spec } => {
                let cs = &chunk.closure_specs[spec as usize];
                let mut caps = Vec::with_capacity(cs.caps.len());
                for cap in &cs.caps {
                    match *cap {
                        CapSrc::GlobalRef(gid) => {
                            caps.push(CaptureVal::GlobalRef(gid as usize));
                        }
                        CapSrc::GlobalVal(gid) => {
                            caps.push(CaptureVal::Val(self.globals[gid as usize].clone()));
                        }
                        CapSrc::LocalI(r, kind) => {
                            caps.push(CaptureVal::Val(kind.box_val(regs_i[r as usize])));
                        }
                        CapSrc::LocalOptI { val, some, kind } => {
                            caps.push(CaptureVal::Val(
                                kind.box_opt(regs_i[val as usize], regs_i[some as usize]),
                            ));
                        }
                        CapSrc::LocalSmallTuple { base, len, kinds } => {
                            let mut fs = Vec::with_capacity(len as usize);
                            for k in 0..len as usize {
                                fs.push(kinds[k].box_val(regs_i[base as usize + k]));
                            }
                            caps.push(CaptureVal::Val(Value::Tuple(TupleVal::Boxed(Rc::new(fs)))));
                        }
                        CapSrc::LocalV(r) => {
                            caps.push(CaptureVal::Val(regs_v[r as usize].clone()));
                        }
                    }
                }
                regs_v[dst as usize] = Value::Closure(cs.cid as usize, Rc::new(caps));
            }
            Op::TakeLocalV { dst, src } => {
                regs_v[dst as usize] = take_value(&mut regs_v[src as usize], spans[cur_pc])?;
            }
            Op::TakePlaceV { dst, path } => {
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = match pp.root {
                    PlaceRoot::LocalV(v) => &mut regs_v[v as usize],
                    PlaceRoot::Global(gid) => &mut self.globals[gid as usize],
                };
                regs_v[dst as usize] =
                    with_place_cell(root, &pp.steps, regs_i, &keys, spans[cur_pc], |cell| {
                        take_value(cell, spans[cur_pc])
                    })?;
            }
            Op::CallVmFeed { spec } => {
                let fc = &chunk.feed_calls[spec as usize];
                let fed_chunk = compiled.fed_fns[fc.fn_id]
                    .as_ref()
                    .expect("fed_fn compiled");
                let callee_base_i = vm_regs_i.len();
                vm_regs_i.resize(callee_base_i + fed_chunk.num_i, 0);
                let mut callee_regs_v = self.take_regs_v(fed_chunk.num_v);
                {
                    let mut ii = 0usize;
                    let mut iv = 0usize;
                    for loc in fed_chunk.slot_map.iter().take(fed_chunk.param_count) {
                        match *loc {
                            SlotLoc::I(r, _) => {
                                let val = vm_regs_i[base_i + fc.args_i[ii] as usize];
                                vm_regs_i[callee_base_i + r as usize] = val;
                                ii += 1;
                            }
                            SlotLoc::OptI { val, some, .. } => {
                                let v = vm_regs_i[base_i + fc.args_i[ii] as usize];
                                let s = vm_regs_i[base_i + fc.args_i[ii + 1] as usize];
                                vm_regs_i[callee_base_i + val as usize] = v;
                                vm_regs_i[callee_base_i + some as usize] = s;
                                ii += 2;
                            }
                            SlotLoc::V(r) => {
                                callee_regs_v[r as usize] = regs_v[fc.args_v[iv] as usize].clone();
                                iv += 1;
                            }
                            SlotLoc::SmallTuple { .. } => {
                                unreachable!("params are never SmallTuple")
                            }
                        }
                    }
                }
                let f = &self.program.fns[fc.fn_id];
                self.call_frames.push((f.name.as_str(), spans[cur_pc]));
                self.suspend(
                    chunk,
                    cur_pc,
                    base_i,
                    regs_v,
                    Body::Fed(fc.fn_id),
                    self.base,
                );
                let dom_mark = self.saved_domains.len();
                for &d in &fed_chunk.guarded_domains {
                    self.saved_domains.push((d, self.domains[d]));
                }
                let caller_base = self.base;
                let pushed_stack = fed_chunk.needs_stack;
                if pushed_stack {
                    let new_base = self.stack.len();
                    self.stack
                        .resize(new_base + fed_chunk.slot_map.len(), Value::Unit);
                    self.base = new_base;
                }
                let mut ctx = FedCtx {
                    caller_chunk: chunk,
                    caller_base_i: base_i,
                    caller_regs_v: regs_v,
                    caller_base,
                    body_pc: fc.body_pc,
                    var_loc: fc.var_loc,
                };
                let res = self.exec_chunk_core(
                    compiled,
                    fed_chunk,
                    callee_base_i,
                    vm_regs_i,
                    &mut callee_regs_v,
                    0,
                    Some(&mut ctx),
                );
                if pushed_stack {
                    self.stack.truncate(self.base);
                    self.base = caller_base;
                }
                while self.saved_domains.len() > dom_mark {
                    let (id, prev) = self.saved_domains.pop().unwrap();
                    self.domains[id] = prev;
                }
                self.unsuspend();
                let mut back = Ok(());
                for &(param, to) in &fc.muts {
                    match (to, fed_chunk.slot_map[param]) {
                        (FeedBack::I(dst), SlotLoc::I(r, _)) => {
                            vm_regs_i[base_i + dst as usize] =
                                vm_regs_i[callee_base_i + r as usize];
                        }
                        (FeedBack::OptI(dst_val, dst_some), SlotLoc::OptI { val, some, .. }) => {
                            vm_regs_i[base_i + dst_val as usize] =
                                vm_regs_i[callee_base_i + val as usize];
                            vm_regs_i[base_i + dst_some as usize] =
                                vm_regs_i[callee_base_i + some as usize];
                        }
                        (to, from) => {
                            let val = match (to, from) {
                                (FeedBack::Place(_), SlotLoc::V(r)) => {
                                    std::mem::take(&mut callee_regs_v[r as usize])
                                }
                                (FeedBack::PlaceI(_, kind), SlotLoc::I(r, _)) => {
                                    kind.box_val(vm_regs_i[callee_base_i + r as usize])
                                }
                                (FeedBack::PlaceOptI(_, kind), SlotLoc::OptI { val, some, .. }) => {
                                    kind.box_opt(
                                        vm_regs_i[callee_base_i + val as usize],
                                        vm_regs_i[callee_base_i + some as usize],
                                    )
                                }
                                _ => unreachable!("a mut argument is kept as its parameter is"),
                            };
                            let (FeedBack::Place(path)
                            | FeedBack::PlaceI(path, _)
                            | FeedBack::PlaceOptI(path, _)) = to
                            else {
                                unreachable!("a register write-back is matched above")
                            };
                            let pp = &chunk.places[path as usize];
                            let keys = place_keys(pp, regs_v);
                            let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                            let regs_i = &vm_regs_i[base_i..base_i + chunk.num_i];
                            if back.is_ok() {
                                back = with_place_cell(
                                    root,
                                    &pp.steps,
                                    regs_i,
                                    &keys,
                                    spans[cur_pc],
                                    |cell| {
                                        *cell = val;
                                        Ok(())
                                    },
                                );
                            }
                        }
                    }
                }
                self.return_regs_v(callee_regs_v);
                vm_regs_i.truncate(callee_base_i);
                let res = res?;
                back?;
                self.call_frames.pop();
                match res {
                    ExecExit::FeedNext | ExecExit::FeedBreak => {}
                    ExecExit::Return(v) => return Ok(Some(ExecExit::Return(v))),
                }
            }
            Op::FeedYieldI { src, kind, back } => {
                let raw = regs_i[src as usize];
                let ctx = fed.as_deref_mut().expect("FeedYieldI outside fed call");
                match ctx.var_loc {
                    SlotLoc::I(r, _) => {
                        vm_regs_i[ctx.caller_base_i + r as usize] = raw;
                    }
                    SlotLoc::OptI { val, some, .. } => {
                        vm_regs_i[ctx.caller_base_i + val as usize] = raw;
                        vm_regs_i[ctx.caller_base_i + some as usize] = 1;
                    }
                    SlotLoc::V(r) => {
                        ctx.caller_regs_v[r as usize] = kind.box_val(raw);
                    }
                    SlotLoc::SmallTuple { .. } => {
                        unreachable!("scalar yield into SmallTuple");
                    }
                }
                let saved_base = self.base;
                self.feeding();
                self.base = ctx.caller_base;
                let body_res = self.exec_chunk_core(
                    compiled,
                    ctx.caller_chunk,
                    ctx.caller_base_i,
                    vm_regs_i,
                    ctx.caller_regs_v,
                    ctx.body_pc,
                    None,
                );
                self.base = saved_base;
                match body_res? {
                    ExecExit::FeedNext if back != 0 => {
                        let val = read_slot(
                            &ctx.var_loc,
                            &vm_regs_i[ctx.caller_base_i..],
                            ctx.caller_regs_v,
                        );
                        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        write_slot(&chunk.slot_map[back as usize - 1], regs_i, regs_v, val);
                    }
                    ExecExit::FeedNext => {}
                    ExecExit::FeedBreak => return Ok(Some(ExecExit::FeedBreak)),
                    ExecExit::Return(v) => return Ok(Some(ExecExit::Return(v))),
                }
            }
            Op::FeedYieldV { src, back } => {
                let moved = back != 0 && chunk.slot_map[back as usize - 1] == SlotLoc::V(src);
                let val = if moved || src >= chunk.num_local_v as u16 {
                    std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                } else {
                    regs_v[src as usize].clone()
                };
                let ctx = fed.as_deref_mut().expect("FeedYieldV outside fed call");
                store_fed_val(ctx, vm_regs_i, val);
                let saved_base = self.base;
                self.feeding();
                self.base = ctx.caller_base;
                let body_res = self.exec_chunk_core(
                    compiled,
                    ctx.caller_chunk,
                    ctx.caller_base_i,
                    vm_regs_i,
                    ctx.caller_regs_v,
                    ctx.body_pc,
                    None,
                );
                self.base = saved_base;
                match body_res? {
                    ExecExit::FeedNext if back != 0 => {
                        let val = read_slot(
                            &ctx.var_loc,
                            &vm_regs_i[ctx.caller_base_i..],
                            ctx.caller_regs_v,
                        );
                        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        write_slot(&chunk.slot_map[back as usize - 1], regs_i, regs_v, val);
                    }
                    ExecExit::FeedNext => {}
                    ExecExit::FeedBreak => return Ok(Some(ExecExit::FeedBreak)),
                    ExecExit::Return(v) => return Ok(Some(ExecExit::Return(v))),
                }
            }
            Op::FeedYieldExtend { src } => {
                let seq = regs_v[src as usize].clone();
                let ctx = fed
                    .as_deref_mut()
                    .expect("FeedYieldExtend outside fed call");
                let items: Vec<Value> = match seq {
                    Value::List(xs) => xs.iter().cloned().collect(),
                    Value::Array(arr) => (0..arr.len()).map(|i| arr.get_unchecked_val(i)).collect(),
                    _ => {
                        return trap(
                            spans[cur_pc],
                            "internal: a `yields` fn did something else to its list",
                        )
                    }
                };
                for item in items {
                    store_fed_val(ctx, vm_regs_i, item);
                    let saved_base = self.base;
                    self.feeding();
                    self.base = ctx.caller_base;
                    let body_res = self.exec_chunk_core(
                        compiled,
                        ctx.caller_chunk,
                        ctx.caller_base_i,
                        vm_regs_i,
                        ctx.caller_regs_v,
                        ctx.body_pc,
                        None,
                    );
                    self.base = saved_base;
                    match body_res? {
                        ExecExit::FeedNext => {}
                        ExecExit::FeedBreak => return Ok(Some(ExecExit::FeedBreak)),
                        ExecExit::Return(v) => return Ok(Some(ExecExit::Return(v))),
                    }
                }
            }
            Op::FeedBodyNext => return Ok(Some(ExecExit::FeedNext)),
            Op::FeedBodyBreak => return Ok(Some(ExecExit::FeedBreak)),
            Op::FeedEnd => return Ok(Some(ExecExit::FeedNext)),
            Op::ClearV { reg } => {
                regs_v[reg as usize] = Value::Unit;
            }
            Op::ConstV { dst, idx } => {
                regs_v[dst as usize] = chunk.consts_v[idx as usize].clone();
            }
            Op::Print { idx } => {
                let spec = &chunk.prints[idx as usize];
                let vals: Vec<Value> = spec
                    .regs
                    .iter()
                    .map(|&r| regs_v[r as usize].clone())
                    .collect();
                let end = spec.end.map(|r| regs_v[r as usize].clone());
                let base = self.base;
                // The tracer's view of a print reads the locals off the stack.
                let shown = chunk.traced && self.observed();
                if shown {
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, base, true);
                    self.show_lent();
                }
                let printed = self.print_values(spec.args, &vals, end.as_ref(), spans[cur_pc]);
                if shown {
                    self.hide_lent();
                    take_back_values(&chunk.slot_map, regs_v, &mut self.stack, base);
                }
                printed?;
            }
            Op::Format { idx, dst } => {
                let spec = &chunk.formats[idx as usize];
                let vals: Vec<Value> = spec
                    .regs
                    .iter()
                    .map(|&r| regs_v[r as usize].clone())
                    .collect();
                regs_v[dst as usize] = self.format_values(spec.pieces, &vals, spans[cur_pc])?;
            }
            Op::DefDomain { dom, lo, len } => {
                self.domains[dom as usize] = (regs_i[lo as usize], regs_i[len as usize].max(0));
            }
            Op::DefDomainFromVal { dom, src } => {
                if let Value::DomainVal(lo, len) = regs_v[src as usize] {
                    self.domains[dom as usize] = (lo, len.max(0));
                }
            }
            Op::ApplyV { idx, dst, sync } => {
                let spec = &chunk.applies[idx as usize];
                let vals: Vec<Value> = spec
                    .regs
                    .iter()
                    .map(|&r| regs_v[r as usize].clone())
                    .collect();
                let val = if sync {
                    let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    sync_to_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                    self.show_lent();
                    let val = self.apply(spec.expr, vals);
                    self.hide_lent();
                    let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    sync_from_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                    val
                } else {
                    self.apply(spec.expr, vals)
                };
                regs_v[dst as usize] = val?;
            }
            Op::MutatePlace { idx, path, dst } => {
                let spec = &chunk.applies[idx as usize];
                let cpl_ir::ExprKind::Mutate(op, place, _) = &spec.expr.kind else {
                    return trap(spans[cur_pc], "internal: not a mutation");
                };
                let arg = |n: usize| {
                    spec.regs
                        .get(n)
                        .map_or(Value::Unit, |&r| regs_v[r as usize].clone())
                };
                let (arg, arg2) = (arg(0), arg(1));
                let program = self.program;
                let pp = &chunk.places[path as usize];
                let keys = place_keys(pp, regs_v);
                let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                let span = spans[cur_pc];
                regs_v[dst as usize] =
                    with_place_cell(root, &pp.steps, regs_i, &keys, span, |cell| {
                        crate::mutation::mutate_cell(*op, cell, &place.ty, arg, arg2, program, span)
                    })?;
            }
            Op::FallbackStmt {
                idx,
                clear_from_v,
                in_loop,
            } => {
                if let Some(stats) = &mut self.vm_stats {
                    stats.record_with_loop(stmt_tag(chunk.fallback_stmts[idx as usize]), in_loop);
                }
                for (r, held) in regs_v.iter_mut().enumerate().skip(clear_from_v as usize) {
                    // A traced local outlives its last use: the tracer shows it.
                    if !(chunk.traced && chunk.slot_map.contains(&SlotLoc::V(r as u16))) {
                        *held = Value::Unit;
                    }
                }
                sync_to_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let stmt = chunk.fallback_stmts[idx as usize];
                let outer = self.enter_fallback(chunk, cur_pc, regs_i, regs_v);
                let hosts = !chunk.fed_bodies.is_empty();
                if hosts {
                    self.fed_hosts.push((self.base, chunk as *const Chunk<'p>));
                }
                self.show_lent();
                let flow = self.stmt(stmt);
                self.hide_lent();
                if hosts {
                    self.fed_hosts.pop();
                }
                self.leave_fallback(outer);
                sync_from_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let flow = flow?;
                match flow {
                    Flow::Next => {}
                    Flow::Return(v) | Flow::Give(v) => return Ok(Some(ExecExit::Return(v))),
                    Flow::Break(id) => *escaped = Some((true, id)),
                    Flow::Continue(id) => *escaped = Some((false, id)),
                }
            }
            Op::FallbackExprI { idx, dst, in_loop } => {
                if let Some(stats) = &mut self.vm_stats {
                    stats.record_with_loop(
                        expr_tag(chunk.fallback_exprs[idx as usize], self.program),
                        in_loop,
                    );
                }
                sync_to_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let expr = chunk.fallback_exprs[idx as usize];
                self.show_lent();
                let val = self.eval(expr);
                self.hide_lent();
                sync_from_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let val = val?;
                regs_i[dst as usize] = ScalarKind::unbox_val(&val);
            }
            Op::FallbackExprV { idx, dst, in_loop } => {
                if let Some(stats) = &mut self.vm_stats {
                    stats.record_with_loop(
                        expr_tag(chunk.fallback_exprs[idx as usize], self.program),
                        in_loop,
                    );
                }
                sync_to_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let expr = chunk.fallback_exprs[idx as usize];
                self.show_lent();
                let val = self.eval(expr);
                self.hide_lent();
                sync_from_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    self.base,
                    chunk.traced,
                );
                let val = val?;
                regs_v[dst as usize] = val;
            }
            _ => unreachable!("unhandled op in exec_places_and_fallback"),
        }
        Ok(None)
    }
}
