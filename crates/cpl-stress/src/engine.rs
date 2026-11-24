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

use crate::case::{judge_case, Ran, Ruling, Sides, Talked, Wrong};
use crate::choices::{self, Drawn};
use crate::cli::{swap_in_part_brutes, Stage};
use crate::judge::{first_difference, Judge, Verdict};
use crate::program::Program;
use crate::report::StressResult;
use crate::run::StressError as Error;
use crate::run::{stress_with, InputSource, Options, StressError};
use crate::spec::InputSpec;
use cpl_diag::Severity;
use cpl_interp::trace::{
    explain_divergence, stages as trace_stages, stuck_divergence, TraceConfig,
};
use std::time::Duration;

/// Runs the solution (and optional brute / judge) on one input.
pub trait Engine: Sync {
    /// True when a brute is configured alongside the solution.
    fn has_brute(&self) -> bool;

    /// True when `input` stops the solution (runtime error / TLE).
    fn stops(&self, input: &str) -> bool;

    /// How long one run of the solution may take.
    fn limit(&self) -> Duration;

    /// Runs the brute (if any), solution, and judge on `input`, the
    /// solution stopped after `limit` and each helper after ten times it.
    fn run_within(&self, input: &str, limit: Duration) -> Result<Wrong, StressError>;

    /// [`run_within`](Engine::run_within) the engine's own limit.
    fn run(&self, input: &str) -> Result<Wrong, StressError> {
        self.run_within(input, self.limit())
    }
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

    fn limit(&self) -> Duration {
        self.limit
    }

    fn run_within(&self, input: &str, limit: Duration) -> Result<Wrong, StressError> {
        judge_case(self, input, limit)
    }
}

/// Why `program` could not be run at all.
fn unrunnable(program: &Program) -> impl Fn(std::io::Error) -> Error + '_ {
    move |e| Error(format!("can't run {}: {e}", program.name))
}

impl Sides for ProcessEngine<'_> {
    fn brute(&self, input: &str, limit: Duration) -> Result<Option<Ran>, Error> {
        let Some(brute) = self.brute else {
            return Ok(None);
        };
        let ran = crate::judge::run(brute, input, crate::case::brute_limit(limit));
        ran.map(|(ran, _)| Some(ran)).map_err(unrunnable(brute))
    }

    fn solution(&self, input: &str, limit: Duration) -> Result<Ran, Error> {
        let ran = crate::judge::run(self.fast, input, limit);
        ran.map(|(ran, _)| ran).map_err(unrunnable(self.fast))
    }

    fn check(
        &self,
        input: &str,
        got: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> Result<Option<Ruling>, Error> {
        match self.judge {
            Judge::Checker(checker) => crate::judge::check(checker, input, got, expected, limit)
                .map(Some)
                .map_err(unrunnable(checker)),
            _ => Ok(None),
        }
    }

    fn interact(&self, input: &str, limit: Duration) -> Result<Option<Talked>, Error> {
        match self.judge {
            Judge::Interactor(interactor, launch) => {
                crate::judge::interact(self.fast, interactor, *launch, input, limit)
                    .map(|(talked, _)| Some(talked))
                    .map_err(unrunnable(interactor))
            }
            _ => Ok(None),
        }
    }

    fn recheck(&self, input: &str, got: &str, limit: Duration) -> Result<Wrong, Error> {
        let Some(cold) = self.cold else {
            return Ok(None);
        };
        let cold_out = cold
            .run(input, crate::case::brute_limit(limit))
            .map_err(unrunnable(cold))?;
        let wrong = |why: String| {
            Some(crate::case::Failed::new(
                Verdict::Wrong(why),
                cold_out.stdout.clone(),
                got.to_string(),
            ))
        };
        if !cold_out.ok() {
            return Ok(wrong(format!(
                "@cache cold run failed ({:?}) while cached run succeeded",
                cold_out.exit
            )));
        }
        Ok(first_difference(&cold_out.stdout, got)
            .and_then(|why| wrong(format!("@cache changed the output ({why})"))))
    }
}

/// Native CLI `@gen` backend using `CPL_SEED` / `CPL_CHOICES` on a compiled binary.
/// The built `@gen` fn, and the same fn interpreted, which says on which
/// line a draw that fails traps.
pub struct ProcessChoices<'a>(pub &'a Program, pub Option<&'a InterpChoices>);

