//! VM calls on heap frames. A call op pushes its caller's frame and the
//! callee runs in the same loop, so a program's recursion takes heap, not
//! Rust stack, and recurses as deep in a browser as on a judge.

use super::resume::put_ret;
use super::*;
use crate::checkpoint::{chunk_of, Body};
use crate::save_domains;
use cpl_diag::Span;
use cpl_ir::ClosureId;
use std::rc::Rc;

/// Where a frame's `break` or `continue` out of a statement it fell back
/// to is headed.
pub(super) type Escaped = Option<(bool, Option<LoopId>)>;

/// How a frame's ops stopped.
pub(super) enum Stop<'c, 'p> {
    Exit(ExecExit),
    /// A call op made its callee's frame, to run next.
    Call(Callee<'c, 'p>),
}

/// A callee's frame, made by the call op at `pc` of its caller.
pub(super) struct Callee<'c, 'p> {
    pub(super) chunk: &'c Chunk<'p>,
    pub(super) body: Body,
    pub(super) base_i: usize,
    pub(super) regs_v: Vec<Value>,
    pub(super) pc: usize,
    pub(super) back: Back,
}

/// What a call op puts back once its callee returns, and where the result goes.
pub(super) struct Back {
    saved_base: usize,
    frameless: Option<usize>,
    needs_stack: bool,
    dom_mark: usize,
    ret: Ret,
    /// A leaf closure's captures, whose by-reference variables it borrowed
    /// in place.
    pub(super) leaf: Option<Rc<Vec<crate::CaptureVal>>>,
    /// The lent places a closure's by-reference captures were shown at.
    pub(super) shown: Vec<(usize, Rc<crate::value::Part>)>,
}

#[derive(Clone, Copy)]
pub(super) enum Ret {
    /// `CallVmI` or `CallVmMix`.
    Fn(FnRet),
    /// `CallClosure` on the VM, and whether the caller ran in a closure.
    Closure(ClosureRet, bool),
}

/// Where a `CallVmI`'s or `CallVmMix`'s result goes, and the `mut` params
/// of fn `fn_id`, back to its args at `args`.
#[derive(Clone, Copy)]
pub(super) struct FnRet {
    pub(super) fn_id: u16,
    pub(super) args: (u16, u16),
    pub(super) ret_mode: CallRet,
    pub(super) dst: (u16, u16, u16),
    /// The callee takes a fn value, which may read a place the caller holds lent.
    pub(super) lend_back: bool,
}

/// Where a `CallClosure`'s result and `mut` args go.
#[derive(Clone, Copy)]
pub(super) struct ClosureRet {
    pub(super) cid: ClosureId,
    pub(super) muts: u16,
    pub(super) args: (u16, u16),
    pub(super) ret_mode: CallRet,
    pub(super) dst: (u16, u16, u16),
    /// The caller's lent places were shown to the closure.
    pub(super) lends: bool,
    /// Where the closure's frame's share of [`Interp::lent`] starts, when it
    /// holds variables by reference.
    pub(super) held: Option<usize>,
    /// The caller's frame synced its by-reference locals to the stack.
    pub(super) refs: bool,
}

