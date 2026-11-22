//! Execution backends for `cpl-stress`.
//!
//! The stress loop, narrowing, and shrinkers in [`crate::run`] and
//! [`crate::shrink`] are engine-agnostic: they call into [`Engine`] (to run
//! solution, brute, and judge on one input) and [`ChoiceSource`] (to draw and
//! replay `@gen` choice streams).
//!
//! - [`ProcessEngine`] / [`ProcessChoices`]: native CLI backend using compiled
//!   binaries / child processes.
//! - [`InterpEngine`] / [`InterpChoices`]: in-process `cpl-interp` backend with
//!   no filesystem, subprocesses, or threads — shared by native tests and
//!   `cpl-wasm` in the browser.

use crate::choices::{self, Drawn};
use crate::cli::{report_json, swap_in_part_brutes, Stage};
use crate::judge::{first_difference, Judge, Verdict};
use crate::program::Program;
use crate::run::{stress_with, InputSource, Options, Report, StressError};
use crate::spec::InputSpec;
use cpl_diag::Severity;
use cpl_interp::trace::{first_divergence, stages as trace_stages, Divergence, TraceConfig};
use std::time::Duration;

/// One run's result: `None` when the solution is accepted, or
/// `Some((verdict, expected, got))` when it failed.
pub type Wrong = Option<(Verdict, String, String)>;

/// Runs the solution (and optional brute / judge) on one input.
pub trait Engine: Sync {
    /// True when a brute is configured alongside the solution.
    fn has_brute(&self) -> bool;

    /// True when `input` stops the solution (runtime error / TLE).
    fn stops(&self, input: &str) -> bool;

    /// Runs the brute (if any), solution, and judge on `input`.
    fn run(&self, input: &str) -> Result<Wrong, StressError>;
}

/// Runs an in-file `@gen` function by seed or by replaying a recorded choice list.
pub trait ChoiceSource: Sync {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError>;
    fn replay(&self, choices: &[u64]) -> Option<Drawn>;
}

/// Native CLI backend: runs compiled binaries / external programs as OS processes.
pub struct ProcessEngine<'a> {
    pub fast: &'a Program,
    pub brute: Option<&'a Program>,
    pub cold: Option<&'a Program>,
    pub judge: &'a Judge,
    pub limit: Duration,
}

impl Engine for ProcessEngine<'_> {
    fn has_brute(&self) -> bool {
        self.brute.is_some()
    }

    fn stops(&self, input: &str) -> bool {
        self.fast
            .run(input, self.limit)
            .is_ok_and(|outcome| !outcome.ok())
    }

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
        if !judged.verdict.accepted() {
            return Ok(Some((
                judged.verdict,
                expected.unwrap_or_default(),
                judged.outcome.stdout,
            )));
        }
        if let Some(cold) = self.cold {
            let cold_out = cold
                .run(input, self.limit * 10)
                .map_err(|e| io(cold, e))?;
            if !cold_out.ok() {
                return Ok(Some((
                    Verdict::Wrong(format!(
                        "@cache cold run failed ({:?}) while cached run succeeded",
                        cold_out.exit
                    )),
                    cold_out.stdout,
                    judged.outcome.stdout,
                )));
            }
            if let Some(why) = first_difference(&cold_out.stdout, &judged.outcome.stdout) {
                return Ok(Some((
                    Verdict::Wrong(format!("@cache changed the output ({why})")),
                    cold_out.stdout,
                    judged.outcome.stdout,
                )));
            }
        }
        Ok(None)
    }
}

/// Native CLI `@gen` backend using `CPL_SEED` / `CPL_CHOICES` on a compiled binary.
pub struct ProcessChoices<'a>(pub &'a Program);

impl ChoiceSource for ProcessChoices<'_> {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError> {
        choices::draw(self.0, seed)
    }

    fn replay(&self, list: &[u64]) -> Option<Drawn> {
        choices::replay(self.0, list)
    }
}

/// In-process `cpl-interp` backend: no subprocesses, no temp files, no threads.
pub struct InterpEngine {
    pub fast: cpl_ir::Program,
    pub brute: Option<cpl_ir::Program>,
    pub checker: Option<cpl_ir::Program>,
    /// Stack byte limit when running on the calling thread (wasm).
    pub stack_limit: Option<u32>,
}

fn run_ir(
    program: &cpl_ir::Program,
    io: cpl_interp::Io,
    stack_limit: Option<u32>,
) -> Result<String, (String, cpl_interp::Trap)> {
    let mut out = Vec::new();
    let res = match stack_limit {
        Some(bytes) => cpl_interp::run_with_on_this_thread(
            program,
            io,
            &mut out,
            cpl_interp::Limits {
                stack_bytes: bytes as usize,
            },
        ),
        None => cpl_interp::run_with(program, io, &mut out),
    };
    let text = String::from_utf8_lossy(&out).into_owned();
    match res {
        Ok(()) => Ok(text),
        Err(trap) => Err((text, trap)),
    }
}