impl ChoiceSource for ProcessChoices<'_> {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError> {
        choices::draw(self.0, seed).map_err(|built| match self.1 {
            Some(interpreted) => interpreted.draw(seed).err().unwrap_or(built),
            None => built,
        })
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
    /// The file's `@interactor`: the solution runs against it.
    pub interactor: Option<FileJudge>,
    /// Stack byte limit when running on the calling thread (wasm).
    pub stack_limit: Option<u32>,
    /// How long one run may take before it is stopped, a time limit verdict.
    pub limit: Duration,
    /// Tells this engine's [`PRELUDES`] from another's.
    id: u64,
}

/// A judge fn of the file's, and the file, to say where it stops.
pub struct FileJudge {
    pub program: cpl_ir::Program,
    pub name: String,
    pub lines: cpl_diag::LineIndex,
}

/// Loop rounds and calls between the checkpoints a first run takes before
/// it reads: a precompute shorter than this is just run again.
const PRELUDE_EVERY: u64 = 1 << 16;

/// Which of an engine's programs a prelude is of.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Fast,
    Brute,
}

thread_local! {
    /// Per engine and program, once a run has shown it: the last checkpoint
    /// before the program reads or prints, or `None` when there is none.
    /// Every later case starts there, so what the program works out before
    /// it reads (a table over all `k <= 60`, say) runs once, not per case.
    static PRELUDES: std::cell::RefCell<Vec<PreludeOf>> = const { std::cell::RefCell::new(Vec::new()) };
}

type PreludeOf = ((u64, Role), Option<cpl_interp::checkpoint::Checkpoint>);

static ENGINES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl InterpEngine {
    /// [`run_ir`] on the calling thread, from the state `program` reaches
    /// before it first reads, once a first run has found it.
    fn run_case(
        &self,
        role: Role,
        program: &cpl_ir::Program,
        input: &str,
        limit: Duration,
    ) -> Result<String, (String, cpl_interp::Trap)> {
        use cpl_interp::checkpoint::Checkpoints;
        let io = cpl_interp::Io::fixed(input.as_bytes());
        let Some(bytes) = self.stack_limit else {
            return run_ir(program, io, None, limit);
        };
        let key = (self.id, role);
        let known = PRELUDES.with(|p| {
            let p = p.borrow();
            p.iter().find(|(k, _)| *k == key).map(|(_, c)| c.clone())
        });
        let want = match &known {
            None => Checkpoints::before_input(PRELUDE_EVERY),
            Some(Some(from)) => Checkpoints::resuming(from),
            Some(None) => Checkpoints::default(),
        };
        let mut out = Vec::new();
        let run = cpl_interp::checkpoint::observe_on_this_thread(
            program,
            io.until(cpl_interp::Stop::after(limit)),
            &mut out,
            cpl_interp::Limits {
                stack_bytes: bytes as usize,
            },
            None,
            want,
        );
        if known.is_none() {
            let prelude = run.taken.into_iter().last();
            PRELUDES.with(|p| {
                let mut p = p.borrow_mut();
                p.retain(|((id, _), _)| *id == self.id);
                p.push((key, prelude));
            });
        }
        let text = String::from_utf8_lossy(&out).into_owned();
        match run.observed.trap {
            None => Ok(text),
            Some(trap) => Err((text, trap)),
        }
    }
}

