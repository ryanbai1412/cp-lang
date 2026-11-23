//! The source line a step is shown at.

use super::{Tracer, View};
use cpl_diag::Span;

impl Tracer {
    pub fn line(&self, span: Span) -> u32 {
        let (start, line) = self.last_line.get();
        if start == span.start {
            return line;
        }
        let line = self.lines.line_of(span.start) + 1;
        self.last_line.set((span.start, line));
        line
    }

    /// The line of `span` as the file shows it. An imported module's fn is
    /// lowered once, with the span of the first call that asked for it, so
    /// a step inside one is shown at the call in the file that is running
    /// it: the call site of the outermost of the imported fns on top of
    /// the stack.
    pub(super) fn step_line(&self, view: &View, span: Span) -> u32 {
        let mut at = span;
        if !self.library.is_empty() {
            for (name, call) in view.call_frames.iter().rev() {
                if !self.library.contains(*name) {
                    break;
                }
                at = *call;
            }
        }
        self.line(at).max(1)
    }
}
