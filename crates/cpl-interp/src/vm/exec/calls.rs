use crate::checkpoint::Body;
use crate::observe::Observer;
use crate::value::{Array, ArrayData, OptVal, TupleVal, Value};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{trap, Interp, Res};
use cpl_ir::{BinOp, CmpOp};
use std::rc::Rc;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(super) fn exec_fields_and_calls(
        &mut self,
        op: Op,
        cur_pc: usize,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
    ) -> Res<Option<ExecExit>> {
        let spans = chunk.spans.as_slice();
        let mut regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        match op {
            Op::FieldI { dst, obj_v, field } => {
                regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => ScalarKind::unbox_val(&fs[field as usize]),
                    Value::Tuple(TupleVal::OneInt(v)) => *v,
                    _ => 0,
                };
                if obj_v as usize >= chunk.num_local_v {
                    regs_v[obj_v as usize] = Value::Unit;
                }
            }
            Op::FieldOptI {
                dst_val,
                dst_some,
                obj_v,
                field,
            } => {
                let (v, s) = match &regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => ScalarKind::unbox_opt(&fs[field as usize]),
                    _ => (0, 0),
                };
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
                if obj_v as usize >= chunk.num_local_v {
                    regs_v[obj_v as usize] = Value::Unit;
                }
            }
            Op::FieldV { dst, obj_v, field } => {
                let val = match &regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => fs[field as usize].clone(),
                    Value::Tuple(TupleVal::OneInt(v)) => Value::Int(*v),
                    _ => Value::Unit,
                };
                if obj_v as usize >= chunk.num_local_v {
                    regs_v[obj_v as usize] = Value::Unit;
                }
                regs_v[dst as usize] = val;
            }
            Op::LoadFieldList1I {
                dst,
                obj_v,
                field,
                r_i,
            } => {
                let idx = regs_i[r_i as usize];
                if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[obj_v as usize] {
                    if let Value::List(list) = &fs[field as usize] {
                        if idx < 0 || (idx as usize) >= list.len() {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {}", list.len()),
                            );
                        }
                        regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list field");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected struct");
                }
            }
            Op::FieldListLen { dst, obj_v, field } => {
                regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => match &fs[field as usize] {
                        Value::List(list) => list.len() as i64,
                        _ => 0,
                    },
                    _ => 0,
                };
            }
            Op::LoadFieldArr1I {
                dst,
                obj_v,
                field,
                r_i,
            } => {
                let idx = regs_i[r_i as usize];
                if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[obj_v as usize] {
                    if let Value::Array(arr) = &fs[field as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, idx as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array field");
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected struct");
                }
            }
            Op::FieldArrLen { dst, obj_v, field } => {
                regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                    Value::Tuple(TupleVal::Boxed(fs)) => match &fs[field as usize] {
                        Value::Array(arr) => arr.dims.first().copied().unwrap_or(0),
                        _ => 0,
                    },
                    _ => 0,
                };
            }
            Op::StrGetI { dst, str_v, r_i } => {
                let idx = regs_i[r_i as usize];
                match &regs_v[str_v as usize] {
                    Value::Str(s) => {
                        if idx < 0 || (idx as usize) >= s.len() {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a string of length {}", s.len()),
                            );
                        }
                        regs_i[dst as usize] = i64::from(s[idx as usize]);
                    }
                    Value::StrView(sv) => {
                        if idx < 0 || (idx as usize) >= sv.len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a string of length {}", sv.len),
                            );
                        }
                        let pos = if sv.rev {
                            sv.lo + sv.len - 1 - idx as usize
                        } else {
                            sv.lo + idx as usize
                        };
                        regs_i[dst as usize] = i64::from(sv.base[pos]);
                    }
                    _ => return trap(spans[cur_pc], "internal: expected string"),
                }
            }
            Op::GStrGetI { dst, gid, r_i } => {
                let idx = regs_i[r_i as usize];
                match &self.globals[gid as usize] {
                    Value::Str(s) => {
                        if idx < 0 || (idx as usize) >= s.len() {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a string of length {}", s.len()),
                            );
                        }
                        regs_i[dst as usize] = i64::from(s[idx as usize]);
                    }
                    Value::StrView(sv) => {
                        if idx < 0 || (idx as usize) >= sv.len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a string of length {}", sv.len),
                            );
                        }
                        let pos = if sv.rev {
                            sv.lo + sv.len - 1 - idx as usize
                        } else {
                            sv.lo + idx as usize
                        };
                        regs_i[dst as usize] = i64::from(sv.base[pos]);
                    }
                    _ => return trap(spans[cur_pc], "internal: expected string"),
                }
            }
            Op::MakeTuple { dst, base_v, len } => {
                let b = base_v as usize;
                let n = len as usize;
                if n == 1 {
                    let v = std::mem::replace(&mut regs_v[b], Value::Unit);
                    regs_v[dst as usize] = match v {
                        Value::Int(x) => Value::Tuple(TupleVal::OneInt(x)),
                        other => Value::Tuple(TupleVal::Boxed(Rc::new(vec![other]))),
                    };
                } else {
                    let mut fs = Vec::with_capacity(n);
                    for k in 0..n {
                        fs.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                    }
                    regs_v[dst as usize] = Value::Tuple(TupleVal::Boxed(Rc::new(fs)));
                }
            }
            Op::MakeList { dst, base_v, len } => {
                let b = base_v as usize;
                let n = len as usize;
                let mut items = Vec::with_capacity(n);
                for k in 0..n {
                    items.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                }
                regs_v[dst as usize] = Value::List(Rc::new(items));
            }
            Op::ListFilled { dst, r_n, val_v } => {
                let n = regs_i[r_n as usize];
                if n < 0 {
                    return trap(
                        spans[cur_pc],
                        format!("List.filled with negative length {n}"),
                    );
                }
                let val = regs_v[val_v as usize].clone();
                regs_v[dst as usize] = Value::List(Rc::new(vec![val; n as usize]));
            }
            Op::ToList { dst, src_v } => {
                regs_v[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Array(arr) => Value::List(Rc::new(arr.to_values())),
                    Value::List(items) => Value::List(Rc::new((**items).clone())),
                    Value::ListView(view) => Value::List(Rc::new(view.to_vec())),
                    other => other.clone(),
                };
            }
            Op::MakeFill {
                dst,
                dims_base,
                ndims,
                fill_v,
                fill_ty,
            } => {
                let mut dims_vec = Vec::with_capacity(ndims as usize);
                let mut cells: i64 = 1;
                for k in 0..ndims as usize {
                    let len = regs_i[dims_base as usize + k];
                    if len < 0 {
                        return trap(spans[cur_pc], format!("an array can't have length {len}"));
                    }
                    cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                        Some(c) => c,
                        None => {
                            return trap(spans[cur_pc], "this array is too large to allocate");
                        }
                    };
                    dims_vec.push(len);
                }
                let cell = if fill_v == u16::MAX {
                    if fill_ty == u16::MAX {
                        Value::Unit
                    } else {
                        crate::zero(chunk.fill_tys[fill_ty as usize], self.program)
                    }
                } else if fill_v as usize >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[fill_v as usize], Value::Unit)
                } else {
                    regs_v[fill_v as usize].clone()
                };
                regs_v[dst as usize] = Value::Array(Rc::new(Array::filled(dims_vec, cell)));
            }
            Op::ListSlice {
                dst,
                src_v,
                r_lo,
                r_hi,
            } => {
                let lo = regs_i[r_lo as usize];
                let hi = regs_i[r_hi as usize];
                let (view, len) = match &regs_v[src_v as usize] {
                    Value::List(items) => {
                        let n = items.len();
                        (crate::ListViewVal::whole(items.clone(), false), n)
                    }
                    Value::ListView(v) => {
                        let n = v.len;
                        ((**v).clone(), n)
                    }
                    _ => return trap(spans[cur_pc], "slice expects a list"),
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        spans[cur_pc],
                        format!("slice {lo}..{hi} is outside a list of length {len}"),
                    );
                }
                regs_v[dst as usize] =
                    Value::ListView(Rc::new(view.slice(lo as usize, hi as usize)));
            }
            Op::StrSlice {
                dst,
                src_v,
                r_lo,
                r_hi,
            } => {
                let lo = regs_i[r_lo as usize];
                let hi = regs_i[r_hi as usize];
                let (view, len) = match &regs_v[src_v as usize] {
                    Value::Str(bytes) => {
                        let n = bytes.len();
                        (crate::StrViewVal::whole(bytes.clone(), false), n)
                    }
                    Value::StrView(v) => {
                        let n = v.len;
                        ((**v).clone(), n)
                    }
                    _ => return trap(spans[cur_pc], "slice expects a string"),
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        spans[cur_pc],
                        format!("slice {lo}..{hi} is outside a string of length {len}"),
                    );
                }
                regs_v[dst as usize] =
                    Value::StrView(Rc::new(view.slice(lo as usize, hi as usize)));
            }
            Op::TakeLArr1V { dst, arr_v, r_i } => {
                let idx = regs_i[r_i as usize];
                let val = if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                    let len = arr.dims[0];
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {len}"),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => {
                            std::mem::replace(&mut vs[idx as usize], Value::Unit)
                        }
                        _ => a.get(idx as usize).unwrap_or(Value::Unit),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                };
                regs_v[dst as usize] = val;
            }
            Op::TakeGArr1V { dst, gid, r_i } => {
                let idx = regs_i[r_i as usize];
                let val = if let Value::Array(arr) = &mut self.globals[gid as usize] {
                    let len = arr.dims[0];
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside an array dimension of length {len}"),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => {
                            std::mem::replace(&mut vs[idx as usize], Value::Unit)
                        }
                        _ => a.get(idx as usize).unwrap_or(Value::Unit),
                    }
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                };
                regs_v[dst as usize] = val;
            }
            Op::TakeLList1V { dst, list_v, r_i } => {
                let idx = regs_i[r_i as usize];
                let val = if let Value::List(items) = &mut regs_v[list_v as usize] {
                    let len = items.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    std::mem::replace(&mut Rc::make_mut(items)[idx as usize], Value::Unit)
                } else {
                    return trap(spans[cur_pc], "internal: expected list");
                };
                regs_v[dst as usize] = val;
            }
            Op::TakeGList1V { dst, gid, r_i } => {
                let idx = regs_i[r_i as usize];
                let val = if let Value::List(items) = &mut self.globals[gid as usize] {
                    let len = items.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(
                            spans[cur_pc],
                            format!("index {idx} is outside a list of length {len}"),
                        );
                    }
                    std::mem::replace(&mut Rc::make_mut(items)[idx as usize], Value::Unit)
                } else {
                    return trap(spans[cur_pc], "internal: expected list");
                };
                regs_v[dst as usize] = val;
            }
            Op::LoadLArr2V {
                dst,
                arr_v,
                r_i,
                r_j,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let val = if let Value::Array(arr) = &regs_v[arr_v as usize] {
                    let d0 = arr.dims[0];
                    let d1 = arr.dims[1];
                    if i < 0 || i >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {i} is outside an array dimension of length {d0}"),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            format!("index {j} is outside an array dimension of length {d1}"),
                        );
                    }
                    arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                };
                regs_v[dst as usize] = val;
            }
            Op::LoadGArr2V { dst, gid, r_i, r_j } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let val = if let Value::Array(arr) = &self.globals[gid as usize] {
                    let d0 = arr.dims[0];
                    let d1 = arr.dims[1];
                    if i < 0 || i >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {i} is outside an array dimension of length {d0}"),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            format!("index {j} is outside an array dimension of length {d1}"),
                        );
                    }
                    arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                };
                regs_v[dst as usize] = val;
            }
            Op::LoadArrNV { dst, arr_v, idx, n } => {
                let val = match &regs_v[arr_v as usize] {
                    Value::Array(arr) => {
                        let indices = &regs_i[idx as usize..(idx + n) as usize];
                        let flat = nd_flat_index(&arr.dims, indices, spans[cur_pc])?;
                        arr.get(flat).unwrap_or(Value::Unit)
                    }
                    _ => return trap(spans[cur_pc], "internal: expected array"),
                };
                regs_v[dst as usize] = val;
            }
            Op::StoreLArr2V {
                arr_v,
                r_i,
                r_j,
                r_val,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let val = if r_val as usize >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                } else {
                    regs_v[r_val as usize].clone()
                };
                if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                    let d0 = arr.dims[0];
                    let d1 = arr.dims[1];
                    if i < 0 || i >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {i} is outside an array dimension of length {d0}"),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            format!("index {j} is outside an array dimension of length {d1}"),
                        );
                    }
                    Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::StoreGArr2V {
                gid,
                r_i,
                r_j,
                r_val,
            } => {
                let i = regs_i[r_i as usize];
                let j = regs_i[r_j as usize];
                let val = if r_val as usize >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                } else {
                    regs_v[r_val as usize].clone()
                };
                if let Value::Array(arr) = &mut self.globals[gid as usize] {
                    let d0 = arr.dims[0];
                    let d1 = arr.dims[1];
                    if i < 0 || i >= d0 {
                        return trap(
                            spans[cur_pc],
                            format!("index {i} is outside an array dimension of length {d0}"),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            format!("index {j} is outside an array dimension of length {d1}"),
                        );
                    }
                    Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                } else {
                    return trap(spans[cur_pc], "internal: expected array");
                }
            }
            Op::ConstF64 { dst, idx } => {
                regs_v[dst as usize] =
                    Value::Float(f64::from_bits(chunk.consts_i[idx as usize] as u64));
            }
            Op::FloatBin { dst, a, b, op } => {
                let (Value::Float(fa), Value::Float(fb)) =
                    (&regs_v[a as usize], &regs_v[b as usize])
                else {
                    return trap(spans[cur_pc], "internal: expected float");
                };
                let res = match op {
                    BinOp::Add => *fa + *fb,
                    BinOp::Sub => *fa - *fb,
                    BinOp::Mul => *fa * *fb,
                    BinOp::Div => *fa / *fb,
                    BinOp::Pow => fa.powf(*fb),
                    _ => return trap(spans[cur_pc], "internal: unsupported float op"),
                };
                regs_v[dst as usize] = Value::Float(res);
            }
            Op::FloatMinMax { dst, a, b, is_max } => {
                let (Value::Float(fa), Value::Float(fb)) =
                    (&regs_v[a as usize], &regs_v[b as usize])
                else {
                    return trap(spans[cur_pc], "internal: expected float");
                };
                let res = if is_max {
                    if *fa >= *fb {
                        *fa
                    } else {
                        *fb
                    }
                } else if *fa <= *fb {
                    *fa
                } else {
                    *fb
                };
                regs_v[dst as usize] = Value::Float(res);
            }
            Op::CmpV { dst, a_v, b_v, op } => {
                let a = &regs_v[a_v as usize];
                let b = &regs_v[b_v as usize];
                let res = match op {
                    CmpOp::Eq => a == b,
                    CmpOp::Ne => a != b,
                    CmpOp::Lt => a.cmp(b) == std::cmp::Ordering::Less,
                    CmpOp::Le => a.cmp(b) != std::cmp::Ordering::Greater,
                    CmpOp::Gt => a.cmp(b) == std::cmp::Ordering::Greater,
                    CmpOp::Ge => a.cmp(b) != std::cmp::Ordering::Less,
                    _ => false,
                };
                regs_i[dst as usize] = i64::from(res);
            }
            Op::CallVmI {
                fn_id,
                arg_base,
                argc,
                dst,
            } => {
                let f = &self.program.fns[fn_id as usize];
                self.tick(spans[cur_pc])?;
                self.stack_left(spans[cur_pc], Some(&f.name))?;
                let callee = compiled.fns[fn_id as usize].as_ref().unwrap();
                let ab = arg_base as usize;
                let mut arg_buf = [0i64; 8];
                let n = argc as usize;
                arg_buf[..n].copy_from_slice(&regs_i[ab..ab + n]);
                let needs_stack = callee.needs_stack;
                if chunk.has_by_ref_closure {
                    sync_to_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                }
                let saved_base = self.base;
                if needs_stack {
                    self.base = self.stack.len();
                    self.stack.resize(self.base + f.locals.len(), Value::Unit);
                    self.frame_locals.push(&f.locals);
                }
                let frameless = self.frameless_base;
                if !needs_stack {
                    self.frameless_base = Some(saved_base);
                }
                self.call_frames.push((f.name.as_str(), spans[cur_pc]));
                self.suspend(
                    chunk,
                    cur_pc,
                    base_i,
                    regs_v,
                    Body::Fn(usize::from(fn_id)),
                    saved_base,
                );
                let res = self.exec_chunk(compiled, callee, &arg_buf[..n], vm_regs_i);
                self.unsuspend();
                self.frameless_base = frameless;
                if needs_stack {
                    self.frame_locals.pop();
                    self.stack.truncate(self.base);
                    self.base = saved_base;
                }
                let val = res?;
                self.call_frames.pop();
                regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                if chunk.has_by_ref_closure {
                    sync_from_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                }
                regs_i[dst as usize] = ScalarKind::unbox_val(&val);
                self.keep_result(chunk, cur_pc, regs_i, regs_v);
            }
            Op::CallTraced {
                fn_id,
                callee_v,
                arg_base_v,
                argc,
                dst_v,
                dst_i,
                lends,
            } => {
                let id = usize::from(fn_id);
                let span = spans[cur_pc];
                let callee = (fn_id == u16::MAX).then(|| regs_v[callee_v as usize].clone());
                if callee.is_none() {
                    self.stack_left(span, Some(&self.program.fns[id].name))?;
                }
                let abv = arg_base_v as usize;
                let mut args: Vec<Value> = regs_v[abv..abv + argc as usize]
                    .iter_mut()
                    .map(|r| std::mem::replace(r, Value::Unit))
                    .collect();
                let lent = chunk.lent_calls.get(usize::from(lends)).copied();
                let index_regs = abv + argc as usize;
                let caller = self.base;
                sync_to_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    caller,
                    true,
                );
                let outer = self.enter_fallback(chunk, cur_pc, regs_i, regs_v);
                self.show_lent();
                // Lent once the globals and lent values are back in place.
                let returned = match self.lend_args(lent, &mut args, regs_v, index_regs) {
                    Ok(()) => match callee {
                        Some(fv) => self.call_lending(fv, args, &[], span).map(|(v, _)| v),
                        None => {
                            let base = self.stack.len();
                            self.stack.extend(args);
                            let locals = self.program.fns[id].locals.len();
                            self.stack.resize(base + locals, Value::Unit);
                            self.call_at(id, base, span, None)
                        }
                    },
                    Err(e) => Err(e),
                };
                self.hide_lent();
                self.leave_fallback(outer);
                sync_from_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    caller,
                    true,
                );
                let value = returned?;
                if dst_i != u16::MAX {
                    regs_i[dst_i as usize] = ScalarKind::unbox_val(&value);
                } else {
                    regs_v[dst_v as usize] = value;
                }
                self.keep_result(chunk, cur_pc, regs_i, regs_v);
            }
            Op::CallVmMix {
                fn_id,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            } => {
                let f = &self.program.fns[fn_id as usize];
                self.tick(spans[cur_pc])?;
                self.stack_left(spans[cur_pc], Some(&f.name))?;
                let callee = compiled.fns[fn_id as usize].as_ref().unwrap();
                let mut ni = 0usize;
                let mut nv = 0usize;
                for loc in callee.slot_map.iter().take(callee.param_count) {
                    match loc {
                        SlotLoc::I(..) => ni += 1,
                        SlotLoc::OptI { .. } => ni += 2,
                        SlotLoc::V(_) => nv += 1,
                        SlotLoc::SmallTuple { .. } => {
                            unreachable!("params are never SmallTuple")
                        }
                    }
                }
                let mut arg_buf_i = [0i64; 8];
                let abi = arg_base_i as usize;
                arg_buf_i[..ni].copy_from_slice(&regs_i[abi..abi + ni]);
                let mut arg_buf_v: [Value; 8] = std::array::from_fn(|_| Value::Unit);
                let abv = arg_base_v as usize;
                for k in 0..nv {
                    arg_buf_v[k] = std::mem::replace(&mut regs_v[abv + k], Value::Unit);
                }
                let needs_stack = callee.needs_stack;
                if chunk.has_by_ref_closure {
                    sync_to_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                }
                let saved_base = self.base;
                if needs_stack {
                    self.base = self.stack.len();
                    self.stack.resize(self.base + f.locals.len(), Value::Unit);
                    self.frame_locals.push(&f.locals);
                }
                let frameless = self.frameless_base;
                if !needs_stack {
                    self.frameless_base = Some(saved_base);
                }
                self.call_frames.push((f.name.as_str(), spans[cur_pc]));
                self.suspend(
                    chunk,
                    cur_pc,
                    base_i,
                    regs_v,
                    Body::Fn(usize::from(fn_id)),
                    saved_base,
                );
                let res = self.exec_chunk_mix(
                    compiled,
                    callee,
                    &mut arg_buf_i[..ni],
                    &mut arg_buf_v[..nv],
                    &f.params,
                    vm_regs_i,
                );
                self.unsuspend();
                self.frameless_base = frameless;
                if needs_stack {
                    self.frame_locals.pop();
                    self.stack.truncate(self.base);
                    self.base = saved_base;
                }
                let val = res?;
                self.call_frames.pop();
                regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                if chunk.has_by_ref_closure {
                    sync_from_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                }
                if f.params.iter().any(|p| p.mutable) {
                    regs_i[abi..abi + ni].copy_from_slice(&arg_buf_i[..ni]);
                    for k in 0..nv {
                        regs_v[abv + k] = std::mem::replace(&mut arg_buf_v[k], Value::Unit);
                    }
                }
                match ret_mode {
                    CallRet::Unit => {}
                    CallRet::I => regs_i[dst_i as usize] = ScalarKind::unbox_val(&val),
                    CallRet::OptI => {
                        let (v, s) = ScalarKind::unbox_opt(&val);
                        regs_i[dst_i as usize] = v;
                        regs_i[dst_some as usize] = s;
                    }
                    CallRet::V => regs_v[dst_v as usize] = val,
                }
                self.keep_result(chunk, cur_pc, regs_i, regs_v);
            }
            Op::CallClosure {
                callee_v,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            } => {
                if let Value::Closure(cid, _) = &regs_v[callee_v as usize] {
                    if let Some(fast) = compiled.fast_closures[*cid] {
                        let abi = arg_base_i as usize;
                        let res_i = match fast {
                            FastClosure::Param(p) => regs_i[abi + p as usize],
                            FastClosure::Add(a, b) => {
                                let x = regs_i[abi + a as usize];
                                let y = regs_i[abi + b as usize];
                                match x.checked_add(y) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            format!("`{x} + {y}` overflows `Int`"),
                                        );
                                    }
                                }
                            }
                            FastClosure::Sub(a, b) => {
                                let x = regs_i[abi + a as usize];
                                let y = regs_i[abi + b as usize];
                                match x.checked_sub(y) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            format!("`{x} - {y}` overflows `Int`"),
                                        );
                                    }
                                }
                            }
                            FastClosure::Mul(a, b) => {
                                let x = regs_i[abi + a as usize];
                                let y = regs_i[abi + b as usize];
                                match x.checked_mul(y) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            format!("`{x} * {y}` overflows `Int`"),
                                        );
                                    }
                                }
                            }
                            FastClosure::Min(a, b) => {
                                regs_i[abi + a as usize].min(regs_i[abi + b as usize])
                            }
                            FastClosure::Max(a, b) => {
                                regs_i[abi + a as usize].max(regs_i[abi + b as usize])
                            }
                            FastClosure::BitAnd(a, b) => {
                                regs_i[abi + a as usize] & regs_i[abi + b as usize]
                            }
                            FastClosure::BitOr(a, b) => {
                                regs_i[abi + a as usize] | regs_i[abi + b as usize]
                            }
                            FastClosure::BitXor(a, b) => {
                                regs_i[abi + a as usize] ^ regs_i[abi + b as usize]
                            }
                            FastClosure::MulAdd(a, b, c) => {
                                let x = regs_i[abi + a as usize];
                                let y = regs_i[abi + b as usize];
                                let z = regs_i[abi + c as usize];
                                let prod = match x.checked_mul(y) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            format!("`{x} * {y}` overflows `Int`"),
                                        );
                                    }
                                };
                                match prod.checked_add(z) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            format!("`{prod} + {z}` overflows `Int`"),
                                        );
                                    }
                                }
                            }
                        };
                        if callee_v as usize >= chunk.num_local_v {
                            regs_v[callee_v as usize] = Value::Unit;
                        }
                        match ret_mode {
                            CallRet::Unit => {}
                            CallRet::I => regs_i[dst_i as usize] = res_i,
                            CallRet::OptI => {
                                regs_i[dst_i as usize] = res_i;
                                regs_i[dst_some as usize] = 1;
                            }
                            CallRet::V => regs_v[dst_v as usize] = Value::Int(res_i),
                        }
                        self.keep_result(chunk, cur_pc, regs_i, regs_v);
                        return Ok(None);
                    }
                }
                let (cid, caps) = match &regs_v[callee_v as usize] {
                    Value::Closure(cid, caps) => (*cid, Rc::clone(caps)),
                    _ => return trap(spans[cur_pc], "internal: expected closure"),
                };
                if callee_v as usize >= chunk.num_local_v {
                    regs_v[callee_v as usize] = Value::Unit;
                }
                let def = &self.program.closures[cid];
                // The closure may read the place a param of this frame is lent from.
                let lends = self.lends();
                if lends {
                    sync_to_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                    self.show_lent();
                }
                let val = if let Some(callee) = compiled.closures[cid].as_ref() {
                    self.tick(spans[cur_pc])?;
                    self.stack_left(spans[cur_pc], None)?;
                    let mut ni = 0usize;
                    let mut nv = 0usize;
                    for loc in callee.slot_map.iter().take(callee.param_count) {
                        match loc {
                            SlotLoc::I(..) => ni += 1,
                            SlotLoc::OptI { .. } => ni += 2,
                            SlotLoc::V(_) => nv += 1,
                            SlotLoc::SmallTuple { .. } => {
                                unreachable!("params are never SmallTuple")
                            }
                        }
                    }
                    let mut arg_buf_i = [0i64; 8];
                    let abi = arg_base_i as usize;
                    arg_buf_i[..ni].copy_from_slice(&regs_i[abi..abi + ni]);
                    let mut arg_buf_v: [Value; 8] = std::array::from_fn(|_| Value::Unit);
                    let abv = arg_base_v as usize;
                    for k in 0..nv {
                        arg_buf_v[k] = std::mem::replace(&mut regs_v[abv + k], Value::Unit);
                    }
                    let needs_stack = callee.needs_stack;
                    let saved_base = self.base;
                    if needs_stack {
                        self.base = self.stack.len();
                        self.stack.resize(self.base + def.locals.len(), Value::Unit);
                        self.frame_locals.push(&def.locals);
                    }
                    let frameless = self.frameless_base;
                    if !needs_stack {
                        self.frameless_base = Some(saved_base);
                    }
                    self.call_frames.push(("<closure>", spans[cur_pc]));
                    self.suspend(
                        chunk,
                        cur_pc,
                        base_i,
                        regs_v,
                        Body::Closure(cid),
                        saved_base,
                    );
                    let prev_in_closure = std::mem::replace(&mut self.in_closure, true);
                    let c_base_i = vm_regs_i.len();
                    vm_regs_i.resize(c_base_i + callee.num_i, 0);
                    let mut c_regs_v = self.take_regs_v(callee.num_v);
                    let mut ii = 0usize;
                    let mut iv = 0usize;
                    for loc in callee.slot_map.iter().take(callee.param_count) {
                        match *loc {
                            SlotLoc::I(r, _) => {
                                vm_regs_i[c_base_i + r as usize] = arg_buf_i[ii];
                                ii += 1;
                            }
                            SlotLoc::OptI { val, some, .. } => {
                                vm_regs_i[c_base_i + val as usize] = arg_buf_i[ii];
                                vm_regs_i[c_base_i + some as usize] = arg_buf_i[ii + 1];
                                ii += 2;
                            }
                            SlotLoc::V(r) => {
                                c_regs_v[r as usize] =
                                    std::mem::replace(&mut arg_buf_v[iv], Value::Unit);
                                iv += 1;
                            }
                            SlotLoc::SmallTuple { .. } => {
                                unreachable!("params are never SmallTuple")
                            }
                        }
                    }
                    for (cap, cval) in def.captures.iter().zip(caps.iter()) {
                        let crate::CaptureVal::Val(v) = cval else {
                            unreachable!("compiled closures only have value captures")
                        };
                        match callee.slot_map[cap.inner] {
                            SlotLoc::I(r, _) => {
                                vm_regs_i[c_base_i + r as usize] = ScalarKind::unbox_val(v);
                            }
                            SlotLoc::OptI { val, some, .. } => {
                                let (rv, rs) = ScalarKind::unbox_opt(v);
                                vm_regs_i[c_base_i + val as usize] = rv;
                                vm_regs_i[c_base_i + some as usize] = rs;
                            }
                            SlotLoc::SmallTuple { base: b, len, .. } => {
                                if let Value::Tuple(TupleVal::Boxed(fs)) = v {
                                    for (k, f) in fs.iter().take(len as usize).enumerate() {
                                        vm_regs_i[c_base_i + b as usize + k] =
                                            ScalarKind::unbox_val(f);
                                    }
                                }
                            }
                            SlotLoc::V(r) => c_regs_v[r as usize] = v.clone(),
                        }
                    }
                    let dom_mark = self.saved_domains.len();
                    for &d in &callee.guarded_domains {
                        self.saved_domains.push((d, self.domains[d]));
                    }
                    let res =
                        self.exec_chunk_inner(compiled, callee, c_base_i, vm_regs_i, &mut c_regs_v);
                    self.unsuspend();
                    self.frameless_base = frameless;
                    while self.saved_domains.len() > dom_mark {
                        let (id, prev) = self.saved_domains.pop().unwrap();
                        self.domains[id] = prev;
                    }
                    self.return_regs_v(c_regs_v);
                    vm_regs_i.truncate(c_base_i);
                    self.in_closure = prev_in_closure;
                    if needs_stack {
                        self.frame_locals.pop();
                        self.stack.truncate(self.base);
                        self.base = saved_base;
                    }
                    let v = res?;
                    self.call_frames.pop();
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    v
                } else {
                    let mut boxed = Vec::with_capacity(def.params.len());
                    let mut ii = arg_base_i as usize;
                    let mut iv = arg_base_v as usize;
                    for &pslot in &def.params {
                        let ty = &def.locals[pslot].ty;
                        if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
                            boxed.push(kind.box_val(regs_i[ii]));
                            ii += 1;
                        } else if let Some(kind) = ScalarKind::of_opt_ty(ty, self.program) {
                            boxed.push(kind.box_opt(regs_i[ii], regs_i[ii + 1]));
                            ii += 2;
                        } else {
                            boxed.push(std::mem::replace(&mut regs_v[iv], Value::Unit));
                            iv += 1;
                        }
                    }
                    // With `lends` the frame is already on the stack.
                    let has_frame = chunk.needs_stack && !lends;
                    if has_frame {
                        sync_to_stack(
                            &chunk.slot_map,
                            regs_i,
                            regs_v,
                            &mut self.stack,
                            self.base,
                            chunk.traced,
                        );
                    }
                    let res = self.call_value(Value::Closure(cid, caps), boxed, spans[cur_pc]);
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    if has_frame {
                        sync_from_stack(
                            &chunk.slot_map,
                            regs_i,
                            regs_v,
                            &mut self.stack,
                            self.base,
                            chunk.traced,
                        );
                    }
                    res?
                };
                if lends {
                    self.hide_lent();
                    sync_from_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                }
                match ret_mode {
                    CallRet::Unit => {}
                    CallRet::I => regs_i[dst_i as usize] = ScalarKind::unbox_val(&val),
                    CallRet::OptI => {
                        let (v, s) = ScalarKind::unbox_opt(&val);
                        regs_i[dst_i as usize] = v;
                        regs_i[dst_some as usize] = s;
                    }
                    CallRet::V => regs_v[dst_v as usize] = val,
                }
                self.keep_result(chunk, cur_pc, regs_i, regs_v);
            }
            Op::RetI { src } => {
                let raw = regs_i[src as usize];
                let kind = chunk.ret_kind.unwrap_or(ScalarKind::Int);
                return Ok(Some(ExecExit::Return(kind.box_val(raw))));
            }
            Op::RetV { src } => {
                return Ok(Some(ExecExit::Return(std::mem::replace(
                    &mut regs_v[src as usize],
                    Value::Unit,
                ))));
            }
            Op::RetUnit => return Ok(Some(ExecExit::Return(Value::Unit))),
            Op::IsSomeV { dst_i, src_v } => {
                regs_i[dst_i as usize] =
                    i64::from(!matches!(regs_v[src_v as usize], Value::Opt(OptVal::None)));
            }
            Op::UnwrapV { dst_v, src_v } => {
                let v = match &regs_v[src_v as usize] {
                    Value::Opt(OptVal::Int(n)) => Value::Int(*n),
                    Value::Opt(OptVal::Boxed(b)) => (**b).clone(),
                    Value::Opt(OptVal::None) => {
                        return trap(spans[cur_pc], "unwrapped a `None` value");
                    }
                    other => other.clone(),
                };
                if src_v as usize >= chunk.num_local_v {
                    regs_v[src_v as usize] = Value::Unit;
                }
                regs_v[dst_v as usize] = v;
            }
            Op::UnwrapVI { dst_i, src_v } => {
                let v = match &regs_v[src_v as usize] {
                    Value::Opt(OptVal::Int(n)) => *n,
                    Value::Opt(OptVal::Boxed(b)) => ScalarKind::unbox_val(b),
                    Value::Opt(OptVal::None) => {
                        return trap(spans[cur_pc], "unwrapped a `None` value");
                    }
                    other => ScalarKind::unbox_val(other),
                };
                if src_v as usize >= chunk.num_local_v {
                    regs_v[src_v as usize] = Value::Unit;
                }
                regs_i[dst_i as usize] = v;
            }
            Op::WrapSomeV { dst_v, src_v } => {
                let inner = if (src_v as usize) >= chunk.num_local_v {
                    std::mem::replace(&mut regs_v[src_v as usize], Value::Unit)
                } else {
                    regs_v[src_v as usize].clone()
                };
                regs_v[dst_v as usize] = Value::Opt(OptVal::some(inner));
            }
            Op::ConstNoneV { dst_v } => {
                regs_v[dst_v as usize] = Value::Opt(OptVal::None);
            }
            Op::SeqLastV { dst_v, seq_v } | Op::SeqFirstV { dst_v, seq_v } => {
                let last = matches!(op, Op::SeqLastV { .. });
                let opt = self.seq_end(&regs_v[seq_v as usize], last, spans[cur_pc])?;
                if seq_v as usize >= chunk.num_local_v {
                    regs_v[seq_v as usize] = Value::Unit;
                }
                regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(opt));
            }
            Op::SeqLastOptI {
                dst_val,
                dst_some,
                seq_v,
            }
            | Op::SeqFirstOptI {
                dst_val,
                dst_some,
                seq_v,
            } => {
                let last = matches!(op, Op::SeqLastOptI { .. });
                let (v, s) = match self.seq_end(&regs_v[seq_v as usize], last, spans[cur_pc])? {
                    Some(x) => (ScalarKind::unbox_val(&x), 1),
                    None => (0, 0),
                };
                if seq_v as usize >= chunk.num_local_v {
                    regs_v[seq_v as usize] = Value::Unit;
                }
                regs_i[dst_val as usize] = v;
                regs_i[dst_some as usize] = s;
            }
            _ => unreachable!("non-call op routed to exec_fields_and_calls"),
        }
        Ok(None)
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The `mut` arguments of a `CallTraced`, lent as the tree-walker lends
    /// them, their places' indices in `regs_v` from `first` on.
    fn lend_args(
        &mut self,
        lent: Option<&'p [cpl_ir::Arg]>,
        args: &mut [Value],
        regs_v: &mut [Value],
        mut first: usize,
    ) -> Res<()> {
        for (i, arg) in lent.into_iter().flatten().enumerate() {
            let cpl_ir::Arg::Mut(place) = arg else {
                continue;
            };
            let mut exprs = Vec::new();
            place_indices(place, &mut exprs);
            let indices = (first..first + exprs.len())
                .map(|r| std::mem::replace(&mut regs_v[r], Value::Unit))
                .collect();
            first += exprs.len();
            args[i] = self.lend_at(place, indices)?;
        }
        Ok(())
    }
}
