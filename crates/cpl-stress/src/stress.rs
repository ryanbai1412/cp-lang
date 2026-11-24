//! The one stress front end. [`Stress::build`] reads the file once and
//! picks a [`Runner`] for each role it needs (solution, brute, generator,
//! checker, interactor, `@cache` cold build); [`Stress::run`] is the case
//! loop, the choice of what to explain, the retrace and the trap, for the
//! browser and `cpl stress` alike. Where they differ is only where their
//! programs run: in the VM, or built and run as processes.

use crate::case::Roles;
use crate::choices::{ChoiceSource, Drawn};
use crate::cli::{swap_in_part_brutes, Traced, Tracer};
use crate::judge::{explaining, Judge, Trapped};
use crate::program::Program;
use crate::report::StressResult;
use crate::run::{stress_with, InputSource, Options, StressError};
use crate::runner::{Runner, Vm};
use crate::spec::InputSpec;
use cpl_diag::Severity;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the file's own programs run.
pub enum Place<'a> {
    /// In the VM, in this process: the browser, and tests.
    InProcess { stack_limit: Option<u32> },
    /// Built and run as processes, with the programs the command line names.
    Native(Native<'a>),
}

/// What `cpl stress` names in place of the file's own programs.
pub struct Native<'a> {
    /// `--fast`: the solution run in place of the file.
    pub fast: Option<&'a Path>,
    pub brute: Option<&'a Path>,
    pub gen: Option<&'a Path>,
    /// The judge `--checker`, `--interactor` or the file's own picks.
    pub judge: Judge,
    /// `--checker` or `--interactor` names the judge.
    pub judge_given: bool,
}

/// Where a case's input comes from.
pub enum Inputs {
    /// Made from the program's own `read()` types, and shrinkable.
    Typed(InputSpec),
    /// A generator run as `gen <seed> <size>`; its output is the input.
    Generator(Runner),
    /// The file's own `@gen` fn. It shrinks through the choices its `rand`
    /// and `shuffle` made.
    Gen(Drawing),
}

/// A `@gen` fn, and the same fn in the VM, which says on which line a
/// draw that fails traps.
pub struct Drawing {
    pub gen: Runner,
    pub limit: Duration,
}

impl ChoiceSource for Drawing {
    fn draw(&self, seed: u64) -> Result<Drawn, StressError> {
        self.gen.draw(seed, self.limit)
    }

    fn replay(&self, choices: &[u64]) -> Option<Drawn> {
        self.gen.replay(choices, self.limit)
    }
}

/// Why a file can't be stress tested: its diagnostics, rendered, or a
/// message.
#[derive(Debug)]
pub enum Unbuilt {
    Rendered(String),
    Said(String),
}

impl From<Unbuilt> for StressError {
    fn from(unbuilt: Unbuilt) -> StressError {
        match unbuilt {
            Unbuilt::Rendered(text) | Unbuilt::Said(text) => StressError(text),
        }
    }
}

/// A file ready to stress: its runners, and what it judges by.
pub struct Stress {
    name: String,
    path: PathBuf,
    source: String,
    pub roles: Roles,
    pub inputs: Inputs,
    /// The file's own `@alt(program)` fn.
    pub own_brute: bool,
    /// The file with each `@alt(f)` fn in `f`'s place.
    pub part_brute: Option<String>,
    /// The solution is the `.cp` file itself, which retraces and traps
    /// run in the debug VM.
    pub file_solves: bool,
    /// The brute or the judge is the file's own.
    in_file: bool,
    in_process: bool,
    stack_limit: Option<u32>,
}

/// The file's lowered programs, or its diagnostics.
fn lowered(
    lowered: Result<cpl_ir::Program, Vec<cpl_diag::Diagnostic>>,
    name: &str,
    source: &str,
) -> Result<cpl_ir::Program, Unbuilt> {
    lowered.map_err(|diagnostics| {
        let lines = cpl_diag::LineIndex::new(source);
        Unbuilt::Rendered(
            (diagnostics.iter())
                .map(|d| cpl_diag::render(d, name, source, &lines))
                .collect(),
        )
    })
}

