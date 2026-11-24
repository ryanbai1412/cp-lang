//! How often each statement ran and how long the run spent there, keyed by
//! the statement's span in the user's file. Library code is lowered with
//! the span of the user's expression that asked for it, so its time lands
//! on that line.
//!
//! Time is a sampler's attribution: the clock runs on the statement that
//! most recently began, so a loop header is charged for its own test and a
//! call's line for the time until the callee's first statement. The C++ a
//! profiled build writes keeps the same books, so the two profiles agree on
//! every count.

use cpl_diag::Span;
use cpl_ir::{Program, Stmt};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// One statement's tally.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Site {
    /// How many times it began.
    pub hits: u64,
    /// Time from each start to the next statement's start, in nanoseconds.
    pub nanos: u64,
}

/// Every statement that ran, by span, in source order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    pub sites: Vec<(Span, Site)>,
}

pub(crate) struct Profiler {
    counted: HashSet<*const Stmt>,
    sites: HashMap<Span, Site>,
    last: Option<Span>,
    since: Instant,
}

impl Profiler {
    pub fn new(program: &Program) -> Profiler {
        Profiler {
            counted: cpl_ir::sites::user_stmts(program),
            sites: HashMap::new(),
            last: None,
            since: Instant::now(),
        }
    }

    pub fn enter(&mut self, stmt: &Stmt) {
        if !self.counted.contains(&(stmt as *const Stmt)) {
            return;
        }
        let Some(span) = stmt.span() else { return };
        let now = Instant::now();
        self.charge(now);
        self.last = Some(span);
        self.sites.entry(span).or_default().hits += 1;
    }

    fn charge(&mut self, now: Instant) {
        if let Some(last) = self.last {
            let spent = now.duration_since(self.since).as_nanos() as u64;
            self.sites.entry(last).or_default().nanos += spent;
        }
        self.since = now;
    }

    pub fn finish(mut self) -> Profile {
        self.charge(Instant::now());
        let mut sites: Vec<_> = self.sites.into_iter().collect();
        cpl_diag::sort_by_key(&mut sites, |(span, _)| span.order());
        Profile { sites }
    }
}
