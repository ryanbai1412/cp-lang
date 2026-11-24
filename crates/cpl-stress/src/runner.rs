//! How a program a case needs runs: in the VM, in this process, or as a
//! process of its own. The one stress front end, [`crate::stress`], takes a
//! [`Runner`] for each role (solution, brute, generator, checker,
//! interactor); the browser and the file's `.cp` helpers run in the VM, and
//! the CLI's built C++, `.cpp`, `.py` and executables run as processes.

use crate::case::{Ran, Ruling, Talked};
use crate::choices::Drawn;
use crate::judge::Launch;
use crate::program::Program;
use crate::run::StressError;
use std::time::Duration;

/// A lowered program run in the VM, and the file it is from, to say where
/// it stops.
#[derive(Clone, Debug)]
pub struct Vm {
    pub program: cpl_ir::Program,
    pub name: String,
    pub lines: cpl_diag::LineIndex,
    /// Stack byte limit when running on the calling thread (wasm).
    pub stack_limit: Option<u32>,
    /// Tells this program's [`PRELUDES`] from another's.
    id: u64,
}

/// How each program of a case runs.
pub enum Runner {
    InProcess(Box<Vm>),
    Process(Program),
    /// A run made already: what it printed, such as a test's answer.
    Answer(String),
    /// A run decided by the test itself.
    #[cfg(test)]
    Canned(Canned),
}

/// A test's stand-in for running a program.
#[cfg(test)]
pub type RunFn = dyn Fn(&str, Duration) -> Ran + Sync;

/// What a test's runner gives back.
#[cfg(test)]
pub enum Canned {
    Runs(Box<RunFn>),
    Rules(Ruling),
    Talks(Box<Talked>),
}

/// How many programs' preludes are kept at once.
const KEPT_PRELUDES: usize = 8;

/// Loop rounds and calls between the checkpoints a first run takes before
/// it reads: a precompute shorter than this is just run again.
const PRELUDE_EVERY: u64 = 1 << 16;

thread_local! {
    /// Per program run in the VM, once a run has shown it: the last checkpoint
    /// before the program reads or prints, or `None` when there is none.
    /// Every later case starts there, so what the program works out before
    /// it reads (a table over all `k <= 60`, say) runs once, not per case.
    static PRELUDES: std::cell::RefCell<Vec<PreludeOf>> = const { std::cell::RefCell::new(Vec::new()) };
}

type PreludeOf = (u64, Option<cpl_interp::checkpoint::Checkpoint>);

/// An interpreted run's output, or its output up to the trap; the trap is
/// boxed, as it is far larger than the output.
type Interpreted = Result<String, (String, Box<cpl_interp::Trap>)>;

static VMS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Vm {
    /// [`run_ir`] on the calling thread, from the state `program` reaches
    /// before it first reads, once a first run has found it.
    fn run_resumed(&self, input: &str, limit: Duration) -> Interpreted {
        use cpl_interp::checkpoint::Checkpoints;
        let io = cpl_interp::Io::fixed(input.as_bytes());
        let Some(bytes) = self.stack_limit else {
            return run_ir(&self.program, io, None, limit);
        };
        let key = self.id;
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
            &self.program,
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
                p.retain(|(id, _)| *id != self.id);
                if p.len() >= KEPT_PRELUDES {
                    p.remove(0);
                }
                p.push((key, prelude));
            });
        }
        let text = String::from_utf8_lossy(&out).into_owned();
        match run.observed.trap {
            None => Ok(text),
            Some(trap) => Err((text, Box::new(trap))),
        }
    }
}

fn run_ir(
    program: &cpl_ir::Program,
    io: cpl_interp::Io,
    stack_limit: Option<u32>,
    limit: Duration,
) -> Interpreted {
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
        Err(trap) => Err((text, Box::new(trap))),
    }
}

/// How an interpreted run ended.
fn ran(run: Interpreted) -> Ran {
    match run {
        Ok(out) => Ran {
            out,
            stop: None,
            cost: None,
        },
        Err((out, trap)) => Ran {
            out,
            stop: Some(crate::judge::interp_failure(*trap)),
            cost: None,
        },
    }
}