impl Stress {
    /// Reads `source`, the file `path` shown as `name`, and picks each
    /// role's runner for `place`.
    pub fn build(
        path: &Path,
        name: &str,
        source: &str,
        place: Place,
        options: &Options,
    ) -> Result<Stress, Unbuilt> {
        let is_cp = path.extension().is_some_and(|e| e == "cp");
        let native = match &place {
            Place::Native(native) => Some(native),
            Place::InProcess { .. } => None,
        };
        let stack_limit = match place {
            Place::InProcess { stack_limit } => stack_limit,
            Place::Native(_) => None,
        };
        let given_brute = native.and_then(|n| n.brute);
        let given_gen = native.and_then(|n| n.gen);
        let mut db = cpl_check::Session::new();
        db.set_file(name, source);
        let analysis = match is_cp || native.is_none() {
            true => db.analyze(name),
            false => None,
        };
        if native.is_none() {
            let analysis = (analysis.as_deref())
                .ok_or_else(|| Unbuilt::Said(format!("could not analyze {name}")))?;
            let diagnostics = cpl_ir::check(analysis);
            if diagnostics.iter().any(|d| d.severity == Severity::Error) {
                let lines = cpl_diag::LineIndex::new(source);
                return Err(Unbuilt::Rendered(
                    (diagnostics.iter())
                        .map(|d| cpl_diag::render(d, name, source, &lines))
                        .collect(),
                ));
            }
        }
        let analysis = analysis.as_deref();
        let analyzed =
            || analysis.ok_or_else(|| Unbuilt::Said(format!("could not analyze {name}")));
        let vm = |program: cpl_ir::Program| {
            Runner::InProcess(Box::new(Vm::new(program, name, source, stack_limit)))
        };
        // The file's own `@alt(program)`, unless `--brute` names another.
        let own_brute = match given_brute {
            None => analysis.and_then(cpl_ir::brute),
            Some(_) => None,
        };
        // `@alt(f) fn g`: the brute is the whole file with `g` in `f`'s place.
        let part_brute = match (given_brute, &own_brute) {
            (None, None) if is_cp || native.is_none() => swap_in_part_brutes(source),
            _ => None,
        };
        // The file's own judge has to build before it can rule on a case.
        let external_judge = native.is_some_and(|n| n.judge_given);
        let (mut checker, mut interactor) = (None, None);
        if let (Some(analysis), false) = (analysis, external_judge) {
            if let Some((checker_fn, _)) = cpl_ir::checker(analysis) {
                checker = Some(lowered(
                    cpl_ir::lower_checker(analysis, &checker_fn),
                    name,
                    source,
                )?);
            } else if let Some(judge) = cpl_ir::interactor(analysis) {
                interactor = Some(lowered(
                    cpl_ir::lower_judge(analysis, &judge),
                    name,
                    source,
                )?);
            }
        }
        let judged_in_file = checker.is_some() || interactor.is_some();
        let own_gen = match given_gen {
            None => analysis.and_then(cpl_ir::generator),
            Some(_) => None,
        };
        let size = i64::try_from(options.size).unwrap_or(i64::MAX);
        let typed = || {
            match InputSpec::from_source(source) {
            Ok(spec) if spec.is_empty() => Err(Unbuilt::Said(match native {
                Some(_) => format!("{name} has no typed `read()` to make inputs from; write `let n: 1..=100 = read()`, a `@gen fn` that prints an input, or pass `--gen <program>`"),
                None => format!("{name} has no typed `read()` to make inputs from; write `let n: 1..=100 = read()` or a `@gen fn`"),
            })),
            Ok(spec) => Ok(Inputs::Typed(spec)),
            Err(error) => {
                let line = cpl_diag::LineIndex::new(source).line_of(error.span.start) + 1;
                Err(Unbuilt::Said(format!("{name}:{line}: {}", error.message)))
            }
        }
        };
        let said = |e: crate::program::BuildError| Unbuilt::Said(e.to_string());
        let roles = match native {
            None => {
                let analysis = analyzed()?;
                let fast = lowered(cpl_ir::lower_sampling_laws(analysis), name, source)?;
                let brute = match (&own_brute, &part_brute) {
                    (Some((brute_fn, _)), _) => Some(lowered(
                        cpl_ir::lower_brute(analysis, brute_fn),
                        name,
                        source,
                    )?),
                    (None, Some(swapped)) => {
                        let bname = format!(".{name}.brute.cp");
                        db.set_file(&bname, swapped.as_str());
                        let swapped_analysis = (db.analyze(&bname)).ok_or_else(|| {
                            Unbuilt::Said(format!("could not analyze part brute of {name}"))
                        })?;
                        Some(lowered(cpl_ir::lower(&swapped_analysis), &bname, swapped)?)
                    }
                    (None, None) => None,
                };
                Roles {
                    solution: vm(fast),
                    brute: brute.map(vm),
                    checker: checker.map(vm),
                    interactor: interactor
                        .map(|program| (vm(program), crate::judge::Launch::Stdio)),
                    cold: None,
                    limit: options.limit,
                }
            }
            Some(native) => {
                let resolve = |path: &Path| Program::resolve(path).map_err(said);
                let file_solves = native.fast.is_none() && is_cp;
                let cached = file_solves
                    && source.contains("@cache")
                    && analysis.is_some_and(cpl_ir::has_cached_fields);
                // The builds are the whole wait before the first case, so
                // they run side by side.
                let (fast, brute, cold) = std::thread::scope(|scope| {
                    let brute = scope.spawn(|| match (native.brute, &own_brute) {
                        (Some(path), _) => resolve(path).map(Some),
                        (None, Some((fn_name, _))) => Program::resolve_brute(path, fn_name)
                            .map(Some)
                            .map_err(said),
                        (None, None) => match &part_brute {
                            Some(swapped) => crate::cli::build_part_brute(path, name, swapped)
                                .map(Some)
                                .map_err(Unbuilt::Said),
                            None => Ok(None),
                        },
                    });
                    let cold = scope.spawn(|| match cached {
                        true => Program::resolve_cold_cache(path).map(Some).map_err(said),
                        false => Ok(None),
                    });
                    let fast = crate::cli::solution(path, native.fast).map_err(Unbuilt::Said);
                    (
                        fast,
                        brute.join().expect("a build does not panic"),
                        cold.join().expect("a build does not panic"),
                    )
                });
                let process = |program: &Program| Runner::Process(program.clone());
                Roles {
                    solution: Runner::Process(fast?),
                    brute: brute?.map(Runner::Process),
                    checker: match &native.judge {
                        Judge::Checker(program) => Some(process(program)),
                        _ => None,
                    },
                    interactor: match &native.judge {
                        Judge::Interactor(program, launch) => Some((process(program), *launch)),
                        _ => None,
                    },
                    cold: cold?.map(Runner::Process),
                    limit: options.limit,
                }
            }
        };
        let inputs = match (given_gen, &own_gen, native) {
            (Some(gen), ..) => {
                Inputs::Generator(Runner::Process(Program::resolve(gen).map_err(said)?))
            }
            (None, Some((gen_fn, _)), None) => {
                let analysis = analyzed()?;
                let program = lowered(cpl_ir::lower_gen(analysis, gen_fn, size), name, source)?;
                Inputs::Gen(Drawing {
                    gen: vm(program),
                    limit: options.limit,
                })
            }
            // Its choices are recorded and replayed in the VM, so a native
            // stress draws there too.
            (None, Some((fn_name, _)), Some(_)) => Inputs::Gen(Drawing {
                gen: Runner::InProcess(Box::new(Vm::new(
                    Program::lowered_gen(path, size).map_err(said)?,
                    &format!("{name} (`{fn_name}`)"),
                    source,
                    stack_limit,
                ))),
                limit: options.limit,
            }),
            (None, None, _) => typed()?,
        };
        let file_solves = is_cp && native.is_none_or(|n| n.fast.is_none());
        let in_file = own_brute.is_some()
            || part_brute.is_some()
            || judged_in_file
            || native.is_some_and(|n| n.brute.is_none() && n.judge != Judge::Tokens);
        Ok(Stress {
            name: name.to_string(),
            path: path.to_path_buf(),
            source: source.to_string(),
            roles,
            inputs,
            own_brute: own_brute.is_some(),
            part_brute,
            file_solves: file_solves || native.is_none(),
            in_file,
            in_process: native.is_none(),
            stack_limit,
        })
    }

