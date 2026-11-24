//! What a stress run found, as one typed value both `cpl stress
//! --format json` and the browser serialize through `cpl_json`.

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

    /// [`Self::to_json`] as one line, as the CLI and the browser print it.
    pub fn render(&self) -> String {
        format!("{}\n", cpl_json::to_string(&self.to_json()))
    }
}

/// A run that stopped before a verdict, in the same schema.
pub fn error_json(error: &StressError) -> Value {
    obj([
        ("status", "error".into()),
        ("error", error.0.as_str().into()),
    ])
}

/// [`error_json`] as one line.
pub fn render_error(error: &StressError) -> String {
    format!("{}\n", cpl_json::to_string(&error_json(error)))
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
