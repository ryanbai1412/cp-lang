//! The source line a step is shown at.

use super::{Tracer, View};
use cpl_diag::Span;
use cpl_ir::Program;
use std::collections::HashMap;

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

/// A fn's source name by its lowered one, where they differ.
pub(super) fn shown_names(program: &Program) -> HashMap<String, String> {
    (program.fns.iter().zip(&program.fn_display))
        .filter(|(f, shown)| f.name != **shown)
        .map(|(f, shown)| (f.name.clone(), shown.clone()))
        .collect()
}

impl Tracer {
    /// The step's call stack, under the names the source gives its fns.
    pub(super) fn frame_of(&mut self, view: &View) -> u32 {
        // Taken out so the names it lends outlive the borrow of `self`;
        // its strings stay where they are, which `Log::frame` relies on.
        let shown = std::mem::take(&mut self.shown);
        let stack: Vec<(&str, u32)> = view
            .call_frames
            .iter()
            .map(|(name, at)| {
                let name = shown.get(*name).map_or(*name, String::as_str);
                (name, self.line(*at).max(1))
            })
            .collect();
        let frame = self.rec.log.frame(&stack);
        drop(stack);
        self.shown = shown;
        frame
    }
}