fn observe_ir(
    program: &cpl_ir::Program,
    input: &[u8],
    source: &str,
    stack_limit: Option<u32>,
    assignments: bool,
) -> cpl_interp::Observed {
    let mut sink = Vec::new();
    let config = TraceConfig {
        source: source.to_string(),
        assignments,
        ..TraceConfig::default()
    };
    match stack_limit {
        Some(bytes) => cpl_interp::observe_on_this_thread(
            program,
            input,
            &mut sink,
            cpl_interp::Limits {
                stack_bytes: bytes as usize,
            },
            Some(config),
        ),
        None => cpl_interp::run_observed(program, cpl_interp::Io::fixed(input), &mut sink, Some(config)),
    }
}

impl Engine for InterpEngine {
    fn has_brute(&self) -> bool {
        self.brute.is_some()
    }

    fn stops(&self, input: &str) -> bool {
        run_ir(&self.fast, cpl_interp::Io::fixed(input.as_bytes()), self.stack_limit).is_err()
    }

    fn run(&self, input: &str) -> Result<Wrong, StressError> {
        let expected = match &self.brute {
            Some(brute) => match run_ir(brute, cpl_interp::Io::fixed(input.as_bytes()), self.stack_limit) {
                Ok(out) => Some(out),
                Err((_, trap)) => {
                    return Err(StressError(format!(
                        "the brute failed on this input ({}):\n{input}",
                        trap.message
                    )));
                }
            },
            None => None,
        };
        let got = match run_ir(&self.fast, cpl_interp::Io::fixed(input.as_bytes()), self.stack_limit) {
            Ok(out) => out,
            Err((out, trap)) => {
                return Ok(Some((
                    Verdict::RuntimeError(trap.message),
                    expected.unwrap_or_default(),
                    out,
                )));
            }
        };
        if let Some(checker) = &self.checker {
            let mut printed = Some(got.as_bytes().to_vec());
            let io = cpl_interp::Io::judge(input.as_bytes(), Box::new(move || printed.take()));
            return match run_ir(checker, io, self.stack_limit) {
                Ok(_) => Ok(None),
                Err((_, trap)) => Ok(Some((
                    Verdict::Wrong(trap.message),
                    expected.unwrap_or_default(),
                    got,
                ))),
            };
        }
        let exp = expected.unwrap_or_default();
        Ok(first_difference(&exp, &got).map(|why| (Verdict::Wrong(why), exp, got)))
    }
}

/// In-process `@gen` backend using `cpl-interp` with seeded `rand`/`shuffle`.
pub struct InterpChoices {
    pub program: cpl_ir::Program,
    pub name: String,
    pub stack_limit: Option<u32>,
}

impl ChoiceSource for InterpChoices {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError> {
        match run_ir(&self.program, cpl_interp::Io::fixed(b"").seeded(seed), self.stack_limit) {
            Ok(input) => Ok(Drawn {
                choices: Vec::new(),
                sites: Vec::new(),
                input,
            }),
            Err((_, trap)) => Err(StressError(format!(
                "the `@gen` fn of {} failed on seed {seed}: {}",
                self.name, trap.message
            ))),
        }
    }

    fn replay(&self, _choices: &[u64]) -> Option<Drawn> {
        None
    }
}

/// Full outcome of an in-process stress run, serializable to the same JSON as
/// `cpl stress --format json`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SourceStressResult {
    pub report: Report,
    pub stages: Vec<Stage>,
    pub explained: Option<Result<Option<Divergence>, String>>,
}

impl SourceStressResult {
    /// Serializes to the exact JSON format of `cpl stress --format json`.
    pub fn to_json(&self) -> String {
        report_json(
            &self.report,
            None,
            &self.stages,
            None,
            self.explained.as_ref(),
        )
    }
}