    /// Stresses the file: the cases, and for the first that fails, what
    /// explains it and where a helper or the solution trapped. `tracer`
    /// renders the retraced failing case as a page.
    pub fn run(
        &self,
        options: &Options,
        explain: bool,
        tracer: Option<Tracer>,
    ) -> Result<Stressed, StressError> {
        let mut options = options.clone();
        if self.in_process {
            options.jobs = 1;
        }
        let source = match &self.inputs {
            Inputs::Typed(spec) => InputSource::Typed(spec),
            Inputs::Generator(generator) => InputSource::Generator(generator),
            Inputs::Gen(drawing) => InputSource::Choices(drawing),
        };
        let report = stress_with(&source, &self.roles, &options)?;
        let failure = report
            .failure
            .as_ref()
            .filter(|f| !f.verdict.helper_failed());
        let interactive = self.roles.interactor.is_some();
        let brute = self.own_brute || self.part_brute.is_some();
        let how = failure.and_then(|f| explaining(&f.verdict, interactive, brute));
        let traced = match failure {
            Some(failure)
                if self.file_solves
                    && self.in_file
                    && how.is_some()
                    && (explain || !failure.verdict.timed_out()) =>
            {
                Some(crate::traced::trace(
                    &self.name,
                    &self.source,
                    &failure.input,
                    explain,
                    how.as_ref(),
                    self.stack_limit,
                    tracer,
                ))
            }
            _ => None,
        };
        let explained = match (failure, &traced, how) {
            (None, ..) | (Some(_), _, None) => None,
            // A part's wrong call is shown without `--explain`: calls are always kept.
            (Some(_), Some(Ok(traced)), _)
                if self.part_brute.is_some() && traced.divergence.is_some() =>
            {
                Some(Ok(traced.divergence.clone()))
            }
            _ if !explain => None,
            (Some(_), Some(Ok(traced)), Some(Ok(_))) => Some(Ok(traced.divergence.clone())),
            (Some(_), Some(Err(why)), Some(Ok(_))) => Some(Err(why.clone())),
            (Some(_), _, Some(Err(why))) => Some(Err(why)),
            (Some(_), None, Some(Ok(_))) => Some(Err(
                "`--explain` runs the file against its own `@alt` fns".to_string(),
            )),
        };
        let stages = match &traced {
            Some(Ok(traced)) => traced.stages.clone(),
            _ => Vec::new(),
        };
        let trap = (report.failure.as_ref())
            .filter(|_| self.file_solves)
            .and_then(|failure| self.trap(failure));
        Ok(Stressed {
            result: StressResult {
                stages,
                explained,
                trap,
                ..StressResult::new(report)
            },
            traced,
        })
    }

