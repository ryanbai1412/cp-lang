//! Why a solution failed at run time, in the words of `cpl run`.
//!
//! The built C++ stops with a signal or an exit code and no message. Run
//! on the same input in the debug interpreter, a `.cp` solution says what
//! went wrong and where, as an `R0001` with the source line. A wrong
//! answer can be a trap too: an `Int` overflow wraps in the C++ and stops
//! the interpreter at the line that overflowed.

use cpl_diag::Shown;
use std::path::Path;

/// The runtime error the `.cp` file at `path` stops with on `input`, as
/// `cpl run` would print it. `None` when the file is not CP Lang, does not
/// build, runs cleanly in the interpreter, or is still running after
/// [`LIMIT`].
/// How long the interpreter runs a failing case before giving up on it.
const LIMIT: std::time::Duration = std::time::Duration::from_secs(10);

pub fn runtime_error(path: &Path, input: &str) -> Option<String> {
    trap_report(path, input, false, false)
}

/// Where a debug run of the file's solution, or with `brute` of its own
/// `@alt(program)` fn, stops on `input`: the trap with its locals, rendered
/// as `cpl run` prints it or, with `json`, as `cpl check --format json` does.
pub fn trap_report(path: &Path, input: &str, brute: bool, json: bool) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    trap_report_in(path, &text, input, brute, json, None)
}

/// [`trap_report`] of `text` in place of the file's own source, such as the
/// file with its `@brute(f)` fns swapped in, whose lines stay where they were.
/// With `stack_limit`, it runs on the calling thread (wasm), without the
/// locals, which take a second run on a thread of its own.
pub fn trap_report_in(
    path: &Path,
    text: &str,
    input: &str,
    brute: bool,
    json: bool,
    stack_limit: Option<u32>,
) -> Option<String> {
    if path.extension().is_none_or(|ext| ext != "cp") {
        return None;
    }
    let name = path.shown();
    let text = text.to_string();
    let mut db = cpl_check::Database::new();
    db.set_file(&name, text.as_str());
    let analysis = db.analyze(&name)?;
    if analysis
        .diagnostics
        .iter()
        .any(|d| d.severity == cpl_diag::Severity::Error)
    {
        return None;
    }
    let program = match brute {
        false => cpl_ir::lower(&analysis).ok()?,
        true => cpl_ir::lower_brute(&analysis, &cpl_ir::brute(&analysis)?.0).ok()?,
    };
    // Alone, an interactive solution only ever sees its input end.
    if program.interactive {
        return None;
    }
    let mut sink = Vec::new();
    let io = cpl_interp::Io::fixed(input.as_bytes()).until(cpl_interp::Stop::after(LIMIT));
    let trap = match stack_limit {
        Some(bytes) => {
            let limits = cpl_interp::Limits {
                stack_bytes: bytes as usize,
            };
            cpl_interp::run_with_on_this_thread(&program, io, &mut sink, limits).err()?
        }
        None => cpl_interp::run_with(&program, io, &mut sink).err()?,
    };
    if trap.message == cpl_interp::OUT_OF_TIME {
        return None;
    }
    let lines = cpl_diag::LineIndex::new(&text);
    let mut diagnostic = trap.to_diagnostic(&name, &lines);
    // The locals where it stopped, as `cpl run` names them.
    if stack_limit.is_some() {
        return Some(render(&diagnostic, &name, &text, &lines, json));
    }
    let config = cpl_interp::trace::trap_config(&text);
    let tail = cpl_interp::trace::resumed_tail(
        &program,
        input.as_bytes(),
        &mut std::io::sink(),
        config,
        1,
    );
    let at = text
        .get(trap.span.start as usize..trap.span.end as usize)
        .unwrap_or("");
    let locals = match (tail.trap, tail.recording) {
        (Some(again), Some(recording)) if again.message == trap.message => {
            cpl_interp::trace::trap_locals(&recording, &text, at)
        }
        _ => None,
    };
    if let Some(locals) = locals {
        diagnostic = diagnostic.with_note(format!("locals: {locals}"));
    }
    Some(render(&diagnostic, &name, &text, &lines, json))
}

fn render(
    diagnostic: &cpl_diag::Diagnostic,
    name: &str,
    text: &str,
    lines: &cpl_diag::LineIndex,
    json: bool,
) -> String {
    match json {
        true => cpl_diag::json::diagnostic(diagnostic, lines).to_string(),
        false => cpl_diag::render(diagnostic, name, text, lines),
    }
}
