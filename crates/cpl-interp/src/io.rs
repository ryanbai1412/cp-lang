//! What a run reads: [`Io`].

use crate::{More, Stop};

/// What a program reads.
pub struct Io<'a> {
    /// What `read()` reads.
    pub(crate) input: Feed<'a>,
    /// What an `@interactor` hears from the solution: `sol.recv()`.
    pub(crate) peer: Option<More<'a>>,
    /// Where `rand` starts, when not the default.
    pub(crate) seed: Option<u64>,
    /// What ends the run early, if anything.
    pub(crate) stop: Option<Stop>,
}

pub(crate) enum Feed<'a> {
    Fixed(&'a [u8]),
    Live(More<'a>),
}

impl<'a> Io<'a> {
    /// The whole input, known before the program starts.
    pub fn fixed(input: &'a [u8]) -> Io<'a> {
        Io {
            input: Feed::Fixed(input),
            peer: None,
            seed: None,
            stop: None,
        }
    }

    /// An input that arrives while the program runs: the solution's side
    /// of an interactive problem.
    pub fn live(more: More<'a>) -> Io<'a> {
        Io {
            input: Feed::Live(more),
            peer: None,
            seed: None,
            stop: None,
        }
    }

    /// The judge's side: `input` is the hidden test, and `solution` is
    /// what the solution prints.
    pub fn judge(input: &'a [u8], solution: More<'a>) -> Io<'a> {
        Io {
            input: Feed::Fixed(input),
            peer: Some(solution),
            seed: None,
            stop: None,
        }
    }

    /// The same, with `rand` and `shuffle` drawing from `seed`: one
    /// seed, one run.
    pub fn seeded(self, seed: u64) -> Io<'a> {
        Io {
            seed: Some(seed),
            ..self
        }
    }
}
