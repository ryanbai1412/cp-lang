//! The stress loop: make a case, run the solution and the brute, judge,
//! and on the first failure shrink it and save it.

use crate::choices::{self, Drawn};
use crate::engine::{ChoiceSource, Engine, ProcessChoices, ProcessEngine, Wrong};
use crate::gen::{generate, Case, Size};
use crate::judge::{Judge, Verdict};
use crate::program::Program;
use crate::rng::Rng;
use crate::shrink::shrink;
use crate::spec::{InputSpec, DEFAULT_INT};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where inputs come from (process-based CLI form).
#[derive(Clone, Debug)]
pub enum Source {
    /// Made from the program's own `read()` types, and shrinkable.
    Typed(InputSpec),
    /// A generator run as `gen <seed> <size>`; its output is the input.
    Generator(Program),
    /// The file's own `@gen` fn, built for one size. It shrinks through
    /// the choices its `rand` and `shuffle` made.
    InFile(Program),
    /// The file's own `@gen` fn, interpreted where no C++ compiler is.
    Interpreted {
        program: Box<cpl_ir::Program>,
        name: String,
    },
}

/// Engine-agnostic input source for [`stress_with`].
pub enum InputSource<'a> {
    /// Made from the program's own `read()` types, and shrinkable.
    Typed(&'a InputSpec),
    /// An external generator run as `gen <seed> <size>`; its output is the input.
    Generator(&'a Program),
    /// An in-file `@gen` fn behind a [`ChoiceSource`].
    Choices(&'a dyn ChoiceSource),
}

#[derive(Clone, Debug)]
pub struct Options {
    pub cases: usize,
    /// The largest `n` a case uses, so a brute finishes.
    pub size: i128,
    /// The first seed; case `i` uses `seed + i`.
    pub seed: u64,
    pub limit: Duration,
    /// How many cases run at once.
    pub jobs: usize,
    /// How many runs shrinking may spend.
    pub shrink_budget: usize,
    /// Cases saved before, by seed (`<file>.tests/stress-<seed>.in`). They
    /// run first, and one that still fails is reported as it is.
    pub saved: Vec<(u64, String)>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            cases: 1000,
            size: 8,
            seed: 1,
            limit: Duration::from_secs(5),
            jobs: std::thread::available_parallelism().map_or(1, |n| n.get()),
            shrink_budget: 2000,
            saved: Vec::new(),
        }
    }
}

/// A case the solution got wrong.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure {
    /// 1-based.
    pub case: usize,
    pub seed: u64,
    /// The input after shrinking.
    pub input: String,
    /// The input by the program's names (`n = 3  a = [1, 5, 2]`), when
    /// it was made from types.
    pub described: Option<String>,
    /// What the brute printed.
    pub expected: String,
    pub got: String,
    pub verdict: Verdict,
    /// How many numbers the case had before and after shrinking.
    pub shrunk: Option<(usize, usize)>,
    /// One of [`Options::saved`], still failing.
    pub saved: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Report {
    /// How many cases ran, the failing one included.
    pub cases: usize,
    /// Seeds whose case could not be made (an empty range).
    pub skipped: usize,
    pub failure: Option<Failure>,
    /// Inputs thrown away: they stopped the program for what a guessed
    /// number was, not for what the program does with it.
    pub discarded: usize,
    /// What was learned from them, one line each: `n is 2 or more`.
    pub narrowed: Vec<String>,
    /// How many of [`Options::saved`] passed before the new cases.
    pub saved: usize,
}

/// Something that stopped the run before a verdict.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StressError(pub String);

impl std::fmt::Display for StressError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a case was made from, kept to shrink it.
enum Made {
    Typed(Case),
    Drawn(Drawn),
    /// An outside generator's output: there is nothing to shrink it by.
    Opaque,
}

/// How a run on an input that may be no input at all went.
enum Tried {
    /// The solution ran and was judged.
    Judged(Wrong),
    /// The program stopped, and nothing that ran says it should not
    /// have: the brute stopped too, or there is none.
    Stopped,
}

/// Like `engine.run`, for an input made from guessed ranges.
fn attempt(engine: &dyn Engine, input: &str) -> Result<Tried, StressError> {
    match engine.run(input) {
        Ok(Some((Verdict::RuntimeError(_) | Verdict::TimeLimit, _, _))) if !engine.has_brute() => {
            Ok(Tried::Stopped)
        }
        Ok(Some((Verdict::BruteError(_), _, _))) if engine.stops(input) => Ok(Tried::Stopped),
        Ok(wrong) => Ok(Tried::Judged(wrong)),
        Err(_) if engine.stops(input) => Ok(Tried::Stopped),
        Err(e) => Err(e),
    }
}

