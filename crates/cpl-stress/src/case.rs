//! One case judged. Every engine runs its programs its own way (built
//! processes, the interpreter in the browser) and hands the runs to
//! [`judge_case`], the one place a case's verdict is picked.

use crate::judge::{first_difference, Verdict};
use crate::run::StressError;
use std::time::Duration;

/// How one run of a program ended: what it printed, and why it stopped
/// early (a runtime error or the time limit), if it did.
#[derive(Clone, PartialEq, Debug)]
pub struct Ran {
    pub out: String,
    pub stop: Option<Verdict>,
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

/// How an engine runs each program a case needs. A helper is a brute,
/// checker or interactor; each gets ten times `limit`.
pub trait Sides {
    /// The brute's run, or `None` when there is no brute.
    fn brute(&self, input: &str, limit: Duration) -> Result<Option<Ran>, StressError>;
    /// The solution's run on its own.
    fn solution(&self, input: &str, limit: Duration) -> Result<Ran, StressError>;
    /// A checker's ruling on `got`, or `None` when there is no checker.
    fn check(
        &self,
        input: &str,
        got: &str,
        expected: Option<&str>,
        limit: Duration,
    ) -> Result<Option<Ruling>, StressError>;
    /// The solution run against the interactor on the hidden test
    /// `input`, or `None` when there is no interactor.
    fn interact(&self, input: &str, limit: Duration) -> Result<Option<Talked>, StressError>;
    /// A second opinion on an accepted run, such as a `@cache` cold build.
    fn recheck(&self, _input: &str, _got: &str, _limit: Duration) -> Result<Wrong, StressError> {
        Ok(None)
    }
}

/// The verdict on one case: the brute's failure first, then the
/// solution's, then the judge's.
pub fn judge_case(sides: &dyn Sides, input: &str, limit: Duration) -> Result<Wrong, StressError> {
    let expected = match sides.brute(input, limit)? {
        Some(Ran {
            stop: Some(stop), ..
        }) => {
            let why = match stop {
                Verdict::RuntimeError(why) => why,
                _ => "it ran past ten times the time limit".to_string(),
            };
            return Ok(Some(Failed::new(
                Verdict::BruteError(why),
                String::new(),
                String::new(),
            )));
        }
        Some(ran) => Some(ran.out),
        None => None,
    };
    if let Some(talked) = sides.interact(input, limit)? {
        return Ok(interaction(talked, expected.unwrap_or_default()));
    }
    let ran = sides.solution(input, limit)?;
    if let Some(stop) = ran.stop {
        return Ok(Some(Failed::new(
            stop,
            expected.unwrap_or_default(),
            ran.out,
        )));
    }
    let verdict = match sides.check(input, &ran.out, expected.as_deref(), limit)? {
        Some(Ruling::Accepted) => None,
        Some(Ruling::Rejected(said)) => Some(Verdict::Wrong(match said {
            Some(said) => format!("checker: {said}"),
            None => "the checker rejected the output".to_string(),
        })),
        Some(Ruling::Broke(trap)) => Some(Verdict::JudgeError("checker", trap)),
        // With no expected output, a token judge accepts any clean run.
        None => expected
            .as_deref()
            .and_then(|expected| first_difference(expected, &ran.out))
            .map(Verdict::Wrong),
    };
    if let Some(verdict) = verdict {
        return Ok(Some(Failed::new(
            verdict,
            expected.unwrap_or_default(),
            ran.out,
        )));
    }
    sides.recheck(input, &ran.out, limit)
}

/// What an interactive run is called. The time limit comes first; an
/// interactor left with nothing to read after the solution failed (say,
/// over its query limit) only follows from that failure.
fn interaction(talked: Talked, expected: String) -> Wrong {
    let Talked {
        solution,
        ruling,
        transcript,
        trap,
        solution_trap,
    } = talked;
    let starved = |said: &Option<String>| {
        said.as_deref()
            .is_some_and(|said| said.starts_with("the solution's output ended"))
    };
    let verdict = match (&solution.stop, &ruling) {
        (Some(Verdict::TimeLimit), _) => Verdict::TimeLimit,
        // The interactor accepted; a crash after the last answer still counts.
        (stop, Ruling::Accepted) => stop.clone().unwrap_or(Verdict::Accepted),
        (Some(stop), Ruling::Rejected(said)) if starved(said) => stop.clone(),
        (_, Ruling::Broke(why)) => Verdict::JudgeError("interactor", why.clone()),
        (_, Ruling::Rejected(said)) => Verdict::Wrong(match said {
            Some(said) => format!("interactor: {said}"),
            None => "the interactor rejected the run".to_string(),
        }),
    };
    if verdict.accepted() {
        return None;
    }
    Some(Failed {
        expected,
        got: solution.out,
        transcript: Some(transcript),
        trap: match verdict {
            Verdict::JudgeError(..) => trap,
            Verdict::RuntimeError(_) => solution_trap,
            _ => None,
        },
        verdict,
        ruling: Some(ruling),
    })
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

    impl Sides for Canned {
        fn brute(&self, _: &str, _: Duration) -> Result<Option<Ran>, StressError> {
            Ok(self.brute.clone())
        }
        fn solution(&self, _: &str, _: Duration) -> Result<Ran, StressError> {
            self.solution
                .clone()
                .ok_or_else(|| StressError("no solution".to_string()))
        }
        fn check(
            &self,
            _: &str,
            _: &str,
            _: Option<&str>,
            _: Duration,
        ) -> Result<Option<Ruling>, StressError> {
            Ok(self.check.clone())
        }
        fn interact(&self, _: &str, _: Duration) -> Result<Option<Talked>, StressError> {
            Ok(self.talked.clone())
        }
    }

    fn ran(out: &str, stop: Option<Verdict>) -> Option<Ran> {
        Some(Ran {
            out: out.to_string(),
            stop,
        })
    }

    fn verdict(sides: Canned) -> Verdict {
        judge_case(&sides, "", Duration::from_secs(1))
            .unwrap()
            .map_or(Verdict::Accepted, |failed| failed.verdict)
    }

    fn talked(stop: Option<Verdict>, ruling: Ruling) -> Option<Talked> {
        Some(Talked {
            solution: Ran {
                out: String::new(),
                stop,
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
