//! `cpl run --trace`, `cpl run --format json` and the tracing half of
//! `cpl stress`: a run in the debug interpreter, recorded.

use cpl_diag::Shown;
use cpl_interp::trace::{document, first_divergence, stages, Divergence, Recording, TraceConfig};
use std::collections::HashMap;
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
    let note = if recording.skipped > 0 {
        "A window of the run from a later step: under `--live`, Earlier steps and Later steps load the rest."
    } else if recording.truncated {
        "The run was longer than a trace holds: the first steps are kept."
    } else {
        "Every change to a top-level variable, every print and every debug(...)."
    };
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
) -> Result<cpl_stress::cli::Traced, String> {
    let name = path.shown();
    let source = std::fs::read_to_string(path).map_err(|e| format!("can't read {name}: {e}"))?;
    let mut db = cpl_check::Database::new();
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
            (Some(brute_fn), Some(slow))
        }
        None => match cpl_stress::cli::swap_in_part_brutes(&source) {
            Some(swapped) => {
                let mut swapped_db = cpl_check::Database::new();
                swapped_db.set_file(&name, swapped.as_str());
                let analysis = (swapped_db.analyze(&name))
                    .ok_or("the file with its `@alt` fns swapped in can't be checked")?;
                let slow = cpl_ir::lower(&analysis).map_err(first)?;
                (Some("its @alt fns".to_string()), Some(slow))
            }
            None => (None, None),
        },
    };
    let record = |program: &cpl_ir::Program| {
        let mut sink = Vec::new();
        let io = cpl_interp::Io::fixed(input.as_bytes())
            .until(cpl_interp::Stop::after(std::time::Duration::from_secs(10)));
        let config = cpl_interp::trace::TraceConfig {
            assignments: explain,
            ..config(&source, None)
        };
        cpl_interp::run_observed(program, io, &mut sink, Some(config))
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
            divergence: None,
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
        // The wrong call of a part needs no `explain`: calls are always kept.
        divergence: first_call_divergence(&source, &fast, &slow)
            .or_else(|| explain.then(|| first_divergence(&fast, &slow)).flatten()),
        html: html(&versus(&name, &source, &brute_fn, &fast, &slow)),
    })
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
                cpl_json::escape_script_safe_string(&live.name),
                cpl_json::escape_script_safe_string(&live.value)
            )
        })
        .collect();
    let mut out = format!(
        "{{\"values\":[{}],\"stdout\":{},\"time_ms\":{millis}",
        values.join(","),
        cpl_json::escape_script_safe_string(&String::from_utf8_lossy(stdout))
    );
    if let Some((line, message)) = error {
        out += &format!(
            ",\"error\":{{\"line\":{line},\"message\":{}}}",
            cpl_json::escape_script_safe_string(message)
        );
    }
    out.push_str("}\n");
    out
}

/// With `@alt(f)` fns swapped in, the call of a part `f` that first
/// returns something else than its alt does on the same arguments, with
/// every part it calls right: in a recursion, the innermost wrong call.
fn first_call_divergence(source: &str, fast: &Recording, slow: &Recording) -> Option<Divergence> {
    let alts = cpl_stress::cli::part_alts(source);
    let line_of = |name: &str| {
        let head = format!("fn {name}(");
        (source
            .lines()
            .position(|line| line.trim_start().starts_with(&head)))
        .map_or(0, |at| at as u32 + 1)
    };
    let wrong: Vec<_> = (fast.calls.iter().enumerate())
        .filter_map(|(at, ours)| {
            let (_, alt) = alts.iter().find(|(part, _)| *part == ours.name)?;
            let theirs = (slow.calls.iter())
                .find(|call| call.name == ours.name && call.args == ours.args)?;
            match (&ours.returned, &theirs.returned) {
                (Some(a), Some(b)) if a != b => Some((at, ours, alt, a, b)),
                _ => None,
            }
        })
        .collect();
    // The calls made inside a call follow it, deeper than it: they end
    // where a call no deeper starts.
    let mut ends = vec![fast.calls.len(); fast.calls.len()];
    let mut open: Vec<usize> = Vec::new();
    for (at, call) in fast.calls.iter().enumerate() {
        while let Some(&o) = open.last().filter(|&&o| fast.calls[o].depth >= call.depth) {
            ends[o] = at;
            open.pop();
        }
        open.push(at);
    }
    let &(at, ours, alt, a, b) = wrong.iter().enumerate().find_map(|(k, entry)| {
        let next_wrong = wrong.get(k + 1).map_or(usize::MAX, |(other, ..)| *other);
        (next_wrong >= ends[entry.0]).then_some(entry)
    })?;
    let nested: Vec<_> = (fast.calls[at + 1..ends[at]].iter())
        .map(|call| call.entry..call.exit)
        .collect();
    let mut inside: Vec<(u32, String)> = Vec::new();
    for step in (ours.entry..ours.exit).rev() {
        if inside.len() == 6 {
            break;
        }
        if nested.iter().any(|range| range.contains(&step)) {
            continue;
        }
        let Some(step) = fast.step(step) else {
            continue;
        };
        let shown = (step.line, shown_args(&step.locals));
        if inside.last() != Some(&shown) {
            inside.push(shown);
        }
    }
    inside.reverse();
    let own: Vec<_> = (ours.entry..ours.exit)
        .filter(|step| !nested.iter().any(|range| range.contains(step)))
        .filter_map(|step| fast.step(step))
        .collect();
    let mut blame = leaked(source, fast, ours);
    blame.extend(made(source, fast, ours, &own));
    Some(Divergence {
        inside,
        blame,
        call: true,
        name: format!("{}({})", ours.name, shown_args(&ours.args)),
        nth: ours.ordinal,
        fast: Some((line_of(&ours.name), a.clone())),
        brute: Some((line_of(alt), b.clone())),
    })
}

