//! `--json` output: what tools read instead of the text a person reads.

use cpl_diag::LineIndex;
use cpl_json::{obj, Value};

pub(crate) use cpl_diag::json::diagnostic;

fn text(s: &str) -> Value {
    Value::String(s.to_string())
}

/// `cpl lint`, with `--json` printing the `files` shape `cpl check --json` prints.
pub(crate) fn lint(
    args: &[String],
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> u8 {
    cpl_lint::cli::run_with(
        args,
        stdout,
        stderr,
        Some(&lint_files),
        Some(&crate::check::reported_errors),
    )
}

/// `cpl fmt`, with `--json` printing the shared diagnostic shape.
pub(crate) fn fmt(
    args: &[String],
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> u8 {
    cpl_fmt::cli::run_with(args, stdout, stderr, Some(&fmt_files))
}

fn fmt_files(files: &[cpl_fmt::cli::FileReport]) -> String {
    let files = files.iter().map(|f| {
        let lines = LineIndex::new(&f.src);
        let diagnostics = f.diagnostics.iter().map(|d| diagnostic(d, &lines));
        obj([
            ("path", text(&f.path)),
            ("changed", Value::Bool(f.changed)),
            ("ok", Value::Bool(f.ok)),
            ("diagnostics", Value::Array(diagnostics.collect())),
            ("error", f.error.as_deref().map_or(Value::Null, text)),
        ])
    });
    cpl_json::to_string(&obj([("files", Value::Array(files.collect()))]))
}

fn lint_files(files: &[cpl_lint::cli::FileReport]) -> String {
    let files = files.iter().map(|f| {
        let lines = LineIndex::new(&f.src);
        let diagnostics = f.diagnostics.iter().map(|d| diagnostic(d, &lines));
        obj([
            ("path", text(&f.path)),
            ("ok", Value::Bool(f.ok)),
            ("diagnostics", Value::Array(diagnostics.collect())),
        ])
    });
    cpl_json::to_string(&obj([("files", Value::Array(files.collect()))]))
}
