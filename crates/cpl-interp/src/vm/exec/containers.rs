use crate::observe::Observer;
use crate::value::{ArrayData, OptVal, TupleVal, Value};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{trap, Interp, Res};
use cpl_ir::CmpOp;
use std::rc::Rc;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    #[inline]
    pub(super) fn exec_seq_and_list(
        &mut self,
        op: Op,
        cur_pc: usize,
        chunk: &Chunk<'p>,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<()> {
        let spans = chunk.spans.as_slice();
        match op {
            Op::SeqLenI { dst, seq_v } => {
                regs_i[dst as usize] = match &regs_v[seq_v as usize] {
                    Value::Array(arr) => arr.len() as i64,
                    Value::List(items) => items.len() as i64,
                    Value::ListView(view) => view.len as i64,
                    Value::Str(bytes) => bytes.len() as i64,
                    Value::StrView(view) => view.len as i64,
                    Value::Map(m) => m.len() as i64,
                    Value::Set(s) => s.len() as i64,
                    Value::Range(lo, hi) => (hi - lo).max(0),
                    Value::Range128(lo, hi) if lo >= hi => 0,
                    Value::Range128(lo, hi) => {
                        match hi.checked_sub(*lo).and_then(|d| i64::try_from(d).ok()) {
                            Some(d) => d,
                            None => {
                                return trap(
                                        spans[cur_pc],
                                        format!("integer overflow: range `{lo}..{hi}` length does not fit in an `Int` (64 bits)"),
                                    );
                            }
                        }
                    }
                    Value::DomainVal(_, len) => *len,
                    _ => 0,
                };
                if seq_v as usize >= chunk.num_local_v {
                    regs_v[seq_v as usize] = Value::Unit;
                }
            }
            Op::RangeLoI { dst, src_v } => {
                regs_i[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Range(lo, _) => *lo,
                    _ => 0,
                };
                if src_v as usize >= chunk.num_local_v {
                    regs_v[src_v as usize] = Value::Unit;
                }
            }
            Op::RangeBoundsI { lo, hi, src_v } => {
                let (l, h) = match &regs_v[src_v as usize] {
                    Value::Range(l, h) => (*l, *h),
                    _ => return trap(spans[cur_pc], "internal: expected a range"),
                };
                regs_i[lo as usize] = l;
                regs_i[hi as usize] = h;
                if src_v as usize >= chunk.num_local_v {
                    regs_v[src_v as usize] = Value::Unit;
                }
            }
            Op::RangeHiI { dst, src_v } => {
                regs_i[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Range(_, hi) => *hi,
                    _ => 0,
                };
                if src_v as usize >= chunk.num_local_v {
                    regs_v[src_v as usize] = Value::Unit;
                }
            }
            Op::BitCeilI { dst, src } => {
                let x = regs_i[src as usize];
                regs_i[dst as usize] = if x <= 1 {
                    1
                } else {
                    let shift = 64 - ((x - 1) as u64).leading_zeros();
                    if shift > 62 {
                        return trap(
                                spans[cur_pc],
                                format!(
                                    "integer overflow: `bit_ceil({x})` does not fit in an `Int` (64 bits)"
                                ),
                            );
                    }
                    1i64 << shift
                };
            }
            Op::PopcountI { dst, src } => {
                regs_i[dst as usize] = i64::from((regs_i[src as usize] as u64).count_ones());
            }
            Op::CtzI { dst, src } => {
                regs_i[dst as usize] = i64::from((regs_i[src as usize] as u64).trailing_zeros());
            }
            Op::ClzI { dst, src } => {
                regs_i[dst as usize] = i64::from((regs_i[src as usize] as u64).leading_zeros());
            }
            Op::MinMaxGArr1SmallTuple {
                gid,
                r_i,
                src_base,
                len,
                is_max,
            } => {
                let idx = regs_i[r_i as usize];
                let cand = &regs_i[src_base as usize..src_base as usize + len as usize];
                if let Value::Array(arr) = &mut self.globals[gid as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    let n = len as usize;
                    if let ArrayData::SmallIntTuple { stride, data, .. } = &mut a.data {
                        let off = idx as usize * (*stride as usize);
                        let cur = &mut data[off..off + n];
                        let better = if is_max { cand > cur } else { cand < cur };
                        if better {
                            cur.copy_from_slice(cand);
                        }
                    } else {
                        let vs = a.as_mut_values();
                        let mut cur = [0i64; 4];
                        if let Value::Tuple(TupleVal::Boxed(fs)) = &vs[idx as usize] {
                            for k in 0..n {
                                cur[k] = ScalarKind::unbox_val(&fs[k]);
                            }
                        }
                        let better = if is_max {
                            cand > &cur[..n]
                        } else {
                            cand < &cur[..n]
                        };
                        if better {
                            if let Value::Tuple(TupleVal::Boxed(fs)) = &mut vs[idx as usize] {
                                let fs = Rc::make_mut(fs);
                                for k in 0..n {
                                    fs[k] = Value::Int(cand[k]);
                                }
                            }
                        }
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::MinMaxLArr1SmallTuple {
                arr_v,
                r_i,
                src_base,
                len,
                is_max,
            } => {
                let idx = regs_i[r_i as usize];
                let cand = &regs_i[src_base as usize..src_base as usize + len as usize];
                if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    let n = len as usize;
                    if let ArrayData::SmallIntTuple { stride, data, .. } = &mut a.data {
                        let off = idx as usize * (*stride as usize);
                        let cur = &mut data[off..off + n];
                        let better = if is_max { cand > cur } else { cand < cur };
                        if better {
                            cur.copy_from_slice(cand);
                        }
                    } else {
                        let vs = a.as_mut_values();
                        let mut cur = [0i64; 4];
                        if let Value::Tuple(TupleVal::Boxed(fs)) = &vs[idx as usize] {
                            for k in 0..n {
                                cur[k] = ScalarKind::unbox_val(&fs[k]);
                            }
                        }
                        let better = if is_max {
                            cand > &cur[..n]
                        } else {
                            cand < &cur[..n]
                        };
                        if better {
                            if let Value::Tuple(TupleVal::Boxed(fs)) = &mut vs[idx as usize] {
                                let fs = Rc::make_mut(fs);
                                for k in 0..n {
                                    fs[k] = Value::Int(cand[k]);
                                }
                            }
                        }
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ListPushGArr1I {
                gid,
                r_i,
                val_i,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let v = kind.box_val(regs_i[val_i as usize]);
                if let Value::Array(arr) = &mut self.globals[gid as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize]
                    {
                        Rc::make_mut(list).push(v);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list element");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ListPushGArr1V { gid, r_i, val_v } => {
                let idx = regs_i[r_i as usize];
                let v = if val_v as usize >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                } else {
                    regs_v[val_v as usize].clone()
                };
                if let Value::Array(arr) = &mut self.globals[gid as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize]
                    {
                        Rc::make_mut(list).push(v);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list element");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ListPushLArr1I {
                arr_v,
                r_i,
                val_i,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let v = kind.box_val(regs_i[val_i as usize]);
                if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize]
                    {
                        Rc::make_mut(list).push(v);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list element");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ListPushLArr1V { arr_v, r_i, val_v } => {
                let idx = regs_i[r_i as usize];
                let v = if val_v as usize >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                } else {
                    regs_v[val_v as usize].clone()
                };
                if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                    let d0 = arr.dims[0];
                    if idx < 0 || idx >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {d0}"),
                        );
                    }
                    if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize]
                    {
                        Rc::make_mut(list).push(v);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list element");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ListPushLV { list_v, val_v } => {
                let item = if (val_v as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                } else {
                    regs_v[val_v as usize].clone()
                };
                match &mut regs_v[list_v as usize] {
                    Value::List(items) => Rc::make_mut(items).push(item),
                    Value::Str(bytes) => {
                        if let Value::Char(c) = item {
                            Rc::make_mut(bytes).push(c);
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list or string"),
                }
            }
            Op::ListPushGV { gid, val_v } => {
                let item = if (val_v as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                } else {
                    regs_v[val_v as usize].clone()
                };
                match &mut self.globals[gid as usize] {
                    Value::List(items) => Rc::make_mut(items).push(item),
                    Value::Str(bytes) => {
                        if let Value::Char(c) = item {
                            Rc::make_mut(bytes).push(c);
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list or string"),
                }
            }
            Op::ListPushLI {
                list_v,
                val_i,
                kind,
            } => {
                let raw = regs_i[val_i as usize];
                match &mut regs_v[list_v as usize] {
                    Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                    Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                    _ => return trap(spans[cur_pc], "internal: expected list or string"),
                }
            }
            Op::ListPushGI { gid, val_i, kind } => {
                let raw = regs_i[val_i as usize];
                match &mut self.globals[gid as usize] {
                    Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                    Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                    _ => return trap(spans[cur_pc], "internal: expected list or string"),
                }
            }
            Op::ListPopLV { dst_v, list_v } => {
                let popped = match &mut regs_v[list_v as usize] {
                    Value::List(items) => Rc::make_mut(items).pop(),
                    Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
            }
            Op::ListPopGV { dst_v, gid } => {
                let popped = match &mut self.globals[gid as usize] {
                    Value::List(items) => Rc::make_mut(items).pop(),
                    Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
            }
            Op::ListPopOptIL {
                dst_val,
                dst_some,
                list_v,
            } => {
                let popped = match &mut regs_v[list_v as usize] {
                    Value::List(items) => Rc::make_mut(items).pop(),
                    Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                match popped {
                    Some(x) => {
                        regs_i[dst_val as usize] = ScalarKind::unbox_val(&x);
                        regs_i[dst_some as usize] = 1;
                    }
                    None => {
                        regs_i[dst_val as usize] = 0;
                        regs_i[dst_some as usize] = 0;
                    }
                }
            }
            Op::ListPopOptIG {
                dst_val,
                dst_some,
                gid,
            } => {
                let popped = match &mut self.globals[gid as usize] {
                    Value::List(items) => Rc::make_mut(items).pop(),
                    Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                match popped {
                    Some(x) => {
                        regs_i[dst_val as usize] = ScalarKind::unbox_val(&x);
                        regs_i[dst_some as usize] = 1;
                    }
                    None => {
                        regs_i[dst_val as usize] = 0;
                        regs_i[dst_some as usize] = 0;
                    }
                }
            }
            Op::ListPopDiscardL { list_v } => match &mut regs_v[list_v as usize] {
                Value::List(items) => {
                    Rc::make_mut(items).pop();
                }
                Value::Str(bytes) => {
                    Rc::make_mut(bytes).pop();
                }
                _ => return trap(spans[cur_pc], "internal: expected list"),
            },
            Op::ListPopDiscardG { gid } => match &mut self.globals[gid as usize] {
                Value::List(items) => {
                    Rc::make_mut(items).pop();
                }
                Value::Str(bytes) => {
                    Rc::make_mut(bytes).pop();
                }
                _ => return trap(spans[cur_pc], "internal: expected list"),
            },
            Op::LoadGlobalVMove { dst, gid } => {
                regs_v[dst as usize] =
                    std::mem::replace(&mut self.globals[gid as usize], Value::Unit);
            }
            Op::StoreLFieldI {
                obj_v,
                field,
                src_i,
                kind,
            } => {
                let val = kind.box_val(regs_i[src_i as usize]);
                match &mut regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => {
                        Rc::make_mut(fs)[field as usize] = val;
                    }
                    Value::Tuple(TupleVal::OneInt(v)) if field == 0 => {
                        *v = regs_i[src_i as usize];
                    }
                    _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                }
            }
            Op::StoreLFieldV {
                obj_v,
                field,
                src_v,
            } => {
                let val = if (src_v as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[src_v as usize], Value::Unit)
                } else {
                    regs_v[src_v as usize].clone()
                };
                match &mut regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => {
                        Rc::make_mut(fs)[field as usize] = val;
                    }
                    _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                }
            }
            Op::StoreLFieldOptI {
                obj_v,
                field,
                val,
                some,
                kind,
            } => {
                let boxed = kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                match &mut regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => {
                        Rc::make_mut(fs)[field as usize] = boxed;
                    }
                    _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                }
            }
            Op::StoreLFieldIdx1I {
                obj_v,
                field,
                r_i,
                r_val,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let val = regs_i[r_val as usize];
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array of length {len}"),
                                );
                            }
                            write_array_i(Rc::make_mut(arr), idx as usize, val);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::StoreLFieldIdx1V {
                obj_v,
                field,
                r_i,
                r_val,
            } => {
                let idx = regs_i[r_i as usize];
                let val = if (r_val as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                } else {
                    regs_v[r_val as usize].clone()
                };
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array of length {len}"),
                                );
                            }
                            Rc::make_mut(arr).set(idx as usize, val);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            Rc::make_mut(list)[idx as usize] = val;
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::StoreLFieldIdx1OptI {
                obj_v,
                field,
                r_i,
                val,
                some,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let v = regs_i[val as usize];
                let s = regs_i[some as usize];
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array of length {len}"),
                                );
                            }
                            write_array_opt_i(Rc::make_mut(arr), idx as usize, v, s, kind);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            Rc::make_mut(list)[idx as usize] = kind.box_opt(v, s);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::ListPushFieldLI {
                obj_v,
                field,
                val_i,
                kind,
            } => {
                let raw = regs_i[val_i as usize];
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                        Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::ListPushFieldLV {
                obj_v,
                field,
                val_v,
            } => {
                let item = if (val_v as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                } else {
                    regs_v[val_v as usize].clone()
                };
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::List(items) => Rc::make_mut(items).push(item),
                        Value::Str(bytes) => {
                            if let Value::Char(c) = item {
                                Rc::make_mut(bytes).push(c);
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::ListPopDiscardFieldL { obj_v, field } => {
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    match &mut Rc::make_mut(fs)[field as usize] {
                        Value::List(items) => {
                            Rc::make_mut(items).pop();
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::StoreGList1V { gid, r_i, r_val } => {
                let idx = regs_i[r_i as usize];
                let val = if (r_val as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                } else {
                    regs_v[r_val as usize].clone()
                };
                if let Value::List(list) = &mut self.globals[gid as usize] {
                    let len = list.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    Rc::make_mut(list)[idx as usize] = val;
                } else {
                    return trap(spans[cur_pc], "internal: expected list");
                }
            }
            Op::StoreLList1V { list_v, r_i, r_val } => {
                let idx = regs_i[r_i as usize];
                let val = if (r_val as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                } else {
                    regs_v[r_val as usize].clone()
                };
                if let Value::List(list) = &mut regs_v[list_v as usize] {
                    let len = list.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    Rc::make_mut(list)[idx as usize] = val;
                } else {
                    return trap(spans[cur_pc], "internal: expected list");
                }
            }
            Op::LoadGList1V { dst, gid, r_i } => {
                let idx = regs_i[r_i as usize];
                if let Value::List(list) = &self.globals[gid as usize] {
                    let len = list.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    regs_v[dst as usize] = list[idx as usize].clone();
                } else {
                    return trap(spans[cur_pc], "internal: expected list");
                }
            }
            Op::LoadLList1V { dst, list_v, r_i } => {
                let idx = regs_i[r_i as usize];
                match &regs_v[list_v as usize] {
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        regs_v[dst as usize] = list[idx as usize].clone();
                    }
                    Value::ListView(view) => {
                        let len = view.len as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        regs_v[dst as usize] =
                            view.get(idx as usize).cloned().unwrap_or(Value::Unit);
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                }
            }
            Op::LoadLList1OptI {
                dst_val,
                dst_some,
                list_v,
                r_i,
            } => {
                let idx = regs_i[r_i as usize];
                let (v, s) = match &regs_v[list_v as usize] {
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        ScalarKind::unbox_opt(&list[idx as usize])
                    }
                    Value::ListView(view) => {
                        let len = view.len as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        ScalarKind::unbox_opt(view.get(idx as usize).unwrap_or(&Value::Unit))
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                };
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
            }
            Op::SeqItems { dst, src } => {
                let items: Vec<Value> = match &regs_v[src as usize] {
                    Value::Set(members) => members.iter().cloned().collect(),
                    Value::Map(entries) => entries
                        .iter()
                        .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))
                        .collect(),
                    _ => return trap(spans[cur_pc], "internal: expected a set or a map"),
                };
                regs_v[dst as usize] = Value::List(Rc::new(items));
            }
            Op::ListPopFieldOptIL {
                dst_val,
                dst_some,
                obj_v,
                field,
            } => {
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    if let Value::List(items) = &mut Rc::make_mut(fs)[field as usize] {
                        match Rc::make_mut(items).pop() {
                            Some(v) => {
                                regs_i[dst_val as usize] = ScalarKind::unbox_val(&v);
                                regs_i[dst_some as usize] = 1;
                            }
                            None => {
                                regs_i[dst_val as usize] = 0;
                                regs_i[dst_some as usize] = 0;
                            }
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::ListPopFieldLV {
                dst_v,
                obj_v,
                field,
            } => {
                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                    if let Value::List(items) = &mut Rc::make_mut(fs)[field as usize] {
                        regs_v[dst_v as usize] =
                            Value::Opt(OptVal::from_option(Rc::make_mut(items).pop()));
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected tuple/struct");
                }
            }
            Op::MakeRange { dst, r_lo, r_hi } => {
                let lo = regs_i[r_lo as usize];
                let hi = regs_i[r_hi as usize];
                if hi > lo && hi.checked_sub(lo).is_none() {
                    return trap(
                            spans[cur_pc],
                            format!(
                                "integer overflow: range `{lo}..{hi}` span does not fit in an `Int` (64 bits)"
                            ),
                        );
                }
                regs_v[dst as usize] = Value::Range(lo, hi);
            }
            Op::CmpOptI {
                dst,
                a_val,
                a_some,
                b_val,
                b_some,
                op,
            } => {
                let a = if regs_i[a_some as usize] != 0 {
                    Some(regs_i[a_val as usize])
                } else {
                    None
                };
                let b = if regs_i[b_some as usize] != 0 {
                    Some(regs_i[b_val as usize])
                } else {
                    None
                };
                let res = match op {
                    CmpOp::Eq => a == b,
                    CmpOp::Ne => a != b,
                    CmpOp::Lt => a < b,
                    CmpOp::Le => a <= b,
                    CmpOp::Gt => a > b,
                    CmpOp::Ge => a >= b,
                    _ => false,
                };
                regs_i[dst as usize] = i64::from(res);
            }
            Op::LoadGIdxFieldI {
                dst,
                gid,
                r_i,
                field,
            } => {
                let idx = regs_i[r_i as usize];
                let v = match &self.globals[gid as usize] {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        match &arr.data {
                            ArrayData::SmallIntTuple { stride, data, .. } => {
                                data[idx as usize * (*stride as usize) + field as usize]
                            }
                            ArrayData::Values(vs) => match &vs[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    ScalarKind::unbox_val(&fs[field as usize])
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x,
                                _ => 0,
                            },
                            _ => 0,
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        match &list[idx as usize] {
                            Value::Tuple(TupleVal::Boxed(fs)) => {
                                ScalarKind::unbox_val(&fs[field as usize])
                            }
                            Value::Tuple(TupleVal::OneInt(x)) => *x,
                            _ => 0,
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                };
                regs_i[dst as usize] = v;
            }
            Op::StoreGIdxFieldI {
                gid,
                r_i,
                field,
                r_val,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let raw = regs_i[r_val as usize];
                match &mut self.globals[gid as usize] {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        match &mut a.data {
                            ArrayData::SmallIntTuple { stride, data, .. } => {
                                data[idx as usize * (*stride as usize) + field as usize] = raw;
                            }
                            ArrayData::Values(vs) => match &mut vs[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                _ => return trap(spans[cur_pc], "internal: expected tuple"),
                            },
                            _ => return trap(spans[cur_pc], "internal: expected tuple array"),
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        match &mut Rc::make_mut(list)[idx as usize] {
                            Value::Tuple(TupleVal::Boxed(fs)) => {
                                Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                            }
                            Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                            _ => return trap(spans[cur_pc], "internal: expected tuple"),
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                }
            }
            Op::LoadLIdxFieldI {
                dst,
                seq_v,
                r_i,
                field,
            } => {
                let idx = regs_i[r_i as usize];
                let v = match &regs_v[seq_v as usize] {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        match &arr.data {
                            ArrayData::SmallIntTuple { stride, data, .. } => {
                                data[idx as usize * (*stride as usize) + field as usize]
                            }
                            ArrayData::Values(vs) => match &vs[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    ScalarKind::unbox_val(&fs[field as usize])
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x,
                                _ => 0,
                            },
                            _ => 0,
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        match &list[idx as usize] {
                            Value::Tuple(TupleVal::Boxed(fs)) => {
                                ScalarKind::unbox_val(&fs[field as usize])
                            }
                            Value::Tuple(TupleVal::OneInt(x)) => *x,
                            _ => 0,
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                };
                regs_i[dst as usize] = v;
            }
            Op::StoreLIdxFieldI {
                seq_v,
                r_i,
                field,
                r_val,
                kind,
            } => {
                let idx = regs_i[r_i as usize];
                let raw = regs_i[r_val as usize];
                match &mut regs_v[seq_v as usize] {
                    Value::Array(arr) => {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        match &mut a.data {
                            ArrayData::SmallIntTuple { stride, data, .. } => {
                                data[idx as usize * (*stride as usize) + field as usize] = raw;
                            }
                            ArrayData::Values(vs) => match &mut vs[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                _ => return trap(spans[cur_pc], "internal: expected tuple"),
                            },
                            _ => return trap(spans[cur_pc], "internal: expected tuple array"),
                        }
                    }
                    Value::List(list) => {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        match &mut Rc::make_mut(list)[idx as usize] {
                            Value::Tuple(TupleVal::Boxed(fs)) => {
                                Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                            }
                            Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                            _ => return trap(spans[cur_pc], "internal: expected tuple"),
                        }
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array or list"),
                }
            }
            _ => unreachable!("non-container op routed to exec_seq_and_list"),
        }
        Ok(())
    }
}
