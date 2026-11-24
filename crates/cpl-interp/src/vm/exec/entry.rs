//! Entering the VM: running `main`, setup, a function or closure from the
//! stack, or a fed loop body, and the pooled register files they run on.

use super::*;

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
        let Some(chunk) = compiled.closures[cid].as_ref() else {
            return trap(
                cpl_diag::Span::default(),
                cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
            );
        };
        let def = &self.program.closures[cid];
        let by_ref = (def.captures.iter())
            .filter(|cap| cap.by_ref)
            .map(|cap| cap.inner);
        self.run_chunk_lending(compiled, chunk, by_ref)
    }

    pub(crate) fn run_fn_vm_from_stack(
        &mut self,
        compiled: &CompiledProgram<'p>,
        fn_id: FnId,
    ) -> Res<Value> {
        let Some(chunk) = compiled.fns[fn_id].as_ref() else {
            return trap(
                cpl_diag::Span::default(),
                cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
            );
        };
        self.run_chunk_lending(compiled, chunk, 0..chunk.param_count)
    }

    /// Runs `chunk` on the frame at `self.base`. A slot in `lent` that
    /// names a caller's place (a `mut` param lent on, a part of a value, or
    /// a variable a closure captured by reference) holds the value while
    /// the chunk runs.
    fn run_chunk_lending(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        lent: impl Iterator<Item = usize>,
    ) -> Res<Value> {
        let mut aliases: Vec<(usize, Rc<Part>)> = Vec::new();
        for k in lent {
            let part = match &self.stack[self.base + k] {
                &Value::Alias(mut global, mut at) => {
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
                Value::Part(part) => Rc::clone(part),
                _ => continue,
            };
            let val = std::mem::replace(self.lender(&part)?, Value::Moved);
            self.stack[self.base + k] = val;
            aliases.push((k, part));
        }
        let lent = self.lent.len();
        let base = self.base;
        let held = aliases.iter().map(|(k, part)| (base + k, Rc::clone(part)));
        self.lent.extend(held);
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
        self.lent.truncate(lent);
        for (k, part) in aliases {
            // A traced chunk leaves its locals on the stack.
            let val = match chunk.slot_map[k] {
                _ if chunk.traced => {
                    std::mem::replace(&mut self.stack[self.base + k], Value::Moved)
                }
                SlotLoc::I(r, kind) => kind.box_val(vm_regs_i[base_i + r as usize]),
                SlotLoc::OptI { val, some, kind } => kind.box_opt(
                    vm_regs_i[base_i + val as usize],
                    vm_regs_i[base_i + some as usize],
                ),
                SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
                SlotLoc::SmallTuple { .. } => {
                    return trap(
                        cpl_diag::Span::default(),
                        cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]),
                    )
                }
            };
            *self.lender(&part)? = val;
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        self.vm_regs_i = vm_regs_i;
        res
    }
}