fn run_ir(
    program: &cpl_ir::Program,
    io: cpl_interp::Io,
    stack_limit: Option<u32>,
    limit: Duration,
) -> Result<String, (String, cpl_interp::Trap)> {
    let io = io.until(cpl_interp::Stop::after(limit));
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

/// How long the traced run that explains a counterexample may take.
const TRACE_LIMIT: Duration = Duration::from_secs(10);

/// The first steps of a run that never finishes: enough to see it go round.
pub const STUCK_STEPS: usize = 100_000;

fn observe_ir(
    program: &cpl_ir::Program,
    input: &[u8],
    stack_limit: Option<u32>,
    config: TraceConfig,
) -> cpl_interp::Observed {
    let mut sink = Vec::new();
    let io = cpl_interp::Io::fixed(input).until(cpl_interp::Stop::after(TRACE_LIMIT));
    match stack_limit {
        Some(bytes) => cpl_interp::observe_on_this_thread(
            program,
            io,
            &mut sink,
            cpl_interp::Limits {
                stack_bytes: bytes as usize,
            },
            Some(config),
        ),
        None => cpl_interp::run_observed(program, io, &mut sink, Some(config)),
    }
}

impl Engine for InterpEngine {
    fn has_brute(&self) -> bool {
        self.brute.is_some()
    }

    fn stops(&self, input: &str) -> bool {
        self.run_case(Role::Fast, &self.fast, input, self.limit)
            .is_err()
    }

    fn limit(&self) -> Duration {
        self.limit
    }

    fn run_within(&self, input: &str, limit: Duration) -> Result<Wrong, StressError> {
        judge_case(self, input, limit)
    }
}

/// How an interpreted run ended, as [`Sides`] takes it.
fn ran(run: Result<String, (String, cpl_interp::Trap)>) -> Ran {
    match run {
        Ok(out) => Ran { out, stop: None },
        Err((out, trap)) => Ran {
            out,
            stop: Some(match trap.message.as_str() {
                cpl_interp::OUT_OF_TIME => Verdict::TimeLimit,
                _ => Verdict::RuntimeError(trap.message),
            }),
        },
    }
}

/// What a judge's trap says of the run: a bug in the judge, or its verdict.
fn ruling(run: Result<String, (String, cpl_interp::Trap)>) -> Ruling {
    match run {
        Ok(_) => Ruling::Accepted,
        Err((_, trap)) if crate::judge::judge_bug(trap.code().0) => Ruling::Broke(trap.message),
        Err((_, trap)) => Ruling::Rejected(Some(trap.message)),
    }
}

impl Sides for InterpEngine {
    fn brute(&self, input: &str, limit: Duration) -> Result<Option<Ran>, Error> {
        let limit = crate::case::brute_limit(limit);
        Ok(self
            .brute
            .as_ref()
            .map(|brute| ran(self.run_case(Role::Brute, brute, input, limit))))
    }

    fn solution(&self, input: &str, limit: Duration) -> Result<Ran, Error> {
        Ok(ran(self.run_case(Role::Fast, &self.fast, input, limit)))
    }

    fn check(
        &self,
        input: &str,
        got: &str,
        _: Option<&str>,
        limit: Duration,
    ) -> Result<Option<Ruling>, Error> {
        Ok(self.checker.as_ref().map(|checker| {
            let mut printed = Some(got.as_bytes().to_vec());
            let io = cpl_interp::Io::judge(input.as_bytes(), Box::new(move || printed.take()));
            ruling(run_ir(
                checker,
                io,
                self.stack_limit,
                crate::case::judge_limit(limit),
            ))
        }))
    }

    fn interact(&self, input: &str, limit: Duration) -> Result<Option<Talked>, Error> {
        let Some(interactor) = &self.interactor else {
            return Ok(None);
        };
        let spoken = crate::duplex::talk(
            &self.fast,
            &interactor.program,
            input.as_bytes(),
            limit,
            self.stack_limit,
        );
        let json = |trap: &cpl_interp::Trap| {
            let diagnostic = trap.to_diagnostic(&interactor.name, &interactor.lines);
            cpl_diag::json::diagnostic(&diagnostic, &interactor.lines)
        };
        let trap = spoken.interactor.as_ref().map(json);
        let solution_trap = spoken.solution.as_ref().err().map(|(_, trap)| json(trap));
        let ruled = spoken
            .interactor
            .map_or(Ok(String::new()), |trap| Err((String::new(), trap)));
        Ok(Some(Talked {
            solution: ran(spoken.solution),
            ruling: ruling(ruled),
            transcript: spoken.transcript,
            trap,
            solution_trap,
        }))
    }
}

/// In-process `@gen` backend using `cpl-interp` with seeded `rand`/`shuffle`.
#[derive(Clone, Debug)]
pub struct InterpChoices {
    pub program: cpl_ir::Program,
    pub name: String,
    /// The lines of the file the fn is in, to say where it traps.
    pub lines: cpl_diag::LineIndex,
    pub stack_limit: Option<u32>,
    /// How long one draw may take before it is stopped.
    pub limit: Duration,
}

impl InterpChoices {
    /// One run of the generator, and the choices it made.
    fn run(&self, io: cpl_interp::Io, replay: Option<Vec<u64>>) -> Result<Drawn, cpl_interp::Trap> {
        let choices = std::sync::Arc::new(std::sync::Mutex::new(cpl_interp::Choices {
            replay,
            made: Vec::new(),
        }));
        let io = io.choosing(std::sync::Arc::clone(&choices));
        let input = run_ir(&self.program, io, self.stack_limit, self.limit).map_err(|(_, t)| t)?;
        let made = std::mem::take(&mut choices.lock().unwrap().made);
        let (sites, choices) = made.into_iter().unzip();
        Ok(Drawn {
            choices,
            sites,
            input,
        })
    }
}

impl ChoiceSource for InterpChoices {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError> {
        self.run(cpl_interp::Io::fixed(b"").seeded(seed), None)
            .map_err(|trap| {
                StressError(format!(
                    "the `@gen` fn of {} failed on seed {seed} at line {}: {}",
                    self.name,
                    self.lines.line_of(trap.span.start) + 1,
                    trap.message
                ))
            })
    }

    fn replay(&self, choices: &[u64]) -> Option<Drawn> {
        self.run(cpl_interp::Io::fixed(b""), Some(choices.to_vec()))
            .ok()
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
) -> Result<StressResult, StressError> {
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

    let fast = cpl_ir::lower_sampling_laws(&analysis).map_err(|diags| {
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
        let bname = format!(".{name}.brute.cp");
        db.set_file(&bname, swapped.as_str());
        let b_analysis = db
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
        Some((checker_fn, _)) => Some(cpl_ir::lower_checker(&analysis, &checker_fn).map_err(
            |diags| {
                let lines = cpl_diag::LineIndex::new(source);
                let text: String = diags
                    .iter()
                    .map(|d| cpl_diag::render(d, name, source, &lines))
                    .collect();
                StressError(text)
            },
        )?),
        None => None,
    };

    let interactor = match cpl_ir::interactor(&analysis) {
        Some(judge) => Some(FileJudge {
            program: cpl_ir::lower_judge(&analysis, &judge).map_err(|diags| {
                let lines = cpl_diag::LineIndex::new(source);
                let text: String = diags
                    .iter()
                    .map(|d| cpl_diag::render(d, name, source, &lines))
                    .collect();
                StressError(text)
            })?,
            name: name.to_string(),
            lines: cpl_diag::LineIndex::new(source),
        }),
        None => None,
    };

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
                lines: cpl_diag::LineIndex::new(source),
                stack_limit,
                limit: options.limit,
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
        interactor,
        stack_limit,
        limit: options.limit,
        id: ENGINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    };
    let mut single = options.clone();
    single.jobs = 1;
    let report = stress_with(&input_source, &engine, &single)?;

    let config = |assignments: bool| TraceConfig {
        source: source.to_string(),
        assignments,
        ..TraceConfig::default()
    };
    let mut stages = Vec::new();
    let mut explained = None;
    let timed_out = |failure: &crate::run::Failure| matches!(failure.verdict, Verdict::TimeLimit);
    if let (Some(failure), true) = (&report.failure, explain) {
        if timed_out(failure) {
            let rec = observe_ir(
                &engine.fast,
                failure.input.as_bytes(),
                stack_limit,
                TraceConfig {
                    max_steps: Some(STUCK_STEPS),
                    stop_when_full: true,
                    record_frames: true,
                    ..config(false)
                },
            )
            .recording
            .unwrap_or_default();
            explained = Some(stuck_divergence(source, &rec).map(Some).ok_or(format!(
                "the solution never finishes, and none of its first {STUCK_STEPS} steps repeats an earlier one"
            )));
        }
    }
    if let (Some(failure), Some(brute_prog)) = (&report.failure, &engine.brute) {
        let brute_failed = matches!(failure.verdict, Verdict::BruteError(_));
        if own_brute.is_some() && !timed_out(failure) && !brute_failed {
            let fast_rec = observe_ir(
                &engine.fast,
                failure.input.as_bytes(),
                stack_limit,
                config(explain),
            )
            .recording
            .unwrap_or_default();
            let slow_rec = observe_ir(
                brute_prog,
                failure.input.as_bytes(),
                stack_limit,
                config(explain),
            )
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
                explained = Some(Ok(explain_divergence(source, &fast_rec, &slow_rec)));
            }
        }
    }

    let error = report.failure.as_ref().and_then(|failure| {
        if failure.trap.is_some() || failure.transcript.is_some() {
            return failure.trap.clone();
        }
        let path = std::path::Path::new(name);
        let brute = match failure.verdict {
            Verdict::RuntimeError(_) => false,
            Verdict::BruteError(_) if own_brute.is_some() => true,
            Verdict::JudgeError("checker", _) => {
                return crate::explain::checker_trap_in(
                    path,
                    source,
                    &failure.input,
                    &failure.got,
                    stack_limit,
                )
                .map(|t| t.json());
            }
            _ => return None,
        };
        crate::explain::trap_report_in(path, source, &failure.input, brute, stack_limit)
            .map(|t| t.json())
    });
    Ok(StressResult {
        stages,
        explained,
        trap: error,
        ..StressResult::new(report)
    })
}
