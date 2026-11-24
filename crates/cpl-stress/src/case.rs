//! One case judged. Each program a case needs runs behind its own
//! [`Runner`] (a built process, or the VM in the browser); [`Roles::judge`]
//! runs them and `judge.rs` picks the verdict from what they did.

use crate::judge::{Launch, Verdict};
use crate::program::Outcome;
use crate::run::StressError;
use crate::runner::Runner;
use std::time::Duration;

/// How one run of a program ended: what it printed, and why it stopped
/// early (a runtime error or the time limit), if it did.
#[derive(Clone, PartialEq, Debug)]
pub struct Ran {
    pub out: String,
    pub stop: Option<Verdict>,
    /// What the run cost, when it ran as a process.
    pub cost: Option<Outcome>,
}

/// What a checker or interactor said of a run.
#[derive(Clone, PartialEq, Debug)]
pub enum Ruling {
    Accepted,
    /// It rejected the run, saying why when it did.
    Rejected(Option<String>),
    /// It failed itself (an index or overflow trap, say): its trap.
    Broke(String),
}

/// Which side of an interactive run printed a line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Solution,
    Interactor,
}

impl Side {
    pub fn name(self) -> &'static str {
        match self {
            Side::Solution => "solution",
            Side::Interactor => "interactor",
        }
    }
}

/// What the two sides of an interactive run said, in order: the first
/// [`Transcript::KEPT`] lines, and how many more there were.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Transcript {
    pub lines: Vec<(Side, String)>,
    pub truncated: usize,
}

impl Transcript {
    pub const KEPT: usize = 200;

    /// Adds what `side` printed, a line at a time.
    pub fn say(&mut self, side: Side, text: &str) {
        for line in text.lines() {
            match self.lines.len() < Self::KEPT {
                true => self.lines.push((side, line.to_string())),
                false => self.truncated += 1,
            }
        }
    }
}

/// One interactive run: the solution's side, the interactor's ruling,
/// what they said, and the interactor's trap as `cpl check --format json`
/// gives a diagnostic when it stopped at one.
#[derive(Clone, PartialEq, Debug)]
pub struct Talked {
    pub solution: Ran,
    pub ruling: Ruling,
    pub transcript: Transcript,
    /// The interactor's trap, and the solution's.
    pub trap: Option<cpl_json::Value>,
    pub solution_trap: Option<cpl_json::Value>,
}

/// A case the solution, or a helper, failed.
#[derive(Clone, PartialEq, Debug)]
pub struct Failed {
    pub verdict: Verdict,
    /// What the brute printed.
    pub expected: String,
    /// What the solution printed.
    pub got: String,
    /// What an interactive run's two sides said.
    pub transcript: Option<Transcript>,
    /// The interactor's ruling in its own words, for an interactive run.
    pub ruling: Option<Ruling>,
    /// Where a helper that ran during the case stopped, as a diagnostic.
    pub trap: Option<cpl_json::Value>,
}

impl Failed {
    pub fn new(verdict: Verdict, expected: String, got: String) -> Failed {
        Failed {
            verdict,
            expected,
            got,
            transcript: None,
            ruling: None,
            trap: None,
        }
    }
}

/// How long a brute may take: it is slow by design.
pub fn brute_limit(limit: Duration) -> Duration {
    limit * 10
}

/// How long a checker or interactor may take beyond the solution's limit.
pub fn judge_limit(limit: Duration) -> Duration {
    limit.max(Duration::from_secs(10))
}

/// One run's result: `None` when the case passed.
pub type Wrong = Option<Failed>;

/// The programs a case runs, each behind its [`Runner`]: the solution,
/// and the helpers that judge it. A helper is a brute, checker or
/// interactor; each gets ten times the solution's limit.
pub struct Roles {
    pub solution: Runner,
    pub brute: Option<Runner>,
    pub checker: Option<Runner>,
    pub interactor: Option<(Runner, Launch)>,
    /// A second opinion on an accepted run: the `@cache` file built cold.
    pub cold: Option<Runner>,
    /// How long one run of the solution may take.
    pub limit: Duration,
}

