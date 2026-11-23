//! A traced chunk tells the tracer of a write it just did itself.

use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::Interp;
use cpl_ir::{Place, PlaceKind, Ty};

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
                if let (SlotLoc::I(r, _), Value::Int(_) | Value::Bool(_) | Value::Char(_)) =
                    (chunk.slot_map[slot], held)
                {
                    regs_i[r as usize] = ScalarKind::unbox_val(held);
                }
            }
            TracePoint::Cell(place, ref indices)
                if self.trace_raw_cell(&chunk.slot_map, regs_i, regs_v, place, indices) => {}
            ref point => {
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
                    self.trace_point(point);
                    take_back_values(&chunk.slot_map, regs_v, &mut self.stack, self.base);
                }
            }
        }
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
            TracePoint::Loop(slot, span) => {
                self.trace_local(slot, span);
                self.trace_assigned_local(slot, span);
            }
            TracePoint::LoopEnd(_) => {}
        }
    }

    /// Reports a write to `place` with the locals read straight from the
    /// registers, when every local the step shows is a raw `Int`; returns
    /// whether it did, or the caller must sync the stack and report it.
    pub(super) fn trace_raw_cell(
        &mut self,
        slot_map: &[SlotLoc],
        regs_i: &[i64],
        regs_v: &[Value],
        place: &'p Place,
        indices: &[u16],
    ) -> bool {
        if !self.observed() || self.assignments() {
            return false;
        }
        let mut raw = std::mem::take(&mut self.trace_raw);
        raw.clear();
        let base = self.base;
        let locals = self.locals();
        let all_raw = locals.iter().enumerate().all(|(slot, local)| {
            if local.name.is_empty() || local.name.starts_with('_') {
                return true;
            }
            let Some(held) = self.stack.get(base + slot) else {
                return true;
            };
            let value = match slot_map.get(slot) {
                Some(SlotLoc::I(r, _)) if !matches!(held, Value::Unit) => {
                    if !matches!(local.ty, Ty::Int) {
                        return false;
                    }
                    regs_i[*r as usize]
                }
                Some(SlotLoc::V(r)) if !matches!(regs_v[*r as usize], Value::Unit) => {
                    match (&local.ty, &regs_v[*r as usize]) {
                        (Ty::Int, Value::Int(v)) => *v,
                        _ => return false,
                    }
                }
                Some(SlotLoc::I(..)) | None | Some(SlotLoc::V(_)) => match held {
                    Value::Unit | Value::Moved => return true,
                    _ => return false,
                },
                Some(_) => return matches!(held, Value::Unit),
            };
            raw.push((local.name.as_str(), value));
            value.unsigned_abs() <= crate::trace::MAX_EXACT
        });
        if all_raw {
            let mut path = std::mem::take(&mut self.trace_path);
            path.clear();
            path.extend(indices.iter().map(|&r| regs_i[r as usize].max(0) as usize));
            let mut tracer = self.tracer.take().expect("traced");
            let view = crate::trace::View {
                program: self.program,
                globals: &self.globals,
                domains: &self.domains,
                locals: Vec::new(),
                raw: Some(&raw),
                depth: self.call_frames.len(),
                call_frames: &self.call_frames,
            };
            match &place.kind {
                PlaceKind::Global(id) if tracer.raw_write(*id, &view, place.span, &[]) => {}
                PlaceKind::Global(id) => tracer.declared(*id, &view, place.span),
                _ => {
                    let global = crate::root_global(place).expect("a global's cell");
                    if !tracer.raw_write(global, &view, place.span, &path) {
                        tracer.changed(global, &view, place.span, &path);
                    }
                }
            }
            self.tracer = Some(tracer);
            self.trace_path = path;
        }
        self.trace_raw = raw;
        all_raw
    }
}
