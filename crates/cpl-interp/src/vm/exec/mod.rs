use crate::observe::Observer;
use crate::value::{TupleVal, Value};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{describe, trap, Interp, Res};
use cpl_ir::{CmpOp, FnId, LoopId};
use std::rc::Rc;

mod calls;
mod containers;
mod places;
mod trace;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    fn take_regs_v(&mut self, n: usize) -> Vec<Value> {
        if n == 0 {
            return Vec::new();
        }
        let mut v = self.vm_pool_v.pop().unwrap_or_default();
        v.clear();
        v.resize(n, Value::Unit);
        v
    }

    fn return_regs_v(&mut self, mut v: Vec<Value>) {
        if v.capacity() > 0 && self.vm_pool_v.len() < 64 {
            v.clear();
            self.vm_pool_v.push(v);
        }
    }

    pub(crate) fn run_main_vm(&mut self, compiled: &CompiledProgram<'p>) -> Res<()> {
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        if vm_regs_i.capacity() < 4096 {
            vm_regs_i.reserve(4096);
        }
        let res = self.exec_chunk(compiled, &compiled.main, &[], &mut vm_regs_i);
        self.vm_regs_i = vm_regs_i;
        let _ = res?;
        Ok(())
    }

    pub(crate) fn run_fn_vm_from_stack(
        &mut self,
        compiled: &CompiledProgram<'p>,
        fn_id: FnId,
    ) -> Res<Value> {
        let chunk = compiled.fns[fn_id].as_ref().unwrap();
        let mut aliases: Vec<(usize, bool, usize)> = Vec::new();
        for k in 0..chunk.param_count {
            if let Value::Alias(is_global, at) = self.stack[self.base + k] {
                let val = if is_global {
                    std::mem::replace(&mut self.globals[at], Value::Moved)
                } else {
                    std::mem::replace(&mut self.stack[at], Value::Moved)
                };
                self.stack[self.base + k] = val;
                aliases.push((k, is_global, at));
            }
        }
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        let mut regs_v = self.take_regs_v(chunk.num_v);
        sync_from_stack(
            &chunk.slot_map,
            &mut vm_regs_i[base_i..base_i + chunk.num_i],
            &mut regs_v,
            &mut self.stack,
            self.base,
            chunk.traced,
        );
        let res = self.exec_chunk_inner(compiled, chunk, base_i, &mut vm_regs_i, &mut regs_v);
        for (k, is_global, at) in aliases {
            let val = match chunk.slot_map[k] {
                SlotLoc::I(r, kind) => kind.box_val(vm_regs_i[base_i + r as usize]),
                SlotLoc::OptI { val, some, kind } => kind.box_opt(
                    vm_regs_i[base_i + val as usize],
                    vm_regs_i[base_i + some as usize],
                ),
                SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
                SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
            };
            if is_global {
                self.globals[at] = val;
            } else {
                self.stack[at] = val;
            }
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        self.vm_regs_i = vm_regs_i;
        res
    }

    fn exec_chunk(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        args_i: &[i64],
        vm_regs_i: &mut Vec<i64>,
    ) -> Res<Value> {
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        for (i, &v) in args_i.iter().enumerate() {
            if let Some(SlotLoc::I(r, _)) = chunk.slot_map.get(i) {
                unsafe {
                    *vm_regs_i.get_unchecked_mut(base_i + *r as usize) = v;
                }
            }
        }
        let mut regs_v = self.take_regs_v(chunk.num_v);
        let dom_mark = self.saved_domains.len();
        for &d in &chunk.guarded_domains {
            self.saved_domains.push((d, self.domains[d]));
        }
        let res = self.exec_chunk_inner(compiled, chunk, base_i, vm_regs_i, &mut regs_v);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        res
    }

    fn exec_chunk_mix(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        args_i: &mut [i64],
        args_v: &mut [Value],
        params: &[cpl_ir::ParamDef],
        vm_regs_i: &mut Vec<i64>,
    ) -> Res<Value> {
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        let mut regs_v = self.take_regs_v(chunk.num_v);
        let mut ii = 0usize;
        let mut iv = 0usize;
        for loc in chunk.slot_map.iter().take(chunk.param_count) {
            match *loc {
                SlotLoc::I(r, _) => {
                    unsafe {
                        *vm_regs_i.get_unchecked_mut(base_i + r as usize) = args_i[ii];
                    }
                    ii += 1;
                }
                SlotLoc::OptI { val, some, .. } => {
                    unsafe {
                        *vm_regs_i.get_unchecked_mut(base_i + val as usize) = args_i[ii];
                        *vm_regs_i.get_unchecked_mut(base_i + some as usize) = args_i[ii + 1];
                    }
                    ii += 2;
                }
                SlotLoc::V(r) => {
                    regs_v[r as usize] = std::mem::replace(&mut args_v[iv], Value::Unit);
                    iv += 1;
                }
                SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
            }
        }
        let dom_mark = self.saved_domains.len();
        for &d in &chunk.guarded_domains {
            self.saved_domains.push((d, self.domains[d]));
        }
        let res = self.exec_chunk_inner(compiled, chunk, base_i, vm_regs_i, &mut regs_v);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        if res.is_ok() && params.iter().any(|p| p.mutable) {
            let mut ii = 0usize;
            let mut iv = 0usize;
            for (k, loc) in chunk.slot_map.iter().take(chunk.param_count).enumerate() {
                let is_mut = params[k].mutable;
                match *loc {
                    SlotLoc::I(r, _) => {
                        if is_mut {
                            args_i[ii] = vm_regs_i[base_i + r as usize];
                        }
                        ii += 1;
                    }
                    SlotLoc::OptI { val, some, .. } => {
                        if is_mut {
                            args_i[ii] = vm_regs_i[base_i + val as usize];
                            args_i[ii + 1] = vm_regs_i[base_i + some as usize];
                        }
                        ii += 2;
                    }
                    SlotLoc::V(r) => {
                        if is_mut {
                            args_v[iv] = std::mem::replace(&mut regs_v[r as usize], Value::Unit);
                        }
                        iv += 1;
                    }
                    SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
                }
            }
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        res
    }

    fn exec_chunk_inner(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
    ) -> Res<Value> {
        let exit = self.exec_chunk_core(compiled, chunk, base_i, vm_regs_i, regs_v, 0, None);
        if chunk.traced {
            // The end of the run and a trap show the locals as they were left.
            let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
            sync_to_stack(
                &chunk.slot_map,
                regs_i,
                regs_v,
                &mut self.stack,
                self.base,
                true,
            );
            if exit.is_err() {
                self.keep_trap_frame();
            }
        }
        match exit? {
            ExecExit::Return(v) => Ok(v),
            ExecExit::FeedNext | ExecExit::FeedBreak => Ok(Value::Unit),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn exec_chunk_core(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
        start_pc: usize,
        mut fed: Option<&mut FedCtx<'_, 'p>>,
    ) -> Res<ExecExit> {
        let ops = chunk.ops.as_slice();
        let spans = chunk.spans.as_slice();
        let mut pc: usize = start_pc;
        let mut regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        let mut escaped: Option<(bool, Option<LoopId>)> = None;

        loop {
            let op = unsafe { *ops.get_unchecked(pc) };
            let cur_pc = pc;
            pc += 1;
            match op {
                Op::ConstImmI { dst, val } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) = i64::from(val);
                },
                Op::ConstI { dst, idx } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) =
                        *chunk.consts_i.get_unchecked(idx as usize);
                },
                Op::MoveI { dst, src } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) = *regs_i.get_unchecked(src as usize);
                },
                Op::MoveV { dst, src } => {
                    let v = if src as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                    } else {
                        regs_v[src as usize].clone()
                    };
                    regs_v[dst as usize] = v;
                }
                Op::MoveVConsume { dst, src } => {
                    regs_v[dst as usize] =
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit);
                }
                Op::BoxI { dst_v, src_i, kind } => {
                    regs_v[dst_v as usize] = kind.box_val(regs_i[src_i as usize]);
                }
                Op::UnboxI { dst_i, src_v } => {
                    regs_i[dst_i as usize] = ScalarKind::unbox_val(&regs_v[src_v as usize]);
                }
                Op::BoxOptI {
                    dst_v,
                    val,
                    some,
                    kind,
                } => {
                    regs_v[dst_v as usize] =
                        kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                }
                Op::UnboxOptI {
                    dst_val,
                    dst_some,
                    src_v,
                } => {
                    let (v, s) = ScalarKind::unbox_opt(&regs_v[src_v as usize]);
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::LoadGlobalI { dst, gid } => {
                    regs_i[dst as usize] = ScalarKind::unbox_val(&self.globals[gid as usize]);
                }
                Op::StoreGlobalI { gid, src, kind } => {
                    set_scalar(&mut self.globals[gid as usize], kind, regs_i[src as usize]);
                }
                Op::LoadGlobalOptI {
                    dst_val,
                    dst_some,
                    gid,
                } => {
                    let (v, s) = ScalarKind::unbox_opt(&self.globals[gid as usize]);
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::StoreGlobalOptI {
                    gid,
                    val,
                    some,
                    kind,
                } => {
                    self.globals[gid as usize] =
                        kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                }
                Op::LoadGlobalV { dst, gid } => {
                    regs_v[dst as usize] = self.globals[gid as usize].clone();
                }
                Op::StoreGlobalV { gid, src } => {
                    self.globals[gid as usize] = if src as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                    } else {
                        regs_v[src as usize].clone()
                    };
                }
                Op::StoreGlobalVMove { gid, src } => {
                    self.globals[gid as usize] =
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit);
                }
                Op::AddI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_add(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::SubI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_sub(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} - {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::MulI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_mul(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} * {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::DivI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("division by zero: `{av} / 0`"));
                    }
                    match av.checked_div_euclid(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} / {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::RemI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("division by zero: `{av} % 0`"));
                    }
                    regs_i[dst as usize] = av.checked_rem_euclid(bv).unwrap_or(0);
                }
                Op::ModAddI { dst, a, b, modulus } => {
                    let s = regs_i[a as usize] + regs_i[b as usize];
                    let m = modulus as i64;
                    regs_i[dst as usize] = if s >= m { s - m } else { s };
                }
                Op::ModSubI { dst, a, b, modulus } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let m = modulus as i64;
                    regs_i[dst as usize] = if av >= bv { av - bv } else { av - bv + m };
                }
                Op::ModMulI { dst, a, b, modulus } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let m = modulus as i64;
                    let prod = av * bv;
                    regs_i[dst as usize] = if prod >= 0 {
                        prod % m
                    } else {
                        prod.rem_euclid(m)
                    };
                }
                Op::ModNegI { dst, a, modulus } => {
                    let av = regs_i[a as usize];
                    regs_i[dst as usize] = if av == 0 { 0 } else { modulus as i64 - av };
                }
                Op::AddImmI { dst, a, imm } => {
                    let av = regs_i[a as usize];
                    let bv = i64::from(imm);
                    match av.checked_add(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::NegI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_neg().filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "integer overflow: `-{av}` does not fit in an `Int` (64 bits)"
                                ),
                            )
                        }
                    }
                }
                Op::BitAndI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av & bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} & {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::BitOrI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av | bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} | {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::BitXorI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av ^ bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} ^ {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::ShlI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("a shift by {bv} is outside 0..64"));
                    }
                    if av < 0 {
                        return trap(spans[cur_pc], format!("`<<` of the negative number {av}"));
                    }
                    if bv > 0 && av > (i64::MAX >> bv) {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} << {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = av << bv;
                }
                Op::ShrI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("a shift by {bv} is outside 0..64"));
                    }
                    regs_i[dst as usize] = av >> bv;
                }
                Op::NotBool { dst, a } => {
                    regs_i[dst as usize] = i64::from(regs_i[a as usize] == 0);
                }
                Op::MinI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].min(regs_i[b as usize]);
                }
                Op::MaxI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].max(regs_i[b as usize]);
                }
                Op::WrapAddI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_add(regs_i[b as usize]);
                }
                Op::WrapSubI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_sub(regs_i[b as usize]);
                }
                Op::WrapMulI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_mul(regs_i[b as usize]);
                }
                Op::AbsI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_abs().filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => {
                            return trap(
                                spans[cur_pc],
                                format!(
                                "integer overflow: `abs({av})` does not fit in an `Int` (64 bits)"
                            ),
                            )
                        }
                    }
                }
                Op::DomainLen { dst, dom } => {
                    regs_i[dst as usize] = self.domains[dom as usize].1;
                }
                Op::DomainLo { dst, dom } => {
                    regs_i[dst as usize] = self.domains[dom as usize].0;
                }
                Op::FromLabel { dst, src, dom } => {
                    let (lo, len) = self.domains[dom as usize];
                    let label = regs_i[src as usize];
                    if label < lo || label - lo >= len {
                        let name = &self.program.domains[dom as usize];
                        return trap(
                            spans[cur_pc],
                            format!("{label} is not in `{name}`, which is {}", describe(lo, len)),
                        );
                    }
                    regs_i[dst as usize] = label - lo;
                }
                Op::FromOrd { dst, src, dom } => {
                    let (lo, len) = self.domains[dom as usize];
                    let ord = regs_i[src as usize];
                    if ord < 0 || ord >= len {
                        let name = &self.program.domains[dom as usize];
                        return trap(
                            spans[cur_pc],
                            format!(
                                "`{name}` has no position {ord}: it is {}",
                                describe(lo, len)
                            ),
                        );
                    }
                    regs_i[dst as usize] = ord;
                }
                Op::LoadGArr1SmallTuple {
                    dst_base,
                    len,
                    gid,
                    r_i,
                } => {
                    let i = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple(i as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr1SmallTuple {
                    dst_base,
                    len,
                    arr_v,
                    r_i,
                } => {
                    let i = regs_i[r_i as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple(i as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1SmallTuple {
                    gid,
                    r_i,
                    src_base,
                    len,
                    kinds,
                } => {
                    let i = regs_i[r_i as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(i as usize, n, &regs_i[b..b + n], &kinds);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr1SmallTuple {
                    arr_v,
                    r_i,
                    src_base,
                    len,
                    kinds,
                } => {
                    let i = regs_i[r_i as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(i as usize, n, &regs_i[b..b + n], &kinds);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2SmallTuple {
                    dst_base,
                    len,
                    gid,
                    r_i,
                    r_j,
                } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index ({i}, {j}) is outside an array of shape ({d0}, {d1})"
                                ),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple((i * d1 + j) as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr2SmallTuple {
                    dst_base,
                    len,
                    arr_v,
                    r_i,
                    r_j,
                } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index ({i}, {j}) is outside an array of shape ({d0}, {d1})"
                                ),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple((i * d1 + j) as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr2SmallTuple {
                    gid,
                    r_i,
                    r_j,
                    src_base,
                    len,
                    kinds,
                } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index ({i}, {j}) is outside an array of shape ({d0}, {d1})"
                                ),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(
                            (i * d1 + j) as usize,
                            n,
                            &regs_i[b..b + n],
                            &kinds,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr2SmallTuple {
                    arr_v,
                    r_i,
                    r_j,
                    src_base,
                    len,
                    kinds,
                } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index ({i}, {j}) is outside an array of shape ({d0}, {d1})"
                                ),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(
                            (i * d1 + j) as usize,
                            n,
                            &regs_i[b..b + n],
                            &kinds,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::BoxSmallTuple {
                    dst_v,
                    src_base,
                    len,
                    kinds,
                } => {
                    let b = src_base as usize;
                    let n = len as usize;
                    let mut fs = Vec::with_capacity(n);
                    for k in 0..n {
                        fs.push(kinds[k].box_val(regs_i[b + k]));
                    }
                    regs_v[dst_v as usize] = Value::Tuple(TupleVal::Boxed(Rc::new(fs)));
                }
                Op::UnboxSmallTuple {
                    dst_base,
                    len,
                    src_v,
                } => {
                    let b = dst_base as usize;
                    let n = len as usize;
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[src_v as usize] {
                        for (k, f) in fs.iter().take(n).enumerate() {
                            regs_i[b + k] = ScalarKind::unbox_val(f);
                        }
                    }
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                }
                Op::PosNext {
                    dst_val,
                    dst_some,
                    src,
                    len,
                } => {
                    let next = regs_i[src as usize] + 1;
                    let ok = next < regs_i[len as usize];
                    regs_i[dst_val as usize] = next;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosPrev {
                    dst_val,
                    dst_some,
                    src,
                } => {
                    let ord = regs_i[src as usize];
                    let ok = ord > 0;
                    regs_i[dst_val as usize] = ord - 1;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosOffset {
                    dst_val,
                    dst_some,
                    src,
                    delta,
                    dom,
                } => {
                    let len = self.domains[dom as usize].1;
                    let target = regs_i[src as usize].saturating_add(regs_i[delta as usize]);
                    let ok = target >= 0 && target < len;
                    regs_i[dst_val as usize] = target;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosTryOrd {
                    dst_val,
                    dst_some,
                    src,
                    dom,
                } => {
                    let len = self.domains[dom as usize].1;
                    let ord = regs_i[src as usize];
                    let ok = ord >= 0 && ord < len;
                    regs_i[dst_val as usize] = ord;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::BoundStepL { dst, ord, k, dom } => {
                    let len = self.domains[dom as usize].1;
                    let kv = regs_i[k as usize];
                    let b = regs_i[ord as usize] + 1 - kv;
                    if b < 0 || b > len {
                        return trap(
                            spans[cur_pc],
                            format!("window of {kv} leaves domain (0..={len})"),
                        );
                    }
                    regs_i[dst as usize] = b;
                }
                Op::BoundStepR { dst, ord, k, dom } => {
                    let len = self.domains[dom as usize].1;
                    let kv = regs_i[k as usize];
                    let b = regs_i[ord as usize] + kv;
                    if b < 0 || b > len {
                        return trap(
                            spans[cur_pc],
                            format!("window of {kv} leaves domain (0..={len})"),
                        );
                    }
                    regs_i[dst as usize] = b;
                }
                Op::UnwrapOptI { dst, val, some } => {
                    if regs_i[some as usize] == 0 {
                        return trap(spans[cur_pc], "`!` on `None`: there is no value here");
                    }
                    regs_i[dst as usize] = regs_i[val as usize];
                }
                Op::CmpI { dst, a, b, op } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let res = match op {
                        CmpOp::Lt => av < bv,
                        CmpOp::Le => av <= bv,
                        CmpOp::Gt => av > bv,
                        CmpOp::Ge => av >= bv,
                        CmpOp::Eq => av == bv,
                        CmpOp::Ne => av != bv,
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(res);
                }
                Op::Jmp { target } => {
                    if target as usize <= cur_pc && self.tick(spans[cur_pc])? && fed.is_none() {
                        self.take_checkpoint(compiled, chunk, cur_pc, base_i, vm_regs_i, regs_v);
                        regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    }
                    pc = target as usize
                }
                Op::JmpEscape {
                    brk,
                    id,
                    innermost,
                    target,
                } => {
                    if let Some((was_brk, was)) = escaped {
                        if was_brk == brk && (was == Some(id) || (was.is_none() && innermost)) {
                            escaped = None;
                            pc = target as usize;
                        }
                    }
                }
                Op::JmpIfZero { reg, target } => {
                    if regs_i[reg as usize] == 0 {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpIfNonZero { reg, target } => {
                    if regs_i[reg as usize] != 0 {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpLtI { a, b, target } => {
                    if regs_i[a as usize] < regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpLeI { a, b, target } => {
                    if regs_i[a as usize] <= regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpGtI { a, b, target } => {
                    if regs_i[a as usize] > regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpGeI { a, b, target } => {
                    if regs_i[a as usize] >= regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpEqI { a, b, target } => {
                    if regs_i[a as usize] == regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::JmpNeI { a, b, target } => {
                    if regs_i[a as usize] != regs_i[b as usize] {
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::ForInc { r_i, r_hi, target } => {
                    // The variable keeps its last value after the loop, as
                    // on the tree-walker, so a checkpoint reads the same.
                    let next = regs_i[r_i as usize] + 1;
                    if next < regs_i[r_hi as usize] {
                        regs_i[r_i as usize] = next;
                        if target as usize <= cur_pc && self.tick(spans[cur_pc])? && fed.is_none() {
                            self.take_checkpoint(
                                compiled, chunk, cur_pc, base_i, vm_regs_i, regs_v,
                            );
                            regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        }
                        pc = target as usize;
                    }
                }
                Op::ForIncClosed { r_i, r_hi, target } => {
                    let cur = regs_i[r_i as usize];
                    if cur < regs_i[r_hi as usize] {
                        regs_i[r_i as usize] = cur + 1;
                        if target as usize <= cur_pc && self.tick(spans[cur_pc])? && fed.is_none() {
                            self.take_checkpoint(
                                compiled, chunk, cur_pc, base_i, vm_regs_i, regs_v,
                            );
                            regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        }
                        pc = target as usize;
                    }
                }
                Op::ForDec { r_i, r_lo, target } => {
                    let cur = regs_i[r_i as usize];
                    if cur > regs_i[r_lo as usize] {
                        regs_i[r_i as usize] = cur - 1;
                        if target as usize <= cur_pc {
                            self.tick(spans[cur_pc])?;
                        }
                        pc = target as usize;
                    }
                }
                Op::LoadGArr1I { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
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
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1I { gid, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), idx as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2I { dst, gid, r_i, r_j } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, (i0 * l1 + i1) as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreGArr2I {
                    gid,
                    r_i,
                    r_j,
                    r_val,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadLArr1I { dst, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[arr_v as usize] {
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
                            regs_i[dst as usize] = read_array_i(arr, idx as usize);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::StoreLArr1I { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    match &mut regs_v[arr_v as usize] {
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
                            let slot = &mut Rc::make_mut(list)[idx as usize];
                            *slot = match slot {
                                Value::Bool(_) => Value::Bool(val != 0),
                                Value::Char(_) => Value::Char(val as u8),
                                Value::Float(_) => Value::Float(f64::from_bits(val as u64)),
                                Value::U64(_) => Value::U64(val as u64),
                                _ => Value::Int(val),
                            };
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::LoadLArr2I {
                    dst,
                    arr_v,
                    r_i,
                    r_j,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, (i0 * l1 + i1) as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreLArr2I {
                    arr_v,
                    r_i,
                    r_j,
                    r_val,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadGArr1OptI {
                    dst_val,
                    dst_some,
                    gid,
                    r_i,
                } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, idx as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1OptI {
                    gid,
                    r_i,
                    r_val,
                    r_some,
                    kind,
                } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), idx as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2OptI {
                    dst_val,
                    dst_some,
                    gid,
                    r_i,
                    r_j,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, (i0 * l1 + i1) as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreGArr2OptI {
                    gid,
                    r_i,
                    r_j,
                    r_val,
                    r_some,
                    kind,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_opt_i(
                            Rc::make_mut(arr),
                            (i0 * l1 + i1) as usize,
                            val,
                            some,
                            kind,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadLArr1OptI {
                    dst_val,
                    dst_some,
                    arr_v,
                    r_i,
                } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, idx as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr1OptI {
                    arr_v,
                    r_i,
                    r_val,
                    r_some,
                    kind,
                } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), idx as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr2OptI {
                    dst_val,
                    dst_some,
                    arr_v,
                    r_i,
                    r_j,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, (i0 * l1 + i1) as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreLArr2OptI {
                    arr_v,
                    r_i,
                    r_j,
                    r_val,
                    r_some,
                    kind,
                } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_opt_i(
                            Rc::make_mut(arr),
                            (i0 * l1 + i1) as usize,
                            val,
                            some,
                            kind,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadGList1I { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &self.globals[gid as usize] {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] =
                                ScalarKind::unbox_val(view.get(idx as usize).unwrap());
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::StoreGList1I {
                    gid,
                    r_i,
                    r_val,
                    kind,
                } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::List(list) = &mut self.globals[gid as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::LoadLList1I { dst, list_v, r_i } => {
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
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] =
                                ScalarKind::unbox_val(view.get(idx as usize).unwrap());
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::StoreLList1I {
                    list_v,
                    r_i,
                    r_val,
                    kind,
                } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::List(list) = &mut regs_v[list_v as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::GArrLen { dst, gid } => {
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        regs_i[dst as usize] = arr.len() as i64;
                    } else {
                        regs_i[dst as usize] = 0;
                    }
                }
                Op::LArrLen { dst, arr_v } => {
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        regs_i[dst as usize] = arr.len() as i64;
                    } else {
                        regs_i[dst as usize] = 0;
                    }
                }
                Op::GListLen { dst, gid } => {
                    regs_i[dst as usize] = match &self.globals[gid as usize] {
                        Value::List(list) => list.len() as i64,
                        Value::ListView(view) => view.len as i64,
                        _ => 0,
                    };
                }
                Op::LListLen { dst, list_v } => {
                    regs_i[dst as usize] = match &regs_v[list_v as usize] {
                        Value::List(list) => list.len() as i64,
                        Value::ListView(view) => view.len as i64,
                        _ => 0,
                    };
                }
                Op::LoadGArr1V { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        regs_v[dst as usize] = arr.get(idx as usize).unwrap_or(Value::Unit);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1V { gid, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "index {idx} is outside an array dimension of length {len}"
                                ),
                            );
                        }
                        Rc::make_mut(arr).set(idx as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr1V { dst, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[arr_v as usize] {
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
                            regs_v[dst as usize] = arr.get(idx as usize).unwrap_or(Value::Unit);
                        }
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
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::StoreLArr1V { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    match &mut regs_v[arr_v as usize] {
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
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::Trace { idx } => {
                    self.exec_trace(chunk, idx, regs_i, regs_v);
                }
                other => {
                    if let Some(exit) = self.exec_ext_op(
                        other,
                        cur_pc,
                        compiled,
                        chunk,
                        base_i,
                        vm_regs_i,
                        regs_v,
                        &mut escaped,
                        &mut fed,
                    )? {
                        return Ok(exit);
                    }
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn exec_ext_op(
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
        match op {
            Op::FieldI { .. }
            | Op::FieldOptI { .. }
            | Op::FieldV { .. }
            | Op::LoadFieldList1I { .. }
            | Op::FieldListLen { .. }
            | Op::LoadFieldArr1I { .. }
            | Op::FieldArrLen { .. }
            | Op::StrGetI { .. }
            | Op::GStrGetI { .. }
            | Op::MakeTuple { .. }
            | Op::MakeList { .. }
            | Op::ListFilled { .. }
            | Op::ToList { .. }
            | Op::MakeFill { .. }
            | Op::ListSlice { .. }
            | Op::StrSlice { .. }
            | Op::TakeLArr1V { .. }
            | Op::TakeGArr1V { .. }
            | Op::TakeLList1V { .. }
            | Op::TakeGList1V { .. }
            | Op::LoadLArr2V { .. }
            | Op::LoadGArr2V { .. }
            | Op::StoreLArr2V { .. }
            | Op::StoreGArr2V { .. }
            | Op::ConstF64 { .. }
            | Op::FloatBin { .. }
            | Op::FloatMinMax { .. }
            | Op::CmpV { .. }
            | Op::CallVmI { .. }
            | Op::CallVmMix { .. }
            | Op::CallClosure { .. }
            | Op::RetI { .. }
            | Op::RetV { .. }
            | Op::RetUnit
            | Op::IsSomeV { .. }
            | Op::UnwrapV { .. }
            | Op::UnwrapVI { .. }
            | Op::WrapSomeV { .. }
            | Op::ConstNoneV { .. }
            | Op::SeqLastV { .. }
            | Op::SeqLastOptI { .. }
            | Op::SeqFirstV { .. }
            | Op::SeqFirstOptI { .. } => {
                self.exec_fields_and_calls(op, cur_pc, compiled, chunk, base_i, vm_regs_i, regs_v)
            }
            Op::SeqLenI { .. }
            | Op::RangeLoI { .. }
            | Op::RangeHiI { .. }
            | Op::BitCeilI { .. }
            | Op::PopcountI { .. }
            | Op::CtzI { .. }
            | Op::ClzI { .. }
            | Op::MinMaxGArr1SmallTuple { .. }
            | Op::MinMaxLArr1SmallTuple { .. }
            | Op::ListPushGArr1I { .. }
            | Op::ListPushGArr1V { .. }
            | Op::ListPushLArr1I { .. }
            | Op::ListPushLArr1V { .. }
            | Op::ListPushLV { .. }
            | Op::ListPushGV { .. }
            | Op::ListPushLI { .. }
            | Op::ListPushGI { .. }
            | Op::ListPopLV { .. }
            | Op::ListPopGV { .. }
            | Op::ListPopOptIL { .. }
            | Op::ListPopOptIG { .. }
            | Op::ListPopDiscardL { .. }
            | Op::ListPopDiscardG { .. }
            | Op::LoadGlobalVMove { .. }
            | Op::StoreLFieldI { .. }
            | Op::StoreLFieldV { .. }
            | Op::StoreLFieldOptI { .. }
            | Op::StoreLFieldIdx1I { .. }
            | Op::StoreLFieldIdx1V { .. }
            | Op::StoreLFieldIdx1OptI { .. }
            | Op::ListPushFieldLI { .. }
            | Op::ListPushFieldLV { .. }
            | Op::ListPopDiscardFieldL { .. }
            | Op::StoreGList1V { .. }
            | Op::StoreLList1V { .. }
            | Op::LoadGList1V { .. }
            | Op::LoadLList1V { .. }
            | Op::LoadLList1OptI { .. }
            | Op::ListPopFieldOptIL { .. }
            | Op::ListPopFieldLV { .. }
            | Op::MakeRange { .. }
            | Op::CmpOptI { .. }
            | Op::LoadGIdxFieldI { .. }
            | Op::StoreGIdxFieldI { .. }
            | Op::LoadLIdxFieldI { .. }
            | Op::StoreLIdxFieldI { .. } => {
                let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                self.exec_seq_and_list(op, cur_pc, chunk, regs_i, regs_v)?;
                Ok(None)
            }
            _ => self.exec_places_and_fallback(
                op, cur_pc, compiled, chunk, base_i, vm_regs_i, regs_v, escaped, fed,
            ),
        }
    }
}