/// Runs `options.cases` cases using OS processes. Stops at the first one, by
/// case number, that the solution gets wrong.
pub fn stress(
    source: &Source,
    fast: &Program,
    brute: Option<&Program>,
    judge: &Judge,
    options: &Options,
) -> Result<Report, StressError> {
    stress_with_cold(source, fast, brute, None, judge, options)
}

/// Like [`stress`], also comparing each accepted run against a `@cache`
/// cold-mode build when `cold` is `Some`.
pub fn stress_with_cold(
    source: &Source,
    fast: &Program,
    brute: Option<&Program>,
    cold: Option<&Program>,
    judge: &Judge,
    options: &Options,
) -> Result<Report, StressError> {
    let engine = ProcessEngine {
        fast,
        brute,
        cold,
        judge,
        limit: options.limit,
    };
    let (choices, interpreted);
    let input_source = match source {
        Source::Typed(spec) => InputSource::Typed(spec),
        Source::Generator(program) => InputSource::Generator(program),
        Source::InFile(program) => {
            choices = ProcessChoices(program);
            InputSource::Choices(&choices)
        }
        Source::Interpreted { program, name } => {
            interpreted = crate::InterpChoices {
                program: (**program).clone(),
                name: name.clone(),
                stack_limit: None,
                limit: options.limit,
            };
            InputSource::Choices(&interpreted)
        }
    };
    stress_with(&input_source, &engine, options)
}