/// A caller waiting on its callee: where it goes on. Its chunk is
/// `body`'s, or with none the chunk the run of frames started in.
pub(crate) struct VmFrame {
    body: Option<Body>,
    base_i: usize,
    regs_v: Vec<Value>,
    pc: usize,
    escaped: Escaped,
    back: Back,
    /// What the callee's frame adds to [`Interp::frame_bytes`].
    bytes: usize,
}

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    /// Runs `chunk` from `start_pc` with its int registers at `base_i` and in
    /// `regs_v`, and every VM call it makes, each on a heap frame, until it
    /// returns or, fed by `fed`, ends a round of the loop it feeds.
    pub(super) fn exec_chunk_core<'c>(
        &mut self,
        compiled: &'c CompiledProgram<'p>,
        (chunk, base_i, start_pc): (&'c Chunk<'p>, usize, usize),
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
        mut fed: Option<&mut FedCtx<'_, 'p>>,
    ) -> Res<ExecExit> {
        let (entry, entry_v) = (chunk, regs_v);
        let mut frames = self.vm_frames.pop().unwrap_or_default();
        let (mut chunk, mut base_i, mut pc) = (chunk, base_i, start_pc);
        let mut body = None;
        // The registers of the innermost frame, once it is a callee's.
        let mut own: Vec<Value> = Vec::new();
        let mut escaped: Escaped = None;
        let mut unfed = None;
        loop {
            let (regs_v, fed_here) = match frames.is_empty() {
                true => (&mut *entry_v, &mut fed),
                false => (&mut own[..], &mut unfed),
            };
            let stop = self.run_ops(
                compiled,
                chunk,
                base_i,
                vm_regs_i,
                regs_v,
                pc,
                &mut escaped,
                fed_here,
            );
            let mut ended = match stop {
                Ok(Stop::Call(callee)) => {
                    let bytes = frame_bytes(callee.chunk);
                    self.frame_bytes += bytes;
                    frames.push(VmFrame {
                        body: body.replace(callee.body),
                        base_i,
                        regs_v: std::mem::replace(&mut own, callee.regs_v),
                        pc: callee.pc,
                        escaped: escaped.take(),
                        back: callee.back,
                        bytes,
                    });
                    (chunk, base_i, pc) = (callee.chunk, callee.base_i, 0);
                    continue;
                }
                Ok(Stop::Exit(exit)) => Ok(exit),
                Err(e) => Err(e),
            };
            // The innermost frame ended: back to its caller, or on out with
            // a trap until a caller is left.
            loop {
                let Some(frame) = frames.pop() else {
                    self.vm_frames.push(frames);
                    return ended;
                };
                self.frame_bytes -= frame.bytes;
                let callee = (chunk, base_i, std::mem::replace(&mut own, frame.regs_v));
                // The chunk the call op that made this frame's callee ran in.
                let up = match frame.body {
                    None => Some(entry),
                    Some(b) => chunk_of(compiled, b),
                };
                let Some(up) = up else {
                    self.vm_frames.push(frames);
                    return Err(crate::interp_failed(Span::default()));
                };
                let caller = (up, frame.base_i, frame.pc);
                let caller_v = match frames.is_empty() {
                    true => &mut *entry_v,
                    false => &mut own[..],
                };
                let back = self.return_to(callee, vm_regs_i, ended, frame.back, caller, caller_v);
                (chunk, base_i, pc, escaped) = (up, frame.base_i, frame.pc + 1, frame.escaped);
                body = frame.body;
                match back {
                    Ok(()) => break,
                    Err(e) => ended = Err(e),
                }
            }
        }
    }

    /// The call op at `pc` of `chunk`, whose registers are at `base_i` and
    /// in `regs_v`, to fn `fn_id` with its args at `args`: the callee's frame.
    pub(super) fn call_fn_vm<'c>(
        &mut self,
        compiled: &'c CompiledProgram<'p>,
        mut call: FnRet,
        (chunk, cur_pc, base_i): (&Chunk<'p>, usize, usize),
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
    ) -> Res<Option<Stop<'c, 'p>>> {
        let FnRet { fn_id, args, .. } = call;
        let spans = chunk.spans.as_slice();
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        let f = &self.program.fns[fn_id as usize];
        let span = spans[cur_pc];
        self.tick(span)?;
        self.stack_left(span, Some(&f.name))?;
        let Some(callee) = compiled.fns[fn_id as usize].as_ref() else {
            return self.no_body(fn_id as usize, span);
        };
        // A fn value passed on may read a place this frame holds lent.
        call.lend_back = callee.takes_fn && self.lends();
        if call.lend_back {
            let (lent, base) = (&self.lent, self.base);
            sync_reachable_to_stack(chunk, lent, regs_i, regs_v, &mut self.stack, base);
            self.show_lent()?;
        } else if chunk.has_by_ref_closure {
            sync_refs_to_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
        }
        let (c_base_i, mut c_regs_v) = self.callee_regs(callee, vm_regs_i);
        let from = (base_i + args.0 as usize, args.1 as usize);
        pass_args(callee, vm_regs_i, c_base_i, regs_v, &mut c_regs_v, from);
        let at = (chunk, cur_pc, base_i, &*regs_v);
        let body = Body::Fn(usize::from(fn_id));
        let entered = self.enter_vm_call(
            callee.needs_stack,
            &f.locals,
            f.name.as_str(),
            span,
            at,
            body,
        );
        let back = self.vm_back(callee, entered, Ret::Fn(call));
        Ok(Some(Stop::Call(Callee {
            chunk: callee,
            body,
            base_i: c_base_i,
            regs_v: c_regs_v,
            pc: cur_pc,
            back,
        })))
    }

    /// Before a VM call from `caller` (its chunk, the call's pc, its
    /// registers) at `span` to `body` with `locals`, named `name`: its
    /// frame on the stack when it `needs_stack`, and what the call
    /// stack and the checkpoints see. Gives the caller's base and frameless
    /// base, for [`Self::vm_back`].
    pub(super) fn enter_vm_call(
        &mut self,
        needs_stack: bool,
        locals: &'p [cpl_ir::Local],
        name: &'p str,
        span: Span,
        (chunk, pc, base_i, regs_v): (&Chunk<'p>, usize, usize, &[Value]),
        body: Body,
    ) -> (usize, Option<usize>, bool) {
        let saved_base = self.base;
        if needs_stack {
            self.base = self.stack.len();
            self.stack.resize(self.base + locals.len(), Value::Unit);
            self.frame_locals.push(locals);
        }
        let frameless = self.frameless_base;
        if !needs_stack {
            self.frameless_base = Some(saved_base);
        }
        self.call_frames.push((name, span));
        self.suspend(chunk, pc, base_i, regs_v, body, saved_base);
        (saved_base, frameless, needs_stack)
    }

    /// The registers of a frame for `callee`.
    pub(super) fn callee_regs(
        &mut self,
        callee: &Chunk<'p>,
        vm_regs_i: &mut Vec<i64>,
    ) -> (usize, Vec<Value>) {
        let base = vm_regs_i.len();
        vm_regs_i.resize(base + callee.num_i, 0);
        (base, self.take_regs_v(callee.num_v))
    }

    /// What the call [`Self::enter_vm_call`] `entered` puts back, once
    /// `callee`'s guarded domains are saved.
    pub(super) fn vm_back(
        &mut self,
        callee: &Chunk<'p>,
        (saved_base, frameless, needs_stack): (usize, Option<usize>, bool),
        ret: Ret,
    ) -> Back {
        Back {
            saved_base,
            frameless,
            needs_stack,
            dom_mark: save_domains(
                &mut self.saved_domains,
                &self.domains,
                &callee.guarded_domains,
            ),
            ret,
            leaf: None,
            shown: Vec::new(),
        }
    }

    /// `callee` (its chunk, registers) ended with `value`: what its call
    /// op set up undone, and on a return, the result and `mut` args in the
    /// caller (its chunk, registers, the call's pc).
    fn return_to(
        &mut self,
        (callee, callee_i, mut callee_v): (&Chunk<'p>, usize, Vec<Value>),
        vm_regs_i: &mut Vec<i64>,
        mut value: Res<ExecExit>,
        back: Back,
        caller: (&Chunk<'p>, usize, usize),
        regs_v: &mut [Value],
    ) -> Res<()> {
        if callee.traced {
            let regs_i = &mut vm_regs_i[callee_i..callee_i + callee.num_i];
            self.left_traced(callee, regs_i, &mut callee_v, value.is_err());
        }
        if let (Ret::Closure(call, _), Some(caps)) = (back.ret, &back.leaf) {
            let program = self.program;
            let c_regs_i = &vm_regs_i[callee_i..];
            let caps = program.closures[call.cid].captures.iter().zip(caps.iter());
            for (cap, cval) in caps.filter(|(_, c)| !matches!(c, crate::CaptureVal::Val(_))) {
                let loc = &callee.slot_map[cap.inner];
                let v = take_slot(loc, c_regs_i, &mut callee_v);
                match self.held_mut(cval) {
                    Ok(at) => *at = v.unwrap_or_else(|| read_slot(loc, c_regs_i, &callee_v)),
                    Err(e) => value = Err(e),
                }
            }
        }
        if let Ret::Closure(
            ClosureRet {
                held: Some(mark), ..
            },
            _,
        ) = back.ret
        {
            let held = self.lent.split_off(mark);
            if let Err(e) = self.give_back(callee, held, &vm_regs_i[callee_i..], &mut callee_v) {
                value = Err(e);
            }
        }
        self.restore_domains(back.dom_mark);
        self.unsuspend();
        self.frameless_base = back.frameless;
        if let Ret::Closure(_, in_closure) = back.ret {
            self.in_closure = in_closure;
        }
        if back.needs_stack {
            self.frame_locals.pop();
            self.stack.truncate(self.base);
            self.base = back.saved_base;
        }
        let (low, high) = vm_regs_i.split_at_mut(callee_i);
        let callee_regs = (callee, &*high, &mut callee_v[..]);
        let shown = back.shown;
        let res = value.and_then(|exit| {
            if let Ret::Closure(call, _) = back.ret {
                let (chunk, base_i, _) = caller;
                let regs_i = &mut low[base_i..base_i + chunk.num_i];
                if call.refs {
                    sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
                }
                self.hide_held_captures(shown)?;
            }
            self.returned(exit, back.ret, callee_regs, caller, low, regs_v)
        });
        self.return_regs_v(callee_v);
        vm_regs_i.truncate(callee_i);
        res
    }

    /// A traced `chunk`'s run ended, by a trap if `failed`: the end of the
    /// run and a trap show the locals as they were left.
    pub(super) fn left_traced(
        &mut self,
        chunk: &Chunk<'p>,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
        failed: bool,
    ) {
        sync_to_stack(
            &chunk.slot_map,
            regs_i,
            regs_v,
            &mut self.stack,
            self.base,
            true,
        );
        if failed {
            self.keep_trap_frame();
        }
    }

    /// [`Self::return_to`] once the callee returned `val`, by `ret`: the
    /// caller's frame synced, its `mut` args back from the callee's params,
    /// the result in place.
    fn returned(
        &mut self,
        exit: ExecExit,
        ret: Ret,
        (callee, callee_i, callee_v): (&Chunk<'p>, &[i64], &mut [Value]),
        (chunk, base_i, pc): (&Chunk<'p>, usize, usize),
        vm_regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<()> {
        self.call_frames.pop();
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        let program = self.program;
        let (args, ret_mode, dst, lends, refs) = match ret {
            Ret::Fn(c) => (
                c.args,
                c.ret_mode,
                c.dst,
                c.lend_back,
                chunk.has_by_ref_closure,
            ),
            Ret::Closure(c, _) => (c.args, c.ret_mode, c.dst, c.lends, false),
        };
        if lends {
            self.hide_lent()?;
            let (lent, base) = (&self.lent, self.base);
            sync_reachable_from_stack(chunk, lent, regs_i, regs_v, &mut self.stack, base);
        } else if refs {
            sync_refs_from_stack(chunk, regs_i, regs_v, &mut self.stack, self.base);
        }
        let (mut ii, mut iv) = (args.0 as usize, args.1 as usize);
        for (k, loc) in callee.slot_map.iter().take(callee.param_count).enumerate() {
            let back = match ret {
                Ret::Fn(call) => program.fns[call.fn_id as usize].params[k].mutable,
                Ret::Closure(call, _) => call.muts & 1 << k != 0,
            };
            match *loc {
                SlotLoc::I(r, _) => {
                    if back {
                        regs_i[ii] = callee_i[r as usize];
                    }
                    ii += 1;
                }
                SlotLoc::OptI { val, some, .. } => {
                    if back {
                        regs_i[ii] = callee_i[val as usize];
                        regs_i[ii + 1] = callee_i[some as usize];
                    }
                    ii += 2;
                }
                SlotLoc::V(r) => {
                    if back {
                        regs_v[iv] = std::mem::take(&mut callee_v[r as usize]);
                    }
                    iv += 1;
                }
                SlotLoc::SmallTuple { len, .. } => ii += len as usize,
            }
        }
        // A tuple the caller takes in registers stays in `ret_tuple`.
        match (ret_mode, exit) {
            (CallRet::Tuple(len), ExecExit::ReturnTuple) => {
                let (d, n) = (dst.0 as usize, len as usize);
                regs_i[d..d + n].copy_from_slice(&self.ret_tuple[..n]);
            }
            (_, exit) => {
                let val = self.exit_value(exit);
                put_ret(ret_mode, regs_i, regs_v, dst, val);
            }
        }
        self.keep_result(chunk, pc, regs_i, regs_v);
        Ok(())
    }

    /// After the `CallClosure` at `pc` of `chunk` returned `val`: the
    /// caller's lent places hidden again, its `mut` args back from
    /// `lent_back`, the result in place.
    pub(super) fn closure_returned(
        &mut self,
        (chunk, pc): (&Chunk<'p>, usize),
        call: ClosureRet,
        (val, lent_back): (Value, Vec<Value>),
        regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) -> Res<()> {
        if call.lends {
            self.hide_lent()?;
            let (lent, base) = (&self.lent, self.base);
            sync_reachable_from_stack(chunk, lent, regs_i, regs_v, &mut self.stack, base);
        }
        if !lent_back.is_empty() {
            let def = &self.program.closures[call.cid];
            let mut back = lent_back.into_iter();
            let (mut ii, mut iv) = (call.args.0 as usize, call.args.1 as usize);
            for (k, &pslot) in def.params.iter().enumerate() {
                let ty = &def.locals[pslot].ty;
                let v = if call.muts & 1 << k != 0 {
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
        put_ret(call.ret_mode, regs_i, regs_v, call.dst, val);
        self.keep_result(chunk, pc, regs_i, regs_v);
        Ok(())
    }
}

/// Moves the args at `ii` of `vm_regs_i` and `iv` of `regs_v` into the
/// params of `callee`, whose registers are at `c_base_i` and `c_regs_v`.
pub(super) fn pass_args(
    callee: &Chunk<'_>,
    vm_regs_i: &mut [i64],
    c_base_i: usize,
    regs_v: &mut [Value],
    c_regs_v: &mut [Value],
    (mut ii, mut iv): (usize, usize),
) {
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
                c_regs_v[r as usize] = std::mem::take(&mut regs_v[iv]);
                iv += 1;
            }
            SlotLoc::SmallTuple { base, len, .. } => {
                let n = len as usize;
                vm_regs_i.copy_within(ii..ii + n, c_base_i + base as usize);
                ii += n;
            }
        }
    }
}

/// The heap a frame for `chunk` holds: the frame and its registers.
fn frame_bytes(chunk: &Chunk<'_>) -> usize {
    std::mem::size_of::<VmFrame>()
        + chunk.num_i * std::mem::size_of::<i64>()
        + chunk.num_v * std::mem::size_of::<Value>()
}
