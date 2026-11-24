//! A run from a checkpoint on the VM: a frame's chunk entered again with
//! its locals, at the top of its loop or at the statement whose call it
//! waits on.

use crate::checkpoint::{chunk_of, Body, FrameAt};
use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{trap, Flow, Interp, Res};

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    /// Frame `k` of `frames`, its locals on the stack at `self.base`, on the
    /// VM from where it was to its end.
    pub(crate) fn resume_frame(&mut self, frames: &[FrameAt], k: usize) -> Res<Flow> {
        let frame = &frames[k];
        let compiled = std::rc::Rc::clone(&self.vm);
        let base = self.base;
        let exit = match chunk_of(&compiled, frame.body) {
            Some(_) => self.run_body_vm(&compiled, frame.body, Some((frames, k))),
            None => Ok(Value::Unit),
        };
        if (self.unwait() || chunk_of(&compiled, frame.body).is_none()) && exit.is_ok() {
            return trap(
                frame.span,
                cpl_diag::msgs::interp::CHECKPOINT_DOES_NOT_MATCH_PROGRAM.at(&[]),
            );
        }
        if let Body::Fn(id) = frame.body {
            let params = &self.program.fns[id].params;
            if params.iter().any(|p| p.mutable) {
                self.ckpt.lent_back = self.stack[base..base + params.len()].to_vec();
            }
        }
        let v = exit?;
        Ok(match frame.body {
            Body::Main => Flow::Next,
            Body::Setup if matches!(v, Value::Int(1)) => Flow::Next,
            _ => Flow::Return(v),
        })
    }

    /// In a run from a checkpoint, whether call op `op` at `pc` is one the
    /// statement waiting on a call made, or that call itself; then what it
    /// returned is in its destination and its `mut` arguments are back.
    #[cold]
    #[inline(never)]
    pub(super) fn resume_op(
        &mut self,
        op: Op,
        pc: usize,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<bool> {
        let (ret, dst, args, fn_id) = match op {
            Op::CallVmI { dst, .. } => (CallRet::I, (dst, 0, 0), (0, 0), None),
            Op::CallVmMix {
                fn_id,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
                ..
            } => {
                let args = (arg_base_i as usize, arg_base_v as usize);
                (ret_mode, (dst_i, dst_some, dst_v), args, Some(fn_id))
            }
            Op::CallClosure {
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
                ..
            } => (ret_mode, (dst_i, dst_some, dst_v), (0, 0), None),
            _ => return Ok(false),
        };
        let Some(val) = self.resumed_call(chunk, pc)? else {
            return Ok(false);
        };
        if let Some(callee) = fn_id.and_then(|id| compiled.fns[id as usize].as_ref()) {
            if !self.ckpt.lent_back.is_empty() {
                self.lend_back_resumed(callee, regs_i, regs_v, args);
            }
        }
        put_ret(ret, regs_i, regs_v, dst, val);
        Ok(true)
    }

    /// The registers of fed fn `chunk`, at `base_i` and `regs_v`, as frame 0
    /// of `frames` had them; where it starts again.
    pub(super) fn resume_regs(
        &mut self,
        chunk: &Chunk<'p>,
        frames: &[FrameAt],
        base_i: usize,
        vm_regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<usize> {
        let locals = &frames[0].locals;
        let at = match chunk.needs_stack {
            true => self.base,
            false => self.stack.len(),
        };
        self.stack.truncate(at);
        self.stack.extend(locals.iter().cloned());
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        sync_from_stack(
            &chunk.slot_map,
            regs_i,
            regs_v,
            &mut self.stack,
            at,
            chunk.traced,
        );
        if !chunk.needs_stack {
            self.stack.truncate(at);
        }
        self.resume_at(chunk, frames, 0, regs_i, regs_v)
    }

    /// What a call op that a run from a checkpoint resumes writes back to
    /// its `mut` arguments, from `arg_i` and `arg_v` on, in `callee`'s
    /// parameter order.
    pub(super) fn lend_back_resumed(
        &mut self,
        callee: &Chunk<'p>,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
        (arg_i, arg_v): (usize, usize),
    ) {
        let back = std::mem::take(&mut self.ckpt.lent_back);
        let (mut ii, mut iv) = (arg_i, arg_v);
        for (loc, v) in callee.slot_map.iter().take(callee.param_count).zip(back) {
            match *loc {
                SlotLoc::I(..) => {
                    regs_i[ii] = ScalarKind::unbox_val(&v);
                    ii += 1;
                }
                SlotLoc::OptI { .. } => {
                    (regs_i[ii], regs_i[ii + 1]) = ScalarKind::unbox_opt(&v);
                    ii += 2;
                }
                SlotLoc::V(_) => {
                    regs_v[iv] = v;
                    iv += 1;
                }
                SlotLoc::SmallTuple { len, .. } => {
                    unbox_small_tuple(&v, &mut regs_i[ii..], len);
                    ii += usize::from(len);
                }
            }
        }
    }
}

/// Writes what a call returned to its destination by `ret`.
pub(super) fn put_ret(
    ret: CallRet,
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    (dst_i, dst_some, dst_v): (u16, u16, u16),
    val: Value,
) {
    match ret {
        CallRet::Unit => {}
        CallRet::I => regs_i[dst_i as usize] = ScalarKind::unbox_val(&val),
        CallRet::OptI => {
            let (v, s) = ScalarKind::unbox_opt(&val);
            regs_i[dst_i as usize] = v;
            regs_i[dst_some as usize] = s;
        }
        CallRet::V => regs_v[dst_v as usize] = val,
        CallRet::Tuple(len) => {
            if let Value::Tuple(crate::value::TupleVal::Boxed(fs)) = &val {
                for (k, f) in fs.iter().take(len as usize).enumerate() {
                    regs_i[dst_i as usize + k] = ScalarKind::unbox_val(f);
                }
            }
        }
    }
}