/// Engine-agnostic stress loop: generates inputs from `source`, evaluates them
/// through `engine`, narrows guessed ranges, and shrinks the first failure.
pub fn stress_with(
    source: &InputSource<'_>,
    engine: &dyn Engine,
    options: &Options,
) -> Result<Report, StressError> {
    for (index, (seed, input)) in options.saved.iter().enumerate() {
        if let Some((verdict, expected, got)) = engine.run(input)? {
            let failure = Failure {
                case: index + 1,
                seed: *seed,
                input: input.clone(),
                described: None,
                expected,
                got,
                verdict,
                shrunk: None,
                saved: true,
            };
            return Ok(Report {
                cases: index + 1,
                skipped: 0,
                failure: Some(failure),
                discarded: 0,
                narrowed: Vec::new(),
                saved: index,
            });
        }
    }
    let saved = options.saved.len();
    let size_arg = options.size.to_string();
    // The ranges cases are drawn from: the program's own, narrowed where
    // a guessed one let through numbers the program stops on.
    let mut typed = match source {
        InputSource::Typed(spec) => Some((*spec).clone()),
        _ => None,
    };
    let mut discarded = 0;
    let mut narrowed: Vec<String> = Vec::new();
    let mut skipped = 0;
    let mut done = 0;
    'narrow: loop {
        let current = match &typed {
            Some(spec) => InputSource::Typed(spec),
            None => match source {
                InputSource::Typed(spec) => InputSource::Typed(spec),
                InputSource::Generator(p) => InputSource::Generator(p),
                InputSource::Choices(c) => InputSource::Choices(*c),
            },
        };
        let make = |seed: u64| -> Result<Option<(Made, String)>, StressError> {
            match &current {
                InputSource::Typed(spec) => Ok(generate(
                    spec,
                    &mut Rng::new(seed),
                    Size::Small(options.size),
                )
                .map(|case| {
                    let input = case.render();
                    (Made::Typed(case), input)
                })),
                InputSource::Generator(generator) => {
                    let outcome = generator
                        .with_args(&[&seed.to_string(), &size_arg])
                        .run("", Duration::from_secs(10))
                        .map_err(|e| StressError(format!("can't run {}: {e}", generator.name)))?;
                    if !outcome.ok() {
                        let said = outcome.stderr.trim_end();
                        let said = said.rsplit('\n').take(5).collect::<Vec<_>>();
                        let said: Vec<&str> = said.into_iter().rev().collect();
                        return Err(StressError(format!(
                            "the generator {} failed on seed {seed} ({:?}){}",
                            generator.name,
                            outcome.exit,
                            if said.is_empty() || said == [""] {
                                String::new()
                            } else {
                                format!(":\n{}", said.join("\n"))
                            }
                        )));
                    }
                    Ok(Some((Made::Opaque, outcome.stdout)))
                }
                InputSource::Choices(generator) => {
                    let drawn = generator.draw(seed)?;
                    let input = drawn.input.clone();
                    Ok(Some((Made::Drawn(drawn), input)))
                }
            }
        };

        let jobs = if cfg!(target_family = "wasm") {
            1
        } else {
            options.jobs.max(1)
        };
        let round = done;
        while done < options.cases {
            // The first case of a round runs alone: when it stops the
            // program the ranges are narrowed, and its neighbours, made from
            // the ranges it refutes, would only have been waited for.
            let jobs = if done == round { 1 } else { jobs };
            let batch: Vec<usize> = (done..options.cases.min(done + jobs)).collect();
            type BatchItem = Result<Option<(Made, Tried, String)>, StressError>;
            let results: Vec<BatchItem> = if jobs <= 1 {
                batch
                    .iter()
                    .map(|index| {
                        let seed = options.seed.wrapping_add(*index as u64);
                        let Some((case, input)) = make(seed)? else {
                            return Ok(None);
                        };
                        Ok(Some((case, attempt(engine, &input)?, input)))
                    })
                    .collect()
            } else {
                // A batch runs at once; results are read in case order, so the
                // failure reported is the same whatever `jobs` is.
                std::thread::scope(|scope| {
                    let handles: Vec<_> = batch
                        .iter()
                        .map(|index| {
                            let seed = options.seed.wrapping_add(*index as u64);
                            let make = &make;
                            scope.spawn(move || -> Result<_, StressError> {
                                let Some((case, input)) = make(seed)? else {
                                    return Ok(None);
                                };
                                Ok(Some((case, attempt(engine, &input)?, input)))
                            })
                        })
                        .collect();
                    handles.into_iter().map(|h| h.join().unwrap()).collect()
                })
            };
            for (index, result) in batch.iter().zip(results) {
                let Some((case, tried, input)) = result? else {
                    skipped += 1;
                    continue;
                };
                let seed = options.seed.wrapping_add(*index as u64);
                let wrong = match tried {
                    Tried::Judged(wrong) => wrong,
                    Tried::Stopped => {
                        // A narrower range that this seed runs under is taken,
                        // and the cases from here on are made again from it.
                        let found = match (&typed, &case) {
                            (Some(spec), Made::Typed(case)) => {
                                narrow(spec, case, seed, options.size, engine)
                            }
                            _ => None,
                        };
                        if let Some((spec, learned)) = found {
                            typed = Some(spec);
                            narrowed
                                .retain(|line| line.split(' ').next() != learned.split(' ').next());
                            narrowed.push(learned);
                            discarded += 1;
                            done = *index;
                            continue 'narrow;
                        }
                        // Nothing explains it away: it is reported as it is.
                        engine.run(&input)?
                    }
                };
                let Some((verdict, expected, got)) = wrong else {
                    continue;
                };
                let mut failure = Failure {
                    case: index + 1,
                    seed,
                    input,
                    described: None,
                    expected,
                    got,
                    verdict,
                    shrunk: None,
                    saved: false,
                };
                // Same bug only: a candidate must fail the same way.
                let kind = std::mem::discriminant(&failure.verdict);
                let fails_as = |verdict: &Verdict| std::mem::discriminant(verdict) == kind;
                let limits = shrink_limits(engine, matches!(failure.verdict, Verdict::TimeLimit));
                if let (InputSource::Typed(spec), Made::Typed(case)) = (&current, &case) {
                    let mut smallest = case.clone();
                    for &within in &limits {
                        let found = shrink(spec, case, options.shrink_budget, &mut |candidate| {
                            matches!(engine.run_within(&candidate.render(), within),
                                Ok(Some((verdict, _, _))) if fails_as(&verdict))
                        });
                        if found == *case {
                            break;
                        }
                        if let Some((verdict, expected, got)) = engine.run(&found.render())? {
                            if fails_as(&verdict) {
                                failure.verdict = verdict;
                                failure.expected = expected;
                                failure.got = got;
                                failure.input = found.render();
                                smallest = found;
                                break;
                            }
                        }
                    }
                    failure.shrunk = Some((case.weight().0, smallest.weight().0));
                    failure.described = Some(smallest.describe(spec));
                }
                if let (InputSource::Choices(generator), Made::Drawn(drawn)) = (&current, &case) {
                    let mut smallest = drawn.clone();
                    for &within in &limits {
                        let found = choices::shrink_with(
                            *generator,
                            drawn,
                            options.shrink_budget,
                            &mut |input| matches!(engine.run_within(input, within), Ok(Some((verdict, _, _))) if fails_as(&verdict)),
                        );
                        if found == *drawn {
                            break;
                        }
                        if let Some((verdict, expected, got)) = engine.run(&found.input)? {
                            if fails_as(&verdict) {
                                failure.verdict = verdict;
                                failure.expected = expected;
                                failure.got = got;
                                failure.input = found.input.clone();
                                smallest = found;
                                break;
                            }
                        }
                    }
                    let numbers = |input: &str| input.split_whitespace().count();
                    failure.shrunk = Some((numbers(&drawn.input), numbers(&smallest.input)));
                }
                return Ok(Report {
                    cases: index + 1,
                    skipped,
                    failure: Some(failure),
                    discarded,
                    narrowed,
                    saved,
                });
            }
            done += batch.len();
        }
        return Ok(Report {
            cases: options.cases,
            skipped,
            failure: None,
            discarded,
            narrowed,
            saved,
        });
    }
}

