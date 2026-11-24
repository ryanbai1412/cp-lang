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

/// Where a trap is shown: one inside library code is shown at the user's
/// call that reached it. A library fn is lowered once for all its callers,
/// so its own spans name only the first of them.
pub(crate) fn user_span(program: &cpl_ir::Program, frames: &[(&str, Span)], span: Span) -> Span {
    let library = |name: &str| {
        let id = program.fns.iter().position(|f| f.name == name);
        id.is_some_and(|id| program.library_fns.get(id).copied().unwrap_or(false))
    };
    let run = frames.iter().rev().take_while(|(name, _)| library(name));
    run.last().map_or(span, |&(_, call)| call)
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

/// A call stack's frames with each run of one call written once and counted,
/// `spin (a.cp:2) 40000 times`; the omitted middle (`None`, with its count)
/// joins a run on both sides of it.
pub(crate) fn runs(parts: &[(Option<String>, usize)]) -> Vec<String> {
    let mut out: Vec<(Option<String>, usize)> = Vec::new();
    for (i, (text, n)) in parts.iter().enumerate() {
        let around = |j: usize| parts.get(j).and_then(|p| p.0.as_ref());
        let text = match text {
            None if i > 0 && around(i - 1).is_some() && around(i - 1) == around(i + 1) => {
                around(i - 1).cloned()
            }
            t => t.clone(),
        };
        match out.last_mut() {
            Some((last, count)) if last.is_some() && *last == text => *count += n,
            _ => out.push((text, *n)),
        }
    }
    let show = |(text, n): (Option<String>, usize)| match text {
        None => format!("... {n} frames omitted ..."),
        Some(t) if n > 1 => format!("{t} {n} times"),
        Some(t) => t,
    };
    out.into_iter().map(show).collect()
}