/// What a judge's trap says of the run: a bug in the judge, or its verdict.
fn ruling(run: Interpreted) -> Ruling {
    match run {
        Ok(_) => Ruling::Accepted,
        Err((_, trap)) if crate::judge::judge_bug(trap.code().0) => Ruling::Broke(trap.message),
        Err((_, trap)) => Ruling::Rejected(Some(trap.message)),
    }
}

impl Vm {
    pub fn new(program: cpl_ir::Program, name: &str, source: &str, stack_limit: Option<u32>) -> Vm {
        Vm {
            program,
            name: name.to_string(),
            lines: cpl_diag::LineIndex::new(source),
            stack_limit,
            id: VMS.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// The trap as `cpl check --format json` gives a diagnostic.
    fn json(&self, trap: &cpl_interp::Trap) -> cpl_json::Value {
        let diagnostic = trap.to_diagnostic(&self.name, &self.lines);
        cpl_diag::json::diagnostic(&diagnostic, &self.lines)
    }

    /// One draw of a `@gen` fn, and the choices it made.
    fn choose(
        &self,
        io: cpl_interp::Io,
        replay: Option<Vec<u64>>,
        limit: Duration,
    ) -> Result<Drawn, cpl_interp::Trap> {
        let choices = std::sync::Arc::new(std::sync::Mutex::new(cpl_interp::Choices {
            replay,
            made: Vec::new(),
        }));
        let io = io.choosing(std::sync::Arc::clone(&choices));
        let input = run_ir(&self.program, io, self.stack_limit, limit).map_err(|(_, t)| *t)?;
        let made = std::mem::take(&mut choices.lock().unwrap().made);
        let (sites, choices) = made.into_iter().unzip();
        Ok(Drawn {
            choices,
            sites,
            input,
        })
    }
}

/// Why `program` could not be run at all.
fn unrunnable(name: &str) -> impl Fn(std::io::Error) -> StressError + '_ {
    move |e| StressError(format!("can't run {name}: {e}"))
}

impl Runner {
    pub fn name(&self) -> &str {
        match self {
            Runner::InProcess(vm) => &vm.name,
            Runner::Process(program) => &program.name,
            Runner::Answer(_) => "the answer",
            #[cfg(test)]
            Runner::Canned(_) => "a canned run",
        }
    }

    fn cannot(&self, what: &str) -> StressError {
        StressError(format!("{} can't {what}", self.name()))
    }

    /// The program run on `input`, stopped after `limit`, with what the
    /// run cost when it ran as a process.
    pub fn run(&self, input: &str, limit: Duration) -> Result<Ran, StressError> {
        Ok(match self {
            Runner::InProcess(vm) => ran(vm.run_resumed(input, limit)),
            Runner::Process(program) => {
                (crate::judge::run(program, input, limit)).map_err(unrunnable(&program.name))?
            }
            Runner::Answer(out) => Ran {
                out: out.clone(),
                stop: None,
                cost: None,
            },
            #[cfg(test)]
            Runner::Canned(Canned::Runs(run)) => run(input, limit),
            #[cfg(test)]
            Runner::Canned(_) => return Err(self.cannot("run")),
        })
    }

