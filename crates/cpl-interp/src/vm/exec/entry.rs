//! Entering the VM: running `main`, setup, a function or closure from the
//! stack, or a fed loop body, and the pooled register files they run on.

use super::*;
use crate::checkpoint::{chunk_of, Body, FrameAt};

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    pub(super) fn take_regs_v(&mut self, n: usize) -> Vec<Value> {
        if n == 0 {
            return Vec::new();
        }
        let mut v = self.vm_pool_v.pop().unwrap_or_default();
        v.clear();
        // Built in place: `resize` would call `Value::clone` per register.
        v.extend(std::iter::repeat_with(|| Value::Unit).take(n));
        v
    }

    pub(super) fn return_regs_v(&mut self, mut v: Vec<Value>) {
        debug_assert!(
            !self.ckpt.waits_on(&v),
            "registers freed while a caller waits on them"
        );
        if v.capacity() > 0 && self.vm_pool_v.len() < 64 {
            v.clear();
            self.vm_pool_v.push(v);
        }
    }

    /// The body of fed loop `id` in the frame at `caller`, on the traced
    /// chunk that left the loop to the tree-walker, if one holds it.
    pub(crate) fn fed_body_vm(&mut self, id: LoopId, caller: usize) -> Option<Res<Flow>> {
        let &(_, host) = self.fed_hosts.iter().rev().find(|h| h.0 == caller)?;
        let compiled = self.vm.clone()?;
        let chunk = compiled
            .chunks()
            .find(|c| std::ptr::eq(*c as *const Chunk<'p> as *const (), host))?;
        let &(_, pc) = chunk.fed_bodies.iter().find(|b| b.0 == id)?;
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        let mut regs_v = self.take_regs_v(chunk.num_v);
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        sync_from_stack(
            &chunk.slot_map,
            regs_i,
            &mut regs_v,
            &mut self.stack,
            self.base,
            true,
        );
        let exit = self.exec_chunk_core(
            &compiled,
            chunk,
            base_i,
            &mut vm_regs_i,
            &mut regs_v,
            pc as usize,
            None,
        );
        let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        sync_to_stack(
            &chunk.slot_map,
            regs_i,
            &mut regs_v,
            &mut self.stack,
            self.base,
            true,
        );
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        self.vm_regs_i = vm_regs_i;
        Some(exit.map(|exit| match exit {
            ExecExit::FeedNext => Flow::Next,
            ExecExit::FeedBreak => Flow::Break(Some(id)),
            ExecExit::Return(value) => Flow::Return(value),
        }))
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

    /// The code above `@!multitest` on the VM: true when a top-level
    /// `return` there ended the program.
    pub(crate) fn run_setup_vm(
        &mut self,
        compiled: &CompiledProgram<'p>,
        setup: &Chunk<'p>,
    ) -> Res<bool> {
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        let res = self.exec_chunk(compiled, setup, &[], &mut vm_regs_i);
        self.vm_regs_i = vm_regs_i;
        Ok(!matches!(res?, Value::Int(1)))
    }

    /// Closure `cid`'s body on the VM, in the frame at `self.base` that
    /// holds its parameters and captures, all values.
    pub(crate) fn run_closure_vm_from_stack(
        &mut self,
        compiled: &CompiledProgram<'p>,
        cid: cpl_ir::ClosureId,
    ) -> Res<Value> {
        self.run_body_vm(compiled, Body::Closure(cid), None)
    }

    pub(crate) fn run_fn_vm_from_stack(
        &mut self,
        compiled: &CompiledProgram<'p>,
        fn_id: FnId,
    ) -> Res<Value> {
        self.run_body_vm(compiled, Body::Fn(fn_id), None)
    }

    /// `body` on the VM in the frame at `self.base`, lending its `mut`
    /// params or by-reference captures; from frame `k` of `frames` when
    /// `resume` names one.
    pub(super) fn run_body_vm(
        &mut self,
        compiled: &CompiledProgram<'p>,
        body: Body,
        resume: Option<(&[FrameAt], usize)>,
    ) -> Res<Value> {
        let Some(chunk) = chunk_of(compiled, body) else {
            return trap(
                cpl_diag::Span::default(),
                cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
            );
        };
        let mut params = 0..0;
        let mut by_ref;
        let lent: &mut dyn Iterator<Item = usize> = match body {
            Body::Main | Body::Setup => &mut params,
            Body::Fn(_) | Body::Fed(_) => {
                params = 0..chunk.param_count;
                &mut params
            }
            Body::Closure(cid) => {
                by_ref = (self.program.closures[cid].captures.iter())
                    .filter(|cap| cap.by_ref)
                    .map(|cap| cap.inner);
                &mut by_ref
            }
        };
        self.run_chunk_lending(compiled, chunk, lent, resume)
    }

    /// Runs `chunk` on the frame at `self.base`. A slot in `lent` that
    /// names a caller's place (a `mut` param lent on, a part of a value, or
    /// a variable a closure captured by reference) holds the value while
    /// the chunk runs. Run from a checkpoint, it goes on from where frame
    /// `k` of `frames` was, and leaves its locals on the stack.
    /// The place an alias to slot `at` (of the globals or the stack)
    /// finally names, following aliases of aliases.
    pub(super) fn alias_part(&self, mut global: bool, mut at: usize) -> Rc<Part> {
        while let Value::Alias(g, a) = match global {
            true => &self.globals[at],
            false => &self.stack[at],
        } {
            (global, at) = (*g, *a);
        }
        Rc::new(Part {
            global,
            at,
            steps: Vec::new(),
        })
    }

    /// Puts each value `held` lent to `chunk`'s frame back where it was
    /// lent from, out of the frame's registers (a traced chunk leaves its
    /// locals on the stack).
    pub(super) fn give_back(
        &mut self,
        chunk: &Chunk<'p>,
        held: Vec<(usize, Rc<Part>)>,
        regs_i: &[i64],
        regs_v: &mut [Value],
    ) -> Res<()> {
        for (at, part) in held {
            let val = match chunk.traced {
                true => Some(std::mem::replace(&mut self.stack[at], Value::Moved)),
                false => take_slot(&chunk.slot_map[at - self.base], regs_i, regs_v),
            };
            let Some(val) = val else {
                return trap(
                    cpl_diag::Span::default(),
                    cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                );
            };
            *self.lender(&part)? = val;
        }
        Ok(())
    }

    fn run_chunk_lending(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        lent: &mut dyn Iterator<Item = usize>,
        resume: Option<(&[FrameAt], usize)>,
    ) -> Res<Value> {
        let lent_mark = self.lent.len();
        for k in lent {
            let part = match &self.stack[self.base + k] {
                &Value::Alias(global, at) => self.alias_part(global, at),
                Value::Part(part) => Rc::clone(part),
                _ => continue,
            };
            let val = std::mem::replace(self.lender(&part)?, Value::Moved);
            self.stack[self.base + k] = val;
            self.lent.push((self.base + k, part));
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
        let start = match resume {
            Some((frames, k)) => {
                let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                self.resume_at(chunk, frames, k, regs_i, &mut regs_v)
            }
            None => Ok(0),
        };
        let res = match start {
            Ok(pc) => {
                self.exec_chunk_inner(compiled, chunk, base_i, &mut vm_regs_i, &mut regs_v, pc)
            }
            Err(e) => Err(e),
        };
        if resume.is_some() && !chunk.traced {
            let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
            sync_to_stack(
                &chunk.slot_map,
                regs_i,
                &mut regs_v,
                &mut self.stack,
                self.base,
                true,
            );
        }
        let held = self.lent.split_off(lent_mark);
        self.give_back(chunk, held, &vm_regs_i[base_i..], &mut regs_v)?;
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        self.vm_regs_i = vm_regs_i;
        res
    }
}
