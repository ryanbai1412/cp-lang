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
    trap_report(path, input, false).map(|trapped| trapped.text())
}

/// Where a debug run stopped: the trap as a diagnostic on the file.
pub struct Trapped {
    diagnostic: cpl_diag::Diagnostic,
    name: String,
    text: String,
}

impl Trapped {
    fn new(diagnostic: cpl_diag::Diagnostic, name: &str, text: &str) -> Trapped {
        Trapped {
            diagnostic,
            name: name.to_string(),
            text: text.to_string(),
        }
    }

    /// As `cpl run` prints it.
    pub fn text(&self) -> String {
        let lines = cpl_diag::LineIndex::new(&self.text);
        cpl_diag::render(&self.diagnostic, &self.name, &self.text, &lines)
    }

    /// As `cpl check --format json` gives it.
    pub fn json(&self) -> cpl_json::Value {
        cpl_diag::json::diagnostic(&self.diagnostic, &cpl_diag::LineIndex::new(&self.text)).into()
    }
}

/// Where a debug run of the file's solution, or with `brute` of its own
/// `@alt(program)` fn, stops on `input`: the trap with its locals.
pub fn trap_report(path: &Path, input: &str, brute: bool) -> Option<Trapped> {
    let text = std::fs::read_to_string(path).ok()?;
    trap_report_in(
        &cpl_facade::Session::default(),
        path,
        &text,
        input,
        brute,
        None,
    )
}

/// [`trap_report`] of `text` in place of the file's own source, such as the
/// file with its `@alt(f)` fns swapped in, whose lines stay where they were.
/// With `stack_limit`, it runs on the calling thread (wasm), without the
/// locals, which take a second run on a thread of its own.
pub fn trap_report_in(
    session: &cpl_facade::Session,
    path: &Path,
    text: &str,
    input: &str,
    brute: bool,
    stack_limit: Option<u32>,
) -> Option<Trapped> {
    let (name, db) = opened(session, path, text)?;
    let text = text.to_string();
    let job = match brute {
        false => cpl_facade::Job::program(),
        true => cpl_facade::Job::of(cpl_facade::Root::Alt(db.roles(&name)?.alt()?.0)),
    };
    let program = db.program(&name, &job).ok()?;
    // Alone, an interactive solution only ever sees its input end.
    if program.interactive() {
        return None;
    }
    let mut sink = Vec::new();
    let io = cpl_facade::RunInput::fixed(input.as_bytes()).until(cpl_facade::Stop::after(LIMIT));
    let trap = match stack_limit {
        Some(bytes) => {
            let limits = cpl_facade::Limits {
                stack_bytes: bytes as usize,
            };
            cpl_facade::run_within(&program, io, &mut sink, limits).err()?
        }
        None => cpl_facade::run_to_end(&program, io, &mut sink).err()?,
    };
    if trap.message() == cpl_facade::OUT_OF_TIME.text() {
        return None;
    }
    let lines = cpl_diag::LineIndex::new(&text);
    let mut diagnostic = trap.to_diagnostic(&name, &lines);
    // The locals where it stopped, as `cpl run` names them.
    if stack_limit.is_some() {
        return Some(Trapped::new(diagnostic, &name, &text));
    }
    let config = cpl_facade::trace::trap_config(&text);
    let tail = cpl_facade::trace::resumed_tail(
        &program,
        input.as_bytes(),
        &mut std::io::sink(),
        config,
        1,
    );
    let at = text
        .get(trap.span().start as usize..trap.span().end as usize)
        .unwrap_or("");
    let locals = match (tail.trap, tail.recording) {
        (Some(again), Some(recording)) if again.message() == trap.message() => {
            cpl_facade::trace::trap_locals(&recording, &text, at)
        }
        _ => None,
    };
    if let Some(locals) = locals {
        diagnostic = diagnostic.with_note(format!("locals: {locals}"));
    }
    Some(Trapped::new(diagnostic, &name, &text))
}

/// The trap the file's `@check` fn stops at, judging `got` for `input`,
/// `None` when it has no
/// `@check` fn, or the fn accepts or rejects the output without a trap.
pub fn checker_trap_in(
    session: &cpl_facade::Session,
    path: &Path,
    text: &str,
    input: &str,
    got: &str,
    stack_limit: Option<u32>,
) -> Option<Trapped> {
    let (name, db) = opened(session, path, text)?;
    let (checker, _) = db.roles(&name)?.checker()?;
    let job = cpl_facade::Job::of(cpl_facade::Root::Checker(checker.to_string()));
    let program = db.program(&name, &job).ok()?;
    let mut printed = Some(got.as_bytes().to_vec());
    let io = cpl_facade::RunInput::judge(input.as_bytes(), Box::new(move || printed.take()))
        .until(cpl_facade::Stop::after(LIMIT));
    let mut sink = Vec::new();
    let trap = match stack_limit {
        Some(bytes) => {
            let limits = cpl_facade::Limits {
                stack_bytes: bytes as usize,
            };
            cpl_facade::run_within(&program, io, &mut sink, limits).err()?
        }
        None => cpl_facade::run_to_end(&program, io, &mut sink).err()?,
    };
    if trap.message() == cpl_facade::OUT_OF_TIME.text() {
        return None;
    }
    let lines = cpl_diag::LineIndex::new(text);
    let diagnostic = trap.to_diagnostic(&name, &lines);
    Some(Trapped::new(diagnostic, &name, text))
}

/// What the debug interpreter prints running the `.cp` file at `path` on
/// `input`. `None` when the file is not CP Lang, does not build, is
/// interactive, traps or is still running after [`LIMIT`].
pub fn interpreted_output(path: &Path, input: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let (name, db) = opened(&cpl_facade::Session::default(), path, &text)?;
    let program = db.program(&name, &cpl_facade::Job::program()).ok()?;
    if program.interactive() {
        return None;
    }
    let mut sink = Vec::new();
    let io = cpl_facade::RunInput::fixed(input.as_bytes()).until(cpl_facade::Stop::after(LIMIT));
    cpl_facade::run_to_end(&program, io, &mut sink).ok()?;
    String::from_utf8(sink).ok()
}

/// `session` with the `.cp` file at `path`, read as `text`, under its shown
/// name; `None` when it is not CP Lang.
fn opened(
    session: &cpl_facade::Session,
    path: &Path,
    text: &str,
) -> Option<(String, cpl_facade::Session)> {
    if path.extension().is_none_or(|ext| ext != "cp") {
        return None;
    }
    let name = path.shown();
    let db = session.with_file(&name, text);
    Some((name, db))
}