/// One case judged, and the solution's run when it ran.
pub struct Judgement {
    pub wrong: Wrong,
    pub solution: Option<Ran>,
}

impl Roles {
    pub fn has_brute(&self) -> bool {
        self.brute.is_some()
    }

    /// True when `input` stops the solution (runtime error / TLE).
    pub fn stops(&self, input: &str) -> bool {
        (self.solution.run(input, self.limit)).is_ok_and(|ran| ran.stop.is_some())
    }

    pub fn limit(&self) -> Duration {
        self.limit
    }

    /// [`Roles::run_within`] the solution's own limit.
    pub fn run(&self, input: &str) -> Result<Wrong, StressError> {
        self.run_within(input, self.limit)
    }

    /// The case judged with the solution stopped after `limit`.
    pub fn run_within(&self, input: &str, limit: Duration) -> Result<Wrong, StressError> {
        Ok(self.judge(input, limit)?.wrong)
    }

    /// The verdict on one case: the brute's failure first, then the
    /// solution's, then the judge's.
    pub fn judge(&self, input: &str, limit: Duration) -> Result<Judgement, StressError> {
        let expected = match &self.brute {
            Some(brute) => match brute.run(input, brute_limit(limit))? {
                Ran {
                    stop: Some(stop), ..
                } => {
                    return Ok(Judgement {
                        wrong: Some(crate::judge::brute_failed(stop)),
                        solution: None,
                    });
                }
                ran => Some(ran.out),
            },
            None => None,
        };
        if let Some((interactor, launch)) = &self.interactor {
            let talked = interactor.talk(&self.solution, *launch, input, limit)?;
            let solution = Some(talked.solution.clone());
            return Ok(Judgement {
                wrong: crate::judge::interaction(talked, expected.unwrap_or_default()),
                solution,
            });
        }
        let ran = self.solution.run(input, limit)?;
        let ruling = match &self.checker {
            Some(checker) => Some(checker.check(input, &ran.out, expected.as_deref(), limit)?),
            None => None,
        };
        let verdict = crate::judge::ruled(ran.stop.clone(), ruling, expected.as_deref(), &ran.out);
        let wrong = match verdict {
            Some(verdict) => Some(Failed::new(
                verdict,
                expected.unwrap_or_default(),
                ran.out.clone(),
            )),
            None => self.recheck(input, &ran.out, limit)?,
        };
        Ok(Judgement {
            wrong,
            solution: Some(ran),
        })
    }

