//! What ends a run before it finishes: its stack running out, a deadline,
//! or a cancel another thread asks for. The interpreter counts down at every loop round and
//! call, traced or not, and looks at the [`Stop`] when the count runs out,
//! so a run with nothing to stop it pays one decrement there.

use crate::{observe::Observer, trap, Interp, Io, Res};
use cpl_diag::Span;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// How much stack a program's calls may take before the run is stopped.
/// There is no limit on how deep calls nest: only on the stack they fill.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Bytes of the interpreter's own stack those calls may use: the
    /// program's recursion is the interpreter's.
    pub stack_bytes: usize,
}

/// The message of the trap that ends a run at its deadline.
pub const OUT_OF_TIME: &str = "the run took longer than its time limit";
/// The message of the trap that ends a run [`Stop::cancel`] stopped.
pub const CANCELLED: &str = "the run was stopped";

/// Loop rounds and calls between two looks at the [`Stop`].
pub(crate) const EVERY: u32 = 4096;

/// What ends a run before it finishes. Clones share the cancel, so the
/// caller keeps one and hands the run the other.
#[derive(Clone, Debug, Default)]
pub struct Stop {
    cancelled: Arc<AtomicBool>,
    /// None in the browser, which has no clock; it stops a run by ending
    /// its worker.
    deadline: Option<std::time::Instant>,
}

impl Stop {
    /// A stop that only [`Stop::cancel`] sets off.
    pub fn new() -> Stop {
        Stop::default()
    }

    /// A stop that also goes off once the run has taken `limit`.
    pub fn after(limit: std::time::Duration) -> Stop {
        Stop {
            #[cfg(not(target_family = "wasm"))]
            deadline: std::time::Instant::now().checked_add(limit),
            #[cfg(target_family = "wasm")]
            deadline: {
                let _ = limit;
                None
            },
            ..Stop::default()
        }
    }

    /// Ends the run at its next loop round or call.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Why the run must end now, if it must: the trap message.
    pub(crate) fn reason(&self) -> Option<&'static str> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Some(CANCELLED);
        }
        #[cfg(not(target_family = "wasm"))]
        if self
            .deadline
            .is_some_and(|d| std::time::Instant::now() >= d)
        {
            return Some(OUT_OF_TIME);
        }
        None
    }
}

impl<'a> Io<'a> {
    /// The same, ended as a trap ([`OUT_OF_TIME`] or [`CANCELLED`]) when
    /// `stop` goes off: every caller that must answer bounds a run this way.
    pub fn until(self, stop: Stop) -> Io<'a> {
        Io {
            stop: Some(stop),
            ..self
        }
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// Counts a loop round or a call; every [`EVERY`] of them, a trap
    /// when the run's [`Stop`] has gone off.
    #[inline(always)]
    pub(crate) fn tick(&mut self, span: Span) -> Res<()> {
        self.countdown -= 1;
        if self.countdown == 0 {
            return self.look_at_stop(span);
        }
        Ok(())
    }

    #[cold]
    fn look_at_stop(&mut self, span: Span) -> Res<()> {
        let Some(stop) = &self.stop else {
            self.countdown = u32::MAX;
            return Ok(());
        };
        self.countdown = EVERY;
        match stop.reason() {
            Some(reason) => trap(span, reason),
            None => Ok(()),
        }
    }

    pub(crate) fn stack_left(&self, span: Span, callee: Option<&str>) -> Res<()> {
        let here = 0u8;
        let used = self.stack_top.abs_diff(std::ptr::addr_of!(here) as usize);
        if used > self.stack_budget {
            let callee = callee.map_or("a closure".into(), |name| format!("`{name}`"));
            return trap(
                span,
                format!("the stack ran out: {callee} never stops calling itself?"),
            );
        }
        Ok(())
    }
}
