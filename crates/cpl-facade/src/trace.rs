//! What tools read from a run's recording, in the facade's own forms (contract 8).

use cpl_interp::trace::{Assigned, Call, Live, Place, Shown, Step, Steps, Var};

/// What a run left behind: opaque, so a later engine can answer it from its event stream.
#[derive(Clone, Debug, Default)]
pub struct Recording(pub(crate) cpl_interp::trace::Recording);

impl From<cpl_interp::trace::Recording> for Recording {
    fn from(r: cpl_interp::trace::Recording) -> Recording {
        Recording(r)
    }
}

impl Recording {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn step(&self, step: usize) -> Option<Step> {
        self.0.step(step)
    }

    pub fn steps(&self, range: std::ops::Range<usize>) -> Steps<'_> {
        self.0.steps(range)
    }

    /// The 1-based line `step` ran.
    pub fn line(&self, step: usize) -> Option<u32> {
        self.0.line(step)
    }

    /// The calls running around `step`.
    pub fn depth(&self, step: usize) -> usize {
        self.0.depth(step)
    }

    pub fn checkpoints(&self) -> impl Iterator<Item = (usize, &str)> {
        self.0.checkpoints()
    }

    pub fn bytes(&self) -> usize {
        self.0.bytes()
    }

    /// `step` in source terms, which a recording after an edit finds again.
    pub fn place(&self, step: usize) -> Option<Place> {
        self.0.place(step)
    }

    /// The step at `place`, or `fallback`.
    pub fn find(&self, place: Place, fallback: usize) -> usize {
        self.0.find(place, fallback)
    }

    /// What a trace of this run holds, as its document's description says it.
    pub fn note(&self) -> &'static str {
        self.0.note()
    }

    /// Steps past the limit were not kept.
    pub fn truncated(&self) -> bool {
        self.0.truncated
    }

    /// The first step kept is step `skipped` of the run.
    pub fn skipped(&self) -> usize {
        self.0.skipped
    }

    /// The calls, in the order they started.
    pub fn calls(&self) -> &[Call] {
        &self.0.calls
    }

    pub fn vars(&self) -> &[Var] {
        &self.0.vars
    }

    pub fn lives(&self) -> &[Live] {
        &self.0.lives
    }

    pub fn shown(&self) -> &[Shown] {
        &self.0.shown
    }

    /// Each value a named variable was given, in the order they ran.
    pub fn assigned(&self) -> &[Assigned] {
        &self.0.assigned
    }
}

/// A checkpoint both runs reached, with what each showed there.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
}

impl From<cpl_interp::trace::Stage> for Stage {
    fn from(d: cpl_interp::trace::Stage) -> Stage {
        Stage {
            name: d.name,
            line: d.line,
            fast: d.fast,
            brute: d.brute,
        }
    }
}

/// Where a solution and its brute first give a variable they both name
/// different values: its `nth` value (1-based, a repeat of the value
/// before not counted), with the line and the value on each side, `None`
/// on a side that never gives it an `nth` value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    pub name: String,
    pub nth: usize,
    pub fast: Option<(u32, String)>,
    pub brute: Option<(u32, String)>,
    /// The solution's own last steps in a wrong call, not those of the
    /// calls it makes: each line and its locals, as `k: 1, s: 0`.
    pub inside: Vec<(u32, String)>,
    /// The lines to look at, each with why: where a wrong call made its
    /// value, where what that line reads last changed, or the declaration
    /// of a variable an earlier test left a value in.
    pub blame: Vec<(u32, String)>,
    /// `name` is a call, `f(k: 1)`, and `nth` which call of `f` it is.
    pub call: bool,
    /// The solution never finishes: `name` is the loop it goes round in,
    /// `fast` that loop's line and the state it keeps coming back to, and
    /// `nth` how many steps one time round takes.
    pub stuck: bool,
}

impl From<cpl_interp::trace::Divergence> for Divergence {
    fn from(d: cpl_interp::trace::Divergence) -> Divergence {
        Divergence {
            name: d.name,
            nth: d.nth,
            fast: d.fast,
            brute: d.brute,
            inside: d.inside,
            blame: d.blame,
            call: d.call,
            stuck: d.stuck,
        }
    }
}
