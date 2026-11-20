//! The differential check: one generated program through every path of
//! a toolchain, against the reference evaluator.
//!
//! A path passes a program with a defined output when it exits 0, prints
//! exactly what the evaluator printed, and no sanitizer reports anything.
//! A path passes a trapping program when it stops with a nonzero exit
//! code (not a crash, not a sanitizer report) after printing no more
//! than the evaluator had printed by then.

use crate::backend::{sanitizer_report, scratch_dir, Cxx, Interpreter, Transpiler};
use crate::fuzz::eval::Trap;
use crate::fuzz::gen::{self, Case};
use crate::fuzz::rng::Rng;
use crate::fuzz::{ast::Program, eval, print, shrink};
use crate::process::{self, Exit, Output};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub const RUN_LIMIT: Duration = Duration::from_secs(20);

/// What is under test.
pub struct Toolchain {
    /// `.cp` to C++17, compiled by `cxx` and run.
    pub transpiler: Box<dyn Transpiler>,
    pub cxx: Cxx,
    /// Runs `.cp` directly. `None` until `cpl-interp` can run programs.
    pub interpreter: Option<Box<dyn Interpreter>>,
    /// The runtime errors this toolchain promises to stop on cleanly.
    /// Trapping programs are only generated for these.
    pub traps: Vec<Trap>,
    /// Diagnostics that mark a refusal the language allows: the checker
    /// may turn down a program whose arithmetic it cannot prove in range,
    /// even in code that never runs. A transpiler error containing one of
    /// these is counted as refused, not as a failure.
    pub static_refusals: Vec<String>,
}

