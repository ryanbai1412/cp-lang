//! The steps a VM write takes when its locals are read straight from the
//! registers: the same log entries as [`Tracer::step`] and
//! [`Tracer::declared`] / [`Tracer::changed`], with no `Value` view.

use super::{int_item, string, Kind, Tracer, View, MAX_EXACT};
use crate::value::Value;
use cpl_diag::Span;
use cpl_ir::Ty;

impl Tracer {
    pub(super) fn raw_locals(&mut self, raw: &[(&str, i64)]) {
        let (mut names, mut ints) = (
            std::mem::take(&mut self.local_names),
            std::mem::take(&mut self.local_ints),
        );
        ints.clear();
        ints.extend(raw.iter().map(|&(_, v)| v));
        let at = raw
            .iter()
            .map(|(name, _)| (name.as_ptr() as usize, name.len()));
        if !at.clone().eq(self.local_name_at.iter().copied()) {
            self.local_name_at.clear();
            self.local_name_at.extend(at);
            names.clear();
            for (name, _) in raw {
                names.push('\n');
                string(&mut names, name);
            }
        }
        self.rec.log.locals_ints(&names, &ints);
        (self.local_names, self.local_ints) = (names, ints);
    }

    /// The step a VM write to `Int` global `id`, or to `Int` cell `path` of
    /// it, makes when nothing filters steps and the locals are raw: what
    /// [`Tracer::declared`] or [`Tracer::changed`] would record, with less
    /// on the way. False when they must take it.
    pub fn raw_write(&mut self, id: usize, view: &View, span: Span, path: &[usize]) -> bool {
        let Some(raw) = view.raw else { return false };
        if !self.unfiltered || self.finishing || self.case == Some(0) || !self.visible[id] {
            return false;
        }
        if self.all_touched || !self.touched.is_empty() || view.program.globals[id].span.end == 0 {
            return false;
        }
        let Some(var) = self.of_global[id] else {
            return false;
        };
        let len = self.rec.log.len();
        if len >= self.max_steps || self.rec.log.bytes() >= self.max_bytes {
            return false;
        }
        let (value, ty) = (&view.globals[id], &view.program.globals[id].ty);
        match (self.kinds[var], path) {
            (Kind::Scalar, []) => match (value, ty) {
                (Value::Int(v), Ty::Int) if v.unsigned_abs() <= MAX_EXACT => {
                    self.rec.log.snapshot_if_due();
                    self.rec.log.set_int(var, *v);
                }
                _ => return false,
            },
            (Kind::Array, &[i]) => {
                let Some(v) = int_item(value, ty, i) else {
                    return false;
                };
                self.rec.log.snapshot_if_due();
                if !self.rec.log.set_int_item(var, i, v) {
                    return false;
                }
            }
            _ => return false,
        }
        self.raw_locals(raw);
        let focus = (!path.is_empty()).then_some((id, path));
        self.step_rest(view, span, None, None, focus);
        true
    }
}
