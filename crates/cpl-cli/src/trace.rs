//! `cpl run --trace`, `cpl run --format json` and the tracing half of
//! `cpl stress`: a run in the debug interpreter, recorded.

use cpl_facade::trace::document;
use cpl_facade::Recording;
use cpl_facade::TraceConfig;
use cpl_stress::traced::Retraced;
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

/// The longest naming a trap's locals may take: retracing the run's end
/// can cost far more than the run when its state is large, and the trap
/// is reported without them past this.
const TRAP_RETRACE: std::time::Duration = std::time::Duration::from_secs(3);

/// The innermost call's locals where `program` trapped on `input`, from a
/// second run, recorded; `None` when it has none or stops elsewhere.
pub fn trap_locals(
    program: &cpl_facade::Program,
    source: &str,
    input: &[u8],
    trap: &cpl_facade::Trap,
) -> Option<String> {
    // A run stopped from outside would not stop at the same place again.
    if [cpl_facade::OUT_OF_TIME.text(), cpl_facade::CANCELLED.text()].contains(&trap.message()) {
        return None;
    }
    // Untraced again with checkpoints, and only the end traced from the last one.
    let config = cpl_facade::trace::trap_config(source);
    let stop = cpl_facade::Stop::after(TRAP_RETRACE);
    let observed = cpl_facade::trace::resumed_tail_until(
        program,
        input,
        &mut std::io::sink(),
        config,
        1,
        stop,
    );
    (observed.trap?.message() == trap.message()).then_some(())?;
    let at = source.get(trap.span().start as usize..trap.span().end as usize);
    cpl_facade::trace::trap_locals(&observed.recording?, source, at.unwrap_or(""))
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

/// The page `cpl stress --trace` writes for a failing case: the
/// solution's run beside its brute's, or its own run alone when it has no
/// brute to line up with.
pub fn page(name: &str, source: &str, brute_fn: &str, retraced: &Retraced) -> String {
    html(&match &retraced.slow {
        Some(slow) => versus(name, source, brute_fn, &retraced.fast, slow),
        None => single(name, source, &retraced.fast),
    })
}

/// What `cpl run --format json` prints: the editor's inline values.
pub fn run_report(
    recording: &Recording,
    stdout: &[u8],
    millis: u128,
    error: Option<cpl_json::Value>,
    view: &cpl_diag::view::View,
) -> String {
    let mut out = cpl_json::write_with(|w| {
        w.object().key("values").array();
        for live in recording.lives() {
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
        view.write(w);
        w.end();
    });
    out.push('\n');
    out
}
