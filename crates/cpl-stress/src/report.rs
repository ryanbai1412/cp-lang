//! What a stress run found, as one typed value both `cpl stress
//! --format json` and the browser serialize through `cpl_json`.

use crate::case::{Ruling, Transcript};
use crate::cli::Stage;
use crate::run::{Report, StressError};
use cpl_interp::trace::Divergence;
use cpl_json::{obj, Value};
use std::collections::BTreeMap;

/// A stress run's outcome with everything the editor and the page show
/// about it (`editors/vscode/README.md`, `crates/cpl-wasm/js/cpl.d.ts`).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct StressResult {
    pub report: Report,
    /// The toolchain that built the solution, when the CLI built it.
    pub build: Option<String>,
    /// Where the failing case was saved, shown.
    pub saved: Option<String>,
    /// Where its trace was written, shown.
    pub trace: Option<String>,
    pub stages: Vec<Stage>,
    pub explained: Option<Result<Option<Divergence>, String>>,
    /// The trap the failing case stops at, as `cpl check --format json`
    /// gives a diagnostic.
    pub trap: Option<Value>,
    /// The built solution and the interpreter disagree on the case.
    pub toolchain_bug: bool,
}

impl StressResult {
    pub fn new(report: Report) -> StressResult {
        StressResult {
            report,
            ..StressResult::default()
        }
    }

    pub fn to_json(&self) -> Value {
        let report = &self.report;
        let mut out = BTreeMap::new();
        let mut put = |key: &str, value: Value| out.insert(key.to_string(), value);
        put("case_count", report.cases.into());
        if let Some(build) = &self.build {
            put("build", build.as_str().into());
        }
        let Some(failure) = &report.failure else {
            put("status", "ok".into());
            put("saved_count", report.saved.into());
            put("tried", report.reached.describe().into());
            return out.into();
        };
        put("status", "fail".into());
        let mut case = BTreeMap::new();
        let mut field = |key: &str, value: Value| case.insert(key.to_string(), value);
        field("case", failure.case.into());
        field("seed", Value::Int(failure.seed as i64));
        let described = failure.described.as_deref();
        field(
            "input",
            described.unwrap_or(failure.input.trim_end()).into(),
        );
        field("raw_input", failure.input.as_str().into());
        field("expected", failure.expected.as_str().into());
        field("got", failure.got.as_str().into());
        field("verdict", failure.verdict.code().into());
        if let Some(trap) = &self.trap {
            field("error", trap.clone());
        }
        if self.toolchain_bug {
            field("toolchain_bug", true.into());
        }
        if let (Some(transcript), Some(ruling)) = (&failure.transcript, &failure.ruling) {
            field("interaction", interaction_json(transcript, ruling));
        }
        field(
            "stages",
            self.stages
                .iter()
                .map(stage_json)
                .collect::<Vec<_>>()
                .into(),
        );
        field("saved", optional(self.saved.as_deref()));
        field("rerun", failure.saved.into());
        field("trace", optional(self.trace.as_deref()));
        if let Some(explained) = &self.explained {
            let explain = match explained {
                Ok(divergence) => obj([(
                    "divergence",
                    divergence.as_ref().map_or(Value::Null, divergence_json),
                )]),
                Err(why) => obj([("error", why.as_str().into())]),
            };
            field("explain", explain);
        }
        put("counterexample", case.into());
        out.into()
    }

    /// [`Self::to_json`] as one line, as the CLI and the browser print it,
    /// in the key order `spec/tools/cli.md` publishes.
    pub fn render(&self) -> String {
        render_json(&self.to_json())
    }
}

/// What the two sides of an interactive case said, and the interactor's
/// ruling in its own words.
fn interaction_json(transcript: &Transcript, ruling: &Ruling) -> Value {
    let lines: Vec<Value> = (transcript.lines.iter())
        .map(|(side, line)| obj([("from", side.name().into()), ("line", line.as_str().into())]))
        .collect();
    let (verdict, message) = match ruling {
        Ruling::Accepted => ("accepted", None),
        Ruling::Rejected(said) => ("rejected", said.as_deref()),
        Ruling::Broke(trap) => ("error", Some(trap.as_str())),
    };
    obj([
        ("transcript", lines.into()),
        ("truncated", transcript.truncated.into()),
        (
            "interactor",
            obj([("verdict", verdict.into()), ("message", optional(message))]),
        ),
    ])
}

/// The published key order of `cpl stress --format json`.
const ORDER: cpl_json::KeyOrder = cpl_json::KeyOrder(&[
    (
        "",
        &[
            "status",
            "build",
            "case_count",
            "saved_count",
            "tried",
            "counterexample",
            "error",
        ],
    ),
    (
        "counterexample",
        &[
            "case",
            "seed",
            "input",
            "raw_input",
            "expected",
            "got",
            "verdict",
            "error",
            "toolchain_bug",
            "interaction",
            "stages",
            "saved",
            "rerun",
            "trace",
            "explain",
        ],
    ),
    (
        "counterexample.interaction",
        &["transcript", "truncated", "interactor"],
    ),
    ("counterexample.interaction.transcript", &["from", "line"]),
    (
        "counterexample.interaction.interactor",
        &["verdict", "message"],
    ),
    ("counterexample.stages", &["name", "line", "fast", "brute"]),
    (
        "counterexample.explain.divergence",
        &[
            "name", "nth", "call", "stuck", "fast", "brute", "inside", "blame",
        ],
    ),
]);

/// A run that stopped before a verdict, in the same schema.
pub fn error_json(error: &StressError) -> Value {
    obj([
        ("status", "error".into()),
        ("error", error.0.as_str().into()),
    ])
}

/// [`error_json`] as one line.
pub fn render_error(error: &StressError) -> String {
    render_json(&error_json(error))
}

/// A stress result's JSON as one line in its published key order.
pub fn render_json(value: &Value) -> String {
    format!("{}\n", cpl_json::to_string_ordered(value, &ORDER))
}

fn optional(text: Option<&str>) -> Value {
    text.map_or(Value::Null, Value::from)
}

fn stage_json(stage: &Stage) -> Value {
    obj([
        ("name", stage.name.as_str().into()),
        ("line", stage.line.into()),
        ("fast", stage.fast.as_str().into()),
        ("brute", stage.brute.as_str().into()),
    ])
}

/// One divergence, as `cpl stress --json` (under `explain`) and `cpl
/// locate --json` give it.
pub fn divergence_json(d: &Divergence) -> Value {
    let side = |side: &Option<(u32, String)>| {
        side.as_ref().map_or(Value::Null, |(line, value)| {
            obj([("line", (*line).into()), ("value", value.as_str().into())])
        })
    };
    let lines = |key: &str, lines: &[(u32, String)]| -> Value {
        (lines.iter())
            .map(|(line, text)| obj([("line", (*line).into()), (key, text.as_str().into())]))
            .collect::<Vec<_>>()
            .into()
    };
    let mut out = BTreeMap::new();
    let mut put = |key: &str, value: Value| out.insert(key.to_string(), value);
    put("name", d.name.as_str().into());
    put("nth", d.nth.into());
    put("call", d.call.into());
    if d.stuck {
        put("stuck", true.into());
    }
    put("fast", side(&d.fast));
    put("brute", side(&d.brute));
    if !d.inside.is_empty() {
        put("inside", lines("locals", &d.inside));
    }
    if !d.blame.is_empty() {
        put("blame", lines("why", &d.blame));
    }
    out.into()
}