/// Whether `name` is a word of `text`.
fn mentions(text: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(name)
        .any(|(at, _)| !text[..at].ends_with(word) && !text[at + name.len()..].starts_with(word))
}

/// A top-level `var` declared outside every fn that a later call of a fn
/// starts with another value than its 1st call did: one test left it for
/// the next. Each such variable's declaration, with both values.
fn leaked(source: &str, rec: &Recording, call: &cpl_interp::trace::Call) -> Vec<(u32, String)> {
    let Some(first) = (rec.calls.iter()).find(|c| c.name == call.name && c.ordinal == 1) else {
        return Vec::new();
    };
    if call.ordinal == 1 {
        return Vec::new();
    }
    let (Some(then), Some(now)) = (rec.step(first.entry), rec.step(call.entry)) else {
        return Vec::new();
    };
    let lines: Vec<&str> = source.lines().collect();
    let Some(body) = body_of(&lines, &call.name) else {
        return Vec::new();
    };
    let body = lines[body].join("\n");
    let mut out = Vec::new();
    for (var, value) in &now.values {
        let name = &rec.vars[*var].name;
        let before = then.values.iter().find(|(v, _)| v == var).map(|(_, v)| v);
        let declared = (lines.iter()).position(|line| {
            line.strip_prefix("var ")
                .is_some_and(|rest| rest.starts_with(name.as_str()) && mentions(rest, name))
        });
        if let (Some(before), Some(at)) = (before, declared) {
            if before != value && mentions(&body, name) {
                out.push((
                    at as u32 + 1,
                    format!(
                        "`{name}` is declared once for the whole run: this call of `{}` starts with `{name}` = {value}, its 1st call with {before}",
                        call.name
                    ),
                ));
            }
        }
    }
    out
}

/// The lines of the body of `fn name`, 0-based, after its head.
fn body_of(lines: &[&str], name: &str) -> Option<std::ops::Range<usize>> {
    let head = format!("fn {name}(");
    let at = lines.iter().position(|line| line.starts_with(&head))?;
    let end = (at + 1..lines.len())
        .find(|&k| !lines[k].is_empty() && !lines[k].starts_with(' '))
        .unwrap_or(lines.len());
    let end = (at + 1..end)
        .rev()
        .find(|&k| !lines[k].trim().is_empty())
        .map_or(at + 1, |k| k + 1);
    Some(at..end)
}

/// Every variable a step shows, top-level and local, by name, as JSON.
fn named_values(rec: &Recording, step: &cpl_interp::trace::Step) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = (step.values.iter())
        .map(|(var, value)| (rec.vars[*var].name.clone(), value.to_string()))
        .collect();
    if let Ok(cpl_json::Value::Object(locals)) = cpl_json::parse(&step.locals) {
        for (name, value) in locals.iter() {
            out.insert(name.to_string(), cpl_json::to_string(value));
        }
    }
    out
}

/// Where a wrong call made its value: the `return` it ran, else the last
/// line of its body, and the lines that last changed each variable that
/// line reads.
fn made(
    source: &str,
    rec: &Recording,
    call: &cpl_interp::trace::Call,
    own: &[cpl_interp::trace::Step],
) -> Vec<(u32, String)> {
    let Some(last) = own.last() else {
        return Vec::new();
    };
    let lines: Vec<&str> = source.lines().collect();
    let text_of = |line: u32| lines.get(line as usize - 1).copied().unwrap_or("");
    let returned = mentions(text_of(last.line), "return");
    let line = match body_of(&lines, &call.name) {
        Some(body) if !returned && body.len() > 1 => body.end as u32,
        _ => last.line,
    };
    let text = text_of(line);
    let mut out = vec![(line, format!("`{}` makes its value here", call.name))];
    let before = (call.entry.checked_sub(1))
        .and_then(|step| rec.step(step))
        .map(|step| named_values(rec, &step))
        .unwrap_or_default();
    let steps: Vec<_> = own
        .iter()
        .map(|step| (step.line, named_values(rec, step)))
        .collect();
    let mut names: Vec<&String> = (steps.last().map(|(_, values)| values.keys()))
        .into_iter()
        .flatten()
        .filter(|name| mentions(text, name))
        .collect();
    names.sort();
    let mut changed = Vec::new();
    for name in names {
        let mut prev = before.get(name);
        let mut at = None;
        for (line, values) in &steps {
            let now = values.get(name);
            if let Some(value) = now.filter(|_| now != prev) {
                at = Some((*line, value.clone()));
            }
            prev = now;
        }
        if let Some((at, value)) = at.filter(|(at, _)| *at != line) {
            changed.push((at, format!("`{name}` last changed here, to {value}")));
        }
    }
    changed.sort();
    out.extend(changed.into_iter().take(3));
    out
}

/// A call's arguments, JSON by name, as `k: 1, a: [2, 1]`.
fn shown_args(json: &str) -> String {
    match cpl_json::parse(json) {
        Ok(cpl_json::Value::Object(args)) => (args.iter())
            .map(|(name, value)| format!("{name}: {}", cpl_json::to_string(value)))
            .collect::<Vec<_>>()
            .join(", "),
        _ => json.to_string(),
    }
}
