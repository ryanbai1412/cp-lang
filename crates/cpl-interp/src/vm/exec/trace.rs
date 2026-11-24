//! A traced chunk tells the tracer of a write it just did itself.

use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::Interp;
use cpl_ir::{Place, PlaceKind};
use std::rc::Rc;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    /// The `Trace` op of `chunk`: kept out of the op dispatch, which a
    /// traced loop runs once a round.
    #[inline(never)]
    pub(super) fn exec_trace(
        &mut self,
        chunk: &Chunk<'p>,
        idx: u32,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
    ) {
        match chunk.trace_points[idx as usize] {
            TracePoint::LoopEnd(slot) => {
                let held = &self.stack[self.base + slot];
                // A loop that never ran leaves the variable as it was.
                if let (SlotLoc::I(r, _), false) = (
                    chunk.slot_map[slot],
                    matches!(held, Value::Unit | Value::Moved),
                ) {
                    regs_i[r as usize] = ScalarKind::unbox_val(held);
                }
            }
            TracePoint::Declared(slot) => {
                if matches!(self.stack[self.base + slot], Value::Unit) {
                    self.stack[self.base + slot] = Value::Moved;
                }
            }
            TracePoint::Sorted => {
                if let Some(tracer) = self.tracer.as_mut() {
                    tracer.globals_may_change();
                }
            }
            TracePoint::Cell(place, ref indices)
                if self.trace_int_write(chunk, regs_i, regs_v, place, indices) => {}
            TracePoint::Cell(place, _) | TracePoint::Assign(place) if self.quiet_write(place) => {}
            TracePoint::Assign(place)
                if matches!(place.kind, PlaceKind::Global(_))
                    && self.trace_global_unseen(place, &[]) => {}
            TracePoint::Cell(place, ref indices)
                if self.trace_cell_unseen(place, regs_i, indices) => {}
            TracePoint::Cell(place, ref indices)
                if self.trace_raw_cell(chunk, regs_i, regs_v, place, indices) => {}
            ref point => {
                // A local's step in a call that the tracer skips needs no
                // view, so no copy of the registers to the stack.
                let local = match *point {
                    TracePoint::Let(slot, _) | TracePoint::Loop(slot, _) => Some(slot),
                    TracePoint::Assign(place)
                    | TracePoint::Cell(place, _)
                    | TracePoint::Mutated(place, _) => place.root_local(),
                    _ => None,
                };
                if local.is_some_and(|slot| self.trace_local_unseen(slot)) {
                    if let TracePoint::Let(slot, _) | TracePoint::Loop(slot, _) = *point {
                        // Declared: later syncs and checkpoints read its register.
                        self.stack[self.base + slot] = Value::Moved;
                    }
                    return;
                }
                let mut quiet = false;
                if let TracePoint::Let(slot, _) | TracePoint::Loop(slot, _) = *point {
                    // A top-level local takes no step: only its own
                    // slot needs the register's value.
                    match chunk.slot_map[slot] {
                        SlotLoc::I(r, kind)
                            if self.call_frames.is_empty() && !self.assignments() =>
                        {
                            set_scalar(&mut self.stack[self.base + slot], kind, regs_i[r as usize]);
                            quiet = true;
                        }
                        _ => self.stack[self.base + slot] = Value::Moved,
                    }
                }
                if !quiet {
                    sync_to_stack(
                        &chunk.slot_map,
                        regs_i,
                        regs_v,
                        &mut self.stack,
                        self.base,
                        chunk.traced,
                    );
                    self.show_lent();
                    self.trace_point(point);
                    self.hide_lent();
                    take_back_values(&chunk.slot_map, regs_v, &mut self.stack, self.base);
                }
            }
        }
    }

    /// Before the tracer or the tree-walker sees a traced call's frame:
    /// each `mut` param holds an alias to the caller's place, which holds
    /// the value, as when the tree-walker runs the call.
    pub(super) fn show_lent(&mut self) {
        for k in (0..self.lent.len()).rev() {
            let (held, part) = (self.lent[k].0, Rc::clone(&self.lent[k].1));
            if held < self.base {
                break;
            }
            let lent = match part.steps.is_empty() {
                true => Value::Alias(part.global, part.at),
                false => Value::Part(Rc::clone(&part)),
            };
            let value = std::mem::replace(&mut self.stack[held], lent);
            *self.lender(&part).expect("a lent place is still there") = value;
        }
    }

    /// After: the value is back in the param, the caller's place lent.
    pub(super) fn hide_lent(&mut self) {
        for k in (0..self.lent.len()).rev() {
            let (held, part) = (self.lent[k].0, Rc::clone(&self.lent[k].1));
            if held < self.base {
                break;
            }
            let place = self.lender(&part).expect("a lent place is still there");
            self.stack[held] = std::mem::replace(place, Value::Moved);
        }
    }

    pub(super) fn lends(&self) -> bool {
        self.lent
            .last()
            .is_some_and(|&(held, ..)| held >= self.base)
            && self.frameless_base != Some(self.base)
    }

    fn trace_point(&mut self, point: &TracePoint<'p>) {
        match *point {
            TracePoint::Let(slot, init) => {
                self.trace_local(slot, init.span);
                self.trace_assigned_local(slot, init.span);
            }
            TracePoint::Assign(place) | TracePoint::Cell(place, _) => {
                let mut indices = std::mem::take(&mut self.trace_indices);
                indices.clear();
                if self.indices(place, &mut indices).is_ok() {
                    self.trace_change(place, &indices);
                }
                indices.clear();
                self.trace_indices = indices;
                self.trace_assigned(place);
            }
            TracePoint::Mutated(place, span) => self.trace_mutated(place, span),
            TracePoint::Loop(slot, span) => {
                self.trace_local(slot, span);
                self.trace_assigned_local(slot, span);
            }
            TracePoint::LoopEnd(_)
            | TracePoint::Tick
            | TracePoint::Sorted
            | TracePoint::Declared(_) => {}
        }
    }

    /// [`Interp::trace_global_unseen`] for a cell of a global at the indices
    /// in registers `indices`.
    fn trace_cell_unseen(&mut self, place: &Place, regs_i: &[i64], indices: &[u16]) -> bool {
        let mut path = std::mem::take(&mut self.trace_path);
        path.clear();
        for &r in indices {
            path.push(regs_i[r as usize].max(0) as usize);
        }
        let done = self.trace_global_unseen(place, &path);
        self.trace_path = path;
        done
    }

    /// The named locals of the running frame, read straight from the
    /// registers into `trace_raw`, when each is an exact `Int`.
    fn raw_frame(&mut self, chunk: &Chunk<'p>, regs_i: &[i64], regs_v: &[Value]) -> bool {
        let mut raw = std::mem::take(&mut self.trace_raw);
        raw.clear();
        let base = self.base;
        let all_raw = chunk.named_locals.iter().all(|&(slot, name, int)| {
            let Some(held) = self.stack.get(base + slot) else {
                return true;
            };
            let value = match chunk.slot_map.get(slot) {
                Some(SlotLoc::I(r, _)) if !matches!(held, Value::Unit) => {
                    if !int {
                        return false;
                    }
                    regs_i[*r as usize]
                }
                Some(SlotLoc::V(r)) if !matches!(regs_v[*r as usize], Value::Unit) => {
                    match (int, &regs_v[*r as usize]) {
                        (true, Value::Int(v)) => *v,
                        _ => return false,
                    }
                }
                Some(SlotLoc::I(..)) | None | Some(SlotLoc::V(_)) => match held {
                    Value::Unit | Value::Moved => return true,
                    _ => return false,
                },
                Some(_) => return matches!(held, Value::Unit),
            };
            raw.push((name, value));
            value.unsigned_abs() <= crate::trace::MAX_EXACT
        });
        self.trace_raw = raw;
        all_raw
    }

    /// The indices of a cell write, evaluated again into `trace_path`.
    fn cell_path(&mut self, regs_i: &[i64], indices: &[u16]) -> Vec<usize> {
        let mut path = std::mem::take(&mut self.trace_path);
        path.clear();
        path.extend(indices.iter().map(|&r| regs_i[r as usize].max(0) as usize));
        path
    }

    /// A kept step for a write to an `Int` global or cell, recorded from
    /// the registers with no view: false when the tracer needs one.
    pub(super) fn trace_int_write(
        &mut self,
        chunk: &Chunk<'p>,
        regs_i: &[i64],
        regs_v: &[Value],
        place: &'p Place,
        indices: &[u16],
    ) -> bool {
        if !self.observed() || self.lends() {
            return false;
        }
        let Some(id) = place.root_global() else {
            return false;
        };
        if !self.raw_frame(chunk, regs_i, regs_v) {
            return false;
        }
        let path = self.cell_path(regs_i, indices);
        let global = &self.program.globals[id];
        let write = crate::trace::IntWrite {
            id,
            path: &path,
            value: &self.globals[id],
            ty: &global.ty,
            named: global.span.end != 0,
            span: place.span,
            raw: &self.trace_raw,
            call_frames: &self.call_frames,
        };
        let done = (self.tracer.as_mut()).is_some_and(|tracer| tracer.int_write(&write));
        self.trace_path = path;
        done
    }

    /// A write to a global or a cell of one whose step the tracer takes
    /// from a view of raw locals, when each is an exact `Int`.
    pub(super) fn trace_raw_cell(
        &mut self,
        chunk: &Chunk<'p>,
        regs_i: &[i64],
        regs_v: &[Value],
        place: &'p Place,
        indices: &[u16],
    ) -> bool {
        if !self.observed() || self.assignments() || self.lends() {
            return false;
        }
        if !self.raw_frame(chunk, regs_i, regs_v) {
            return false;
        }
        let path = self.cell_path(regs_i, indices);
        let depth = self.call_frames.len();
        if (self.tracer.as_ref()).is_some_and(|t| t.keeps_step(depth)) {
            self.fill_shown();
        }
        let raw = std::mem::take(&mut self.trace_raw);
        let mut tracer = self.tracer.take().expect("traced");
        let view = crate::trace::View {
            program: self.program,
            globals: &self.globals,
            domains: &self.domains,
            locals: Vec::new(),
            raw: Some(&raw),
            depth,
            call_frames: &self.call_frames,
            shown: &self.trace_shown,
            read: self.input.cursor(),
        };
        match &place.kind {
            PlaceKind::Global(id) => tracer.declared(*id, &view, place.span),
            _ => {
                let global = place.root_global().expect("a global's cell");
                tracer.changed(global, &view, place.span, &path);
            }
        }
        self.tracer = Some(tracer);
        self.trace_path = path;
        self.trace_raw = raw;
        true
    }
}

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    /// Calls the fn value `fv` with its `mut` arguments in `lent` moved out
    /// of their places for the call and put back after, as a traced run
    /// lends them; their places' index values start at `regs_v[first]`.
    pub(super) fn call_moving_lent(
        &mut self,
        fv: Value,
        mut args: Vec<Value>,
        lent: &'p [cpl_ir::Arg],
        regs_v: &mut [Value],
        mut first: usize,
        span: cpl_diag::Span,
    ) -> crate::Res<Value> {
        let mut parts = Vec::new();
        for (i, arg) in lent.iter().enumerate() {
            let cpl_ir::Arg::Mut(place) = arg else {
                continue;
            };
            let mut exprs = Vec::new();
            place_indices(place, &mut exprs);
            let indices: Vec<Value> = (first..first + exprs.len())
                .map(|r| std::mem::replace(&mut regs_v[r], Value::Unit))
                .collect();
            first += exprs.len();
            let part = self.part_of(place, &mut indices.into_iter());
            let value = std::mem::replace(self.part_mut(&part, place.span)?, Value::Moved);
            if matches!(value, Value::Moved) {
                return crate::trap(place.span, cpl_diag::msgs::interp::LENT.at(&[]));
            }
            args[i] = value;
            parts.push((i, place, part));
        }
        let at: Vec<usize> = parts.iter().map(|&(i, ..)| i).collect();
        let (value, back) = self.call_lending(fv, args, &at, span)?;
        for ((_, place, part), value) in parts.into_iter().zip(back) {
            *self.part_mut(&part, place.span)? = value;
        }
        Ok(value)
    }
}