    /// Where the program a failing case trapped in stopped, in the shape
    /// `cpl check --format json` gives.
    fn trap(&self, failure: &crate::run::Failure) -> Option<cpl_json::Value> {
        if failure.trap.is_some() {
            return failure.trap.clone();
        }
        let (path, source, input) = (self.path.as_path(), self.source.as_str(), &failure.input);
        let stack = self.stack_limit;
        let trapped = match failure.verdict.trapped()? {
            Trapped::Interactor => {
                let (interactor, _) = self.roles.interactor.as_ref()?;
                return interactor.trap(&self.roles.solution, input, self.roles.limit);
            }
            _ if failure.transcript.is_some() => return None,
            Trapped::Solution => crate::explain::trap_report_in(path, source, input, false, stack),
            Trapped::Brute => self.brute_trap(input),
            Trapped::Checker => {
                crate::explain::checker_trap_in(path, source, input, &failure.got, stack)
            }
        };
        trapped.map(|t| t.json())
    }
}

impl Stress {
    /// Where a debug run of the file's brute stops on `input`: its own
    /// `@alt(program)` fn, or the file with its `@alt(f)` fns swapped in.
    pub fn brute_trap(&self, input: &str) -> Option<crate::explain::Trapped> {
        let (path, stack) = (self.path.as_path(), self.stack_limit);
        match (self.own_brute, &self.part_brute) {
            _ if !self.file_solves => None,
            (true, _) => crate::explain::trap_report_in(path, &self.source, input, true, stack),
            (false, Some(swapped)) => {
                crate::explain::trap_report_in(path, swapped, input, false, stack)
            }
            (false, None) => None,
        }
    }
}

/// A stress run: its result, and the failing case's trace when one ran.
pub struct Stressed {
    pub result: StressResult,
    pub traced: Option<Result<Traced, String>>,
}
