//! What a run reads: [`Io`].

use crate::{More, Stop};
use std::sync::{Arc, Mutex};

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
    /// The file's text, when the run reports each top-level variable's
    /// last value.
    pub(crate) last_values: Option<&'a str>,
    /// Where `rand` and `shuffle` write down their choices, and the
    /// choices they replay.
    pub(crate) choices: Option<Arc<Mutex<Choices>>>,
}

/// The choices a run's `rand` and `shuffle` made: each is how far above
/// its low end a draw landed, with the source offset that drew it. With
/// `replay`, the draws take those choices in order (0 past its end, the
/// highest a draw allows when one is too big) instead of the seed's.
#[derive(Default, Debug)]
pub struct Choices {
    pub replay: Option<Vec<u64>>,
    pub made: Vec<(u32, u64)>,
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
            last_values: None,
            choices: None,
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
            last_values: None,
            choices: None,
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
            last_values: None,
            choices: None,
        }
    }

    /// The same, and the run's recording holds each top-level variable's
    /// last value ([`crate::trace::Recording::lives`]) as a traced run's
    /// does, without tracing it. `source` is the file's text, for their
    /// lines.
    pub fn with_last_values(self, source: &'a str) -> Io<'a> {
        Io {
            last_values: Some(source),
            ..self
        }
    }

    /// The same, with `rand` and `shuffle` writing their choices down in
    /// `choices`, or replaying the ones it holds.
    pub fn choosing(self, choices: Arc<Mutex<Choices>>) -> Io<'a> {
        Io {
            choices: Some(choices),
            ..self
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

/// A draw in `0..span` by `rand` or `shuffle` at `site`, as `cpl-codegen`'s
/// `rng_below` makes it: replayed from `choices` or drawn from `rng`, and
/// written down there.
pub(crate) fn draw_below(
    choices: Option<&std::sync::Mutex<Choices>>,
    rng: &mut u64,
    span: u128,
    site: u32,
) -> u128 {
    let Some(choices) = choices else {
        return splitmix64(rng) as u128 % span;
    };
    let mut choices = choices.lock().unwrap();
    let at = choices.made.len();
    let pick = match &choices.replay {
        Some(list) => (list.get(at).copied().unwrap_or(0) as u128).min(span - 1),
        None => splitmix64(rng) as u128 % span,
    };
    choices.made.push((site, pick as u64));
    pick
}

pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
