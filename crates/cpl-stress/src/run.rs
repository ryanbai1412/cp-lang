//! The stress loop: make a case, run the solution and the brute, judge,
//! and on the first failure shrink it and save it.

use crate::choices::{self, Drawn};
use crate::gen::{generate, Case, Size};
use crate::judge::{Judge, Verdict};
use crate::program::Program;
use crate::rng::Rng;
use crate::shrink::shrink;
use crate::spec::InputSpec;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where inputs come from.
#[derive(Clone, Debug)]
pub enum Source {
    /// Made from the program's own `read()` types, and shrinkable.
    Typed(InputSpec),
    /// A generator run as `gen <seed> <size>`; its output is the input.
    Generator(Program),
    /// The file's own `@gen` fn, built for one size. It shrinks through
    /// the choices its `rand` and `shuffle` made.
    InFile(Program),
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Report {
    /// How many cases ran, the failing one included.
    pub cases: usize,
    /// Seeds whose case could not be made (an empty range).
    pub skipped: usize,
    pub failure: Option<Failure>,
}

/// Something that stopped the run before a verdict.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StressError(pub String);

impl std::fmt::Display for StressError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

struct Runner<'a> {
    fast: &'a Program,
    brute: Option<&'a Program>,
    judge: &'a Judge,
    limit: Duration,
}

/// What a case was made from, kept to shrink it.
enum Made {
    Typed(Case),
    Drawn(Drawn),
    /// An outside generator's output: there is nothing to shrink it by.
    Opaque,
}

/// One run: `None` when the solution is right.
type Wrong = Option<(Verdict, String, String)>;

impl Runner<'_> {
    fn run(&self, input: &str) -> Result<Wrong, StressError> {
        let io = |program: &Program, e: std::io::Error| {
            StressError(format!("can't run {}: {e}", program.name))
        };
        let expected = match self.brute {
            Some(brute) => {
                // A brute is slow by design; it gets ten times the limit.
                let outcome = brute
                    .run(input, self.limit * 10)
                    .map_err(|e| io(brute, e))?;
                if !outcome.ok() {
                    return Err(StressError(format!(
                        "the brute {} failed on this input ({:?}), so there is nothing to compare with:\n{input}",
                        brute.name, outcome.exit
                    )));
                }
                Some(outcome.stdout)
            }
            None => None,
        };
        let judged = self
            .judge
            .run(self.fast, input, expected.as_deref(), self.limit)
            .map_err(|e| io(self.fast, e))?;
        Ok((!judged.verdict.accepted()).then(|| {
            (
                judged.verdict,
                expected.unwrap_or_default(),
                judged.outcome.stdout,
            )
        }))
    }
}

/// Runs `options.cases` cases. Stops at the first one, by case number,
/// that the solution gets wrong.
pub fn stress(
    source: &Source,
    fast: &Program,
    brute: Option<&Program>,
    judge: &Judge,
    options: &Options,
) -> Result<Report, StressError> {
    let runner = Runner {
        fast,
        brute,
        judge,
        limit: options.limit,
    };
    let size_arg = options.size.to_string();
    let make = |seed: u64| -> Result<Option<(Made, String)>, StressError> {
        match source {
            Source::Typed(spec) => {
                Ok(
                    generate(spec, &mut Rng::new(seed), Size::Small(options.size)).map(|case| {
                        let input = case.render();
                        (Made::Typed(case), input)
                    }),
                )
            }
            Source::Generator(generator) => {
                let outcome = generator
                    .with_args(&[&seed.to_string(), &size_arg])
                    .run("", Duration::from_secs(10))
                    .map_err(|e| StressError(format!("can't run {}: {e}", generator.name)))?;
                if !outcome.ok() {
                    return Err(StressError(format!(
                        "the generator {} failed on seed {seed} ({:?})",
                        generator.name, outcome.exit
                    )));
                }
                Ok(Some((Made::Opaque, outcome.stdout)))
            }
            Source::InFile(generator) => {
                let drawn = choices::draw(generator, seed)?;
                let input = drawn.input.clone();
                Ok(Some((Made::Drawn(drawn), input)))
            }
        }
    };

    let jobs = options.jobs.max(1);
    let mut skipped = 0;
    let mut done = 0;
    while done < options.cases {
        let batch: Vec<usize> = (done..options.cases.min(done + jobs)).collect();
        // A batch runs at once; results are read in case order, so the
        // failure reported is the same whatever `jobs` is.
        let results: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|index| {
                    let seed = options.seed.wrapping_add(*index as u64);
                    let (make, runner) = (&make, &runner);
                    scope.spawn(move || -> Result<_, StressError> {
                        let Some((case, input)) = make(seed)? else {
                            return Ok(None);
                        };
                        Ok(Some((case, runner.run(&input)?, input)))
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        for (index, result) in batch.iter().zip(results) {
            let Some((case, wrong, input)) = result? else {
                skipped += 1;
                continue;
            };
            let Some((verdict, expected, got)) = wrong else {
                continue;
            };
            let seed = options.seed.wrapping_add(*index as u64);
            let mut failure = Failure {
                case: index + 1,
                seed,
                input,
                described: None,
                expected,
                got,
                verdict,
                shrunk: None,
            };
            // Same bug only: a candidate must fail the same way.
            let kind = std::mem::discriminant(&failure.verdict);
            if let (Source::Typed(spec), Made::Typed(case)) = (source, &case) {
                let case = case.clone();
                let mut error = None;
                let smallest =
                    shrink(
                        spec,
                        &case,
                        options.shrink_budget,
                        &mut |candidate| match runner.run(&candidate.render()) {
                            Ok(Some((verdict, _, _))) => std::mem::discriminant(&verdict) == kind,
                            Ok(None) => false,
                            Err(e) => {
                                error.get_or_insert(e);
                                false
                            }
                        },
                    );
                if smallest != case {
                    if let Some((verdict, expected, got)) = runner.run(&smallest.render())? {
                        failure.verdict = verdict;
                        failure.expected = expected;
                        failure.got = got;
                        failure.input = smallest.render();
                    }
                }
                failure.shrunk = Some((case.weight().0, smallest.weight().0));
                failure.described = Some(smallest.describe(spec));
            }
            if let (Source::InFile(generator), Made::Drawn(drawn)) = (source, &case) {
                let smallest = choices::shrink(generator, drawn, options.shrink_budget, &mut |input| {
                    matches!(runner.run(input), Ok(Some((verdict, _, _))) if std::mem::discriminant(&verdict) == kind)
                });
                if smallest != *drawn {
                    if let Some((verdict, expected, got)) = runner.run(&smallest.input)? {
                        failure.verdict = verdict;
                        failure.expected = expected;
                        failure.got = got;
                        failure.input = smallest.input.clone();
                    }
                }
                let numbers = |input: &str| input.split_whitespace().count();
                failure.shrunk = Some((numbers(&drawn.input), numbers(&smallest.input)));
            }
            return Ok(Report {
                cases: index + 1,
                skipped,
                failure: Some(failure),
            });
        }
        done += batch.len();
    }
    Ok(Report {
        cases: options.cases,
        skipped,
        failure: None,
    })
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
