//! `cpl run --trace`, `cpl run --format json` and the tracing half of
//! `cpl stress`: a run in the debug interpreter, recorded.

use cpl_diag::Shown;
use cpl_interp::trace::{document, Recording, TraceConfig};
use cpl_stress::judge::Explaining;
use cpl_stress::traced::{retrace, Programs};
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

/// The trace a page made by [`html`] holds, or `page` itself when it is
/// the trace JSON.
pub fn data(page: &str) -> &str {
    let open = "<script id=\"trace-data\" type=\"application/json\">";
    let Some(start) = page.find(open).map(|at| at + open.len()) else {
        return page;
    };
    page[start..]
        .find("</script>")
        .map_or(page, |end| &page[start..start + end])
}

/// The viewer with one trace in it, no samples and this build's version:
/// a self-contained page. Trace JSON never holds a `<`, so it cannot end
/// the element.
pub fn html(trace_json: &str) -> String {
    let page = fill(VIEWER, "trace-samples", "[]");
    let page = fill(&page, "trace-version", &format!("\"{}\"", crate::VERSION));
    fill(&page, "trace-data", trace_json)
}

/// The innermost call's locals where `program` trapped on `input`, from a
/// second run, recorded; `None` when it has none or stops elsewhere.
pub fn trap_locals(
    program: &cpl_ir::Program,
    source: &str,
    input: &[u8],
    trap: &cpl_interp::Trap,
) -> Option<String> {
    // A run stopped from outside would not stop at the same place again.
    if [cpl_interp::OUT_OF_TIME, cpl_interp::CANCELLED].contains(&trap.message.as_str()) {
        return None;
    }
    // Untraced again with checkpoints, and only the end traced from the last one.
    let config = cpl_interp::trace::trap_config(source);
    let observed = cpl_interp::trace::resumed_tail(program, input, &mut std::io::sink(), config, 1);
    (observed.trap?.message == trap.message).then_some(())?;
    let at = source.get(trap.span.start as usize..trap.span.end as usize);
    cpl_interp::trace::trap_locals(&observed.recording?, source, at.unwrap_or(""))
}

fn file_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
}

/// The newest `.in` in the file's `<name>.tests/`: the failing case
/// `cpl stress` saved last, or a judge case put there.
pub fn pinned_input(path: &Path) -> Option<std::path::PathBuf> {
    let dir = path.with_extension("tests");
    let modified = |p: &Path| p.metadata().and_then(|m| m.modified()).ok();
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "in"))
        .max_by_key(|p| (modified(p), p.clone()))
}

/// How to record a run of `source`.
pub fn config(source: &str, only_fn: Option<&str>) -> TraceConfig {
    TraceConfig {
        source: source.to_string(),
        only_fn: only_fn.map(str::to_string),
        record_frames: true,
        ..TraceConfig::default()
    }
}

/// The trace document of one run of the file at `path`.
pub fn single(path: &str, source: &str, recording: &Recording) -> String {
    let name = file_name(path);
    let note = recording.note();
    document(name, note, name, source, ("cpl run", recording), None)
}

/// The trace document of a solution and its brute on one input.
pub fn versus(
    path: &str,
    source: &str,
    brute_fn: &str,
    fast: &Recording,
    slow: &Recording,
) -> String {
    let name = file_name(path);
    let title = format!("{name} against its brute `{brute_fn}`");
    let note = "The same input through the solution and through its brute, lined up at each print and each debug(...) both ran.";
    document(
        &title,
        note,
        name,
        source,
        ("solution", fast),
        Some((brute_fn, slow)),
    )
}

/// Runs the solution and the `@alt(program)` of the file at `path` on
/// `input` in the debug interpreter: what `cpl stress` shows of a
/// failing case. A file judged by its `@check` alone has no brute to
/// line up with, and its trace is the solution's own run. With `explain`,
/// every value each run gives a variable is logged too, for the first one
/// they give differently.
pub fn against_brute(
    path: &Path,
    input: &str,
    explain: bool,
    how: Option<&Result<Explaining, String>>,
) -> Result<cpl_stress::cli::Traced, String> {
    let name = path.shown();
    let source = std::fs::read_to_string(path).map_err(|e| format!("can't read {name}: {e}"))?;
    let mut db = cpl_check::Session::new();
    db.set_file(&name, source.as_str());
    let analysis = db.analyze(&name).ok_or("the file can't be checked")?;
    let first = |diagnostics: Vec<cpl_diag::Diagnostic>| {
        diagnostics
            .first()
            .map_or("the file does not build".to_string(), |d| d.message.clone())
    };
    let fast = cpl_ir::lower(&analysis).map_err(first)?;
    // The brute is the file's `@alt(program)` fn, else the file with each
    // `@alt(f)` fn in `f`'s place; its lines stay where they were.
    let (brute_fn, slow) = match cpl_ir::brute(&analysis) {
        Some((brute_fn, _)) => {
            let slow = cpl_ir::lower_brute(&analysis, &brute_fn).map_err(first)?;
            (brute_fn, Some(slow))
        }
        None => match cpl_stress::cli::swap_in_part_brutes(&source) {
            Some(swapped) => {
                let mut swapped_db = cpl_check::Session::new();
                swapped_db.set_file(&name, swapped.as_str());
                let analysis = (swapped_db.analyze(&name))
                    .ok_or("the file with its `@alt` fns swapped in can't be checked")?;
                let slow = cpl_ir::lower(&analysis).map_err(first)?;
                ("its @alt fns".to_string(), Some(slow))
            }
            None => (String::new(), None),
        },
    };
    // An interactive solution talks to the file's `@interactor`.
    let interactor = match fast.interactive {
        true => {
            let judge = cpl_ir::interactor(&analysis).ok_or(
                "an interactive solution judged by another program is traced with `cpl run --trace`",
            )?;
            Some(cpl_ir::lower_judge(&analysis, &judge).map_err(first)?)
        }
        false => None,
    };
    let programs = Programs {
        fast: &fast,
        brute: slow.as_ref(),
        interactor: interactor.as_ref(),
    };
    let retraced = retrace(&source, programs, input, how, explain, None);
    let html = html(&match &retraced.slow {
        Some(slow) => versus(&name, &source, &brute_fn, &retraced.fast, slow),
        None => single(&name, &source, &retraced.fast),
    });
    Ok(cpl_stress::cli::Traced {
        stages: retraced.stages,
        divergence: retraced.divergence?,
        html,
    })
}

/// What `cpl run --format json` prints: the editor's inline values.
pub fn run_report(
    recording: &Recording,
    stdout: &[u8],
    millis: u128,
    error: Option<cpl_json::Value>,
) -> String {
    let mut out = cpl_json::write_with(|w| {
        w.object().key("values").array();
        for live in &recording.lives {
            w.object().key("line").int(live.line);
            w.key("name")
                .str(&live.name)
                .key("value")
                .str(&live.value)
                .end();
        }
        w.end().key("stdout").str(&String::from_utf8_lossy(stdout));
        w.key("time_ms").int(millis);
        if let Some(error) = &error {
            w.key("error").value(error);
        }
        w.end();
    });
    out.push('\n');
    out
}