impl Toolchain {
    fn allows_refusal(&self, finding: &Finding) -> bool {
        finding.kind == FindingKind::Rejected
            && self
                .static_refusals
                .iter()
                .any(|m| finding.detail.contains(m.as_str()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// The transpiler refused a valid program.
    Rejected,
    /// The emitted C++ does not compile.
    BadCxx,
    WrongOutput,
    Sanitizer,
    /// Nonzero exit or a signal on a program with a defined output.
    Crash,
    Timeout,
    /// A program that must stop with an error ran to the end.
    MissedTrap,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    /// Which path failed: the transpiler's or interpreter's name.
    pub path: String,
    pub detail: String,
}

/// What the program must do.
#[derive(Debug, Clone)]
pub enum Expect {
    Output(String),
    /// Stops with an error after printing a prefix of this.
    Trap(String),
}

fn expectation(program: &Program) -> Result<Expect, Trap> {
    match eval::run(program) {
        Ok(out) => Ok(Expect::Output(out)),
        Err(f) => match f.trap {
            Trap::Overflow | Trap::DivByZero | Trap::IndexOutOfRange => Ok(Expect::Trap(f.printed)),
            other => Err(other),
        },
    }
}

fn judge(path: &str, out: &Output, expect: &Expect) -> Option<Finding> {
    let finding = |kind, detail: String| {
        Some(Finding {
            kind,
            path: path.to_string(),
            detail,
        })
    };
    if let Some(line) = sanitizer_report(&out.stderr) {
        return finding(FindingKind::Sanitizer, line.to_string());
    }
    if out.exit == Exit::TimedOut {
        return finding(FindingKind::Timeout, format!("no exit after {RUN_LIMIT:?}"));
    }
    match expect {
        Expect::Output(want) => {
            if out.exit != Exit::Code(0) {
                let stderr = out.stderr.lines().next().unwrap_or("");
                return finding(FindingKind::Crash, format!("{:?}: {stderr}", out.exit));
            }
            if out.stdout != *want {
                return finding(
                    FindingKind::WrongOutput,
                    first_difference(want, &out.stdout),
                );
            }
        }
        Expect::Trap(printed) => match out.exit {
            Exit::Code(0) => {
                return finding(FindingKind::MissedTrap, "exit code 0".into());
            }
            Exit::Code(_) if printed.starts_with(&out.stdout) => {}
            Exit::Code(_) => {
                return finding(
                    FindingKind::WrongOutput,
                    first_difference(printed, &out.stdout),
                );
            }
            Exit::Signal | Exit::TimedOut => {
                return finding(FindingKind::Crash, "killed by a signal".into());
            }
        },
    }
    None
}

fn first_difference(want: &str, got: &str) -> String {
    let (mut w, mut g) = (want.lines(), got.lines());
    let mut line = 1;
    loop {
        match (w.next(), g.next()) {
            (Some(a), Some(b)) if a == b => line += 1,
            (a, b) => {
                let show =
                    |s: Option<&str>| s.map_or("<end of output>".to_string(), |s| format!("`{s}`"));
                return format!("line {line}: expected {}, got {}", show(a), show(b));
            }
        }
    }
}

/// Runs one program through every path. `work` is a scratch directory.
pub fn check_program(
    tc: &Toolchain,
    program: &Program,
    expect: &Expect,
    work: &Path,
) -> Option<Finding> {
    let source = work.join("source.cp");
    std::fs::write(&source, print::to_source(program)).expect("writing the source");
    if let Some(interp) = &tc.interpreter {
        let out = interp
            .run(&source, b"", RUN_LIMIT)
            .unwrap_or_else(|e| panic!("the harness could not start the interpreter: {e}"));
        if let Some(f) = judge(&interp.name(), &out, expect) {
            return Some(f);
        }
    }

    check_cxx_path(tc, &source, expect, work)
}

/// A failing seed, with the program cut down as far as the budget allowed.
#[derive(Debug, Clone)]
pub struct Failure {
    pub seed: u64,
    pub finding: Finding,
    /// The generated program.
    pub source: String,
    /// The smallest program found that fails the same way.
    pub shrunk: String,
}

#[derive(Debug, Default)]
pub struct Report {
    /// Programs with a defined output that were run and judged.
    pub programs: u64,
    /// Trapping programs that were run and judged.
    pub trap_programs: u64,
    /// Programs the transpiler turned down with an allowed diagnostic.
    pub refused: u64,
    pub failures: Vec<Failure>,
}

#[derive(Debug, Clone)]
pub struct Campaign {
    pub seeds: std::ops::Range<u64>,
    pub jobs: usize,
    pub gen: gen::Config,
    /// `still_fails` calls the shrinker may make per failure.
    pub shrink_budget: usize,
}

impl Campaign {
    pub fn new(seeds: std::ops::Range<u64>) -> Self {
        Campaign {
            seeds,
            jobs: std::thread::available_parallelism().map_or(4, |n| n.get()),
            gen: gen::Config::default(),
            shrink_budget: 150,
        }
    }
}

/// The cases for one seed: the generated program, and one variant per
/// trap the toolchain promises to catch.
fn cases(seed: u64, tc: &Toolchain, cfg: &gen::Config) -> Vec<Case> {
    let base = gen::generate(seed, cfg);
    let mut rng = Rng::new(seed ^ 0x7ea9);
    let mut all: Vec<Case> = tc
        .traps
        .iter()
        .filter_map(|trap| gen::with_trap(&base, trap, &mut rng))
        .collect();
    all.insert(0, base);
    all
}

fn shrink_failure(
    tc: &Toolchain,
    program: &Program,
    kind: FindingKind,
    budget: usize,
    work: &Path,
) -> Program {
    shrink::shrink(program, budget, |cand| {
        let Ok(expect) = expectation(cand) else {
            return false;
        };
        check_program(tc, cand, &expect, work).is_some_and(|f| f.kind == kind)
    })
}

/// Checks every seed of `campaign`, `jobs` at a time.
pub fn run(tc: &Toolchain, campaign: &Campaign) -> Report {
    let next = AtomicU64::new(campaign.seeds.start);
    let report = Mutex::new(Report::default());
    std::thread::scope(|s| {
        for _ in 0..campaign.jobs.max(1) {
            s.spawn(|| {
                let work = scratch_dir("fuzz").expect("creating a scratch directory");
                loop {
                    let seed = next.fetch_add(1, Ordering::Relaxed);
                    if seed >= campaign.seeds.end {
                        break;
                    }
                    for (i, case) in cases(seed, tc, &campaign.gen).into_iter().enumerate() {
                        let expect =
                            expectation(&case.program).expect("generated programs are defined");
                        let finding = check_program(tc, &case.program, &expect, &work);
                        let refused = finding.as_ref().is_some_and(|f| tc.allows_refusal(f));
                        {
                            let mut report = report.lock().unwrap();
                            match (refused, i) {
                                (true, _) => report.refused += 1,
                                (false, 0) => report.programs += 1,
                                (false, _) => report.trap_programs += 1,
                            }
                        }
                        if refused {
                            // The trap variants contain the same code.
                            break;
                        }
                        let Some(finding) = finding else { continue };
                        let small = shrink_failure(
                            tc,
                            &case.program,
                            finding.kind,
                            campaign.shrink_budget,
                            &work,
                        );
                        report.lock().unwrap().failures.push(Failure {
                            seed,
                            finding,
                            source: print::to_source(&case.program),
                            shrunk: print::to_source(&small),
                        });
                        // A trap variant would fail the same way.
                        break;
                    }
                }
                let _ = std::fs::remove_dir_all(&work);
            });
        }
    });
    let mut report = report.into_inner().unwrap();
    report.failures.sort_by_key(|f| f.seed);
    report
}

/// Checks one hand-written program against an expected output.
pub fn check_source(tc: &Toolchain, source: &str, expect: &Expect) -> Option<Finding> {
    let work = scratch_dir("case").expect("creating a scratch directory");
    let path = work.join("source.cp");
    std::fs::write(&path, source).expect("writing the source");
    let finding = check_cxx_path(tc, &path, expect, &work);
    let _ = std::fs::remove_dir_all(&work);
    finding
}

/// Transpiles, compiles and runs `source`, and judges the run.
fn check_cxx_path(tc: &Toolchain, source: &Path, expect: &Expect, work: &Path) -> Option<Finding> {
    let path = tc.transpiler.name();
    let finding = |kind, detail: &str| {
        Some(Finding {
            kind,
            path: path.clone(),
            detail: detail.trim().to_string(),
        })
    };
    let (cpp, exe) = (work.join("out.cpp"), work.join("out"));
    if let Err(diag) = tc.transpiler.transpile(source, &cpp) {
        return finding(FindingKind::Rejected, &diag);
    }
    if let Err(diag) = tc.cxx.compile(&cpp, &exe) {
        let first_error = diag.lines().find(|l| l.contains("error")).unwrap_or(&diag);
        return finding(FindingKind::BadCxx, first_error);
    }
    let out = process::run(&mut Command::new(&exe), b"", RUN_LIMIT)
        .unwrap_or_else(|e| panic!("the harness could not run the compiled program: {e}"));
    judge(&path, &out, expect)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(stdout: &str, stderr: &str, exit: Exit) -> Output {
        Output {
            stdout: stdout.into(),
            stderr: stderr.into(),
            exit,
            elapsed: Duration::ZERO,
        }
    }

    fn kind(out: &Output, expect: &Expect) -> Option<FindingKind> {
        judge("test", out, expect).map(|f| f.kind)
    }

    #[test]
    fn a_defined_program_must_print_exactly_the_expected_output() {
        let want = Expect::Output("1\n2\n".into());
        assert_eq!(kind(&output("1\n2\n", "", Exit::Code(0)), &want), None);
        assert_eq!(
            kind(&output("1\n3\n", "", Exit::Code(0)), &want),
            Some(FindingKind::WrongOutput)
        );
        assert_eq!(
            kind(&output("1\n2\n", "", Exit::Code(3)), &want),
            Some(FindingKind::Crash)
        );
        assert_eq!(
            kind(&output("1\n", "", Exit::TimedOut), &want),
            Some(FindingKind::Timeout)
        );
    }

    #[test]
    fn a_sanitizer_report_fails_even_when_the_output_is_right() {
        let want = Expect::Output("1\n".into());
        let out = output(
            "1\n",
            "a.cpp:3:5: runtime error: signed integer overflow",
            Exit::Code(0),
        );
        assert_eq!(kind(&out, &want), Some(FindingKind::Sanitizer));
    }

    #[test]
    fn a_trap_must_be_a_clean_nonzero_exit() {
        let want = Expect::Trap("1\n2\n".into());
        assert_eq!(
            kind(&output("1\n", "line 3: overflow", Exit::Code(3)), &want),
            None
        );
        assert_eq!(
            kind(&output("1\n2\n", "", Exit::Code(0)), &want),
            Some(FindingKind::MissedTrap)
        );
        assert_eq!(
            kind(&output("", "", Exit::Signal), &want),
            Some(FindingKind::Crash)
        );
        assert_eq!(
            kind(&output("9\n", "", Exit::Code(3)), &want),
            Some(FindingKind::WrongOutput)
        );
    }

    #[test]
    fn the_first_differing_line_is_named() {
        assert_eq!(
            first_difference("1\n2\n", "1\n5\n"),
            "line 2: expected `2`, got `5`"
        );
        assert_eq!(
            first_difference("1\n", "1\n5\n"),
            "line 2: expected <end of output>, got `5`"
        );
    }
}
