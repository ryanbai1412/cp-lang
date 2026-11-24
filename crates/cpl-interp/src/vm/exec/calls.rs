use super::resume::put_ret;
use crate::checkpoint::Body;
use crate::observe::Observer;
use crate::value::{Array, Cells, OptVal, TupleVal, Value};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{bug, trap, Flow, Interp, Res};
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
        if self.ckpt.waits() && self.resume_op(op, cur_pc, compiled, chunk, regs_i, regs_v)? {
            return Ok(None);
        }
        match op {
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
                                cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                    .at(&[&idx, &len]),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, idx as usize);
                    } else {
                        return bug(spans[cur_pc], "expected array field");
                    }
                } else {
                    return bug(spans[cur_pc], "expected struct");
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
                                cpl_diag::msgs::interp::INDEX_OUTSIDE_STRING_LENGTH
                                    .at(&[&idx, &s.len()]),
                            );
                        }
                        regs_i[dst as usize] = i64::from(s[idx as usize]);
                    }
                    Value::StrView(sv) => {
                        if idx < 0 || (idx as usize) >= sv.len {
                            return trap(
                                spans[cur_pc],
                                cpl_diag::msgs::interp::INDEX_OUTSIDE_STRING_LENGTH
                                    .at(&[&idx, &sv.len]),
                            );
                        }
                        let pos = if sv.rev {
                            sv.lo + sv.len - 1 - idx as usize
                        } else {
                            sv.lo + idx as usize
                        };
                        regs_i[dst as usize] = i64::from(sv.base[pos]);
                    }
                    _ => return bug(spans[cur_pc], "expected string"),
                }
            }
            Op::GStrGetI { dst, gid, r_i } => {
                let idx = regs_i[r_i as usize];
                match &self.globals[gid as usize] {
                    Value::Str(s) => {
                        if idx < 0 || (idx as usize) >= s.len() {
                            return trap(
                                spans[cur_pc],
                                cpl_diag::msgs::interp::INDEX_OUTSIDE_STRING_LENGTH
                                    .at(&[&idx, &s.len()]),
                            );
                        }
                        regs_i[dst as usize] = i64::from(s[idx as usize]);
                    }
                    Value::StrView(sv) => {
                        if idx < 0 || (idx as usize) >= sv.len {
                            return trap(
                                spans[cur_pc],
                                cpl_diag::msgs::interp::INDEX_OUTSIDE_STRING_LENGTH
                                    .at(&[&idx, &sv.len]),
                            );
                        }
                        let pos = if sv.rev {
                            sv.lo + sv.len - 1 - idx as usize
                        } else {
                            sv.lo + idx as usize
                        };
                        regs_i[dst as usize] = i64::from(sv.base[pos]);
                    }
                    _ => return bug(spans[cur_pc], "expected string"),
                }
            }
            Op::MakeTupleI {
                dst,
                base_i,
                len,
                kind,
            } => {
                let b = base_i as usize;
                let fs =
                    crate::value::Shared::collect(len as usize, |k| kind.box_val(regs_i[b + k]));
                regs_v[dst as usize] = Value::Tuple(TupleVal::Boxed(fs));
            }
            Op::MakeTuple { dst, base_v, len } => {
                let b = base_v as usize;
                let n = len as usize;
                if n == 1 {
                    let v = std::mem::replace(&mut regs_v[b], Value::Unit);
                    regs_v[dst as usize] = match v {
                        Value::Int(x) => Value::Tuple(TupleVal::OneInt(x)),
                        other => {
                            Value::Tuple(TupleVal::Boxed(crate::value::Shared::new(vec![other])))
                        }
                    };
                } else {
                    let fs = crate::value::Shared::collect(n, |k| {
                        std::mem::replace(&mut regs_v[b + k], Value::Unit)
                    });
                    regs_v[dst as usize] = Value::Tuple(TupleVal::Boxed(fs));
                }
            }
            Op::MakeList { dst, base_v, len } => {
                let b = base_v as usize;
                let n = len as usize;
                let mut items = Vec::with_capacity(n);
                for k in 0..n {
                    items.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                }
                regs_v[dst as usize] = Value::List(Rc::new(crate::value::Cells::from(items)));
            }
            Op::ListFilled { dst, r_n, val_v } => {
                let n = regs_i[r_n as usize];
                if n < 0 {
                    return trap(
                        spans[cur_pc],
                        cpl_diag::msgs::interp::LIST_FILLED_WITH_NEGATIVE_LENGTH.at(&[&n]),
                    );
                }
                let val = regs_v[val_v as usize].clone();
                regs_v[dst as usize] =
                    Value::List(Rc::new(crate::value::Cells::filled(n as usize, val)));
            }
            Op::ToList { dst, src_v } => {
                regs_v[dst as usize] = match &regs_v[src_v as usize] {
                    Value::Array(arr) => Value::List(Rc::new(arr.data.clone())),
                    Value::List(items) => Value::List(Rc::new((**items).clone())),
                    Value::ListView(view) => {
                        Value::List(Rc::new(crate::value::Cells::from(view.to_vec())))
                    }
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
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::ARRAY_CAN_HAVE_LENGTH.at(&[&len]),
                        );
                    }
                    cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                        Some(c) => c,
                        None => {
                            return trap(
                                spans[cur_pc],
                                cpl_diag::msgs::interp::THIS_ARRAY_TOO_LARGE_TO.at(&[]),
                            );
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
                    _ => {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::SLICE_EXPECTS_LIST.at(&[]),
                        )
                    }
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        spans[cur_pc],
                        cpl_diag::msgs::interp::SLICE_OUTSIDE_LIST_LENGTH.at(&[&lo, &hi, &len]),
                    );
                }
                regs_v[dst as usize] = Value::ListView(crate::value::Shared::new(
                    view.slice(lo as usize, hi as usize),
                ));
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
                    _ => {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::SLICE_EXPECTS_STRING.at(&[]),
                        )
                    }
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        spans[cur_pc],
                        cpl_diag::msgs::interp::SLICE_OUTSIDE_STRING_LENGTH.at(&[&lo, &hi, &len]),
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&idx, &len]),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        Cells::Values(vs) => std::mem::replace(&mut vs[idx as usize], Value::Unit),
                        _ => a.get(idx as usize).unwrap_or(Value::Unit),
                    }
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&idx, &len]),
                        );
                    }
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        Cells::Values(vs) => std::mem::replace(&mut vs[idx as usize], Value::Unit),
                        _ => a.get(idx as usize).unwrap_or(Value::Unit),
                    }
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                        );
                    }
                    std::mem::replace(
                        &mut Rc::make_mut(items).as_mut_values()[idx as usize],
                        Value::Unit,
                    )
                } else {
                    return bug(spans[cur_pc], "expected list");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &len]),
                        );
                    }
                    std::mem::replace(
                        &mut Rc::make_mut(items).as_mut_values()[idx as usize],
                        Value::Unit,
                    )
                } else {
                    return bug(spans[cur_pc], "expected list");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&i, &d0]),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&j, &d1]),
                        );
                    }
                    arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&i, &d0]),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&j, &d1]),
                        );
                    }
                    arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                    _ => return bug(spans[cur_pc], "expected array"),
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&i, &d0]),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&j, &d1]),
                        );
                    }
                    Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&i, &d0]),
                        );
                    }
                    if j < 0 || j >= d1 {
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_ARRAY_DIMENSION_LENGTH
                                .at(&[&j, &d1]),
                        );
                    }
                    Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                } else {
                    return bug(spans[cur_pc], "expected array");
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
                    return bug(spans[cur_pc], "expected float");
                };
                let res = match op {
                    BinOp::Add => *fa + *fb,
                    BinOp::Sub => *fa - *fb,
                    BinOp::Mul => *fa * *fb,
                    BinOp::Div => *fa / *fb,
                    BinOp::Pow => fa.powf(*fb),
                    _ => return bug(spans[cur_pc], "unsupported float op"),
                };
                regs_v[dst as usize] = Value::Float(res);
            }
            Op::FloatMinMax { dst, a, b, is_max } => {
                let (Value::Float(fa), Value::Float(fb)) =
                    (&regs_v[a as usize], &regs_v[b as usize])
                else {
                    return bug(spans[cur_pc], "expected float");
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
                let Some(callee) = compiled.fns[fn_id as usize].as_ref() else {
                    return trap(
                        spans[cur_pc],
                        cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                    );
                };
                let ab = arg_base as usize;
                let mut arg_buf = [0i64; 8];
                let n = argc as usize;
                arg_buf[..n].copy_from_slice(&regs_i[ab..ab + n]);
                let needs_stack = callee.needs_stack;
                if chunk.has_by_ref_closure {
                    sync_refs_to_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
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
                    sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
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
                    chunk.traced,
                );
                let outer = self.enter_fallback(chunk, cur_pc, regs_i, regs_v);
                self.show_lent()?;
                // Lent once the globals and lent values are back in place.
                let resumed = match self.ckpt.waits() {
                    true => {
                        let lends = lent.filter(|_| callee.is_none());
                        let at = (&mut args[..], lends, &mut regs_v[..], index_regs);
                        self.resume_traced(chunk, cur_pc, at)
                    }
                    false => Ok(None),
                };
                let returned = match resumed {
                    Ok(Some(value)) => Ok(value),
                    Err(e) => Err(e),
                    Ok(None) => match (callee, lent) {
                        (Some(fv), Some(lent)) => {
                            self.call_moving_lent(fv, args, lent, regs_v, index_regs, span)
                        }
                        (callee, _) => match self.lend_args(lent, &mut args, regs_v, index_regs) {
                            Ok(()) => match callee {
                                Some(fv) => self.call_lending(fv, args, &[], span).map(|(v, _)| v),
                                None => {
                                    let base = self.stack.len();
                                    self.stack.extend(args);
                                    let locals = self.program.fns[id].locals.len();
                                    self.stack.resize(base + locals, Value::Unit);
                                    let feed = (chunk.traced_feeds.iter())
                                        .find(|f| f.0 == cur_pc as u32)
                                        .map(|f| f.1);
                                    match feed {
                                        Some(feed) => {
                                            self.call_traced_feed(chunk, feed, id, base, span)
                                        }
                                        None => self.call_at(id, base, span, None),
                                    }
                                }
                            },
                            Err(e) => Err(e),
                        },
                    },
                };
                self.hide_lent()?;
                self.leave_fallback(outer);
                sync_from_stack(
                    &chunk.slot_map,
                    regs_i,
                    regs_v,
                    &mut self.stack,
                    caller,
                    chunk.traced,
                );
                let value = returned?;
                if let Some(Flow::Return(v) | Flow::Give(v)) = self.fed_left.take() {
                    return Ok(Some(ExecExit::Return(v)));
                }
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
                let Some(callee) = compiled.fns[fn_id as usize].as_ref() else {
                    return trap(
                        spans[cur_pc],
                        cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                    );
                };
                let mut ni = 0usize;
                let mut nv = 0usize;
                for loc in callee.slot_map.iter().take(callee.param_count) {
                    match loc {
                        SlotLoc::I(..) => ni += 1,
                        SlotLoc::OptI { .. } => ni += 2,
                        SlotLoc::V(_) => nv += 1,
                        SlotLoc::SmallTuple { len, .. } => ni += *len as usize,
                    }
                }
                let mut arg_buf_i = [0i64; 16];
                let abi = arg_base_i as usize;
                arg_buf_i[..ni].copy_from_slice(&regs_i[abi..abi + ni]);
                let abv = arg_base_v as usize;
                let needs_stack = callee.needs_stack;
                if chunk.has_by_ref_closure {
                    sync_refs_to_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
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
                    &mut regs_v[abv..abv + nv],
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
                let exit = res?;
                self.call_frames.pop();
                regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                if chunk.has_by_ref_closure {
                    sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                }
                if f.params.iter().any(|p| p.mutable) {
                    regs_i[abi..abi + ni].copy_from_slice(&arg_buf_i[..ni]);
                }
                // A tuple the caller takes in registers stays in `ret_tuple`.
                match (ret_mode, exit) {
                    (CallRet::Tuple(len), ExecExit::ReturnTuple) => {
                        let (d, n) = (dst_i as usize, len as usize);
                        regs_i[d..d + n].copy_from_slice(&self.ret_tuple[..n]);
                    }
                    (_, exit) => {
                        let val = self.exit_value(exit);
                        put_ret(ret_mode, regs_i, regs_v, (dst_i, dst_some, dst_v), val);
                    }
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
                muts,
            } => {
                let forward = match &regs_v[callee_v as usize] {
                    Value::Closure(cid, _) if muts == 0 => compiled.forwards[*cid],
                    _ => None,
                };
                if let Some(fn_id) = forward {
                    if callee_v as usize >= chunk.num_local_v {
                        regs_v[callee_v as usize] = Value::Unit;
                    }
                    let call = Op::CallVmMix {
                        fn_id,
                        arg_base_i,
                        arg_base_v,
                        dst_i,
                        dst_v,
                        dst_some,
                        ret_mode,
                    };
                    let (bi, rv) = (base_i, regs_v);
                    return self
                        .exec_fields_and_calls(call, cur_pc, compiled, chunk, bi, vm_regs_i, rv);
                }
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
                                            cpl_diag::msgs::interp::OVERFLOWS_INT_BINOP
                                                .at(&[&x, &"+", &y]),
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
                                            cpl_diag::msgs::interp::OVERFLOWS_INT_BINOP
                                                .at(&[&x, &"-", &y]),
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
                                            cpl_diag::msgs::interp::OVERFLOWS_INT_BINOP
                                                .at(&[&x, &"*", &y]),
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
                                            cpl_diag::msgs::interp::OVERFLOWS_INT_BINOP
                                                .at(&[&x, &"*", &y]),
                                        );
                                    }
                                };
                                match prod.checked_add(z) {
                                    Some(v) if v != i64::MIN => v,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            cpl_diag::msgs::interp::OVERFLOWS_INT_BINOP
                                                .at(&[&prod, &"+", &z]),
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
                            CallRet::Tuple(_) => {
                                return bug(spans[cur_pc], "a tuple from an Int closure")
                            }
                        }
                        self.keep_result(chunk, cur_pc, regs_i, regs_v);
                        return Ok(None);
                    }
                }
                let (cid, caps) = match &regs_v[callee_v as usize] {
                    Value::Closure(cid, caps) => (*cid, Rc::clone(caps)),
                    _ => return bug(spans[cur_pc], "expected closure"),
                };
                if callee_v as usize >= chunk.num_local_v {
                    regs_v[callee_v as usize] = Value::Unit;
                }
                let def = &self.program.closures[cid];
                // The closure may read the place a param of this frame is lent from.
                let lends = self.lends();
                if lends {
                    let (lent, base) = (&self.lent, self.base);
                    sync_reachable_to_stack(chunk, lent, regs_i, regs_v, &mut self.stack, base);
                    self.show_lent()?;
                }
                // The `mut` params' values, for their arg slots once the frame is synced.
                let mut lent_back: Vec<Value> = Vec::new();
                // A variable the closure holds by reference is lent to its frame
                // on the stack for the call, as `run_chunk_lending` lends it.
                let by_val = caps.iter().all(|c| matches!(c, crate::CaptureVal::Val(_)));
                let lendable = |c: &&Chunk<'p>| {
                    let tuple = |cap: &cpl_ir::Capture| {
                        cap.by_ref && matches!(c.slot_map[cap.inner], SlotLoc::SmallTuple { .. })
                    };
                    by_val || !c.traced && !def.captures.iter().any(tuple)
                };
                // With `lends` the frame is already on the stack.
                let has_frame = chunk.needs_stack && !lends;
                let val = if let Some(callee) = compiled.closures[cid].as_ref().filter(lendable) {
                    self.tick(spans[cur_pc])?;
                    self.stack_left(spans[cur_pc], None)?;
                    if !by_val && has_frame {
                        sync_refs_to_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                    }
                    let needs_stack = callee.needs_stack || !by_val;
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
                    // The args go straight from this frame's registers to the callee's.
                    let c_base_i = vm_regs_i.len();
                    vm_regs_i.resize(c_base_i + callee.num_i, 0);
                    let mut c_regs_v = self.take_regs_v(callee.num_v);
                    let mut ii = base_i + arg_base_i as usize;
                    let mut iv = arg_base_v as usize;
                    for loc in callee.slot_map.iter().take(callee.param_count) {
                        match *loc {
                            SlotLoc::I(r, _) => {
                                vm_regs_i[c_base_i + r as usize] = vm_regs_i[ii];
                                ii += 1;
                            }
                            SlotLoc::OptI { val, some, .. } => {
                                vm_regs_i[c_base_i + val as usize] = vm_regs_i[ii];
                                vm_regs_i[c_base_i + some as usize] = vm_regs_i[ii + 1];
                                ii += 2;
                            }
                            SlotLoc::V(r) => {
                                c_regs_v[r as usize] =
                                    std::mem::replace(&mut regs_v[iv], Value::Unit);
                                iv += 1;
                            }
                            SlotLoc::SmallTuple { base, len, .. } => {
                                let n = len as usize;
                                vm_regs_i.copy_within(ii..ii + n, c_base_i + base as usize);
                                ii += n;
                            }
                        }
                    }
                    self.suspend(
                        chunk,
                        cur_pc,
                        base_i,
                        regs_v,
                        Body::Closure(cid),
                        saved_base,
                    );
                    let prev_in_closure = std::mem::replace(&mut self.in_closure, true);
                    let lent_mark = self.lent.len();
                    for (cap, cval) in def.captures.iter().zip(caps.iter()) {
                        let (loc, c_regs_i) =
                            (&callee.slot_map[cap.inner], &mut vm_regs_i[c_base_i..]);
                        let part = match cval {
                            crate::CaptureVal::Val(v) => {
                                write_slot(loc, c_regs_i, &mut c_regs_v, v.clone());
                                continue;
                            }
                            &crate::CaptureVal::StackRef(at) => self.alias_part(false, at),
                            &crate::CaptureVal::GlobalRef(id) => self.alias_part(true, id),
                            crate::CaptureVal::PartRef(part) => Rc::clone(part),
                        };
                        let v = std::mem::replace(self.lender(&part)?, Value::Moved);
                        write_slot(loc, c_regs_i, &mut c_regs_v, v);
                        self.lent.push((self.base + cap.inner, part));
                    }
                    let dom_mark = self.saved_domains.len();
                    for &d in &callee.guarded_domains {
                        self.saved_domains.push((d, self.domains[d]));
                    }
                    let res = self.exec_chunk_inner(
                        compiled,
                        callee,
                        c_base_i,
                        vm_regs_i,
                        &mut c_regs_v,
                        0,
                    );
                    let c_regs_i = &vm_regs_i[c_base_i..];
                    let held = self.lent.split_off(lent_mark);
                    self.give_back(callee, held, c_regs_i, &mut c_regs_v)?;
                    if res.is_ok() && muts != 0 {
                        for (k, &pslot) in def.params.iter().enumerate() {
                            if muts & 1 << k == 0 {
                                continue;
                            }
                            match take_slot(&callee.slot_map[pslot], c_regs_i, &mut c_regs_v) {
                                Some(v) => lent_back.push(v),
                                None => {
                                    return trap(
                                        spans[cur_pc],
                                        cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                                    )
                                }
                            }
                        }
                    }
                    self.unsuspend();
                    self.frameless_base = frameless;
                    self.restore_domains(dom_mark);
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
                    if !by_val && has_frame {
                        sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                    }
                    v
                } else {
                    let mut boxed = Vec::with_capacity(def.params.len());
                    let mut ii = arg_base_i as usize;
                    let mut iv = arg_base_v as usize;
                    for (k, &pslot) in def.params.iter().enumerate() {
                        let ty = &def.locals[pslot].ty;
                        let tuple = (muts & 1 << k == 0)
                            .then(|| crate::vm::scalar::small_tuple_shape(ty, self.program))
                            .flatten();
                        if let Some((len, kinds)) = tuple {
                            let fs = crate::value::Shared::collect(len as usize, |f| {
                                kinds[f].box_val(regs_i[ii + f])
                            });
                            boxed.push(Value::Tuple(TupleVal::Boxed(fs)));
                            ii += len as usize;
                        } else if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
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
                    // The closure reaches this frame only through what it
                    // and its kin captured by reference.
                    if has_frame {
                        sync_refs_to_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                    }
                    let lent: Vec<usize> = (0..def.params.len())
                        .filter(|k| muts & 1 << k != 0)
                        .collect();
                    let res =
                        self.call_lending(Value::Closure(cid, caps), boxed, &lent, spans[cur_pc]);
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    let res = res.map(|(val, back)| {
                        lent_back = back;
                        val
                    });
                    if has_frame {
                        sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                    }
                    res?
                };
                if lends {
                    self.hide_lent()?;
                    let (lent, base) = (&self.lent, self.base);
                    sync_reachable_from_stack(chunk, lent, regs_i, regs_v, &mut self.stack, base);
                }
                if !lent_back.is_empty() {
                    let mut back = lent_back.into_iter();
                    let (mut ii, mut iv) = (arg_base_i as usize, arg_base_v as usize);
                    for (k, &pslot) in def.params.iter().enumerate() {
                        let ty = &def.locals[pslot].ty;
                        let v = if muts & 1 << k != 0 {
                            back.next()
                        } else {
                            None
                        };
                        if ScalarKind::of_ty(ty, self.program).is_some() {
                            if let Some(v) = v {
                                regs_i[ii] = ScalarKind::unbox_val(&v);
                            }
                            ii += 1;
                        } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                            if let Some(v) = v {
                                (regs_i[ii], regs_i[ii + 1]) = ScalarKind::unbox_opt(&v);
                            }
                            ii += 2;
                        } else {
                            if let Some(v) = v {
                                regs_v[iv] = v;
                            }
                            iv += 1;
                        }
                    }
                }
                put_ret(ret_mode, regs_i, regs_v, (dst_i, dst_some, dst_v), val);
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
            Op::RetSmallTuple { base, len, kinds } => {
                let (b, n) = (base as usize, len as usize);
                self.ret_tuple[..n].copy_from_slice(&regs_i[b..b + n]);
                self.ret_tuple_len = len;
                self.ret_tuple_kinds = kinds;
                return Ok(Some(ExecExit::ReturnTuple));
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
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::UNWRAPPED_NONE_VALUE.at(&[]),
                        );
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
                        return trap(
                            spans[cur_pc],
                            cpl_diag::msgs::interp::UNWRAPPED_NONE_VALUE.at(&[]),
                        );
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
            _ => {
                return trap(
                    spans[cur_pc],
                    cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                )
            }
        }
        Ok(None)
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The `mut` arguments of a `CallTraced`, lent as the tree-walker lends
    /// them, their places' indices in `regs_v` from `first` on.
    pub(super) fn lend_args(
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
