//! `trace(Job, input)` (contract 8, handbook/design/next/events.md): one run of the db's
//! `program(Job)` built with events, as the recording tools read through methods. Each step
//! is an event of the stream and its `At`; only `source_of` turns a site into a place in a
//! file.
use crate::linked::{Built, Job, Linked};
use crate::{Diag, Form, Outcome};
use cpl_next_db::db::Db;
use cpl_next_ir::ir::EventKind;
use cpl_next_ir::run::{at, At, Fired};
use std::sync::Arc;

/// A run's events, each with its `At`, the calls they make up and how the run ended.
pub struct Recording {
    events: Vec<Fired>,
    at: Vec<At>,
    /// Each fn's name, by instance.
    names: Vec<Arc<str>>,
    calls: Vec<Call>,
    outcome: Outcome,
}

/// One event of a run: what it reports, in which fn, the values it shows and how many calls
/// are open around it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step<'a> {
    pub kind: &'a EventKind,
    pub func: &'a str,
    pub vals: &'a [String],
    pub depth: usize,
}

/// One call of a fn: the steps it took and what it returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub name: Arc<str>,
    /// 1-based: the third call of `name` in the run is call 3.
    pub ordinal: usize,
    /// The steps taken while it ran are `entry..=exit`; a call the run never returned from
    /// runs to the last step.
    pub entry: usize,
    pub exit: Option<usize>,
    /// Fns running around it; a fn called from the top level is at 1.
    pub depth: usize,
    /// Its shown parameters, in order.
    pub args: Vec<String>,
    /// Its shown result, unless it returns nothing or never returned.
    pub returned: Option<String>,
}

impl Recording {
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn step(&self, i: usize) -> Option<Step<'_>> {
        let (e, a) = (self.events.get(i)?, &self.at[i]);
        Some(Step {
            kind: &e.kind,
            func: &self.names[e.func as usize],
            vals: &e.vals,
            depth: a.call.len(),
        })
    }

    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    /// Where step `i` is in the run: its calls, its site and which time the site ran.
    pub fn place(&self, i: usize) -> Option<&At> {
        self.at.get(i)
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }
}

/// `job`'s program run on `input`, its events reported, or every diagnostic that stops it.
pub fn trace(db: &Db, job: &Job, input: &str) -> Result<Recording, Arc<[Diag]>> {
    let job = Job {
        events: true,
        ..job.clone()
    };
    let p = match db.get::<Linked>(&job) {
        Ok(Built(built)) => built?,
        Err(e) => return Err(Arc::from(vec![Diag::internal(&e)])),
    };
    let outcome = p.run(input);
    let funcs = p.form(Form::Raw).1;
    let at = at(&outcome.events, funcs);
    let names: Vec<Arc<str>> = funcs.iter().map(|f| Arc::from(f.name.as_str())).collect();
    let calls = calls(&outcome.events, &names);
    Ok(Recording {
        events: outcome.events.clone(),
        at,
        names,
        calls,
        outcome,
    })
}

/// The calls the stream's `Enter`s open and its `Exit`s close.
fn calls(events: &[Fired], names: &[Arc<str>]) -> Vec<Call> {
    let (mut calls, mut open) = (Vec::<Call>::new(), Vec::new());
    let mut seen = vec![0; names.len()];
    for (i, e) in events.iter().enumerate() {
        match e.kind {
            EventKind::Enter => {
                seen[e.func as usize] += 1;
                open.push(calls.len());
                calls.push(Call {
                    name: names[e.func as usize].clone(),
                    ordinal: seen[e.func as usize],
                    entry: i,
                    exit: None,
                    depth: open.len(),
                    args: e.vals.clone(),
                    returned: None,
                });
            }
            EventKind::Exit => {
                if let Some(c) = open.pop() {
                    calls[c].exit = Some(i);
                    calls[c].returned = e.vals.first().cloned();
                }
            }
            EventKind::Write { .. } => {}
        }
    }
    calls
}

#[cfg(test)]
mod tests;
