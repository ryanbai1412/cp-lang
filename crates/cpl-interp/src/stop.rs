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
    /// None in the browser, which has no clock: there `looks_left` keeps
    /// the time.
    deadline: Option<std::time::Instant>,
    /// Looks at the stop left before the run is out of time: a budget of
    /// loop rounds and calls, each look [`EVERY`] of them.
    looks_left: Option<u64>,
}

/// Loop rounds and calls a second in the browser, which runs the VM: what
/// [`Stop::after`] budgets there for each second of its limit. Under Node a
/// tight loop does about 2.7e7 a second and a hash map write in each round
/// about 7e6, so a run the budget ends would take about as long natively.
const ROUNDS_PER_SECOND: f64 = 1e7;

impl Stop {
    /// A stop that only [`Stop::cancel`] sets off.
    pub fn new() -> Stop {
        Stop::default()
    }

    /// A stop that also goes off once the run has taken `limit`.
    /// In the browser, which has no clock, the limit is a budget of loop
    /// rounds and calls ([`Stop::after_rounds`]).
    pub fn after(limit: std::time::Duration) -> Stop {
        if cfg!(target_family = "wasm") {
            return Stop::after_rounds((limit.as_secs_f64() * ROUNDS_PER_SECOND) as u64);
        }
        Stop {
            #[cfg(not(target_family = "wasm"))]
            deadline: std::time::Instant::now().checked_add(limit),
            ..Stop::default()
        }
    }

    /// A stop that goes off, as out of time, once the run has taken about
    /// `rounds` loop rounds and calls, on any clock.
    pub fn after_rounds(rounds: u64) -> Stop {
        Stop {
            looks_left: Some(rounds / u64::from(EVERY)),
            ..Stop::default()
        }
    }

    /// Ends the run at its next loop round or call.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Why the run must end now, if it must: the trap message.
    pub(crate) fn reason(&mut self) -> Option<&'static str> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Some(CANCELLED);
        }
        if let Some(left) = &mut self.looks_left {
            if *left == 0 {
                return Some(OUT_OF_TIME);
            }
            *left -= 1;
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
    /// when the run's [`Stop`] has gone off. True when a checkpoint is due,
    /// which only the slow path says, so the hot path stays a decrement.
    #[inline(always)]
    pub(crate) fn tick(&mut self, span: Span) -> Res<bool> {
        self.countdown -= 1;
        if self.countdown == 0 {
            return self.look_at_stop(span);
        }
        Ok(false)
    }

    #[cold]
    fn look_at_stop(&mut self, span: Span) -> Res<bool> {
        let watched = self.stop.is_some() || self.ckpt.on() || self.tracer.is_some();
        let next = self.ckpt.reload(if watched { EVERY } else { u32::MAX });
        self.countdown = self.ckpt.after_look(next);
        if let Some(reason) = self.stop.as_mut().and_then(Stop::reason) {
            return trap(span, reason);
        }
        self.unstopped(span)?;
        Ok(self.ckpt.due())
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