    /// The `@cache` file built cold on an accepted run's input.
    fn recheck(&self, input: &str, got: &str, limit: Duration) -> Result<Wrong, StressError> {
        let Some(cold) = &self.cold else {
            return Ok(None);
        };
        let ran = cold.run(input, brute_limit(limit))?;
        Ok(ran
            .cost
            .and_then(|outcome| crate::judge::recheck(&outcome, got)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Programs that ran already: each side says what it is given.
    #[derive(Default)]
    struct Canned {
        brute: Option<Ran>,
        solution: Option<Ran>,
        check: Option<Ruling>,
        talked: Option<Talked>,
    }

    fn roles(canned: Canned) -> Roles {
        use crate::runner::Canned as Runs;
        let runs = |ran: Ran| Runner::Canned(Runs::Runs(Box::new(move |_, _| ran.clone())));
        Roles {
            solution: runs(canned.solution.unwrap_or(Ran {
                out: String::new(),
                stop: None,
                cost: None,
            })),
            brute: canned.brute.map(runs),
            checker: canned
                .check
                .map(|ruling| Runner::Canned(Runs::Rules(ruling))),
            interactor: (canned.talked)
                .map(|talked| (Runner::Canned(Runs::Talks(Box::new(talked))), Launch::Stdio)),
            cold: None,
            limit: Duration::from_secs(1),
        }
    }

    fn ran(out: &str, stop: Option<Verdict>) -> Option<Ran> {
        Some(Ran {
            out: out.to_string(),
            stop,
            cost: None,
        })
    }

    fn verdict(sides: Canned) -> Verdict {
        roles(sides)
            .run("")
            .unwrap()
            .map_or(Verdict::Accepted, |failed| failed.verdict)
    }

    fn talked(stop: Option<Verdict>, ruling: Ruling) -> Option<Talked> {
        Some(Talked {
            solution: Ran {
                out: String::new(),
                stop,
                cost: None,
            },
            ruling,
            transcript: Transcript::default(),
            trap: None,
            solution_trap: None,
        })
    }

    #[test]
    fn a_failing_brute_is_named_before_the_solution() {
        let sides = Canned {
            brute: ran("", Some(Verdict::TimeLimit)),
            solution: ran("", Some(Verdict::RuntimeError("x".to_string()))),
            ..Canned::default()
        };
        let why = "it ran past ten times the time limit".to_string();
        assert_eq!(verdict(sides), Verdict::BruteError(why));
    }

    #[test]
    fn a_stopped_solution_is_its_own_verdict_before_any_judge() {
        let sides = Canned {
            brute: ran("1", None),
            solution: ran("", Some(Verdict::TimeLimit)),
            check: Some(Ruling::Accepted),
            ..Canned::default()
        };
        assert_eq!(verdict(sides), Verdict::TimeLimit);
    }

    #[test]
    fn a_checker_rules_in_its_own_words_or_breaks() {
        let rejected = |said: Option<&str>| Canned {
            solution: ran("2", None),
            check: Some(Ruling::Rejected(said.map(str::to_string))),
            ..Canned::default()
        };
        assert_eq!(
            verdict(rejected(Some("too big"))),
            Verdict::Wrong("checker: too big".to_string())
        );
        let silent = Verdict::Wrong("the checker rejected the output".to_string());
        assert_eq!(verdict(rejected(None)), silent);
        let broke = Canned {
            solution: ran("2", None),
            check: Some(Ruling::Broke("R0101".to_string())),
            ..Canned::default()
        };
        assert_eq!(
            verdict(broke),
            Verdict::JudgeError("checker", "R0101".to_string())
        );
    }

    #[test]
    fn without_a_brute_or_judge_a_clean_run_passes() {
        let sides = Canned {
            solution: ran("anything", None),
            ..Canned::default()
        };
        assert_eq!(verdict(sides), Verdict::Accepted);
    }

    #[test]
    fn an_interactor_starved_by_a_failed_solution_blames_the_solution() {
        let over = Verdict::RuntimeError("over the query limit".to_string());
        let starved = Ruling::Rejected(Some("the solution's output ended".to_string()));
        let sides = Canned {
            talked: talked(Some(over.clone()), starved),
            ..Canned::default()
        };
        assert_eq!(verdict(sides), over);
    }

    #[test]
    fn an_interactive_time_limit_comes_first_and_a_broken_interactor_is_its_bug() {
        let late = Canned {
            talked: talked(Some(Verdict::TimeLimit), Ruling::Broke("R01".to_string())),
            ..Canned::default()
        };
        assert_eq!(verdict(late), Verdict::TimeLimit);
        let broke = Canned {
            talked: talked(None, Ruling::Broke("R01".to_string())),
            ..Canned::default()
        };
        assert_eq!(
            verdict(broke),
            Verdict::JudgeError("interactor", "R01".to_string())
        );
        let accepted = Canned {
            talked: talked(None, Ruling::Accepted),
            ..Canned::default()
        };
        assert_eq!(verdict(accepted), Verdict::Accepted);
    }

    #[test]
    fn a_transcript_keeps_its_first_lines_and_counts_the_rest() {
        let mut transcript = Transcript::default();
        for i in 0..Transcript::KEPT + 3 {
            let side = [Side::Solution, Side::Interactor][i % 2];
            transcript.say(side, &format!("{i}\n"));
        }
        assert_eq!(transcript.lines.len(), Transcript::KEPT);
        assert_eq!(transcript.lines[1], (Side::Interactor, "1".to_string()));
        assert_eq!(transcript.truncated, 3);
    }
}