/// How much sooner a candidate is stopped while a time limit shrinks:
/// a stuck run costs a hundredth of the limit, not all of it.
const TLE_SHRINK_SPLIT: u32 = 100;

/// The limits a failure shrinks under. A time limit shrinks under a short
/// one first; the case it reaches must still run past the full limit, or
/// it shrinks again under that.
fn shrink_limits(engine: &dyn Engine, time_limit: bool) -> Vec<Duration> {
    let full = engine.limit();
    if time_limit {
        vec![full / TLE_SHRINK_SPLIT, full]
    } else {
        vec![full]
    }
}

/// A narrower version of `spec` that explains why `case` stopped the
/// program, and what it says about the input. It must leave `case` out
/// and the program must run on its own case for `seed`. One loose `Int`
/// drawn as a size is tried first, then every guessed size one higher,
/// then every guessed size under the largest of `case`.
fn narrow(
    spec: &InputSpec,
    case: &Case,
    seed: u64,
    size: i128,
    engine: &dyn Engine,
) -> Option<(InputSpec, String)> {
    let runs = |spec: &InputSpec| {
        generate(spec, &mut Rng::new(seed), Size::Small(size))
            .is_some_and(|case| matches!(attempt(engine, &case.render()), Ok(Tried::Judged(_))))
    };
    let outside = |name: &str, lo: i128, hi: i128| {
        case.values_of(spec, name)
            .iter()
            .any(|value| *value < lo || *value > hi)
    };
    for name in spec.loose() {
        let counted = spec.counting(&name);
        if outside(&name, 1, size.max(1)) && runs(&counted) {
            return Some((counted, format!("{name} is small and 1 or more")));
        }
    }
    let mut next = spec.clone();
    while let Some(raised) = next.raising() {
        next = raised;
        let lowest = 1 + next.raised;
        if next
            .counted()
            .iter()
            .any(|name| outside(name, lowest, i128::MAX))
            && runs(&next)
        {
            let names = next.counted().join(", ");
            return Some((next, format!("{names}: {lowest} or more")));
        }
    }
    // A cell of an array that the program counts or indexes with: none
    // below 0, none below 1, then small as well.
    let top = DEFAULT_INT.1;
    for (lo, hi, learned) in [
        (0, top, "0 or more".to_string()),
        (1, top, "1 or more".to_string()),
        (0, size.max(1), format!("0 to {}", size.max(1))),
        (1, size.max(1), format!("1 to {}", size.max(1))),
    ] {
        if spec
            .cells
            .as_deref()
            .is_some_and(|&(was_lo, was_hi)| lo <= was_lo && hi >= was_hi)
        {
            continue;
        }
        if let Some(celled) = spec.celled(lo, hi).filter(|celled| runs(celled)) {
            return Some((celled, format!("cells of the arrays read: {learned}")));
        }
    }
    // A size too large for something the program holds. Only a brute
    // that stops on it too says the input is at fault.
    if !engine.has_brute() {
        return None;
    }
    let mut all = spec.clone();
    for name in spec.loose() {
        all = all.counting(&name);
    }
    let largest = all
        .counted()
        .iter()
        .flat_map(|name| case.values_of(spec, name))
        .max()?;
    let names = all.counted().join(", ");
    (1..largest)
        .rev()
        .take(16)
        .map_while(|most| Some((all.capping(most)?, most)))
        .find(|(capped, _)| runs(capped))
        .map(|(capped, most)| (capped, format!("{names}: {most} or less")))
}

/// Saves a failing case next to the solution as a regression test:
/// `<name>.tests/stress-<seed>.in` and `.out` (the brute's output).
/// Returns the path of the `.in` file.
pub fn save(solution: &Path, failure: &Failure) -> std::io::Result<PathBuf> {
    let dir = solution.with_extension("tests");
    std::fs::create_dir_all(&dir)?;
    let input = dir.join(format!("stress-{}.in", failure.seed));
    std::fs::write(&input, &failure.input)?;
    if !failure.expected.is_empty() {
        std::fs::write(input.with_extension("out"), &failure.expected)?;
    }
    Ok(input)
}
