//! Steps the tracer counts without a view of the run, as before a
//! window: the run need not copy its state out to show it.

use super::Tracer;
use cpl_diag::Span;

impl Tracer {
    /// [`Tracer::local_changed`] at call depth `depth`, without a view, when
    /// no step is kept from it: true when it took note, false when it needs
    /// the view to record the step.
    pub(crate) fn local_changed_unseen(&mut self, depth: usize, name: &str) -> bool {
        if self.skip == 0 && self.recording(depth) {
            return false;
        }
        if depth > 0 {
            self.touch_all();
        }
        if depth == 0 || name.is_empty() || name.starts_with('_') || !self.watched(name) {
            return true;
        }
        if self.recording(depth) {
            self.skipped();
        }
        true
    }

    /// [`Tracer::changed`] for global `id`, named `name`, at call depth
    /// `depth`, without a view, when no step is kept from it: true when it
    /// took note, false when it needs the view to record the step.
    pub(crate) fn changed_unseen(
        &mut self,
        id: usize,
        name: &str,
        depth: usize,
        path: &[usize],
    ) -> bool {
        if self.skip == 0 && self.recording(depth) {
            return false;
        }
        self.touch(id, (path.len() == 1).then(|| path[0]));
        if self.watched(name) && self.recording(depth) {
            self.skipped();
        }
        true
    }

    /// [`Tracer::declared`] the same way; `unnamed` for a global the
    /// source never names.
    pub(crate) fn declared_unseen(
        &mut self,
        id: usize,
        name: &str,
        unnamed: bool,
        depth: usize,
        span: Span,
    ) -> bool {
        if self.skip == 0 && self.recording(depth) || !self.library.is_empty() {
            return false;
        }
        self.touch(id, None);
        let fresh = !self.visible[id];
        self.visible[id] = true;
        if unnamed || !self.watched(name) || !self.recording(depth) {
            return true;
        }
        // Before the window nothing is kept, so no step to join.
        let at = (self.line(span).max(1), self.stage_id());
        self.skip_declaration(fresh, at);
        self.skipped();
        self.skipped_at = (self.rec.log.len() == 0).then_some(at);
        true
    }
}
