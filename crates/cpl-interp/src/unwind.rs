//! What a trap keeps of the calls it unwinds: the call stack it names, and
//! the innermost call's locals for the trace's last step.

use super::{Frame, Interp, Span};
use crate::observe::Observer;
use crate::Value;
use cpl_ir::Ty;

/// A trap's call stack, innermost first; past 32 calls, the 16 innermost
/// and the 8 outermost around a count of the rest.
/// The innermost frame's depth and named locals, kept at a trap.
pub(crate) type TrapFrame<'p> = (usize, Vec<(&'p str, &'p Ty, Value)>);

pub(crate) fn call_stack(program: &cpl_ir::Program, frames: &[(&str, Span)]) -> Vec<Frame> {
    let frame = |&(name, call_span): &(&str, Span)| Frame {
        name: program.shown_fn(name).to_string(),
        call_span,
    };
    if frames.len() <= 32 {
        return frames.iter().rev().map(frame).collect();
    }
    let mut stack: Vec<Frame> = frames[frames.len() - 16..]
        .iter()
        .rev()
        .map(frame)
        .collect();
    stack.push(Frame {
        name: format!("... {} frames omitted ...", frames.len() - 24),
        call_span: Span::default(),
    });
    stack.extend(frames[..8].iter().rev().map(frame));
    stack
}

impl<O: Observer> Interp<'_, '_, O> {
    /// Keeps the locals of the call a trap is leaving, before unwinding drops
    /// them: only the innermost, and only when the run is traced.
    pub(crate) fn keep_trap_frame(&mut self) {
        if !self.observed() || self.trap_frame.is_some() {
            return;
        }
        let locals = self
            .named_locals()
            .map(|(name, ty, value)| (name, ty, value.clone()));
        self.trap_frame = Some((self.call_frames.len(), locals.collect()));
    }
}
