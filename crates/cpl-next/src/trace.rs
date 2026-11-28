//! `trace(Job, input)` (contract 8, handbook/design/next/events.md): one run of the db's
//! `program(Job)` built with events, as the recording tools read through methods. Each step
//! is an event of the stream and its `At`; only `source_of` turns a site into a place in a
//! file.
use crate::linked::{Built, Job, Linked};
use crate::{Diag, Form, Outcome};
use cpl_next_db::db::Db;
use cpl_next_ir::ir::EventKind;
use cpl_next_ir::prog::CTy;
use cpl_next_ir::run::{at, At, Fired};
use cpl_next_ir::ty::StructId;
use std::sync::Arc;

/// A run's events, each with its `At`, the calls they make up and how the run ended.
pub struct Recording {
    events: Vec<Fired>,
    at: Vec<At>,
    /// Each fn's name, by instance.
    names: Vec<Arc<str>>,
    calls: Vec<Call>,
    outcome: Outcome,
    /// Per step, the line of the user's file its site is on; 0 for none.
    lines: Vec<u32>,
    /// Per `Write` step, the value it shows as JSON, typed by the binding's type.
    values: Vec<Option<String>>,
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

    /// The line of the user's file step `i` is on; 0 inside std or for none.
    pub fn line(&self, i: usize) -> u32 {
        self.lines.get(i).copied().unwrap_or(0)
    }

    /// `cpl run --trace --format json`'s document: each step that writes a top-level
    /// binding, with its line, that line's text as its label and the value written. With
    /// `watch` named, only the steps writing one of them (Rtrace.2).
    pub fn json(&self, file: &str, code: &str, watch: &[String]) -> String {
        let kept = |i: &usize| match &self.events[*i].kind {
            EventKind::Write { shown, .. } => {
                watch.is_empty() || watch.iter().any(|w| **w == **shown)
            }
            _ => false,
        };
        let steps: Vec<usize> = (0..self.events.len()).filter(kept).collect();
        let mut vars: Vec<&str> = Vec::new();
        for &i in &steps {
            if let EventKind::Write { shown, .. } = &self.events[i].kind {
                if !vars.contains(&&**shown) {
                    vars.push(shown);
                }
            }
        }
        let label = |line: u32| {
            (line as usize)
                .checked_sub(1)
                .and_then(|l| code.lines().nth(l))
                .map_or("", str::trim)
        };
        cpl_json::write_with(|w| {
            w.object()
                .key("schemaVersion")
                .int(1)
                .key("title")
                .str(file);
            w.key("source")
                .object()
                .key("file")
                .str(file)
                .key("code")
                .str(code)
                .end();
            w.key("variables").object();
            for v in &vars {
                w.key(v)
                    .object()
                    .key("name")
                    .str(v)
                    .key("kind")
                    .str("global")
                    .end();
            }
            w.end()
                .key("runs")
                .object()
                .key("fast")
                .object()
                .key("label")
                .str("run");
            w.key("steps").array();
            for (id, &i) in steps.iter().enumerate() {
                let line = self.line(i);
                w.object().key("id").int(id).key("line").int(line);
                w.key("label").str(label(line)).key("values").object();
                if let (EventKind::Write { shown, .. }, Some(v)) =
                    (&self.events[i].kind, &self.values[i])
                {
                    w.key(shown).json(v);
                }
                w.end().end();
            }
            w.end().end().end().end();
        })
    }
}

/// What a trace's JSON says of a value of type `t` its event shows as `shown`: a `Bool` and a
/// number as themselves, an `option` as `null` or the value it holds, anything else as text.
fn value_json(t: &CTy, shown: &str, option: Option<StructId>) -> String {
    let text = || {
        cpl_json::write_with(|w| {
            w.str(shown);
        })
    };
    match t.unlent() {
        CTy::Bool if matches!(shown, "true" | "false") => shown.into(),
        CTy::Int | CTy::U64 | CTy::I128 | CTy::Range(..) | CTy::RangeTo(..)
            if shown.parse::<i128>().is_ok() || shown.parse::<u64>().is_ok() =>
        {
            shown.into()
        }
        CTy::F64 if shown.parse::<f64>().is_ok_and(f64::is_finite) => shown.into(),
        CTy::Struct(s, ts, _) if Some(*s) == option && ts.len() == 1 => match shown {
            "none" => "null".into(),
            _ => value_json(&ts[0], shown, option),
        },
        _ => text(),
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
    let c = &p.0;
    let lines = (outcome.events.iter())
        .map(|e| {
            let origin = funcs[e.func as usize].origins.get(e.site as usize);
            origin.map_or(0, |o| c.line(*o))
        })
        .collect();
    let values = (outcome.events.iter())
        .map(|e| match (&e.kind, e.vals.first()) {
            (EventKind::Write { global, .. }, Some(v)) => {
                let t = &c.prog.globals[*global as usize];
                Some(value_json(t, v, c.option))
            }
            _ => None,
        })
        .collect();
    Ok(Recording {
        events: outcome.events.clone(),
        at,
        names,
        calls,
        outcome,
        lines,
        values,
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
