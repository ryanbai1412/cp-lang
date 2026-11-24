//! The fn contexts around the one being lowered, innermost last.

use super::{FnCx, Lowerer};

/// Grows only by `enter_closure_cx`, and shrinks only by `leave_closure_cx`
/// taking back the frame it gave, so a frame's cx is always there to restore.
#[derive(Clone, Default)]
pub(in crate::lower) struct OuterCxs(Vec<FnCx>);

impl std::ops::Deref for OuterCxs {
    type Target = [FnCx];
    fn deref(&self) -> &[FnCx] {
        &self.0
    }
}

impl std::ops::DerefMut for OuterCxs {
    fn deref_mut(&mut self) -> &mut [FnCx] {
        &mut self.0
    }
}

/// The cx a closure's body was entered from, at its level of `outer_cxs`.
#[must_use]
pub(in crate::lower) struct CxFrame {
    level: usize,
}

impl CxFrame {
    pub(in crate::lower) fn level(&self) -> usize {
        self.level
    }
}

impl Lowerer<'_> {
    /// Makes `inner` the cx, keeping the current one as the innermost outer cx.
    pub(in crate::lower) fn enter_closure_cx(&mut self, inner: FnCx) -> CxFrame {
        let level = self.outer_cxs.0.len();
        let outer = std::mem::replace(&mut self.cx, inner);
        self.outer_cxs.0.push(outer);
        CxFrame { level }
    }

    /// Restores the cx `frame` was entered from, giving back the closure's.
    pub(in crate::lower) fn leave_closure_cx(&mut self, frame: CxFrame) -> FnCx {
        self.outer_cxs.0.truncate(frame.level + 1);
        let outer = self.outer_cxs.0.swap_remove(frame.level);
        std::mem::replace(&mut self.cx, outer)
    }
}
