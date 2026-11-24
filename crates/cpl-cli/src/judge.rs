//! `cpl judge`: a file's `@interactor` or `@check` on a solution's
//! input and output, its verdict, and how a trap is rendered.

use crate::{compile, compile_side, interact, json_out, Side, EXIT_FAILED, EXIT_USAGE};
use std::io::Write;

/// `cpl judge`: the file's `@interactor`, talking to a solution on
/// standard input and output. The first line of what it says on failure
/// is the reason, which is what `cpl stress` reports.
/// How `cpl judge` reports: one JSON object on stdout with `--json`, and
/// with `--diagnostic <file>` a trap's diagnostic, as JSON, to that file,
/// which stress reads since the solution reads the interactor's stdout.
#[derive(Clone, Copy)]
pub(crate) struct Say<'a> {
    pub(crate) json: bool,
    pub(crate) diagnostic: Option<&'a str>,
}

pub(crate) fn judge_file(
    path: &str,
    input: &str,
    say: Say,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let input = match std::fs::read(input) {
        Ok(input) => input,
        Err(error) => {
            *stderr += &format!("cpl: can't read `{input}`: {error}\n");
            return EXIT_USAGE;
        }
    };
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let Some((_, judge)) = &compiled.judge else {
        *stderr += &format!("cpl: `{path}` declares no `@interactor`\n");
        return EXIT_USAGE;
    };
    let result = interact::judge_alone(judge, &input);
    verdict(path, &compiled.text, result, say, stdout, stderr)
}

/// `cpl judge <file> <input> <output>`: the file's `@check` on what a
/// solution printed. The first line it says on failure is the reason.
pub(crate) fn check_output(
    path: &str,
    input: &str,
    output: &str,
    say: Say,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let read = |file: &str, stderr: &mut String| match std::fs::read(file) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            *stderr += &format!("cpl: can't read `{file}`: {error}\n");
            Err(EXIT_USAGE)
        }
    };
    let (input, output) = match (read(input, stderr), read(output, stderr)) {
        (Ok(input), Ok(output)) => (input, output),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let compiled = match compile_side(path, Side::Checker, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let mut printed = Some(output);
    let io = cpl_interp::Io::judge(&input, Box::new(move || printed.take()));
    let mut sink = Vec::new();
    let result = cpl_interp::run_with(&compiled.program, io, &mut sink);
    verdict(path, &compiled.text, result, say, stdout, stderr)
}

/// What `cpl judge` says of the judge's verdict: nothing when it accepts,
/// else the reason and where it failed; with `--json`, one object with
/// `accepted`, `reason` and `diagnostic`.
fn verdict(
    path: &str,
    text: &str,
    result: Result<(), cpl_interp::Trap>,
    Say { json, diagnostic }: Say,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let code = if result.is_ok() { 0 } else { EXIT_FAILED };
    if json {
        let lines = cpl_diag::LineIndex::new(text);
        let (reason, diagnostic) = match &result {
            Ok(()) => (cpl_json::Value::Null, cpl_json::Value::Null),
            Err(trap) => (
                trap.message().into(),
                json_out::diagnostic(&trap.to_diagnostic(path, &lines), &lines),
            ),
        };
        let report = cpl_json::obj([
            ("accepted", (code == 0).into()),
            ("reason", reason),
            ("diagnostic", diagnostic),
        ]);
        let _ = writeln!(stdout, "{}", cpl_json::to_string(&report));
    } else if let Err(trap) = &result {
        *stderr += &format!("{}\n", trap.message());
        *stderr += &render_trap(path, text, trap);
        if let Some(file) = diagnostic {
            let lines = cpl_diag::LineIndex::new(text);
            let json = cpl_diag::json::diagnostic(&trap.to_diagnostic(path, &lines), &lines);
            let _ = std::fs::write(file, json.to_string());
        }
    }
    code
}

pub(crate) fn render_trap(path: &str, text: &str, trap: &cpl_interp::Trap) -> String {
    render_trap_noting(path, text, trap, None)
}

/// The trap's diagnostic, with a note naming `locals` when there are some.
pub(crate) fn render_trap_noting(
    path: &str,
    text: &str,
    trap: &cpl_interp::Trap,
    locals: Option<String>,
) -> String {
    let lines = cpl_diag::LineIndex::new(text);
    let mut diagnostic = trap.to_diagnostic(path, &lines);
    if let Some(locals) = locals {
        diagnostic = diagnostic.with_note(format!("locals: {locals}"));
    }
    cpl_diag::render(&diagnostic, path, text, &lines)
}