    /// The program as a checker's ruling on `got`, the output for `input`.
    pub fn check(
        &self,
        input: &str,
        got: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> Result<Ruling, StressError> {
        match self {
            Runner::InProcess(vm) => {
                let mut printed = Some(got.as_bytes().to_vec());
                let io = cpl_interp::Io::judge(input.as_bytes(), Box::new(move || printed.take()));
                let limit = crate::case::judge_limit(limit);
                Ok(ruling(run_ir(&vm.program, io, vm.stack_limit, limit)))
            }
            Runner::Process(checker) => crate::judge::check(checker, input, got, expected, limit)
                .map_err(unrunnable(&checker.name)),
            #[cfg(test)]
            Runner::Canned(Canned::Rules(ruling)) => Ok(ruling.clone()),
            _ => Err(self.cannot("check an answer")),
        }
    }

    /// The program as an interactor, talking to `solution` on the hidden
    /// test `input`.
    pub fn talk(
        &self,
        solution: &Runner,
        launch: Launch,
        input: &str,
        limit: Duration,
    ) -> Result<Talked, StressError> {
        match (self, solution) {
            (Runner::InProcess(interactor), Runner::InProcess(fast)) => {
                let stack = interactor.stack_limit;
                let spoken = crate::duplex::talk(
                    &fast.program,
                    &interactor.program,
                    input.as_bytes(),
                    limit,
                    stack,
                );
                let trap = spoken.interactor.as_ref().map(|t| interactor.json(t));
                let solution_trap =
                    (spoken.solution.as_ref().err()).map(|(_, t)| interactor.json(t));
                let ruled = (spoken.interactor)
                    .map_or(Ok(String::new()), |t| Err((String::new(), Box::new(t))));
                Ok(Talked {
                    solution: ran(spoken.solution.map_err(|(out, t)| (out, Box::new(t)))),
                    ruling: ruling(ruled),
                    transcript: spoken.transcript,
                    trap,
                    solution_trap,
                })
            }
            (Runner::InProcess(interactor), Runner::Process(fast)) => {
                let (outcome, transcript, trap) =
                    crate::judge::interact_in_vm(fast, &interactor.program, input, limit)
                        .map_err(unrunnable(&fast.name))?;
                let json = trap.as_ref().map(|t| interactor.json(t));
                let ruled = trap.map_or(Ok(String::new()), |t| Err((String::new(), Box::new(t))));
                Ok(Talked {
                    solution: Ran {
                        out: outcome.stdout.clone(),
                        stop: crate::judge::failure(&outcome),
                        cost: Some(outcome),
                    },
                    ruling: ruling(ruled),
                    transcript,
                    trap: json,
                    solution_trap: None,
                })
            }
            (Runner::Process(interactor), Runner::Process(fast)) => {
                crate::judge::interact(fast, interactor, launch, input, limit)
                    .map(|(talked, _)| talked)
                    .map_err(unrunnable(&interactor.name))
            }
            #[cfg(test)]
            (Runner::Canned(Canned::Talks(talked)), _) => Ok((**talked).clone()),
            _ => Err(StressError(format!(
                "{} and {} run in different places, so they can't talk",
                self.name(),
                solution.name()
            ))),
        }
    }

    /// Where an interactor's run on `input` against `solution` trapped,
    /// from a second run that asks it to say.
    pub fn trap(&self, solution: &Runner, input: &str, limit: Duration) -> Option<cpl_json::Value> {
        let (Runner::Process(interactor), Runner::Process(fast)) = (self, solution) else {
            return None;
        };
        let json = crate::judge::interactor_trap_json(fast, interactor, input, limit)?;
        cpl_json::parse(&json).ok()
    }

    /// The input a generator run as `gen <seed> <size>` prints.
    pub fn generate(&self, seed: u64, size: &str) -> Result<String, StressError> {
        let Runner::Process(generator) = self else {
            return Err(self.cannot("generate from a seed and a size"));
        };
        let outcome = generator
            .with_args(&[&seed.to_string(), size])
            .run("", Duration::from_secs(10))
            .map_err(unrunnable(&generator.name))?;
        if outcome.ok() {
            return Ok(outcome.stdout);
        }
        let said = outcome.stderr.trim_end();
        let said: Vec<&str> = said.rsplit('\n').take(5).collect();
        let said: Vec<&str> = said.into_iter().rev().collect();
        Err(StressError(format!(
            "the generator {} failed on seed {seed} ({:?}){}",
            generator.name,
            outcome.exit,
            match said.is_empty() || said == [""] {
                true => String::new(),
                false => format!(":\n{}", said.join("\n")),
            }
        )))
    }

    /// The case a `@gen` fn draws from `seed`.
    pub fn draw(&self, seed: u64, limit: Duration) -> Result<Drawn, StressError> {
        match self {
            Runner::InProcess(vm) => {
                (vm.choose(cpl_interp::Io::fixed(b"").seeded(seed), None, limit)).map_err(|trap| {
                    StressError(format!(
                        "the `@gen` fn of {} failed on seed {seed} at line {}: {}",
                        vm.name,
                        vm.lines.line_of(trap.span.start) + 1,
                        trap.message
                    ))
                })
            }
            _ => Err(self.cannot("draw an input")),
        }
    }

    /// The case `choices` give a `@gen` fn, or `None` when it rejects them.
    pub fn replay(&self, choices: &[u64], limit: Duration) -> Option<Drawn> {
        match self {
            Runner::InProcess(vm) => vm
                .choose(cpl_interp::Io::fixed(b""), Some(choices.to_vec()), limit)
                .ok(),
            _ => None,
        }
    }
}
