//! Inputs from the file's own `@gen` fn, and how they shrink.
//!
//! Every `rand(lo, hi)` and every step of a `shuffle` in the generator
//! is one choice: how far above `lo` it landed. `cpl stress` records
//! the choices of a failing case and replays shorter and smaller lists
//! of them, so a tree or a sorted list stays a tree or a sorted list
//! while it shrinks. A choice past the end of a list is 0, the low end.

use crate::program::Program;
use crate::run::StressError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// One case a generator printed, and the choices that made it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drawn {
    pub choices: Vec<u64>,
    /// The line of the generator each choice was made on.
    sites: Vec<u32>,
    pub input: String,
}

impl Drawn {
    /// Shorter first, then smaller: the order shrinking goes down.
    fn rank(&self) -> (usize, &[u64]) {
        (self.choices.len(), &self.choices)
    }
}

const LIMIT: Duration = Duration::from_secs(10);

/// A file no other run is using.
fn scratch() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("cpl-choices-{}-{id}", std::process::id()))
}

fn run(generator: &Program, env: &[(&str, String)]) -> std::io::Result<Option<Drawn>> {
    let log = scratch();
    let mut program = generator.clone();
    program.env.extend(env.iter().map(|(key, value)| (key.to_string(), value.clone())));
    program.env.push(("CPL_CHOICES_OUT".to_string(), log.display().to_string()));
    let outcome = program.run("", LIMIT);
    let choices = std::fs::read_to_string(&log).unwrap_or_default();
    let _ = std::fs::remove_file(&log);
    let outcome = outcome?;
    let ok = outcome.ok();
    let (sites, choices) = choices
        .lines()
        .filter_map(|line| {
            let (site, choice) = line.split_once(' ')?;
            Some((site.parse::<u32>().ok()?, choice.parse::<u64>().ok()?))
        })
        .unzip();
    let input = outcome.stdout;
    Ok(ok.then_some(Drawn { choices, sites, input }))
}

/// The case `seed` gives.
pub fn draw(generator: &Program, seed: u64) -> Result<Drawn, StressError> {
    let failed = |why: String| StressError(format!("the `@gen` fn of {} {why}", generator.name));
    match run(generator, &[("CPL_SEED", seed.to_string())]) {
        Ok(Some(drawn)) => Ok(drawn),
        Ok(None) => Err(failed(format!("failed on seed {seed}"))),
        Err(e) => Err(failed(format!("can't run: {e}"))),
    }
}

/// The case `choices` give, or `None` when the generator rejects them.
/// The choices it comes back with are the ones the generator used.
pub fn replay(generator: &Program, choices: &[u64]) -> Option<Drawn> {
    let list = scratch();
    let text: String = choices.iter().map(|choice| format!("{choice}\n")).collect();
    std::fs::write(&list, text).ok()?;
    let drawn = run(generator, &[("CPL_CHOICES", list.display().to_string())]);
    let _ = std::fs::remove_file(&list);
    drawn.ok().flatten()
}

/// The smallest case found, within `budget` runs, that `still_fails`.
pub fn shrink(
    generator: &Program,
    first: &Drawn,
    budget: usize,
    still_fails: &mut dyn FnMut(&str) -> bool,
) -> Drawn {
    let mut best = first.clone();
    let mut left = budget;
    let mut attempt = |best: &mut Drawn, choices: Vec<u64>| -> bool {
        if left == 0 {
            return false;
        }
        left -= 1;
        match replay(generator, &choices) {
            Some(candidate) if candidate.rank() < best.rank() && still_fails(&candidate.input) => {
                *best = candidate;
                true
            }
            _ => false,
        }
    };
    loop {
        let before = best.clone();
        // Drop runs of choices, long runs first, from the end back.
        let mut run = best.choices.len().next_power_of_two() / 2;
        while run > 0 {
            let mut start = best.choices.len().saturating_sub(run);
            loop {
                if start + run <= best.choices.len() {
                    let mut fewer = best.choices.clone();
                    fewer.drain(start..start + run);
                    attempt(&mut best, fewer);
                }
                if start == 0 {
                    break;
                }
                start = start.saturating_sub(run);
            }
            run /= 2;
        }
        // Drop one element of every list at once: a count goes down by
        // one, and each line that drew many times loses the same draw,
        // counted from its last. A tree of `n` loses a vertex this way,
        // its parent and its value together.
        let mut back = 1;
        loop {
            let mut seen: HashMap<u32, usize> = HashMap::new();
            best.sites.iter().for_each(|site| *seen.entry(*site).or_default() += 1);
            let mut from_last: HashMap<u32, usize> = HashMap::new();
            let (mut gone, mut counts) = (Vec::new(), Vec::new());
            for at in (0..best.choices.len()).rev() {
                let nth = from_last.entry(best.sites[at]).or_default();
                *nth += 1;
                if seen[&best.sites[at]] == 1 {
                    counts.push(at);
                } else if *nth == back {
                    gone.push(at);
                }
            }
            if gone.is_empty() {
                break;
            }
            let mut dropped = false;
            for count in counts {
                if dropped || best.choices[count] == 0 {
                    continue;
                }
                let mut fewer = best.choices.clone();
                fewer[count] -= 1;
                for &at in &gone {
                    fewer.remove(at);
                }
                dropped = attempt(&mut best, fewer);
            }
            if !dropped {
                back += 1;
            }
        }
        // Lower each choice: to 0, or else as far as it still fails.
        let mut at = 0;
        while at < best.choices.len() {
            let with = |best: &Drawn, value: u64| {
                let mut lower = best.choices.clone();
                lower[at] = value;
                lower
            };
            let high = best.choices[at];
            let zero = with(&best, 0);
            if high > 0 && !attempt(&mut best, zero) {
                // `passes` is a value that does not fail; `high` fails.
                let (mut passes, mut high) = (0, high);
                while passes + 1 < high && at < best.choices.len() {
                    let middle = passes + (high - passes) / 2;
                    let lower = with(&best, middle);
                    if attempt(&mut best, lower) {
                        high = middle;
                    } else {
                        passes = middle;
                    }
                }
            }
            at += 1;
        }
        if best == before {
            return best;
        }
    }
}
