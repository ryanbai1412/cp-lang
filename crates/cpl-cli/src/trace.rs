//! `cpl run --trace`, `cpl run --format json` and the tracing half of
//! `cpl stress`: a run in the debug interpreter, recorded.

use cpl_interp::trace::{document, stages, Recording, TraceConfig};
use std::path::Path;

/// The trace viewer: one page with nothing to fetch. A trace is put in
/// its `trace-data` element.
const VIEWER: &str = include_str!("../../../tools/trace/index.html");

/// The contents of `<script id="{id}" ...>` replaced by `json`.
fn fill(page: &str, id: &str, json: &str) -> String {
    let open = format!("<script id=\"{id}\" type=\"application/json\">");
    let Some(start) = page.find(&open).map(|at| at + open.len()) else {
        return page.to_string();
    };
    let Some(end) = page[start..].find("</script>").map(|at| at + start) else {
        return page.to_string();
    };
    format!("{}{json}{}", &page[..start], &page[end..])
}

/// The viewer with one trace in it and no samples: a self-contained
/// page. Trace JSON never holds a `<`, so it cannot end the element.
pub fn html(trace_json: &str) -> String {
    fill(&fill(VIEWER, "trace-samples", "[]"), "trace-data", trace_json)
}

fn file_name(path: &str) -> &str {
    Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path)
}

/// How to record a run of `source`.
pub fn config(source: &str, only_fn: Option<&str>) -> TraceConfig {
    TraceConfig {
        source: source.to_string(),
        only_fn: only_fn.map(str::to_string),
    }
}

/// The trace document of one run of the file at `path`.
pub fn single(path: &str, source: &str, recording: &Recording) -> String {
    let name = file_name(path);
    let note = if recording.truncated {
        "The run was longer than a trace holds: the first steps are kept."
    } else {
        "Every change to a top-level variable, every print and every debug(...)."
    };
    document(name, note, name, source, ("cpl run", recording), None)
}

/// The trace document of a solution and its brute on one input.
pub fn versus(path: &str, source: &str, brute_fn: &str, fast: &Recording, slow: &Recording) -> String {
    let name = file_name(path);
    let title = format!("{name} against its brute `{brute_fn}`");
    let note = "The same input through the solution and through its @brute(main), lined up at each print and each debug(...) both ran.";
    document(&title, note, name, source, ("solution", fast), Some((brute_fn, slow)))
}

/// Runs the solution and the `@brute(main)` of the file at `path` on
/// `input` in the debug interpreter: what `cpl stress` shows of a
/// failing case. A file judged by its `@checker` alone has no brute to
/// line up with, and its trace is the solution's own run.
pub fn against_brute(path: &Path, input: &str) -> Result<cpl_stress::cli::Traced, String> {
    let name = path.display().to_string();
    let source = std::fs::read_to_string(path).map_err(|e| format!("can't read {name}: {e}"))?;
    let mut db = cpl_check::Database::new();
    db.set_file(&name, source.as_str());
    let analysis = db.analyze(&name).ok_or("the file can't be checked")?;
    let brute_fn = cpl_ir::brute(&analysis).map(|(name, _)| name);
    let first = |diagnostics: Vec<cpl_diag::Diagnostic>| {
        diagnostics.first().map_or("the file does not build".to_string(), |d| d.message.clone())
    };
    let fast = cpl_ir::lower(&analysis).map_err(first)?;
    let slow = match &brute_fn {
        Some(brute_fn) => Some(cpl_ir::lower_brute(&analysis, brute_fn).map_err(first)?),
        None => None,
    };
    let record = |program: &cpl_ir::Program| {
        let mut sink = Vec::new();
        let io = cpl_interp::Io::fixed(input.as_bytes());
        cpl_interp::run_observed(program, io, &mut sink, Some(config(&source, None)))
            .recording
            .unwrap_or_default()
    };
    // An interactive solution has no one to talk to here.
    if fast.interactive {
        return Err("an interactive solution is traced with `cpl run --trace`".to_string());
    }
    let fast = record(&fast);
    let (Some(brute_fn), Some(slow)) = (brute_fn, slow) else {
        return Ok(cpl_stress::cli::Traced {
            stages: Vec::new(),
            html: html(&single(&name, &source, &fast)),
        });
    };
    let slow = record(&slow);
    let stages = stages(&fast, &slow)
        .into_iter()
        .map(|stage| cpl_stress::cli::Stage {
            name: stage.name,
            line: stage.line,
            fast: stage.fast,
            brute: stage.brute,
        })
        .collect();
    Ok(cpl_stress::cli::Traced {
        stages,
        html: html(&versus(&name, &source, &brute_fn, &fast, &slow)),
    })
}

/// `text` as a JSON string.
pub fn json_string(text: &str) -> String {
    let mut out = String::new();
    cpl_interp::trace::string(&mut out, text);
    out
}

/// What `cpl run --format json` prints: the editor's inline values.
pub fn run_report(
    recording: &Recording,
    stdout: &[u8],
    millis: u128,
    error: Option<(u32, &str)>,
) -> String {
    let values: Vec<String> = recording
        .lives
        .iter()
        .map(|live| {
            format!(
                "{{\"line\":{},\"name\":{},\"value\":{}}}",
                live.line,
                json_string(&live.name),
                json_string(&live.value)
            )
        })
        .collect();
    let mut out = format!(
        "{{\"values\":[{}],\"stdout\":{},\"time_ms\":{millis}",
        values.join(","),
        json_string(&String::from_utf8_lossy(stdout))
    );
    if let Some((line, message)) = error {
        out += &format!(",\"error\":{{\"line\":{line},\"message\":{}}}", json_string(message));
    }
    out.push_str("}\n");
    out
}
