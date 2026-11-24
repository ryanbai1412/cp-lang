//! The steps a VM write takes when its locals are read straight from the
//! registers: the same log entries as [`Tracer::step`] and
//! [`Tracer::declared`] / [`Tracer::changed`], with no `Value` view.

use super::log::RawStep;
use super::{int_item, string, Kind, Tracer, MAX_EXACT};
use crate::value::Value;
use cpl_diag::Span;
use cpl_ir::Ty;

impl Tracer {
    /// [`crate::observe::Observer::quiet_write`]: outside the traced fns a
    /// write only marks its global for the next kept step.
    pub(crate) fn quiet_write(&mut self, place: &cpl_ir::Place, depth: usize) -> bool {
        if self.assignments || self.recording(depth) {
            return false;
        }
        let Some(id) = place.root_global().filter(|&id| self.visible[id]) else {
            return false;
        };
        self.touch(id, None);
        true
    }

    pub(super) fn raw_locals(&mut self, raw: &[(&str, i64)]) {
        let (names, ints) = (&mut self.local_names, &mut self.local_ints);
        ints.clear();
        ints.extend(raw.iter().map(|&(_, v)| v));
        let at = raw
            .iter()
            .map(|(name, _)| (name.as_ptr() as usize, name.len()));
        let renamed = !at.clone().eq(self.local_name_at.iter().copied());
        if renamed {
            self.local_name_at.clear();
            self.local_name_at.extend(at);
            names.clear();
            for (name, _) in raw {
                names.push('\n');
                string(names, name);
            }
        }
        self.log.locals_ints(names, renamed, ints);
    }

    /// The step a VM write to `Int` global `id`, or to `Int` cell `path` of
    /// it, makes when the step is kept, nothing filters steps and the
    /// locals are raw: what [`Tracer::declared`] or [`Tracer::changed`]
    /// would record, with no view on the way. False when they must take it.
    pub(crate) fn int_write(&mut self, w: &IntWrite) -> bool {
        let id = w.id;
        if self.assignments
            || self.skip != 0
            || !self.unfiltered
            || self.finishing
            || self.case == Some(0)
            || !self.visible[id]
            || !w.named
            || self.all_touched
            || !self.touched.is_empty()
            || !self.recording(w.call_frames.len())
        {
            return false;
        }
        let Some(var) = self.of_global[id] else {
            return false;
        };
        let len = self.log.len();
        if len >= self.max_steps || self.log.bytes() >= self.max_bytes && self.full() {
            return false;
        }
        match (self.kinds[var], w.path) {
            (Kind::Scalar, []) => match (w.value, w.ty) {
                (Value::Int(v), Ty::Int) if v.unsigned_abs() <= MAX_EXACT => {
                    if self.pend_int(w, var, *v) {
                        return true;
                    }
                    self.log.snapshot_if_due();
                    self.log.set_int(var, *v);
                }
                _ => return false,
            },
            (Kind::Array, &[i]) => {
                let Some(v) = int_item(w.value, w.ty, i) else {
                    return false;
                };
                self.log.snapshot_if_due();
                if !self.log.set_int_item(var, i, v) {
                    return false;
                }
            }
            _ => return false,
        }
        self.raw_locals(w.raw);
        let focus = (!w.path.is_empty()).then_some((id, w.path));
        self.step_rest(w.call_frames, w.span, None, None, focus);
        true
    }

    /// The log holds [`Tracer::max_bytes`] once what it keeps raw is logged.
    fn full(&mut self) -> bool {
        self.log.render();
        self.log.bytes() >= self.max_bytes
    }

    /// The step of an `Int` write to scalar `var`, kept raw for the log to
    /// render later: when the locals have the names of the last step.
    fn pend_int(&mut self, w: &IntWrite, var: usize, value: i64) -> bool {
        let Ok(var) = u16::try_from(var) else {
            return false;
        };
        let at = w.raw.iter().map(|(name, _)| (name.as_ptr() as usize, name.len()));
        if !at.eq(self.local_name_at.iter().copied()) {
            return false;
        }
        self.local_ints.clear();
        self.local_ints.extend(w.raw.iter().map(|&(_, v)| v));
        let line = self.step_line(w.call_frames, w.span);
        let frame = self.record_frames.then(|| self.frame_of(w.call_frames));
        let stage = self.stage_id();
        let step = RawStep { line, var, value };
        if !self.log.pend(frame, stage, step, &self.local_ints) {
            return false;
        }
        self.keep_last_steps();
        true
    }
}

/// A VM write [`Tracer::int_write`] may record.
pub(crate) struct IntWrite<'a> {
    pub id: usize,
    pub path: &'a [usize],
    pub value: &'a Value,
    pub ty: &'a Ty,
    /// The source names the global.
    pub named: bool,
    pub span: Span,
    /// The named locals, each an `Int`.
    pub raw: &'a [(&'a str, i64)],
    pub call_frames: &'a [(&'a str, Span)],
}
