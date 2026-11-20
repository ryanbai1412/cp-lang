//! Loop invariants: true on entry, and kept by every pass of the body.
//!
//! The body is a list of [`Step`]s, which is all a proof needs to know
//! about it: what is assigned, what is forgotten, what is assumed.
//!
//! ```
//! use cpl_prover::{Context, Loop, Step};
//! // var l = 0
//! // for r in 0..n:
//! //     while a[r] - a[l] > k: l += 1      invariant l <= r
//! let mut ctx = Context::new();
//! ctx.var("n", "1..=2e5").unwrap();
//! ctx.var("k", "0..=1e9").unwrap();
//! ctx.var("r", "0..n").unwrap();
//! ctx.var("l", "0..n").unwrap();
//! ctx.array("a", "Int").unwrap();
//! ctx.assume("l <= r").unwrap();
//! let inner = Loop::new("a[r] - a[l] > k").unwrap()
//!     .invariant("l <= r").unwrap()
//!     .step(Step::assign("l", "l + 1").unwrap());
//! assert!(ctx.check_loop(&inner).unwrap().holds());
//! ```

use std::fmt;

use crate::context::{Context, Verdict};
use crate::parse;
use crate::term::{Error, Term};

/// One statement of a loop body, as far as a proof cares.
#[derive(Clone, PartialEq, Debug)]
pub enum Step {
    /// `name = term`.
    Assign(String, Term),
    /// The variable changes in a way the prover is not told: a call
    /// that may write it, a read from input.
    Havoc(String),
    /// A fact that holds here: an `assert`, an `ensures` of a call.
    Assume(Term),
    /// `if cond: then else: other`.
    If(Term, Vec<Step>, Vec<Step>),
}

impl Step {
    pub fn assign(name: &str, term: &str) -> Result<Step, Error> {
        Ok(Step::Assign(name.to_string(), parse::term(term)?))
    }

    pub fn havoc(name: &str) -> Step {
        Step::Havoc(name.to_string())
    }

    pub fn assume(fact: &str) -> Result<Step, Error> {
        Ok(Step::Assume(parse::term(fact)?))
    }

    pub fn branch(cond: &str, then: Vec<Step>, other: Vec<Step>) -> Result<Step, Error> {
        Ok(Step::If(parse::term(cond)?, then, other))
    }

    fn modified(&self, into: &mut Vec<String>) {
        match self {
            Step::Assign(name, _) | Step::Havoc(name) => {
                if !into.contains(name) {
                    into.push(name.clone());
                }
            }
            Step::Assume(_) => {}
            Step::If(_, then, other) => then
                .iter()
                .chain(other)
                .for_each(|step| step.modified(into)),
        }
    }
}

/// `while guard: body`, with its invariants.
#[derive(Clone, PartialEq, Debug)]
pub struct Loop {
    pub guard: Term,
    pub invariants: Vec<Term>,
    pub body: Vec<Step>,
}

impl Loop {
    pub fn new(guard: &str) -> Result<Loop, Error> {
        Ok(Loop {
            guard: parse::term(guard)?,
            invariants: Vec::new(),
            body: Vec::new(),
        })
    }

    pub fn invariant(mut self, invariant: &str) -> Result<Loop, Error> {
        self.invariants.push(parse::term(invariant)?);
        Ok(self)
    }

    pub fn step(mut self, step: Step) -> Loop {
        self.body.push(step);
        self
    }

    /// The variables the body may change.
    pub fn modified(&self) -> Vec<String> {
        let mut names = Vec::new();
        self.body.iter().for_each(|step| step.modified(&mut names));
        names
    }
}

/// For each invariant, whether it holds on entry and whether the body keeps it.
#[derive(Clone, PartialEq, Debug)]
pub struct LoopReport {
    pub entry: Vec<(String, Verdict)>,
    pub preserved: Vec<(String, Verdict)>,
}

impl LoopReport {
    pub fn holds(&self) -> bool {
        self.entry
            .iter()
            .chain(&self.preserved)
            .all(|(_, verdict)| verdict.is_proved())
    }
}

impl fmt::Display for LoopReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .entry
            .iter()
            .map(|(invariant, verdict)| format!("invariant {invariant} on entry: {verdict}"))
            .chain(self.preserved.iter().map(|(invariant, verdict)| {
                format!("invariant {invariant} after the body: {verdict}")
            }))
            .collect::<Vec<_>>();
        write!(f, "{}", lines.join("\n"))
    }
}

impl Context {
    /// The contexts after running `steps`: one per path through its `if`s.
    pub fn run(&self, steps: &[Step]) -> Result<Vec<Context>, Error> {
        let mut paths = vec![self.clone()];
        for step in steps {
            let mut next = Vec::with_capacity(paths.len());
            for mut ctx in paths {
                match step {
                    Step::Assign(name, term) => {
                        ctx.assign_term(name, term)?;
                        next.push(ctx);
                    }
                    Step::Havoc(name) => {
                        ctx.havoc(name)?;
                        next.push(ctx);
                    }
                    Step::Assume(fact) => {
                        ctx.assume_term(fact)?;
                        next.push(ctx);
                    }
                    Step::If(cond, then, other) => {
                        let mut taken = ctx.clone();
                        taken.assume_term(cond)?;
                        next.extend(taken.run(then)?);
                        ctx.assume_term(&cond.clone().negate())?;
                        next.extend(ctx.run(other)?);
                    }
                }
            }
            paths = next;
        }
        Ok(paths)
    }

    /// Check the invariants of a loop that starts in this context: each
    /// holds on entry, and each holds again after the body runs from any
    /// state that satisfies the invariants and the guard.
    pub fn check_loop(&self, lp: &Loop) -> Result<LoopReport, Error> {
        let mut entry = Vec::with_capacity(lp.invariants.len());
        for invariant in &lp.invariants {
            entry.push((invariant.to_string(), self.clone().prove_term(invariant)?));
        }
        let mut preserved = Vec::with_capacity(lp.invariants.len());
        let paths = self.at_iteration(lp)?.run(&lp.body)?;
        for invariant in &lp.invariants {
            let mut verdict = Verdict::Proved;
            for path in &paths {
                verdict = path.clone().prove_term(invariant)?;
                if !verdict.is_proved() {
                    break;
                }
            }
            preserved.push((invariant.to_string(), verdict));
        }
        Ok(LoopReport { entry, preserved })
    }

    /// The context at the top of an arbitrary pass of the loop: what the
    /// body changes is unknown again, except for the invariants and the guard.
    pub fn at_iteration(&self, lp: &Loop) -> Result<Context, Error> {
        let mut ctx = self.at_head(lp)?;
        ctx.assume_term(&lp.guard)?;
        Ok(ctx)
    }

    /// The context after the loop: the invariants hold and the guard does not.
    pub fn after_loop(&self, lp: &Loop) -> Result<Context, Error> {
        let mut ctx = self.at_head(lp)?;
        ctx.assume_term(&lp.guard.clone().negate())?;
        Ok(ctx)
    }

    fn at_head(&self, lp: &Loop) -> Result<Context, Error> {
        let mut ctx = self.clone();
        for name in lp.modified() {
            ctx.havoc(&name)?;
        }
        for invariant in &lp.invariants {
            ctx.assume_term(invariant)?;
        }
        Ok(ctx)
    }
}