/// Runs stress testing completely in-process (no filesystem, no subprocesses, no
/// threads) on CP Lang `source`, using the shared generator and shrinker.
pub fn stress_source(
    name: &str,
    source: &str,
    options: &Options,
    stack_limit: Option<u32>,
    explain: bool,
) -> Result<SourceStressResult, StressError> {
    let mut db = cpl_check::Database::new();
    db.set_file(name, source);
    let analysis = db
        .analyze(name)
        .ok_or_else(|| StressError(format!("could not analyze {name}")))?;
    let diagnostics = cpl_ir::check(&analysis);
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        let lines = cpl_diag::LineIndex::new(source);
        let mut rendered = String::new();
        for d in &diagnostics {
            rendered.push_str(&cpl_diag::render(d, name, source, &lines));
        }
        return Err(StressError(rendered));
    }

    if cpl_ir::interactor(&analysis).is_some() {
        return Err(StressError(
            "in-process stress does not support `@interactor` yet; run `cpl stress` on the command line".to_string(),
        ));
    }

    let fast = cpl_ir::lower(&analysis).map_err(|diags| {
        let lines = cpl_diag::LineIndex::new(source);
        let text: String = diags
            .iter()
            .map(|d| cpl_diag::render(d, name, source, &lines))
            .collect();
        StressError(text)
    })?;

    let own_brute = cpl_ir::brute(&analysis);
    let brute = if let Some((brute_fn, _)) = &own_brute {
        Some(cpl_ir::lower_brute(&analysis, brute_fn).map_err(|diags| {
            let lines = cpl_diag::LineIndex::new(source);
            let text: String = diags
                .iter()
                .map(|d| cpl_diag::render(d, name, source, &lines))
                .collect();
            StressError(text)
        })?)
    } else if let Some(swapped) = swap_in_part_brutes(source) {
        let mut bdb = cpl_check::Database::new();
        let bname = format!(".{name}.brute.cp");
        bdb.set_file(&bname, swapped.as_str());
        let b_analysis = bdb
            .analyze(&bname)
            .ok_or_else(|| StressError(format!("could not analyze part brute of {name}")))?;
        Some(cpl_ir::lower(&b_analysis).map_err(|diags| {
            let lines = cpl_diag::LineIndex::new(&swapped);
            let text: String = diags
                .iter()
                .map(|d| cpl_diag::render(d, &bname, &swapped, &lines))
                .collect();
            StressError(text)
        })?)
    } else {
        None
    };

    let checker = match cpl_ir::checker(&analysis) {
        Some((checker_fn, _)) => Some(cpl_ir::lower_checker(&analysis, &checker_fn).map_err(|diags| {
            let lines = cpl_diag::LineIndex::new(source);
            let text: String = diags
                .iter()
                .map(|d| cpl_diag::render(d, name, source, &lines))
                .collect();
            StressError(text)
        })?),
        None => None,
    };

    if brute.is_none() && checker.is_none() {
        return Err(StressError(format!(
            "there is nothing to compare {name} with: write a `@brute(main)` / `@alt(program)` fn or a `@checker` / `@check` fn"
        )));
    }

    let own_gen = cpl_ir::generator(&analysis);
    let interp_choices = match &own_gen {
        Some((gen_fn, _)) => {
            let size = i64::try_from(options.size).unwrap_or(i64::MAX);
            let program = cpl_ir::lower_gen(&analysis, gen_fn, size).map_err(|diags| {
                let lines = cpl_diag::LineIndex::new(source);
                let text: String = diags
                    .iter()
                    .map(|d| cpl_diag::render(d, name, source, &lines))
                    .collect();
                StressError(text)
            })?;
            Some(InterpChoices {
                program,
                name: name.to_string(),
                stack_limit,
            })
        }
        None => None,
    };

    let typed_spec;
    let input_source = match &interp_choices {
        Some(choices) => InputSource::Choices(choices),
        None => match InputSpec::from_source(source) {
            Ok(spec) if spec.is_empty() => {
                return Err(StressError(format!(
                    "{name} has no typed `read()` to make inputs from; write `let n: 1..=100 = read()` or a `@gen fn`"
                )));
            }
            Ok(spec) => {
                typed_spec = spec;
                InputSource::Typed(&typed_spec)
            }
            Err(error) => {
                let line = cpl_diag::LineIndex::new(source).line_of(error.span.start) + 1;
                return Err(StressError(format!("{name}:{line}: {}", error.message)));
            }
        },
    };

    let engine = InterpEngine {
        fast,
        brute,
        checker,
        stack_limit,
    };
    let mut single = options.clone();
    single.jobs = 1;
    let report = stress_with(&input_source, &engine, &single)?;

    let mut stages = Vec::new();
    let mut explained = None;
    if let (Some(failure), Some(brute_prog)) = (&report.failure, &engine.brute) {
        if own_brute.is_some() {
            let fast_rec = observe_ir(&engine.fast, failure.input.as_bytes(), source, stack_limit, explain)
                .recording
                .unwrap_or_default();
            let slow_rec = observe_ir(brute_prog, failure.input.as_bytes(), source, stack_limit, explain)
                .recording
                .unwrap_or_default();
            stages = trace_stages(&fast_rec, &slow_rec)
                .into_iter()
                .map(|s| Stage {
                    name: s.name,
                    line: s.line,
                    fast: s.fast,
                    brute: s.brute,
                })
                .collect();
            if explain {
                explained = Some(Ok(first_divergence(&fast_rec, &slow_rec)));
            }
        }
    }

    Ok(SourceStressResult {
        report,
        stages,
        explained,
    })
}
